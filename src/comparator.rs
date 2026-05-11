//! Comparator type for the csaf-rs/vls library.
//!
//! The `Comparator` enum represents the different types of comparators that can be used
//! in version constraints, such as = (implicit or explicit), !=, <, <=, >, and >=.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use strum::AsRefStr;

/// Comparator for version constraints.
///
/// This enum represents the different types of comparators that can be used
/// in version constraints. Each comparator defines how a version is compared
/// to the constraint version.
///
/// # Equality
///
/// [`Comparator::EqualImplicit`] and [`Comparator::EqualExplicit`] are considered
/// equal by [`PartialEq`] (i.e. `EqualImplicit == EqualExplicit` is `true`).
///
/// However, their [`Display`] representations differ: `EqualImplicit` formats as `""` (empty string)
/// while `EqualExplicit` formats as `"="`.
///
/// If you need to distinguish between the two variants, use pattern matching to compare the enums or
/// use [`Comparator::is_same_variant()`] instead of an equality check.
#[derive(Debug, Clone, Copy, AsRefStr)]
pub enum Comparator {
    /// Implicit equal - The version must be exactly equal to the constraint version.
    #[strum(serialize = "")]
    EqualImplicit,
    /// Explicit equal (=) - The version must be exactly equal to the constraint version.
    #[strum(serialize = "=")]
    EqualExplicit,
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

/// [`EqualImplicit`](Comparator::EqualImplicit) and
/// [`EqualExplicit`](Comparator::EqualExplicit) are treated as equal because they
/// carry the same semantic meaning ("exactly this version"). All other variants
/// are compared by discriminant.
impl PartialEq for Comparator {
    fn eq(&self, other: &Self) -> bool {
        if self.is_equal() && other.is_equal() {
            return true;
        }
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Eq for Comparator {}

impl Hash for Comparator {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let disc: u8 = match self {
            Comparator::EqualImplicit | Comparator::EqualExplicit => 0,
            Comparator::NotEqual => 1,
            Comparator::LessThan => 2,
            Comparator::LessThanOrEqual => 3,
            Comparator::GreaterThan => 4,
            Comparator::GreaterThanOrEqual => 5,
        };
        disc.hash(state);
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.as_ref())
    }
}

impl Comparator {
    /// Returns true if the comparator represents equality (implicit or explicit)
    pub const fn is_equal(&self) -> bool {
        matches!(self, Comparator::EqualImplicit | Comparator::EqualExplicit)
    }

    /// Returns `true` if `self` and `other` are the exact same variant.
    ///
    /// This differs from [`PartialEq`], which treats [`EqualImplicit`](Comparator::EqualImplicit)
    /// and [`EqualExplicit`](Comparator::EqualExplicit) as equal.
    pub fn is_same_variant(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }

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
        } else if let Some(stripped) =
            constraint_str.strip_prefix(Comparator::EqualExplicit.as_ref())
        {
            (Comparator::EqualExplicit, stripped)
        } else {
            (Comparator::EqualImplicit, constraint_str)
        }
    }
}
