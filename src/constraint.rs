pub use crate::comparator::Comparator;
pub use crate::version::VersionString;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use thiserror::Error;

/// A single constraint pairing a [`Comparator`] with a validated [`VersionString`].
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Constraint {
    comparator: Comparator,
    version: VersionString,
}

impl Constraint {
    /// Returns the comparator of this constraint.
    pub fn comparator(&self) -> &Comparator {
        &self.comparator
    }

    /// Returns a reference to the [`VersionString`].
    pub fn version(&self) -> &VersionString {
        &self.version
    }
}

impl FromStr for Constraint {
    type Err = ConstraintError;

    fn from_str(constraint_str: &str) -> Result<Self, Self::Err> {
        // Check if the constraint is empty
        if constraint_str.is_empty() {
            return Err(ConstraintError::EmptyConstraint);
        }

        // Match the comparators
        let (comparator, version_str) = Comparator::extract_comparator(constraint_str);

        // Parse and validate the version string (checks empty + invalid chars)
        let version: VersionString = version_str.parse()?;

        Ok(Constraint {
            comparator,
            version,
        })
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}{}", self.comparator, self.version)
    }
}

/// Errors specific to a single constraint within a VLS string.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ConstraintError {
    /// A constraint segment was empty (e.g. from `||`, a leading `|`, or a trailing `|`).
    #[error("Empty constraint")]
    EmptyConstraint,

    /// The version part of a constraint was empty (e.g. `>=` without a version).
    #[error("Empty version in constraint")]
    EmptyConstraintVersion,

    /// The version string contains characters outside the allowed grammar.
    /// See [`Vls`](crate::Vls) for more details on the grammar.
    #[error("Invalid character(s) in version string: {}", .0.iter().map(|c| format!("'{}'", c.escape_default())).collect::<Vec<_>>().join(", "))]
    InvalidConstraintVersionCharacters(Vec<char>),
}
