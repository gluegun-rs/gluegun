# Related Work

There are many great alternatives out there aiming at a similar set of problems.

## General purpose binding generators

GlueGun's distinguishing characteristic is its focus on convention-over-configuration and covering the whole workflow. Our goal is that no annotations are needed for the common case.

Mozilla's [UniFFI](https://mozilla.github.io/uniffi-rs/) has a similar design. It uses either proc macros or an external IDL file ("UDL") to specify the interface. It is focused on supporting phone deployment but includes some 

[Diplomat](https://rust-diplomat.github.io/book/) distinguishing characteristic is its focus on convention-over-configuration and covering the whole workflow. Our goal is that no annotations are needed for the common case.

## Language specific bindings

* `cxx`
* `duchess`
* `pyo3`
* `cbindgen`
* `bindgen`

