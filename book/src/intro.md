# GlueGun

> This is a **README FROM THE FUTURE**, in that it described the workflow for something that doesn't exist yet.

## Write once, Rust anywhere

**GlueGun** is a project for authoring pure-Rust libraries that can be integrated into any language on any operating system or environment. *GlueGun* can help you...

* publish a cross-language API usable from Java/Kotlin, Python, Swift, JavaScript, C, C++, and Go (and adding more languages is easy);
* package up common code for use across mobile devices.

GlueGun is highly configurable. The core GlueGun project includes several backends but it's easy to write your own -- or use backends written by others and published to crates.io.

## GlueGun in 30s

Imagine you have a `hello-world` Rust crate that you want to expose it to other languages:

```rust
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}
```

With GlueGun, you just run

```bash
> cargo gluegun java
```

to create a `hello-world-java` crate. You can then run

```bash
> cargo run -p hello-world-java -- jar
```

and it will create a `target/hello-world.jar` file for you to distribute. Java users can then just run `hello_world.Functions.greet("Duke")`.

Java not enough for you? Try

```bash
> cargo gluegun python
> carun run -p gluegun-py -- publish
```

and you will create a Python wrapper and publish it to PyPI. Pretty cool!

## Any language you want, and then some

GlueGun ships with support for these languages:

* Java
* Python
* C
* C++
* JavaScript
* Swift
* Go

but creating a new language binding is easy. Just create a 

## But wait, there's more!

GlueGun is designed to get you up and going as quickly as possible, but it's infinitely customizable. Perhaps you want to make a Java version that does things a bit different? Or you want to integrate with your internal build system at work? No problem at all.

GlueGun is a kind of "meta project":

* The core GlueGun parses your Rust code to extract the interface, represented in [Interface Definition Language](./idl.md).
* It then invokes a separate executable to map that IDL to some other language:
    * The GlueGun repository includes a number of languages, but you can make your own just by creating a new crate and uploading it to crates.io. No need to centrally coordinate.

