//! The core [`Vls`] type.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::Comparator;
use crate::constraint::{Constraint, ConstraintError, VersionString};
use crate::valid_chars::{VlsSpecialCharSet, collect_invalid_characters};
pub use crate::vls_error::VlsError;
use std::collections::{BTreeSet, HashSet};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

/// A **Vers-like Specifier** (VLS).
///
/// VLS is the `<constraints>` part of a [vers](https://github.com/package-url/vers-spec)
/// URL *without* the `vers:<type>/` prefix.
///
/// It is an ordered, `|`-separated list of [`Constraint`] values.
///
/// Due to the unspecified format of the versions, only exact matching is possible and range containment checks are not supported.
///
/// Be aware that when parsing of a string into [`Vls`], the parser returns the first [`VlsError`] encountered in the parsing process.
/// This will obfuscate other errors that might appear further into the parsing process.
///
/// # Syntax
///
/// Derived from the [vers specification](https://www.packageurl.org/docs/vers/how-to-parse) and
/// its comments in
/// [CSAF 2.0](https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html#31232-branches-type---name-under-product-version-range)
/// and
/// [CSAF 2.1](https://docs.oasis-open.org/csaf/csaf/v2.1/csaf-v2.1.html#branches-type---name-under-product-version-range).
///
/// There currently is no "official" grammar for vers-like specifier / the `<constraints>` part of
/// vers. This is a best-effort attempt used for this library.
///
/// **Note:** This grammar may need to be updated once vers has been ratified through ECMA / following changes
/// to / clarifications of CSAF 2.0 or CSAF 2.1.
///
/// ```text
/// vls            = constraint *( "|" constraint )
/// constraint     = comparator version-string / version-string
/// comparator     = "!=" / "<=" / ">=" / "<" / ">"
/// version-string = 1*( ALPHA / DIGIT / "-" / "." / "_" / "+" / "~" )
/// ```
///
/// For validation, this leads to two sets of characters allowed in the context of the grammar.
///
/// For `constraints`: `ALPHA / DIGIT / "-" / "." / "_" / "+" / "~" / "=" / "!" / "<" / ">" / "|"`
///
/// For `version-string`: `ALPHA / DIGIT / "-" / "." / "_" / "+" / "~"`
///
/// Whitespace is insignificant and trimmed before validation anywhere outside of a version
/// string itself — this includes leading/trailing whitespace, whitespace surrounding the `|`
/// constraint delimiter, and whitespace between a comparator and its version (e.g.
/// `" >= 1.0 | <= 2.0 "` is equivalent to `">=1.0|<=2.0"`). Whitespace *within* a version
/// string, such as `"1.0 beta"`, remains invalid.
///
/// # Examples
///
/// ```
/// use vers_like_specifier::Vls;
///
/// let vls: Vls = "<=2".parse().unwrap();
/// assert_eq!(vls.constraints().len(), 1);
///
/// let vls: Vls = ">10.9a|!=10.9c|!=10.9f|<=10.9k".parse().unwrap();
/// assert_eq!(vls.constraints().len(), 4);
/// assert_eq!(vls.to_string(), ">10.9a|!=10.9c|!=10.9f|<=10.9k");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Vls {
    /// An ordered, `|`-separated list of [`Constraint`] values (always non-empty).
    constraints: Vec<Constraint>,
}

impl Vls {
    /// Return the constraints.
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Return `true` if this specifier pins exactly one version,
    /// i.e. it contains a single equal constraint [`Comparator::Equal`]
    pub fn is_single_version(&self) -> bool {
        self.constraints.len() == 1 && matches!(self.constraints[0].comparator(), Comparator::Equal)
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Vls {
    /// Parse a `vls` as a [`Vls`] string, exposed as the static `Vls.parse(vls)` in JS.
    ///
    /// # Errors
    /// Throws a JS `Error` if `vls` is not a valid vers-like specifier.
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = parse))]
    pub fn wasm_parse(vls: &str) -> Result<Vls, VlsError> {
        vls.parse()
    }

    /// Return `true` if this specifier pins exactly one version, exposed as the
    /// instance method `vls.isSingleVersion()` in JS.
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = isSingleVersion))]
    pub fn wasm_is_single_version(&self) -> bool {
        self.is_single_version()
    }
}

impl FromStr for Vls {
    type Err = VlsError;

    /// Try to parse the provided string as [Vls].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Whitespace used purely to separate the input (leading/trailing, or around the `|`
        // constraint delimiter) is insignificant and trimmed away before validation.
        let trimmed = s.trim();

        // If the string is empty, return an error
        if trimmed.is_empty() {
            return Err(VlsError::EmptyInput);
        }

        // If the string is a single wildcard, return an error
        if trimmed == "*" {
            return Err(VlsError::ForbiddenAnyUsed);
        }

        // The next two checks are not strictly necessary, as we would try to parse
        // a string containing the vers URI prefix and / or a type component as part of
        // the first constraint, which would fail the parsing.
        // As this library is tightly coupled to csaf-rs, we still include them for easier /
        // more informative error handling there, as both indicate this might be a vers string.

        // If the string contains the vers URI prefix, return an error
        if trimmed.starts_with("vers:") {
            return Err(VlsError::ContainsVersPrefix);
        }

        // `/` is not a valid character in the vls grammar, but is used as the type delimiter in vers.
        // Its presence indicates the string contains a "<type>/" component
        if trimmed.contains('/') {
            return Err(VlsError::ContainsVersType);
        }

        // Split the constraints, trimming insignificant whitespace around each one.
        let parts: Vec<&str> = trimmed.split('|').map(str::trim).collect();

        // Reject any character that is not part of the 'constraints' grammar. Whitespace is
        // always permitted at this level (it is insignificant anywhere outside of a version
        // string), so any invalid whitespace *within* a version string is instead caught later,
        // when each constraint's version string is parsed and validated on its own.
        let joined = parts.join("|");
        if let Some(invalid) =
            collect_invalid_characters(&joined, VlsSpecialCharSet::ConstraintsString)
        {
            return Err(VlsError::InvalidCharacters(invalid));
        }

        // Parse the constraints, generating parsed Constraint or ConstraintErrors for each
        let mut constraints: Vec<Constraint> = Vec::with_capacity(parts.len());
        let mut constraint_errors: Option<Vec<ConstraintError>> = None;

        for part in parts {
            match part.parse::<Constraint>() {
                Ok(constraint) => constraints.push(constraint),
                Err(error) => constraint_errors.get_or_insert_default().push(error),
            }
        }

        // Report constraint errors before parse errors
        if let Some(constraint_errors) = constraint_errors {
            return Err(VlsError::InvalidConstraints(constraint_errors));
        }

        // Check for duplicate constraints
        let mut seen_versions: HashSet<&VersionString> = HashSet::new();
        let mut duplicate_versions: Option<BTreeSet<String>> = None;
        for c in &constraints {
            if !seen_versions.insert(c.version()) {
                duplicate_versions
                    .get_or_insert_default()
                    .insert(c.version().to_string());
            }
        }
        if let Some(duplicate_versions) = duplicate_versions {
            return Err(VlsError::DuplicateConstraintVersions(duplicate_versions));
        }

        Ok(Self { constraints })
    }
}

impl Display for Vls {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut iter = self.constraints.iter();
        if let Some(first) = iter.next() {
            first.fmt(f)?;
            for c in iter {
                f.write_str("|")?;
                c.fmt(f)?;
            }
        }
        Ok(())
    }
}
