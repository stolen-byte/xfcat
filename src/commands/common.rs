// SPDX-License-Identifier: GPL-3.0-or-later

// =============================================================================
#[derive(clap::Args)]
pub struct PathArgs {
	/// glob pattern to match file paths against.
	///
	///     ?  Matches any single character
	///     *  Matches zero or more characters, except for path separators
	///    **  Matches zero or more characters, including path separators
	/// [...]  Matches any character inside the brackets, supports !/^ negation.
	/// [a-b]  Matches any character in range a - b, supports !/^ negation.
	/// {a,b}  Matches one of the patterns a or b, supports nesting.
	///     !  Negates result of the match.
	#[arg(short, long, verbatim_doc_comment, name = "PATTERN")]
	filter: Option<String>,
}

impl PathArgs {
	pub fn is_filtered(&self, path: &str) -> bool {
		self.filter
			.as_ref()
			.is_some_and(|f| !fast_glob::glob_match(f, path))
	}
}
