use crate::VersionConstraintError;
use crate::comparator::Comparator;
use crate::version::VersionString;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
    type Err = Vec<VersionConstraintError>;

    fn from_str(constraint_str: &str) -> Result<Self, Self::Err> {
        // Check if the constraint is empty
        if constraint_str.is_empty() {
            return Err(vec![VersionConstraintError::EmptyConstraint]);
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
