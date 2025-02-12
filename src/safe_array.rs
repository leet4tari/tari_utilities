// Copyright 2022. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! An array-like type with safety features that make it suitable for cryptographic keys.

use alloc::vec::Vec;
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use subtle::ConstantTimeEq;
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

/// Sometimes it is not good that an array be used for a cryptographic key.
///
/// For example, creating `Hidden` data out of such an array may cause copies to arise if the data is dereferenced.
/// Further, you likely want constant-time equality testing.
///
/// A `SafeArray<T, const N: usize>` is a useful generic type that looks like an array where it counts for keys.
/// It supports reference access by implementing `AsRef<[T]>` and `AsMut<[T]>`, but does not implement `Copy`.
/// It also supports `Deref` and `DerefMut` with `[T]` targets.
/// Further, you get `Default` for handy instantiation, as well as `Clone`.
/// It automatically handles equality checking in constant time.
///
/// Under the hood, it's just `Vec<T>`, but don't tell anybody.
///
/// It's recommended that you use it as part of a `Hidden` type when you need a cryptographic key, like this:
///
/// ```edition2018
/// # #[macro_use] extern crate tari_utilities;
/// # use rand::rngs::OsRng;
/// # use rand::RngCore;
/// # use tari_utilities::{hidden_type, hidden::Hidden, safe_array::SafeArray};
/// # use zeroize::Zeroize;
/// # fn main() {
/// // Use the hidden type macro to build a new type for a 32-byte cryptographic key
/// hidden_type!(CipherKey, SafeArray<u8, 32>);
///
/// // Create a new default key
/// let mut key = CipherKey::from(SafeArray::default());
///
/// // You can access the data as an array reference
/// assert_eq!(key.reveal().as_ref(), &[0u8; 32]);
///
/// // Fill the key with random data, which requires `&mut [u8]`
/// let mut rng = OsRng;
/// rng.fill_bytes(key.reveal_mut());
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SafeArray<T, const N: usize>(Vec<T>);

impl<T, const N: usize> SafeArray<T, N> {
    /// The fixed number of elements
    pub const LEN: usize = N;
}

impl<T, const N: usize> AsRef<[T]> for SafeArray<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const N: usize> AsMut<[T]> for SafeArray<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T, const N: usize> Deref for SafeArray<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T, const N: usize> DerefMut for SafeArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

#[cfg(feature = "zeroize")]
impl<T, const N: usize> Zeroize for SafeArray<T, N>
where T: Zeroize
{
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<T, const N: usize> Default for SafeArray<T, N>
where T: Clone + Default
{
    fn default() -> Self {
        Self(vec![T::default(); N])
    }
}

impl<T, const N: usize> ConstantTimeEq for SafeArray<T, N>
where T: ConstantTimeEq
{
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.0.ct_eq(&other.0)
    }
}

impl<T, const N: usize> Eq for SafeArray<T, N> where T: ConstantTimeEq {}
impl<T, const N: usize> PartialEq for SafeArray<T, N>
where T: ConstantTimeEq
{
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        use rand::{rngs::OsRng, RngCore};
        use zeroize::Zeroize;

        use crate::{hidden::Hidden, hidden_type};

        hidden_type!(CipherKey, SafeArray<u8, 32>);
        let key_a = CipherKey::from(SafeArray::<u8, 32>::default());
        let key_b = CipherKey::from(SafeArray::<u8, 32>::default());

        // Test equality in constant time between the `SafeArray` types
        assert_eq!(key_a.reveal(), key_b.reveal());

        // Test equality (not in constant time) using an array reference
        assert_eq!(key_a.reveal().as_ref(), &[0u8; 32]);

        // Test mutable reference access
        let mut key_c = CipherKey::from(SafeArray::<u8, 32>::default());
        let mut rng = OsRng;
        rng.fill_bytes(key_c.reveal_mut());
        assert_ne!(key_c.reveal().as_ref(), &[0u8; 32]);
    }

    #[test]
    fn len() {
        const N: usize = 64;
        assert_eq!(SafeArray::<u8, N>::default().len(), N);
        assert_eq!(SafeArray::<u8, 64>::LEN, N);
    }
}
