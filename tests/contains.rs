use rstest::rstest;
use vers_like_specifier::Vls;

#[rstest]
// Equal comparator (implicit; there is no explicit "=" in the grammar): exact match is contained.
#[case("1.2.3", "1.2.3", true)]
// Equal comparator: non-matching version is not contained.
#[case("1.2.3", "1.2.4", false)]
// GreaterThanOrEqual: exact match of the boundary is contained.
#[case(">=1.0.0", "1.0.0", true)]
// GreaterThanOrEqual: any other version cannot be resolved, even if "obviously" greater.
#[case(">=1.0.0", "2.0.0", false)]
#[case(">=1.0.0", "0.9.0", false)]
// LessThanOrEqual: exact match of the boundary is contained.
#[case("<=2.0.0", "2.0.0", true)]
// LessThanOrEqual: any other version cannot be resolved.
#[case("<=2.0.0", "1.0.0", false)]
#[case("<=2.0.0", "3.0.0", false)]
// NotEqual: exact match is explicitly excluded.
#[case("!=1.5.0", "1.5.0", false)]
// NotEqual: any other version cannot be resolved.
#[case("!=1.5.0", "1.6.0", false)]
// LessThan / GreaterThan: an exact match can never satisfy a strict inequality.
#[case("<2.0.0", "2.0.0", false)]
#[case(">1.0.0", "1.0.0", false)]
// LessThan / GreaterThan: any other version cannot be resolved either.
#[case("<2.0.0", "1.0.0", false)]
#[case(">1.0.0", "2.0.0", false)]
fn contains_single_constraint(#[case] vls: &str, #[case] version: &str, #[case] expected: bool) {
    let vls: Vls = vls
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert_eq!(vls.contains(version), expected);
}

#[test]
fn contains_multiple_constraints_resolves_matching_exact_constraint() {
    let vls: Vls = ">=1.0.0|!=1.5.0|<=2.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");

    // Lower and upper boundaries are contained (via >= and <=).
    assert!(vls.contains("1.0.0"));
    assert!(vls.contains("2.0.0"));

    // Explicitly excluded version is not contained.
    assert!(!vls.contains("1.5.0"));

    // Versions not exactly matching any constraint can't be resolved.
    assert!(!vls.contains("1.2.0"));
    assert!(!vls.contains("0.5.0"));
    assert!(!vls.contains("3.0.0"));
}

#[test]
fn contains_multiple_constraints_with_only_strict_and_not_equal() {
    let vls: Vls = ">1.0.0|!=1.5.0|<2.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");

    // Strict comparators never resolve containment, even for their own boundary.
    assert!(!vls.contains("1.0.0"));
    assert!(!vls.contains("2.0.0"));

    // Explicitly excluded version is not contained.
    assert!(!vls.contains("1.5.0"));

    // Unrelated version can't be resolved.
    assert!(!vls.contains("1.2.0"));
}

#[test]
fn contains_is_case_sensitive() {
    let vls: Vls = "1.2.3-RC1"
        .parse()
        .expect("Expected valid vls constraints to parse");

    assert!(vls.contains("1.2.3-RC1"));
    assert!(!vls.contains("1.2.3-rc1"));
}

#[rstest]
#[case("  1.0.0", true)]
#[case("1.0.0  ", true)]
#[case("  1.0.0  ", true)]
#[case("  2.0.0  ", false)]
fn contains_strips_surrounding_whitespace_from_queried_version(
    #[case] version: &str,
    #[case] expected: bool,
) {
    let vls: Vls = "1.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert_eq!(vls.contains(version), expected);
}

#[test]
fn contains_does_not_strip_internal_whitespace() {
    let vls: Vls = "1.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert!(!vls.contains(" 1.0.0 extra "));
}
