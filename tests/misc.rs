mod util;

use assert_fs::fixture::FileWriteStr;
use indoc::indoc;
use predicates::boolean::PredicateBooleanExt;
use util::with_eelco;

#[test]
fn empty_file() {
    with_eelco(|_file, eelco| {
        eelco
            .assert()
            .failure()
            .stderr("Error: could not find any examples\n");
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

                ```nix
                null
                ```

                ```nix-repl
                nix-repl> 1 + 2
                3
                ```
            "})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco.assert().success().stderr(
            predicates::str::contains(format!("PASS: {file_path}:1"))
                .and(predicates::str::contains(format!("PASS: {file_path}:10"))),
        );
    });
}
