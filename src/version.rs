use crate::ConstraintError;
use crate::valid_chars::{VlsSpecialCharSet, collect_invalid_characters};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// A validated version string.
///
/// A `VersionString` is guaranteed to be non-empty and to contain only characters
/// allowed by the version-string grammar. See [`Vls`](crate::Vls) for more details on the grammar.
#[derive(Clone, PartialEq, Eq, Debug, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VersionString(String);

impl VersionString {
    /// Return the version string as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VersionString {
    /// Construct a new [`VersionString`], validating it against the vls grammar. Exposed as
    /// `new VersionString(version)` in JS.
    ///
    /// # Errors
    /// Throws a JS `Error` if `version` is empty or contains characters outside the grammar.
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(version: &str) -> Result<VersionString, ConstraintError> {
        version.parse()
    }

    /// Return the version string, exposed as `version.toString()` in JS.
    #[wasm_bindgen(js_name = toString)]
    pub fn wasm_to_string(&self) -> String {
        self.to_string()
    }
}

impl FromStr for VersionString {
    type Err = ConstraintError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ConstraintError::EmptyConstraintVersion);
        }

        let invalid_chars = collect_invalid_characters(input, VlsSpecialCharSet::VersionString);
        if let Some(invalid_chars) = invalid_chars {
            return Err(ConstraintError::InvalidConstraintVersionCharacters(
                invalid_chars,
            ));
        }

        Ok(Self(input.to_string()))
    }
}

impl Display for VersionString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}
