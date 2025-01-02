use accessors_rs::Accessors;

use crate::Ty;

#[derive(Accessors)]
#[accessors(get)]
pub struct Universe {
    pub(crate) module: Module,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Module {
    pub(crate) items: Vec<Item>,
}

/// Module item.
#[non_exhaustive]
pub enum Item {
    Resource(Resource),
    Record(Record),
    Function(Function),
    Module(Module),
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Function {
    /// Name in Rust syntax, like `crate::foo::bar`, relative
    pub(crate) name: RustPath,
    pub(crate) signature: Signature,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Resource {
    pub(crate) constructor: Option<Function>,

    /// We recognize "Builder field methods" as a special case.
    pub(crate) builder_field_methods: Vec<BuilderFieldMethod>,

    /// Instance and static methods.
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct BuilderFieldMethod {
    /// A "builder field method" is one that looks like
    /// * `fn name(self, ...) -> Self`
    /// * `fn name(&mut self, ...) -> &mut Self`
    /// depending on which variant of the builder pattern this method is using.
    pub(crate) self_kind: SelfKind,

    /// Name of the method.
    pub(crate) name: String,

    /// Input types. The return type will always be `Self` or `&mut Self`, depending on
    /// `self_kind`.
    pub(crate) signature: Signature,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Method {
    /// Self kind of the method. If `None`, this is a "static" method.
    pub(crate) self_kind: Option<SelfKind>,

    /// Name of the method.
    pub(crate) name: String,

    /// Method signature.
    pub(crate) signature: Signature,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SelfKind {
    ByValue,
    ByRef,
    ByRefMut,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Record {
    pub(crate) name: String,
    pub(crate) fields: Vec<Field>,
    pub(crate) methods: Vec<Method>,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Field {
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct Signature {
    pub(crate) is_async: IsAsync,
    pub(crate) inputs: Vec<FunctionInput>,
    pub(crate) outputs: Vec<Ty>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum IsAsync {
    No,
    Yes,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct FunctionInput {
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Accessors)]
#[accessors(get)]
pub struct RustPath {
    pub(crate) crate_name: String,
    pub(crate) path: Vec<String>,
}
