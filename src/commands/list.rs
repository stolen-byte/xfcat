// SPDX-License-Identifier: GPL-3.0-or-later
use super::common::{FilterArgs, SortArgs, SortMode};
use crate::log;
use xf::cat::{self, Entry, Result as CatResult};
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
	filter: FilterArgs,

	#[command(flatten)]
	sort: SortArgs,
}

impl Command {
	pub fn execute(&self) -> Result<()> {
		for path in &self.inputs {
			// allow specifying path to either .cat, .dat, or omission.
			let source = match path.extension().and_then(|e| e.to_str()) {
				Some("dat") | None => path.with_extension("cat"),
				_ => path.clone(),
			};

			if let Err(e) = list(source, self.human_readable, &self.filter, &self.sort) {
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
fn list<P: AsRef<Path>>(source: P, human_readable: bool, filter: &FilterArgs, sort: &SortArgs) -> Result<()> {
	let mut out = stdout().lock();
	let source = source.as_ref();
	let reader = cat::Reader::new(File::open(source).with_context(|| source.as_context())?);

	let iter = reader.filter(|r| r.as_ref().map_or(true, |entry| !filter.is_filtered(&entry.path)));

	writeln!(out, cstr!("<m><u>{}</>:"), source.display())?;
	let count = if sort.by.name || sort.by.size || sort.by.time {
		list_sorted(iter, human_readable, &sort.by, sort.reverse, &mut out)
	} else {
		list_entries(iter, human_readable, &mut out)
	}?;

	writeln!(out, cstr!("total: <g>{}</> entries.\n"), count)?;
	Ok(())
}

fn list_sorted<I, O>(
	entries: I,
	human_readable: bool,
	sort: &SortMode,
	reverse: bool,
	out: &mut O,
) -> Result<usize>
where
	I: Iterator<Item = CatResult<Entry>>,
	O: Write,
{
	let mut tmp = entries.collect::<CatResult<Vec<_>>>()?; // short circuit errors here
	match (sort.name, sort.size, sort.time) {
		(true, _, _) => tmp.sort_by(|a, b| Ord::cmp(&a.path, &b.path)), // alphabetical (case-sensitive)
		(_, true, _) => tmp.sort_by(|a, b| Ord::cmp(&b.size, &a.size)), // descending
		(_, _, true) => tmp.sort_by(|a, b| Ord::cmp(&b.timestamp, &a.timestamp)), // descending
		_ => std::unreachable!(),
	}

	let iter = tmp.into_iter().map(Ok);
	if reverse {
		list_entries(iter.rev(), human_readable, out)
	} else {
		list_entries(iter, human_readable, out)
	}
}

fn list_entries<I, O>(entries: I, human_readable: bool, out: &mut O) -> Result<usize>
where
	I: Iterator<Item = CatResult<Entry>>,
	O: Write,
{
	let mut count = 0;
	for entry in entries {
		let entry = entry?;

		let sf = if human_readable {
			SizeFormat::Human(entry.size)
		} else {
			SizeFormat::Raw(entry.size)
		};

		writeln!(out, cstr!("  <b>{:>7}</> {:#} {}"), sf, entry.timestamp, entry.path)?;
		count += 1;
	}

	Ok(count)
}
