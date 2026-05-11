pub use crate::comparator::Comparator;
pub use crate::version::VersionString;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

/// A single version constraint pairing a [`Comparator`] with a validated [`VersionString`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VersionConstraint {
    comparator: Comparator,
    version: VersionString,
}

impl VersionConstraint {
    /// Returns the comparator of this constraint.
    pub fn comparator(&self) -> &Comparator {
        &self.comparator
    }

    /// Returns a reference to the [`VersionString`].
    pub fn version(&self) -> &VersionString {
        &self.version
    }
}

impl FromStr for VersionConstraint {
    type Err = VersionConstraintError;

    fn from_str(constraint_str: &str) -> Result<Self, Self::Err> {
        // Check if the constraint is empty
        if constraint_str.is_empty() {
            return Err(VersionConstraintError::EmptyConstraint);
        }

        // Match the comparators
        let (comparator, version_str) = Comparator::extract_comparator(constraint_str);

        // Parse and validate the version string (checks empty + invalid chars)
        let version: VersionString = version_str.parse()?;

        Ok(VersionConstraint {
            comparator,
            version,
        })
    }
}

impl Display for VersionConstraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.comparator, self.version)
    }
}

/// Errors specific to a single constraint within a VLS string.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum VersionConstraintError {
    /// A constraint segment was empty (e.g. from `||`, a leading `|`, or a trailing `|`).
    #[error("Empty constraint")]
    EmptyConstraint,

    /// The version part of a constraint was empty (e.g. `>=` without a version).
    #[error("Empty version in constraint")]
    EmptyVersion,

    /// The version string contains characters outside the allowed grammar.
    /// See vls::Vls for more details on the grammar.
    #[error("Invalid character(s) in version string: {}", .0.iter().map(|c| format!("'{}'", c.escape_default())).collect::<Vec<_>>().join(", "))]
    InvalidVersionCharacters(Vec<char>),
}
