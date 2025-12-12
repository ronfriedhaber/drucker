# Drucker

`drucker` is a tiny Rust helper crate that builds safe command lines for
[`lp`](https://www.cups.org/doc/options.html) and `lpr`, letting you send text or
files to a CUPS compatible printer without hand-written shell escaping.

## Features

* Shell-safe command construction for `lp` (default) or `lpr`.
* Simple `DruckerOptions` builder for destination, copies, title, and arbitrary
  `-o key=value` job options.
* Print raw text (written to a temp file) or an existing file path.

## Supported Platforms

`drucker` issues the same commands you would type into a terminal, so it works on
any platform that provides the `lp`/`lpr` utilities from CUPS:

* macOS (tested)
* Linux (expected to work, contributions welcome)

## Installation

Add `drucker` to your `Cargo.toml`:

```toml
[dependencies]
drucker = "0.1"
```

## Quick start

Print a text receipt to your default printer with the safe `lp` defaults:

```rust
use drucker::lp::{Drucker, DruckerContent, DruckerOptions};

fn main() -> Result<(), ()> {
    let receipt = "Order #123\nTotal: $42.00\n";

    let drucker = Drucker::new(DruckerOptions::default());

    drucker.print(DruckerContent::Text(receipt.into()))
}
```

Switch to `lpr`, target a specific printer, and print an existing PDF using the
builder API:

```rust
use std::path::PathBuf;

use drucker::lp::{Drucker, DruckerContent, DruckerOptions};

fn print_pdf() -> Result<(), ()> {
    let options = DruckerOptions::builder()
        .use_lpr(true)
        .destination("Office-Color")
        .copies(2)
        .title("Quarterly Summary")
        .job_option("sides", "two-sided-long-edge")
        .build();

    let drucker = Drucker::new(options);

    drucker.print(DruckerContent::File(PathBuf::from("reports/q2.pdf")))
}

Keep the same `drucker` handle around if you want to send multiple jobs with
identical options:

```rust
fn batch() -> Result<(), ()> {
    let drucker = Drucker::new(DruckerOptions::default());
    drucker.print(DruckerContent::Text("first".into()))?;
    drucker.print(DruckerContent::Text("second".into()))?;
    Ok(())
}
```
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
