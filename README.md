# GlueGun

> This is a **README FROM THE FUTURE**, in that it described the workflow for something that doesn't exist yet.

## Write once, Rust anywhere

**GlueGun** is a project for authoring pure-Rust libraries that can be integrated into any language on any operating system or environment. *GlueGun* can help you...

* publish a cross-language API usable from Java/Kotlin, Python, Swift, JavaScript, C, C++, and Go (and adding more languages is easy);
* package up common code for use across mobile devices.

GlueGun is highly configurable. The core GlueGun project includes several backends but it's easy to write your own -- or use backends written by others and published to crates.io.

## Just write an idiomatic Rust API and let GlueGun do the rest

Using GlueGun starts by writing an ordinary Rust library. GlueGun will scan the public interface of this library and attempt to identify generic patterns that can be ported across languages. As much as possible we try to have you document your intentions by using Rust idioms.

For example, maybe you are building a core library for a role-playing game. You are going to export this library for a number of languages, including Java. You might start with a struct that represents a character:

```rust
// rpg/src/lib.rs

pub struct Character {
    name: String,
    level: u32,
}

impl Character {
    // ...
}
```

A public struct with private fields is called a *resource* in GlueGun -- the names are taken from [WebAssembly Interface Types][WIT]. Resources map to classes in most languages. The methods on the class are taken from what appears in the Rust `impl` block:

```rust
pub struct Character { ... }

impl Character {
    pub fn new(name: impl AsRef<str>) -> Self {
        let name: &str = name.as_ref();
        Character {
            name: name.to_string(),
            level: 1,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn level_up(&mut self) -> anyhow::Result<()> {
        if self.level == 20 {
            anyhow::bail!("character has reached maximum level");
        }
        self.level += 1;
        Ok(())
    }
}
```

You could then run `cargo gluegun python` to generate Python bindings for `Character`. This will define a Rust project using `pyo3` to create a Python class `Character` with the same methods. Rust idioms like `AsRef` are understood and translated appropriately; `Result` return types are translated into Python exceptions.

```python
class Character:
    def __init__(name):
        # invokes `Character::new` in Rust via native code

    def name(self):
        # invokes `Character::new` in Rust via native code

    def level(self):
        # invokes `Character::new` in Rust via native code

    def level_up(self):
        # invokes `Character::new` in Rust via native code
```

Of course you can create more than Python. You could also do `cargo gluegun java` for Java code or `cargo gluegun cpp` for C++ code.

To see a more complex example, check out the [tutorial](./tutorial.md).

## Open-ended

GlueGun ships with many common languages built-in, but you can easily extend it just by adding a new executable.
You can add languages to GlueGun simply by installing a new executable.
When you run `cargo gluegun some_id`, it will search for `gluegun-some_id`, even installing it from crates.io if needed.

