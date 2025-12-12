use super::command;
use super::{Drucker, DruckerContent, DruckerOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
use std::time::{SystemTime, UNIX_EPOCH};

fn has_cmd(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn integration_printer() -> Option<String> {
    env::var("DRUCKER_PRINTER")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| env::var("PRINTER").ok())
}

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
            || cmd.contains("-J 'Q2 '\''Report'\'''"),
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
    let esc = command::sh_escape(s);
    println!("escaped: {esc}");
    let dequoted = {
        assert!(esc.starts_with('\'') && esc.ends_with('\''));
        let inner = &esc[1..esc.len() - 1];
        inner.replace("'\"'\"'", "'")
    };
    assert_eq!(dequoted, s);

    let esc_empty = command::sh_escape("");
    println!("escaped empty: {esc_empty}");
    assert_eq!(esc_empty, "''");
}

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
