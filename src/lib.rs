//! # vls — vers-like specifier parser
//!
//! `vls` is a parser for **vers-like specifiers** (vls), the `<version-constraint>` part
//! of a [vers](https://github.com/package-url/vers-spec) URL *without* the
//! `vers:<scheme>/` prefix.
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Vls`] | At least one [`VersionConstraint`] |
//! | [`VersionConstraint`] | A [`Comparator`] / [`VersionString`] pair |
//! | [`Comparator`] | A comparator used by [`VersionConstraint`] |
//! | [`VersionString`] | A validated version string, used by [`VersionConstraint`] |
//!
//! ## Error Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`VlsError`] | Returned when parsing a vls string fails |
//! | [`VersionConstraintError`] | Returned when parsing a single [`VersionConstraint`] fails |

pub use constraint::{Comparator, VersionConstraint, VersionConstraintError, VersionString};
pub use vls::{Vls, VlsError};

mod comparator;
mod constraint;
mod valid_chars;
mod version;
mod vls;
