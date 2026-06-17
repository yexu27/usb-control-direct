use file_access::autorun::parse_autorun_targets;

#[test]
fn parse_open_command() {
    let content = "[autorun]\r\nopen=setup.exe\r\nicon=icon.ico\r\n";
    let targets = parse_autorun_targets(content);
    assert_eq!(targets, vec!["setup.exe"]);
}

#[test]
fn parse_open_with_arguments() {
    let content = "[autorun]\r\nopen=setup.exe /silent\r\n";
    let targets = parse_autorun_targets(content);
    assert_eq!(targets, vec!["setup.exe"]);
}

#[test]
fn parse_shellexecute() {
    let content = "[autorun]\r\nshellexecute=installer.exe\r\n";
    let targets = parse_autorun_targets(content);
    assert_eq!(targets, vec!["installer.exe"]);
}

#[test]
fn parse_shell_command() {
    let content = "[autorun]\r\nshell\\open\\command=launch.exe\r\nshell\\explore\\command=browser.exe /s\r\n";
    let targets = parse_autorun_targets(content);
    assert!(targets.contains(&"launch.exe".to_string()));
    assert!(targets.contains(&"browser.exe".to_string()));
}

#[test]
fn parse_mixed_case_keys() {
    let content = "[AutoRun]\r\nOPEN=Setup.EXE\r\nShellExecute=Install.exe\r\n";
    let targets = parse_autorun_targets(content);
    assert!(targets.contains(&"Setup.EXE".to_string()));
    assert!(targets.contains(&"Install.exe".to_string()));
}

#[test]
fn parse_paths_with_subdirectory() {
    let content = "[autorun]\r\nopen=subdir\\setup.exe\r\n";
    let targets = parse_autorun_targets(content);
    assert_eq!(targets, vec!["subdir\\setup.exe"]);
}

#[test]
fn parse_empty_content_returns_empty() {
    let targets = parse_autorun_targets("");
    assert!(targets.is_empty());
}

#[test]
fn parse_no_executable_references() {
    let content = "[autorun]\r\nicon=myicon.ico\r\nlabel=My USB\r\n";
    let targets = parse_autorun_targets(content);
    assert!(targets.is_empty());
}

#[test]
fn parse_quoted_path() {
    let content = "[autorun]\r\nopen=\"my setup.exe\" /quiet\r\n";
    let targets = parse_autorun_targets(content);
    assert_eq!(targets, vec!["my setup.exe"]);
}

#[test]
fn parse_deduplicates_targets() {
    let content = "[autorun]\r\nopen=setup.exe\r\nshellexecute=setup.exe\r\n";
    let targets = parse_autorun_targets(content);
    assert_eq!(targets, vec!["setup.exe"]);
}
