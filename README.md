# GlueGun

> This is a **README FROM THE FUTURE**, in that it described the workflow for something that doesn't exist yet.

## Write once, Rust anywhere

**GlueGun** is a project for authoring pure-Rust libraries that can be integrated into any language on any operating system or environment. *GlueGun* can help you...

* publish a cross-language API for accessing your cloud service;
    * *gluegun* currently supports Java/Kotlin, Python, JavaScript, C, C++, and Go, but adding new langauges is easy.
* package up common code for use across mobile devices.

*gluegun* can be used in three modes:

| Mode                              | Performance | Sandboxing | Distribution                                    |
| --------------------------------- | ----------- | ---------- | ----------------------------------------------- |
| Raw FFI                           | 😎 Native  | ⚠️ No    | Portable binary artifact                        |
| Sandboxed FFI using [rlbox.dev][] | Good        | ✅ Yes!    | Portable binary artifact                        |
| WebAssembly component             | Good        | ✅ Yes!    | WASM module running anywhere, including the web |

Here are the key differences between the modes

* Raw FFI -- compiles Rust to native code and packages the library as a binary artifact with [cosmpolitan][]. This offers the best performance, particularly for libraries that make good use of SIMD, but (a) means that you have to distribute a binary artifact, which can be a hassle; and (b) does not offer allow the library to be sandboxed.
* Sandboxed FFI -- compiles Rust to WebAssembly and then uses [rlbox.dev][] to compile that to native code. Indirecting through WebAssembly costs some performance (typically around 10%) but gives the benefit of sandboxing. This means that the Rust code can be treated as untrusted by the host application.
* WebAssembly component -- compiles Rust to WebAssembly. This comes with a slight performance hit but offers sandboxing and means that you can distribute one binary that runs everywhere (including in browsers!).

[rlbox.dev]: https://rlbox.dev/
[cm]: 
[cosmopolitan]: https://github.com/jart/cosmopolitan

## How gluegun works

You start by creating a Rust library whose public interfaces follows the *gluegun* conventions, which means that you stick to Rust types and features that can readily be translated across languages. The body of those functions can make use of whatever logic you want. For example, suppose you wanted to publish some logic based on Rust's best-in-class [regex][] library. You might write:

```rust
pub fn find_username(s: &str) -> String {
    let r = regex::compile("@([a-zA-Z]+)").unwrap();
    if let Some(m) = r.captures(s) {
        m.to_string()
    } else {
        panic!("no username found")
    }
}
```

You would then install and run `gluegun`:

```bash
> cargo install gluegun
> cargo gluegun build
```

Since you don't have a `gluegun.toml`, you'll be asked a few questions, and then gluegun will run. The result will be a set of libraries that allow your code to be used transparently from other languages. You can also run `cargo gluegun setup` if you prefer to just run the setup commands and not do the actual build.

## More advanced Rust code

The `find_username` function is fairly basic. `gluegun` supports more advanced interfaces as well.

### Public item types

gluegun works by parsing your `lib.rs` module to determine your public interface. It only allows the following kinds of `pub` items:

* `pub fn` to define a public function.
* `pub struct` or `pub enum` to define a public struct, enum, or class (see below).
* `pub use crate::some::path` to publish some part of your crate.

You will get an error if you have other public items in your `lib.rs` because *gluegun* does not know how to translate them to a public API. If you wish to include them anyway, you can tag them with the `#[gluegun::ignore]` attribute. This will cause them to be ignored, which means that they will only be available to Rust consumers of your library.

### Basic Rust types

You can use the following built-in Rust types in your public interfaces:

* numeric scalar types like `i8`, `u16`, `f32` up to 64 bits;
* `char`;
* `&str` and `String`;
* Slices (`&[T]`) and vectors (`Vec<T>`), where `T` is some other supported type;
* Maps (`HashMap`, `BTreeMap`, `IndexMap`) and sets (`HashSet`, `BTreeSet`, `IndexSet`);
* Options `Option<T>` and results `Result<T, U>`;
* tuples.

Function parameters can also be `&`-references to the above types, e.g., `&HashSet<String>`
(in fact, this is recommended unless ownership is truly required).

### Simple structs and enums

You can define public structs and enums:

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

### "Classes" (types with methods)

```rust
/// Translated to a WebAssembly [resource][]
/// 
/// [record]: https://component-model.bytecodealliance.org/design/wit.html#records
pub struct MyResource {
    field: T,
}

impl MyResource {
    pub fn new() -> Self {

    }

    pub fn method1(&self) {

    }

    pub fn static_method1(&self) {

    }
}
```

## WebAssembly

## Configuration

## Frequently asked questions

### Why the name gluegun?

The name *gluegun* comes from the idea that this package enables clean interop between various languages. Ordinarily that would require N^2 different bits of code, but since *gluegun* leverages WebAssembly's [interface types][wit], we can enable interop with just one.

[wit]: https://component-model.bytecodealliance.org/design/wit.html
