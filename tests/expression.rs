mod util;

use assert_fs::fixture::FileWriteStr;
use indoc::{formatdoc, indoc};
use predicates::{
    prelude::PredicateBooleanExt,
    str::{contains, starts_with},
};
use util::with_eelco;

#[test]
fn assertion_fail() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
            ```nix
            assert false; null
            ```
        "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().failure().stderr(
            starts_with(format!("Error: {file_path}:1\n")).and(contains("assert false; null")),
        );
    });
}

#[test]
fn fail_non_null() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix
                0
                ```
            "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().failure().stderr(formatdoc! {"
            Error: {file_path}:1
            evaluated into non-null
            note: examples must evaluate into null
            value: 0
        "});
    });
}

#[test]
fn pass() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix
                null
                ```
            "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco
            .assert()
            .success()
            .stderr(format!("PASS: {file_path}:1\n"));
    });
}

#[test]
fn io_error() {
    with_eelco(|file, _eelco| {
        file.write_str(indoc! {"
                ```nix
                null
                ```
            "})
            .unwrap();

        let mut eelco = assert_cmd::Command::cargo_bin("eelco").unwrap();

        eelco.arg("brix");
        eelco.arg(file.as_os_str());

        eelco
            .assert()
            .failure()
            .stderr("Error: No such file or directory (os error 2)\n");
    });
}
