[![Build status](https://github.com/stolen-byte/xfcat/workflows/ci/badge.svg)](https://github.com/stolen-byte/xfcat/actions)
[![Latest](https://img.shields.io/github/v/release/stolen-byte/xfcat)](https://github.com/stolen-byte/xfcat/releases/latest)
![Windows](https://img.shields.io/badge/platform-windows-blue)
![Linux](https://img.shields.io/badge/platform-linux-blue)
![macOS](https://img.shields.io/badge/platform-macos-blue)

# xfcat

**xfcat** is a command-line tool for packing & unpacking [X4: Foundations](https://www.egosoft.com/games/x4/info_en.php) & mod `.cat`/`.dat` files.

---

## Quick Links

- [Installation](#installation)
- [Usage](#usage)
- [Filter Syntax](https://docs.rs/fast-glob/latest/fast_glob/)
- [Building](#building)
- [License](#license)

---

## Installation

Pre-built binaries are available for Linux, Windows, and macOS from the [releases](https://github.com/stolen-byte/xfcat/releases) page, just download, and place wherever you like.

## Usage

Since X4 packages consist of `.cat`/`.dat` file pairs, we will use the term `package` when refering to them as a whole.

xfcat functionality is divided into various 'commands', a brief overview:

> [!NOTE]
> Package paths for both `list`/`unpack` commands can point to _either_ the `.cat` file, or the `.dat` file,
> or the extension can be omitted altogether, however an incorrect file extension will still be seen as an error.

```
Usage: xfcat <COMMAND>

Commands:
  list    list contents specified packages
  unpack  extract contents of specified packages
  pack    pack a directory into a mod package
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

To list the contents of packages, use the `list` command:

```
Usage: xfcat list [OPTIONS] <INPUTS>...

Arguments:
  <INPUTS>...  input files

Options:
  -H, --human-readable    display sizes as K/M/G etc
  -f, --filter <PATTERN>  glob pattern to match file paths against.
  -n, --name              sort alphabetically by name
  -S, --size              sort by file size, largest first
  -t, --time              sort by time, newest first
  -r, --reverse           reverse order while sorting
  -h, --help              Print help (see more with '--help')
```

To extract 1 or more packages to a given location, use the the `unpack` command:

```
Usage: xfcat unpack [OPTIONS] <INPUTS>...

Arguments:
  <INPUTS>...  input files

Options:
  -o, --out <DIR>         output directory [default: ./out]
  -t, --threads <COUNT>   number of threads to use
  -n, --no-verify         skip verification of file hashes
  -u, --use-subdirs       create separate subdirectories for each package
  -f, --filter <PATTERN>  glob pattern to match file paths against.
  -h, --help              Print help (see more with '--help')
```

To pack a given directory into a mod package, use the `pack` command:

```
Usage: xfcat pack [OPTIONS] <DIR>

Arguments:
  <DIR>  source directory

Options:
  -n, --name <NAME>       output name
  -o, --out <OUT>         output directory
  -f, --filter <PATTERN>  glob pattern to match file paths against.
  -h, --help              Print help (see more with '--help')
```

## Building

To build, you will need a working installation of [Rust](https://www.rust-lang.org/) (including the necessary toolchain for your platform), xfcat should compile with 1.94.1 (stable) or newer.

To build, either clone this repo, or grab a source archive from the releases page, and then:

```
$ cargo build --release
$ ./target/release/xfcat --version
xfcat x.y.z
```

### Running tests

Most critical functionality is covered by tests, to run them all, use:

```
$ cargo test --all
```

from the repository root.

---

## License

```
xfcat is distributed completely free of charge: you can redistribute
it and/or modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.
```

To see the full license text, see [LICENSE](LICENSE)
