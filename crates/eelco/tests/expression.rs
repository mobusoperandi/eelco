use assert_fs::fixture::FileWriteStr;
use indoc::{formatdoc, indoc};
use predicates::{prelude::PredicateBooleanExt, str::{starts_with, contains}};
use util::with_eelco;

#[test]
fn nix_fail() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
            ```nix
            assert false; null
            ```
        "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().failure().stderr(
            starts_with(format!("FAIL: {file_path}:1\n"))
                .and(contains("assert false; null")),
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

        eelco.assert().failure().stderr(formatdoc! {r#"
            FAIL: {file_path}:1
                evaluated into non-null
                note: examples must evaluate into null
                value: 0
        "#});
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
