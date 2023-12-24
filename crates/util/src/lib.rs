use assert_cmd::Command;
use assert_fs::TempDir;

pub fn test_eelco(pattern: &str) -> (TempDir, Command) {
    let dir = TempDir::new().unwrap();
    let mut command = Command::cargo_bin("eelco").unwrap();
    let path = dir.path().to_str().unwrap();
    command
        .current_dir(&dir)
        .args(["nix", &format!("{path}/{pattern}")]);
    (dir, command)
}
