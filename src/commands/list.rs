// SPDX-License-Identifier: GPL-3.0-or-later
use crate::log;
use xf::cat;
use xf::utils::PathContext;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anstream::stdout;
use anyhow::{Context, Result};
use color_print::*;

// =============================================================================
#[derive(clap::Args)]
#[command(after_help = "note: file times are displayed in UTC.")]
pub struct Command {
	/// input files
	#[arg(required = true)]
	inputs: Vec<PathBuf>,
}

impl Command {
	pub fn execute(&self) -> Result<()> {
		for path in &self.inputs {
			let path = path.with_extension("cat");

			if let Err(e) = list(path) {
				if crate::is_sigpipe(&e) {
					return Err(e);
				}
				log::error!("{e:#}");
			}
		}

		Ok(())
	}
}

// =============================================================================
fn list<P: AsRef<Path>>(source: P) -> Result<()> {
	let mut out = stdout().lock();
	let source = source.as_ref();
	let mut reader = cat::Reader::new(File::open(source).with_context(|| source.as_context())?);
	let mut entry = cat::Entry::with_capacity(256);

	writeln!(out, cstr!("<m><u>{}</>:"), source.display())?;

	let mut count = 0;
	while reader.read_entry(&mut entry)? {
		writeln!(out, cstr!("  <b>{:>7}</> {:#} {}"), entry.size, entry.timestamp, entry.path)?;
		count += 1;
	}

	writeln!(out, cstr!("total: <g>{}</> entries.\n"), count)?;
	Ok(())
}
