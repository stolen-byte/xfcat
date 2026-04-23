// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
	collections::HashSet,
	fs::{self, File},
	io::Write,
	path::{Path, PathBuf},
};

use anstream::stdout;
use anyhow::{Context, Result, bail};
use color_print::cstr;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use rayon::prelude::*;

use crate::{commands::common::FilterArgs, log};
use xf::{
	cat::{self, Entry, Result as CatResult},
	io::{StreamCopier, TeeWriter},
	md5,
	utils::{self, PathContext},
};

// =============================================================================
#[derive(clap::Args)]
pub struct Command {
	/// input files
	#[arg(required = true)]
	inputs: Vec<PathBuf>,

	/// output directory
	#[arg(short, long, name = "DIR", default_value = "./out")]
	out: PathBuf,

	/// number of threads to use
	#[arg(short, long, name = "COUNT")]
	threads: Option<usize>,

	/// skip verification of file hashes
	#[arg(short, long, default_value_t = false)]
	no_verify: bool,

	/// create separate subdirectories for each package
	///
	/// subdirectories are named according to the parent directory of
	/// each package
	#[arg(short, long, default_value_t = false)]
	use_subdirs: bool,

	#[command(flatten)]
	filter: FilterArgs,
}

impl Command {
	pub fn execute(&self) -> Result<()> {
		utils::init_threadpool(self.threads);

		let mp = MultiProgress::with_draw_target(ProgressDrawTarget::stdout());
		let now = std::time::Instant::now();
		writeln!(stdout().lock(), cstr!("<b>::</> extracting packages..."))?;

		let jobs = build_jobs(&self.inputs, &self.filter, &self.out, self.use_subdirs);
		let prefix = utils::common_prefix(jobs.iter().map(|j| j.source.as_path())).unwrap_or_default();

		jobs.into_par_iter().for_each(|job| {
			// NOTE:
			// not using `enable_steady_tick()` because that spawns an extra system thread
			// for every single progress bar...:(
			// it is also for this reason we don't include a spinner in the style template, as
			// they look absolutely weird without steady tick.
			let pb = utils::add_progress(job.total_size, &mp);
			let dpath = job.source.strip_prefix(&prefix).unwrap();
			pb.set_message(dpath.to_string_lossy().into_owned());

			match extract_all(job, !self.no_verify, &pb) {
				Ok(_) => pb.finish(),
				Err(e) => pb.abandon_with_message(log::format_error!("{e:#}")),
			}
		});

		writeln!(stdout().lock(), cstr!("<b>::</> done in <g>{:?}</>\n"), now.elapsed())?;
		Ok(())
	}
}

// =============================================================================
type PartialJob = (PathBuf, Vec<Entry>);

struct JobData {
	source: PathBuf,
	dest: PathBuf,
	entries: Vec<Entry>,
	total_size: u64,
}

fn build_jobs(inputs: &[PathBuf], filter: &FilterArgs, out: &Path, subdirs: bool) -> Vec<JobData> {
	let initial = inputs
		.par_iter()
		.map(|p| load_entries(fs::canonicalize(p)?, filter))
		.inspect(|j| {
			if let Err(e) = j {
				log::error!("{e:#}");
			}
		})
		.filter_map(Result::ok)
		.collect::<Vec<PartialJob>>();

	let mut result = Vec::with_capacity(initial.len());
	let mut seen = HashSet::new();

	// dedupe entries across all jobs
	// initial will contain each job in order of ascending priority
	// the key here is: the *last* entry encountered (in the source) always wins
	for (mut source, pending) in initial.into_iter().rev() {
		let mut entries = Vec::with_capacity(pending.len());
		let mut total_size = 0;

		// also iterate entries in reverse, so we don't have to remove duplicates
		for entry in pending.into_iter().rev() {
			let key = utils::make_hash(&entry.path);
			let extract = entry.size != 0;

			if seen.insert(key) {
				if extract {
					total_size += entry.size;
					entries.push(entry);
				}
			}
		}

		source.set_extension("dat");

		let mut dest = out.to_owned();
		if subdirs {
			// relies on the fact that we canonicalize the source paths
			let sub = source
				.parent()
				.and_then(|p| p.file_name())
				.or_else(|| source.file_prefix())
				.unwrap();

			dest.push(sub);
		}

		// note: we keep empty jobs just for display purposes
		result.push(JobData {
			source,
			dest,
			entries,
			total_size,
		});
	}

	result
}

/// parse .cat file and return a `PartialJob` describing files to be extracted
fn load_entries(source: PathBuf, filter: &FilterArgs) -> Result<PartialJob> {
	let source = match source.extension().and_then(|ext| ext.to_str()) {
		Some("dat") | None => source.with_extension("cat"),
		_ => source,
	};

	let file = File::open(&source).with_context(|| source.as_context())?;
	let entries = cat::Reader::new(file)
		.filter(|r| r.as_ref().map_or(true, |entry| !filter.is_filtered(&entry.path)))
		.collect::<CatResult<Vec<Entry>>>()?;

	Ok((source, entries))
}

fn extract_all(job: JobData, verify: bool, pb: &ProgressBar) -> Result<()> {
	use std::io::{Error, ErrorKind};

	let mut copier = StreamCopier::new();
	let mut dat = File::open(&job.source).with_context(|| job.source.as_context())?;
	let mut buf = job.dest;

	for entry in job.entries {
		let dest = utils::join_path(&mut buf, &entry.path);
		if let Some(parent) = dest.parent() {
			fs::create_dir_all(parent).with_context(|| parent.as_context())?;
		}

		let mut out = File::create(&dest).with_context(|| dest.as_context())?;
		let mut source = entry.reader(&mut dat)?;

		let written = if verify {
			let mut ctx = md5::Context::new();
			let mut writer = TeeWriter::new(&mut out, &mut ctx);
			let result = copier.copy(&mut source, &mut writer)?;
			if ctx.finalize() != entry.hash {
				pb.suspend(|| log::warn!("{}: hash mismatch", entry.path));
			}
			result
		} else {
			copier.copy(&mut source, &mut out)?
		};

		if written < entry.size {
			bail!("{}: {}", entry.path, Error::from(ErrorKind::UnexpectedEof));
		}

		out.set_times(entry.timestamp.as_file_time())?;
		pb.inc(entry.size);
	}

	Ok(())
}
