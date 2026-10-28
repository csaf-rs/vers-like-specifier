//! # vls — VERS-like specifier parser
//!
//! `vls` is a parser for **VERS-like specifiers** (vls), the `<constraints>` part
//! of a [VERS](https://github.com/package-url/vers-spec) URL *without* the
//! `vers:<type>/` prefix.
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Vls`] | At least one [`Constraint`] |
//! | [`Constraint`] | A [`Comparator`] / [`VersionString`] pair |
//! | [`Comparator`] | A comparator used by [`Constraint`] |
//! | [`VersionString`] | A validated version string, used by [`Constraint`] |
//! | [`Containment`] | The tri-state result of [`Vls::contains`] |
//!
//! ## Error Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`VlsError`] | Returned when parsing a vls string fails |
//! | [`ConstraintError`] | Returned when parsing a single [`Constraint`] fails |

pub use constraint::{Comparator, Constraint, ConstraintError, VersionString};
pub use vls::{Containment, Vls, VlsError};

mod comparator;
mod constraint;
mod valid_chars;
mod version;
mod vls;
mod vls_error;

#[cfg(feature = "wasm")]
pub mod wasm;
