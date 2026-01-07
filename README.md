# Drucker

A Rust Library For Interfacing With Hardware Printers On POSIX Machines.

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
use drucker::{Drucker, DruckerContent, DruckerOptions};

fn main() -> Result<(), ()> {
    let receipt = "Order #123\nTotal: $42.00\n";

    let drucker = Drucker::new(DruckerOptions::default());

    drucker.print(DruckerContent::Text(receipt.into()))
}
```

```rust
use std::path::PathBuf;

use drucker::{Drucker, DruckerContent, DruckerOptions};

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

``
