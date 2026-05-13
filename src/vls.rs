//! The core [`Vls`] type.

use crate::constraint::{VersionConstraint, VersionConstraintError, VersionString};
use crate::valid_chars::{VlsSpecialCharSet, collect_invalid_characters};
use std::collections::{BTreeSet, HashSet};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use thiserror::Error;

/// A **Vers-like Specifier** (VLS).
///
/// VLS is the `<version-constraint>` part of a [vers](https://github.com/package-url/vers-spec)
/// URL *without* the `vers:<scheme>/` prefix.
///
/// It is an ordered, `|`-separated list of [`VersionConstraint`] values.
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
/// There currently is no "official" grammar for vers-like specifier / the `<version-constraint>` part of
/// vers. This is a best-effort attempt used for this library.
///
/// **Note:** This grammar may need to be updated once vers has been ratified through ECMA / following changes
/// to / clarifications of CSAF 2.0 or CSAF 2.1.
///
/// ```text
/// vls            = constraint *( "|" constraint )
/// constraint     = comparator version-string / version-string
/// comparator     = "!=" / "<=" / ">=" / "=" / "<" / ">"
/// version-string = 1*( ALPHA / DIGIT / "-" / "." / "_" / "+" / "~" )
/// ```
///
/// For validation, this leads to two sets of characters allowed in the context of the grammar.
///
/// For `constraints`: `ALPHA / DIGIT / "-" / "." / "_" / "+" / "~" / "=" / "!" / "<" / ">" / "|"`
///
/// For `version-string`: `ALPHA / DIGIT / "-" / "." / "_" / "+" / "~"`
///
/// # Examples
///
/// ```
/// use vls::Vls;
///
/// let vls: Vls = "<=2".parse().unwrap();
/// assert_eq!(vls.constraints().len(), 1);
///
/// let vls: Vls = ">10.9a|!=10.9c|!=10.9f|<=10.9k".parse().unwrap();
/// assert_eq!(vls.constraints().len(), 4);
/// assert_eq!(vls.to_string(), ">10.9a|!=10.9c|!=10.9f|<=10.9k");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vls {
    /// An ordered, `|`-separated list of [`VersionConstraint`] values (always non-empty).
    constraints: Vec<VersionConstraint>,
}

impl Vls {
    /// Return the constraints.
    pub fn constraints(&self) -> &[VersionConstraint] {
        &self.constraints
    }

    /// Return `true` if this specifier pins exactly one version,
    /// i.e. it contains a single equal constraint [`EqualImplicit`](crate::comparator::Comparator::EqualImplicit) or [`EqualExplicit`](crate::comparator::Comparator::EqualExplicit)
    pub fn is_single_version(&self) -> bool {
        self.constraints.len() == 1 && self.constraints[0].comparator().is_equal()
    }
}

impl FromStr for Vls {
    type Err = VlsError;

    /// Try to parse the provided string as [Vls].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // If the string is empty, return an error
        if s.is_empty() {
            return Err(VlsError::EmptyInput);
        }

        // If the string is a single wildcard, return an error
        if s == "*" {
            return Err(VlsError::IsAny);
        }

        // The next two checks are not strictly necessary, as we would try to parse
        // a string containing the vers URI prefix and / or a scheme component as part of
        // the first constraint, which would fail the parsing.
        // As this library is tightly coupled to csaf-rs, we still include them for easier /
        // more informative error handling there, as both indicate this might be a vers string.

        // If the string contains the vers URI prefix, return an error
        if s.starts_with("vers:") {
            return Err(VlsError::ContainsVersPrefix);
        }

        // `/` is not a valid character in the vls grammar, but is used as the scheme delimiter in vers.
        // Its presence indicates the string contains a "<scheme>/" component
        if s.contains('/') {
            return Err(VlsError::ContainsVersioningScheme);
        }

        // Reject any character that is not part of the 'constraints' grammar.
        if let Some(invalid) = collect_invalid_characters(s, VlsSpecialCharSet::ConstraintsString) {
            return Err(VlsError::InvalidCharacters(invalid));
        }

        // Split the constraints
        let parts: Vec<&str> = s.split('|').collect();

        // Parse the constraints, generating parsed VersionConstraint or VersionConstraintErrors for each
        let mut constraints: Vec<VersionConstraint> = Vec::with_capacity(parts.len());
        let mut constraint_errors: Option<Vec<VersionConstraintError>> = None;

        for part in parts {
            match part.parse::<VersionConstraint>() {
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

/// Errors that can occur when parsing a vls string.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum VlsError {
    /// The input string was empty.
    #[error("Empty vls input")]
    EmptyInput,

    /// The input is a wildcard (`*`), which is not allowed.
    #[error("'*' (vers syntax for matching all versions) is not allowed as a vls string")]
    IsAny,

    /// The input contains characters not allowed by the VLS grammar.
    /// See [`Vls`] for more details on the grammar.
    #[error("Invalid character(s) in VLS: {}", .0.iter().map(|c| format!("'{}'", c.escape_default())).collect::<Vec<_>>().join(", "))]
    InvalidCharacters(Vec<char>),

    /// The input contains a `vers:` URI prefix, which is not allowed in a VLS string.
    #[error("VLS must not contain a 'vers:' URI prefix")]
    ContainsVersPrefix,

    /// The input most likely contains a `vers` versioning-scheme
    /// component (e.g. `gem/>=2.2.0`), indicated by the presence of the scheme delimiter `/`.
    #[error("VLS must not contain a versioning-scheme component")]
    ContainsVersioningScheme,

    /// One or more version strings contain characters outside the allowed grammar.
    #[error("Invalid constraint(s): {}", .0.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "))]
    InvalidConstraints(Vec<VersionConstraintError>),

    /// The input contains duplicate constraint versions, irrespective of their comparators.
    #[error("Duplicate constraint version(s): {}", .0.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(", "))]
    DuplicateConstraintVersions(BTreeSet<String>),
}
