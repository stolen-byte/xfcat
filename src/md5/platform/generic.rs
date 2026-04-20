// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;

// =============================================================================
#[inline]
pub fn decode_digest(input: &[u8; 32], output: &mut [u8; 16]) -> bool {
	let max = output.len();

	unsafe {
		for i in 0..max {
			let high = HEX_DECODE_LUT[*input.get_unchecked(i * 2) as usize];
			let low = HEX_DECODE_LUT[*input.get_unchecked(i * 2 + 1) as usize];

			if (low | high) == u8::MAX {
				return false;
			}

			*output.get_unchecked_mut(i) = (high << 4) | low;
		}
	}

	true
}

#[inline]
pub fn encode_digest(input: &[u8; 16], output: &mut [u8; 32]) {
	let mut i = 0;
	unsafe {
		for &byte in input {
			*output.get_unchecked_mut(i) = *HEX_CHARS.get_unchecked((byte >> 4) as usize);
			*output.get_unchecked_mut(i + 1) = *HEX_CHARS.get_unchecked((byte & 0x0f) as usize);
			i += 2;
		}
	}
}
