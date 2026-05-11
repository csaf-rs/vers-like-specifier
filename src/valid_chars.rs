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
        if bytes[i] < ASCII_TABLE_SIZE as u8 {
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
    /// Valid characters for a `version-string` string.
    /// See vls::Vls for more details on the grammar.
    VersionString,
    /// Valid characters for a `constraints` string.
    /// See vls::Vls for more details on the grammar.
    ConstraintsString,
}

impl VlsSpecialCharSet {
    fn get_lookup(&self) -> &'static AsciiLookup {
        // Generate compile-time lookups
        const VERSION_STRING: AsciiLookup = build_lookup("-._+~");
        const CONSTRAINTS_STRING: AsciiLookup = build_lookup("-._+~=!<>|");
        match self {
            VlsSpecialCharSet::VersionString => &VERSION_STRING,
            VlsSpecialCharSet::ConstraintsString => &CONSTRAINTS_STRING,
        }
    }
}

/// Collects characters from `input` that are **not** ASCII-alphanumeric and **not**
/// contained in `special_charset`, returning them sorted and deduplicated.
///
/// Returns `None` if every character is valid.
pub fn collect_invalid_characters(
    input: &str,
    special_charset: VlsSpecialCharSet,
) -> Option<Vec<char>> {
    let lookup = special_charset.get_lookup();
    let mut invalid: Vec<char> = input
        .chars()
        .filter(|ch| {
            let idx = *ch as usize;
            // check if the char is ASCII alphanumeric and contained in the allowed special chars
            !ch.is_ascii_alphanumeric() && !(idx < ASCII_TABLE_SIZE && lookup[idx])
        })
        .collect();

    if invalid.is_empty() {
        return None;
    }

    invalid.sort();
    invalid.dedup();
    Some(invalid)
}
