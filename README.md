# vls

A Rust library for parsing and validating **Vers-like Specifiers** (vls) as defined in [CSAF 2.0](https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html#31232-branches-type---name-under-product-version-range) and [CSAF 2.1](https://docs.oasis-open.org/csaf/csaf/v2.1/csaf-v2.1.html#branches-type---name-under-product-version-range).

vls is the `<constraints>` portion of a [vers](https://github.com/package-url/vers-spec) URL **without** the `vers:<type>/` prefix.

It represents a `|`-separated list of constraints each consisting of an implicit or explicit comparator and a version string.

**Due to the undefined / unknown schema, it is nearly impossible for tools to reliably determine whether a given version is in the range or not. vls is a fallback option and SHOULD NOT be used unless really necessary.**

## Minimum Supported Rust Version (MSRV)

1.88.0

## Installation

Add `vls` to your `Cargo.toml`:

```toml
[dependencies]
vls = "0.1"
```

### Note: This crate has not yet been published to [crates.io](https://crates.io). In the meantime, use a git dependency:

```toml
[dependencies]
vls = { git = "https://github.com/csaf-rs/vls" }
```

## Usage

```rust
use vls::{Vls, Comparator};

let vls: Vls = ">=1.0.0|<2.0.0".parse().unwrap();
assert_eq!(vls.constraints().len(), 2);
assert_eq!(vls.to_string(), ">=1.0.0|<2.0.0");


let constraints = vls.constraints();
assert_eq!(constraints[0].comparator(), &Comparator::GreaterThanOrEqual);
assert_eq!(constraints[0].version().to_string(), "1.0.0");
assert_eq!(constraints[1].comparator(), &Comparator::LessThan);
assert_eq!(constraints[1].version().to_string(), "2.0.0");
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
