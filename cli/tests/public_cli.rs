use std::process::{Command, Output};

fn agcu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agcu"))
        .args(args)
        .output()
        .expect("run source-built agcu")
}

#[test]
fn public_help_version_and_grid_options() {
    let version = agcu(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        format!("agcu {}\n", env!("CARGO_PKG_VERSION"))
    );
    let help = agcu(&["--help"]);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("Usage: agcu"));
    assert!(help.contains("https://github.com/elithecho/agcu"));
    let screenshot = agcu(&["screenshot", "--help"]);
    assert!(screenshot.status.success());
    let screenshot = String::from_utf8(screenshot.stdout).unwrap();
    assert!(screenshot.contains("Usage: agcu screenshot"));
    assert!(screenshot.contains("--grid"));
    let invalid = agcu(&["screenshot", "--grid", "--unknown-flag"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8(invalid.stderr)
        .unwrap()
        .contains("Usage: agcu screenshot"));
}

#[test]
fn shell_completions_target_the_public_command() {
    let output = agcu(&["completions", "bash"]);
    assert!(output.status.success());
    let script = String::from_utf8(output.stdout).unwrap();
    assert!(script.contains("_agcu()"));
    assert!(script.contains("complete -F _agcu"));
    assert!(!script.contains("agent-computer-use"));
}

#[test]
fn pid_help_and_invalid_arguments_are_public() {
    let help = agcu(&["snapshot", "--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8(help.stdout).unwrap().contains("--pid"));
    for args in [
        vec!["snapshot", "--pid", "0"],
        vec!["snapshot", "--pid", "2147483648"],
        vec!["click", "--pid", "42", "--app", "Calculator"],
    ] {
        let result = agcu(&args);
        assert_eq!(result.status.code(), Some(2));
    }
}

#[cfg(unix)]
#[test]
fn packaged_artifact_still_advertises_agcu_in_usage() {
    use std::os::unix::process::CommandExt;
    let output = Command::new(env!("CARGO_BIN_EXE_agcu"))
        .arg0("agcu-macos-arm64")
        .args(["screenshot", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Usage: agcu screenshot"));
}

#[test]
fn explicit_project_setup_adds_only_the_agcu_rule_and_is_idempotent() {
    let fixture = std::env::temp_dir().join(format!(
        "agcu-setup-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let settings = fixture.join(".claude/settings.local.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        r#"{"permissions":{"allow":["Read"]},"other":true}"#,
    )
    .unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_agcu"))
            .args(["setup", "--mode", "unsupervised", "--scope", "project"])
            .current_dir(&fixture)
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    let first = std::fs::read(&settings).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&first).unwrap(),
        serde_json::json!({"permissions":{"allow":["Read", "Bash(agcu *)"]}, "other":true})
    );
    assert!(run().status.success());
    assert_eq!(std::fs::read(&settings).unwrap(), first);
    std::fs::remove_dir_all(fixture).unwrap();
}
