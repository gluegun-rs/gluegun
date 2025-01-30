use std::{collections::BTreeMap, sync::Arc};

use syn::spanned::Spanned;

use crate::{
    Enum, Error, Field, Function, FunctionInput, FunctionOutput, IsAsync, Item, Method, MethodCategory, Name, OwnedKind, QualifiedName, Record, RefdTy, Resource, SelfKind, Signature, Span, Ty, TypeKind, Variant, VariantArm
};

use super::{
    known_rust::{
        KnownRustFn, KnownRustType, RustPath, KNOWN_RUST_IMPL_TRAIT_TYPES, KNOWN_RUST_TYPES,
    },
    modifier::Modifier,
    util, Definition, DefinitionKind, SourcePath,
};

pub(super) struct Elaborator<'arena> {
    source: Option<SourcePath>,
    module_qname: QualifiedName,
    recognized: Arc<BTreeMap<QualifiedName, Definition<'arena>>>,
    out_items: BTreeMap<QualifiedName, Item>,
}

impl<'arena> Elaborator<'arena> {
    pub(super) fn new(recognized: Arc<BTreeMap<QualifiedName, Definition<'arena>>>) -> Self {
        Self {
            recognized,
            source: None,
            module_qname: QualifiedName::new(vec![]),
            out_items: BTreeMap::new(),
        }
    }

    /// Access the source for the current definition;
    /// should only be used when processing a definition (which is almost always)
    fn source(&self) -> &SourcePath {
        self.source.as_ref().unwrap()
    }

    fn error(&self, variant: fn(Span) -> Error, spanned: impl Spanned) -> Error {
        variant(self.source().span(spanned))
    }

    pub(super) fn into_elaborated_items(mut self) -> crate::Result<BTreeMap<QualifiedName, Item>> {
        let recognized = self.recognized.clone();
        for (qname, definition) in recognized.iter() {
            self.source = Some(definition.source.clone());
            self.module_qname.set_to_module_of(qname);

            // Convert the input definition and produce the output definition.
            if let Some(item) = self.elaborate_definition(qname, definition)? {
                self.out_items.insert(qname.clone(), item);
            }

            self.source = None;
            self.module_qname.clear();
        }
        Ok(self.out_items)
    }

    fn elaborate_definition(
        &mut self,
        qname: &QualifiedName,
        definition: &Definition<'arena>,
    ) -> crate::Result<Option<Item>> {
        match &definition.kind {
            DefinitionKind::Record(item) => Ok(Some(Item::Record(
                self.elaborate_record(qname, definition, item)?,
            ))),
            DefinitionKind::Resource(item) => Ok(Some(Item::Resource(
                self.elaborate_resource(qname, definition, item)?,
            ))),
            DefinitionKind::Variant(item, variants) => Ok(Some(Item::Variant(
                self.elaborate_variant(qname, definition, item, variants)?,
            ))),
            DefinitionKind::Enum(item, variants) => Ok(Some(Item::Enum(
                self.elaborate_enum(qname, definition, item, variants)?,
            ))),
            DefinitionKind::Function(item_fn) => Ok(Some(Item::Function(
                self.elaborate_function(qname, definition, item_fn)?,
            ))),
            DefinitionKind::FileModule => {
                // We don't do model modules explicitly in the output, they are inferred by the set of public definitions.
                Ok(None)
            }
        }
    }

    fn elaborate_record(
        &mut self,
        qname: &QualifiedName,
        definition: &Definition<'arena>,
        item: &syn::ItemStruct,
    ) -> crate::Result<Record> {
        if item.generics.params.len() > 0 {
            return Err(self.error(Error::GenericsNotPermitted, &item.generics));
        }

        let span = self.source().span(&item.ident);
        let self_ty = Ty::user(span.clone(), qname);
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;

        Ok(Record {
            span,
            name: qname.tail_name(),
            fields: self.elaborate_record_fields(&self_ty, item)?,
            methods,
        })
    }

    /// Recognize fields for a record.
    fn elaborate_record_fields(
        &mut self,
        self_ty: &Ty,
        item: &syn::ItemStruct,
    ) -> crate::Result<Vec<Field>> {
        item.fields
            .iter()
            .zip(0..)
            .map(|(field, index)| self.elaborate_record_field(self_ty, index, field))
            .collect()
    }

    fn elaborate_record_field(
        &mut self,
        self_ty: &Ty,
        index: usize,
        field: &syn::Field,
    ) -> crate::Result<Field> {
        match &field.ident {
            Some(name) => Ok(Field {
                span: self.source().span(name),
                name: util::recognize_name(name),
                ty: self.elaborate_owned_ty(Some(self_ty), &mut vec![], &field.ty)?,
            }),
            None => Ok(Field {
                span: self.source().span(field),
                name: Name::from(format!("f{index}")),
                ty: self.elaborate_owned_ty(Some(self_ty), &mut vec![], &field.ty)?,
            }),
        }
    }

    /// A "resource" has private fields -- co-data.
    fn elaborate_resource(
        &mut self,
        qname: &QualifiedName,
        definition: &Definition<'arena>,
        item: &syn::ItemStruct,
    ) -> crate::Result<Resource> {
        let span = || self.source().span(&item.ident);
        let self_ty = Ty::user(span(), qname);
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;

        Ok(Resource {
            span: span(),
            name: qname.tail_name(),
            methods,
        })
    }

    fn elaborate_variant(
        &mut self,
        qname: &QualifiedName,
        definition: &Definition<'arena>,
        item: &syn::ItemEnum,
        variants: &[&syn::Variant],
    ) -> crate::Result<Variant> {
        let span = self.source().span(&item.ident);
        let self_ty = Ty::user(span.clone(), qname);
        let arms = variants
            .iter()
            .map(|&v| self.elaborate_variant_arm(&self_ty, v))
            .collect::<crate::Result<Vec<_>>>()?;
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;
        Ok(Variant {
            span,
            name: util::recognize_name(&item.ident),
            arms,
            methods,
        })
    }

    fn elaborate_variant_arm(
        &mut self,
        self_ty: &Ty,
        variant: &syn::Variant,
    ) -> crate::Result<VariantArm> {
        let name = util::recognize_name(&variant.ident);
        match &variant.fields {
            syn::Fields::Named(fields) => Ok(VariantArm {
                span: self.source().span(&variant.ident),
                name,
                fields: fields
                    .named
                    .iter()
                    .zip(0..)
                    .map(|(field, index)| self.elaborate_record_field(self_ty, index, field))
                    .collect::<crate::Result<Vec<_>>>()?,
            }),
            syn::Fields::Unnamed(fields) => Ok(VariantArm {
                span: self.source().span(&variant.ident),
                name,
                fields: fields
                    .unnamed
                    .iter()
                    .zip(0..)
                    .map(|(field, index)| self.elaborate_record_field(self_ty, index, field))
                    .collect::<crate::Result<Vec<_>>>()?,
            }),
            syn::Fields::Unit => Ok(VariantArm {
                span: self.source().span(&variant.ident),
                name,
                fields: Default::default(),
            }),
        }
    }

    fn elaborate_enum(
        &mut self,
        qname: &QualifiedName,
        definition: &Definition<'arena>,
        item: &syn::ItemEnum,
        variants: &[&syn::Variant],
    ) -> crate::Result<Enum> {
        let arms = variants
            .iter()
            .map(|variant| {
                assert!(matches!(variant.fields, syn::Fields::Unit));
                crate::EnumArm {
                    span: self.source().span(&variant.ident),
                    name: util::recognize_name(&variant.ident),
                }
            })
            .collect::<Vec<_>>();
        let span = self.source().span(&item.ident);
        let self_ty = Ty::user(span.clone(), qname);
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;
        Ok(Enum {
            span,
            name: util::recognize_name(&item.ident),
            arms,
            methods,
        })
    }

    fn elaborate_methods(
        &self,
        module: &syn::File,
        self_ty: &Ty,
        ident: &syn::Ident,
    ) -> crate::Result<Vec<Method>> {
        let mut methods = vec![];

        let inherent_impls = self.find_inherent_impls(module, ident);

        for impl_item in inherent_impls {
            for item_in_impl in &impl_item.items {
                match item_in_impl {
                    syn::ImplItem::Fn(fn_item) => {
                        self.parse_method(&mut methods, self_ty, impl_item, fn_item)?;
                    }

                    syn::ImplItem::Const(item_in_impl) => {
                        if !util::ignore(&item_in_impl.vis, &item_in_impl.attrs) {
                            return Err(self.error(Error::UnsupportedItem, &item_in_impl));
                        }
                    }
                    syn::ImplItem::Type(item_in_impl) => {
                        if !util::ignore(&item_in_impl.vis, &item_in_impl.attrs) {
                            return Err(self.error(Error::UnsupportedItem, &item_in_impl));
                        }
                    }
                    syn::ImplItem::Macro(item_in_impl) => {
                        if !util::ignore_from_attrs(&item_in_impl.attrs) {
                            return Err(self.error(Error::UnsupportedItem, &item_in_impl));
                        }
                    }

                    syn::ImplItem::Verbatim(impl_item) => {
                        return Err(self.error(Error::UnsupportedItem, &impl_item));
                    }

                    _ => return Err(self.error(Error::UnrecognizedItem, &item_in_impl)),
                }
            }
        }

        Ok(methods)
    }

    fn parse_method(
        &self,
        methods: &mut Vec<Method>,
        self_ty: &Ty,
        impl_item: &syn::ItemImpl,
        fn_item: &syn::ImplItemFn,
    ) -> crate::Result<()> {
        if util::ignore(&fn_item.vis, &fn_item.attrs) {
            return Ok(());
        }

        if !impl_item.generics.params.is_empty() {
            return Err(self.error(Error::GenericsNotPermitted, &impl_item.generics));
        }

        let method = self.elaborate_fn_sig(Some(self_ty), &fn_item.sig)?;
        methods.push(method);
        Ok(())
    }

    fn function_input_name(&self, input: &syn::PatType) -> crate::Result<Name> {
        match &*input.pat {
            syn::Pat::Ident(ident) => Ok(util::recognize_name(&ident.ident)),
            _ => Err(self.error(Error::UnsupportedInputPattern, &input)),
        }
    }

    /// Elaborate a Rust type under the given modifiers;
    fn elaborate_owned_ty(
        &self,
        self_ty: Option<&Ty>,
        modifiers: &mut Vec<Modifier>,
        ty: &syn::Type,
    ) -> crate::Result<Ty> {
        match self.elaborate_ty(self_ty, modifiers, ty)? {
            RefdTy::Owned(OwnedKind::Owned, ty) => Ok(ty),
            RefdTy::Ref(..) => Err(self.error(Error::UnsupportedType, ty)),
        }
    }

    fn elaborate_return_ty(
        &self,
        is_async: &mut IsAsync,
        self_ty: Option<&Ty>,
        return_ty: &syn::ReturnType,
    ) -> crate::Result<FunctionOutput> {
        match return_ty {
            syn::ReturnType::Default => {
                let span = self.source().span(return_ty);
                Ok(FunctionOutput { main_ty: Ty::unit(span).owned(), error_ty: None })
            }
            syn::ReturnType::Type(_, ty) => self.elaborate_returned_ty(is_async, self_ty, ty),
        }
    }

    fn elaborate_returned_ty(
        &self,
        is_async: &mut IsAsync,
        self_ty: Option<&Ty>,
        ty: &syn::Type,
    ) -> crate::Result<FunctionOutput> {
        let fallback = || {
            let main_ty = self.elaborate_ty(self_ty, &mut vec![], &ty)?;
            Ok(FunctionOutput { main_ty, error_ty: None })
        };

        // Special case: returning `-> Result<X, E>`, `-> impl Future<Output = T>`, etc
        let span = self.source().span(ty);
        match ty {
            syn::Type::Group(ty) => return self.elaborate_returned_ty(is_async, self_ty, &ty.elem),

            syn::Type::Paren(ty) => return self.elaborate_returned_ty(is_async, self_ty, &ty.elem),

            syn::Type::Path(type_path) => {
                let rust_path = self.elaborate_type_path(self_ty, type_path)?;
                if let Some((main_ty, err_ty)) = self.match_type_path(self_ty, &mut vec![], ty, &rust_path, &["std", "result", "Result"])? {
                    let err_ty = err_ty.owned_or_err()?.clone();
                    Ok(FunctionOutput { main_ty, error_ty: Some(err_ty) })
                } else if let Some(main_ty) = self.match_type_path(self_ty, &mut vec![], ty, &rust_path, &["anyhow", "Result"])? {
                    let err_ty = Ty::anyhow_error(span);
                    Ok(FunctionOutput { main_ty, error_ty: Some(err_ty) })
                } else {
                    fallback()
                }
            }

            syn::Type::ImplTrait(_) => {
                // FIXME: we want to detect `-> impl Future` and treat it as equivalent to an async function.
                fallback()
            }

            _ => fallback(),
        }
    }

    fn elaborate_ty(
        &self,
        self_ty: Option<&Ty>,
        modifiers: &mut Vec<Modifier>,
        ty: &syn::Type,
    ) -> crate::Result<RefdTy> {
        match ty {
            syn::Type::Group(ty) => self.elaborate_ty(self_ty, modifiers, &ty.elem),

            syn::Type::Paren(ty) => self.elaborate_ty(self_ty, modifiers, &ty.elem),

            syn::Type::ImplTrait(impl_trait_ty) => {
                // `impl Trait` are permitted if we recognize the trait
                self.elaborate_impl_trait_ty(self_ty, modifiers, ty, impl_trait_ty)
            }

            syn::Type::Path(type_path) => {
                // Type names can either come from the user or be a reference to something in the Rust stdlib
                // or well-known Rust crates.

                let rust_path = self.elaborate_type_path(self_ty, type_path)?;
                self.elaborate_ty_from_path(self_ty, modifiers, ty, rust_path)
            }

            syn::Type::Reference(ty) => {
                // Treat `&T` the same as `T`

                if let Some(m) = &ty.mutability {
                    // Do not permit `&mut`
                    return Err(self.error(Error::UnsupportedType, &m));
                }

                if let Some(m) = &ty.lifetime {
                    // Do not permit named lifetimes for now (do they do any harm though?)
                    return Err(self.error(Error::UnsupportedType, &m));
                }

                // `&T` is the same from an abstract point of view, only the Rust representation is affected.
                Self::with_modifier(
                    modifiers,
                    Modifier::Ref(crate::RefKind::AnonRef),
                    |modifiers| self.elaborate_ty(self_ty, modifiers, &ty.elem),
                )
            }

            syn::Type::Slice(ty) => {
                // Treat `[T]` as a list of `T`

                let span = self.source().span(ty);
                if let [Modifier::Ref(r)] = &**modifiers {
                    let elem = self.elaborate_owned_ty(self_ty, &mut vec![], &ty.elem)?;
                    Ok(TypeKind::Vec {
                        element: elem.clone(),
                        repr: crate::VecRepr::SliceRef,
                    }
                    .refd(span, r.clone()))
                } else {
                    Err(Error::UnsupportedType(span))
                }
            }

            syn::Type::Tuple(ty) => {
                // Tuples are first-class in our IR

                let span = self.source().span(ty);
                let tys = ty
                    .elems
                    .iter()
                    .map(|ty| self.elaborate_owned_ty(self_ty, &mut vec![], ty))
                    .collect::<crate::Result<Vec<_>>>()?;
                self.maybe_referenced(
                    modifiers,
                    ty,
                    Ty::new(
                        span,
                        TypeKind::Tuple {
                            elements: tys.clone(),
                            repr: crate::TupleRepr::Tuple(tys.len()),
                        },
                    ),
                )
            }

            // Everything else is not recognized.
            _ => return Err(self.error(Error::UnsupportedType, &ty)),
        }
    }

    fn elaborate_ty_from_path(
        &self,
        self_ty: Option<&Ty>,
        modifiers: &mut Vec<Modifier>,
        ty: &syn::Type,
        rust_path: RustPath<'_>,
    ) -> crate::Result<RefdTy> {
        // Check for `Self`
        if rust_path.idents.len() == 1 && rust_path.idents[0] == "Self" {
            if let Some(self_ty) = self_ty {
                self.maybe_referenced(modifiers, ty, self_ty.clone())
            } else {
                Err(self.error(Error::UnresolvedName, &ty))
            }
        } else if let Some(rust_ty) =
            self.elaborate_rust_type(self_ty, modifiers, ty, &rust_path, &KNOWN_RUST_TYPES)?
        {
            // Found a well-known Rust type.
            Ok(rust_ty)
        } else if let Some(user_ty) =
            self.elaborate_user_type(ty, &rust_path.idents, &rust_path.tys)?
        {
            // Found a type defined by the user in the input somewhere.

            // Currently we don't have any kind of user types etc that support bindings.
            if !rust_path.bindings.is_empty() {
                return Err(self.error(Error::BindingNotExpected, ty));
            }

            self.maybe_referenced(modifiers, ty, user_ty)
        } else {
            // Unknown or unsupported type.
            Err(self.error(Error::UnresolvedName, &ty))
        }
    }

    /// Match the path, deconstructed into `idents` and `tys`, that appears in `ty` against the list `krts` of known Rust types.
    /// Returns `Ok(Some(ty))` if the match is successful or `Ok(None)` if there is no match.
    /// Returns an error if there is a match for the name but the arity is wrong or some other similar situation.
    fn elaborate_rust_type(
        &self,
        self_ty: Option<&Ty>,
        modifiers: &mut Vec<Modifier>,
        ty: &syn::Type,
        path: &RustPath<'_>,
        krts: &[KnownRustType],
    ) -> crate::Result<Option<RefdTy>> {
        let Some(krt) = krts.iter().find(|krt| self.type_path_matches(path, krt.name)) else {
            return Ok(None); 
        };

        // Construct the type kind.
        let span = self.source().span(ty);
        match &krt.kr_fn {
            KnownRustFn::MakeType(f) => {
                let tys = path
                    .tys
                    .iter()
                    .map(|ty| self.elaborate_owned_ty(self_ty, modifiers, ty))
                    .collect::<crate::Result<Vec<Ty>>>()?;
                let ty = f(span.clone(), &modifiers, &tys, &path.bindings)?;
                Ok(Some(ty))
            }
            KnownRustFn::Modifier(modifier) => {
                if path.tys.len() != 1 {
                    return Err(self.error(Error::UnsupportedType, &ty));
                }
                Self::with_modifier(modifiers, modifier.clone(), |modifiers| {
                    let ty = path.tys[0];
                    Ok(Some(self.elaborate_ty(self_ty, modifiers, ty)?))
                })
            }
        }
    }

    /// Returns true if the user-provided `path` matches against the known Rust path (e.g., `std::option::Option`) we are looking for.
    fn type_path_matches(&self, path: &RustPath<'_>, known_rust_path: &[&str]) -> bool {
        if path.idents.len() == 1 {
            // If the user just wrote `Foo`, search just the last identifier.
            // We just assume all std Rust types are either in the prelude or are imported by some `use`.
            // This is a bit of a hack because the user may have shadowed e.g. `HashMap` with their own `HashMap`
            // and we won't notice. Oh well, I'm lazy.
            path.idents[0] == known_rust_path.last().unwrap()
        } else {
                path.idents.len() == known_rust_path.len()
                    && path.idents.iter().zip(known_rust_path).all(|(a, b)| a == b)
        }
    }

    /// If `path` matches against `known_rust_path`, returns the type parameters in vector form
    fn match_type_path<M>(&self, self_ty: Option<&Ty>, modifiers: &mut Vec<Modifier>, ty: &syn::Type, path: &RustPath<'_>, known_rust_path: &[&str]) -> crate::Result<Option<M>> 
    where 
    M: MatchArity,{
        if !self.type_path_matches(path, known_rust_path) {
            return Ok(None);
        }

        if !path.bindings.is_empty() {
            return Err(self.error(Error::BindingNotExpected, ty));
        }

        let tys = path
        .tys
        .iter()
        .map(|ty| self.elaborate_ty(self_ty, modifiers, ty))
        .collect::<crate::Result<Vec<_>>>()?;

        Ok(M::match_arity(tys))
    }

    /// Match the impl trait type `ty`, deconstructed into `impl_trait_ty`.
    fn elaborate_impl_trait_ty(
        &self,
        self_ty: Option<&Ty>,
        modifiers: &mut Vec<Modifier>,
        ty: &syn::Type,
        impl_trait_ty: &syn::TypeImplTrait,
    ) -> crate::Result<RefdTy> {
        for bound in impl_trait_ty.bounds.iter() {
            match bound {
                syn::TypeParamBound::Trait(bound) => {
                    let rust_path = self.elaborate_path(self_ty, &bound.path)?;
                    if let Some(ty) = self.elaborate_rust_type(
                        self_ty,
                        modifiers,
                        ty,
                        &rust_path,
                        KNOWN_RUST_IMPL_TRAIT_TYPES,
                    )? {
                        return Ok(ty);
                    } else {
                        return Err(self.error(Error::UnsupportedType, &bound));
                    }
                }

                syn::TypeParamBound::Lifetime(bound) => {
                    if bound.ident == "static" {
                        // OK
                    } else {
                        return Err(self.error(Error::UnsupportedType, &bound));
                    }
                }

                syn::TypeParamBound::PreciseCapture(_) => {
                    // ignore these, not relevant to FFI
                }

                _ => return Err(self.error(Error::UnsupportedType, &bound)),
            }
        }
        return Err(self.error(Error::UnsupportedType, &ty));
    }

    /// Try to resolve the path type `ty`, broken down into `idents` and `tys`,
    /// against the user-defined this we are aware of. `idents` can be an absolute or relative path.
    ///
    /// Returns
    ///
    /// * `Ok(None)` if the first name does not match anything
    /// * `Ok(Some(ty))` upon success
    /// * `Err(_)` otherwise
    fn elaborate_user_type(
        &self,
        ty: &syn::Type,
        idents: &[syn::Ident],
        syn_tys: &[&syn::Type],
    ) -> crate::Result<Option<Ty>> {
        let Some((ident0, idents_rest)) = idents.split_first() else {
            unreachable!("empty list of idents")
        };

        let tys = syn_tys
            .iter()
            .map(|ty| self.elaborate_owned_ty(None, &mut vec![], ty))
            .collect::<crate::Result<Vec<_>>>()?;

        if ident0 == "crate" {
            // A path beginning with `crate::foo` is an absolute path relative to the crate name.
            let crate_name = self.module_qname.just_crate();
            match self.elaborate_user_ty_in_module_relative_to(
                ty,
                &crate_name,
                idents_rest,
                &tys,
            )? {
                Some(ty) => Ok(Some(ty)),
                None => Err(self.error(Error::UnresolvedName, &ty)),
            }
        } else {
            // Other paths are relative to the current module.
            self.elaborate_user_ty_in_module_relative_to(ty, &self.module_qname, idents, &tys)
        }
    }

    /// Try to resolve the remainder of a path against the list of exports from this module.
    ///
    /// Returns
    ///
    /// * `Ok(None)` if the first name does not match anything
    /// * `Ok(Some(ty))` upon success
    /// * `Err(_)` otherwise
    fn elaborate_user_ty_in_module_relative_to(
        &self,
        ty: &syn::Type,
        qname: &QualifiedName,
        idents: &[syn::Ident],
        tys: &[Ty],
    ) -> crate::Result<Option<Ty>> {
        let Some((ident0, idents_rest)) = idents.split_first() else {
            return Ok(None);
        };

        let path = qname.join(&Name::from_ident(ident0));
        match self.recognized.get(&path) {
            None => Ok(None),

            Some(definition) => match &definition.kind {
                DefinitionKind::FileModule => {
                    match self.elaborate_user_ty_in_module_relative_to(
                        ty,
                        &path,
                        idents_rest,
                        tys,
                    )? {
                        None => Err(self.error(Error::UnresolvedName, &ty)),
                        Some(ty) => Ok(Some(ty)),
                    }
                }
                DefinitionKind::Record(_)
                | DefinitionKind::Variant(..)
                | DefinitionKind::Enum(..)
                | DefinitionKind::Resource(_) => {
                    if !tys.is_empty() {
                        Err(self.error(Error::GenericsNotPermitted, &ty))
                    } else {
                        Ok(Some(Ty::user(self.source().span(ident0), &path)))
                    }
                }
                DefinitionKind::Function(_) => Err(self.error(Error::NotType, &ty)),
            },
        }
    }

    fn elaborate_type_path<'syn>(
        &self,
        self_ty: Option<&Ty>,
        type_path: &'syn syn::TypePath,
    ) -> crate::Result<RustPath<'syn>> {
        let syn::TypePath { qself, path } = type_path;

        if let Some(qself) = qself {
            return Err(self.error(Error::UnsupportedType, qself.span()));
        }

        self.elaborate_path(self_ty, path)
    }

    /// Resolves a path like `bar::Foo<T, U>` etc to a series of identifies (e.g., `[bar, Foo]`) and type arguments (e.g., `[T]`).
    fn elaborate_path<'syn>(
        &self,
        self_ty: Option<&Ty>,
        path: &'syn syn::Path,
    ) -> crate::Result<RustPath<'syn>> {
        let mut segments = path.segments.iter();
        let mut idents = vec![];
        let mut tys = vec![];
        let mut bindings = BTreeMap::new();

        while let Some(segment) = segments.next() {
            idents.push(segment.ident.clone());
            match &segment.arguments {
                syn::PathArguments::None => continue,
                syn::PathArguments::AngleBracketed(args) => {
                    for arg in &args.args {
                        match arg {
                            syn::GenericArgument::Type(ty) => {
                                tys.push(ty);
                            }
                            syn::GenericArgument::AssocType(assoc_ty) => {
                                let ty =
                                    self.elaborate_owned_ty(self_ty, &mut vec![], &assoc_ty.ty)?;
                                bindings.insert(Name::from_ident(&assoc_ty.ident), ty);
                            }
                            _ => {
                                return Err(self.error(Error::UnsupportedType, &arg));
                            }
                        }
                    }
                }
                syn::PathArguments::Parenthesized(args) => {
                    return Err(self.error(Error::UnsupportedType, &args));
                }
            }
        }

        if let Some(extra_segment) = segments.next() {
            return Err(self.error(Error::UnsupportedType, &extra_segment));
        }

        Ok(RustPath {
            idents,
            tys,
            bindings,
        })
    }

    fn with_modifier<R>(
        modifiers: &mut Vec<Modifier>,
        modifier: Modifier,
        op: impl FnOnce(&mut Vec<Modifier>) -> R,
    ) -> R {
        modifiers.push(modifier);
        let result = op(modifiers);
        modifiers.pop().unwrap();
        result
    }

    fn maybe_referenced(
        &self,
        modifiers: &Vec<Modifier>,
        spanned: impl Spanned,
        value: Ty,
    ) -> crate::Result<RefdTy> {
        if modifiers.is_empty() {
            Ok(value.owned())
        } else if let [Modifier::Ref(r)] = &modifiers[..] {
            Ok(value.refd(r.clone()))
        } else {
            Err(self.error(Error::UnsupportedType, spanned))
        }
    }

    // Given a struct name like `Foo`,
    fn find_inherent_impls(
        &self,
        module: &'arena syn::File,
        ident: &syn::Ident,
    ) -> Vec<&'arena syn::ItemImpl> {
        module
            .items
            .iter()
            .filter_map(|item| {
                if let syn::Item::Impl(item_impl) = item {
                    Some(item_impl)
                } else {
                    None
                }
            })
            .filter(|item_impl| item_impl.trait_.is_none())
            .filter(|item_impl| {
                if let syn::Type::Path(path) = &*item_impl.self_ty {
                    path.path.is_ident(ident)
                } else {
                    false
                }
            })
            .collect()
    }

    fn elaborate_function(
        &self,
        _qname: &QualifiedName,
        _definition: &Definition<'arena>,
        item_fn: &&syn::ItemFn,
    ) -> crate::Result<Function> {
        let Method {
            span,
            category: _,
            name,
            signature,
        } = self.elaborate_fn_sig(None, &item_fn.sig)?;
        Ok(Function {
            span,
            name,
            signature,
        })
    }

    fn elaborate_fn_sig(
        &self,
        self_ty: Option<&Ty>,
        sig: &syn::Signature,
    ) -> crate::Result<Method> {
        if !sig.generics.params.is_empty() {
            return Err(self.error(Error::GenericsNotPermitted, &sig.generics));
        }

        let name = util::recognize_name(&sig.ident);

        // Check for `&self` and friends
        let self_kind = if let Some(syn::FnArg::Receiver(receiver)) = sig.inputs.first() {
            if let Some(colon) = receiver.colon_token {
                return Err(self.error(Error::ExplicitSelfNotSupported, &colon));
            }

            if receiver.reference.is_none() {
                Some(SelfKind::ByValue)
            } else if receiver.mutability.is_none() {
                Some(SelfKind::ByRef)
            } else {
                Some(SelfKind::ByRefMut)
            }
        } else {
            None
        };

        // Check for inputs
        let mut inputs = vec![];
        for input in &sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    // Already handled this.
                    continue;
                }

                syn::FnArg::Typed(input) => {
                    let ty = self.elaborate_ty(self_ty, &mut vec![], &input.ty)?;
                    inputs.push(FunctionInput {
                        span: self.source().span(&input.pat),
                        name: self.function_input_name(input)?,
                        refd_ty: ty,
                    })
                }
            }
        }

        let mut is_async = if sig.asyncness.is_some() {
            IsAsync::Yes
        } else {
            IsAsync::No
        };

        let output_ty = self.elaborate_return_ty(&mut is_async, self_ty, &sig.output)?;

        // Detect if the return type is an owned copy of the self type;
        // this will be used to decide whether to categorize this as a builder
        // method.
        let output_is_self = if let Some(self_ty) = self_ty {
            output_ty.main_ty.owned_ty() == Some(self_ty)
        } else {
            false
        };

        // Categorize the function
        let category = match self_kind {
            None if sig.ident == "new" && output_is_self => MethodCategory::Constructor,
            None => MethodCategory::StaticMethod,
            Some(SelfKind::ByValue) if output_is_self => {
                MethodCategory::BuilderMethod(self_kind.unwrap())
            }
            Some(self_kind) => MethodCategory::InstanceMethod(self_kind),
        };

        Ok(Method {
            span: self.source().span(&sig.ident),
            category,
            name,
            signature: Signature {
                is_async,
                inputs,
                output_ty,
            },
        })
    }
}

pub trait MatchArity: Sized {
    fn match_arity(v: Vec<RefdTy>) -> Option<Self>;
}

impl MatchArity for RefdTy {
    fn match_arity(v: Vec<RefdTy>) -> Option<Self> {
        if v.len() == 1 {
            Some(v[0].clone())
        } else {
            None
        }
    }
}

impl MatchArity for (RefdTy, RefdTy) {
    fn match_arity(v: Vec<RefdTy>) -> Option<Self> {
        if v.len() == 2 {
            Some((v[0].clone(), v[1].clone()))
        } else {
            None
        }
    }
}