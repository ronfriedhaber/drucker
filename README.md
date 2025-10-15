# Drucker

`drucker` is a tiny Rust helper crate that builds safe command lines for
[`lp`](https://www.cups.org/doc/options.html) / `lpr` so that you can send text or
files to a CUPS compatible printer without sprinkling shell escaping logic
throughout your application.

## Features

* Shell-safe command construction for `lp` (default) or `lpr`.
* Simple option struct for destination, copies, title, and arbitrary
  `-o key=value` job options.
* Ability to print raw text (written to a temp file) or an existing file.
* Optional integration tests that exercise real printers when the required
  commands are available.

## Supported Platforms

`drucker` issues the same commands you would type into a terminal, so it works on
any platform that provides the `lp`/`lpr` utilities from CUPS:

* macOS (tested)
* Linux (expected to work, contributions welcome)

## Installation

Add `drucker` to your `Cargo.toml`:

```toml
[dependencies]
drucker = { path = "../drucker" }
```

> **Note**: The crate is not yet published on crates.io. Use a path or git
> dependency until it is released.

## Quick start

Print a text receipt to your default printer:

```rust
use drucker::lp::{Drucker, DruckerContent, DruckerOptions};

fn main() -> Result<(), ()> {
    let receipt = "Order #123\nTotal: $42.00\n";

    let job = Drucker {
        options: DruckerOptions::default(),
        content: DruckerContent::Text(receipt.into()),
    };

    job.print()
}
```

Target a specific printer with additional options while printing an existing
file:

```rust
use std::path::PathBuf;

use drucker::lp::{Drucker, DruckerContent, DruckerOptions};

fn print_pdf() -> Result<(), ()> {
    let mut options = DruckerOptions::default();
    options.destination = Some("Office-Color".into());
    options.copies = Some(2);
    options.title = Some("Quarterly Summary".into());
    options.job_options.insert("sides".into(), "two-sided-long-edge".into());

    let job = Drucker {
        options,
        content: DruckerContent::File(PathBuf::from("reports/q2.pdf")),
    };

    job.print()
}
```

Under the hood the crate builds a single POSIX-shell-safe command string and
spawns `sh -c ...`, so *no* `unsafe` blocks or raw command string concatenation
are required in your code.

## Running the tests

Unit tests run purely offline:

```bash
cargo test
```

There are also ignored integration tests that submit real jobs via `lp` and
`lpr`. To opt in, export the printer you want to hit and run tests with
`--ignored`:

```bash
export DRUCKER_PRINTER="YourPrinter"
cargo test -- --ignored --nocapture
```

If `DRUCKER_PRINTER` is not set, the tests fall back to the `PRINTER`
environment variable or let CUPS choose the default destination.

## Contributing

Issues and pull requests are welcome! If you have success (or trouble) on a
platform not listed above, please share your findings so we can document and
improve support.
