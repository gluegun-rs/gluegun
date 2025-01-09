# Mapping to C

The GlueGun IDL is mapped to Java as follows:

* Primitive types map to C in the obvious ways
* Enums map without associated data map to Java enums
* For all other types, we generate C wrappers of various kinds, as described below

## Collections

We will create new struct types for each collection as needed.

These will include helper methods to:

* create an empty collection and insert new elements;
* read the "length" of the collection;
* iterate over the collection;
* for vectors, access the ith member or treat the collection as a C array;
* for sets, access the ith member or treat the collection as a C array;
* for maps, lookup the element for a key.