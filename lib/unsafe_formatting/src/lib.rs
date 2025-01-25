#![no_std]
#![forbid(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    unused,
    clippy::all,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used
)]
#![deny(clippy::pedantic, trivial_numeric_casts)]
#![allow(clippy::inline_always)] // the whole point of this wankery is to do things like this

//! This crate is essentially itoa but with commas.
//!
//! It's very hacky, I don't think it would be very useful outside of my specific use cases, and there are probably
//! better crates out there that do this and much more. However, I just want something really fast, with no allocation,
//! and only the standard thousands separator (a comma, groups of 3); and this works.
//!
//! There's a lot of `unsafe`s strewn about here, I don't know that there's a better way to do this, but it works, and
//! it runs well on my network of tiny embedded devices, so that's all I care about.

use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroU8, NonZeroUsize,
};

const U128_MAX_LEN: usize = 39;
// A u128 can be up to 39 digits, which will have at most 12 separators.
pub(crate) const MAX_BUF_LEN: usize = U128_MAX_LEN + 12;
pub(crate) const SEPARATOR: *const u8 = b",".as_ptr();
pub(crate) const TABLE: &[u8; 200] = b"\
    00010203040506070809\
    10111213141516171819\
    20212223242526272829\
    30313233343536373839\
    40414243444546474849\
    50515253545556575859\
    60616263646566676869\
    70717273747576777879\
    80818283848586878889\
    90919293949596979899\
";

mod buffer;
mod macros;

pub use buffer::Buffer;

mod sealed {
    use crate::macros::impl_sealed;

    pub trait Sealed {}
    impl_sealed!(u8, u16, u32, u64, usize, u128);
    impl_sealed!(i8, i16, i32, i64, isize, i128);
}

macros::impl_sealed!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroUsize, NonZeroU64, NonZeroU128);
macros::impl_sealed!(NonZeroI8, NonZeroI16, NonZeroI32, NonZeroIsize, NonZeroI64, NonZeroI128);

/// Analogous to [`itoa::Integer`]. A marker trait for an integer that can be written into a [`Buffer`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait Integer: sealed::Sealed + Sized {
    /// Writes the integer into the buffer, returning the number of bytes written.
    fn to_buffer(&self, buf: &mut Buffer) -> usize;
}

impl Integer for u8 {
    #[inline(always)]
    fn to_buffer(&self, buf: &mut Buffer) -> usize {
        buf.write_with_itoa(*self)
    }
}

macros::impl_ux!(u16, u32, u64, usize, u128);
macros::impl_nonzero_ux!(
    NonZeroU16,
    u16,
    NonZeroU32,
    u32,
    NonZeroU64,
    u64,
    NonZeroUsize,
    usize,
    NonZeroU128,
    u128
);
macros::impl_ix!(i8, i16, i32, i64, isize, i128);

#[inline(always)]
#[allow(clippy::cast_possible_wrap)]
fn do_the_thing(mut n: u128, buf: &mut Buffer) -> usize {
    buf.pos = MAX_BUF_LEN;
    buf.end = MAX_BUF_LEN;
    let mut sep_pos = MAX_BUF_LEN - 4;
    while n >= 10_000 {
        let rem = n % 10_000;
        let idx = ((rem % 100) << 1) as isize;
        buf.write_two_bytes(&mut sep_pos, idx);
        let idx = ((rem / 100) << 1) as isize;
        buf.write_two_bytes(&mut sep_pos, idx);
        n /= 10_000;
    }
    let mut n = n as isize;
    while n >= 100 {
        let idx = (n % 100) << 1;
        buf.write_two_bytes(&mut sep_pos, idx);
        n /= 100;
    }
    let idx = n << 1;
    if n >= 10 {
        buf.write_two_bytes(&mut sep_pos, idx);
    } else {
        buf.write_one_byte(&mut sep_pos, idx + 1);
    }
    buf.end - buf.pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned() {
        let mut buf = Buffer::new();
        buf.write(&23_247);
        assert_eq!(buf.as_str(), "23,247");
    }

    #[test]
    fn signed() {
        let mut buf = Buffer::new();
        buf.write(&-23_247);
        assert_eq!(buf.as_str(), "-23,247");
    }
}
