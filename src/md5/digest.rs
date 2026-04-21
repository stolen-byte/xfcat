// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fmt, ops, str};

use super::platform::*;

// =============================================================================
#[derive(Debug, PartialEq)]
pub enum DigestError {
	InvalidCharacter(char, usize),
	HashLength(usize),
}

impl std::error::Error for DigestError {}

impl fmt::Display for DigestError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match *self {
			Self::InvalidCharacter(c, i) => write!(f, "invalid hex character {c:?} at index {i} in hash"),
			Self::HashLength(len) => write!(f, "invalid hash length ({len})"),
		}
	}
}

pub type Result<T> = anyhow::Result<T, DigestError>;

// =============================================================================
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Digest(pub [u8; 16]);

impl Digest {
	pub fn decode_into(s: &str, out: &mut Self) -> Result<()> {
		let bytes = s
			.as_bytes()
			.try_into()
			.map_err(|_| DigestError::HashLength(s.len()))?;

		if !imp::decode_digest(bytes, out) {
			return Err(invalid_digest_error(bytes));
		}

		Ok(())
	}
}

impl From<Digest> for [u8; 16] {
	fn from(value: Digest) -> Self {
		value.0
	}
}

impl ops::Deref for Digest {
	type Target = [u8; 16];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ops::DerefMut for Digest {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl str::FromStr for Digest {
	type Err = DigestError;

	#[inline]
	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let mut r = Digest::default();
		Digest::decode_into(s, &mut r)?;
		Ok(r)
	}
}

impl fmt::Display for Digest {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut bytes = [0u8; 32];
		imp::encode_digest(self, &mut bytes);

		if f.alternate() {
			f.write_str("0x")?;
		}

		f.write_str(unsafe { str::from_utf8_unchecked(&bytes) })
	}
}

// =============================================================================
/// assumes input contains at least 1 invalid character
#[inline]
fn invalid_digest_error(input: &[u8; 32]) -> DigestError {
	for (i, byte) in input.iter().enumerate() {
		if HEX_DECODE_LUT[*byte as usize] == u8::MAX {
			return DigestError::InvalidCharacter(*byte as char, i);
		}
	}

	unsafe { std::hint::unreachable_unchecked() };
}
