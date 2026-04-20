// SPDX-License-Identifier: GPL-3.0-or-later

// =============================================================================
mod timestamp;
pub use timestamp::*;

mod size;
pub use size::*;

#[cfg(test)]
mod tests;

// =============================================================================
// small helper for adding paths to an error context
pub trait PathContext {
	fn as_context(&self) -> String;
}

impl<P: AsRef<std::path::Path>> PathContext for P {
	fn as_context(&self) -> String {
		self.as_ref().display().to_string()
	}
}
