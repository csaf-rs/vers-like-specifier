use rstest::rstest;
use vers_like_specifier::{Comparator, Vls};

#[rstest]
#[case("<=2", Comparator::LessThanOrEqual, "2", false)]
#[case("<4.2", Comparator::LessThan, "4.2", false)]
#[case(">=8.1.5", Comparator::GreaterThanOrEqual, "8.1.5", false)]
#[case(">1.0.0", Comparator::GreaterThan, "1.0.0", false)]
#[case("1.2.3", Comparator::Equal, "1.2.3", true)]
#[case("!=5.0", Comparator::NotEqual, "5.0", false)]
#[case(">=1.0,0", Comparator::GreaterThanOrEqual, "1.0,0", false)]
#[case(">=1.0+build.1", Comparator::GreaterThanOrEqual, "1.0+build.1", false)]
#[case("!=1.0:beta", Comparator::NotEqual, "1.0:beta", false)]
#[case("<=1.0@2", Comparator::LessThanOrEqual, "1.0@2", false)]
#[case("1.0#rc1", Comparator::Equal, "1.0#rc1", true)]
#[case(">=1.0&2.0", Comparator::GreaterThanOrEqual, "1.0&2.0", false)]
#[case("1.0'beta", Comparator::Equal, "1.0'beta", true)]
#[case("1.0;2", Comparator::Equal, "1.0;2", true)]
#[case("1.0?beta", Comparator::Equal, "1.0?beta", true)]
#[case("1.0[2]", Comparator::Equal, "1.0[2]", true)]
#[case("1.0^2", Comparator::Equal, "1.0^2", true)]
#[case("1.0`beta", Comparator::Equal, "1.0`beta", true)]
#[case("1.0{2}", Comparator::Equal, "1.0{2}", true)]
#[case("1.0\"beta\"", Comparator::Equal, "1.0\"beta\"", true)]
fn parse_single_versioned_constraint(
    #[case] input: &str,
    #[case] expected_cmp: Comparator,
    #[case] expected_ver: &str,
    #[case] expected_is_single_version: bool,
) {
    let v: Vls = input
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert_eq!(v.to_string(), input);
    let cs = v.constraints();
    assert_eq!(cs.len(), 1);
    assert_eq!(*cs[0].comparator(), expected_cmp);
    assert_eq!(cs[0].version().as_str(), expected_ver);
    assert_eq!(v.is_single_version(), expected_is_single_version);
}

#[rstest]
#[case::leading_whitespace(" >=1.0", ">=1.0")]
#[case::trailing_whitespace(">=1.0 ", ">=1.0")]
#[case::leading_and_trailing_whitespace("  >=1.0  ", ">=1.0")]
#[case::whitespace_around_pipe(">=1.0 | <=2.0", ">=1.0|<=2.0")]
#[case::multiple_whitespace_around_pipes(">=1.0  |  <=2.0  |  !=3.0", ">=1.0|<=2.0|!=3.0")]
#[case::whitespace_after_comparator(">= 1.0", ">=1.0")]
#[case::multiple_whitespace_after_comparator(">=  1.0", ">=1.0")]
#[case::whitespace_everywhere("  >=  1.0  |  <=  2.0  ", ">=1.0|<=2.0")]
#[case::whitespace_before_implicit_equal_version("  1.0  ", "1.0")]
fn parse_insignificant_whitespace_is_trimmed(#[case] input: &str, #[case] expected: &str) {
    let v: Vls = input
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert_eq!(v.to_string(), expected);
}

#[rstest]
#[case("1.0%3Ebeta", "1.0%3Ebeta")]
#[case("1.0%3Cbeta", "1.0%3Cbeta")]
#[case("1.0%3Dbeta", "1.0%3Dbeta")]
#[case("1.0%21beta", "1.0%21beta")]
#[case("1.0%2Abeta", "1.0%2Abeta")]
#[case("1.0%7Cbeta", "1.0%7Cbeta")]
#[case("1.0%25beta", "1.0%25beta")]
#[case("1.0%3ebeta", "1.0%3ebeta")]
#[case("1.0%3e%3c%3d%21%2a%7c%25beta", "1.0%3e%3c%3d%21%2a%7c%25beta")]
// Percent-encoding is validated generally, not just for the seven VERS-reserved characters:
// any well-formed `%` + two hex digits triplet is valid, regardless of what it decodes to.
#[case("1.0%41beta", "1.0%41beta")]
#[case("1.0%ffbeta", "1.0%ffbeta")]
#[case("1.0%00beta", "1.0%00beta")]
#[case("1.0%20beta", "1.0%20beta")]
fn parse_percent_encoded_characters_in_version(
    #[case] input: &str,
    #[case] expected_version: &str,
) {
    let v: Vls = input
        .parse()
        .expect("Expected valid vls constraints to parse");
    let cs = v.constraints();
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].version().as_str(), expected_version);
    assert_eq!(v.to_string(), input);
}

#[test]
fn parse_multiple_constraints() {
    let input = ">10.9a|!=10.9c|!=10.9f|<=10.9k";

    let v: Vls = input
        .parse()
        .expect("Expected valid vls constraints to parse");
    assert!(!v.is_single_version());

    let cs = v.constraints();
    assert_eq!(cs.len(), 4);

    assert_eq!(*cs[0].comparator(), Comparator::GreaterThan);
    assert_eq!(cs[0].version().as_str(), "10.9a");

    assert_eq!(*cs[1].comparator(), Comparator::NotEqual);
    assert_eq!(cs[1].version().as_str(), "10.9c");

    assert_eq!(*cs[2].comparator(), Comparator::NotEqual);
    assert_eq!(cs[2].version().as_str(), "10.9f");

    assert_eq!(*cs[3].comparator(), Comparator::LessThanOrEqual);
    assert_eq!(cs[3].version().as_str(), "10.9k");

    assert_eq!(v.to_string(), input);
}
