// SPDX-License-Identifier: GPL-3.0-or-later
use super::common::PathArgs;
use crate::log;
use xf::cat;
use xf::utils::{PathContext, SizeFormat};

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

	/// display sizes as K/M/G etc.
	#[arg(short = 'H', long, default_value_t = false)]
	human_readable: bool,

	#[command(flatten)]
	path: PathArgs,
}

impl Command {
	pub fn execute(&self) -> Result<()> {
		for path in &self.inputs {
			// allow specifying path to either .cat, .dat, or omission.
			let source = match path.extension().and_then(|e| e.to_str()) {
				Some("dat") | None => path.with_extension("cat"),
				_ => path.clone(),
			};

			if let Err(e) = list(source, self.human_readable, &self.path) {
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
fn list<P: AsRef<Path>>(source: P, human_readable: bool, pargs: &PathArgs) -> Result<()> {
	let mut out = stdout().lock();
	let source = source.as_ref();
	let mut reader = cat::Reader::new(File::open(source).with_context(|| source.as_context())?);
	let mut entry = cat::Entry::with_capacity(256);

	writeln!(out, cstr!("<m><u>{}</>:"), source.display())?;

	let mut count = 0;
	while reader.read_entry(&mut entry)? {
		if pargs.is_filtered(&entry.path) {
			continue;
		}

		let sf = if human_readable {
			SizeFormat::Human(entry.size)
		} else {
			SizeFormat::Raw(entry.size)
		};

		writeln!(out, cstr!("  <b>{:>7}</> {:#} {}"), sf, entry.timestamp, entry.path)?;
		count += 1;
	}

	writeln!(out, cstr!("total: <g>{}</> entries.\n"), count)?;
	Ok(())
}
