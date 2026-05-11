use rstest::rstest;
use std::collections::BTreeSet;
use vls::{VersionConstraintError, Vls, VlsError};

#[test]
fn parse_empty_string_is_error() {
    let err = "".parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::EmptyInput));
}

#[rstest]
#[case::vers("vers:gem/>=2.2.0")]
#[case::vers_all("vers:all/*")]
fn parse_vers_prefix_is_error(#[case] input: &str) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::ContainsVersPrefix));
}

#[rstest]
#[case::gem_scheme("gem/>=2.2.0")]
#[case::only_scheme_delimiter("/>=2.2.0")]
fn parse_bare_scheme_is_error(#[case] input: &str) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::ContainsVersioningScheme));
}

#[rstest]
#[case::space_in_version(">=1.0 beta", vec![' '])]
#[case::leading_whitespace(" >=1.0", vec![' '])]
#[case::trailing_whitespace(">=1.0 ", vec![' '])]
#[case::whitespace_around_pipe(">=1.0 | <=2.0", vec![' '])]
#[case::only_whitespace(" ", vec![' '])]
#[case::only_multi_whitespace("   ", vec![' '])]
#[case::comma(">=1,0", vec![','])]
#[case::at_sign(">=1.0@beta", vec!['@'])]
#[case::hash("1.0#rc1", vec!['#'])]
#[case::dollar_sign(">=1.0|<=2.0$beta", vec!['$'])]
#[case::space_and_comma(">=1,0 beta", vec![' ', ','])]
#[case::hash_and_at("1.0#rc1@beta", vec!['#', '@'])]
#[case::multiple_different("$1.0 @2.0#3", vec![' ', '#', '$', '@'])]
#[case::star_in_version(">=1.0*", vec!['*'])]
#[case::star_and_equals_equals_is_valid(">=1.0*=2", vec!['*'])]
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
    assert!(matches!(err, VlsError::InvalidConstraintError(_)));
    if let VlsError::InvalidConstraintError(errors) = err {
        assert_eq!(errors.len(), expected_count);
        assert!(
            errors
                .iter()
                .all(|e| matches!(e, VersionConstraintError::EmptyConstraint))
        );
    }
}

#[rstest]
#[case::bare_equal("=", 1)]
#[case::bare_not_equal("!=", 1)]
#[case::bare_less_than("<", 1)]
#[case::bare_less_than_or_equal("<=", 1)]
#[case::bare_greater_than(">", 1)]
#[case::bare_greater_than_or_equal(">=", 1)]
#[case::two_bare_comparators(">=|<=", 2)]
#[case::bare_with_valid(">|<=2.0", 1)]
#[case::valid_with_bare(">=1.0|<", 1)]
#[case::multiple_bare("=|!=|<", 3)]
fn parse_empty_version_is_error(#[case] input: &str, #[case] expected_count: usize) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::InvalidConstraintError(_)));
    if let VlsError::InvalidConstraintError(errors) = err {
        assert_eq!(errors.len(), expected_count);
        assert!(
            errors
                .iter()
                .all(|e| matches!(e, VersionConstraintError::EmptyVersion))
        );
    }
}

#[rstest]
#[case::equals_in_version(">=1.0=2", vec![vec!['=']])]
#[case::bang_in_version(">=1.0!beta", vec![vec!['!']])]
#[case::less_than_in_version(">=1.0<2", vec![vec!['<']])]
#[case::greater_than_in_version(">=1.0>2", vec![vec!['>']])]
#[case::multiple_invalid_in_version(">=1.0=!beta", vec![vec!['!', '=']])]
fn parse_invalid_version_characters_is_error(
    #[case] input: &str,
    #[case] expected_chars: Vec<Vec<char>>,
) {
    let err = input.parse::<Vls>().unwrap_err();
    assert!(matches!(err, VlsError::InvalidConstraintError(_)));
    if let VlsError::InvalidConstraintError(errors) = err {
        assert_eq!(errors.len(), expected_chars.len());
        for (i, error) in errors.iter().enumerate() {
            match error {
                VersionConstraintError::InvalidVersionCharacters(chars) => {
                    assert_eq!(chars, &expected_chars[i]);
                }
                other => panic!("Expected InvalidVersionCharacters, got: {:?}", other),
            }
        }
    }
}

#[rstest]
#[case::same_comparator(">1.0|>1.0", vec!["1.0"])]
#[case::different_comparator("!=3.0|>3.0", vec!["3.0"])]
#[case::different_comparator_impl_expl_equals("!=3.0|=3.0", vec!["3.0"])]
#[case::triple_duplicate(">=1.0|!=1.0|=1.0", vec!["1.0"])]
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
