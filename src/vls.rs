//! The core [`Vls`] type.

use crate::VersionString;
use crate::comparator::Comparator;
use crate::constraint::VersionConstraint;
use crate::error::{VersionConstraintError, VlsError};
use crate::valid_chars::{VlsSpecialCharSet, collect_invalid_characters};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

/// A **Vers-like Specifier** (VLS).
///
/// VLS is the `<version-constraint>` part of a [vers](https://github.com/package-url/vers-spec)
/// URL *without* the `vers:<scheme>/` prefix.
///
/// It is either an ordered, `|`-separated list of [`VersionConstraint`] values (via [Constraints](Self::Constraints))
/// or a wildcard (`*`) indicating that any version is acceptable (via [Any](Self::Any)).
///
/// Due to the unspecified format of the versions, only exact matching is possible and containment checks are not supported.
///
/// Be aware that when parsing of a string into [Vls], the parser returns the first [VlsError] encountered in the parsing process.
/// This will obfuscate other errors that might appear further into the parsing process.
///
/// # Syntax
///
/// Derived from the [vers specification](https://www.packageurl.org/docs/vers/how-to-parse).
/// There currently is no "official" grammar for vers-like specifier / the <version-constraint> part of
/// vers. This is a best-effort attempt used for this library.
///
/// TODO: Revisit this once vers has been ratified through ECMA, which might include an official grammar.
///
/// ```text
/// vls            = constraints / "*"
/// constraints    = constraint *( "|" constraint )
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
pub enum Vls {
    /// Matches any version (`*`).
    Any,
    /// An ordered, `|`-separated list of [`VersionConstraint`] values (always non-empty).
    Constraints(Vec<VersionConstraint>),
}

impl Vls {
    /// Return the constraints, or an empty slice for [`Any`](Self::Any).
    pub fn constraints(&self) -> &[VersionConstraint] {
        match self {
            Self::Any => &[],
            Self::Constraints(cs) => cs,
        }
    }

    /// Return `true` if this was parsed from a single `*`.
    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    /// Return `true` if this specifier pins exactly one version,
    /// i.e. it contains a single equal constraint [`Comparator::EqualImplicit`] or [`Comparator::EqualExplicit`]
    pub fn is_single_version(&self) -> bool {
        matches!(
            self,
            Self::Constraints(cs)
                if cs.len() == 1 && cs[0].comparator().is_equal()
        )
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

        // Early return for Any
        if s == "*" {
            return Ok(Self::Any);
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
                Err(errors) => constraint_errors.get_or_insert_default().extend(errors),
            }
        }

        // Report constraint errors before parse errors
        if let Some(constraint_errors) = constraint_errors {
            return Err(VlsError::InvalidConstraintError(constraint_errors));
        }

        // Check for duplicate constraints
        let mut seen_versions: HashSet<&VersionString> = HashSet::new();
        let mut duplicate_versions: Option<HashSet<String>> = None;
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

        Ok(Self::Constraints(constraints))
    }
}

impl Display for Vls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => write!(f, "*"),
            Self::Constraints(constraints) => {
                let mut first = true;
                for c in constraints {
                    if !first {
                        f.write_str("|")?;
                    }
                    first = false;
                    c.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}
