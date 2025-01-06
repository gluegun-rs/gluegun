use std::{
    collections::BTreeMap, path::{Path, PathBuf}, str::FromStr
};

use syn::{spanned::Spanned, ImplItemFn};

use crate::{Enum, Error, Field, Function, IsAsync, Item, Method, Module, Name, QualifiedName, Record, Resource, RustRepr, RustReprKind, SelfKind, Ty, Universe, Variant, VariantArm};

pub fn parse_path(path: &Path) -> crate::Result<Universe> {
    let text = std::fs::read_to_string(path)?;
    let tokens = proc_macro2::TokenStream::from_str(&text)?;
    let ast: syn::File = syn::parse2(tokens)?;

    todo!()
}

#[derive(Default)]
pub struct ParserArena {
    files: typed_arena::Arena<syn::File>,
}

struct Parser<'arena> {
    crate_name: Name,
    arena: &'arena ParserArena,
    definitions: BTreeMap<QualifiedName, Definition<'arena>>,
}

/// Internal intermediate structure representing some kind of public user-visible definition.
struct Definition<'p> {
    /// The syn module from which this was parsed.
    module: &'p syn::File, 

    /// The kind of definition.
    kind: DefinitionKind<'p>,
}

/// Internal intermediate structure representing kind of some public user-visible definition.
/// The names reference [WIT](https://component-model.bytecodealliance.org/design/wit.html).
enum DefinitionKind<'p> {
    /// *Resources* are "class-like" structures defined by their methods.
    /// In Rust, they are represented by a struct with private fields or a `#[non_exhaustive]` attribute.
    Resource(&'p syn::ItemStruct),

    /// *Records* are "struct-like" structures defined by their fields.
    /// In Rust, they are represented by a struct with public fields and no `#[non_exhaustive]` attribute.
    Record(&'p syn::ItemStruct),

    /// *Variants* are "enum-like" structures with data-carrying fields.
    /// In Rust, they are represented by a non-C-like enum that is not marked with `#[non_exhaustive]`.
    Variant(&'p syn::ItemEnum, Vec<&'p syn::Variant>),

    /// *Enums* are "enum-like" structures with no data-carrying fields.
    /// In Rust, they are represented by a C-like enum that is not marked with `#[non_exhaustive]`.
    Enum(&'p syn::ItemEnum, Vec<&'p syn::Variant>),

    /// *Functions* are top-level, callable functions (!).
    Function(&'p syn::ItemFn),

    /// *Modules* are public Rust modules; unlike the other variants, these are not mapped to output items,
    /// but they are used in name resolution.
    FileModule(&'p syn::File),
}

impl<'p> Parser<'p> {
    pub fn new(arena: &'p ParserArena, crate_name: Name) -> Self {
        Self {
            arena,
            crate_name,
            definitions: BTreeMap::new(),
        }
    }

    /// The path of the root crate file (typically something like `src/lib.rs`)
    pub fn parse(&mut self, crate_root_path: impl AsRef<Path>) -> crate::Result<()> {
        self.parse_path(crate_root_path.as_ref())
    }

}

/// Pass 1: Recognize types, imports, and things. Don't fill out the details (fields, methods).
mod pass1;

/// Pass 2: Fill out the fields, methods, etc. In this pass we resolve types.
mod pass2;

mod known_rust;

mod util;

// struct Recognizer<'p> {
//     parser: &'p mut Parser,
//     ast: &'p syn::File,
// }

// impl<'p> Recognizer<'p> {
//     fn recognize_all(&mut self) -> crate::Result<()> {
//         for item in &self.ast.items {
//             self.recognize_item(item)?;
//         }
//         Ok(())
//     }

//     fn recognize_item(&mut self, item: &syn::Item) -> crate::Result<()> {
//         match item {
//             syn::Item::Struct(item) => self.recognize_struct(item),

//             syn::Item::Enum(item) => self.recognize_enum(item),

//             syn::Item::Mod(item) => todo!(),

//             syn::Item::Trait(item) => todo!(),

//             syn::Item::Fn(item_fn) => todo!(),

//             syn::Item::Type(item_type) => todo!(),

//             syn::Item::Use(item_use) => todo!(),

//             _ => {}
//         }

//         Ok(())
//     }

//     fn recognize_struct(&mut self, item: &syn::ItemStruct) -> crate::Result<Option<Item>> {
//         if self.ignore(&item.vis, &item.attrs) {
//             return Ok(None);
//         }

//         if item.generics.params.len() > 0 {
//             return Err(Error::GenericsNotPermitted(item.generics.span()));
//         }

//         let public_fields = item
//             .fields
//             .iter()
//             .filter(|field| self.is_public(&field.vis))
//             .count();

//         if public_fields > 0 && public_fields == item.fields.len() {
//             // All public fields: this is a struct.
//             //
//             // It can have methods, but they have to be `&self` or `self`.
//             Ok(Some(Item::Record(self.recognize_struct_as_record(item)?)))
//         } else if public_fields == 0 {
//             // All private fields, this is a class
//             Ok(Some(Item::Resource(self.recognize_struct_as_resource(item)?)))
//         } else {
//             // Some public, some private fields -- error.

//             Err(Error::MixedPublicPrivateFields(item.span()))
//         }
//     }

//     /// A "record" has fields -- data.
//     fn recognize_struct_as_record(&mut self, item: &syn::ItemStruct) -> crate::Result<Record> {
//         self.work_queue.push(WorkItem::RecognizeRecordMethods(item.ident.clone()));
//         Ok(Record {
//             name: self.recognize_name(&item.ident),
//             fields: self.recognize_record_fields(item)?,
//             methods: vec![],
//         })
//     }

//     /// Recognize fields for a record.
//     fn recognize_record_fields(&mut self, item: &syn::ItemStruct) -> crate::Result<Vec<Field>> {
//         item.fields.iter().map(|field| {
//             match &field.ident {
//                 Some(name) => {
//                     Ok(crate::Field {
//                         name: self.recognize_name(&item.ident),
//                         ty: self.recognize_ty(&field.ty)?,
//                     })
//                 }
//                 None => {
//                     Err(Error::AnonymousField(field.span()))
//                 }
//             }
//         }).collect()
//     }

//     /// A "resource" has private fields -- co-data.
//     fn recognize_struct_as_resource(&mut self, item: &syn::ItemStruct) -> crate::Result<Resource> {
//         self.work_queue.push(WorkItem::RecognizeResourceMethods(item.ident.clone()));
//         Ok(Resource {
//             constructor: Default::default(),,
//             builder_field_methods: Default::default(),
//             methods: Default::default(),
//         })
//     }

//     fn recognize_enum(&mut self, item: &syn::ItemEnum) -> crate::Result<Option<Item>> {
//         if self.ignore(&item.vis, &item.attrs) {
//             return Ok(None);
//         }

//         if item.generics.params.len() > 0 {
//             return Err(Error::GenericsNotPermitted(item.generics.span()));
//         }

//         let unignored_variants = item.variants.iter().filter(|variant| !self.ignore_from_attrs(&variant.attrs)).collect::<Vec<_>>();

//         let variants_have_args = unignored_variants.iter().any(|v| match &v.fields {
//             syn::Fields::Named(_) | syn::Fields::Unnamed(_) => true,
//             syn::Fields::Unit => false,
//         });

//         if variants_have_args {
//             Ok(Some(Item::Variant(self.recognize_enum_as_variant(item, unignored_variants)?)))
//         } else {
//             Ok(Some(Item::Enum(self.recognize_enum_as_enum(item, unignored_variants)?)))
//         }
//     }

//     fn recognize_enum_as_variant(&mut self, item: &syn::ItemEnum, unignored_variants: Vec<&syn::Variant>) -> crate::Result<Variant> {
//         let arms = unignored_variants.iter().map(|v| self.recognize_variant_arm(v)).collect::<crate::Result<Vec<_>>>()?;
//         Ok(Variant {
//             name: self.recognize_name(&item.ident),
//             arms,
//             methods: Default::default(),
//         })
//     }

//     fn recognize_variant_arm(&mut self, variant: &syn::Variant) -> crate::Result<VariantArm> {
//         let name = self.recognize_name(&variant.ident);
//         match &variant.fields {
//             syn::Fields::Named(fields_named) => Err(Error::AnonymousFieldRequired(fields_named.span())),
//             syn::Fields::Unnamed(fields_unnamed) => Ok(VariantArm {
//                 name,
//                 fields: fields_unnamed
//                     .unnamed
//                     .iter()
//                     .map(|field| self.recognize_ty(&field.ty))
//                     .collect::<crate::Result<Vec<_>>>()?,
//             }),
//             syn::Fields::Unit => Ok(VariantArm {
//                 name, 
//                 fields: Default::default(),
//             })
//         }
//     }

//     fn recognize_enum_as_enum(&mut self, item: &syn::ItemEnum, unignored_variants: Vec<&syn::Variant>) -> crate::Result<Enum> {
//         let arms = unignored_variants.iter().map(|v| self.recognize_enum_arm(v)).collect::<crate::Result<Vec<_>>>()?;
//         Ok(Enum {
//             name: self.recognize_name(&item.ident),
//             arms,
//             methods: Default::default(),
//         })
//     }

//     fn recognize_enum_arm(&mut self, variant: &syn::Variant) -> crate::Result<crate::EnumArm> {
//         assert!(matches!(variant.fields, syn::Fields::Unit));
//         Ok(crate::EnumArm { name: self.recognize_name(&variant.ident) })
//     }

//     fn parse_methods(&self, inherent_impls: &[&syn::ItemImpl]) -> crate::Result<Vec<Method>> {
//         let mut methods = vec![];

//         for inherent_impl in inherent_impls {
//             for impl_item in &inherent_impl.items {
//                 match impl_item {
//                     syn::ImplItem::Fn(impl_item) => {
//                         methods.extend(self.parse_method(impl_item))?;
//                     }

//                     syn::ImplItem::Const(impl_item) => {
//                         if !self.ignore(&impl_item.vis, &impl_item.attrs) {
//                             return Err(Error::UnsupportedItem(impl_item.span()))
//                         }
//                     }
//                     syn::ImplItem::Type(impl_item) => {
//                         if !self.ignore(&impl_item.vis, &impl_item.attrs) {
//                             return Err(Error::UnsupportedItem(impl_item.span()))
//                         }
//                     }
//                     syn::ImplItem::Macro(impl_item) => {
//                         if !self.ignore_from_attrs(&impl_item.attrs) {
//                             return Err(Error::UnsupportedItem(impl_item.span()))
//                         }
//                     }

//                     syn::ImplItem::Verbatim(impl_item) => {
//                     }
                    
//                     _ => {
//                         return Err(Error::UnrecognizedItem(impl_item.span()))
//                     }
//                 }
//             }
//         }

//         Ok(methods)
//     }

//     fn parse_method(&self, impl_item: &ImplItemFn) -> crate::Result<Option<Method>> {
//         if self.ignore(&impl_item.vis, &impl_item.attrs) {
//             return Ok(None);
//         }

//         let is_async = if impl_item.sig.asyncness.is_some() {
//             IsAsync::Yes
//         } else {
//             IsAsync::No
//         };

//         // Check for `&self` and friends
//         let self_kind = if let Some(syn::FnArg::Receiver(receiver)) = impl_item.sig.inputs.first() {
//             if let Some(colon_span) = receiver.colon_token {
//                 return Err(Error::ExplicitSelfNotSupported(colon_span));
//             }

//             if receiver.reference.is_none() {
//                 Some(SelfKind::ByValue)    
//             } else if receiver.mutability.is_none() {
//                 Some(SelfKind::ByRef)
//             } else {
//                 Some(SelfKind::ByRefMut)
//             }
//         } else {
//             None
//         };

//         // Check for inputs
//         let mut input_tys = vec![];
//         for input in &impl_item.sig.inputs {
//             match input {
//                 syn::FnArg::Receiver(_) => {
//                     // Already handled this.
//                     continue
//                 }

//                 syn::FnArg::Typed(input) => {
//                     input_ts.push(self.recognize_ty(&input.ty)?);
//                 }
//             }
//         }

//         todo!()
//     }

//     fn recognize_ty(&self, ty: &syn::Type) -> crate::Result<Ty> {
//         match ty {
//             syn::Type::Group(ty) => self.recognize_ty(&ty.elem),
//             syn::Type::ImplTrait(ty) => todo!(),
//             syn::Type::Paren(ty) => self.recognize_ty(&ty.elem),
//             syn::Type::Path(ty) => {
//                 if let Some(qself) = &ty.qself {
//                     return Err(Error::UnsupportedType(m.span()));
//                 }

                
//             }
//             syn::Type::Reference(ty) => {
//                 if let Some(m) = &ty.mutability {
//                     return Err(Error::UnsupportedType(m.span()));
//                 }

//                 if let Some(m) = &ty.lifetime {
//                     return Err(Error::UnsupportedType(m.span()));
//                 }

//                 let inner_ty = self.recognize_ty(&ty.elem)?;

//                 // `&T` is the same from an abstract point of view, only the Rust representation is affected.
//                 Ok(inner_ty.with_repr(RustReprKind::Ref))
//             }
//             syn::Type::Slice(ty) => {
//                 let elem = self.recognize_ty(&ty.elem)?;
//                 Ok(Ty::new(
//                     crate::TypeKind::Vec { element: elem.clone() },
//                     RustReprKind::Slice(elem),
//                 ))
//             }
//             syn::Type::Tuple(ty) => {
//                 let tys = ty.elems.iter().map(|ty| self.recognize_ty(ty)).collect::<crate::Result<Vec<_>>>()?;
//                 Ok(Ty::new(
//                     crate::TypeKind::Tuple { elements: tys.clone() },
//                     RustReprKind::Tuple(tys),
//                 ))
//             }
//             _ => return Err(Error::UnsupportedType(ty.span())),
//         }
//     }

//     // Given a struct name like `Foo`,
//     fn find_inherent_impls(&self, struct_name: &syn::Ident) -> Vec<&'p syn::ItemImpl> {
//         self.ast
//             .items
//             .iter()
//             .filter_map(|item| {
//                 if let syn::Item::Impl(item_impl) = item {
//                     Some(item_impl)
//                 } else {
//                     None
//                 }
//             })
//             .filter(|item_impl| item_impl.trait_.is_none())
//             .filter(|item_impl| {
//                 if let syn::Type::Path(path) = &*item_impl.self_ty {
//                     path.path.is_ident(struct_name)
//                 } else {
//                     false
//                 }
//             })
//             .collect()
//     }

//     /// If true, ignore this item.
//     fn ignore(&self, vis: &syn::Visibility, attrs: &[syn::Attribute]) -> bool {
//         // Only look at public things
//         if !self.is_public(vis) {
//             return true;
//         }

//         self.ignore_from_attrs(attrs)
//     }

//     fn ignore_from_attrs(&self, attrs: &[syn::Attribute]) -> bool {
//         // Only look at things that are not cfg(test)
//         if attrs.iter().any(|attr| attr.path().is_ident("cfg")) {
//             // FIXME: check that the attribute is test
//             return true;
//         }

//         // Ignore things tagged with `squared::ignore`
//         if attrs.iter().any(|attr| attr.path().is_ident("ignore")) {
//             // FIXME: check that attribute is "squared::ignore"
//             return true;
//         }

//         false
//     }

//     /// Returns true if this is fully public.
//     /// Non-public items don't concern us.
//     fn is_public(&self, vis: &syn::Visibility) -> bool {
//         match vis {
//             syn::Visibility::Public(_) => true,
//             syn::Visibility::Restricted(_) => false,
//             syn::Visibility::Inherited => false,
//         }
//     }

//     fn recognize_name(&self, ident: &syn::Ident) -> Name {
//         Name { text: ident.to_string() }
//     }
// }
