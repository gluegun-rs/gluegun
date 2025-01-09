# GlueGun

> This is a **README FROM THE FUTURE**, in that it described the workflow for something that doesn't exist yet.

## Write once, Rust anywhere

**GlueGun** is a project for authoring pure-Rust libraries that can be integrated into any language on any operating system or environment. *GlueGun* can help you...

* publish a cross-language API usable from Java/Kotlin, Python, Swift, JavaScript, C, C++, and Go (and adding more languages is easy);
* package up common code for use across mobile devices.

## Just write an idiomatic Rust API and let GlueGun do the rest

Using GlueGun starts by writing an ordinary Rust library, like this one:

```rust
#[derive(Copy, Clone)]
pub enum Class {
    Fighter,
    Wizard,
    Rogue,
    Cleric,
}
pub struct Character {
    name: String,
    class: Class,
    level: u32,
}

impl Character {
    pub fn new(name: &str, class: Class) -> Self {
        Character {
            name: name.to_string(),
            class,
            level: 1,
        }
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn level_up(&mut self) {
        self.level += 1;
    }
    
    pub fn level(&self) -> u32 {
        self.level
    }
}
```

With GlueGun, you can automatically generate Rust projects that wrap this library for use from other languages.
GlueGun recognizes the patterns in your code and tries to create idiomatic translations.
For example, the Java library might look like

```java
public enum Class { Fighter, Wizard, Rogue, Cleric }
public class Character {
    public Character(String name, Class klass) {
        /* invokes Rust code via JNI */
    }

    public Class class() { /* invokes Rust code via JNI */ }
    public String name() { /* invokes Rust code via JNI */ }
    public void levelUp() { /* invokes Rust code via JNI */ }
    public int level() { /* invokes Rust code via JNI */ }
}
```

whereas the Python library would model `Class` as a [Python enum](https://docs.python.org/3/library/enum.html) and a Python class.

Intrigued? Read more in our [tutorial](./tutorial.md).

## Pick your poison

GlueGun can be used in several modes:

| Mode                              | Performance | Sandboxing | Distribution                                    |
| --------------------------------- | ----------- | ---------- | ----------------------------------------------- |
| Raw FFI                           | 😎 Native   | ⚠️ No       | Binary artifact built against libc              |
| Portable FFI                      | 😎 Native   | ⚠️ No       | Portable binary artifact usable on Mac/Windows/Linux (via [cosmopolitan][]) |
| Sandboxed FFI                     | Good        | ✅ Yes!    | Binary artifact                        |
| WebAssembly component             | Good        | ✅ Yes!    | WASM module running anywhere, including the web |

Here are the key differences between the modes

* Raw FFI -- compiles Rust to native code and creates the relevant FFI code to invoke that from the target language
* Portable FFI -- as above, but the Rust library is built as a binary artifact with [cosmopolitan][]. This allows is to be distributed to Windows, Mac, and linux equally well.
* Sandboxed FFI -- compiles Rust to WebAssembly and then uses [rlbox.dev][] to compile that to native code. Indirecting through WebAssembly costs some performance (typically around 10%) but gives the benefit of sandboxing. This means that the Rust code can be treated as untrusted by the host application.
* WebAssembly component -- compiles Rust to WebAssembly. This comes with a slight performance hit but offers sandboxing and means that you can distribute one binary that runs everywhere (including in browsers!).

[rlbox.dev]: https://rlbox.dev/
[cosmopolitan]: https://github.com/jart/cosmopolitan
