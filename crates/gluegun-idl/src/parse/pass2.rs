use std::{collections::BTreeMap, sync::Arc};

use syn::spanned::Spanned;

use crate::{
    Enum, Error, ErrorSpan, Field, Function, FunctionInput, FunctionOutput, IsAsync, Item, Method,
    MethodCategory, Name, QualifiedName, Record, Resource, RustName, RustReprKind, SelfKind,
    Signature, Ty, TypeKind, Variant, VariantArm,
};

use super::{
    known_rust::{
        self, elaborate_rust_type, RustPath, KNOWN_RUST_IMPL_TRAIT_TYPES, KNOWN_RUST_TYPES,
    },
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

    fn error(&self, variant: fn(ErrorSpan) -> Error, spanned: impl Spanned) -> Error {
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

        let self_ty = Ty::user(qname);
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;

        Ok(Record {
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
            .map(|field| match &field.ident {
                Some(name) => Ok(Field {
                    name: util::recognize_name(name),
                    ty: self.elaborate_ty(Some(self_ty), &field.ty)?,
                }),
                None => Err(self.error(Error::AnonymousField, &field)),
            })
            .collect()
    }

    /// A "resource" has private fields -- co-data.
    fn elaborate_resource(
        &mut self,
        qname: &QualifiedName,
        definition: &Definition<'arena>,
        item: &syn::ItemStruct,
    ) -> crate::Result<Resource> {
        let self_ty = Ty::user(qname);
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;

        Ok(Resource {
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
        let self_ty = Ty::user(qname);
        let arms = variants
            .iter()
            .map(|v| self.elaborate_variant_arm(&self_ty, v))
            .collect::<crate::Result<Vec<_>>>()?;
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;
        Ok(Variant {
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
            syn::Fields::Named(fields_named) => {
                Err(self.error(Error::AnonymousFieldRequired, &fields_named))
            }
            syn::Fields::Unnamed(fields_unnamed) => Ok(VariantArm {
                name,
                fields: fields_unnamed
                    .unnamed
                    .iter()
                    .map(|field| self.elaborate_ty(Some(self_ty), &field.ty))
                    .collect::<crate::Result<Vec<_>>>()?,
            }),
            syn::Fields::Unit => Ok(VariantArm {
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
                    name: util::recognize_name(&variant.ident),
                }
            })
            .collect::<Vec<_>>();
        let self_ty = Ty::user(qname);
        let methods = self.elaborate_methods(definition.module, &self_ty, &item.ident)?;
        Ok(Enum {
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

    /// Elaborate the Rust type into a [`FunctionOutput`][]; certain patterns (e.g., returning a result or `impl Future`)
    /// are specially recognized.
    fn elaborate_output_ty(
        &self,
        is_async: &mut IsAsync,
        self_ty: Option<&Ty>,
        return_ty: &syn::ReturnType,
    ) -> crate::Result<FunctionOutput> {
        // First convert the type in Rust to a `Ty`
        let elaborated_ty = match return_ty {
            syn::ReturnType::Default => Ty::unit(),
            syn::ReturnType::Type(_, ty) => self.elaborate_ty(self_ty, ty)?,
        };

        self.elaborated_ty_to_output(is_async, return_ty, &elaborated_ty)
    }

    fn elaborated_ty_to_output(
        &self,
        is_async: &mut IsAsync,
        return_ty: &syn::ReturnType,
        elaborated_ty: &Ty,
    ) -> crate::Result<FunctionOutput> {
        // Look for some special cases. For example, returning a `Result` is translated into a "fallible" function.
        match elaborated_ty.rust_repr().kind() {
            RustReprKind::Named(RustName::Result, args, _) => {
                assert_eq!(args.len(), 2);
                return Ok(FunctionOutput {
                    main_ty: args[0].clone(),
                    error_ty: Some(args[1].clone()),
                });
            }
            RustReprKind::Named(RustName::Future, _, bindings) => {
                if let Some(output) = bindings.get(&Name::output()) {
                    self.elaborated_ty_to_output(is_async, return_ty, output)
                } else {
                    Err(Error::BindingNotFound(
                        self.source().span(return_ty),
                        Name::output(),
                    ))
                }
            }
            _ => Ok(FunctionOutput {
                main_ty: elaborated_ty.clone(),
                error_ty: None,
            }),
        }
    }

    fn elaborate_ty(&self, self_ty: Option<&Ty>, ty: &syn::Type) -> crate::Result<Ty> {
        match ty {
            syn::Type::Group(ty) => self.elaborate_ty(self_ty, &ty.elem),

            syn::Type::Paren(ty) => self.elaborate_ty(self_ty, &ty.elem),

            syn::Type::ImplTrait(impl_trait_ty) => {
                // `impl Trait` are permitted if we recognize the trait
                self.elaborate_impl_trait_ty(self_ty, ty, impl_trait_ty)
            }

            syn::Type::Path(type_path) => {
                // Type names can either come from the user or be a reference to something in the Rust stdlib
                // or well-known Rust crates.

                let rust_path = self.elaborate_type_path(self_ty, type_path)?;
                self.elaborate_ty_from_path(self_ty, ty, rust_path)
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

                let inner_ty = self.elaborate_ty(self_ty, &ty.elem)?;

                // `&T` is the same from an abstract point of view, only the Rust representation is affected.
                Ok(inner_ty.with_repr(RustReprKind::Ref))
            }
            syn::Type::Slice(ty) => {
                // Treat `[T]` as a list of `T`

                let elem = self.elaborate_ty(self_ty, &ty.elem)?;
                Ok(Ty::new(
                    TypeKind::Vec {
                        element: elem.clone(),
                    },
                    RustReprKind::Slice(elem),
                ))
            }
            syn::Type::Tuple(ty) => {
                // Tuples are first-class in our IR

                let tys = ty
                    .elems
                    .iter()
                    .map(|ty| self.elaborate_ty(self_ty, ty))
                    .collect::<crate::Result<Vec<_>>>()?;
                Ok(Ty::new(
                    TypeKind::Tuple {
                        elements: tys.clone(),
                    },
                    RustReprKind::Tuple(tys),
                ))
            }

            // Everything else is not recognized.
            _ => return Err(self.error(Error::UnsupportedType, &ty)),
        }
    }

    fn elaborate_ty_from_path(
        &self,
        self_ty: Option<&Ty>,
        ty: &syn::Type,
        rust_path: RustPath,
    ) -> crate::Result<Ty> {
        // Check for `Self`
        if rust_path.idents.len() == 1 && rust_path.idents[0] == "Self" {
            if let Some(self_ty) = self_ty {
                Ok(self_ty.clone())
            } else {
                Err(self.error(Error::UnresolvedName, &ty))
            }
        } else if let Some(user_ty) =
            self.elaborate_user_type(ty, &rust_path.idents, &rust_path.tys)?
        {
            // Found a type defined by the user in the input somewhere.

            // Currently we don't have any kind of user types etc that support bindings.
            if !rust_path.bindings.is_empty() {
                return Err(self.error(Error::BindingNotExpected, ty));
            }

            Ok(user_ty)
        } else if let Some(rust_ty) =
            elaborate_rust_type(self.source(), ty, rust_path, &KNOWN_RUST_TYPES)?
        {
            // Found a well-known Rust type.
            Ok(rust_ty)
        } else {
            // Unknown or unsupported type.
            Err(self.error(Error::UnresolvedName, &ty))
        }
    }

    /// Match the impl trait type `ty`, deconstructed into `impl_trait_ty`.
    fn elaborate_impl_trait_ty(
        &self,
        self_ty: Option<&Ty>,
        ty: &syn::Type,
        impl_trait_ty: &syn::TypeImplTrait,
    ) -> crate::Result<Ty> {
        for bound in impl_trait_ty.bounds.iter() {
            match bound {
                syn::TypeParamBound::Trait(bound) => {
                    let rust_path = self.elaborate_path(self_ty, &bound.path)?;
                    if let Some(ty) = known_rust::elaborate_rust_type(
                        self.source(),
                        ty,
                        rust_path,
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
        tys: &[Ty],
    ) -> crate::Result<Option<Ty>> {
        let Some((ident0, idents_rest)) = idents.split_first() else {
            unreachable!("empty list of idents")
        };

        if ident0 == "crate" {
            // A path beginning with `crate::foo` is an absolute path relative to the crate name.
            let crate_name = self.module_qname.just_crate();
            match self.elaborate_user_ty_in_module_relative_to(ty, &crate_name, idents_rest, tys)? {
                Some(ty) => Ok(Some(ty)),
                None => Err(self.error(Error::UnresolvedName, &ty)),
            }
        } else {
            // Other paths are relative to the current module.
            self.elaborate_user_ty_in_module_relative_to(ty, &self.module_qname, idents, tys)
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
                        Ok(Some(Ty::user(&path)))
                    }
                }
                DefinitionKind::Function(_) => Err(self.error(Error::NotType, &ty)),
            },
        }
    }

    fn elaborate_type_path(
        &self,
        self_ty: Option<&Ty>,
        type_path: &syn::TypePath,
    ) -> crate::Result<RustPath> {
        let syn::TypePath { qself, path } = type_path;

        if let Some(qself) = qself {
            return Err(self.error(Error::UnsupportedType, qself.span()));
        }

        self.elaborate_path(self_ty, path)
    }

    /// Resolves a path like `bar::Foo<T, U>` etc to a series of identifies (e.g., `[bar, Foo]`) and type arguments (e.g., `[T]`).
    fn elaborate_path(&self, self_ty: Option<&Ty>, path: &syn::Path) -> crate::Result<RustPath> {
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
                                tys.push(self.elaborate_ty(self_ty, ty)?);
                            }
                            syn::GenericArgument::AssocType(assoc_ty) => {
                                let ty = self.elaborate_ty(self_ty, &assoc_ty.ty)?;
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
            category: _,
            name,
            signature,
        } = self.elaborate_fn_sig(None, &item_fn.sig)?;
        Ok(Function { name, signature })
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
                    let input_ty = self.elaborate_ty(self_ty, &input.ty)?;
                    inputs.push(FunctionInput {
                        name: self.function_input_name(input)?,
                        ty: input_ty,
                    })
                }
            }
        }

        let mut is_async = if sig.asyncness.is_some() {
            IsAsync::Yes
        } else {
            IsAsync::No
        };

        let output_ty = self.elaborate_output_ty(&mut is_async, self_ty, &sig.output)?;

        let output_is_self = if let Some(self_ty) = self_ty {
            output_ty.main_ty == *self_ty
        } else {
            false
        };

        let category = match self_kind {
            None if sig.ident == "new" && output_is_self => MethodCategory::Constructor,
            None => MethodCategory::StaticMethod,
            Some(SelfKind::ByValue) if output_is_self => {
                MethodCategory::BuilderMethod(self_kind.unwrap())
            }
            Some(self_kind) => MethodCategory::InstanceMethod(self_kind),
        };

        Ok(Method {
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
