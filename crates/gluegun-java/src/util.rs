use std::path::PathBuf;

use gluegun_core::idl::{Name, QualifiedName};

/// A qualified name following Java conventions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct JavaQName {
    /// like `java.lang`
    pub(crate) package: QualifiedName,

    /// like `String`
    pub(crate) class_name: Name,
}

/// Convert a qualified name from Rust to Java conventions and break apart the module/class name
pub(crate) fn class_package_and_name(qname: &QualifiedName) -> JavaQName  {
    let (module_name, type_name) = qname.camel_case().split_module_name();
    JavaQName {
        package: module_name,
        class_name: type_name.upper_camel_case(),
    }
}

/// Return a path like `java/lang/String.java`
pub(crate) fn class_file_name(qname: &QualifiedName) -> PathBuf {
    let JavaQName { package, class_name } = class_package_and_name(qname);
    let mut path = PathBuf::new();
    for name in package.names() {
        path.push(name.text());
    }
    path.push(class_name.text());
    path.set_extension("java");
    path
}

/// Return a string like `java.lang.String`
pub(crate) fn class_dot_name(qname: &QualifiedName) -> String {
    let JavaQName { package, class_name } = class_package_and_name(qname);
    format!("{}.{}", package.dotted(), class_name)
}
