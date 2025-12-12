use std::collections::BTreeMap;

/// Options for a print job (CUPS `lp`/`lpr`).
#[derive(Debug, Clone)]
pub struct DruckerOptions {
    /// Printer name (passed as `lp -d <printer>` or `lpr -P <printer>`).
    pub destination: Option<String>,
    /// Number of copies (`lp -n <copies>` or `lpr -#<copies>`).
    pub copies: Option<u32>,
    /// Job title (`lp -t <title>` or `lpr -J <title>` when available).
    pub title: Option<String>,
    /// Arbitrary `-o key=value` options (e.g., `sides=two-sided-long-edge`).
    pub job_options: BTreeMap<String, String>,
    /// Use `lpr` instead of `lp`.
    pub use_lpr: bool,
}

impl Default for DruckerOptions {
    fn default() -> Self {
        Self {
            destination: None,
            copies: None,
            title: None,
            job_options: BTreeMap::new(),
            use_lpr: false,
        }
    }
}

impl DruckerOptions {
    /// Start building [`DruckerOptions`] using the builder pattern.
    pub fn builder() -> DruckerOptionsBuilder {
        DruckerOptionsBuilder {
            options: Self::default(),
        }
    }
}

/// Builder for [`DruckerOptions`].
pub struct DruckerOptionsBuilder {
    options: DruckerOptions,
}

impl DruckerOptionsBuilder {
    /// Set the destination printer name.
    pub fn destination<T: Into<String>>(mut self, destination: T) -> Self {
        self.options.destination = Some(destination.into());
        self
    }

    /// Set the destination printer from an optional value.
    pub fn destination_if<T: Into<String>>(mut self, destination: Option<T>) -> Self {
        self.options.destination = destination.map(Into::into);
        self
    }

    /// Clear any previously set destination printer.
    pub fn clear_destination(mut self) -> Self {
        self.options.destination = None;
        self
    }

    /// Set the number of copies to print.
    pub fn copies(mut self, copies: u32) -> Self {
        self.options.copies = Some(copies);
        self
    }

    /// Clear any previously set copies value.
    pub fn clear_copies(mut self) -> Self {
        self.options.copies = None;
        self
    }

    /// Set the job title.
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.options.title = Some(title.into());
        self
    }

    /// Clear any previously set job title.
    pub fn clear_title(mut self) -> Self {
        self.options.title = None;
        self
    }

    /// Replace the entire set of job options.
    pub fn job_options(mut self, job_options: BTreeMap<String, String>) -> Self {
        self.options.job_options = job_options;
        self
    }

    /// Insert or replace a single job option key/value pair.
    pub fn job_option<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.options.job_options.insert(key.into(), value.into());
        self
    }

    /// Configure whether to use `lpr` instead of `lp`.
    pub fn use_lpr(mut self, use_lpr: bool) -> Self {
        self.options.use_lpr = use_lpr;
        self
    }

    /// Finish building and return the [`DruckerOptions`].
    pub fn build(self) -> DruckerOptions {
        self.options
    }
}
