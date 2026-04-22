// SPDX-License-Identifier: GPL-3.0-or-later
mod size;
mod timestamp;

#[cfg(test)]
mod tests;

// =============================================================================
pub use {size::*, timestamp::*};

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

// =============================================================================
pub fn init_threadpool(threads: Option<usize>) {
	use rayon::ThreadPoolBuilder;
	use std::thread;

	ThreadPoolBuilder::new()
		.num_threads(threads.unwrap_or_else(|| thread::available_parallelism().unwrap().get()))
		.build_global()
		.unwrap();
}
