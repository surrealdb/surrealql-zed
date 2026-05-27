use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

const BINARY_NAME: &str = "surrealql-language-server";
const GITHUB_REPO: &str = "surrealdb/surrealql-language-server";

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
        let binary_path = self.resolve_binary(language_server_id, worktree)?;
        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: Default::default(),
        })
    }
}

impl SurrealQLExtension {
    fn resolve_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String> {
        // Prefer a locally installed binary (dev/manual installs via cargo install)
        if let Some(path) = worktree.which(BINARY_NAME) {
            return Ok(path);
        }

        // Return the cached path if the file still exists
        if let Some(cached) = &self.cached_binary_path {
            if std::fs::metadata(cached).map_or(false, |m| m.is_file()) {
                return Ok(cached.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
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
        })?;

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

        if !std::fs::metadata(&binary_path).map_or(false, |m| m.is_file()) {
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
