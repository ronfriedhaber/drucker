use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::content::DruckerContent;
use super::options::DruckerOptions;

/// Build a full shell-safe command line string to submit the job.
///
/// For [`DruckerContent::Text`], this writes a temp file and targets that path.
pub(super) fn build_command(
    options: &DruckerOptions,
    content: &DruckerContent,
) -> Result<String, ()> {
    let mut base = if options.use_lpr {
        String::from("lpr")
    } else {
        String::from("lp")
    };

    if let Some(dest) = &options.destination {
        if options.use_lpr {
            base.push_str(&format!(" -P {}", sh_escape(dest)));
        } else {
            base.push_str(&format!(" -d {}", sh_escape(dest)));
        }
    }

    if let Some(n) = options.copies {
        if options.use_lpr {
            base.push_str(&format!(" -#{n}"));
        } else {
            base.push_str(&format!(" -n {n}"));
        }
    }

    if let Some(t) = &options.title {
        if options.use_lpr {
            base.push_str(&format!(" -J {}", sh_escape(t)));
        } else {
            base.push_str(&format!(" -t {}", sh_escape(t)));
        }
    }

    for (k, v) in &options.job_options {
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

/// POSIX-shell escaping using single quotes.
///
/// Replaces `'` with the sequence `"'"'"'` within a surrounding single-quoted string.
pub(super) fn sh_escape(s: &str) -> String {
    if s.is_empty() {
        "''".to_string()
    } else {
        let escaped = s.replace('\'', "'\"'\"'");
        format!("'{}'", escaped)
    }
}

fn sh_escape_path(p: &Path) -> String {
    let s = p.to_string_lossy();
    sh_escape(&s)
}

fn temp_path(prefix: &str, ext: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    let mut pb = std::env::temp_dir();
    pb.push(format!("{prefix}-{ts}.{ext}"));
    pb
}
