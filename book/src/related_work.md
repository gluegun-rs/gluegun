# Related Work

There are many great alternatives out there aiming at a similar set of problems.

## General purpose binding generators

GlueGun's distinguishing characteristic is its focus on convention-over-configuration and covering the whole workflow. Our goal is that no annotations are needed for the common case and that custom plugins are available and feel natural to use.

Mozilla's [UniFFI](https://mozilla.github.io/uniffi-rs/) has a similar design. It uses either proc macros or an external IDL file ("UDL") to specify the interface. It is focused on supporting phone deployment but includes some other languages.

[Diplomat](https://rust-diplomat.github.io/book/) was initially built for ICU. It has some support for other languages. External tools are possible but not integrated into the primary workflow.

## Language specific bindings

* `cxx`
* `duchess`
* `pyo3`
* `cbindgen`
* `bindgen`

