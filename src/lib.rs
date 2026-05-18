//! # vls — vers-like specifier parser
//!
//! `vls` is a parser for **vers-like specifiers** (vls), the `<constraints>` part
//! of a [vers](https://github.com/package-url/vers-spec) URL *without* the
//! `vers:<type>/` prefix.
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Vls`] | At least one [`Constraint`] |
//! | [`Constraint`] | A [`Comparator`] / [`VersionString`] pair |
//! | [`Comparator`] | A comparator used by [`Constraint`] |
//! | [`VersionString`] | A validated version string, used by [`Constraint`] |
//!
//! ## Error Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`VlsError`] | Returned when parsing a vls string fails |
//! | [`ConstraintError`] | Returned when parsing a single [`Constraint`] fails |

pub use constraint::{Comparator, Constraint, ConstraintError, VersionString};
pub use vls::{Vls, VlsError};

mod comparator;
mod constraint;
mod valid_chars;
mod version;
mod vls;
