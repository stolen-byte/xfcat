// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
	fs::File,
	io::Write,
	path::{Path, PathBuf},
};

use anstream::stdout;
use anyhow::{Context, Result};
use color_print::cstr;
use rayon::prelude::*;

use crate::{
	commands::common::{FilterArgs, SortArgs},
	log,
};
use xf::{
	cat::{self, Entry, Result as CatResult},
	fs::path::PathContext,
	utils::{self, SizeFormat},
};

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
	filter: FilterArgs,

	#[command(flatten)]
	sort: SortArgs,
}

impl Command {
	pub fn execute(&self) -> Result<()> {
		utils::init_threadpool(None);

		self.inputs
			.par_iter()
			.enumerate()
			.map(|(i, p)| (i, load_entries(p, &self.filter, &self.sort)))
			.try_for_each(|(i, result)| {
				// lock here so smaller packages can be slowly writing out to term while others are processing
				let mut out = stdout().lock();
				let source = &self.inputs[i];

				match result {
					Ok(entries) => {
						let count = entries.len();
						writeln!(out, cstr!("\n<m><u>{}</>:"), source.display())?;

						for entry in entries {
							let sf = if self.human_readable {
								SizeFormat::Human(entry.size)
							} else {
								SizeFormat::Raw(entry.size)
							};
							writeln!(out, cstr!("  <b>{:>7}</> {:#} {}"), sf, entry.timestamp, entry.path)?;
						}

						writeln!(out, cstr!("total: <g>{}</> entries.\n"), count)?;
					}
					Err(e) => {
						if crate::is_sigpipe(&e) {
							return Err(e);
						}
						log::error!("{e:#}");
					}
				}

				Ok(())
			})
	}
}

// =============================================================================
fn load_entries(source: &Path, filter: &FilterArgs, sort: &SortArgs) -> Result<Vec<Entry>> {
	// allow specifying path to either .cat, .dat, or omission.
	let source = match source.extension().and_then(|e| e.to_str()) {
		Some("dat") | None => source.with_extension("cat"),
		_ => source.to_owned(),
	};

	let file = File::open(&source).with_context(|| source.as_context())?;
	let mut result = cat::Reader::new(file)
		.filter(|r| r.as_ref().map_or(true, |entry| !filter.is_filtered(&entry.path)))
		.collect::<CatResult<Vec<Entry>>>()?;

	match (sort.by.name, sort.by.size, sort.by.time) {
		(true, _, _) => result.sort_by(|a, b| Ord::cmp(&a.path, &b.path)), // alphabetical (case-sensitive)
		(_, true, _) => result.sort_by(|a, b| Ord::cmp(&b.size, &a.size)), // descending
		(_, _, true) => result.sort_by(|a, b| Ord::cmp(&b.timestamp, &a.timestamp)), // descending
		_ => (),
	}

	if sort.reverse {
		result.reverse();
	}

	Ok(result)
}
