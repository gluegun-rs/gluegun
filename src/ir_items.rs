use std::collections::BTreeMap;

use accessors_rs::Accessors;
use serde::{Deserialize, Serialize};

use crate::Ty;

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Universe {
    pub(crate) definitions: BTreeMap<QualifiedName, Item>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[accessors(get)]
pub struct QualifiedName {
    pub(crate) names: Vec<Name>,
}

impl QualifiedName {
    pub(crate) fn empty() -> Self {
        Self::new(vec![])
    }

    pub(crate) fn new(names: Vec<Name>) -> Self {
        QualifiedName { names }
    }

    pub(crate) fn join(&self, name: &Name) -> Self {
        let mut names = self.names.clone();
        names.push(name.clone());
        QualifiedName { names }
    }

    pub fn tail_name(&self) -> Name {
        self.names.last().unwrap().clone()
    }

    /// Set the name to whatever the module is that contains `name` (removes the last item;
    /// errors if `name` is empty).
    pub(crate) fn set_to_module_of(&mut self, name: &QualifiedName) {
        assert!(!self.names.is_empty());
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
}

/// Module item.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Item {
    Resource(Resource),
    Record(Record),
    Variant(Variant),
    Enum(Enum),
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
    /// Name in Rust syntax, like `crate::foo::bar`, relative
    pub(crate) name: Name,
    pub(crate) signature: Signature,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Resource {
    pub(crate) name: Name,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Variant {
    pub(crate) name: Name,
    pub(crate) arms: Vec<VariantArm>,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct VariantArm {
    pub(crate) name: Name,
    pub(crate) fields: Vec<Ty>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Enum {
    pub(crate) name: Name,
    pub(crate) arms: Vec<EnumArm>,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct EnumArm {
    pub(crate) name: Name,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Method {
    /// Method category
    pub(crate) category: MethodCategory,

    /// Name of the method.
    pub(crate) name: Name,

    /// Method signature.
    pub(crate) signature: Signature,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MethodCategory {
    Constructor,
    BuilderMethod(SelfKind),
    InstanceMethod(SelfKind),
    StaticMethod,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum SelfKind {
    ByValue,
    ByRef,
    ByRefMut,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Record {
    pub(crate) name: Name,
    pub(crate) fields: Vec<Field>,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Field {
    pub(crate) name: Name,
    pub(crate) ty: Ty,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct Signature {
    pub(crate) is_async: IsAsync,
    pub(crate) inputs: Vec<FunctionInput>,
    pub(crate) output_ty: Ty,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum IsAsync {
    No,
    Yes,
}

#[derive(Accessors, Clone, Debug, Serialize, Deserialize)]
#[accessors(get)]
pub struct FunctionInput {
    pub(crate) name: Name,
    pub(crate) ty: Ty,
}

