/// --lang en/ru, склонения, TREE_LANG
mod common;
use common::retree;

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_report_in_english() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    retree()
        .args(["--lang", "en"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("director"))
        .stdout(predicate::str::contains("file"));
}

#[test]
fn test_report_in_russian() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    retree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("каталог"))
        .stdout(predicate::str::contains("файл"));
}

#[test]
fn test_russian_plural_one_file() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::write(p.join("single.txt"), "").unwrap();

    retree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("1 файл"));
}

#[test]
fn test_russian_plural_few_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..3 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
    }

    retree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 файла"));
}

#[test]
fn test_russian_plural_many_files() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    for i in 0..5 {
        fs::write(p.join(format!("f{}.txt", i)), "").unwrap();
    }

    retree()
        .args(["--lang", "ru"])
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("5 файлов"));
}

#[test]
fn test_tree_lang_env_switches_language() {
    let dir = tempdir().unwrap();
    let p = dir.path();

    fs::create_dir(p.join("subdir")).unwrap();
    fs::write(p.join("file.txt"), "").unwrap();

    retree()
        .env("TREE_LANG", "ru")
        .arg(p)
        .assert()
        .success()
        .stdout(predicate::str::contains("каталог"));
}
