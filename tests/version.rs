use std::process::Command;

#[test]
fn prints_its_name_and_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .output()
        .expect("the binary runs");

    assert!(output.status.success(), "exit status: {}", output.status);
    assert_eq!(
        String::from_utf8(output.stdout).expect("the output is UTF-8"),
        format!("ghg {}\n", env!("CARGO_PKG_VERSION")),
    );
}
