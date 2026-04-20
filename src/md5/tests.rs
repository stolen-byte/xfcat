// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;

// =============================================================================
#[test]
fn md5_digest_parse() {
	let actual: Digest = match "01020304556677880a0b0c0ddeadbeef".parse() {
		Ok(r) => r,
		Err(e) => panic!("{e}"),
	};

	assert_eq!(
		actual,
		Digest([
			0x01, 0x02, 0x03, 0x04, 0x55, 0x66, 0x77, 0x88, 0x0a, 0x0b, 0x0c, 0x0d, 0xde, 0xad, 0xbe, 0xef,
		])
	);
}

#[test]
fn md5_digest_parse_error() {
	match "010k01".parse::<Digest>() {
		Ok(r) => panic!("expected error, got: {r}"),
		Err(e) => assert_eq!(DigestError::HashLength(6), e, "error should contain correct hash length"),
	}

	match "010k0304556677880a0b0c0ddeadbeef".parse::<Digest>() {
		Ok(r) => panic!("expected error, got: {r}"),
		Err(e) => {
			assert_eq!(DigestError::InvalidCharacter('k', 3), e, "error should contain invalid char & pos");
		}
	}
}

#[test]
fn md5_digest_formatting() {
	let digest = Digest([
		0x01, 0x02, 0x03, 0x04, 0x55, 0x66, 0x77, 0x88, 0x0a, 0x0b, 0x0c, 0x0d, 0xde, 0xad, 0xbe, 0xef,
	]);

	assert_eq!("01020304556677880a0b0c0ddeadbeef", digest.to_string());
	assert_eq!("0x01020304556677880a0b0c0ddeadbeef", format!("{digest:#}"));
}

#[test]
fn md5() {
	// test data from https://www.ietf.org/rfc/rfc1321.txt
	let tests = [
		("", "d41d8cd98f00b204e9800998ecf8427e"),
		("a", "0cc175b9c0f1b6a831c399e269772661"),
		("abc", "900150983cd24fb0d6963f7d28e17f72"),
		("message digest", "f96b697d7cb7938d525a2f31aaf161d0"),
		("abcdefghijklmnopqrstuvwxyz", "c3fcd3d76192e4007dfb496cca67e13b"),
		(
			"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
			"d174ab98d277d9f5a5611c2c9f419d9f",
		),
		(
			"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
			"57edf4a22be3c955ac49da2e2107b67a",
		),
	];

	for (input, expected) in tests {
		let mut ctx = Context::new();

		ctx.update(input);
		let digest = ctx.finalize();

		assert_eq!(digest.to_string(), expected);
	}
}

#[test]
fn md5_write() {
	const DATA: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
	let mut ctx = Context::new();
	assert_eq!(ctx.write(DATA).unwrap(), DATA.len());

	let digest = ctx.finalize();
	assert_eq!(digest.to_string(), "c3fcd3d76192e4007dfb496cca67e13b");
}
