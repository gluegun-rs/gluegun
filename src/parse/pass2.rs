use std::collections::BTreeMap;

use syn::{spanned::Spanned, FnArg};

use crate::{
    Enum, Error, Field, Function, FunctionInput, IsAsync, Item, Method, MethodCategory, Name,
    QualifiedName, Record, Resource, RustReprKind, SelfKind, Signature, Ty, TypeKind, Variant,
    VariantArm,
};

use super::{
    known_rust::{self, elaborate_rust_type, KNOWN_RUST_IMPL_TRAIT_TYPES, KNOWN_RUST_TYPES},
    util, Definition, DefinitionKind,
};

pub(super) struct Elaborator<'pass2, 'arena> {
    in_definitions: &'pass2 BTreeMap<QualifiedName, Definition<'arena>>,
    module_qname: QualifiedName,
    out_items: BTreeMap<QualifiedName, Item>,
}

impl<'pass2, 'arena> Elaborator<'pass2, 'arena> {
    fn elaborate_all(&mut self) -> crate::Result<()> {
        for (qname, definition) in self.in_definitions {
            self.module_qname.set_to_module_of(qname);

            // Convert the input definition and produce the output definition.
            if let Some(item) = self.elaborate_definition(qname, definition)? {
                self.out_items.insert(qname.clone(), item);
            }

            self.module_qname.clear();
        }
        Ok(())
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
            DefinitionKind::Function(item_fn) => Ok(Some(Item::Function((
                self.elaborate_function(qname, definition, item_fn)?,
            )))),
            DefinitionKind::FileModule(_) => {
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
            return Err(Error::GenericsNotPermitted(item.generics.span()));
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
                None => Err(Error::AnonymousField(field.span())),
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
                Err(Error::AnonymousFieldRequired(fields_named.span()))
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
                            return Err(Error::UnsupportedItem(item_in_impl.span()));
                        }
                    }
                    syn::ImplItem::Type(item_in_impl) => {
                        if !util::ignore(&item_in_impl.vis, &item_in_impl.attrs) {
                            return Err(Error::UnsupportedItem(item_in_impl.span()));
                        }
                    }
                    syn::ImplItem::Macro(item_in_impl) => {
                        if !util::ignore_from_attrs(&item_in_impl.attrs) {
                            return Err(Error::UnsupportedItem(item_in_impl.span()));
                        }
                    }

                    syn::ImplItem::Verbatim(impl_item) => {
                        return Err(Error::UnsupportedItem(impl_item.span()));
                    }

                    _ => return Err(Error::UnrecognizedItem(item_in_impl.span())),
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
            return Err(Error::GenericsNotPermitted(impl_item.generics.span()));
        }

        if !fn_item.sig.generics.params.is_empty() {
            return Err(Error::GenericsNotPermitted(fn_item.sig.generics.span()));
        }

        let name = util::recognize_name(&fn_item.sig.ident);

        let is_async = if fn_item.sig.asyncness.is_some() {
            IsAsync::Yes
        } else {
            IsAsync::No
        };

        // Check for `&self` and friends
        let self_kind = if let Some(syn::FnArg::Receiver(receiver)) = fn_item.sig.inputs.first() {
            if let Some(colon) = receiver.colon_token {
                return Err(Error::ExplicitSelfNotSupported(colon.span()));
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
        for input in &fn_item.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    // Already handled this.
                    continue;
                }

                syn::FnArg::Typed(input) => {
                    let input_ty = self.elaborate_ty(Some(self_ty), &input.ty)?;
                    inputs.push(FunctionInput {
                        name: self.function_input_name(input)?,
                        ty: input_ty,
                    })
                }
            }
        }

        let output_ty = self.elaborate_return_ty(Some(self_ty), &fn_item.sig.output)?;

        let output_is_self = output_ty == *self_ty;

        let category = match self_kind {
            None if fn_item.sig.ident == "new" && output_is_self => MethodCategory::Constructor,
            None => MethodCategory::StaticMethod,
            Some(SelfKind::ByValue) if output_is_self => {
                MethodCategory::BuilderMethod(self_kind.unwrap())
            }
            Some(self_kind) => MethodCategory::InstanceMethod(self_kind),
        };

        methods.push(Method {
            category,
            name,
            signature: Signature {
                is_async,
                inputs,
                output_ty,
            },
        });
        Ok(())
    }

    fn function_input_name(&self, input: &syn::PatType) -> crate::Result<Name> {
        match &*input.pat {
            syn::Pat::Ident(ident) => Ok(util::recognize_name(&ident.ident)),
            _ => Err(Error::UnsupportedInputPattern(input.span())),
        }
    }

    fn elaborate_return_ty(
        &self,
        self_ty: Option<&Ty>,
        output: &syn::ReturnType,
    ) -> crate::Result<Ty> {
        match output {
            syn::ReturnType::Default => Ok(Ty::unit()),
            syn::ReturnType::Type(_, ty) => self.elaborate_ty(self_ty, ty),
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

                let (idents, tys) = self.elaborate_type_path(self_ty, type_path)?;

                if let Some(user_ty) = self.elaborate_user_type(ty, &idents, &tys)? {
                    // Found a type defined in this module.
                    Ok(user_ty)
                } else if let Some(rust_ty) =
                    elaborate_rust_type(ty, &idents, &tys, &KNOWN_RUST_TYPES)?
                {
                    // Found a well-known Rust type.
                    Ok(rust_ty)
                } else {
                    // Unknown or unsupported type.
                    Err(Error::UnresolvedName(type_path.span()))
                }
            }

            syn::Type::Reference(ty) => {
                // Treat `&T` the same as `T`

                if let Some(m) = &ty.mutability {
                    // Do not permit `&mut`
                    return Err(Error::UnsupportedType(m.span()));
                }

                if let Some(m) = &ty.lifetime {
                    // Do not permit named lifetimes for now (do they do any harm though?)
                    return Err(Error::UnsupportedType(m.span()));
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
            _ => return Err(Error::UnsupportedType(ty.span())),
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
                    let (idents, tys) = self.elaborate_path(self_ty, &bound.path)?;
                    if let Some(ty) = known_rust::elaborate_rust_type(
                        ty,
                        &idents,
                        &tys,
                        KNOWN_RUST_IMPL_TRAIT_TYPES,
                    )? {
                        return Ok(ty);
                    } else {
                        return Err(Error::UnsupportedType(bound.span()));
                    }
                }
                syn::TypeParamBound::Lifetime(bound) => {
                    if bound.ident == "static" {
                        // OK
                    } else {
                        return Err(Error::UnsupportedType(bound.span()));
                    }
                }
                syn::TypeParamBound::PreciseCapture(_) => {
                    // ignore these, not relevant to FFI
                }
                _ => return Err(Error::UnsupportedType(bound.span())),
            }
        }
        return Err(Error::UnsupportedType(ty.span()));
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
                None => Err(Error::UnresolvedName(ty.span())),
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
        match self.in_definitions.get(&path) {
            None => Ok(None),

            Some(definition) => match &definition.kind {
                DefinitionKind::FileModule(_) => {
                    match self.elaborate_user_ty_in_module_relative_to(
                        ty,
                        &path,
                        idents_rest,
                        tys,
                    )? {
                        None => Err(Error::UnresolvedName(ty.span())),
                        Some(ty) => Ok(Some(ty)),
                    }
                }
                DefinitionKind::Record(_)
                | DefinitionKind::Variant(..)
                | DefinitionKind::Enum(..)
                | DefinitionKind::Resource(_) => {
                    if !tys.is_empty() {
                        Err(Error::GenericsNotPermitted(ty.span()))
                    } else {
                        Ok(Some(Ty::user(&path)))
                    }
                }
                DefinitionKind::Function(_) => Err(Error::NotType(ty.span())),
            },
        }
    }

    fn elaborate_type_path(
        &self,
        self_ty: Option<&Ty>,
        type_path: &syn::TypePath,
    ) -> crate::Result<(Vec<syn::Ident>, Vec<Ty>)> {
        let syn::TypePath { qself, path } = type_path;

        if let Some(qself) = qself {
            return Err(Error::UnsupportedType(qself.span()));
        }

        self.elaborate_path(self_ty, path)
    }

    /// Resolves a path like `bar::Foo<T, U>` etc to a series of identifies (e.g., `[bar, Foo]`) and type arguments (e.g., `[T]`).
    fn elaborate_path(
        &self,
        self_ty: Option<&Ty>,
        path: &syn::Path,
    ) -> crate::Result<(Vec<syn::Ident>, Vec<Ty>)> {
        let mut segments = path.segments.iter();
        let mut idents = vec![];
        let mut tys = vec![];

        while let Some(segment) = segments.next() {
            idents.push(segment.ident.clone());
            match &segment.arguments {
                syn::PathArguments::None => continue,
                syn::PathArguments::AngleBracketed(args) => {
                    tys.extend(
                        args.args
                            .iter()
                            .map(|arg| self.elaborate_generic_argument_as_ty(self_ty, arg))
                            .collect::<crate::Result<Vec<_>>>()?,
                    );
                    break;
                }
                syn::PathArguments::Parenthesized(args) => {
                    return Err(Error::UnsupportedType(args.span()));
                }
            }
        }

        if let Some(extra_segment) = segments.next() {
            return Err(Error::UnsupportedType(extra_segment.span()));
        }

        Ok((idents, tys))
    }

    fn elaborate_generic_argument_as_ty(
        &self,
        self_ty: Option<&Ty>,
        arg: &syn::GenericArgument,
    ) -> crate::Result<Ty> {
        match arg {
            syn::GenericArgument::Type(ty) => self.elaborate_ty(self_ty, ty),
            _ => Err(Error::UnsupportedType(arg.span())),
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
        self_ty: Option<&Ty>,
        qname: &QualifiedName,
        _definition: &Definition<'arena>,
        item_fn: &&syn::ItemFn,
    ) -> crate::Result<Function> {
        let is_async = match item_fn.sig.asyncness {
            Some(_) => IsAsync::Yes,
            None => IsAsync::No,
        };

        let inputs = item_fn
            .sig
            .inputs
            .iter()
            .filter_map(|fn_arg| match fn_arg {
                // Ignore `&self` and friends. They are handled elsewhere.
                FnArg::Receiver(_) => None,
                FnArg::Typed(pat_type) => Some(pat_type),
            })
            .map(|fn_arg| self.elaborate_pat_type(self_ty, fn_arg))
            .collect::<crate::Result<Vec<_>>>()?;

        let output_ty = match &item_fn.sig.output {
            syn::ReturnType::Default => Ty::unit(),
            syn::ReturnType::Type(_, ty) => self.elaborate_ty(self_ty, ty)?,
        };

        let signature = Signature {
            is_async,
            inputs,
            output_ty,
        };

        let name = qname.tail_name();
        Ok(Function { name, signature })
    }

    fn elaborate_pat_type(
        &self,
        self_ty: Option<&Ty>,
        pat_type: &syn::PatType,
    ) -> crate::Result<FunctionInput> {
        let name = match &*pat_type.pat {
            syn::Pat::Ident(pat_ident) => Name::from_ident(&pat_ident.ident),
            _ => return Err(Error::UnsupportedInputPattern(pat_type.pat.span())),
        };

        let ty = self.elaborate_ty(self_ty, &pat_type.ty)?;

        Ok(FunctionInput { name, ty })
    }
}
