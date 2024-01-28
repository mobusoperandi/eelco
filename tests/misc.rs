mod util;

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
