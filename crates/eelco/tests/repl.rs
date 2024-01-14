use assert_fs::prelude::FileWriteStr;
use indoc::{formatdoc, indoc};
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

        eelco
            .assert()
            .failure()
            .stderr("Error: missing prompt LFLine(\"nix-shnepl> nope\\n\")\n");
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

        eelco.assert().failure().stderr(formatdoc! {r#"
            Error: {file_path}:1
            actual (sanitized): 2
            expected          : 3
        "#});
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
