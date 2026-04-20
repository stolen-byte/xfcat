// SPDX-License-Identifier: GPL-3.0-or-later

// =============================================================================
mod common;
pub mod list;

// =============================================================================
macro_rules! cli {
	(
		options = { $( $opt:ident$(: $val:expr )? ),+ $(,)? },
		commands = { $( $cmd:ident: $help:literal ),+ $(,)? }$(,)?
	) => {
		use clap::{Parser, Subcommand};

		paste::paste! {
			#[derive(Subcommand)]
			enum Commands {
				$(
					#[doc = $help]
					[<$cmd:camel>]($cmd::Command),
				)*
			}

			#[derive(Parser)]
			#[command(
				$($opt$( = $val )?,)*
			)]
			struct Cli {
				#[command(subcommand)]
				command: Commands,
			}

			impl Cli {
				pub fn run(&self) -> anyhow::Result<()> {
					match &self.command {
						$(Commands::[<$cmd:camel>](c) => c.execute(),)*
					}
				}
			}
		}
	};
	// match arm exists solely to prevent errors when 0 commands are listed
	// (eg: when a project is first started)
	(
		options = { $( $opt:ident$(: $val:expr )? ),* $(,)? },
		commands = { }
		$(,)?
	) => {
			use clap::Parser;

			#[derive(Parser)]
			#[command(
				$($opt$( = $val )?,)*
			)]
			struct Cli {}

			impl Cli {
				pub fn run(&self) -> anyhow::Result<()> {
					anyhow::bail!("not implemented");
				}
			}
	}
}
