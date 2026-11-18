use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const REFERENCE_PROJECT: &str = "tests/fixtures/orchestration/project";
fn workspace_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(relative)
}

/// GitHub's Windows runners export `%TEMP%` spelled with the 8.3 alias of
/// the profile directory; the selection policy denies alias spellings.
fn alias_free_path(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().expect("fixture path must exist");
    #[cfg(windows)]
    match canonical.to_string_lossy().strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => canonical,
    }
    #[cfg(not(windows))]
    canonical
}

fn lekalo_with(dir: &Path, head: &[&str], adapter_args: &[&str]) -> Output {
    let mut args: Vec<&str> = head.to_vec();
    args.push("--");
    args.extend_from_slice(adapter_args);
    Command::new(env!("CARGO_BIN_EXE_lekalo"))
        .args(&args)
        .current_dir(alias_free_path(dir))
        .output()
        .expect("run the real lekalo binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf8")
}

fn exit_code(output: &Output) -> u8 {
    output.status.code().expect("exit code") as u8
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf8")
}

fn copy_dir(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).expect("create target dir");
    for entry in std::fs::read_dir(source).expect("read source") {
        let entry = entry.expect("read entry");
        let target_path = target.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target_path);
        } else {
            std::fs::copy(entry.path(), target_path).expect("copy file");
        }
    }
}

static NEXT_ROOT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Run a body against a fresh copy of the orchestration fixture project,
/// with the child working directory set to the copy so the adapter
/// vector resolves project-relative, exactly like an operator run.
fn with_project(body: impl FnOnce(&Path)) {
    let id = NEXT_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        + u64::from(std::process::id());
    let root = workspace_path("target")
        .join("orchestration-cli-tests")
        .join(format!("case-{id}"));
    copy_dir(&workspace_path(REFERENCE_PROJECT), &root);
    body(&root);
    let _ = std::fs::remove_dir_all(&root);
}

/// Issue #58: a declared `lekalo/types` input flips the PHP adapter's
/// generation inside the REAL pipeline — the staged evidence path the
/// core names resolves to the types exchange, the apply publishes the
/// whole inventory, and the mapping sidecar (a `.map.json` document
/// without source-map declarations) rides the apply without poisoning
/// the ownership-manifest ingestion. Skips with the platform's honest
/// reason wherever the confined PHP runtime cannot run.
#[test]
fn a_php_types_generation_applies_through_the_pipeline_and_records_the_sidecar() {
    use lekalo_core::target_protocol::{transport::TransportLimits, TargetClient};

    // The confined probe: one real describe exchange through the
    // production client, exactly the dependency every later step has.
    static RUNNABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let runnable = *RUNNABLE.get_or_init(|| {
        let probe_root = std::env::temp_dir().join(format!(
            "lekalo-orchestrate-php-probe-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&probe_root);
        let command = lekalo_core::target_protocol::transport::AdapterCommand {
            program: std::path::PathBuf::from("php"),
            args: vec![workspace_path("adapters/php-laravel/adapter.php")
                .to_string_lossy()
                .to_string()],
        };
        let mut client = TargetClient::new(TransportLimits {
            timeout_ms: 20_000,
            ..TransportLimits::default()
        });
        let outcome = client.describe(&command, &probe_root);
        let _ = std::fs::remove_dir_all(&probe_root);
        matches!(outcome, Ok(described) if described.capabilities.adapter.id == "lekalo-target-php-laravel")
    });
    if !runnable {
        eprintln!(
            "skip: the confined PHP exchange cannot run on this platform; \
             the pipeline types apply proved nothing here"
        );
        return;
    }

    with_project(|root| {
        // The shipped adapter travels inside the project copy: the
        // supply binds project-relative entries only.
        let adapter_home = root.join("adapters").join("php-laravel");
        std::fs::create_dir_all(&adapter_home).expect("adapter home");
        std::fs::copy(
            workspace_path("adapters/php-laravel/adapter.php"),
            adapter_home.join("adapter.php"),
        )
        .expect("adapter copy");
        let php_adapter: &[&str] = &["php", "adapters/php-laravel/adapter.php"];

        // Lock pins the PHP supply.
        let lock = lekalo_with(root, &["--json", "lock"], php_adapter);
        assert_eq!(exit_code(&lock), 0, "stdout={}", stdout(&lock));

        // The first dry run plans the kernel artifact and persists the
        // canonical IR evidence the input document must bind.
        let first = lekalo_with(
            root,
            &["--json", "generate", "--target", "php-laravel", "--dry-run"],
            php_adapter,
        );
        assert_eq!(exit_code(&first), 0, "stdout={}", stdout(&first));
        let evidence = std::fs::read(
            root.join(".lekalo")
                .join("cache")
                .join("ir")
                .join("planner.json"),
        )
        .expect("the staged evidence exists after a generate run");
        let digest = format!("sha256:{}", lekalo_core::digest::sha256_hex(&evidence));

        // The declared types input flips the next generation.
        std::fs::create_dir_all(root.join("lekalo").join("types")).expect("types home");
        std::fs::write(
            root.join("lekalo").join("types").join("planner.types.json"),
            format!(
                concat!(
                    "{{\"identity\":\"dev.lekalo.php-types-input@0.4.0\",",
                    "\"irDigest\":\"{digest}\",\"projectId\":\"planner\",",
                    "\"schemaVersion\":\"lekalo/php-types-input/v0.4.0\"}}\n"
                ),
                digest = digest
            ),
        )
        .expect("types input written");
        let dry = lekalo_with(
            root,
            &["--json", "generate", "--target", "php-laravel", "--dry-run"],
            php_adapter,
        );
        assert_eq!(
            exit_code(&dry),
            0,
            "stdout={} stderr={}",
            stdout(&dry),
            String::from_utf8_lossy(&dry.stderr)
        );
        let receipt: serde_json::Value = serde_json::from_str(&stdout(&dry)).expect("json");
        let writes = receipt["targets"][0]["writes"]
            .as_array()
            .expect("dry run plans writes")
            .clone();
        assert_eq!(writes.len(), 25, "the closed types inventory is planned");
        assert!(
            writes
                .iter()
                .any(|w| w["path"] == ".lekalo/generated/php-laravel/types/types.map.json"),
            "the mapping sidecar is part of the plan"
        );

        // The apply publishes the inventory; the sidecar must not poison
        // the ownership-manifest ingestion, so the run succeeds and the
        // manifest records the applied files.
        let apply = lekalo_with(
            root,
            &["--json", "generate", "--target", "php-laravel"],
            php_adapter,
        );
        assert_eq!(exit_code(&apply), 0, "stdout={}", stdout(&apply));
        let applied: serde_json::Value = serde_json::from_str(&stdout(&apply)).expect("json");
        assert_eq!(
            applied["verdict"],
            "ready",
            "the types apply succeeds end to end: {}",
            stdout(&apply)
        );
        assert_eq!(
            applied["targets"][0]["state"],
            "applied",
            "the php-laravel target applies: {}",
            stdout(&apply)
        );
        let sidecar = root
            .join(".lekalo")
            .join("generated")
            .join("php-laravel")
            .join("types")
            .join("types.map.json");
        let sidecar_bytes = std::fs::read(&sidecar).expect("the applied sidecar exists");
        let sidecar_document =
            serde_json::from_slice::<serde_json::Value>(&sidecar_bytes).expect("sidecar json");
        assert!(
            !sidecar_document["types"]
                .as_array()
                .expect("types entries")
                .is_empty(),
            "the sidecar is the mapping document, not a source map"
        );
        assert!(
            root.join(".lekalo")
                .join("generated")
                .join("manifests")
                .join("ownership.json")
                .exists(),
            "the ownership manifest landed"
        );
    });
}
