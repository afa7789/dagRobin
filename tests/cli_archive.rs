use std::process::Command;
use tempfile::TempDir;

fn dagrobin(dir: &TempDir) -> Command {
    let db_path = dir.path().join("test.db");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_dagRobin"));
    cmd.arg("--db").arg(db_path);
    cmd
}

fn run(dir: &TempDir, args: &[&str]) -> String {
    let out = dagrobin(dir).args(args).output().expect("run dagRobin");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn seed(dir: &TempDir) {
    run(dir, &["add", "a", "A"]);
    run(dir, &["add", "b", "B", "--deps", "a"]);
    run(dir, &["add", "c", "C", "--tags", "ui"]);
    run(dir, &["update", "a", "--status", "done"]);
}

#[test]
fn status_counts_active_tasks() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    let out = run(&dir, &["status"]);
    assert!(out.contains("Round:     1/3"), "{}", out);
    assert!(out.contains("All-time:  1/3"), "{}", out);
}

#[test]
fn archive_defaults_to_done_tasks() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    assert!(run(&dir, &["archive"]).contains("Archived 1 tasks"));

    let out = run(&dir, &["status"]);
    assert!(out.contains("Round:     0/2"), "{}", out);
    assert!(out.contains("Archived:  1/1"), "{}", out);
    assert!(out.contains("All-time:  1/3"), "{}", out);
}

#[test]
fn archived_tasks_are_hidden_from_list_and_ready() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    run(&dir, &["archive", "c"]);

    let list = run(&dir, &["list"]);
    assert!(!list.contains("\nc "), "{}", list);
    assert!(run(&dir, &["list", "--include-archived"]).contains("\nc "));

    let ready = run(&dir, &["ready"]);
    assert!(ready.contains("id: b"), "{}", ready);
    assert!(!ready.contains("id: c"), "{}", ready);
}

#[test]
fn archive_undo_restores_tasks() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    run(&dir, &["archive", "--all"]);
    assert!(run(&dir, &["status"]).contains("Round:     0/0"));

    assert!(run(&dir, &["archive", "--undo", "--all"]).contains("Unarchived 3 tasks"));
    assert!(run(&dir, &["status"]).contains("Round:     1/3"));
}

#[test]
fn archive_by_tag() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    assert!(run(&dir, &["archive", "--tags", "ui"]).contains("Archived 1 tasks"));
    assert!(run(&dir, &["status"]).contains("Archived:  0/1"));
}

#[test]
fn clear_requires_confirmation() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    let out = dagrobin(&dir).arg("clear").output().unwrap();
    assert!(!out.status.success());
    assert!(run(&dir, &["status"]).contains("Round:     1/3"));
}

#[test]
fn clear_wipes_everything() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    assert!(run(&dir, &["clear", "--yes"]).contains("Deleted 3 tasks"));
    assert!(run(&dir, &["status"]).contains("All-time:  0/0"));
}

#[test]
fn clear_filtered_by_status_and_archived() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    run(&dir, &["archive"]);
    assert!(run(&dir, &["clear", "--yes", "--archived-only"]).contains("Deleted 1 tasks"));
    assert!(run(&dir, &["status"]).contains("All-time:  0/2"));

    assert!(run(&dir, &["clear", "--yes", "--status", "pending"]).contains("Deleted 2 tasks"));
}

#[test]
fn status_json_format() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    let out = run(&dir, &["status", "--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["active"]["done"], 1);
    assert_eq!(v["active"]["pending"], 2);
}

#[test]
fn progress_alias_works() {
    let dir = TempDir::new().unwrap();
    seed(&dir);
    assert!(run(&dir, &["progress"]).contains("Round:"));
}
