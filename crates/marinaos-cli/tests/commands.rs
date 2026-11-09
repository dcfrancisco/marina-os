use std::process::Command;

#[test]
fn commands_return_observed_json() {
    for command in ["version", "status", "doctor", "hardware"] {
        let output = Command::new(env!("CARGO_BIN_EXE_marinaosctl"))
            .args([command, "--json"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{command}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(value.is_object());
        if command == "hardware" {
            assert!(value["memory"]["value"]
                .as_str()
                .unwrap()
                .contains("MemTotal:"));
        }
        if command == "status" {
            assert_eq!(value["daemon"], "not implemented");
        }
    }
}

#[test]
fn unknown_commands_fail() {
    let output = Command::new(env!("CARGO_BIN_EXE_marinaosctl"))
        .arg("invalid")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}
