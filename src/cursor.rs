use crate::Position;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;

macro_rules! impl_read_n {
    ($le:ident,$be:ident,$nle:ident,$nbe:ident,$t:tt) => {
        impl<'a> Cursor<'a> {
            #[inline]
            pub fn $le(&self) -> Option<$t> {
                if self.bytes_remaining() < size_of::<$t>() {
                    None
                } else {
                    Some($t::from_le(unsafe {
                        (self.cursor as *const $t).read_unaligned()
                    }))
                }
            }

            #[inline]
            pub fn $be(&self) -> Option<$t> {
                if self.bytes_remaining() < size_of::<$t>() {
                    None
                } else {
                    Some($t::from_be(unsafe {
                        (self.cursor as *const $t).read_unaligned()
                    }))
                }
            }

            #[inline]
            pub fn $nle(&mut self) -> Option<$t> {
                self.$le().inspect(|_| unsafe {
                    self.advance_n_unchecked(size_of::<$t>());
                })
            }

            #[inline]
            pub fn $nbe(&self) -> Option<$t> {
                if self.bytes_remaining() < size_of::<$t>() {
                    None
                } else {
                    Some($t::from_be(unsafe {
                        (self.cursor as *const $t).read_unaligned()
                    }))
                }
            }
        }
    };
}

#[derive(Clone, PartialEq, Eq)]
/// An iterator over a slice.
pub struct Cursor<'a> {
    /// The pointer to the first element.
    start: *const u8,

    /// The pointer to the next element.
    cursor: *const u8,

    /// The pointer to the past-the-end element.
    end: *const u8,

    /// The marker for ownership of `&[u8]`.
    _marker: PhantomData<&'a [u8]>,
}

/// All errors [crate::bytes] can produce.
#[repr(u8)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    /// Encountered a continuation byte where the byte 1 was expected.
    EncounteredContinuationByte,

    /// The input ended while decoding the second byte of a two byte sequence.
    Missing2ndOf2,

    /// The second byte of a two byte sequence is not a continuation byte.
    Invalid2ndOf2,

    /// The input ended while decoding the second byte of a three byte sequence.
    Missing2ndOf3,

    /// The second byte of a three byte sequence is not a continuation byte.
    Invalid2ndOf3,

    /// The input ended while decoding the third byte of a three byte sequence.
    Missing3rdOf3,

    /// The third byte of a three byte sequence is not a continuation byte.
    Invalid3rdOf3,

    /// The input ended while decoding the second byte of a four byte sequence.
    Missing2ndOf4,

    /// The second byte of a four byte sequence is not a continuation byte.
    Invalid2ndOf4,

    /// The input ended while decoding the third byte of a four byte sequence.
    Missing3rdOf4,

    /// The third byte of a four byte sequence is not a continuation byte.
    Invalid3rdOf4,

    /// The input ended while decoding the fourth byte of a four byte sequence.
    Missing4thOf4,

    /// The fourth byte of a four byte sequence is not a continuation byte.
    Invalid4thOf4,
}

impl_read_n!(read_u16_le, read_u16_be, next_u16_le, next_u16_be, u16);
impl_read_n!(read_i16_le, read_i16_be, next_i16_le, next_i16_be, i16);
impl_read_n!(read_u32_le, read_u32_be, next_u32_le, next_u32_be, u32);
impl_read_n!(read_i32_le, read_i32_be, next_i32_le, next_i32_be, i32);
impl_read_n!(read_u64_le, read_u64_be, next_u64_le, next_u64_be, u64);
impl_read_n!(read_i64_le, read_i64_be, next_i64_le, next_i64_be, i64);
impl_read_n!(read_u128_le, read_u128_be, next_u18_le, next_u128_be, u128);
impl_read_n!(read_i128_le, read_i128_be, next_i128_le, next_i128_be, i128);

// F32 and F64 impls
impl Cursor<'_> {
    #[inline]
    pub fn read_f32_le(&self) -> Option<f32> {
        self.read_u32_le().map(f32::from_bits)
    }

    #[inline]
    pub fn read_f32_be(&self) -> Option<f32> {
        self.read_u32_be().map(f32::from_bits)
    }

    #[inline]
    pub fn next_f32_le(&mut self) -> Option<f32> {
        self.next_u32_le().map(f32::from_bits)
    }

    #[inline]
    pub fn next_f32_be(&mut self) -> Option<f32> {
        self.next_u32_be().map(f32::from_bits)
    }

    #[inline]
    pub fn read_f64_le(&self) -> Option<f64> {
        self.read_u64_le().map(f64::from_bits)
    }

    #[inline]
    pub fn read_f64_be(&self) -> Option<f64> {
        self.read_u64_be().map(f64::from_bits)
    }

    #[inline]
    pub fn next_f64_le(&mut self) -> Option<f64> {
        self.next_u64_le().map(f64::from_bits)
    }

    #[inline]
    pub fn next_f64_be(&mut self) -> Option<f64> {
        self.next_u64_be().map(f64::from_bits)
    }
}

impl<'a> Cursor<'a> {
    /// Advances until the next byte is not ASCII whitespace.
    ///
    /// Whitespace being defined by [char::is_ascii_whitespace].
    #[inline]
    pub fn skip_ascii_whitespace(&mut self) {
        loop {
            match self.peek() {
                None => break,
                Some(x) if !x.is_ascii_whitespace() => break,
                Some(_) => unsafe { self.advance_unchecked() },
            }
        }
    }

    /// Returns a position.
    #[inline]
    pub const fn position(&self) -> Position<'a> {
        Position::new(self.cursor)
    }

    /// Constructs a new [Cursor] from a slice.
    #[inline]
    pub const fn new(slice: &[u8]) -> Self {
        Self {
            start: slice.as_ptr(),
            cursor: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            _marker: PhantomData,
        }
    }

    /// Returns the number of bytes consumed.
    #[inline]
    pub fn bytes_consumed(&self) -> usize {
        self.cursor as usize - self.start as usize
    }

    /// Gets the next byte. Normalizes line terminators by mapping CR, CRLF and LF sequences to LF.
    #[inline]
    pub fn next_lfn(&mut self) -> Option<u8> {
        match self.peek() {
            None => None,
            Some(b'\r') => {
                unsafe { self.advance_unchecked() }

                if self.peek() == Some(b'\n') {
                    // SAFETY: Because of `Some(...)` there is a next byte.
                    self.cursor = unsafe { self.cursor.add(1) };
                }
                Some(b'\n')
            }
            x => {
                unsafe { self.advance_unchecked() }
                x
            }
        }
    }

    /// Gets the next byte. Does not normalize line terminators. Advances.
    #[inline]
    #[doc(alias = "next")]
    pub fn next_byte(&mut self) -> Option<u8> {
        if self.has_next() {
            let byte = unsafe { self.peek_unchecked() };
            unsafe { self.advance_unchecked() };
            Some(byte)
        } else {
            None
        }
    }

    /// Gets the next byte. Does not normalize line terminators.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the cursor has a next byte.
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> u8 {
        unsafe {
            let byte = self.peek_unchecked();
            self.advance_unchecked();
            byte
        }
    }

    /// Peeks into the next byte. Does not advance the iterator.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if !self.has_next() {
            None
        } else {
            Some(unsafe { self.peek_unchecked() })
        }
    }

    /// Peeks into the nth byte, first byte is n=0. Does not advance.
    #[inline]
    pub fn peek_n(&self, n: usize) -> Option<u8> {
        // SAFETY: `self.cursor` is strictly less than `self.end`
        let delta = unsafe { self.end.offset_from(self.cursor) } as usize;
        (n < delta).then(|| {
            // SAFETY: `self.cursor` offset by `n` is not out-of-bounds,
            // as checked above.
            let ptr = unsafe { self.cursor.add(n) };
            // SAFETY: `ptr` is a valid pointer to a `u8` and lies within
            // the bounds of the owned allocation.
            unsafe { *ptr }
        })
    }

    /// Returns the number of bytes remaining.
    #[inline]
    pub fn bytes_remaining(&self) -> usize {
        (self.end as usize).saturating_sub(self.cursor as usize)
    }

    /// Checks if the cursor has a next byte.
    #[inline]
    pub fn has_next(&self) -> bool {
        self.cursor < self.end
    }

    /// Peeks into the next byte. Does not advance the iterator.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the cursor has a next byte.
    #[inline]
    pub unsafe fn peek_unchecked(&self) -> u8 {
        unsafe { *self.cursor }
    }

    /// Advances one char, saturates at the upper boundary.
    #[inline]
    pub fn advance(&mut self) {
        if self.has_next() {
            unsafe {
                self.advance_unchecked();
            }
        }
    }

    /// Advances the cursor one byte.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the cursor is not at the end.
    #[inline]
    pub unsafe fn advance_unchecked(&mut self) {
        unsafe { self.cursor = self.cursor.add(1) }
    }

    /// Advances n bytes.
    ///
    /// # Safety
    ///
    /// Advancing the read cursor `n` must not be out-of-bounds for the
    /// underlying allocation.
    #[inline]
    pub unsafe fn advance_n_unchecked(&mut self, n: usize) {
        unsafe {
            self.cursor = self.cursor.add(n);
        }
    }

    /// Advances a UTF-8 character without checking for bounds.
    ///
    /// # Safety
    ///
    /// Advancing the read cursor `n` must not be out-of-bounds for the
    /// underlying allocation.
    #[inline]
    pub unsafe fn advance_char_unchecked(&mut self) {
        unsafe {
            self.cursor = self
                .cursor
                .add(UTF8_CHAR_WIDTH[self.peek_unchecked() as usize] as usize);
        }
    }

    /// Advances the cursor by one char encoded as UTF-8.
    #[inline]
    pub fn advance_char(&mut self) -> Result<(), Error> {
        let first_byte = match self.next() {
            Some(x) => x,
            None => return Ok(()),
        };

        macro_rules! next {
            ($e:expr,$i:expr) => {
                match self.next_lfn() {
                    None => return Err($e),
                    Some(x) if x & 0b1100_0000 != 0b1000_0000 => return Err($i),
                    _ => {}
                }
            };
        }

        match UTF8_CHAR_WIDTH[first_byte as usize] {
            0 => Err(Error::EncounteredContinuationByte),
            1 => {
                if first_byte == b'\r' && self.peek() == Some(b'\n') {
                    unsafe { self.advance_unchecked() }
                }
                Ok(())
            }
            2 => {
                next!(Error::Missing2ndOf2, Error::Invalid2ndOf2);
                Ok(())
            }
            3 => {
                next!(Error::Missing2ndOf3, Error::Invalid2ndOf3);
                next!(Error::Missing3rdOf3, Error::Invalid3rdOf3);
                Ok(())
            }
            4 => {
                next!(Error::Missing2ndOf4, Error::Invalid2ndOf4);
                next!(Error::Missing3rdOf4, Error::Invalid3rdOf4);
                next!(Error::Missing4thOf4, Error::Invalid4thOf4);
                Ok(())
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl Iterator for Cursor<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_byte()
    }
}

const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];
