//! Integration tests for --show-streams (NTFS Alternate Data Streams).
//!
//! All tests are `#[cfg(windows)]` — ADS are an NTFS-only concept.
//! On CI without NTFS (Linux containers) these tests are silently skipped.

#[cfg(windows)]
mod windows_ads {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper: create a file with one or more ADS.
    fn create_file_with_ads(
        dir: &std::path::Path,
        name: &str,
        streams: &[(&str, &[u8])],
    ) -> std::path::PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, b"main content").unwrap();

        for (stream_name, data) in streams {
            let ads_path = format!("{}:{}", file_path.display(), stream_name);
            let mut f = fs::File::create(&ads_path)
                .unwrap_or_else(|e| panic!("cannot create ADS '{}': {}", ads_path, e));
            f.write_all(data).unwrap();
            drop(f);

            let read_back = fs::read(&ads_path)
                .unwrap_or_else(|e| panic!("cannot read back ADS '{}': {}", ads_path, e));
            assert_eq!(read_back, *data, "ADS content mismatch for {}", ads_path);
        }

        file_path
    }

    #[test]
    fn show_streams_lists_ads() {
        let dir = TempDir::new().unwrap();
        create_file_with_ads(
            dir.path(),
            "report.docx",
            &[("Zone.Identifier", b"[ZoneTransfer]\r\nZoneId=3\r\n")],
        );

        Command::cargo_bin("rt")
            .unwrap()
            .args(["--show-streams", "--no-icons"])
            .arg(dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(":Zone.Identifier"));
    }

    #[test]
    fn show_streams_multiple() {
        let dir = TempDir::new().unwrap();
        create_file_with_ads(
            dir.path(),
            "multi.txt",
            &[("stream_a", b"aaa"), ("stream_b", b"bbb")],
        );

        let output = Command::cargo_bin("rt")
            .unwrap()
            .args(["--show-streams", "--no-icons"])
            .arg(dir.path())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(":stream_a"),
            "should list stream_a:\n{}",
            stdout
        );
        assert!(
            stdout.contains(":stream_b"),
            "should list stream_b:\n{}",
            stdout
        );
    }

    #[test]
    fn without_flag_ads_are_hidden() {
        let dir = TempDir::new().unwrap();
        create_file_with_ads(dir.path(), "secret.txt", &[("hidden", b"data")]);

        Command::cargo_bin("rt")
            .unwrap()
            .arg("--no-icons")
            .arg(dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(":hidden").not());
    }

    #[test]
    fn show_streams_with_size() {
        let dir = TempDir::new().unwrap();
        create_file_with_ads(dir.path(), "sized.txt", &[("info", b"0123456789")]);

        let output = Command::cargo_bin("rt")
            .unwrap()
            .args(["--show-streams", "--size", "--no-icons"])
            .arg(dir.path())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(":info"), "ADS name missing:\n{}", stdout);
        assert!(stdout.contains("10"), "ADS size missing:\n{}", stdout);
    }

    #[test]
    fn show_streams_no_ads_clean_output() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("clean.txt"), b"no streams here").unwrap();

        Command::cargo_bin("rt")
            .unwrap()
            .args(["--show-streams", "--no-icons"])
            .arg(dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("clean.txt"));
        // No crash, no spurious output
    }

    #[test]
    fn show_streams_parallel_mode() {
        let dir = TempDir::new().unwrap();
        create_file_with_ads(dir.path(), "par.txt", &[("meta", b"parallel")]);

        Command::cargo_bin("rt")
            .unwrap()
            .args(["--show-streams", "--parallel", "--no-icons"])
            .arg(dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains(":meta"));
    }

    #[test]
    fn show_streams_with_full_path() {
        let dir = TempDir::new().unwrap();
        let file = create_file_with_ads(dir.path(), "fp.txt", &[("data", b"x")]);

        let output = Command::cargo_bin("rt")
            .unwrap()
            .args(["--show-streams", "--full-path", "--no-icons"])
            .arg(dir.path())
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let expected = format!("{}:data", file.display());
        assert!(
            stdout.contains(&expected),
            "full path ADS expected '{}', got:\n{}",
            expected,
            stdout
        );
    }
}
