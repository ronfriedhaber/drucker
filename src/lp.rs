use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Options for a print job (CUPS `lp`/`lpr`).
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

/// Print content variants: raw text or a file path.
pub enum DruckerContent {
    /// Print a text string (written to a temp file internally).
    Text(String),
    /// Print an existing file at this path.
    File(PathBuf),
}

/// Handle that stores printer options and can submit multiple jobs.
pub struct Drucker {
    /// CUPS/LP options that control the job.
    pub options: DruckerOptions,
}

impl Drucker {
    /// Create a new [`Drucker`] configured with the provided options.
    pub fn new(options: DruckerOptions) -> Self {
        Self { options }
    }

    /// Execute a print job using `lp` (or `lpr` if configured).
    ///
    /// Returns `Ok(())` on success (exit code 0), otherwise `Err(())`.
    pub fn print(&self, content: DruckerContent) -> Result<(), ()> {
        let cmd = self.build_command(&content)?;
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|_| ())?;
        if status.success() { Ok(()) } else { Err(()) }
    }

    /// Build a full shell-safe command line string to submit the job.
    ///
    /// For [`DruckerContent::Text`], this writes a temp file and targets that path.
    fn build_command(&self, content: &DruckerContent) -> Result<String, ()> {
        let mut base = if self.options.use_lpr {
            String::from("lpr")
        } else {
            String::from("lp")
        };

        if let Some(dest) = &self.options.destination {
            if self.options.use_lpr {
                base.push_str(&format!(" -P {}", sh_escape(dest)));
            } else {
                base.push_str(&format!(" -d {}", sh_escape(dest)));
            }
        }

        if let Some(n) = self.options.copies {
            if self.options.use_lpr {
                base.push_str(&format!(" -#{n}"));
            } else {
                base.push_str(&format!(" -n {n}"));
            }
        }

        if let Some(t) = &self.options.title {
            if self.options.use_lpr {
                base.push_str(&format!(" -J {}", sh_escape(t)));
            } else {
                base.push_str(&format!(" -t {}", sh_escape(t)));
            }
        }

        // for (k, v) in &self.options.job_options {
        //     if self.options.use_lpr {
        //         base.push_str(&format!(" -o {}={}", sh_escape(k), sh_escape(v)));
        //     } else {
        //         base.push_str(&format!(" -o {}={}", sh_escape(k), sh_escape(v)));
        //     }
        // }
        // -o options (CUPS). Keep them unquoted so they render as `-o key=value`.
        for (k, v) in &self.options.job_options {
            base.push_str(" -o ");
            base.push_str(k);
            base.push('=');
            base.push_str(v);
        }

        let file_arg = match content {
            DruckerContent::File(p) => {
                if p.as_os_str().is_empty() {
                    return Err(());
                }
                sh_escape_path(p)
            }
            DruckerContent::Text(s) => {
                let path = temp_path("drucker", "txt");
                let mut f = fs::File::create(&path).map_err(|_| ())?;
                f.write_all(s.as_bytes()).map_err(|_| ())?;
                sh_escape_path(&path)
            }
        };

        Ok(format!("{base} {file_arg}"))
    }
}

/// POSIX-shell escaping using single quotes.
///
/// Replaces `'` with the sequence `'"'"'` within a surrounding single-quoted string.
fn sh_escape(s: &str) -> String {
    if s.is_empty() {
        "''".to_string()
    } else {
        let escaped = s.replace('\'', "'\"'\"'");
        format!("'{}'", escaped)
    }
}

/// Escape a filesystem path for POSIX shells via [`sh_escape`].
fn sh_escape_path(p: &Path) -> String {
    let s = p.to_string_lossy();
    sh_escape(&s)
}

/// Build a unique temporary path in the system temp directory.
fn temp_path(prefix: &str, ext: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    let mut pb = std::env::temp_dir();
    pb.push(format!("{prefix}-{ts}.{ext}"));
    pb
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use std::{env, fs};

    // ---------- helpers ----------

    /// Best-effort "is this command available".
    fn has_cmd(cmd: &str) -> bool {
        Command::new(cmd)
            .arg("--help")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
    }

    /// Which printer to use for integration tests.
    /// Reads DRUCKER_PRINTER, then PRINTER, else None (let CUPS default kick in).
    fn integration_printer() -> Option<String> {
        env::var("DRUCKER_PRINTER")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| env::var("PRINTER").ok())
    }

    /// Create a temp file with `contents` and extension `ext`.
    fn make_temp_file_with(contents: &str, ext: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros())
            .unwrap_or(0);
        p.push(format!("drucker-test-{ts}.{ext}"));
        let mut f = fs::File::create(&p).expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        p
    }

    /// Extract the final (quoted) path argument from a built command,
    /// approximate shell dequote for POSIX single-quote strategy, and return as `String`.
    fn extract_last_path_from_cmd(cmd: &str) -> Option<String> {
        let last_space = cmd.rfind(' ')?;
        let token = cmd[(last_space + 1)..].trim();

        if !(token.starts_with('\'') && token.ends_with('\'')) {
            return None;
        }
        let inner = &token[1..token.len() - 1];
        let dequoted = inner.replace("'\"'\"'", "'");
        Some(dequoted)
    }

    // ---------- your existing unit tests (unchanged except println! for visibility) ----------

    #[test]
    fn build_command_lp_with_file() {
        let temp_pdf = make_temp_file_with("%PDF-1.4\n", "pdf");

        let drucker = Drucker::new(DruckerOptions::default());
        let content = DruckerContent::File(temp_pdf.clone());

        let cmd = drucker.build_command(&content).expect("build ok");
        println!("lp file cmd: {cmd}");
        assert!(cmd.starts_with("lp "), "expected lp command, got: {cmd}");

        let path = extract_last_path_from_cmd(&cmd).expect("extract path");
        assert_eq!(Path::new(&path), temp_pdf.as_path());
    }

    #[test]
    fn build_command_lpr_with_options_and_escaping() {
        let temp_txt = make_temp_file_with("hello", "txt");

        let opts = DruckerOptions::builder()
            .use_lpr(true)
            .destination("Office Printer")
            .copies(2)
            .title("Q2 'Report'")
            .job_option("sides", "two-sided-long-edge")
            .build();

        let drucker = Drucker::new(opts);
        let content = DruckerContent::File(temp_txt.clone());

        let cmd = drucker.build_command(&content).expect("build ok");
        println!("lpr file cmd: {cmd}");

        assert!(cmd.starts_with("lpr "), "expected lpr: {cmd}");
        assert!(
            cmd.contains("-P 'Office Printer'"),
            "destination not properly escaped: {cmd}"
        );
        assert!(cmd.contains("-#2"), "copies flag missing: {cmd}");
        assert!(
            cmd.contains("-J 'Q2 '\"'\"'Report'\"'\"''")
                || cmd.contains("-J 'Q2 '\\''Report'\\'''"),
            "title escaping not present as POSIX-safe single-quote escape: {cmd}"
        );
        assert!(
            cmd.contains("-o sides=two-sided-long-edge"),
            "-o option missing: {cmd}"
        );

        let path = extract_last_path_from_cmd(&cmd).expect("extract path");
        assert_eq!(Path::new(&path), temp_txt.as_path());
    }

    #[test]
    fn build_command_with_text_creates_tempfile_and_writes_contents() {
        let text = "Hello 'quoted'\nLine 2";

        let drucker = Drucker::new(DruckerOptions::default());
        let content = DruckerContent::Text(text.to_string());

        let cmd = drucker.build_command(&content).expect("build ok");
        println!("lp text cmd: {cmd}");
        assert!(cmd.starts_with("lp "), "expected lp by default: {cmd}");

        let path = extract_last_path_from_cmd(&cmd).expect("extract path");
        let p = PathBuf::from(&path);

        assert!(p.exists(), "temp file does not exist: {p:?}");
        let body = fs::read_to_string(&p).expect("read temp file");
        assert_eq!(body, text);

        let _ = fs::remove_file(&p);
    }

    #[test]
    fn options_builder_sets_expected_fields() {
        let opts = DruckerOptions::builder()
            .destination("Floor1")
            .copies(3)
            .title("Quarterly Report")
            .job_option("media", "na_letter_8.5x11in")
            .use_lpr(true)
            .build();

        assert_eq!(opts.destination.as_deref(), Some("Floor1"));
        assert_eq!(opts.copies, Some(3));
        assert_eq!(opts.title.as_deref(), Some("Quarterly Report"));
        assert_eq!(
            opts.job_options.get("media"),
            Some(&"na_letter_8.5x11in".to_string())
        );
        assert!(opts.use_lpr);
    }

    #[test]
    fn sh_escape_roundtrip_examples() {
        let s = "A B ' C";
        let esc = super::sh_escape(s);
        println!("escaped: {esc}");
        let dequoted = {
            assert!(esc.starts_with('\'') && esc.ends_with('\''));
            let inner = &esc[1..esc.len() - 1];
            inner.replace("'\"'\"'", "'")
        };
        assert_eq!(dequoted, s);

        let esc_empty = super::sh_escape("");
        println!("escaped empty: {esc_empty}");
        assert_eq!(esc_empty, "''");
    }

    // ---------- opt-in integration tests that actually print ----------

    /// Actually prints a tiny text file via `lp`.
    ///
    /// Run with:
    /// `DRUCKER_PRINTER=YourPrinter cargo test -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn integration_print_via_lp() {
        if !has_cmd("lp") {
            eprintln!("skipping: `lp` not available on PATH");
            return;
        }

        let opts = DruckerOptions::builder()
            .destination_if(integration_printer())
            .build();

        let txt = format!(
            "Drucker integration test (lp)\nEpoch us: {}\n",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
        );

        let drucker = Drucker::new(opts);

        println!(
            "about to print via `lp`… opts: dest={:?}",
            drucker.options.destination
        );
        drucker
            .print(DruckerContent::Text(txt))
            .expect("lp print should succeed");
        println!("lp print dispatched OK");
    }

    /// Actually prints a tiny text file via `lpr` with some flags.
    ///
    /// Run with:
    /// `DRUCKER_PRINTER=YourPrinter cargo test -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn integration_print_via_lpr() {
        if !has_cmd("lpr") {
            eprintln!("skipping: `lpr` not available on PATH");
            return;
        }

        let opts = DruckerOptions::builder()
            .use_lpr(true)
            .destination_if(integration_printer())
            .copies(1)
            .title("Drucker Integration")
            .job_option("media", "na_letter_8.5x11in")
            .build();

        let path = make_temp_file_with("Hello from lpr integration\n", "txt");

        let drucker = Drucker::new(opts);

        println!(
            "about to print via `lpr`… dest={:?} file={:?}",
            drucker.options.destination, path
        );
        drucker
            .print(DruckerContent::File(path.clone()))
            .expect("lpr print should succeed");
        println!("lpr print dispatched OK");
    }

    /// End-to-end: build -> print via `lp` with an existing file; verifies the file still exists after submit.
    #[test]
    #[ignore]
    fn integration_print_file_roundtrip_lp() {
        if !has_cmd("lp") {
            eprintln!("skipping: `lp` not available on PATH");
            return;
        }

        let path = make_temp_file_with("Roundtrip file body\n", "txt");
        let opts = DruckerOptions::builder()
            .destination_if(integration_printer())
            .build();

        let drucker = Drucker::new(opts);

        let content = DruckerContent::File(path.clone());
        let cmd = drucker.build_command(&content).expect("build ok");
        println!("lp actual submit cmd: {cmd}");
        drucker
            .print(DruckerContent::File(path.clone()))
            .expect("lp submit ok");

        assert!(path.exists(), "source file should remain after submission");
        let _ = fs::remove_file(path);
    }
}
