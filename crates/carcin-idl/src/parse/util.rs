use crate::Name;

/// If true, ignore this item.
pub(super) fn ignore(vis: &syn::Visibility, attrs: &[syn::Attribute]) -> bool {
    // Only look at public things
    if !is_public(vis) {
        return true;
    }

    ignore_from_attrs(attrs)
}

pub(super) fn ignore_from_attrs(attrs: &[syn::Attribute]) -> bool {
    // Only look at things that are not cfg(test)
    if attrs.iter().any(|attr| attr.path().is_ident("cfg")) {
        // FIXME: check that the attribute is test
        return true;
    }

    // Ignore things tagged with `carcin::ignore`
    if attrs.iter().any(|attr| attr.path().is_ident("ignore")) {
        // FIXME: check that attribute is "carcin::ignore"
        return true;
    }

    false
}

/// Returns true if this is fully public.
/// Non-public items don't concern us.
pub(super) fn is_public(vis: &syn::Visibility) -> bool {
    match vis {
        syn::Visibility::Public(_) => true,
        syn::Visibility::Restricted(_) => false,
        syn::Visibility::Inherited => false,
    }
}

pub(super) fn recognize_name(ident: &syn::Ident) -> Name {
    Name {
        text: ident.to_string(),
    }
}
