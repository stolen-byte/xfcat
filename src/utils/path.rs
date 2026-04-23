// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
	ffi::OsStr,
	path::{Component, Path, PathBuf},
};

// =============================================================================
pub struct JoinedPath<'a> {
	buf: &'a mut PathBuf,
	len: usize,
}

impl<'a> JoinedPath<'a> {
	fn new(buf: &'a mut PathBuf, len: usize) -> Self {
		Self { buf, len }
	}
}

// Bit of the old RAII abuse
impl Drop for JoinedPath<'_> {
	fn drop(&mut self) {
		if self.len == 0 {
			// pop will leave a lone separator untouched, making the below while loop infinite
			self.buf.clear();
		} else {
			// this will be a lot simpler if `OsString::truncate` ever gets stabilised, alas:
			// stabilisation in rust takes longer that the damn c++ committee (and i never thought i'd say that)
			while self.buf.as_os_str().len() > self.len {
				// pop() will just truncate() to the parent's len()
				self.buf.pop();
			}
		}
	}
}

impl AsRef<Path> for JoinedPath<'_> {
	fn as_ref(&self) -> &Path {
		self.buf
	}
}

// so we can use Path members directly without going through `as_ref()`
impl std::ops::Deref for JoinedPath<'_> {
	type Target = Path;

	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}

// =============================================================================
/// assumes `base` is already initialised with the correct base path.
/// panics if `rest` is empty
pub fn join_path<P: AsRef<Path>>(base: &mut PathBuf, rest: P) -> JoinedPath<'_> {
	let len = base.as_os_str().len();
	let mut rest = rest.as_ref();

	assert!(!rest.as_os_str().is_empty(), "attempted to join_path with an empty sub-path");

	if len > 0 && rest.is_absolute() {
		// i refuse to iterate over components() and push() each one
		// reasoning: `push()` will unnecessarily call `components()` itself on each iteration.
		let mut bytes = rest.as_os_str().as_encoded_bytes();
		while bytes[0] == b'/' || bytes[0] == b'\\' {
			bytes = &bytes[1..];
		}
		rest = unsafe { Path::new(OsStr::from_encoded_bytes_unchecked(bytes)) };
	}
	base.push(rest);

	JoinedPath::new(base, len)
}

/// find the common prefix among any number of paths
pub fn common_prefix<P, I>(mut iter: I) -> Option<PathBuf>
where
	P: AsRef<Path>,
	I: Iterator<Item = P>,
{
	let first = iter.next()?; // prevent temporary being dropped too early
	let first = first.as_ref();
	let mut comps = first.components().collect::<Vec<Component>>();

	for p in iter {
		let mut i = 0;
		for (l, r) in comps.iter().zip(p.as_ref().components()) {
			if l == &r {
				i += 1;
			} else {
				break;
			}
		}
		comps.truncate(i);
		if comps.is_empty() {
			return None;
		}
	}

	Some(comps.into_iter().collect::<PathBuf>())
}
