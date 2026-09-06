//! Supabase logging sink and config error helpers (`logging.supabase.*`).

use http::StatusCode;

use super::StackError;

pub(super) fn error_code(err: &StackError) -> Option<&'static str> {
    use StackError::*;
    Some(match err {
        MissingSupabaseApiKey { .. } => "logging.supabase.missing_api_key",
        MissingSupabaseDbUrl { .. } => "logging.supabase.missing_db_url",
        InvalidSupabaseUrl { .. } => "logging.supabase.invalid_url",
        InvalidSupabaseSchema { .. } => "logging.supabase.invalid_schema",
        InvalidSupabaseTablePrefix { .. } => "logging.supabase.invalid_table_prefix",
        SupabaseSinkHttp { .. } => "logging.supabase.http_error",
        SupabaseSinkUnknownTable { .. } => "logging.supabase.unknown_table",
        SupabaseCliFailed { .. } => "logging.supabase.cli_failed",
        _ => return None,
    })
}

pub(super) fn public_message(err: &StackError) -> Option<String> {
    use StackError::*;
    Some(match err {
        MissingSupabaseApiKey { .. } => {
            "secret store is missing Supabase secret API key reference".to_owned()
        }
        MissingSupabaseDbUrl { .. } => {
            "secret store is missing Supabase Postgres writer DB URL reference".to_owned()
        }
        InvalidSupabaseUrl { .. } => "[logging.supabase].url must start with `https://`".to_owned(),
        InvalidSupabaseSchema { .. } => {
            "[logging.supabase].schema is not a safe Postgres identifier".to_owned()
        }
        InvalidSupabaseTablePrefix { .. } => {
            "[logging.supabase].table_prefix is not a safe Postgres identifier prefix".to_owned()
        }
        SupabaseSinkHttp { status, .. } => {
            format!("Supabase sink rejected upload with HTTP {status}")
        }
        SupabaseSinkUnknownTable { table } => {
            format!("Supabase sink received a row for unknown source table `{table}`")
        }
        // The command line and stderr tail stay in local logs.
        SupabaseCliFailed { .. } => "Supabase CLI setup failed".to_owned(),
        _ => return None,
    })
}

pub(super) fn http_status(err: &StackError) -> Option<StatusCode> {
    use StackError::*;
    Some(match err {
        MissingSupabaseApiKey { .. }
        | MissingSupabaseDbUrl { .. }
        | SupabaseSinkHttp { .. }
        | SupabaseSinkUnknownTable { .. }
        | SupabaseCliFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        // Raised by `validate_config` on caller-supplied TOML through the
        // config validate/import routes, so they are client input.
        InvalidSupabaseUrl { .. }
        | InvalidSupabaseSchema { .. }
        | InvalidSupabaseTablePrefix { .. } => StatusCode::BAD_REQUEST,
        _ => return None,
    })
}
