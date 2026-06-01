use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

use serde_json::Value;

use crate::config::SERVER_ID;
use crate::lsp::{BinaryResolver, LspCommandConfig, resolve_initialization_options};

pub struct SurrealQLExtension {
    binary_resolver: BinaryResolver,
}

impl zed::Extension for SurrealQLExtension {
    fn new() -> Self {
        Self {
            binary_resolver: BinaryResolver::new(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let config = LspCommandConfig::from_worktree(worktree);

        if let Some(path) = config.path_override {
            return Ok(zed::Command {
                command: path,
                args: config.args,
                env: config.env,
            });
        }

        let command = self.binary_resolver.resolve(
            language_server_id,
            worktree,
            config.version.as_deref(),
        )?;

        Ok(zed::Command {
            command,
            args: config.args,
            env: config.env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(Some(resolve_initialization_options(worktree)))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        Ok(LspSettings::for_worktree(SERVER_ID, worktree)
            .ok()
            .and_then(|s| s.settings))
    }
}
