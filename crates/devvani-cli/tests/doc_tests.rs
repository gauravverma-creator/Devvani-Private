use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

const BINARY: &str = "devvani-cli";

fn build_binary() -> Command {
    Command::cargo_bin(BINARY).expect("failed to find devvani-cli binary")
}

#[test]
fn test_doc_command_success() {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let src_path = tmp_dir.path().join("test_docs.dvn");
    let cargo_project = tmp_dir.path().join("cargo_project");

    fs::write(
        &src_path,
        "bhashya \"A simple math library\"।\n\
         vritti \"Adds two numbers together\"।\n\
         tippani \"the first number\" para x ।\n\
         tippani \"the second number\" para y ।\n\
         dhātu add x karoti । x yoga 1 iti ।\n",
    )
    .expect("failed to write devvani source");

    let mut cmd = build_binary();
    cmd.arg("doc")
        .arg(&src_path)
        .arg("--output")
        .arg(&cargo_project);

    let output = cmd.output().expect("failed to run devvani doc");
    assert!(output.status.success(), "devvani doc failed: stderr={}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("target/doc/test_docs/index.html"), "stdout did not contain expected path: {}", stdout);
    let index_html = cargo_project
        .join("target")
        .join("doc")
        .join("test_docs")
        .join("index.html");
    assert!(index_html.exists(), "expected doc index.html at {}", index_html.display());
}

#[test]
fn test_doc_command_compile_error() {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let src_path = tmp_dir.path().join("type_error.dvn");

    fs::write(
        &src_path,
        "dhātu bad n karoti । n yoga \"string\" iti ।\n",
    )
    .expect("failed to write devvani source");

    let mut cmd = build_binary();
    let output = cmd.arg("doc").arg(&src_path).output().expect("failed to run devvani doc");

    assert!(!output.status.success(), "expected non-zero exit code for compile error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Compilation error"), "expected diagnostic in stderr:\n{}", stderr);
}

#[test]
fn test_doc_command_content_verification() {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let src_path = tmp_dir.path().join("content_check.dvn");
    let cargo_project = tmp_dir.path().join("cargo_project");

    fs::write(
        &src_path,
        "bhashya \"File level documentation\"।\n\
         vritti \"Item level documentation\"।\n\
         dhātu foo karoti । 1 iti ।\n",
    )
    .expect("failed to write devvani source");

    let mut cmd = build_binary();
    cmd.arg("doc")
        .arg(&src_path)
        .arg("--output")
        .arg(&cargo_project);

    let output = cmd.output().expect("failed to run devvani doc");
    assert!(output.status.success(), "devvani doc failed: stderr={}", String::from_utf8_lossy(&output.stderr));

    let lib_rs = cargo_project.join("src").join("lib.rs");
    assert!(lib_rs.exists(), "expected lib.rs at {}", lib_rs.display());
    let lib_contents = fs::read_to_string(&lib_rs).expect("failed to read lib.rs");
    assert!(
        lib_contents.contains("//! File level documentation"),
        "expected file-level doc comment in lib.rs:\n{}",
        lib_contents
    );
    assert!(
        lib_contents.contains("/// Item level documentation"),
        "expected item-level doc comment in lib.rs:\n{}",
        lib_contents
    );

    let index_html = cargo_project
        .join("target")
        .join("doc")
        .join("content_check")
        .join("index.html");
    assert!(index_html.exists(), "expected doc index.html at {}", index_html.display());
    let html_contents = fs::read_to_string(&index_html).expect("failed to read index.html");
    assert!(
        html_contents.contains("File level documentation"),
        "expected file-level doc in HTML:\n{}",
        html_contents
    );
}
