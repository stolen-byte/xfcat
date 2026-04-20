// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(unused_imports)]
use std::fmt::{self, Arguments};
use std::panic::PanicHookInfo;
use std::sync::{Arc, OnceLock};

use color_print::*;

// =============================================================================
mod console;
pub use console::*;

// =============================================================================
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Level {
	Warn,
	Error,
	Panic,
}

impl fmt::Display for Level {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.pad(match self {
			Level::Warn => cstr!("<y>WARN</>"),
			Level::Error => cstr!("<r>ERROR</>"),
			Level::Panic => cstr!("<r>PANIC</>"),
		})
	}
}

// =============================================================================
pub trait Logger: Send + Sync + 'static {
	fn panic_handler(&self, info: &PanicHookInfo) {
		let payload = info
			.payload()
			.downcast_ref::<String>()
			.map(String::as_str)
			.or_else(|| info.payload().downcast_ref::<&str>().copied())
			.unwrap_or("<non-string panic payload");

		if let Some(loc) = info.location() {
			self.write(
				Level::Panic,
				format_args!(cstr!("{}\n  <dim>at</> <c>{}</>:<c>{}</>"), payload, loc.file(), loc.line()),
			);
		} else {
			self.write(Level::Panic, format_args!("{payload}"));
		}
	}

	fn write(&self, level: Level, args: Arguments);
}

// =============================================================================
macro_rules! __warn {
	($($arg:tt)*) => {
		$crate::log::write($crate::log::Level::Warn, format_args!($($arg)*))
	};
}

macro_rules! __error {
	($($arg:tt)*) => {
		$crate::log::write($crate::log::Level::Error, format_args!($($arg)*))
	};
}

macro_rules! __format {
	($lvl:expr, $($arg:tt)*) => {
		$crate::log::format_message_args($lvl, format_args!($($arg)*))
	}
}

macro_rules! __format_error {
	($($arg:tt)*) => {
		$crate::log::format_message_args($crate::log::Level::Error, format_args!($($arg)*))
	}
}

macro_rules! __format_warn {
	($($arg:tt)*) => {
		$crate::log::format_message_args($crate::log::Level::Warn, format_args!($($arg)*))
	}
}

pub(crate) use __error as error;
pub(crate) use __format as format;
pub(crate) use __format_error as format_error;
pub(crate) use __format_warn as format_warn;
pub(crate) use __warn as warn;

// =============================================================================
static LOGGER: OnceLock<Arc<dyn Logger>> = OnceLock::new();

pub fn current() -> Arc<dyn Logger> {
	LOGGER.get_or_init(|| Arc::new(ConsoleLogger)).clone()
}

pub fn setup<T: Logger>(logger: T) {
	let new = Arc::new(logger);

	if let Ok(()) = LOGGER.set(new.clone()) {
		std::panic::set_hook(Box::new(move |info| {
			new.panic_handler(info);
		}));
	} else {
		// already setup, ignore
	}
}

pub fn write(level: Level, args: Arguments) {
	current().write(level, args);
}

#[allow(dead_code)]
pub fn format_message_args(level: Level, args: Arguments) -> String {
	use std::fmt::Write;
	let mut s = String::new();
	_ = write!(s, "{level}: ");
	_ = s.write_fmt(args);
	s
}
