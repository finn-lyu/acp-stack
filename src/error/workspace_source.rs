//! Error helpers for the workspace code/data-source validators and materialization pipeline.

use http::StatusCode;

use super::StackError;

pub(super) fn error_code(err: &StackError) -> Option<&'static str> {
    use StackError::*;
    Some(match err {
        WorkspaceCodeSourceInvalid { .. } | WorkspaceDataSourceInvalid { .. } => "config.invalid",
        WorkspaceUploadsNotUnderRoot => "config.invalid",
        WorkspaceDestinationNotEmpty { .. } => "workspace.destination_not_empty",
        WorkspaceDestinationOutsideRoot { .. } => "workspace.destination_outside_root",
        WorkspaceMaterializeFailed { .. } => "workspace.materialize_failed",
        WorkspaceCommandFailed { .. } => "workspace.command_failed",
        _ => return None,
    })
}

pub(super) fn public_message(err: &StackError) -> Option<String> {
    use StackError::*;
    Some(match err {
        // Reasons can carry operator-declared local paths and I/O error text
        // from the source validators, so the public surface names only the field.
        WorkspaceCodeSourceInvalid { index, .. } => {
            format!("workspace.code_sources[{index}] is invalid")
        }
        WorkspaceDataSourceInvalid { index, .. } => {
            format!("workspace.data_sources[{index}] is invalid")
        }
        WorkspaceUploadsNotUnderRoot => {
            "workspace.uploads must be inside workspace.root".to_owned()
        }
        WorkspaceDestinationNotEmpty { .. } => {
            "workspace destination is not empty and is not a known acp-stack source directory"
                .to_owned()
        }
        WorkspaceDestinationOutsideRoot { .. } => {
            "workspace destination is outside workspace.root".to_owned()
        }
        // Reasons name destination paths and I/O sources at the call sites.
        WorkspaceMaterializeFailed { .. } => "workspace materialization failed".to_owned(),
        // `stderr_tail` is raw subprocess output; the command label and exit
        // status are the identifiers the API may carry.
        WorkspaceCommandFailed { command, exit, .. } => match exit {
            Some(code) => format!("`{command}` exited with status {code}"),
            None => format!("`{command}` exited without a status"),
        },
        _ => return None,
    })
}

pub(super) fn http_status(err: &StackError) -> Option<StatusCode> {
    use StackError::*;
    Some(match err {
        WorkspaceCodeSourceInvalid { .. } | WorkspaceDataSourceInvalid { .. } => {
            StatusCode::BAD_REQUEST
        }
        WorkspaceUploadsNotUnderRoot => StatusCode::BAD_REQUEST,
        WorkspaceDestinationNotEmpty { .. } | WorkspaceDestinationOutsideRoot { .. } => {
            StatusCode::CONFLICT
        }
        WorkspaceMaterializeFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        WorkspaceCommandFailed { .. } => StatusCode::BAD_GATEWAY,
        _ => return None,
    })
}
