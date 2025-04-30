// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![no_std]
#![forbid(
    unsafe_code,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::cargo
)]
#![deny(clippy::all)] // some #[allow]s in generated code of serde

use core::{
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Erased<T: ?Sized>(pub T);

impl<T: ?Sized> fmt::Debug for Erased<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...")
    }
}

impl<T> From<T> for Erased<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: ?Sized> Deref for Erased<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for Erased<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: FromStr> FromStr for Erased<T> {
    type Err = T::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Erased<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // this is our equivalent of unsafe
            self.0.fmt(f)
        } else {
            write!(f, "...")
        }
    }
}

impl<T: ?Sized + AsRef<Q>, Q: ?Sized> AsRef<Q> for Erased<T> {
    #[inline]
    fn as_ref(&self) -> &Q {
        self.0.as_ref()
    }
}
