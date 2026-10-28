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
/// Derived from the [vers specification](https://www.packageurl.org/docs/vers/how-to-parse),
/// its comments in
/// [CSAF 2.0](https://docs.oasis-open.org/csaf/csaf/v2.0/os/csaf-v2.0-os.html#31232-branches-type---name-under-product-version-range)
/// and
/// [CSAF 2.1](https://docs.oasis-open.org/csaf/csaf/v2.1/csaf-v2.1.html#branches-type---name-under-product-version-range).
///
/// There currently is no "official" grammar for vers-like specifier / the `<constraints>` part of
/// vers. This is a best-effort attempt used for this library.
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

/// The result of a containment check performed by [`Vls::contains`].
///
/// Because the versioning scheme of a [`Vls`] is unspecified, containment can usually only be
/// resolved when the queried version exactly matches a constraint's version:
///
/// * [`Comparator::Equal`], [`Comparator::GreaterThanOrEqual`], [`Comparator::LessThanOrEqual`]:
///   an exact match means the version is contained (`x = x`, `x >= x`, `x <= x` all hold) →
///   [`Containment::Contained`].
/// * [`Comparator::NotEqual`], [`Comparator::LessThan`], [`Comparator::GreaterThan`]:
///   an exact match means the version is explicitly not contained (`x != x`, `x < x`, `x > x`
///   are always false) → [`Containment::NotContained`].
/// * No constraint's version matches exactly: containment cannot be resolved (the versioning
///   scheme is unknown) → [`Containment::Undetermined`].
///
/// The one exception is a [`Vls`] that pins a single version (see [`Vls::is_single_version`]):
/// since it has exactly one [`Comparator::Equal`] constraint, containment reduces to a plain
/// string comparison and is always decidable, regardless of versioning scheme. In that case,
/// *any* queried version resolves to either [`Containment::Contained`] or
/// [`Containment::NotContained`], never [`Containment::Undetermined`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum Containment {
    /// The version is contained in the specifier.
    Contained,
    /// The version is explicitly not contained in the specifier.
    NotContained,
    /// Containment could not be determined: the version does not exactly match any
    /// constraint, and the (unspecified) versioning scheme does not allow any further
    /// comparison.
    Undetermined,
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

    /// Return the single version if this specifier describes exactly one version, otherwise `None`.
    /// See [`Vls::is_single_version`] for details on what constitutes a single version.
    pub fn get_single_version(&self) -> Option<&VersionString> {
        if self.is_single_version() {
            Some(self.constraints[0].version())
        } else {
            None
        }
    }

    /// Return the [`Containment`] of the given `version` in this specifier.
    ///
    /// Because the versioning scheme of a [`Vls`] is unspecified, order-based containment
    /// checks (e.g. whether some version falls between two others) are not possible. Only
    /// constraints using an exact-match comparator can resolve this check:
    ///
    /// * [`Comparator::Equal`], [`Comparator::GreaterThanOrEqual`], [`Comparator::LessThanOrEqual`]:
    ///   if their version is exactly equal to `version`, it is contained (`x = x`, `x >= x`, `x <= x`
    ///   all hold) → [`Containment::Contained`].
    /// * [`Comparator::NotEqual`]: if its version is exactly equal to `version`, it is explicitly
    ///   excluded, so it is not contained → [`Containment::NotContained`].
    /// * [`Comparator::LessThan`], [`Comparator::GreaterThan`]: an exact match can never be
    ///   contained, since `x < x` and `x > x` are always false → [`Containment::NotContained`].
    ///
    /// If `version` does not exactly match any constraint's version, containment cannot be
    /// determined (the versioning scheme is unknown), so this returns [`Containment::Undetermined`].
    ///
    /// **Exception:** if this [`Vls`] pins a single version (see [`Vls::is_single_version`]), its
    /// one [`Comparator::Equal`] constraint makes containment decidable via plain string equality
    /// alone, so `version` always resolves to [`Containment::Contained`] or
    /// [`Containment::NotContained`] — never [`Containment::Undetermined`].
    ///
    /// Note that a [`Vls`] cannot contain two constraints with the same version string (regardless
    /// of comparator), so at most one constraint can ever match `version` exactly.
    ///
    /// Surrounding whitespace in `version` is stripped before comparing, matching the behavior
    /// of the constraints a `Vls` is parsed from (which never contain whitespace).
    ///
    /// # Examples
    ///
    /// ```
    /// use vers_like_specifier::{Containment, Vls};
    ///
    /// let vls: Vls = ">=1.0.0|!=1.5.0|<=2.0.0".parse().unwrap();
    /// assert_eq!(vls.contains("1.0.0"), Containment::Contained);
    /// assert_eq!(vls.contains("2.0.0"), Containment::Contained);
    /// assert_eq!(vls.contains("1.5.0"), Containment::NotContained);
    /// assert_eq!(vls.contains("1.6.0"), Containment::Undetermined);
    /// assert_eq!(vls.contains("  1.0.0  "), Containment::Contained);
    ///
    /// // A single pinned version is always decidable, even for versions that don't match.
    /// let pinned: Vls = "1.0.0".parse().unwrap();
    /// assert_eq!(pinned.contains("1.0.0"), Containment::Contained);
    /// assert_eq!(pinned.contains("2.0.0"), Containment::NotContained);
    /// ```
    pub fn contains(&self, version: &str) -> Containment {
        let version = version.trim();

        if self.is_single_version() {
            return if self.constraints[0].version().as_str() == version {
                Containment::Contained
            } else {
                Containment::NotContained
            };
        }

        self.constraints
            .iter()
            .find(|c| c.version().as_str() == version)
            .map_or(Containment::Undetermined, |c| match c.comparator() {
                Comparator::Equal
                | Comparator::GreaterThanOrEqual
                | Comparator::LessThanOrEqual => Containment::Contained,
                Comparator::NotEqual | Comparator::LessThan | Comparator::GreaterThan => {
                    Containment::NotContained
                }
            })
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

    /// Return the [`Containment`] of the given `version` in this specifier, exposed as the
    /// instance method `vls.contains(version)` in JS.
    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = contains))]
    pub fn wasm_contains(&self, version: &str) -> Containment {
        self.contains(version)
    }
}

impl FromStr for Vls {
    type Err = VlsError;

    /// Try to parse the provided string as [Vls].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Strip surrounding whitespace; literal whitespace remaining within a constraint or
        // version (after trimming) is rejected further down as an invalid character.
        let trimmed = s.trim();

        // If the string is empty, return an error
        if trimmed.is_empty() {
            return Err(VlsError::EmptyInput);
        }

        // If the string is a single wildcard, return an error
        if trimmed == "*" {
            return Err(VlsError::ForbiddenAnyUsed);
        }

        // Reject any character that is not part of the 'constraints' grammar.
        if let Some(invalid) = collect_invalid_characters(s, VlsSpecialCharSet::ConstraintsString) {
            return Err(VlsError::InvalidCharacters(invalid));
        }

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
