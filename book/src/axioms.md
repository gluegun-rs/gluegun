# Design axioms

Our design axioms are

* **Fit to the user, not the other way around.** Our goal is that you write an idiomatic Rust API and you get other languages for free, with no annotations at all.
* **Cover the whole workflow.** GlueGun should not only generate bindings but automate putting those bindings into users' hands.
* **Decentralized.** Creating new backends should not require centralized approval.

## Non-goals

* **Total control:** We are targeting APIs and libraries that are intentionally simple. We expect to cover the "least common denominator" across virtually all targets. While we do provide annotations and configuration options, we expect users who want to 