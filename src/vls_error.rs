use std::collections::BTreeSet;

use crate::constraint::ConstraintError;
use thiserror::Error;
#[cfg(feature = "wasm")]
use {js_sys::Error as JsError, wasm_bindgen::JsValue};

/// Errors that can occur when parsing a vls string.
#[derive(Error, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
pub enum VlsError {
    /// The input string was empty.
    #[error("Empty vls input")]
    EmptyInput,

    /// The input is a wildcard (`*`), which is not allowed.
    #[error("'*' (vers syntax for matching all versions) is not allowed as a vls string")]
    ForbiddenAnyUsed,

    /// The input contains characters not allowed by the VLS grammar.
    /// See [`Vls`] for more details on the grammar.
    #[error("Invalid character(s) in VLS: {}", .0.iter().map(|c| format!("'{}'", c.escape_default())).collect::<Vec<_>>().join(", "))]
    InvalidCharacters(Vec<char>),

    /// The input contains a `vers:` URI prefix, which is not allowed in a VLS string.
    #[error("VLS must not contain a 'vers:' URI prefix")]
    ContainsVersPrefix,

    /// The input most likely contains a vers type
    /// component (e.g. `gem` in `gem/>=2.2.0`), indicated by the presence of the type delimiter `/`.
    #[error("VLS must not contain a vers type component")]
    ContainsVersType,

    /// One or more constraints are invalid, for example due to constraints or version strings
    /// being empty, or due to invalid characters in version strings.
    #[error("Invalid constraint(s): {}", .0.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "))]
    InvalidConstraints(Vec<ConstraintError>),

    /// The input contains duplicate constraint versions, irrespective of their comparators.
    #[error("Duplicate constraint version(s): {}", .0.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(", "))]
    DuplicateConstraintVersions(BTreeSet<String>),

    /// The provided constraints list was empty.
    #[error("At least one constraint is required")]
    EmptyConstraints,

    /// A value could not be (de)serialized across the wasm ABI boundary.
    #[cfg(feature = "wasm")]
    #[error("WASM (de)serialization error: {0}")]
    Serialization(String),
}

/// Convert VlsError into a JS exception value when targeting wasm.
#[cfg(feature = "wasm")]
impl From<VlsError> for JsValue {
    fn from(e: VlsError) -> JsValue {
        JsValue::from(JsError::new(&e.to_string()))
    }
}

/// Convert a `tsify` (de)serialization error into a `VlsError` when a value
/// fails to cross the wasm ABI boundary via [`tsify::Ts`].
#[cfg(feature = "wasm")]
impl From<tsify::Error> for VlsError {
    fn from(e: tsify::Error) -> Self {
        VlsError::Serialization(e.to_string())
    }
}
