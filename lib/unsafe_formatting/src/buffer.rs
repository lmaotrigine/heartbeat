use core::{borrow::Borrow, fmt, ops::Deref};

use crate::{Integer, MAX_BUF_LEN};

/// A stack allocated buffer for the formatted integer to be written into.
#[derive(Clone, Copy)]
pub struct Buffer {
    pub(crate) inner: [u8; MAX_BUF_LEN],
    pub(crate) pos: usize,
    pub(crate) end: usize,
}

impl Buffer {
    /// Creates a new buffer.
    ///
    /// However, I'm lazy so this allocates enough on the stack to format
    /// `u128::MAX`.
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: [0; MAX_BUF_LEN],
            pos: MAX_BUF_LEN,
            end: MAX_BUF_LEN,
        }
    }

    #[inline(always)]
    #[must_use]
    /// Returns a view of the buffer as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner[self.pos..self.end]
    }

    #[inline(always)]
    #[must_use]
    /// Returns a view of the buffer as a string slice.
    pub fn as_str(&self) -> &str {
        // SAFETY: digits are always ASCII, and our only separator is ,
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    #[inline(always)]
    #[must_use]
    /// Whether the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    #[must_use]
    /// The length of the buffer that is actually used.
    pub const fn len(&self) -> usize {
        self.end - self.pos
    }

    #[inline(always)]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut u8 {
        self.inner.as_mut_ptr()
    }

    #[inline(always)]
    pub(crate) fn write_with_itoa<N: itoa::Integer>(&mut self, n: N) -> usize {
        let mut buf_itoa = itoa::Buffer::new();
        let s = buf_itoa.format(n);
        let s_len = s.len();
        self.pos = MAX_BUF_LEN - s_len;
        self.end = MAX_BUF_LEN;
        let dst = &mut self.inner[self.pos..self.end];
        dst.copy_from_slice(s.as_bytes());
        s_len
    }

    #[inline(always)]
    pub(crate) fn write_one_byte(&mut self, sep_pos: &mut usize, idx: isize) {
        self.pos -= 1;
        if *sep_pos == self.pos {
            unsafe {
                core::ptr::copy_nonoverlapping(crate::SEPARATOR, self.as_mut_ptr().add(self.pos), 1);
            }
            *sep_pos -= 4;
            self.pos -= 1;
        }
        unsafe {
            core::ptr::copy_nonoverlapping(crate::TABLE.as_ptr().offset(idx), self.as_mut_ptr().add(self.pos), 1);
        }
    }

    #[inline(always)]
    pub(crate) fn write_two_bytes(&mut self, sep_pos: &mut usize, idx: isize) {
        self.write_one_byte(sep_pos, idx + 1);
        self.write_one_byte(sep_pos, idx);
    }

    /// Writes an integer to the buffer.
    #[inline(always)]
    pub fn write<I: Integer>(&mut self, n: &I) -> usize {
        n.to_buffer(self)
    }
}

impl AsRef<str> for Buffer {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Buffer {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Buffer {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Buffer {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
