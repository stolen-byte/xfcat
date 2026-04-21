// SPDX-License-Identifier: GPL-3.0-or-later
#[macro_use]
mod commands;
mod log;

use std::process::ExitCode;

use commands::*;

// =============================================================================
cli! {
	options = {
		version,
		about,
		long_about: None,
		after_help: "note: path extensions can be either .cat/.dat, or be omitted altogether.",
	},
	commands = {
		list: "list contents specified packages.",
	}
}

// =============================================================================
fn is_sigpipe(e: &anyhow::Error) -> bool {
	for cause in e.chain() {
		if let Some(inner) = cause.downcast_ref::<std::io::Error>() {
			if inner.kind() == std::io::ErrorKind::BrokenPipe {
				return true;
			}
		}
	}
	false
}

fn main() -> ExitCode {
	log::setup(log::ConsoleLogger);

	if let Err(e) = Cli::parse().run() {
		// handle SIGPIPE errors specifically, as rust returns them as regular i/o errors
		if is_sigpipe(&e) {
			return ExitCode::SUCCESS;
		}
		log::error!("{e:#}");
		return ExitCode::FAILURE;
	}

	ExitCode::SUCCESS
}
