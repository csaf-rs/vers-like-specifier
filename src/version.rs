use crate::VersionConstraintError;
use crate::valid_chars::{VlsSpecialCharSet, collect_invalid_characters};
use std::fmt;
use std::str::FromStr;

/// A validated version string.
///
/// A `VersionString` is guaranteed to be non-empty and to contain only characters
/// allowed by the version-string grammar. See vls::Vls for more details on the grammar.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct VersionString(String);

impl VersionString {
    /// Return the version string as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for VersionString {
    type Err = VersionConstraintError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(VersionConstraintError::EmptyVersion);
        }

        let invalid_chars = collect_invalid_characters(input, VlsSpecialCharSet::VersionString);
        if let Some(invalid_chars) = invalid_chars {
            return Err(VersionConstraintError::InvalidVersionCharacters(
                invalid_chars,
            ));
        }

        Ok(Self(input.to_string()))
    }
}

impl fmt::Display for VersionString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
