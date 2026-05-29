use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

use crate::config::{BINARY_NAME, GITHUB_REPO};
use crate::lsp::platform::{cached_binary_suffix, release_asset_name};

pub struct BinaryResolver {
    cached_binary_path: Option<String>,
}

impl BinaryResolver {
    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn resolve(
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
        let asset_name = release_asset_name(platform, arch)?;

        let bin_suffix = cached_binary_suffix(platform);

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
