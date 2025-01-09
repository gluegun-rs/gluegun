# WebAssembly Interface Types

The Carcin IDL can be directly mapped to [WebAssembly interface types][WIT], for the most part:

* Primitive types map directly to WIT primitive types.
* Collection types map to WIT lists:
    * A Rust `Vec<T>` or `HashSet<T>` maps to a WIT `list<T>`.
    * A Rust `HashMap<K, V>` maps to a WIT `list<tuple<K, V>>`.
* Public structs/enums are mapped to WIT records, variants, and enums as appropriate:
    * A struct is mapped to a WIT record.
    * Enums are mapped to WIT enums when possible, else WIT variants.
* Instances of the class pattern are mapped to WIT resource types:
    * The methods can be mapped directly.

[WIT]: https://component-model.bytecodealliance.org/design/wit.html

