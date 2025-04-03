#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

//! A crate providing utilities for slice iteration.

mod cursor;
mod position;
mod tests;

pub use cursor::*;
pub use position::*;

/// Calculates line:col from a source string slice and an offset. CRLF sequences are treated as
/// one line break.
///
/// May be useful for error messages.
///
/// **Lines and columns are 0-indexed**, meaning the first line (or the first column) is 0.
pub fn get_lines_and_columns(source: &str, mut byte_offset: usize) -> (usize, usize) {
    let mut lines = 0;
    let mut columns = 0;
    let mut cursor = Cursor::new(source.as_bytes());

    while byte_offset > 0 {
        match cursor.peek() {
            Some(b'\r') => {
                lines += 1;
                columns = 0;

                unsafe { cursor.advance_unchecked() }

                if let Some(b'\n') = cursor.peek() {
                    unsafe { cursor.advance_unchecked() }
                    byte_offset -= 1;
                }
            }
            Some(b'\n') => {
                lines += 1;
                columns = 0;

                unsafe { cursor.advance_unchecked() }
            }
            Some(_) => {
                columns += 1;
                unsafe { cursor.advance_char_unchecked() }
            }
            None => break,
        }

        byte_offset -= 1;
    }

    (lines, columns)
}
