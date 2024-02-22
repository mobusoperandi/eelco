use assert_fs::fixture::FileWriteStr;
use indoc::indoc;
use util::with_eelco;

mod util;

#[test]
fn pass() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {r#"
                ```nix
                # path: b.nix
                let
                  pkgs = fet;
                in
                  pkgs.writeShellScript "script" 'echo "Hello, world!"'
                ```

                ```nix
                # path: a.nix
                import ./bar.nix
                ```

                ```bash
                nix-channel 
                nix-build ./a.nix
                [[ $(./result) == "Hello, world!" ]]
                ```
            "#})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco
            .assert()
            .success()
            .stderr(format!("PASS: {file_path}:11\n"));
    });
}

#[test]
fn fail() {
    with_eelco(|file, eelco| {
        file.write_str(indoc! {r#"
                ```nix
                # path: a.nix

                ```

                ```bash
                nix-build ./a.nix
                [[ $(./result) == "Hello, world!" ]]
                ```
            "#})
            .unwrap();

        let file_path = file.path().to_str().unwrap();

        eelco
            .assert()
            .success()
            .stderr(format!("PASS: {file_path}:11\n"));
    });
}
