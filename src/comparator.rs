use std::fmt::{Display, Formatter, Result as FmtResult};
use strum::AsRefStr;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// A [`Comparator`] for a vls [`Constraint`](crate::Constraint)
///
/// This enum represents the different types of comparators that can be used
/// in constraints. The comparator defines how a version is compared
/// to the constraint's [`VersionString`](crate::VersionString).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
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
    pub(crate) fn extract_comparator(constraint_str: &str) -> (Comparator, &str) {
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Comparator;

    #[rstest]
    // "good" cases
    #[case::extract_gte(">=1.2.3", Comparator::GreaterThanOrEqual, "1.2.3")]
    #[case::extract_lte("<=1.2.3", Comparator::LessThanOrEqual, "1.2.3")]
    #[case::extract_ne("!=1.2.3", Comparator::NotEqual, "1.2.3")]
    #[case::extract_gt(">1.2.3", Comparator::GreaterThan, "1.2.3")]
    #[case::extract_lt("<1.2.3", Comparator::LessThan, "1.2.3")]
    #[case::extract_eq("1.2.3", Comparator::Equal, "1.2.3")]
    // "bad" cases (comparator chars in version string)
    #[case::double_lt_equal("<<=1.2.3", Comparator::LessThan, "<=1.2.3")]
    #[case::double_gt_equal(">>=1.2.3", Comparator::GreaterThan, ">=1.2.3")]
    #[case::double_ne("!==1.2.3", Comparator::NotEqual, "=1.2.3")]
    #[case::double_lt("<<1.2.3", Comparator::LessThan, "<1.2.3")]
    #[case::double_gt(">>1.2.3", Comparator::GreaterThan, ">1.2.3")]
    #[case::double_explicit_eq("==1.2.3", Comparator::Equal, "==1.2.3")]
    #[case::single_explicit_eq("=1.2.3", Comparator::Equal, "=1.2.3")]
    // "bad" cases (only comparators, empty version string)
    #[case::gte_no_version(">=", Comparator::GreaterThanOrEqual, "")]
    #[case::lte_no_version("<=", Comparator::LessThanOrEqual, "")]
    #[case::ne_no_version("!=", Comparator::NotEqual, "")]
    #[case::gt_no_version(">", Comparator::GreaterThan, "")]
    #[case::lt_no_version("<", Comparator::LessThan, "")]
    #[case::eq_no_version("", Comparator::Equal, "")]
    fn extracts_comparator_and_remaining_version(
        #[case] constraint: &str,
        #[case] expected_comparator: Comparator,
        #[case] expected_version: &str,
    ) {
        assert_eq!(
            Comparator::extract_comparator(constraint),
            (expected_comparator, expected_version)
        );
    }
}
