// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(clippy::disallowed_macros)]
use std::{ffi::OsStr, process::Command};

use regex_lite::Regex;
use semver::Version;

// =============================================================================
const UNVERSIONED: &str = "0.0.1";
const GIT_EXPORT_TAG: &str = "$Format:%(describe:abbrev=7,tags=true)$";
const GIT_EXPORT_COMMIT: &str = "$Format:%h$";
// no way to get *just* the branch name, so we're gonna have to hack this down to something parseable
const GIT_EXPORT_BRANCH: &str = "$Format:%(decorate:prefix=,suffix=,separator=:,pointer=:,tag=)$";

// =============================================================================
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct Repo {
	tag: String,
	branch: Option<String>,
	commit: String,
	is_exact: bool,
}

// =============================================================================
fn run_git<I, S>(args: I) -> Result<String>
where
	S: AsRef<OsStr>,
	I: IntoIterator<Item = S>,
{
	let out = Command::new("git").args(args).output()?;
	if !out.status.success() {
		return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned().into());
	}
	Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn parse_tag(mut tag: String) -> (bool, String) {
	if tag.is_empty() {
		return (false, String::from(UNVERSIONED));
	}

	let rx = Regex::new(r"-\d+-g[0-9A-Fa-f]{7}$").unwrap();
	let mut exact = true;

	if let Some(m) = rx.find(&tag) {
		tag.truncate(m.start());
		exact = false;
	}

	(exact, tag)
}

fn get_repo() -> Result<Repo> {
	let branch = run_git(["rev-parse", "--abbrev-ref", "HEAD"])?;
	let commit = run_git(["rev-parse", "--short=7", "HEAD"])?;
	let (is_exact, tag) = match run_git(["describe", "--tags"]) {
		Ok(t) => parse_tag(t),
		Err(e) => {
			// no way to downcast this error back to a string, because rust is janky af
			if !e.to_string().contains("No names found") {
				return Err(e);
			}
			(false, String::from(UNVERSIONED))
		}
	};

	Ok(Repo {
		tag,
		branch: Some(branch),
		commit,
		is_exact,
	})
}

fn get_version() -> Result<Version> {
	let repo = if GIT_EXPORT_COMMIT.starts_with('$') {
		get_repo()?
	} else {
		// this is a generated source tarball/git export
		let (is_exact, tag) = parse_tag(String::from(GIT_EXPORT_TAG));

		// parse branch name
		// can be
		// - empty (if export is for a specific commit)
		// - HEAD:<branch>[:...]
		// - <tag>
		// we're only interested in the 2nd one
		let branch = if GIT_EXPORT_BRANCH.is_empty() {
			None
		} else {
			let mut parts = GIT_EXPORT_BRANCH.split(':');
			parts.next().is_some().then(|| parts.next().unwrap().to_owned())
		};

		Repo {
			tag,
			branch,
			commit: String::from(GIT_EXPORT_COMMIT),
			is_exact,
		}
	};

	// everything gets put after '+' so we don't restrict having our own pre-release tags
	let tag = if repo.is_exact {
		repo.tag
	} else if let Some(branch) = repo.branch {
		format!("{}+{}-g{}", repo.tag, branch, repo.commit)
	} else {
		format!("{}+g{}", repo.tag, repo.commit)
	};

	Ok(Version::parse(tag.trim_start_matches('v'))?)
}

// =============================================================================
fn main() {
	let version = match std::env::var("XFCAT_VERSION") {
		Ok(v) => v.trim_start_matches('v').to_owned(),
		Err(_) => match get_version() {
			Ok(v) => v.to_string(),
			Err(e) => {
				println!("cargo::error={e}");
				return;
			}
		},
	};
	println!("cargo::rustc-env=CARGO_PKG_VERSION={version}");
	println!("cargo::rerun-if-env-changed=XFCAT_VERSION");
	println!("cargo::rerun-if-changed=build.rs");
}
