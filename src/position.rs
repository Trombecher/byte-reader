use core::marker::PhantomData;
use core::slice;

/// A cursor position.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Position<'a>(*const u8, PhantomData<&'a u8>);

impl<'a> Position<'a> {
    /// Creates a new position.
    #[inline]
    pub(crate) const fn new(ptr: *const u8) -> Self {
        Self(ptr, PhantomData)
    }

    /// Returns the slice bounded by `self` and the parameter `next`.
    ///
    /// **Panics if `self > next`**.
    #[inline]
    pub fn slice_to(self, next: Position<'a>) -> &'a [u8] {
        let size = unsafe { next.0.offset_from(self.0) };

        if size < 0 {
            panic!("Next position is previous");
        }

        unsafe { slice::from_raw_parts(self.0, size as usize) }
    }
}
