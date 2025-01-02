use std::{
    collections::BTreeMap, f32::consts::E, path::{Path, PathBuf}, str::FromStr
};

use syn::{spanned::Spanned, ImplItemFn};

use crate::{IsAsync, Item, Method, Module, RustRepr, RustReprKind, SelfKind, Ty, Universe};

pub struct Parser {
    universe: Universe,
    crate_contents: BTreeMap<PathBuf, Module>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            universe: Universe {
                module: Module { items: vec![] },
            },
            crate_contents: Default::default(),
        }
    }

    /// The path of the root crate file (typically something like `src/lib.rs`)
    pub fn parse(&mut self, crate_root_path: impl AsRef<Path>) -> crate::Result<()> {
        self.parse_path(crate_root_path.as_ref())
    }

    fn parse_path(&mut self, path: &Path) -> crate::Result<()> {
        if self.crate_contents.contains_key(path) {
            return Ok(());
        }

        let text = std::fs::read_to_string(path)?;
        let tokens = proc_macro2::TokenStream::from_str(&text)?;
        let ast: syn::File = syn::parse2(tokens)?;

        Ok(())
    }
}

struct Recognizer<'p> {
    parser: &'p mut Parser,
    ast: &'p syn::File,
}

impl<'p> Recognizer<'p> {
    fn recognize_all(&mut self) -> crate::Result<()> {
        for item in &self.ast.items {
            self.recognize_item(item)?;
        }
        Ok(())
    }

    fn recognize_item(&mut self, item: &syn::Item) -> crate::Result<()> {
        match item {
            syn::Item::Struct(item) => self.recognize_struct(item),

            syn::Item::Enum(item) => todo!(),

            syn::Item::Mod(item) => todo!(),

            syn::Item::Trait(item) => todo!(),

            syn::Item::Fn(item_fn) => todo!(),

            syn::Item::Type(item_type) => todo!(),

            syn::Item::Use(item_use) => todo!(),

            _ => {}
        }

        Ok(())
    }

    fn recognize_struct(&mut self, item: &syn::ItemStruct) -> crate::Result<Item> {
        if self.ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        if item.generics.params.len() > 0 {
            return Err(crate::Error::GenericsNotPermitted(item.generics.span()));
        }

        let public_fields = item
            .fields
            .iter()
            .filter(|field| self.is_public(&field.vis))
            .count();

        if public_fields > 0 && public_fields == item.fields.len() {
            // All public fields: this is a struct.
            //
            // It can have methods, but they have to be `&self` or `self`.
            self.recognize_struct_as_record(item)
        } else if public_fields == 0 {
            // All private fields, this is a class
            self.recognize_struct_as_class(item)
        } else {
            // Some public, some private fields -- error.

            Err(crate::Error::MixedPublicPrivateFields(item.span()))
        }
    }

    /// A "struct" in our parlance corresponds to a 
    fn recognize_struct_as_record(&mut self, item: &syn::ItemStruct) -> crate::Result<Item> {

        Ok(Item::Struct {
            ident: item.ident.clone(),
            fields: item.fields.clone(),
            inherent_impls,
        })
    }

    fn parse_methods(&self, inherent_impls: &[&syn::ItemImpl]) -> crate::Result<Vec<Method>> {
        let mut methods = vec![];

        for inherent_impl in inherent_impls {
            for impl_item in &inherent_impl.items {
                match impl_item {
                    syn::ImplItem::Fn(impl_item) => {
                        methods.extend(self.parse_method(impl_item))?;
                    }

                    syn::ImplItem::Const(impl_item) => {
                        if !self.ignore(&impl_item.vis, &impl_item.attrs) {
                            return Err(crate::Error::UnsupportedItem(impl_item.span()))
                        }
                    }
                    syn::ImplItem::Type(impl_item) => {
                        if !self.ignore(&impl_item.vis, &impl_item.attrs) {
                            return Err(crate::Error::UnsupportedItem(impl_item.span()))
                        }
                    }
                    syn::ImplItem::Macro(impl_item) => {
                        if !self.ignore_from_attrs(&impl_item.attrs) {
                            return Err(crate::Error::UnsupportedItem(impl_item.span()))
                        }
                    }

                    syn::ImplItem::Verbatim(impl_item) => {
                    }
                    
                    _ => {
                        return Err(crate::Error::UnrecognizedItem(impl_item.span()))
                    }
                }
            }
        }

        Ok(methods)
    }

    fn parse_method(&self, impl_item: &ImplItemFn) -> crate::Result<Option<Method>> {
        if self.ignore(&impl_item.vis, &impl_item.attrs) {
            return Ok(None);
        }

        let is_async = if impl_item.sig.asyncness.is_some() {
            IsAsync::Yes
        } else {
            IsAsync::No
        };

        // Check for `&self` and friends
        let self_kind = if let Some(syn::FnArg::Receiver(receiver)) = impl_item.sig.inputs.first() {
            if let Some(colon_span) = receiver.colon_token {
                return Err(crate::Error::ExplicitSelfNotSupported(colon_span));
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
        let mut input_tys = vec![];
        for input in &impl_item.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    // Already handled this.
                    continue
                }

                syn::FnArg::Typed(input) => {
                    input_ts.push(self.recognize_ty(&input.ty)?);
                }
            }
        }

        todo!()
    }

    fn recognize_ty(&self, ty: &syn::Type) -> crate::Result<Ty> {
        match ty {
            syn::Type::Group(ty) => self.recognize_ty(&ty.elem),
            syn::Type::ImplTrait(ty) => todo!(),
            syn::Type::Paren(ty) => self.recognize_ty(&ty.elem),
            syn::Type::Path(ty) => {
                if let Some(qself) = &ty.qself {
                    return Err(crate::Error::UnsupportedType(m.span()));
                }

                
            }
            syn::Type::Reference(ty) => {
                if let Some(m) = &ty.mutability {
                    return Err(crate::Error::UnsupportedType(m.span()));
                }

                if let Some(m) = &ty.lifetime {
                    return Err(crate::Error::UnsupportedType(m.span()));
                }

                let inner_ty = self.recognize_ty(&ty.elem)?;

                // `&T` is the same from an abstract point of view, only the Rust representation is affected.
                Ok(inner_ty.with_repr(RustReprKind::Ref))
            }
            syn::Type::Slice(ty) => {
                let elem = self.recognize_ty(&ty.elem)?;
                Ok(Ty::new(
                    crate::TypeKind::Vec { element: elem.clone() },
                    RustReprKind::Slice(elem),
                ))
            }
            syn::Type::Tuple(ty) => {
                let tys = ty.elems.iter().map(|ty| self.recognize_ty(ty)).collect::<crate::Result<Vec<_>>>()?;
                Ok(Ty::new(
                    crate::TypeKind::Tuple { elements: tys.clone() },
                    RustReprKind::Tuple(tys),
                ))
            }
            _ => return Err(crate::Error::UnsupportedType(ty.span())),
        }
    }

    // Given a struct name like `Foo`,
    fn find_inherent_impls(&self, struct_name: &syn::Ident) -> Vec<&'p syn::ItemImpl> {
        self.ast
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
                    path.path.is_ident(struct_name)
                } else {
                    false
                }
            })
            .collect()
    }

    /// If true, ignore this item.
    fn ignore(&self, vis: &syn::Visibility, attrs: &[syn::Attribute]) -> bool {
        // Only look at public things
        if !self.is_public(vis) {
            return true;
        }

        self.ignore_from_attrs(attrs)
    }

    fn ignore_from_attrs(&self, attrs: &[syn::Attribute]) -> bool {
        // Only look at things that are not cfg(test)
        if attrs.iter().any(|attr| attr.path().is_ident("cfg")) {
            // FIXME: check that the attribute is test
            return true;
        }

        // Ignore things tagged with `squared::ignore`
        if attrs.iter().any(|attr| attr.path().is_ident("ignore")) {
            // FIXME: check that attribute is "squared::ignore"
            return true;
        }

        false
    }

    /// Returns true if this is fully public.
    /// Non-public items don't concern us.
    fn is_public(&self, vis: &syn::Visibility) -> bool {
        match vis {
            syn::Visibility::Public(_) => true,
            syn::Visibility::Restricted(_) => false,
            syn::Visibility::Inherited => false,
        }
    }
}
