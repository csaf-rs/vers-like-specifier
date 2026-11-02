use rstest::rstest;
use vers_like_specifier::{Containment, Vls};

#[rstest]
// GreaterThanOrEqual: exact match of the boundary is contained.
#[case(">=1.0.0", "1.0.0", Containment::Contained)]
// GreaterThanOrEqual: any other version cannot be resolved, even if "obviously" greater.
#[case(">=1.0.0", "2.0.0", Containment::Undetermined)]
#[case(">=1.0.0", "0.9.0", Containment::Undetermined)]
// LessThanOrEqual: exact match of the boundary is contained.
#[case("<=2.0.0", "2.0.0", Containment::Contained)]
// LessThanOrEqual: any other version cannot be resolved.
#[case("<=2.0.0", "1.0.0", Containment::Undetermined)]
#[case("<=2.0.0", "3.0.0", Containment::Undetermined)]
// NotEqual: exact match is explicitly excluded.
#[case("!=1.5.0", "1.5.0", Containment::NotContained)]
// NotEqual: any other version cannot be resolved.
#[case("!=1.5.0", "1.6.0", Containment::NotContained)]
// LessThan / GreaterThan: an exact match can never satisfy a strict inequality.
#[case("<2.0.0", "2.0.0", Containment::NotContained)]
#[case(">1.0.0", "1.0.0", Containment::NotContained)]
// LessThan / GreaterThan: any other version cannot be resolved either.
#[case("<2.0.0", "1.0.0", Containment::Undetermined)]
#[case(">1.0.0", "2.0.0", Containment::Undetermined)]
// Enumeration
#[case("1.0.0|!=2.0.0", "1.0.0", Containment::Contained)]
#[case("1.0.0|!=2.0.0", "2.0.0", Containment::NotContained)]
#[case("1.0.0|!=2.0.0", "3.0.0", Containment::NotContained)]
fn contains_single_constraint(
    #[case] vls: &str,
    #[case] version: &str,
    #[case] expected: Containment,
) {
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
    assert_eq!(vls.contains("1.0.0"), Containment::Contained);
    assert_eq!(vls.contains("2.0.0"), Containment::Contained);

    // Explicitly excluded version is not contained.
    assert_eq!(vls.contains("1.5.0"), Containment::NotContained);

    // Versions not exactly matching any constraint can't be determined.
    assert_eq!(vls.contains("1.2.0"), Containment::Undetermined);
    assert_eq!(vls.contains("0.5.0"), Containment::Undetermined);
    assert_eq!(vls.contains("3.0.0"), Containment::Undetermined);
}

#[test]
fn contains_multiple_constraints_with_only_strict_and_not_equal() {
    let vls: Vls = ">1.0.0|!=1.5.0|<2.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");

    // Strict comparators never resolve containment positively, even for their own boundary.
    assert_eq!(vls.contains("1.0.0"), Containment::NotContained);
    assert_eq!(vls.contains("2.0.0"), Containment::NotContained);

    // Explicitly excluded version is not contained.
    assert_eq!(vls.contains("1.5.0"), Containment::NotContained);

    // Unrelated version can't be determined.
    assert_eq!(vls.contains("1.2.0"), Containment::Undetermined);
}

#[test]
fn contains_is_case_sensitive() {
    let vls: Vls = "1.2.3-RC1|1.2.3-RC2"
        .parse()
        .expect("Expected valid vls constraints to parse");

    assert_eq!(vls.contains("1.2.3-RC1"), Containment::Contained);
    // This vls pins two exact versions.
    // A differently-cased (and thus non-matching) version is definitely not contained, rather than merely undetermined.
    assert_eq!(vls.contains("1.2.3-rc1"), Containment::NotContained);
}

#[rstest]
#[case("  1.0.0", Containment::Contained)]
#[case("1.0.0  ", Containment::Contained)]
#[case("  1.0.0  ", Containment::Contained)]
// This vls pins two exact versions, so a non-matching version is definitely not contained.
#[case("  2.0.0  ", Containment::NotContained)]
fn contains_strips_surrounding_whitespace_from_queried_version(
    #[case] version: &str,
    #[case] expected: Containment,
) {
    let vls: Vls = "1.0.0|3.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert_eq!(vls.contains(version), expected);
}

#[test]
fn contains_does_not_strip_internal_whitespace() {
    let vls: Vls = "1.0.0|3.0.0"
        .parse()
        .expect("Expected valid vls constraints to parse");
    // "1.0.0 extra" (after trimming) does not match "1.0.0" or "3.0.0", so, since this vls pins a
    // two exact versions, it is definitely not contained.
    assert_eq!(vls.contains(" 1.0.0 extra "), Containment::NotContained);
}
