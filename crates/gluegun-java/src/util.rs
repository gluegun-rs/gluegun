use std::path::PathBuf;

use gluegun_core::idl::QualifiedName;

pub(crate) fn class_file_name(qname: &QualifiedName) -> PathBuf {
    let mut path = PathBuf::new();
    for name in qname.camel_case().names() {
        path.push(name.text());
    }
    path.set_extension("java");
    path
}

pub(crate) fn class_dot_name(qname: &QualifiedName) -> String {
    let (module_name, type_name) = qname.split_module_name();
    format!("{}.{}", module_name.dotted(), type_name)
}
