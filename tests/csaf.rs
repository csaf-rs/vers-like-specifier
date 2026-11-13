//! Version range strings from CSAF 2.0 and 2.1 examples
//! Extraction via csaf repo's scripts/extract/extract_product_version_ranges.py

use rstest::rstest;
use vers_like_specifier::{ConstraintError, Vls, VlsError};

#[rstest]
#[case("<4.2")]
#[case(">4.2")]
#[case(">2.1.0|<4.2.0")]
#[case(">1.2.0|<2.4.0")]
#[case(">1.2.0|<=2.1.0")]
#[case(">=2.4.0|<4.2.0")]
// Here, whitespace is fine, discuss TODO.
#[case("<= 2024.04")]
#[case("<V3.1.2.2")]
#[case("<=2024.04")]
#[case(">abc|<=def")]
#[case("<v0.8.15")]
#[case("<4.2.0")]
#[case(">3,16")]
#[case(">=4.2.0")]
#[case("<1.2.0|>=4.2.0")]
#[case(">2.4.0|<4.2.0")]
#[case("<1.2.0|4.2.0")]
#[case(">1.2.0|<2.1.0")]
#[case(">2.1.0|!=3.16.0|<4.2.0")]
fn parse_valid_csaf_version_strings(#[case] input: &str) {
    input
        .parse::<Vls>()
        .expect("expected CSAF version string to parse");
}

#[rstest]
// Intended, should report "<" only though TODO.
#[case("all versions < 4.2.0", VlsError::InvalidConstraints(vec![ConstraintError::InvalidConstraintVersionCharacters(vec![' ', '<'])]))]
// Intended to be false, but recent discussions on vls pointed at white space being fine, so this might be subject to change TODO.
#[case("all versions", VlsError::InvalidConstraints(vec![ConstraintError::InvalidConstraintVersionCharacters(vec![' '])]))]
// Intended, should report "<=" only though TODO.
#[case("> abc and <= def", VlsError::InvalidConstraints(vec![ConstraintError::InvalidConstraintVersionCharacters(vec![' ', '<', '='])]))]
// Intended, should report the "<" only though TODO.
#[case("all versions < v0.8.15", VlsError::InvalidConstraints(vec![ConstraintError::InvalidConstraintVersionCharacters(vec![' ', '<'])]))]
// Intended, single versions are invalid
#[case("4.2.0", VlsError::SingleVersionNotAllowed("4.2.0".to_owned()))]
fn parse_invalid_csaf_version_strings(#[case] input: &str, #[case] expected_error: VlsError) {
    assert_eq!(input.parse::<Vls>().unwrap_err(), expected_error);
}
