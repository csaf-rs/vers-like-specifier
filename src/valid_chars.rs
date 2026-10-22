/// The seven characters VERS reserves for `comparator`s, the `*` ("any version") token, and the
/// `|` segment delimiter. These must be percent-encoded if they are to appear literally within a
/// `version-string` (see ECMA VERS section 5.3.3.2).
const RESERVED_VERSION_CHARS: [char; 7] = ['>', '<', '=', '!', '*', '|', '%'];

/// Identifies which set of characters is considered valid in a given context.
pub enum VlsSpecialCharSet {
    /// Valid characters for a `version-string`.
    /// See [`Vls`](crate::Vls) for more details on the grammar.
    VersionString,
    /// Valid characters for a `constraints` string.
    /// See [`Vls`](crate::Vls) for more details on the grammar.
    ConstraintsString,
}

impl VlsSpecialCharSet {
    /// Returns `true` if `ch` is an ASCII punctuation character allowed, unencoded, in this
    /// character set.
    ///
    /// A VLS string may contain any printable ASCII punctuation character
    /// (`char::is_ascii_punctuation`), except for a handful reserved for grammar purposes:
    ///
    /// - For `VersionString`, the seven characters VERS reserves for `comparator`s and the `|`
    ///   segment delimiter (`>`, `<`, `=`, `!`, `*`, `|`, `%`) must instead appear in their
    ///   percent-encoded form; well-formed percent-encoded triplets are handled separately by
    ///   `collect_invalid_version_characters`.
    /// - For `ConstraintsString`, only `*` (the "any version" token, forbidden everywhere in a
    ///   VLS string) is excluded; the other six reserved characters are allowed unconditionally
    ///   at this coarse level, since they form part of the `comparator` / `|` delimiter syntax.
    ///   Whether a literal, un-encoded occurrence of one of them is valid within a given version
    ///   string is checked later, at the version-string level.
    fn is_valid_punctuation(&self, ch: char) -> bool {
        if !ch.is_ascii_punctuation() {
            return false;
        }
        match self {
            VlsSpecialCharSet::VersionString => !RESERVED_VERSION_CHARS.contains(&ch),
            VlsSpecialCharSet::ConstraintsString => ch != '*',
        }
    }
}

/// Collects characters from `input` that are **not** ASCII-alphanumeric and **not** considered
/// valid punctuation for `special_charset`, returning them sorted and deduplicated.
///
/// Whitespace is always considered valid at this level, as it is permitted everywhere in a VLS
/// string except within a version string itself.
///
/// For VersionString, validation is delegated to `collect_invalid_version_characters`, which
/// is aware of well-formed percent-encoded triplets.
///
/// Returns `None` if every character is valid.
pub fn collect_invalid_characters(
    input: &str,
    special_charset: VlsSpecialCharSet,
) -> Option<Vec<char>> {
    if matches!(special_charset, VlsSpecialCharSet::VersionString) {
        return collect_invalid_version_characters(input);
    }

    let mut invalid: Vec<char> = input
        .chars()
        .filter(|ch| {
            !(ch.is_ascii_alphanumeric()
                || special_charset.is_valid_punctuation(*ch)
                || ch.is_whitespace())
        })
        .collect();

    if invalid.is_empty() {
        return None;
    }

    invalid.sort_unstable();
    invalid.dedup();
    Some(invalid)
}

/// Collects characters from a `version-string` candidate that are invalid, treating any
/// well-formed percent-encoded triplet (`%` followed by two ASCII hex digits, per RFC 3986) as
/// valid, while still rejecting:
///
/// - a literal, un-encoded occurrence of a reserved character (`>`, `<`, `=`, `!`, `*`, `|`,
///   `%`), which must instead be percent-encoded per ECMA VERS section 5.3.3.2;
/// - a `%` that is not followed by two ASCII hex digits (e.g. a malformed or truncated escape);
/// - any character outside of `ALPHA` / `DIGIT` / the version-string special characters.
///
/// Note that validity here does not depend on what a triplet decodes to: `%41` (which decodes
/// to the letter `A`) is just as well-formed as `%3E` (which decodes to the reserved `>`). This
/// crate does not implement percent-decoding, so the percent-encoded sequence itself is left
/// untouched in the stored version string; this function only validates that it is
/// syntactically well-formed.
///
/// Returns `None` if every character (or well-formed percent-encoded triplet) is valid.
fn collect_invalid_version_characters(input: &str) -> Option<Vec<char>> {
    let chars: Vec<char> = input.chars().collect();
    let mut invalid: Vec<char> = Vec::new();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '%' {
            let is_well_formed_triplet = chars
                .get(i + 1..i + 3)
                .is_some_and(|hex| hex.iter().all(char::is_ascii_hexdigit));

            if is_well_formed_triplet {
                i += 3;
            } else {
                invalid.push('%');
                i += 1;
            }
            continue;
        }

        let is_valid =
            ch.is_ascii_alphanumeric() || VlsSpecialCharSet::VersionString.is_valid_punctuation(ch);
        if !is_valid {
            invalid.push(ch);
        }
        i += 1;
    }

    if invalid.is_empty() {
        return None;
    }

    invalid.sort_unstable();
    invalid.dedup();
    Some(invalid)
}
