use std::process::Command;
use tempfile::TempDir;

fn dagrobin(dir: &TempDir) -> Command {
    let db_path = dir.path().join("test.db");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_dagRobin"));
    cmd.arg("--db").arg(db_path);
    cmd
}

mod metadata_parsing {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_single_metadata() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test", "Test"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "add failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = dagrobin(&dir)
            .args(["update", "test", "--metadata", "notes:hello"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "update failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = dagrobin(&dir).args(["get", "test"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("notes: hello"));
    }

    #[test]
    fn test_multiple_metadata_flags() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test2", "Test2"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test2", "--metadata", "a:1", "--metadata", "b:2"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test2"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("a: 1"));
        assert!(stdout.contains("b: 2"));
    }

    #[test]
    fn test_semicolon_separator_multiple_pairs() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test3", "Test3"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test3", "--metadata", "foo:bar;baz:qux"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test3"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("foo: bar"));
        assert!(stdout.contains("baz: qux"));
    }

    #[test]
    fn test_comma_in_value() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test4", "Test4"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test4", "--metadata", "tags:a,b,c"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test4"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("tags: a,b,c"));
    }

    #[test]
    fn test_mixed_semicolon_and_multiple_flags() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test5", "Test5"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args([
                "update",
                "test5",
                "--metadata",
                "notes:x,y",
                "--metadata",
                "agent:me",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test5"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("notes: x,y"));
        assert!(stdout.contains("agent: me"));
    }

    #[test]
    fn test_empty_value_ignored() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test6", "Test6"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test6", "--metadata", "empty:"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test6"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("empty:"));
    }

    #[test]
    fn test_empty_key_ignored() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test7", "Test7"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test7", "--metadata", ":value"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test7"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("value"));
    }

    #[test]
    fn test_whitespace_trimmed() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test8", "Test8"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test8", "--metadata", "  key :  value  "])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test8"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("key: value"));
    }

    #[test]
    fn test_no_colon_ignored() {
        let dir = TempDir::new().unwrap();
        let output = dagrobin(&dir)
            .args(["add", "test9", "Test9"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir)
            .args(["update", "test9", "--metadata", "invalid-no-colon"])
            .output()
            .unwrap();
        assert!(output.status.success());

        let output = dagrobin(&dir).args(["get", "test9"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("invalid"));
    }
}
