# vers-like-specifier

[![crates.io](https://img.shields.io/crates/v/vers-like-specifier.svg)](https://crates.io/crates/vers-like-specifier)
[![npm](https://img.shields.io/npm/v/%40csaf-rs%2Fvers-like-specifier.svg)](https://www.npmjs.com/package/@csaf-rs/vers-like-specifier)

A Rust library for parsing and validating **VERS-like Specifiers** (vls) as defined in [CSAF 2.0](https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html#31232-branches-type---name-under-product-version-range) and [CSAF 2.1](https://docs.oasis-open.org/csaf/csaf/v2.1/csaf-v2.1.html#branches-type---name-under-product-version-range).

VERS-like specifier is the `<constraints>` portion of a [VERS](https://github.com/package-url/vers-spec) URL **without** the `vers:<type>/` prefix.

It represents a `|`-separated list of constraints each consisting of an implicit or explicit comparator and a version string.

**Due to the undefined / unknown VERS type, it is nearly impossible for tools to reliably determine whether a given version is in the range or not. vls is a fallback option and SHOULD NOT be used unless really necessary.**

## Minimum Supported Rust Version (MSRV)

1.88.0

## Installation

Add `vers-like-specifier` to your `Cargo.toml`. The package is published on https://crates.io/crates/vers-like-specifier:

```toml
[dependencies]
vers-like-specifier = "0.2"
```

## Usage

```rust
use vers_like_specifier::{Vls, Comparator, Containment};

let vls: Vls = ">=1.0.0|<2.0.0".parse().unwrap();
assert_eq!(vls.constraints().len(), 2);
assert_eq!(vls.to_string(), ">=1.0.0|<2.0.0");


let constraints = vls.constraints();
assert_eq!(constraints[0].comparator(), Comparator::GreaterThanOrEqual);
assert_eq!(constraints[0].version().to_string(), "1.0.0");
assert_eq!(constraints[1].comparator(), Comparator::LessThan);
assert_eq!(constraints[1].version().to_string(), "2.0.0");

// Because the VERS type is unknown, `contains` can only resolve a definite
// answer when a version exactly matches a constraint; otherwise it is `Undetermined`.
assert_eq!(vls.contains("1.0.0"), Containment::Contained);
assert_eq!(vls.contains("2.0.0"), Containment::NotContained);
assert_eq!(vls.contains("1.5.0"), Containment::Undetermined);

// The one exception: a vls containing only exact-match comparators (equal, not equal) is always decidable, since
// containment then reduces to a plain string comparison if there is a constraint with that version. If there isn't a
// constraint with the version, it is not contained by default. 
let exact_only: Vls = "1.0.0|!=2.0.0".parse().unwrap();
assert_eq!(exact_only.contains("1.0.0"), Containment::Contained);
assert_eq!(exact_only.contains("2.0.0"), Containment::NotContained);
assert_eq!(exact_only.contains("3.0.0"), Containment::NotContained);
```

### JavaScript / WASM Usage

Call `init()` once before using the library. In browsers it loads the wasm module; in Node.js it returns immediately, because the module is already loaded on import.

```js
import init, { Vls, Constraint, Comparator, Containment } from "@csaf-rs/vers-like-specifier";

await init();

const vls = Vls.parse(">=1.0.0|<2.0.0");
const constraint = new Constraint(Comparator.GreaterThanOrEqual, "1.0.0");
console.log(constraint.version.toString()); // "1.0.0"

console.log(vls.contains("1.0.0") === Containment.Contained); // true (matches the >= constraint exactly)
console.log(vls.contains("2.0.0") === Containment.NotContained); // true (2.0.0 < 2.0.0 is always false)
console.log(vls.contains("1.5.0") === Containment.Undetermined); // true (can't be resolved; VERS type is unknown)

try {
  Vls.parse("*");
} catch (e) {
  console.error(e); // Error: '*' (VERS syntax for matching all versions) is not allowed as a vls string
}
```

## Browser Playground

The demo folder contains a minimal browser page that uses the WASM build to parse a vls string and check whether a version is contained in it. A live version is published at https://csaf-rs.github.io/vls/.

### Run it locally

Requires [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) and the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`).

From the repository root:

```sh
demo/run.sh
```

This builds the WASM package and serves the demo at http://localhost:8080. To use another port, set `PORT`, for example `PORT=9000 demo/run.sh`.

## Benchmarks

Parsing benchmarks use [Criterion](https://github.com/bheisler/criterion.rs):

```sh
cargo bench
```

The `vls_parse` group measures parsing throughput for 100 to 100,000 constraints.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
