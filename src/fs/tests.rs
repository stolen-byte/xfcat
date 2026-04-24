// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::{Path, PathBuf};

use super::path::{self, PathContext};

// =============================================================================
#[test]
fn as_context() {
	assert_eq!("/some/file.txt".to_string(), Path::new("/some/file.txt").as_context());
}

#[test]
fn join_path() {
	let tests = [
		// base, sub, expected
		("", "/some/file.txt", "/some/file.txt"), // empty base path is fine
		("", "some/file.txt", "some/file.txt"),
		("", "/", "/"),
		("/", "/some/file.txt", "/some/file.txt"),
		("/", "some/file.txt", "/some/file.txt"),
		("/some/path", "to/a/file.txt", "/some/path/to/a/file.txt"),
		("/some/path", "/to/a/file.txt", "/some/path/to/a/file.txt"),
		("/some/path", "////to/a/file.txt", "/some/path/to/a/file.txt"),
	];

	for (base, sub, expected) in tests {
		let mut buf = PathBuf::from(base);
		let joined = path::join_path(&mut buf, sub);
		assert_eq!(joined.as_os_str(), expected, "while testing '{base}' + '{sub}'");
		drop(joined);
		assert_eq!(buf.as_os_str(), base, "original wasn't restored");
	}
}

#[test]
#[should_panic]
fn join_empty_path() {
	let mut buf = PathBuf::from("/some/path");
	path::join_path(&mut buf, "");
}

#[test]
fn common_prefix() {
	let all = [
		"/foo/bar/baz/one.txt",
		"/foo/bar/quux/quuux/two.txt",
		"/foo/bar/baz/foo/bar.txt",
	];

	let result = path::common_prefix(all.into_iter());
	assert_eq!(result.as_deref(), Some(Path::new("/foo/bar")));
}

#[test]
fn common_prefix_none() {
	let all = ["foo/bar/baz.txt", "bar/baz/qux.txt", "baz/qux.txt"];

	let result = path::common_prefix(all.into_iter());
	assert_eq!(result, None);
}
