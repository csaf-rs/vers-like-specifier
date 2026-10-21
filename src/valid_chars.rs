/// The number of valid ASCII codepoints (0–127)
const ASCII_TABLE_SIZE: usize = 128;

/// A lookup table mapping ASCII codepoints (0–127) to character validity
type AsciiLookup = [bool; ASCII_TABLE_SIZE];

/// O(1) compile-time generated lookup of ASCII characters validity.
/// Creates an `AsciiLookup`, `valid_special` ASCII characters are mapped to their codepoint (0 - 127).
/// If the char is valid, the bool in the resulting `AsciiLookup` is set to true.
const fn build_lookup(valid_special: &str) -> AsciiLookup {
    let mut table = [false; ASCII_TABLE_SIZE];
    let bytes = valid_special.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if (bytes[i] as usize) < ASCII_TABLE_SIZE {
            table[bytes[i] as usize] = true;
        } else {
            panic!(
                "Only ASCII characters (0-127) are allowed in the special character set lookup, found a byte value >= 128. This is a developer error."
            );
        }
        i += 1;
    }
    table
}

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
    fn get_lookup(&self) -> &'static AsciiLookup {
        // Generate compile-time lookups.
        //
        // The version-string special characters follow the ECMA VERS specification
        // (https://ecma-tc54.github.io/ECMA-xxx-VERS/), section 5.3.3.2: a `version` may
        // contain any printable ASCII punctuation character, *except* for the seven characters
        // VERS reserves for `comparator`s and the `|` segment delimiter (`>`, `<`, `=`, `!`,
        // `*`, `|`, `%`), which VERS requires to be percent-encoded when they appear literally
        // in a version. Percent-encoded triplets are handled separately by
        // `collect_invalid_version_characters`.
        const VERSION_STRING: AsciiLookup = build_lookup("\"#$&'()+,-./:;?@[\\]^_`{}~");
        // The constraints-level special characters are the version-string special characters,
        // plus the comparator characters (`=`, `!`, `<`, `>`), the `|` segment delimiter, and
        // `%` (allowed unconditionally at this coarse level; whether a given `%` starts a
        // well-formed percent-encoded triplet is checked later, at the version-string level).
        const CONSTRAINTS_STRING: AsciiLookup = build_lookup("\"#$&'()+,-./:;?@[\\]^_`{}~=!<>|%");
        match self {
            VlsSpecialCharSet::VersionString => &VERSION_STRING,
            VlsSpecialCharSet::ConstraintsString => &CONSTRAINTS_STRING,
        }
    }
}

/// Collects characters from `input` that are **not** ASCII-alphanumeric and **not**
/// contained in `special_charset`, returning them sorted and deduplicated.
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

    let lookup = special_charset.get_lookup();
    let mut invalid: Vec<char> = input
        .chars()
        .filter(|ch| {
            let idx = *ch as usize;
            !(ch.is_ascii_alphanumeric()
                || (idx < ASCII_TABLE_SIZE && lookup[idx])
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
    let lookup = VlsSpecialCharSet::VersionString.get_lookup();
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

        let idx = ch as usize;
        let is_valid = ch.is_ascii_alphanumeric() || (idx < ASCII_TABLE_SIZE && lookup[idx]);
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
