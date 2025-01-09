# Defining your public interface

*squared* works by parsing your `lib.rs` module to determine your public interface. It only allows the following kinds of `pub` items:

* `pub fn` to define a public function.
* `pub struct` or `pub enum` to define a public struct, enum, or class (see below).
* `pub use crate::some::path` to publish some part of your crate.


## Public functions

You can declare top-level Rust functions:

```rust
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
```

The argument and return types of these functions have to consist of [translatable Rust types](#translatable-rust-types).

## Structs defined with the "class" pattern

*Squared* recognizes the common Rust idiom of a public struct with private members and public methods defined in an `impl` block. This pattern is called the *class pattern* and, for OO languages, it will be translated into a class.

```rust
pub struct MyClass {
    // Fields must be private
    field1: Field1
}

impl MyClass {
    /// If you define a `new` function, it becomes the constructor.
    /// Classes can have at most one constructor.
    pub fn new() -> Self {}

    /// Classes can only have `&self` methods.
    pub fn method(&self) {}

    /// Classes can also have "static" methods with no `self`.
    pub fn static_method() {}
}
```

## Public structs and enums

You can define public structs and enums.
The contents of these types must be fully public, which also means you are committed to not changing them.

```rust
/// Translated to a WebAssembly [record][]
/// 
/// [record]: https://component-model.bytecodealliance.org/design/wit.html#records
pub struct MyStruct {
    pub field: T,
}

/// Enums with no values are translated to a WebAssembly enum,
/// which means they will be represented in target languages as
/// the native enum construct.
pub enum MySimpleEnum {
    Variant1,
    Variant2,
}

/// Enums with no values are translated to a WebAssembly enum,
/// which means they will be represented in target languages as
/// the native variant construct.
pub enum MyComplexEnum {
    Variant1(T),
    Variant2,
}
```

## Public uses

You include a `pub use` to import things from elsewhere in your crate and include them in your public interface. You must write the `use` in absolute form:

```rust
pub use crate::path::to::Something;
```

*squared* will look for the definition of `Something` in `src/path/to.rs`.

## Private members and ignored items

Normally all public entries defined in your lib.rs must be fit one of the above categories so that *squared* knows how to translate them. You can also have arbitrary Rust code so long as the items are private to your crate.

Sometimes you would like to include public Rust members that are not part of your public interface.
You can do that by annotation those members with `#[squared::ignore]`.

## Translating Rust types

Your public functions and methods can use the following Rust types.

* numeric scalar types like `i8`, `u16`, `f32` up to 64 bits;
* `char`;
* `&str` and `String`;
* tuples, options `Option<T>` and results `Result<T, U>`;
* collection types:
    * slices (`&[T]`) and vectors (`Vec<T>`)
    * maps (`HashMap`, `BTreeMap`, `IndexMap`)
    * sets (`HashSet`, `BTreeSet`, `IndexSet`)
* user-defined types in your library:
    * [simple structs and enums](#public-structs-and-enums)
    * structs following the [class pattern](#public-classes)
* user-defined types from other squared libraries:
    * XXX importing from other libraries?

Function parameters can be `&`-references to the above types.

Function return types must be owned.

### Toll-free bridging

Using native Rust types for collections is convenient but can incur a performance cost as data must be copied out from native collections into the Rust type and vice versa. To avoid this you can use "toll-free" bridging in your Rust code: this means that you code traits defined in the [squared stdlib](./stdlib.md):

* `impl MapLike<K,V>`
* `impl VecLike<T>`
* `impl SetLike<E>`

You can also write your function to return a type `R` implementing one of those traits. Squared will recognize this pattern and pick appropriate instances of those traits for best performance. For example, for C++, `MapLike` can be instantiated with the STL maps, avoiding the need to copy data into a Rust map. In some cases multiple variants may be created (e.g., if the function is invoked multiple times).

