//! Structures and procedures relevant to the Portable Document Format.


use std::io::{self, Write};


/// Writes out a textual string in PDF format.
///
/// The string is wrapped in parentheses (`(` and `)`), encoded in UTF-16BE with BOM, and all
/// backslashes and parentheses are escaped with a preceding backslash.
pub fn write_pdf_string<W: Write>(string: &str, mut writer: W) -> Result<(), io::Error> {
    const ESCAPE_US: [u16; 3] = [
        b'(' as u16,
        b')' as u16,
        b'\\' as u16,
    ];

    writer.write_all(b"(\xFE\xFF")?;
    for word in string.encode_utf16() {
        if ESCAPE_US.contains(&word) {
            // precede with a backslash
            writer.write_all(b"\x00\x5C")?;
        }
        let word_be_bytes = word.to_be_bytes();
        writer.write_all(&word_be_bytes)?;
    }
    writer.write_all(b")")?;
    Ok(())
}
