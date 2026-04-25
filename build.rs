// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::disallowed_macros)]
use std::{ffi::OsStr, process::Command};

use semver::{BuildMetadata, Prerelease, Version};

// =============================================================================
const TARBALL_VERSION: &str = "$Format:%(describe)$";

// =============================================================================
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn run_git<I, S>(args: I) -> Result<(bool, String)>
where
	S: AsRef<OsStr>,
	I: IntoIterator<Item = S>,
{
	let out = Command::new("git").args(args).output()?;
	if !out.status.success() {
		return Ok((false, String::from_utf8_lossy(&out.stderr).trim().to_owned()));
	}
	Ok((true, String::from_utf8_lossy(&out.stdout).trim().to_owned()))
}

fn version_from_tag<S: AsRef<str>>(tag: S) -> Result<String> {
	let mut v = Version::parse(tag.as_ref().trim_start_matches('v'))?;

	if v.pre != Prerelease::EMPTY {
		// there are only 3 valid combinations of `parts`
		// <pre>    None       None
		// <commit> <distance> <pre>
		// <commit> <distance>
		let mut parts = v.pre.as_str().rsplitn(3, '-');

		let (pre, dist, commit) = match (parts.next(), parts.next(), parts.next()) {
			(Some(pre), None, None) => (pre, "", ""),
			(Some(commit), Some(dist), Some(pre)) => (pre, dist, commit),
			(Some(commit), Some(dist), None) => ("", dist, commit),
			_ => return Err("empty tag".into()),
		};

		// this will only be non-empty if we're on an untagged commit, *after* a previous tag,
		// and therefore we have a valid need for this
		if !commit.is_empty() {
			v.build = BuildMetadata::new(commit)?;
		}

		if pre.is_empty() {
			// increment patch so this version sorts after the tag's normal release
			// semver sorts anything with a prerelease, *before* the matching release
			v.patch = v.patch.saturating_add(1);
			v.pre = Prerelease::new(&format!("dev.{dist}"))?;
		} else {
			// custom pre-release data present
			v.pre = Prerelease::new(pre)?;
		}
	}

	Ok(v.to_string())
}

fn version_from_git() -> Result<String> {
	let (status, tag) = run_git(["describe", "--tags"])?;
	if !status {
		if !tag.contains("No names found") {
			return Err(tag.into());
		}

		let (status, commit) = run_git(["rev-parse", "--short=7", "HEAD"])?;
		if !status {
			return Err(commit.into());
		}

		return Ok(format!("0.0.1-dev+g{commit}"));
	}
	version_from_tag(tag)
}

// =============================================================================
fn main() {
	let version = match std::env::var("XFCAT_VERSION") {
		Ok(v) => v,
		Err(_) => {
			if TARBALL_VERSION.starts_with('$') {
				match version_from_git() {
					Ok(v) => v,
					Err(e) => {
						println!("cargo::error={e}");
						return;
					}
				}
			} else {
				String::from(TARBALL_VERSION)
			}
		}
	};
	println!("cargo::rustc-env=CARGO_PKG_VERSION={}", version.trim_start_matches('v'));
	println!("cargo::rerun-if-env-changed=XFCAT_VERSION");
	println!("cargo::rerun-if-changed=build.rs");
}
