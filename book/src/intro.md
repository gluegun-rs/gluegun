# GlueGun: Write once, Rust anywhere

> This is a **README FROM THE FUTURE**, in that it described the workflow for something that doesn't exist yet.

> **GlueGunization** is a form of convergent evolution in which non-crab crustaceans evolve a crab-like body plan. The term was introduced into evolutionary biology by L. A. Borradaile, who described it as "the many attempts of Nature to evolve a crab". (--[Wikipedia](https://en.wikipedia.org/wiki/GlueGunisation))

**GlueGun** is a project for authoring pure-Rust libraries that can be integrated into any language on any operating system or environment. *GlueGun* can help you...

* publish a cross-language API usable from Java/Kotlin, Python, Swift, JavaScript, C, C++, and Go (and adding more languages is easy);
* package up common code for use across mobile devices.

## Just write an idiomatic Rust API and let GlueGun do the rest

Using GlueGun starts by writing a Rust library. For example, maybe we have a `Greetings` type that can generate greetings in various languages. It has a "builder-style" method `language` for setting the language along with a main `greet` method that takes and returns a string:

```rust
pub struct Greetings {
    language: String,
}

impl Greetings {
    pub fn new() -> Self {
        Self {
            language: "en".to_string(),
        }
    }

    pub fn language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    pub fn greet(self, name: String) -> anyhow::Result<String>  {
        match &self.language {
            "en" => format!("Hello, {name}!"),
            "es" => format!("Hola, {name}!"),
            _ => anyhow!("unknown language {language}"),
        }
    }
}
```

If you know Rust, the above should look pretty simple.

Intrigued? Read more in our [tutorial](./tutorial.md).

## Pick your poison

*gluegun* can be used in three modes:

| Mode                              | Performance | Sandboxing | Distribution                                    |
| --------------------------------- | ----------- | ---------- | ----------------------------------------------- |
| Raw FFI                           | 😎 Native  | ⚠️ No    | Portable binary artifact                        |
| Sandboxed FFI using [rlbox.dev][] | Good        | ✅ Yes!    | Portable binary artifact                        |
| WebAssembly component             | Good        | ✅ Yes!    | WASM module running anywhere, including the web |

Here are the key differences between the modes

* Raw FFI -- compiles Rust to native code and packages the library as a binary artifact with [cosmopolitan][]. This offers the best performance, particularly for libraries that make good use of SIMD, but (a) means that you have to distribute a binary artifact, which can be a hassle; and (b) does not offer allow the library to be sandboxed.
* Sandboxed FFI -- compiles Rust to WebAssembly and then uses [rlbox.dev][] to compile that to native code. Indirecting through WebAssembly costs some performance (typically around 10%) but gives the benefit of sandboxing. This means that the Rust code can be treated as untrusted by the host application.
* WebAssembly component -- compiles Rust to WebAssembly. This comes with a slight performance hit but offers sandboxing and means that you can distribute one binary that runs everywhere (including in browsers!).

[rlbox.dev]: https://rlbox.dev/
[cosmopolitan]: https://github.com/jart/cosmopolitan
