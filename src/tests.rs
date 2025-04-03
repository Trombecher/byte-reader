#![cfg(test)]

use crate::Cursor;

#[test]
fn get_lines_and_columns() {
    assert_eq!(
        crate::get_lines_and_columns(
            "Is this the real life?\nIs this just fantasy\r\nSome other line",
            50
        ),
        (2, 5)
    )
}

#[test]
fn skip_ascii_whitespace() {
    let mut cursor = Cursor::new(b"a\n\t\x0C\r");

    cursor.skip_ascii_whitespace();
    assert_eq!(cursor.peek(), Some(b'a'));

    cursor.advance();
    cursor.skip_ascii_whitespace();

    assert_eq!(cursor.peek(), None)
}

#[test]
fn position() {
    let slice = b"abc";
    let mut cursor = Cursor::new(slice);

    cursor.advance();

    let position = cursor.position();

    cursor.advance();

    assert_eq!(&slice[1..2], position.slice_to(cursor.position()));
}

#[test]
fn bytes_consumed() {
    let mut cursor = Cursor::new(b"ab");
    assert_eq!(cursor.bytes_consumed(), 0);
    cursor.advance();
    assert_eq!(cursor.bytes_consumed(), 1);
    cursor.advance();
    assert_eq!(cursor.bytes_consumed(), 2);
}

#[test]
fn next_lfn() {
    let mut cursor = Cursor::new("A\r\nB\nC\rD".as_bytes());
    assert_eq!(cursor.next_lfn(), Some(b'A'));
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
    assert_eq!(cursor.next_lfn(), Some(b'B'));
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
    assert_eq!(cursor.next_lfn(), Some(b'C'));
    assert_eq!(cursor.next_lfn(), Some(b'\n'));
    assert_eq!(cursor.next_lfn(), Some(b'D'));
    assert_eq!(cursor.next_lfn(), None);
    assert_eq!(cursor.next_lfn(), None);
}

#[test]
fn next() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.next(), Some(b'A'));
    assert_eq!(cursor.next(), Some(b'B'));
    assert_eq!(cursor.next(), None);
}

#[test]
fn peek() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.peek(), Some(b'A'));
    assert_eq!(cursor.peek(), Some(b'A'));

    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek(), Some(b'B'));

    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.peek(), None);
}

#[test]
fn peek_n() {
    let cursor = Cursor::new("AB".as_bytes());

    assert_eq!(cursor.peek_n(0), Some(b'A'));
    assert_eq!(cursor.peek_n(1), Some(b'B'));
    assert_eq!(cursor.peek_n(2), None);
}

#[test]
fn bytes_remaining() {
    let mut cursor = Cursor::new("AB".as_bytes());
    assert_eq!(cursor.bytes_remaining(), 2);
    cursor.advance();
    assert_eq!(cursor.bytes_remaining(), 1);
    cursor.advance();
    assert_eq!(cursor.bytes_remaining(), 0);
}

#[test]
fn has_next() {
    let mut cursor = Cursor::new("A".as_bytes());
    assert_eq!(cursor.has_next(), true);

    unsafe { cursor.advance_unchecked() }
    assert_eq!(cursor.has_next(), false);
}

#[test]
fn advance_char() {
    let mut cursor = Cursor::new("AB€C".as_bytes());

    assert_eq!(cursor.advance_char(), Ok(()));
    assert_eq!(cursor.peek(), Some(b'B'));

    cursor.advance_char().unwrap();
    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek(), Some(b'C'));

    cursor.advance_char().unwrap();
    assert_eq!(cursor.peek(), None);
}
