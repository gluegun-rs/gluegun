use accessors_rs::Accessors;
use convert_case::{Case, Casing};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
};

use crate::{Error, Span, Ty};

#[serde_as]
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Idl {
    /// The name of the crate whose API is being bound to some other language.
    pub(crate) crate_name: Name,

    /// A list of definitions to be exported. Each of them will be located within the crate in question.
    #[serde_as(as = "Vec<(_, _)>")]
    pub(crate) definitions: BTreeMap<QualifiedName, Item>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[accessors(get)]
pub struct QualifiedName {
    pub(crate) names: Vec<Name>,
}

impl From<&Name> for QualifiedName {
    fn from(name: &Name) -> Self {
        QualifiedName::new(vec![name.clone()])
    }
}

impl QualifiedName {
    /// Return a version of the qualified name joined by `.`
    pub fn dotted(&self) -> String {
        self.to_string(".")
    }

    /// Return a string version joined by the given `sep`.
    pub fn to_string(&self, sep: &str) -> String {
        Itertools::intersperse(self.names.iter().map(|name| name.text.as_str()), sep)
            .collect::<String>()
    }

    /// Convert all names to "camelCase".
    pub fn camel_case(&self) -> QualifiedName {
        let names = self
            .names
            .iter()
            .map(|name| Name::from(name.text.to_case(Case::Camel)))
            .collect();
        QualifiedName { names }
    }

    /// Convert all names to "UpperCamelCase".
    pub fn upper_camel_case(&self) -> QualifiedName {
        let names = self
            .names
            .iter()
            .map(|name| Name::from(name.text.to_case(Case::UpperCamel)))
            .collect();
        QualifiedName { names }
    }

    /// Create a qualified name from a vector
    pub(crate) fn new(names: Vec<Name>) -> Self {
        QualifiedName { names }
    }

    /// Returns a version of `self` with a new name appended to the end.
    pub fn join(&self, name: impl Into<Name>) -> Self {
        let mut names = self.names.clone();
        names.push(name.into());
        QualifiedName { names }
    }

    /// Last name
    pub fn tail_name(&self) -> Name {
        self.names.last().unwrap().clone()
    }

    /// The qualified name minus its last component
    pub fn split_module_name(&self) -> (QualifiedName, Name) {
        assert!(!self.names.is_empty());
        (self.module_name(), self.tail_name())
    }

    /// The qualified name minus its last component
    pub fn module_name(&self) -> QualifiedName {
        assert!(!self.names.is_empty());
        QualifiedName {
            names: self.names[0..self.names.len() - 1].to_vec(),
        }
    }

    /// Set the name to whatever the module is that contains `name` (removes the last item;
    /// errors if `name` is empty).
    pub(crate) fn set_to_module_of(&mut self, name: &QualifiedName) {
        assert!(!name.names.is_empty());
        let len = name.names.len();
        self.names.extend(name.names.iter().take(len - 1).cloned());
    }

    /// Clear to an empty list
    pub(crate) fn clear(&mut self) {
        self.names.clear();
    }

    pub(crate) fn just_crate(&self) -> QualifiedName {
        QualifiedName::new(vec![self.names[0].clone()])
    }
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[accessors(get)]
pub struct Name {
    pub(crate) text: String,
}

impl Name {
    pub(crate) fn from_ident(ident: &syn::Ident) -> Self {
        Name {
            text: ident.to_string(),
        }
    }

    /// Convert name to "camelCase".
    pub fn camel_case(&self) -> Name {
        Name {
            text: self.text.to_case(Case::Camel),
        }
    }

    /// Convert name to "UpperCamelCase".
    pub fn upper_camel_case(&self) -> Name {
        Name {
            text: self.text.to_case(Case::UpperCamel),
        }
    }

    pub fn output() -> Self {
        Self::from("Output")
    }
}

impl AsRef<Name> for Name {
    fn as_ref(&self) -> &Name {
        self
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name {
            text: s.to_string(),
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name { text: s }
    }
}

impl From<&Name> for Name {
    fn from(name: &Name) -> Self {
        Name {
            text: name.text.clone(),
        }
    }
}

impl TryFrom<&OsStr> for Name {
    type Error = crate::Error;

    fn try_from(value: &OsStr) -> crate::Result<Self> {
        let Some(s) = value.to_str() else {
            return Err(Error::NotUtf8(value.to_owned()));
        };

        Ok(Name {
            text: s.to_string(),
        })
    }
}

impl TryFrom<OsString> for Name {
    type Error = crate::Error;

    fn try_from(value: OsString) -> crate::Result<Self> {
        Name::try_from(&value[..])
    }
}

/// Module item.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Item {
    /// A *Resource* is a structure with opaque contents and methods.
    /// It typically maps to a class or something like it.
    Resource(Resource),

    /// A *Record* is a structure with a known (and fixed) set of fields and types.
    /// It should map to a value type if that is available.
    Record(Record),

    /// A *Variant* is corresponds to a general Rust enum.
    /// It should map to a value type if that is available.
    Variant(Variant),

    /// An *Enum* is corresponds to a C-like Rust enum.
    /// It should map to a value type if that is available.
    Enum(Enum),

    /// A *Function* is a standalone function that can be called.
    /// Note that each of the various types can also have attached methods.
    Function(Function),
}

impl Item {
    pub fn name(&self) -> &Name {
        match self {
            Item::Resource(r) => &r.name,
            Item::Record(r) => &r.name,
            Item::Variant(v) => &v.name,
            Item::Enum(e) => &e.name,
            Item::Function(f) => &f.name,
        }
    }
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Function {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,
    /// Name in Rust syntax, like `crate::foo::bar`, relative
    pub(crate) name: Name,
    pub(crate) signature: Signature,
}

/// A *Resource* is a structure with opaque contents and methods.
/// It typically maps to a class or something like it.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Resource {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,
    pub(crate) name: Name,
    pub(crate) methods: Vec<Method>,
}

/// A *Variant* is corresponds to a general Rust enum.
/// It should map to a value type if that is available.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Variant {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,
    pub(crate) name: Name,
    pub(crate) arms: Vec<VariantArm>,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct VariantArm {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,
    pub(crate) name: Name,
    pub(crate) fields: Vec<Field>,
}

/// An *Enum* is corresponds to a C-like Rust enum.
/// It should map to a value type if that is available.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Enum {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,
    pub(crate) name: Name,
    pub(crate) arms: Vec<EnumArm>,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct EnumArm {
    pub(crate) span: Span,
    pub(crate) name: Name,
}

/// *Methods* can be attached to various types.
/// They include
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Method {
    /// Location where the item is defined.
    pub(crate) span: Span,

    /// Method category
    pub(crate) category: MethodCategory,

    /// Name of the method.
    pub(crate) name: Name,

    /// Method signature.
    pub(crate) signature: Signature,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MethodCategory {
    /// At most one method can be designated as a constructor.
    /// It creates a new instance of the type.
    /// It can be fallible.
    Constructor,

    /// Builder methods have a signature in Rust that looks like
    /// `fn method(self, ...) -> Self`. They can be treated as ordinary methods
    /// but in some cases you may wish to map types that have builder methods
    /// in some other way.
    BuilderMethod(SelfKind),

    /// Some kind of method that takes `self`, `&self`, or `&mut self`.
    /// Dealing with `&mut self` in particular can be a bit tricky, but that's on you.
    InstanceMethod(SelfKind),

    /// A method with no `self`.
    StaticMethod,
}

//// Defines a `self` parameter type
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SelfKind {
    /// `fn(self)`
    ByValue,

    /// `fn(&self)`
    ByRef,

    /// `fn(&mut self)`
    ByRefMut,
}

/// A *Record* is a structure with a known (and fixed) set of fields and types.
/// It should map to a value type if that is available.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Record {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,

    /// Name of the record.
    pub(crate) name: Name,

    /// List of fields and their types.
    pub(crate) fields: Vec<Field>,

    /// Methods attached to this record.
    pub(crate) methods: Vec<Method>,
}

/// A field in a record.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Field {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,

    /// Name of the field.
    pub(crate) name: Name,

    /// Type of the field.
    pub(crate) ty: Ty,
}

/// Signature to a function or method.
/// Excludes self.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Signature {
    /// Is this an async function?
    pub(crate) is_async: IsAsync,

    /// List of function arguments.
    pub(crate) inputs: Vec<FunctionInput>,

    /// Function return type.
    pub(crate) output_ty: FunctionOutput,
}

/// Indicates if this is an async method or not.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum IsAsync {
    No,
    Yes,
}

/// Function argument.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct FunctionInput {
    /// Span identifying this item in Rust source (currently its name).
    pub(crate) span: Span,

    /// Name of the function parameter.
    pub(crate) name: Name,

    /// Type of the function parameter.
    pub(crate) ty: Ty,
}

/// Function return type. This includes a "main" return type
/// that occurs on success and an optional "error" type that is
/// thrown on failure.
#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct FunctionOutput {
    /// Type of value returned on success.
    pub(crate) main_ty: Ty,

    /// Type of value returned on error.
    pub(crate) error_ty: Option<Ty>,
}
