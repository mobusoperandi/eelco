use assert_fs::NamedTempFile;

pub fn with_eelco(f: impl FnOnce(&mut NamedTempFile, &mut assert_cmd::Command)) {
    let mut tmpfile = NamedTempFile::new("we-dont-particularly-mind.md").unwrap();
    let mut command = assert_cmd::Command::cargo_bin("eelco").unwrap();

    command.arg("nix");
    command.arg(tmpfile.as_os_str());
    f(&mut tmpfile, &mut command);
    drop(tmpfile);
}
