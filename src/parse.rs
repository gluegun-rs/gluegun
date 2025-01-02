use std::{collections::BTreeMap, path::{Path, PathBuf}, str::FromStr};

use syn::spanned::Spanned;

use crate::{Module, Universe};

pub struct Parser {
    universe: Universe,
    crate_contents: BTreeMap<PathBuf, Module>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            universe: Universe { module: Module { items: vec![] } },
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

    fn recognize_struct(&mut self, item: &syn::ItemStruct) -> crate::Result<()> {
        if self.ignore(&item.vis, &item.attrs) {
            return Ok(());
        }

        if item.generics.params.len() > 0 {
            return Err(crate::Error::GenericsNotPermitted(item.generics.span()));
        }


    }

    // Given a struct name like `Foo`, 
    fn find_inherent_impls(&self, struct_name: &str) -> Vec<&'p syn::ItemImpl> {
        self.ast.items.iter().filter_map(|item| {
            if let syn::Item::Impl(item_impl) = item {
                Some(item_impl)
            } else {
                None
            }
        }).filter(|item_impl| item_impl.trait_.is_none())
        .filter(|item_impl| {
            if let syn::Type::Path(path) = &*item_impl.self_ty {
                path.path.is_ident(struct_name)
            } else {
                false
            }
        })
        .collect()
    }

    fn ignore(&self, vis: &syn::Visibility, attrs: &[syn::Attribute]) -> bool {
        // Only look at public things
        if !self.is_public(vis) {
            return true;
        }

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