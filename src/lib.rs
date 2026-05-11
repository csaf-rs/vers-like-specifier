// public api
pub use constraint::{Comparator, VersionConstraint, VersionConstraintError, VersionString};
pub use vls::{Vls, VlsError};

mod comparator;
mod constraint;
mod valid_chars;
mod version;
mod vls;
