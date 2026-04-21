// SPDX-License-Identifier: GPL-3.0-or-later
use std::fmt::{self, Alignment, Display, Formatter, Result, Write};
use std::result::Result as StdResult;

// =============================================================================
const SIZE_SUFFIXES: [&str; 4] = ["B", "K", "M", "G"];
const UNITSIZE: f64 = 1024.0;

// =============================================================================
struct DisplayComponents<'a> {
	value: &'a str,
	suffix: &'a str,
}

impl<'a> DisplayComponents<'a> {
	pub fn new(value: &'a str, suffix: &'a str) -> Self {
		Self { value, suffix }
	}

	pub fn write(self, f: &mut Formatter<'_>) -> Result {
		f.write_str(self.value)?;
		f.write_str(self.suffix)
	}

	pub fn len(&self) -> usize {
		self.value.chars().count() + self.suffix.chars().count()
	}
}

// =============================================================================
struct Padding {
	fill: char,
	size: u16,
}

impl Padding {
	pub fn new(size: u16, fill: char) -> Self {
		Self { fill, size }
	}

	pub fn write(self, f: &mut Formatter<'_>) -> Result {
		for _ in 0..self.size {
			f.write_char(self.fill)?;
		}
		Ok(())
	}

	pub fn write_pre(self, align: Alignment, f: &mut Formatter<'_>) -> StdResult<Padding, fmt::Error> {
		let pad = match align {
			Alignment::Left => 0,
			Alignment::Right => self.size,
			Alignment::Center => self.size / 2,
		};

		for _ in 0..pad {
			f.write_char(self.fill)?;
		}

		Ok(Padding::new(self.size - pad, self.fill))
	}
}

// =============================================================================
// kinda gross, but intended usage is optional, and using a newtype creates problems.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SizeFormat {
	Human(u64),
	Raw(u64),
}

impl Display for SizeFormat {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			SizeFormat::Human(s) => {
				let mut buf = ryu::Buffer::new();
				let comps = make_components(&mut buf, *s);

				if let Some(width) = f.width() {
					#[allow(clippy::cast_possible_truncation, reason = "width is stored as u16")]
					let lpad = Padding::new((width - comps.len()) as u16, f.fill());
					let rpad = lpad.write_pre(f.align().unwrap_or(Alignment::Left), f)?;
					comps.write(f)?;
					rpad.write(f)
				} else {
					comps.write(f)
				}
			}
			SizeFormat::Raw(s) => s.fmt(f),
		}
	}
}

// =============================================================================
#[allow(
	clippy::cast_sign_loss,
	clippy::cast_possible_truncation,
	clippy::cast_precision_loss
)]
fn make_components(buf: &mut ryu::Buffer, value: u64) -> DisplayComponents<'_> {
	let fsize = value as f64;
	if fsize <= 0.0 {
		return DisplayComponents::new("0", SIZE_SUFFIXES[0]);
	}

	let base = fsize.log10() / UNITSIZE.log10();
	let result = buf
		.format((UNITSIZE.powf(base - base.floor()) * 10.0).round() / 10.0)
		.trim_end_matches(".0");

	DisplayComponents::new(result, SIZE_SUFFIXES[base.floor() as usize])
}
