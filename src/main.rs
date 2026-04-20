// SPDX-License-Identifier: GPL-3.0-or-later
// use commands::*;

use std::process::ExitCode;

// =============================================================================
#[macro_use]
mod commands;
mod log;

// =============================================================================
cli! {
	options = {
		version,
		about,
		long_about: None,
		// after_help: "",
	},
	commands = {
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
