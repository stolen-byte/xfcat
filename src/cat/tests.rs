// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use std::io::Cursor;

// =============================================================================
#[test]
fn reader_read_entry() {
	let mut r = Reader::new(Cursor::new(
		b"md/file 1.xml 1234 1234567890 abcdabcdabcdabcdabcdabcdabcdabcd
md/file 2.xml 5678 0987654321 dcbadcbadcbadcbadcbadcbadcbadcba",
	));

	let expected = [
		Entry {
			path: "md/file 1.xml".into(),
			hash: Digest([
				0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab,
				0xcd,
			]),
			timestamp: 1234567890.into(),
			size: 1234,
			offset: 0,
		},
		Entry {
			path: "md/file 2.xml".into(),
			hash: Digest([
				0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc,
				0xba,
			]),
			timestamp: 987654321.into(),
			size: 5678,
			offset: 1234,
		},
	];

	let mut buf = Entry::with_capacity(64);
	for expect in &expected {
		match r.read_entry(&mut buf) {
			Ok(true) => assert_eq!(&buf, expect),
			Ok(false) => panic!(" got: EOF\nwant: {expect:?}"),
			Err(e) => panic!(" got: {e}\nwant: {expect:?}"),
		}
	}

	assert!(!r.read_entry(&mut buf).unwrap(), "expected EOF");
}

#[test]
fn reader_iter() {
	let r = Reader::new(Cursor::new(
		b"md/file 1.xml 1234 1234567890 abcdabcdabcdabcdabcdabcdabcdabcd
md/file 2.xml 5678 0987654321 dcbadcbadcbadcbadcbadcbadcbadcba",
	));

	let expected = [
		Entry {
			path: "md/file 1.xml".into(),
			hash: Digest([
				0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab, 0xcd, 0xab,
				0xcd,
			]),
			timestamp: 1234567890.into(),
			size: 1234,
			offset: 0,
		},
		Entry {
			path: "md/file 2.xml".into(),
			hash: Digest([
				0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc, 0xba, 0xdc,
				0xba,
			]),
			timestamp: 987654321.into(),
			size: 5678,
			offset: 1234,
		},
	];

	for (got, want) in r.zip(expected.iter()) {
		assert_eq!(&got.unwrap(), want);
	}
}

#[test]
fn reader_invalid_entries() {
	let run = |s: &str, desc: &str| {
		let mut reader = Reader::new(Cursor::new(s));
		match reader.next() {
			Some(Err(Error::ParseError(..))) => (),
			x => panic!("[{desc}] expected ParseError, got:\n{x:?}"),
		}
	};

	run("path/to/file.xml ABCD 1234567890 abcdabcdabcdabcdabcdabcdabcdabcd\n", "invalid size");
	run("path/to/file.xml 1234 ABCDEFGHIJ abcdabcdabcdabcdabcdabcdabcdabcd\n", "invalid timestamp");
	run("path/to/file.xml 1234 ABCDEFGHIJ abcdabcdabcdabcdxxxxabcdabcdabcd\n", "invalid hash character");
	run("path/to/file.xml 1234 ABCDEFGHIJ abcdabcdabcdabcd\n", "invalid hash length");
	run("path/to/file.xml 1234 1234567890\n", "missing hash");
	run("path/to/file.xml 1234\n", "missing timestamp");
	run("path/to/file.xml\n", "missing size");
	run("\n", "empty line");
}

#[test]
fn reader_error_includes_line_no() {
	let mut reader = Reader::new(Cursor::new(
		b"file1.xml 1234 1234567890 abcdabcdabcdabcdabcdabcdabcdabcd
file2.xml 1234 1234567890 abcdabcdabcdabcdabcdabcdabcdabcd
file3.xml ASDF 1234567890 abcdabcdabcdabcdabcdabcdabcdabcd",
	));

	reader.next().unwrap().unwrap();
	reader.next().unwrap().unwrap();
	match reader.next() {
		Some(Err(Error::ParseError(n, _))) => assert_eq!(n, 2, "expected line #2, got: #{n}"),
		x => panic!("expected Err(ParseError(...)), got {x:?}"),
	}
}
