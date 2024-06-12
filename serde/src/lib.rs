use std::{error::Error, fmt, io, io::Write, ops};

pub use nimiq_serde_derive::{SerializedMaxSize, SerializedSize};
pub use postcard::{fixint, FixedSizeByteArray};
use serde::{
    de::{Deserializer, Error as _},
    ser::Serializer,
};
pub use serde_derive::{Deserialize, Serialize};

/// Deserialization Error. This error is a wrapper over `postcard::Error`.
#[derive(Eq, PartialEq)]
pub struct DeserializeError(postcard::Error);

impl DeserializeError {
    /// Returns a 'Bad enumeration' error
    pub fn bad_enum() -> DeserializeError {
        DeserializeError(postcard::Error::DeserializeBadEnum)
    }
    /// Returns an 'Unexpected end' error.
    pub fn unexpected_end() -> DeserializeError {
        DeserializeError(postcard::Error::DeserializeUnexpectedEnd)
    }
    /// Returns a 'Bad encoding' error.
    pub fn bad_encoding() -> DeserializeError {
        DeserializeError(postcard::Error::DeserializeBadEncoding)
    }
    /// Returns a 'Serde custom' error.
    pub fn serde_custom() -> DeserializeError {
        DeserializeError(postcard::Error::SerdeDeCustom)
    }
}

impl fmt::Debug for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for DeserializeError {}

/// Types implementing this trait have a constant binary serialization size.
///
/// It can be `#[derive]`d using the [`SerializedSize`] derive macro.
pub trait SerializedSize {
    /// Size in bytes of the serialization.
    const SIZE: usize;
}
/// Types implementing this trait have a maximum binary serialization size.
///
/// It can be `#[derive]`d using the [`SerializedMaxSize`] derive macro.
pub trait SerializedMaxSize {
    /// Maximum size in bytes of the serialization.
    const MAX_SIZE: usize;
}

impl<T: SerializedSize> SerializedMaxSize for T {
    const MAX_SIZE: usize = T::SIZE;
}

#[rustfmt::skip]
mod integer_impls {
    use super::SerializedSize;
    use super::SerializedMaxSize;

    impl SerializedSize for bool { const SIZE: usize = 1; }

    impl SerializedSize for i8 { const SIZE: usize = 1; }
    impl SerializedSize for u8 { const SIZE: usize = 1; }

    impl SerializedMaxSize for i16 { const MAX_SIZE: usize = (16 + 6) / 7; }
    impl SerializedMaxSize for u16 { const MAX_SIZE: usize = (16 + 6) / 7; }
    impl SerializedMaxSize for i32 { const MAX_SIZE: usize = (32 + 6) / 7; }
    impl SerializedMaxSize for u32 { const MAX_SIZE: usize = (32 + 6) / 7; }
    impl SerializedMaxSize for i64 { const MAX_SIZE: usize = (64 + 6) / 7; }
    impl SerializedMaxSize for u64 { const MAX_SIZE: usize = (64 + 6) / 7; }
}

impl<T: SerializedMaxSize> SerializedMaxSize for Option<T> {
    const MAX_SIZE: usize = option_max_size(T::MAX_SIZE);
}

impl<T: SerializedMaxSize, E: SerializedMaxSize> SerializedMaxSize for Result<T, E> {
    const MAX_SIZE: usize = 1 + max(T::MAX_SIZE, E::MAX_SIZE);
}

impl<T: SerializedMaxSize> SerializedMaxSize for ops::Range<T> {
    const MAX_SIZE: usize = 2 * T::MAX_SIZE;
}

impl<const NUM: usize, T: SerializedMaxSize> SerializedMaxSize for [T; NUM] {
    const MAX_SIZE: usize = NUM * T::MAX_SIZE;
}

pub trait SerializeSeqMaxSize {
    type Element;
}

impl<T> SerializeSeqMaxSize for Vec<T> {
    type Element = T;
}

/// Maximum size in bytes for a integer value in binary serialization, given its maximum value.
pub const fn uint_max_size(max_value: u64) -> usize {
    let bits = match max_value.checked_ilog2() {
        Some(n) => n + 1,
        None => 1,
    };
    ((bits + 6) / 7) as usize
}

/// Maximum size in bytes for an `Option<T>` value in binary serialization.
///
/// `inner_size` is the maximum size of its inner value `T`.
pub const fn option_max_size(inner_size: usize) -> usize {
    1 + inner_size
}

/// Maximum size in bytes for a `Vec<T>` value in binary serialization.
///
/// `inner_size` is the maximum size of its inner value `T`. `max_elems` is the maximum number of
/// elements in that `Vec<T>`.
pub const fn seq_max_size(inner_size: usize, max_elems: usize) -> usize {
    uint_max_size(max_elems as u64) + inner_size * max_elems
}

pub const fn max(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}

/// The Nimiq human readable array serialization helper trait
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use nimiq_serde::HexArray;
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     #[serde(with = "HexArray")]
///     arr: [u8; 64],
/// }
/// ```
pub trait HexArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<'de, const N: usize> HexArray<'de> for [u8; N] {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(self))
        } else {
            serde::Serialize::serialize(&FixedSizeByteArray::from(*self), serializer)
        }
    }

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s: String = serde::Deserialize::deserialize(deserializer)?;
            let mut out = [0u8; N];
            hex::decode_to_slice(s, &mut out)
                .map_err(|_| D::Error::custom("Couldn't decode hex string"))?;
            Ok(out)
        } else {
            Ok(
                <FixedSizeByteArray<N> as serde::Deserialize>::deserialize(deserializer)?
                    .into_inner(),
            )
        }
    }
}

pub trait Serialize: serde::Serialize {
    fn serialize_to_writer<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        struct Wrapper<'a, 'b, W: Write> {
            inner: &'a mut W,
            written: &'b mut usize,
            error: &'b mut Option<io::Error>,
        }
        impl<'a, 'b, W: Write> postcard::ser_flavors::Flavor for Wrapper<'a, 'b, W> {
            type Output = ();
            fn try_push(&mut self, data: u8) -> postcard::Result<()> {
                self.try_extend(&[data])
            }
            fn try_extend(&mut self, data: &[u8]) -> postcard::Result<()> {
                assert!(self.error.is_none());
                *self.written += data.len();
                match self.inner.write_all(data) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        *self.error = Some(e);
                        Err(postcard::Error::SerializeBufferFull)
                    }
                }
            }
            fn finalize(self) -> postcard::Result<()> {
                Ok(())
            }
        }
        let mut written = 0;
        let mut error = None;
        let wrapper = Wrapper {
            inner: writer,
            written: &mut written,
            error: &mut error,
        };
        match postcard::serialize_with_flavor(self, wrapper) {
            Ok(()) => Ok(written),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        self.serialize_to_writer(writer)
    }
    fn serialize_to_vec(&self) -> Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }
    fn serialized_size(&self) -> usize {
        let size = postcard::ser_flavors::Size::default();
        postcard::serialize_with_flavor(self, size).unwrap()
    }
}

pub trait Deserialize: serde::de::DeserializeOwned {
    fn deserialize_from_vec(bytes: &[u8]) -> Result<Self, DeserializeError> {
        postcard::from_bytes(bytes).map_err(DeserializeError)
    }
    fn deserialize_take(bytes: &[u8]) -> Result<(Self, &[u8]), DeserializeError> {
        postcard::take_from_bytes(bytes).map_err(DeserializeError)
    }
}

impl<T: serde::Serialize> Serialize for T {}

impl<T: serde::de::DeserializeOwned> Deserialize for T {}
