macro_rules! impl_ux {
    ($($type:ty),+) => {
        $(
            impl $crate::Integer for $type {
                #[inline(always)]
                #[allow(trivial_numeric_casts)]
                fn to_buffer(&self, buf: &mut Buffer) -> usize {
                    let n = *self as u128;
                    do_the_thing(n, buf)
                }
            }
        )+
    };
}
pub(crate) use impl_ux;

macro_rules! impl_ix {
    ($($type:ty),+) => {
        $(
            impl $crate::Integer for $type {
                #[inline(always)]
                fn to_buffer(&self, buf: &mut Buffer) -> usize {
                    if self.is_negative() {
                        let n = (!(*self as u128)).wrapping_add(1); // make positive
                        let c = do_the_thing(n, buf);
                        let minus = b'-';
                        buf.pos -= 1;
                        buf.inner[buf.pos] = minus;
                        c + 1
                    } else {
                        let n = *self as u128;
                        do_the_thing(n, buf)
                    }
                }
            }
        )+
    };
}
pub(crate) use impl_ix;

macro_rules! impl_nonzero_ux {
    ($($type:ty, $relative:ty),+) => {
        $(
            impl $crate::Integer for $type {
                #[inline(always)]
                #[allow(trivial_numeric_casts)]
                fn to_buffer(&self, buf: &mut Buffer) -> usize {
                    let n = self.get() as u128;
                    do_the_thing(n, buf)
                }
            }
        )+
    };
}
pub(crate) use impl_nonzero_ux;

macro_rules! impl_sealed {
    ($($type:ty),+) => {
        $(
            impl $crate::sealed::Sealed for $type {}
        )+
    };
}
pub(crate) use impl_sealed;
