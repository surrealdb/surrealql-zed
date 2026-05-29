use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

use serde_json::{json, Value};

const BINARY_NAME: &str = "surrealql-language-server";
const GITHUB_REPO: &str = "surrealdb/surrealql-language-server";
const SERVER_ID: &str = "surrealql-lsp";

struct SurrealQLExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for SurrealQLExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
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
        if let Some(path) = binary.and_then(|b| b.path) {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        let command = self.resolve_binary(language_server_id, worktree, version.as_deref())?;

        Ok(zed::Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<Value>> {
        let user_options = LspSettings::for_worktree(SERVER_ID, worktree)
            .ok()
            .and_then(|s| s.initialization_options);

        let mut options = default_initialization_options();
        if let Some(user_options) = user_options {
            merge(&mut options, &user_options);
        }

        Ok(Some(options))
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

/// Defaults mirroring the VS Code extension's `buildInitializationOptions`
/// (see `surrealql-vsx/src/settings.ts`). Users override individual leaves via
/// `lsp.surrealql-lsp.initialization_options` in their Zed `settings.json`.
fn default_initialization_options() -> Value {
    json!({
        "surrealql": {
            "connection": {
                "endpoint": "http://localhost:8000",
                "username": "root",
                "password": "root"
            },
            "activeAuthContext": "root",
            "metadata": { "mode": "both" }
        }
    })
}

/// Recursively merges `overlay` into `base`. Objects are merged key by key;
/// any other value type in `overlay` replaces the value in `base`.
fn merge(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                merge(
                    base_map.entry(key.clone()).or_insert(Value::Null),
                    overlay_value,
                );
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}

impl SurrealQLExtension {
    fn resolve_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
        version: Option<&str>,
    ) -> Result<String> {
        let pinned = version.filter(|v| !v.is_empty() && *v != "latest");

        if pinned.is_none() {
            // Prefer a locally installed binary (dev/manual installs via cargo install)
            if let Some(path) = worktree.which(BINARY_NAME) {
                return Ok(path);
            }

            // Return the cached path if the file still exists
            if let Some(cached) = &self.cached_binary_path {
                if std::fs::metadata(cached).is_ok_and(|m| m.is_file()) {
                    return Ok(cached.clone());
                }
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = match pinned {
            Some(tag) => zed::github_release_by_tag_name(GITHUB_REPO, tag)
                .map_err(|_| format!("no release found for {GITHUB_REPO} with tag {tag}"))?,
            None => zed::latest_github_release(
                GITHUB_REPO,
                zed::GithubReleaseOptions {
                    require_assets: true,
                    pre_release: true,
                },
            )
            .map_err(|_| {
                format!(
                    "no release found for {BINARY_NAME}. \
                    Install it manually with: cargo install --git https://github.com/{GITHUB_REPO}"
                )
            })?,
        };

        let (platform, arch) = zed::current_platform();

        // Asset names match the surrealql-language-server release CI uploads:
        //   surrealql-language-server-macos-arm64
        //   surrealql-language-server-linux-amd64
        //   surrealql-language-server-linux-arm64
        //   surrealql-language-server-windows-amd64.exe
        //
        // No native x86_64 macOS build is published; fall back to the arm64 binary
        // (Rosetta 2 is required on Intel Macs).
        let asset_name = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => {
                format!("{BINARY_NAME}-macos-arm64")
            }
            (zed::Os::Mac, zed::Architecture::X8664) => {
                format!("{BINARY_NAME}-macos-arm64")
            }
            (zed::Os::Linux, zed::Architecture::X8664) => {
                format!("{BINARY_NAME}-linux-amd64")
            }
            (zed::Os::Linux, zed::Architecture::Aarch64) => {
                format!("{BINARY_NAME}-linux-arm64")
            }
            (zed::Os::Windows, zed::Architecture::X8664) => {
                format!("{BINARY_NAME}-windows-amd64.exe")
            }
            _ => {
                return Err(format!(
                    "unsupported platform or architecture for {BINARY_NAME}"
                ))
            }
        };

        let bin_suffix = if matches!(platform, zed::Os::Windows) {
            ".exe"
        } else {
            ""
        };

        // Version-tagged filename so we re-download on updates
        let binary_path = format!("{BINARY_NAME}-{}{bin_suffix}", release.version);

        if !std::fs::metadata(&binary_path).is_ok_and(|m| m.is_file()) {
            let asset = release
                .assets
                .iter()
                .find(|a| a.name == asset_name)
                .ok_or_else(|| format!("no release asset found; expected: {asset_name}"))?;

            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            // Binaries are uploaded as raw uncompressed files by the CI workflow
            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| format!("failed to download {BINARY_NAME}: {e}"))?;

            zed::make_file_executable(&binary_path)
                .map_err(|e| format!("failed to mark {BINARY_NAME} as executable: {e}"))?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

zed::register_extension!(SurrealQLExtension);
