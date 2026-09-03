// SPDX-License-Identifier: MIT OR Apache-2.0

use std::borrow::Cow;
use std::error::Error;
use std::fmt;

/// An error returned when captured string bytes violate their semantic invariant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CapturedStringError {
    /// The semantic bytes contain a NUL byte.
    InteriorNul,
}

impl fmt::Display for CapturedStringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul => {
                formatter.write_str("captured string contains an interior NUL byte")
            }
        }
    }
}

impl Error for CapturedStringError {}

/// Bytes captured from a fixed-size kernel string buffer.
///
/// The semantic bytes never contain NUL. A truncated value had no NUL byte in
/// the complete fixed-size source buffer; the flag describes that source state
/// independently of the semantic bytes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct CapturedString {
    bytes: Vec<u8>,
    truncated: bool,
}

impl CapturedString {
    /// Creates a captured string from semantic bytes and the source truncation state.
    ///
    /// Both truncation states are valid when `bytes` contains no NUL byte.
    pub fn new(bytes: Vec<u8>, truncated: bool) -> Result<Self, CapturedStringError> {
        if bytes.contains(&0) {
            Err(CapturedStringError::InteriorNul)
        } else {
            Ok(Self { bytes, truncated })
        }
    }

    /// Returns the semantic bytes captured before the source buffer's first NUL.
    ///
    /// For a truncated value, this is the complete fixed-size source buffer.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns whether the fixed-size source buffer contained no NUL byte.
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Returns a lossy UTF-8 view without escaping control bytes.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.bytes)
    }

    pub(crate) fn from_fixed(bytes: &[u8]) -> Self {
        match bytes.iter().position(|byte| *byte == 0) {
            Some(end) => Self {
                bytes: bytes[..end].to_vec(),
                truncated: false,
            },
            None => Self {
                bytes: bytes.to_vec(),
                truncated: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CapturedString, CapturedStringError};

    #[test]
    fn public_constructor_accepts_both_truncation_states() {
        let complete = CapturedString::new(vec![b'a', 0xff, b'\n'], false).unwrap();
        let truncated = CapturedString::new(vec![b'a', 0xff, b'\n'], true).unwrap();

        assert_eq!(complete.as_bytes(), &[b'a', 0xff, b'\n']);
        assert!(!complete.is_truncated());
        assert_eq!(truncated.as_bytes(), &[b'a', 0xff, b'\n']);
        assert!(truncated.is_truncated());
        assert_eq!(truncated.to_string_lossy(), "a�\n");
    }

    #[test]
    fn public_constructor_rejects_interior_nul() {
        assert_eq!(
            CapturedString::new(b"before\0after".to_vec(), false),
            Err(CapturedStringError::InteriorNul)
        );
        assert_eq!(
            CapturedString::new(vec![0], true),
            Err(CapturedStringError::InteriorNul)
        );
    }

    #[test]
    fn fixed_decoder_constructs_both_source_states() {
        let complete = CapturedString::from_fixed(b"text\0ignored");
        let truncated = CapturedString::from_fixed(b"full");

        assert_eq!(complete.as_bytes(), b"text");
        assert!(!complete.is_truncated());
        assert_eq!(truncated.as_bytes(), b"full");
        assert!(truncated.is_truncated());
    }
}
