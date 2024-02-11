mod util;

use assert_fs::fixture::FileWriteStr;
use indoc::{formatdoc, indoc};
use util::with_eelco;

#[test]
fn empty_file() {
    with_eelco(|_file, eelco| {
        eelco
            .assert()
            .failure()
            .stderr("Error: could not find any REPL examples\n");
    });
}

#[test]
fn all_examples_tested() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {"
                ```nix-repl
                nix-repl> 1 + 1

                2
                ```

                ```nix-repl
                nix-repl> 1 + 2

                3
                ```
            "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().success().stderr(formatdoc! {"
            PASS: {file_path}:1
            PASS: {file_path}:7
        "});
    });
}
