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

#[test]
fn writer() {
	let mut inner = Cursor::new(Vec::with_capacity(200));
	let mut writer = Writer::new(&mut inner);
	let mut hash = Digest([
		0x01, 0x02, 0x03, 0x04, 0x55, 0x66, 0x77, 0x88, 0x0a, 0x0b, 0x0c, 0x0d, 0xde, 0xad, 0xbe, 0xef,
	]);

	if let Err(e) = writer.write("md/file 1.xml", 1234, 123456789.into(), &hash) {
		panic!("{e}");
	}

	hash.reverse();
	if let Err(e) = writer.write("md/file 2.xml", 5678, 987654321.into(), &hash) {
		panic!("{e}");
	}

	drop(writer); // so i don't have to use ugly scoping hacks
	assert_eq!(
		str::from_utf8(inner.get_ref()).unwrap(),
		"md/file 1.xml 1234 123456789 01020304556677880a0b0c0ddeadbeef
md/file 2.xml 5678 987654321 efbeadde0d0c0b0a8877665504030201\n"
	);
}

#[test]
fn write_entry() {
	let mut inner = Cursor::new(Vec::with_capacity(100));
	let mut writer = Writer::new(&mut inner);
	let entry = Entry {
		path: "md/file 1.xml".into(),
		hash: Digest([
			0x01, 0x02, 0x03, 0x04, 0x55, 0x66, 0x77, 0x88, 0x0a, 0x0b, 0x0c, 0x0d, 0xde, 0xad, 0xbe, 0xef,
		]),
		size: 1234,
		timestamp: 123456789.into(),
		..Default::default()
	};

	if let Err(e) = writer.write_entry(&entry) {
		panic!("{e}");
	}

	drop(writer);
	assert_eq!(
		str::from_utf8(inner.get_ref()).unwrap(),
		"md/file 1.xml 1234 123456789 01020304556677880a0b0c0ddeadbeef\n"
	);
}
