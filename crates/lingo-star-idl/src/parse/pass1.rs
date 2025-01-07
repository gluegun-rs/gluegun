use syn::spanned::Spanned;

use crate::{Error, QualifiedName};

use super::{util, Definition, DefinitionKind, Parser};

pub(super) struct Recognizer<'pass1, 'arena> {
    parser: &'pass1 mut Parser<'arena>,
    module_name: QualifiedName,
    ast: &'arena syn::File,
}

impl<'pass1, 'arena> Recognizer<'pass1, 'arena> {
    fn recognize_all(&mut self) -> crate::Result<()> {
        for item in &self.ast.items {
            self.recognize_item(item)?;
        }
        Ok(())
    }

    fn recognize_item(&mut self, item: &'arena syn::Item) -> crate::Result<()> {
        match item {
            syn::Item::Struct(item) => self.recognize_struct(item),

            syn::Item::Enum(item) => self.recognize_enum(item),

            syn::Item::Fn(item) => self.recognize_fn(item),

            syn::Item::Mod(item) => self.recognize_mod(item),

            syn::Item::Trait(item) => self.recognize_trait(item),

            syn::Item::Type(item) => self.recognize_type(item),

            syn::Item::Use(item) => self.recognize_use(item),

            _ => Err(crate::Error::UnsupportedItem(item.span())),
        }
    }

    fn recognize_struct(&mut self, item: &'arena syn::ItemStruct) -> crate::Result<()> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        let name = util::recognize_name(&item.ident);
        let qname = self.module_name.join(&name);

        if item.generics.params.len() > 0 {
            return Err(Error::GenericsNotPermitted(item.generics.span()));
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
            self.parser.definitions.insert(
                qname,
                Definition {
                    module: self.ast,
                    kind: DefinitionKind::Record(item),
                },
            );
            Ok(())
        } else if public_fields == 0 {
            // All private fields, this is a class
            self.parser.definitions.insert(
                qname,
                Definition {
                    module: self.ast,
                    kind: DefinitionKind::Resource(item),
                },
            );
            Ok(())
        } else {
            // Some public, some private fields -- error.

            Err(Error::MixedPublicPrivateFields(item.span()))
        }
    }

    fn recognize_enum(&mut self, item: &'arena syn::ItemEnum) -> crate::Result<()> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        if item.generics.params.len() > 0 {
            return Err(Error::GenericsNotPermitted(item.generics.span()));
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
            self.parser.definitions.insert(
                qname,
                Definition {
                    module: self.ast,
                    kind: DefinitionKind::Variant(item, unignored_variants),
                },
            );
            Ok(())
        } else {
            self.parser.definitions.insert(
                qname,
                Definition {
                    module: self.ast,
                    kind: DefinitionKind::Enum(item, unignored_variants),
                },
            );
            Ok(())
        }
    }

    fn recognize_fn(&mut self, item: &'arena syn::ItemFn) -> crate::Result<()> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        let name = util::recognize_name(&item.sig.ident);
        let qname = self.module_name.join(&name);

        if item.sig.generics.params.len() > 0 {
            return Err(Error::GenericsNotPermitted(item.sig.generics.span()));
        }

        self.parser.definitions.insert(
            qname,
            Definition {
                module: self.ast,
                kind: DefinitionKind::Function(item),
            },
        );
        Ok(())
    }

    fn recognize_mod(&self, item: &syn::ItemMod) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(crate::Error::UnsupportedItem(item.span()))
    }

    fn recognize_trait(&self, item: &syn::ItemTrait) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(crate::Error::UnsupportedItem(item.span()))
    }

    fn recognize_type(&self, item: &syn::ItemType) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(crate::Error::UnsupportedItem(item.span()))
    }

    fn recognize_use(&self, item: &syn::ItemUse) -> Result<(), Error> {
        if util::ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        Err(crate::Error::UnsupportedItem(item.span()))
    }
}
