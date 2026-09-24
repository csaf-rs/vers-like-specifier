//! Comparator type for the csaf-rs/vls library.
//!
//! The `Comparator` enum represents the different types of comparators that can be used
//! in constraints.

use std::fmt::{Display, Formatter, Result as FmtResult};
use strum::AsRefStr;

/// Comparator for constraints.
///
/// This enum represents the different types of comparators that can be used
/// in constraints. Each comparator defines how a version is compared
/// to the constraint version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr)]
pub enum Comparator {
    /// Equal (implicit) - The version must be exactly equal to the constraint version.
    #[strum(serialize = "")]
    Equal,
    /// Not equal (!=) - The version must not be equal to the constraint version.
    #[strum(serialize = "!=")]
    NotEqual,
    /// Less than (<) - The version must be less than the constraint version.
    #[strum(serialize = "<")]
    LessThan,
    /// Less than or equal (<=) - The version must be less than or equal to the constraint version.
    #[strum(serialize = "<=")]
    LessThanOrEqual,
    /// Greater than (>) - The version must be greater than the constraint version.
    #[strum(serialize = ">")]
    GreaterThan,
    /// Greater than or equal (>=) - The version must be greater than or equal to the constraint version.
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.as_ref())
    }
}

impl Comparator {
    /// Extracts a comparator from a constraint string.
    ///
    /// Returns a tuple of the matched [`Comparator`] and the remaining version string.
    /// Contains the implicit "parsing order" of the comparators:
    /// * gte/lte comparators need to take precedence over the gt/lt comparators
    /// * implicit eq needs to come last / be the fallthrough
    pub fn extract_comparator(constraint_str: &str) -> (Comparator, &str) {
        if let Some(stripped) = constraint_str.strip_prefix(Comparator::GreaterThanOrEqual.as_ref())
        {
            (Comparator::GreaterThanOrEqual, stripped)
        } else if let Some(stripped) =
            constraint_str.strip_prefix(Comparator::LessThanOrEqual.as_ref())
        {
            (Comparator::LessThanOrEqual, stripped)
        } else if let Some(stripped) = constraint_str.strip_prefix(Comparator::NotEqual.as_ref()) {
            (Comparator::NotEqual, stripped)
        } else if let Some(stripped) = constraint_str.strip_prefix(Comparator::GreaterThan.as_ref())
        {
            (Comparator::GreaterThan, stripped)
        } else if let Some(stripped) = constraint_str.strip_prefix(Comparator::LessThan.as_ref()) {
            (Comparator::LessThan, stripped)
        } else {
            (Comparator::Equal, constraint_str)
        }
    }
}
