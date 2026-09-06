use super::StackError;
use std::path::PathBuf;

#[test]
fn stack_update_binary_swap_reports_rollback_outcome() {
    let io = || std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let clean = StackError::StackUpdateBinarySwap {
        path: PathBuf::from("/opt/acps/acps"),
        source: io(),
        rollback_errors: Vec::new(),
    };
    assert_eq!(clean.error_code(), "stack.update_binary_swap_failed");
    assert_eq!(
        clean.to_string(),
        "failed to replace /opt/acps/acps during stack update binary swap: denied"
    );

    let broken = StackError::StackUpdateBinarySwap {
        path: PathBuf::from("/opt/acps/acps"),
        source: io(),
        rollback_errors: vec!["failed to restore /opt/acps/acps: denied".to_owned()],
    };
    assert_eq!(broken.error_code(), "stack.update_binary_swap_failed");
    assert_eq!(
        broken.to_string(),
        "failed to replace /opt/acps/acps during stack update binary swap: denied; rollback errors: failed to restore /opt/acps/acps: denied"
    );
    assert!(!broken.public_message().contains("/opt"));
}

#[test]
fn workspace_command_failure_display_formats_exit_status_plainly() {
    let exited = StackError::WorkspaceCommandFailed {
        command: "git clone",
        exit: Some(128),
        stderr_tail: "repository not found".to_owned(),
    }
    .to_string();
    assert_eq!(
        exited,
        "`git clone` exited with status 128: repository not found"
    );
    assert!(
        !exited.contains("Some("),
        "exit status must not expose Option debug formatting: {exited}"
    );

    let signaled = StackError::WorkspaceCommandFailed {
        command: "git clone",
        exit: None,
        stderr_tail: "terminated by signal".to_owned(),
    }
    .to_string();
    assert_eq!(
        signaled,
        "`git clone` exited without a status: terminated by signal"
    );
}

const CANARY_PATH: &str = "/home/operator/canary";
const CANARY_SECRET: &str = "sk-canary-secret";

fn assert_public_message_excludes(err: &StackError, needles: &[&str]) -> String {
    let message = err.public_message();
    for needle in needles {
        assert!(
            !message.contains(needle),
            "public message must not contain `{needle}`: {message}"
        );
    }
    message
}

#[test]
fn agent_test_failed_public_message_drops_reason_and_keeps_stage() {
    let err = StackError::AgentTestFailed {
        stage: "ACP initialize".to_owned(),
        reason: format!("spawn /agent --root {CANARY_PATH} failed"),
        code: "spawn_failed",
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "--root"]);
    assert_eq!(message, "agent test failed at ACP initialize");
}

#[test]
fn agent_initialize_failed_public_message_drops_free_text_reason() {
    let err = StackError::AgentInitializeFailed {
        reason: format!("installer thread join failed: write {CANARY_PATH}: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "agent failed to initialize");
}

#[test]
fn agent_switch_conflict_public_message_keeps_identifier_reason() {
    let err = StackError::AgentSwitchConflict {
        reason: "an earlier switch to `target-b` did not finish (phase `swap`)".to_owned(),
    };
    let message = err.public_message();
    assert!(
        message.contains("target-b"),
        "identifier reason must survive: {message}"
    );
}

#[test]
fn agent_installer_binary_unrunnable_public_message_drops_io_error() {
    let err = StackError::AgentInstallerBinaryUnrunnable {
        path: PathBuf::from(CANARY_PATH),
        source: std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("cannot execute {CANARY_PATH}"),
        ),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "cannot execute"]);
    assert_eq!(
        message,
        "agent installer produced a binary that cannot be spawned on this host"
    );
}

#[test]
fn agent_installer_log_persist_public_message_drops_path() {
    let err = StackError::AgentInstallerLogPersist {
        path: PathBuf::from(CANARY_PATH),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH]);
    assert_eq!(message, "failed to persist the installer log");
}

#[test]
fn skill_install_target_conflict_public_message_drops_path_keeps_reason() {
    let err = StackError::SkillInstallTargetConflict {
        path: PathBuf::from(CANARY_PATH),
        reason: "target exists but is not a directory".to_owned(),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH]);
    assert_eq!(
        message,
        "skill install target conflict: target exists but is not a directory"
    );
}

#[test]
fn skill_install_failed_public_message_drops_free_text_reason() {
    let err = StackError::SkillInstallFailed {
        reason: format!("stat skill target `{CANARY_PATH}`: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "skill install failed");
}

#[test]
fn registry_load_public_message_drops_free_text_reason() {
    let err = StackError::RegistryLoad {
        reason: format!("failed to read operator override {CANARY_PATH}: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "agent registry could not be loaded");
}

#[test]
fn init_run_corrupted_public_message_drops_free_text_reason() {
    let err = StackError::InitRunCorrupted {
        reason: format!("init run 7 has invalid args_json: invalid type: string \"{CANARY_PATH}\""),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH]);
    assert_eq!(message, "init run state is corrupted");
}

#[test]
fn agent_install_all_paths_failed_public_message_keeps_composed_summary() {
    let err = StackError::AgentInstallAllPathsFailed {
        summary: "shell: agent installer exited with status 7; npm: skipped, missing tools: npm"
            .to_owned(),
    };
    let message = err.public_message();
    assert!(
        message.contains("shell: agent installer exited with status 7"),
        "composed per-path summary must survive: {message}"
    );
}

#[test]
fn github_release_archive_extract_public_message_drops_reason_keeps_repo() {
    let err = StackError::GithubReleaseArchiveExtract {
        repo: "owner/repo".to_owned(),
        reason: format!("failed to create destination dir {CANARY_PATH}: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "failed to extract release archive from owner/repo");
}

#[test]
fn workspace_destination_errors_public_message_drops_paths() {
    let not_empty = StackError::WorkspaceDestinationNotEmpty {
        dest: CANARY_PATH.to_owned(),
    };
    let message = assert_public_message_excludes(&not_empty, &[CANARY_PATH]);
    assert_eq!(
        message,
        "workspace destination is not empty and is not a known acp-stack source directory"
    );

    let outside = StackError::WorkspaceDestinationOutsideRoot {
        dest: CANARY_PATH.to_owned(),
        root: format!("{CANARY_PATH}/root"),
    };
    let message = assert_public_message_excludes(&outside, &[CANARY_PATH]);
    assert_eq!(message, "workspace destination is outside workspace.root");
}

#[test]
fn workspace_materialize_failed_public_message_drops_free_text_reason() {
    let err = StackError::WorkspaceMaterializeFailed {
        reason: format!("could not create `{CANARY_PATH}`: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "workspace materialization failed");
}

#[test]
fn workspace_command_failed_public_message_drops_stderr() {
    let exited = StackError::WorkspaceCommandFailed {
        command: "git clone",
        exit: Some(128),
        stderr_tail: format!("fatal: could not write {CANARY_PATH}"),
    };
    let message = assert_public_message_excludes(&exited, &[CANARY_PATH, "fatal"]);
    assert_eq!(message, "`git clone` exited with status 128");

    let signaled = StackError::WorkspaceCommandFailed {
        command: "git clone",
        exit: None,
        stderr_tail: format!("terminated while writing {CANARY_PATH}"),
    };
    let message = assert_public_message_excludes(&signaled, &[CANARY_PATH, "terminated"]);
    assert_eq!(message, "`git clone` exited without a status");
}

#[test]
fn workspace_source_invalid_public_message_drops_free_text_reason() {
    let code = StackError::WorkspaceCodeSourceInvalid {
        index: 0,
        reason: format!("credential_ref `{CANARY_SECRET}` is not a valid secret reference"),
    };
    let message = assert_public_message_excludes(&code, &[CANARY_SECRET]);
    assert_eq!(message, "workspace.code_sources[0] is invalid");

    let data = StackError::WorkspaceDataSourceInvalid {
        index: 1,
        reason: format!("local path `{CANARY_PATH}` is not readable: permission denied"),
    };
    let message = assert_public_message_excludes(&data, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "workspace.data_sources[1] is invalid");
}

#[test]
fn workspace_not_found_public_messages_keep_relative_requested_path() {
    let missing = StackError::WorkspaceNotFound {
        requested: "notes/x.txt".to_owned(),
    };
    assert_eq!(
        missing.public_message(),
        "workspace path `notes/x.txt` was not found"
    );

    let missing_parent = StackError::WorkspaceParentNotFound {
        requested: "notes/x.txt".to_owned(),
    };
    assert_eq!(
        missing_parent.public_message(),
        "workspace parent directory for `notes/x.txt` was not found"
    );
}

#[test]
fn command_cwd_outside_workspace_public_message_drops_path() {
    let err = StackError::CommandCwdOutsideWorkspace {
        requested: CANARY_PATH.to_owned(),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH]);
    assert_eq!(message, "command cwd resolves outside the workspace root");
}

#[test]
fn invalid_secret_ref_name_public_message_drops_offending_name() {
    let err = StackError::InvalidSecretRefName {
        name: CANARY_SECRET.to_owned(),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_SECRET]);
    assert_eq!(
        message,
        "a secret ref name is invalid; use ASCII letters, digits, and underscores, and do not start with a digit"
    );
}

#[test]
fn invalid_secret_ref_name_display_matches_public_guidance() {
    let err = StackError::InvalidSecretRefName {
        name: "9lives".to_owned(),
    };
    let display = err.to_string();
    assert!(
        display.contains("do not start with a digit"),
        "Display guidance must cover the leading-digit rule: {display}"
    );
}

#[test]
fn archive_read_failed_public_message_drops_free_text_reason() {
    let err = StackError::ArchiveReadFailed {
        reason: format!("could not open `{CANARY_PATH}`: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(message, "archive read failed");
}

#[test]
fn safe_download_failed_public_message_drops_reason_keeps_url() {
    let err = StackError::SafeDownloadFailed {
        url: "https://example.com/archive.tar.gz".to_owned(),
        reason: format!("could not create destination `{CANARY_PATH}`: permission denied"),
    };
    let message = assert_public_message_excludes(&err, &[CANARY_PATH, "permission denied"]);
    assert_eq!(
        message,
        "download from https://example.com/archive.tar.gz failed"
    );
}

#[test]
fn download_public_messages_strip_url_credentials() {
    let leaky = format!("https://user:{CANARY_SECRET}@example.com/path?token={CANARY_SECRET}#frag");

    let failed = StackError::SafeDownloadFailed {
        url: leaky.clone(),
        reason: "read failed".to_owned(),
    };
    let message =
        assert_public_message_excludes(&failed, &[CANARY_SECRET, "user", "token", "frag"]);
    assert_eq!(message, "download from https://example.com/path failed");

    let status = StackError::SafeDownloadHttpStatus {
        url: leaky.clone(),
        status: 403,
    };
    let message =
        assert_public_message_excludes(&status, &[CANARY_SECRET, "user", "token", "frag"]);
    assert_eq!(
        message,
        "download from https://example.com/path failed with HTTP status 403"
    );

    let redirect = StackError::SafeDownloadInsecureRedirect { url: leaky };
    let message = assert_public_message_excludes(&redirect, &[CANARY_SECRET, "token", "frag"]);
    assert_eq!(
        message,
        "download URL `https://example.com/path` is not allowed (only https:// is permitted)"
    );
}

#[test]
fn download_public_message_renders_bare_origin_without_trailing_slash() {
    let err = StackError::SafeDownloadFailed {
        url: "https://example.com".to_owned(),
        reason: "connect failed".to_owned(),
    };
    assert_eq!(
        err.public_message(),
        "download from https://example.com failed"
    );
}

#[test]
fn download_public_messages_fall_back_when_url_is_missing_or_unparseable() {
    let empty = StackError::SafeDownloadFailed {
        url: String::new(),
        reason: "connect failed".to_owned(),
    };
    assert_eq!(empty.public_message(), "download failed");

    let unparseable = StackError::SafeDownloadFailed {
        url: "not a url".to_owned(),
        reason: "connect failed".to_owned(),
    };
    let message = assert_public_message_excludes(&unparseable, &["not a url"]);
    assert_eq!(message, "download failed");

    let empty_status = StackError::SafeDownloadHttpStatus {
        url: String::new(),
        status: 502,
    };
    assert_eq!(
        empty_status.public_message(),
        "download failed with HTTP status 502"
    );
}
