use std::{collections::BTreeMap, sync::Arc};

use syn::spanned::Spanned;

use crate::{Error, ErrorSpan, QualifiedName, SourcePath};

use super::{util, Definition, DefinitionKind};

pub(super) struct Recognizer<'ast> {
    source: SourcePath,
    module_name: QualifiedName,
    ast: &'ast syn::File,
    recognized: BTreeMap<QualifiedName, Definition<'ast>>,
}

impl<'ast> Recognizer<'ast> {
    pub(super) fn new(
        source: &SourcePath,
        module_name: QualifiedName,
        ast: &'ast syn::File,
    ) -> Self {
        Self {
            source: source.clone(),
            module_name,
            ast,
            recognized: BTreeMap::new(),
        }
    }

    fn definition(&self, kind: DefinitionKind<'ast>) -> Definition<'ast> {
        Definition {
            kind,
            source: self.source.clone(),
            module: self.ast,
        }
    }

    fn error(&self, variant: fn(ErrorSpan) -> Error, spanned: impl Spanned) -> Error {
        variant(self.source.span(spanned))
    }

    pub(super) fn into_recognized(mut self) -> crate::Result<Arc<BTreeMap<QualifiedName, Definition<'ast>>>> {
        for item in &self.ast.items {
            self.recognize_item(item)?;
        }
        Ok(Arc::new(self.recognized))
    }

    fn recognize_item(&mut self, item: &'ast syn::Item) -> crate::Result<()> {
        match item {
            syn::Item::Struct(item) => self.recognize_struct(item),

            syn::Item::Enum(item) => self.recognize_enum(item),

            syn::Item::Fn(item) => self.recognize_fn(item),

            syn::Item::Mod(item) => self.recognize_mod(item),

            syn::Item::Trait(item) => self.recognize_trait(item),

            syn::Item::Type(item) => self.recognize_type(item),

            syn::Item::Use(item) => self.recognize_use(item),

            _ => Err(self.error(crate::Error::UnsupportedItem, item)),
        }
    }

    fn recognize_struct(&mut self, item: &'ast syn::ItemStruct) -> crate::Result<()> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        let name = util::recognize_name(&item.ident);
        let qname = self.module_name.join(&name);

        if item.generics.params.len() > 0 {
            return Err(self.error(Error::GenericsNotPermitted, &item.generics));
        }

        let public_fields = item
            .fields
            .iter()
            .filter(|field| util::is_public(&field.vis))
            .count();

        if public_fields > 0 && public_fields == item.fields.len() {
            // All public fields: this is a struct.
            //
            // It can have methods, but they have to be `&self` or `self`.
            self.recognized.insert(
                qname,
                self.definition(DefinitionKind::Record(item)),
            );
            Ok(())
        } else if public_fields == 0 {
            // All private fields, this is a class
            self.recognized.insert(
                qname,
                self.definition(DefinitionKind::Resource(item)),
            );
            Ok(())
        } else {
            // Some public, some private fields -- error.

            Err(self.error(Error::MixedPublicPrivateFields, item))
        }
    }

    fn recognize_enum(&mut self, item: &'ast syn::ItemEnum) -> crate::Result<()> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        if item.generics.params.len() > 0 {
            return Err(self.error(Error::GenericsNotPermitted, &item.generics));
        }

        let unignored_variants = item
            .variants
            .iter()
            .filter(|variant| !util::ignore_from_attrs(&variant.attrs))
            .collect::<Vec<_>>();

        let variants_have_args = unignored_variants.iter().any(|v| match &v.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => true,
            syn::Fields::Unit => false,
        });

        let name = util::recognize_name(&item.ident);
        let qname = self.module_name.join(&name);

        if variants_have_args {
            self.recognized.insert(
                qname,
                self.definition(DefinitionKind::Variant(item, unignored_variants)),
            );
            Ok(())
        } else {
            self.recognized.insert(
                qname,
                self.definition(DefinitionKind::Enum(item, unignored_variants)),
            );
            Ok(())
        }
    }

    fn recognize_fn(&mut self, item: &'ast syn::ItemFn) -> crate::Result<()> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        let name = util::recognize_name(&item.sig.ident);
        let qname = self.module_name.join(&name);

        if item.sig.generics.params.len() > 0 {
            return Err(self.error(Error::GenericsNotPermitted, &item.sig.generics));
        }

        self.recognized.insert(
            qname,
            self.definition(DefinitionKind::Function(item)),
        );
        Ok(())
    }

    fn recognize_mod(&self, item: &syn::ItemMod) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(self.error(crate::Error::UnsupportedItem, item))
    }

    fn recognize_trait(&self, item: &syn::ItemTrait) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(self.error(crate::Error::UnsupportedItem, item))
    }

    fn recognize_type(&self, item: &syn::ItemType) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(self.error(crate::Error::UnsupportedItem, item))
    }

    fn recognize_use(&self, item: &syn::ItemUse) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(self.error(crate::Error::UnsupportedItem, item))
    }
}
