# Carcin: Write once, Rust anywhere

> This is a **README FROM THE FUTURE**, in that it described the workflow for something that doesn't exist yet.

**Carcin** is a project for authoring pure-Rust libraries that can be integrated into any language on any operating system or environment. *Carcin* can help you...

* publish a cross-language API for accessing your cloud service;
    * *carcin* currently supports Java/Kotlin, Python, JavaScript, C, C++, and Go, but adding new languages is easy.
* package up common code for use across mobile devices.

Want to see how easy *carcin* can be? Check out our [tutorial](./tutorial.md).

## Pick your poison

*carcin* can be used in three modes:

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
