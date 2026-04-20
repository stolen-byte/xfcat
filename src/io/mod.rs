// SPDX-License-Identifier: GPL-3.0-or-later

// =============================================================================
pub const DEFAULT_BUFSIZE: usize = 1024 * 32; // 32KB

// =============================================================================
mod copier;
pub use copier::*;

#[cfg(test)]
mod tests;
