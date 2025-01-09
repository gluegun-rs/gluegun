# Mapping to Java

The Carcin IDL is mapped to Java as follows:

* Primitive types:
    * `i8`, `u8` to Java `byte`
    * `i16`, `u16` to Java `short`
    * `i32`, `u32` to Java `int`
    * `u64`, `u64` to Java `long`
    * `f32` to Java `float`
    * `f64` to Java `double`
    * `char` to Java `u32` (a Java `char` is not a 32-bit unicode code point)
* Collection types map to Java collections:
    * A Rust `Vec<T>` to a Java `ArrayList<T>`
    * ...
* Tuples and public structs map to Java classes with public fields
* Enums with associated data map to an abstract Java base class and public-struct-like subclasses for each variant
* Enums map without associated data map to Java enums
* Instances of the class pattern map to Java classes with methods