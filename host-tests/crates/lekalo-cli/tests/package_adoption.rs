use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn lekalo_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lekalo"))
        .args(args)
        .current_dir(alias_free_path(dir))
        .env_remove("LEKALO_PROJECT")
        .output()
        .expect("run the real lekalo binary")
}

/// The selection policy denies alias-spelled working directories
/// (`structure.selection-alias`) before any command logic runs; chdir
/// the child into the resolved spelling, stripped of the `\\?\` verbatim
/// prefix `canonicalize` produces on Windows.
fn alias_free_path(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().expect("fixture path must exist");
    #[cfg(windows)]
    match canonical.to_string_lossy().strip_prefix(r"\\?\") {
        // `\\?\C:\...` -> `C:\...`; UNC (`\\?\UNC\...`) stays verbatim.
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => canonical,
    }
    #[cfg(not(windows))]
    canonical
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf8")
}

fn exit_code(output: &Output) -> u8 {
    output.status.code().expect("exit code") as u8
}

fn stdout_json(output: &Output) -> Value {
    serde_json::from_str(stdout_text(output).trim()).expect("stdout json envelope")
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(name)
}

/// Copy a fixture into a fresh alias-free temp directory; adopt runs on
/// the copy so the committed fixtures stay pristine evidence.
fn materialize(name: &str) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("repo");
    copy_tree(&fixture(name), &root);
    (temp, root)
}

/// Recursively copy `from` into `to` (regular files and directories only;
/// the fixtures carry no links).
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create copy root");
    for entry in fs::read_dir(from).expect("read fixture") {
        let entry = entry.expect("fixture entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("entry type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}

/// The Laravel fixture: composer framework hints, the artisan gate
/// proposal, and the composer script workflow gate.
#[test]
fn adopt_detects_the_laravel_layout() {
    let (_temp, root) = materialize("tests/fixtures/adopt/laravel-app");
    let output = lekalo_in(&root, &["--json", "init", "--adopt", "--dry-run"]);
    assert_eq!(exit_code(&output), 0, "{}", stderr_text(&output));
    let envelope = stdout_json(&output);
    assert_eq!(envelope["projectId"], "acme_shop");
    assert_eq!(envelope["projectIdSource"]["source"], "composer.json#name");
    assert_eq!(envelope["projectIdSource"]["original"], "acme/shop");
    let detection = &envelope["detection"];
    let languages = detection["languages"].as_array().expect("languages");
    assert!(languages.iter().any(|hint| hint["label"] == "laravel"
        && hint["kind"] == "framework"
        && hint["confidence"] == "high"));
    let gates = detection["gates"].as_array().expect("gates");
    assert!(gates
        .iter()
        .any(|gate| gate["gate"] == "test" && gate["command"] == "composer test"));
    assert!(gates
        .iter()
        .any(|gate| gate["command"] == "php artisan test" && gate["source"] == "derived"));
    // The write plan is the single project document.
    assert_eq!(envelope["writes"].as_array().expect("writes").len(), 1);
}
