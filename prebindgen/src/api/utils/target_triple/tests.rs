use super::*;

#[test]
fn aarch64_apple_darwin() {
    let tt = TargetTriple::parse("aarch64-apple-darwin").unwrap();
    assert!(
        tt.arch() == Some("aarch64"),
        "Unexpected architecture found {:?}",
        tt.arch()
    );
    assert!(
        tt.vendor() == Some("apple"),
        "Unexpected vendor found {:?}",
        tt.vendor()
    );
    assert!(
        tt.os() == Some("macos"),
        "Unexpected OS found {:?}",
        tt.os()
    );
    assert!(
        tt.env().is_none(),
        "Unexpected environment found {:?}",
        tt.env()
    );
}

#[test]
fn x86_64_unknown_linux() {
    let tt = TargetTriple::parse("x86_64-unknown-linux-gnu").unwrap();
    assert!(
        tt.arch() == Some("x86_64"),
        "Unexpected architecture found {:?}",
        tt.arch()
    );
    assert!(
        tt.vendor() == Some("unknown"),
        "Unexpected vendor found {:?}",
        tt.vendor()
    );
    assert!(
        tt.os() == Some("linux"),
        "Unexpected OS found {:?}",
        tt.os()
    );
    assert!(
        tt.env() == Some("gnu"),
        "Unexpected environment found {:?}",
        tt.env()
    );
}

#[test]
fn armv7_unknown_linux_gnueabihf() {
    let tt = TargetTriple::parse("armv7-unknown-linux-gnueabihf").unwrap();
    assert!(
        tt.arch() == Some("arm"),
        "Unexpected architecture found {:?}",
        tt.arch()
    );
    assert!(
        tt.vendor() == Some("unknown"),
        "Unexpected vendor found {:?}",
        tt.vendor()
    );
    assert!(
        tt.os() == Some("linux"),
        "Unexpected OS found {:?}",
        tt.os()
    );
    assert!(
        tt.env() == Some("gnu"),
        "Unexpected environment found {:?}",
        tt.env()
    );
}
