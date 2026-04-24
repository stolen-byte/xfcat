// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
	fs::{self, File},
	io::Write,
	path::{Path, PathBuf},
};

use anstream::stdout;
use anyhow::{Context, Result, anyhow, bail};
use color_print::cstr;
use indicatif::{ProgressBar, ProgressDrawTarget};

use crate::commands::common::FilterArgs;
use xf::{
	cat,
	fs::{FsEntry, path::PathContext, walk},
	io::{StreamCopier, TeeWriter},
	md5, utils,
};

// =============================================================================
#[derive(clap::Args)]
pub struct Command {
	/// source directory
	dir: PathBuf,

	/// output name
	///
	/// [default: same as source directory name]
	#[arg(short, long)]
	name: Option<String>,

	/// output directory
	///
	/// [default: current directory]
	#[arg(short, long)]
	out: Option<PathBuf>,

	#[command(flatten)]
	filter: FilterArgs,
}

impl Command {
	pub fn execute(&self) -> Result<()> {
		utils::init_threadpool(None);
		let source = fs::canonicalize(&self.dir).with_context(|| self.dir.as_context())?;
		if !source.is_dir() {
			bail!("{}: not a directory", source.display());
		}

		let name = self.name.as_deref().map_or_else(
			|| {
				source
					.file_name()
					.ok_or_else(|| {
						anyhow!("source directory has no name component, please use the --name option")
					})
					.and_then(|f| {
						f.to_str().ok_or_else(|| {
							anyhow!("directory name of the source dir must contain only utf8 characters")
						})
					})
			},
			Ok,
		)?;

		let dest = self
			.out
			.as_ref()
			.map_or_else(std::env::current_dir, |o| Ok(o.clone()))
			.map(|p| p.join(name))?;

		writeln!(stdout().lock(), cstr!("<b>::</> scanning source files..."))?;
		let mut total = 0;
		let sources = walk(&source)?
			.into_iter()
			.filter(|r| {
				r.as_ref()
					.map_or(true, |f| !self.filter.is_filtered(f.relative()))
			})
			// hacky but meh
			.inspect(|r| {
				if let Ok(f) = r {
					total += f.size();
				}
			})
			.collect::<Result<Vec<_>>>()?;

		let pb = utils::create_progress_bar(Some(total), ProgressDrawTarget::stdout());
		match pack_files(sources, dest, &pb) {
			Ok(count) => {
				pb.finish_with_message(format!(cstr!("written <b>{}</> entries"), count));
				writeln!(stdout().lock(), cstr!("<b>::</> done in <g>{:?}</>\n"), pb.elapsed())?;
			}
			Err(e) => {
				pb.abandon_with_message("cancelled");
				return Err(e);
			}
		}

		Ok(())
	}
}

// =============================================================================
fn pack_files<P>(sources: Vec<FsEntry>, dest: P, pb: &ProgressBar) -> Result<usize>
where
	P: AsRef<Path>,
{
	let dest = dest.as_ref();

	if let Some(parent) = dest.parent() {
		fs::create_dir_all(parent)?;
	}

	let mut copier = StreamCopier::new();
	let cfile = dest.with_extension("cat");
	let dfile = dest.with_extension("dat");
	let mut cat = cat::Writer::new(File::create(&cfile).with_context(|| cfile.as_context())?);
	let mut dat = File::create(&dfile).with_context(|| dfile.as_context())?;

	pb.suspend(|| {
		let mut out = stdout().lock();
		_ = writeln!(&mut out, cstr!("<b>::</> packing to:"));
		_ = writeln!(&mut out, cstr!("  <g>></> <m>{}</>"), cfile.display());
		_ = writeln!(&mut out, cstr!("  <g>></> <m>{}</>"), dfile.display());
	});

	sources.into_iter().try_fold(0usize, |count, entry| {
		pb.set_message(entry.relative().to_owned());

		let bytes =
			write(&entry, &mut cat, &mut dat, &mut copier).with_context(|| entry.relative().to_owned())?;

		pb.inc(bytes);
		Ok(count + 1)
	})
}

fn write<C, D>(
	entry: &FsEntry,
	cat: &mut cat::Writer<C>,
	dat: &mut D,
	copier: &mut StreamCopier,
) -> Result<u64>
where
	C: Write,
	D: Write,
{
	let mut file = File::open(entry.path())?;
	let mut ctx = md5::Context::new();
	let mut writer = TeeWriter::new(&mut ctx, dat);

	let bytes = copier.copy(&mut file, &mut writer)?;
	cat.write(entry.relative(), bytes, entry.modified().into(), &ctx.finalize())?;

	Ok(bytes)
}
