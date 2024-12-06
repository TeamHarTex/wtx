use crate::misc::{char_slice, from_utf8_basic, BasicUtf8Error, Lease, Usize};
use core::{
  borrow::Borrow,
  cmp::Ordering,
  fmt::{self, Arguments, Debug, Display, Formatter, Write},
  hash::{Hash, Hasher},
  ops::Deref,
  str,
};

/// Errors of [`ArrayString`].
#[derive(Debug)]
pub enum ArrayStringError {
  #[doc = doc_out_of_bounds_params!()]
  ReplaceHasOutOfBoundsParams,
  #[doc = doc_bad_format!()]
  BadFormat,
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
  #[doc = doc_many_elems_cap_overflow!()]
  PushStrOverflow,
}

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
#[derive(Clone, Copy)]
pub struct ArrayString<const N: usize> {
  len: u32,
  data: [u8; N],
}

impl<const N: usize> ArrayString<N> {
  /// Constructs a new, empty instance.
  #[inline]
  pub fn from_parts(data: [u8; N], len: u32) -> Result<Self, BasicUtf8Error> {
    Self::instance_check();
    let n = Self::instance_u32();
    let _ = from_utf8_basic(&data)?;
    Ok(Self { len: if len > n { n } else { len }, data })
  }

  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self::instance_check();
    Self { len: 0, data: [0; N] }
  }

  /// The filled elements that composed a string.
  #[inline]
  pub fn as_str(&self) -> &str {
    self
  }

  /// The number of elements that can be stored.
  #[inline]
  pub fn capacity(&self) -> u32 {
    Self::instance_u32()
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.len = 0;
  }

  /// Number of bytes
  #[inline]
  pub fn len(&self) -> u32 {
    self.len
  }

  /// Appends an element to the back of the collection.
  #[inline]
  pub fn push(&mut self, ch: char) -> Result<(), ArrayStringError> {
    self.push_bytes(ArrayStringError::PushOverflow, char_slice(&mut [0; 4], ch))
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  ///
  /// # Panics
  ///
  /// If there is no available capacity.
  #[inline]
  pub fn push_str(&mut self, str: &str) -> Result<(), ArrayStringError> {
    self.push_bytes(ArrayStringError::PushStrOverflow, str.as_bytes())
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub fn remaining_capacity(&self) -> u32 {
    self.capacity().wrapping_sub(self.len)
  }

  /// How many elements can be added to this collection.
  #[inline]
  pub fn replace(&mut self, start: usize, str: &str) -> Result<(), ArrayStringError> {
    let Some(slice) = start.checked_add(str.len()).and_then(|end| self.data.get_mut(start..end))
    else {
      return Err(ArrayStringError::ReplaceHasOutOfBoundsParams);
    };
    slice.copy_from_slice(str.as_bytes());
    Ok(())
  }
  /// Shortens the vector, keeping the first `len` elements.
  #[inline]
  pub fn truncate(&mut self, len: u32) {
    self.len = len.min(self.capacity());
  }

  #[inline]
  const fn instance_check() {
    const {
      assert!(N <= Usize::from_u32(u32::MAX).into_usize());
    }
  }

  #[inline]
  const fn instance_u32() -> u32 {
    const {
      let [_, _, _, _, a, b, c, d] = Usize::from_usize(N).into_u64().to_be_bytes();
      u32::from_be_bytes([a, b, c, d])
    }
  }

  #[inline]
  fn push_bytes(&mut self, error: ArrayStringError, other: &[u8]) -> Result<(), ArrayStringError> {
    let Some(len) = u32::try_from(other.len()).ok().filter(|el| self.remaining_capacity() >= *el)
    else {
      return Err(error);
    };
    let begin = *Usize::from(self.len);
    let end = *Usize::from(self.len.wrapping_add(len));
    self.data.get_mut(begin..end).unwrap_or_default().copy_from_slice(other);
    self.len = self.len.wrapping_add(len);
    Ok(())
  }
}

impl<const N: usize> Borrow<str> for ArrayString<N> {
  #[inline]
  fn borrow(&self) -> &str {
    self
  }
}

impl<const N: usize> Debug for ArrayString<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl<const N: usize> Default for ArrayString<N> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<const N: usize> Display for ArrayString<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl<const N: usize> Deref for ArrayString<N> {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: `len` is always less than `N`
    let slice = unsafe { self.data.get(..*Usize::from(self.len)).unwrap_unchecked() };
    // SAFETY: data is always valid UTF-8
    unsafe { str::from_utf8_unchecked(slice) }
  }
}

impl<const N: usize> Lease<str> for ArrayString<N> {
  #[inline]
  fn lease(&self) -> &str {
    self
  }
}

impl<const N: usize> Eq for ArrayString<N> {}

impl<const N: usize> Hash for ArrayString<N> {
  #[inline]
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    Hash::hash(&**self, state);
  }
}

impl<const N: usize> PartialEq for ArrayString<N> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

impl<const N: usize> PartialEq<[u8]> for ArrayString<N> {
  #[inline]
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}

impl<const N: usize> PartialEq<str> for ArrayString<N> {
  #[inline]
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl<const N: usize> PartialOrd for ArrayString<N> {
  #[inline]
  fn ge(&self, other: &Self) -> bool {
    (**self).ge(&**other)
  }

  #[inline]
  fn gt(&self, other: &Self) -> bool {
    (**self).gt(&**other)
  }

  #[inline]
  fn le(&self, other: &Self) -> bool {
    (**self).le(&**other)
  }

  #[inline]
  fn lt(&self, other: &Self) -> bool {
    (**self).lt(&**other)
  }

  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<const N: usize> Ord for ArrayString<N> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    (**self).cmp(other)
  }
}

impl<'args, const N: usize> TryFrom<Arguments<'args>> for ArrayString<N> {
  type Error = ArrayStringError;

  #[inline]
  fn try_from(from: Arguments<'args>) -> Result<Self, Self::Error> {
    let mut v = Self::new();
    v.write_fmt(from).map_err(|_err| ArrayStringError::BadFormat)?;
    Ok(v)
  }
}

impl<const N: usize> TryFrom<&str> for ArrayString<N> {
  type Error = ArrayStringError;

  #[inline]
  fn try_from(from: &str) -> Result<Self, Self::Error> {
    let mut this = Self::default();
    this.push_str(from)?;
    Ok(this)
  }
}

impl<const N: usize> Write for ArrayString<N> {
  #[inline]
  fn write_char(&mut self, c: char) -> fmt::Result {
    self.push(c).map_err(|_err| fmt::Error)
  }

  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.push_str(s).map_err(|_err| fmt::Error)
  }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
  use crate::misc::{ArrayString, Usize};
  use arbitrary::{Arbitrary, Unstructured};

  impl<'any, const N: usize> Arbitrary<'any> for ArrayString<N> {
    #[inline]
    fn arbitrary(u: &mut Unstructured<'any>) -> arbitrary::Result<Self> {
      let mut len = Self::instance_u32();
      len = u32::arbitrary(u)?.min(len);
      let mut data = [0; N];
      for elem in data.iter_mut().take(*Usize::from(len)) {
        loop {
          let byte = u8::arbitrary(u)?;
          if byte.is_ascii_alphanumeric() {
            *elem = byte;
            break;
          }
        }
      }
      Ok(Self { len, data })
    }
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::{from_utf8_basic, ArrayString};
  use core::{fmt::Formatter, marker::PhantomData};
  use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
  };

  impl<'de, const N: usize> Deserialize<'de> for ArrayString<N> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct ArrayStringVisitor<const N: usize>(PhantomData<[u8; N]>);

      impl<const N: usize> Visitor<'_> for ArrayStringVisitor<N> {
        type Value = ArrayString<N>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
          write!(formatter, "a string no more than {} bytes long", N)
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          let rslt = from_utf8_basic(v);
          let str = rslt.map_err(|_| E::invalid_value(de::Unexpected::Bytes(v), &self))?;
          ArrayString::try_from(str).map_err(|_| E::invalid_length(str.len(), &self))
        }

        #[inline]
        fn visit_str<E>(self, str: &str) -> Result<Self::Value, E>
        where
          E: de::Error,
        {
          ArrayString::try_from(str).map_err(|_| E::invalid_length(str.len(), &self))
        }
      }

      deserializer.deserialize_str(ArrayStringVisitor(PhantomData))
    }
  }

  impl<const N: usize> Serialize for ArrayString<N> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      serializer.serialize_str(self)
    }
  }
}
