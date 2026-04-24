// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
	fs::{self, Metadata},
	path::{Path, PathBuf},
	time::SystemTime,
};

use anyhow::{Context, Result, bail};

use super::path::PathContext;

// =============================================================================
macro_rules! otry {
	($e:expr) => {
		match $e {
			Ok(v) => v,
			Err(err) => return Some(Err(From::from(err))),
		}
	};
	($e:expr, $c:expr) => {
		match $e {
			Ok(v) => v,
			Err(err) => {
				return Some(Err(err).with_context(|| $c.as_context()));
			}
		}
	};
}

// =============================================================================
pub struct FsEntry {
	full_path: PathBuf,
	mtime: SystemTime,
	relative: String,
	size: u64,
}

impl FsEntry {
	fn from(root: &Path, full_path: PathBuf, meta: &Metadata) -> Result<Self> {
		let mtime = meta.modified()?;
		// unwrap if fine here, as we will never be passing a root that isn't a prefix of full_path
		let relative = match full_path.strip_prefix(root).unwrap().as_os_str().to_str() {
			Some(rel) => rel.to_owned(),
			None => bail!("path contains invalid utf8 characters"),
		};

		Ok(Self {
			full_path,
			mtime,
			relative,
			size: meta.len(),
		})
	}

	pub fn path(&self) -> &Path {
		&self.full_path
	}

	pub fn size(&self) -> u64 {
		self.size
	}

	pub fn modified(&self) -> &SystemTime {
		&self.mtime
	}

	pub fn relative(&self) -> &str {
		&self.relative
	}
}

pub struct FsWalker {
	root: PathBuf,
}

impl FsWalker {
	pub fn new(root: PathBuf) -> Self {
		Self { root }
	}
}

impl IntoIterator for FsWalker {
	type Item = <Self::IntoIter as Iterator>::Item;
	type IntoIter = FsIter;

	fn into_iter(self) -> Self::IntoIter {
		FsIter {
			root: self.root.clone(),
			cur: None,
			deferred: vec![self.root],
		}
	}
}

pub struct FsIter {
	root: PathBuf,
	cur: Option<fs::ReadDir>,
	deferred: Vec<PathBuf>,
}

impl Iterator for FsIter {
	type Item = Result<FsEntry>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(cur) = self.cur.as_mut() {
			match cur.next() {
				Some(Ok(entry)) => {
					let mut t = otry!(entry.file_type(), entry.path());
					let mut meta = otry!(entry.metadata(), entry.path());
					let path = entry.path();

					// NOTE:
					// since i cannot think of any situation where you would want/need to pack a
					// symlinked directory, we're skipping them entirely, symlinked files however, can
					// be useful, so those are returned.
					if t.is_symlink() {
						meta = otry!(fs::metadata(&path), entry.path());
						t = meta.file_type();
						if t.is_dir() {
							continue;
						}
					}

					if t.is_dir() {
						self.deferred.push(path);
					} else {
						return Some(FsEntry::from(&self.root, path, &meta));
					}
				}
				Some(Err(e)) => return Some(Err(e.into())),
				None => {
					self.cur = None;
				}
			}
		}

		if let Some(dir) = self.deferred.pop() {
			self.cur.replace(otry!(fs::read_dir(&dir), dir));
			return self.next(); //tail(ish) recursion
		}

		None
	}
}

// =============================================================================
pub fn walk<P: AsRef<Path>>(root: P) -> Result<FsWalker> {
	Ok(FsWalker::new(root.as_ref().to_path_buf()))
}
