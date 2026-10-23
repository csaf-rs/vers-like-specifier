use rstest::rstest;
use std::collections::BTreeSet;
use vers_like_specifier::{ConstraintError, Vls, VlsError};

#[rstest]
#[case::empty("")]
#[case::only_whitespace(" ")]
#[case::only_multi_whitespace("   ")]
fn parse_empty_string_is_error(#[case] input: &str) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::EmptyInput));
}

#[test]
fn parse_wildcard_is_error() {
    let err = "*".parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::ForbiddenAnyUsed));
}

#[rstest]
#[case::unicode_letter(">=1.0café", vec!['é'])]
#[case::unicode_in_both_segments("café|naïve", vec!['é', 'ï'])]
#[case::space_in_version(">=1.0 beta", vec![' '])]
#[case::whitespace_around_pipe(">=1.0 | <=2.0", vec![' '])]
#[case::comma(">=1,0", vec![','])]
#[case::at_sign(">=1.0@beta", vec!['@'])]
#[case::hash("1.0#rc1", vec!['#'])]
#[case::dollar_sign(">=1.0|<=2.0$beta", vec!['$'])]
#[case::space_and_comma(">=1,0 beta", vec![' ', ','])]
#[case::hash_and_at("1.0#rc1@beta", vec!['#', '@'])]
#[case::multiple_different("$1.0 @2.0#3", vec![' ', '#', '$', '@'])]
#[case::star_after_version_string(">=1.0*", vec!['*'])]
#[case::star_before_version_string(">=*1.0", vec!['*'])]
#[case::star_before_comparator("*>=1.0*", vec!['*'])]
#[case::vers_uri_prefix("vers:gem/>=2.2.0", vec!['/', ':'])]
#[case::vers_uri_prefix_wildcard("vers:all/*", vec!['*', '/', ':'])]
#[case::vers_type("gem/>=2.2.0", vec!['/'])]
#[case::only_type_delimiter("/>=2.2.0", vec!['/'])]
#[case::star_and_unicode("*1.0café", vec!['*', 'é'])]
fn parse_invalid_constraints_chars_is_error(
    #[case] input: &str,
    #[case] expected_chars: Vec<char>,
) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::InvalidCharacters(_)));
    if let VlsError::InvalidCharacters(chars) = err {
        assert_eq!(chars, expected_chars);
    }
}

#[rstest]
#[case::only_pipes("|||", 4)]
#[case::pipe_between_valid("<4.2||>6", 1)]
#[case::multiple_consecutive_pipes("<4.2|||>6", 2)]
#[case::leading_pipe("|<4.2|>6", 1)]
#[case::trailing_pipe("<4.2|>6|", 1)]
#[case::leading_and_trailing_pipes("||<4.2|>6||", 4)]
#[case::leading_between_trailing("||<4.2|||>6||", 6)]
fn parse_empty_constraint_is_error(#[case] input: &str, #[case] expected_count: usize) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::InvalidConstraints(_)));
    if let VlsError::InvalidConstraints(errors) = err {
        assert_eq!(errors.len(), expected_count);
        assert!(
            errors
                .iter()
                .all(|e| matches!(e, ConstraintError::EmptyConstraint))
        );
    }
}

#[rstest]
#[case::bare_not_equal("!=", 1)]
#[case::bare_less_than("<", 1)]
#[case::bare_less_than_or_equal("<=", 1)]
#[case::bare_greater_than(">", 1)]
#[case::bare_greater_than_or_equal(">=", 1)]
#[case::two_bare_comparators(">=|<=", 2)]
#[case::bare_with_valid(">|<=2.0", 1)]
#[case::valid_with_bare(">=1.0|<", 1)]
#[case::multiple_bare("!=|<", 2)]
fn parse_empty_version_is_error(#[case] input: &str, #[case] expected_count: usize) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::InvalidConstraints(_)));
    if let VlsError::InvalidConstraints(errors) = err {
        assert_eq!(errors.len(), expected_count);
        assert!(
            errors
                .iter()
                .all(|e| matches!(e, ConstraintError::EmptyConstraintVersion))
        );
    }
}

#[rstest]
#[case::bare_explicit_equals("=", vec![vec!['=']])]
#[case::explicit_equals("=3.2.1", vec![vec!['=']])]
#[case::equals_in_version(">=1.0=2", vec![vec!['=']])]
#[case::bang_in_version(">=1.0!beta", vec![vec!['!']])]
#[case::less_than_in_version(">=1.0<2", vec![vec!['<']])]
#[case::greater_than_in_version(">=1.0>2", vec![vec!['>']])]
#[case::multiple_invalid_in_version(">=1.0=!beta", vec![vec!['!', '=']])]
#[case::space_in_version(">=1.0 beta", vec![vec![' ']])]
#[case::multiple_spaces_in_version(">=1.0  beta  rc1", vec![vec![' ']])]
#[case::space_with_otherwise_valid_punctuation(">=1,0 beta", vec![vec![' ']])]
#[case::bare_percent_sign(">=1.0%gobeta", vec![vec!['%']])]
#[case::percent_with_invalid_hex(">=1.0%zzbeta", vec![vec!['%']])]
#[case::percent_with_one_invalid_hex_digit(">=1.0%4zbeta", vec![vec!['%']])]
#[case::percent_with_incomplete_hex(">=1.0%3", vec![vec!['%']])]
#[case::percent_at_end_of_version(">=1.0%", vec![vec!['%']])]
#[case::multiple_bare_percents(">=1.0%go%zz", vec![vec!['%']])]
#[case::literal_reserved_char_still_invalid_alongside_valid_escape(">=1.0%41!beta", vec![vec!['!']])]
fn parse_invalid_version_characters_is_error(
    #[case] input: &str,
    #[case] expected_chars: Vec<Vec<char>>,
) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::InvalidConstraints(_)));
    if let VlsError::InvalidConstraints(errors) = err {
        assert_eq!(errors.len(), expected_chars.len());
        for (i, error) in errors.iter().enumerate() {
            match error {
                ConstraintError::InvalidConstraintVersionCharacters(chars) => {
                    assert_eq!(chars, &expected_chars[i]);
                }
                other => panic!("Expected InvalidVersionCharacters, got: {other:?}"),
            }
        }
    }
}

#[rstest]
#[case::same_comparator(">1.0|>1.0", vec!["1.0"])]
#[case::different_comparator("!=3.0|>3.0", vec!["3.0"])]
#[case::different_comparator_equal_and_not_equal("!=3.0|3.0", vec!["3.0"])]
#[case::triple_duplicate(">=1.0|!=1.0|1.0", vec!["1.0"])]
#[case::multiple_different_duplicates("1.0|>2.0|!=1.0|>=2.0", vec!["1.0", "2.0"])]
#[case::duplicate_unique_mixed("!=1.0|<=2.0|>1.0", vec!["1.0"])]
fn parse_duplicate_constraints_is_error(#[case] input: &str, #[case] expected: Vec<&str>) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::DuplicateConstraintVersions(_)));
    if let VlsError::DuplicateConstraintVersions(dupes) = err {
        let expected: BTreeSet<String> = expected.into_iter().map(String::from).collect();
        assert_eq!(dupes, expected);
    }
}
