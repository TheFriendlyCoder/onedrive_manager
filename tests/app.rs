use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;
const APP_NAME: &str = "onedrive_manager";

#[test]
#[should_panic]
fn test_panic_condition() {
    panic!("Darn it");
}

#[test]
fn init_help() -> TestResult {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(APP_NAME)?
            .arg("init")
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains(
                "Initialize and authenticate the app",
            ));
    }
    Ok(())
}
#[test]
fn usage() -> TestResult {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(APP_NAME)?
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains("USAGE"));
    }
    Ok(())
}

#[test]
fn cli_failure() -> TestResult {
    Command::cargo_bin(APP_NAME)?.assert().failure();
    Ok(())
}
