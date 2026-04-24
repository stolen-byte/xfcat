// SPDX-License-Identifier: GPL-3.0-or-later
mod formatting;
mod size;
mod timestamp;

use std::hash::Hash;

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

#[cfg(test)]
mod tests;

// =============================================================================
const PB_TEMPLATE: &str = " {wide_msg} {binary_bytes:>11} {binary_bytes_per_sec:>13.green} {elapsed} [{bar:.cyan/white}] {percent:>3}%";
const PB_CHARS: &str = "#>-";

pub use {formatting::*, size::*, timestamp::*};

// =============================================================================
pub fn init_threadpool(threads: Option<usize>) {
	use rayon::ThreadPoolBuilder;
	use std::thread;

	ThreadPoolBuilder::new()
		.num_threads(threads.unwrap_or_else(|| thread::available_parallelism().unwrap().get()))
		.build_global()
		.unwrap();
}

pub fn make_hash<T: Hash>(value: &T) -> u64 {
	use std::{collections::hash_map::DefaultHasher, hash::Hasher};

	let mut hasher = DefaultHasher::new();
	value.hash(&mut hasher);
	hasher.finish()
}

pub fn create_pb_style() -> ProgressStyle {
	use indicatif::ProgressState;
	use std::fmt::Write;

	ProgressStyle::with_template(PB_TEMPLATE)
		.unwrap()
		.progress_chars(PB_CHARS)
		.with_key("elapsed", |state: &ProgressState, out: &mut dyn Write| {
			_ = out.write_fmt(format_args!("{}", FormattedDuration(state.elapsed())));
		})
}

pub fn create_progress_bar(max: Option<u64>, target: ProgressDrawTarget) -> ProgressBar {
	ProgressBar::with_draw_target(max, target).with_style(create_pb_style())
}

pub fn add_progress(max: u64, target: &MultiProgress) -> ProgressBar {
	target.add(ProgressBar::new(max).with_style(create_pb_style()))
}
