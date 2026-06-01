use zed_extension_api::Worktree;
use zed_extension_api::settings::LspSettings;

use crate::config::SERVER_ID;

pub struct LspCommandConfig {
    pub path_override: Option<String>,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub version: Option<String>,
}

impl LspCommandConfig {
    pub fn from_worktree(worktree: &Worktree) -> Self {
        let lsp_settings = LspSettings::for_worktree(SERVER_ID, worktree).ok();

        // Optional version pin (`lsp.surrealql-lsp.settings.version`) mirrors the
        // VS Code `surrealql.lsp.version` setting. Empty or "latest" means newest.
        let version = lsp_settings
            .as_ref()
            .and_then(|s| s.settings.as_ref())
            .and_then(|v| v.get("version"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        // `CommandSettings` is not `Clone`, so take ownership and read each field.
        let binary = lsp_settings.and_then(|s| s.binary);

        let args = binary
            .as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_default();
        let env: Vec<(String, String)> = binary
            .as_ref()
            .and_then(|b| b.env.clone())
            .map(|m| m.into_iter().collect())
            .unwrap_or_default();

        // An explicit `binary.path` short-circuits PATH detection and download,
        // mirroring the VS Code `surrealql.lsp.binaryPath` override.
        let path_override = binary.and_then(|b| b.path);

        Self {
            path_override,
            args,
            env,
            version,
        }
    }
}
