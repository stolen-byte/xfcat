// SPDX-License-Identifier: GPL-3.0-or-later
use std::{ptr, slice};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::*;

// =============================================================================
#[inline]
pub fn decode_digest(input: &[u8; 32], output: &mut [u8; 16]) -> bool {
	unsafe {
		match decode_avx2(input) {
			Some(result) => {
				let bytes = slice::from_raw_parts(ptr::from_ref(&result).cast::<u8>(), size_of_val(&result));
				output
					.as_mut_ptr()
					.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());

				true
			}
			None => false,
		}
	}
}

#[inline]
pub fn encode_digest(input: &[u8; 16], output: &mut [u8; 32]) {
	unsafe {
		let result = encode_avx2(input);
		let bytes = slice::from_raw_parts(ptr::from_ref(&result).cast::<u8>(), size_of_val(&result));
		output
			.as_mut_ptr()
			.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
	}
}

// based on http://0x80.pl/notesen/2022-01-17-validating-hex-parse.html#algorithm-3-by-geoff-langdale
#[inline]
#[target_feature(enable = "avx2")]
fn decode_avx2(input: &[u8; 32]) -> Option<__m128i> {
	unsafe {
		let v = input.as_ptr().cast::<__m256i>().read_unaligned();

		let t1 = _mm256_add_epi8(v, _mm256_set1_epi8(0xc6u8.cast_signed())); // 0xff - '9'
		let t2 = _mm256_subs_epu8(t1, _mm256_set1_epi8(6));
		let t3 = _mm256_sub_epi8(t2, _mm256_set1_epi8(0xf0u8.cast_signed()));
		let t4 = _mm256_and_si256(v, _mm256_set1_epi8(0xdfu8.cast_signed()));
		let t5 = _mm256_sub_epi8(t4, _mm256_set1_epi8(b'A'.cast_signed()));
		let t6 = _mm256_adds_epu8(t5, _mm256_set1_epi8(10));
		let t7 = _mm256_min_epu8(t3, t6);

		if _mm256_movemask_epi8(_mm256_adds_epu8(t7, _mm256_set1_epi8(112))) != 0 {
			return None;
		}

		let merged = _mm256_maddubs_epi16(t7, _mm256_set1_epi16(0x0110));
		let packed = _mm256_packus_epi16(merged, _mm256_setzero_si256());
		let result = _mm256_permute4x64_epi64(packed, 0b11_01_10_00);
		Some(_mm256_castsi256_si128(result))
	}
}

// http://0x80.pl/notesen/2008-04-29-sse-hexprint.html
#[inline]
#[target_feature(enable = "avx2")]
fn encode_avx2(input: &[u8; 16]) -> __m256i {
	unsafe {
		let v = input.as_ptr().cast::<__m128i>().read_unaligned();
		let lookup = _mm_lddqu_si128(HEX_CHARS.as_ptr().cast());
		let lmask = _mm_set1_epi8(0x0f);

		let hi = _mm_and_si128(_mm_srli_epi16(v, 4), lmask);
		let lo = _mm_and_si128(v, lmask);

		let out1 = _mm_unpacklo_epi8(hi, lo);
		let out2 = _mm_unpackhi_epi8(hi, lo);

		_mm256_set_m128i(_mm_shuffle_epi8(lookup, out2), _mm_shuffle_epi8(lookup, out1))
	}
}
