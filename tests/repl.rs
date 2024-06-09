mod util;

use assert_fs::prelude::FileWriteStr;
use indoc::{formatdoc, indoc};
use predicates::boolean::PredicateBooleanExt;
use util::with_eelco;

#[test]
fn fails_to_parse() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix-repl
                nix-shnepl> nope
                dope
                ```
            "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().failure().stderr(
            predicates::str::starts_with(format!("Error: {file_path}:1"))
                .and(predicates::str::contains("prompt")),
        );
    });
}

#[test]
fn result_mismatch() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix-repl
                nix-repl> 1 + 1
                3

                ```
            "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().failure().stderr(formatdoc! {"
            Error: {file_path}:1

            Actual:

            ```
            2
            ```

            Expected:

            ```
            3
            ```
        "});
    });
}

#[test]
fn pass() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix-repl
                nix-repl> 1 + 1
                2

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
fn pass_assignment() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix-repl
                nix-repl> a = 1

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
fn pass_subsequent_query() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix-repl
                nix-repl> a = 1

                nix-repl> a
                1

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
fn multiline_result() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
            ```nix-repl
            nix-repl> { a = 2; b = 3; }
            {
              a = 2;
              b = 3;
            }
            
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
