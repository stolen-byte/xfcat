// SPDX-License-Identifier: GPL-3.0-or-later
use cfg_if::cfg_if;

// =============================================================================
pub const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
pub const HEX_DECODE_LUT: &[u8; 256] = &{
	let mut lut = [0; 256];
	let mut i = 0u8;
	loop {
		lut[i as usize] = match i {
			b'0'..=b'9' => i - b'0',
			b'A'..=b'F' => i - b'A' + 10,
			b'a'..=b'f' => i - b'a' + 10,
			_ => u8::MAX, // invalid
		};
		if i == u8::MAX {
			break;
		}
		i += 1;
	}
	lut
};

// =============================================================================
cfg_if! {
	if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
		pub mod x86;
		pub use x86 as imp;
	} else {
		pub mod generic;
		pub use generic as imp;
	}
}
