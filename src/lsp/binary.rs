use zed_extension_api::{self as zed, LanguageServerId, Result, Worktree};

use crate::config::{BINARY_NAME, GITHUB_REPO, pinned_lsp_version, source_build_hint};
use crate::lsp::platform::{cached_binary_suffix, release_asset_name};

/// `settings.version` value that opts out of the pinned release and tracks the
/// newest published one instead.
const LATEST: &str = "latest";

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
        // Only an explicit `settings.version` counts as user intent; an unset or
        // empty value leaves us free to prefer whatever is already installed.
        let requested = version.map(str::trim).filter(|v| !v.is_empty());

        if requested.is_none() {
            // Prefer a locally installed binary (dev/manual installs via cargo install)
            if let Some(path) = worktree.which(BINARY_NAME) {
                return Ok(path);
            }

            // Return the cached path if the file still exists
            if let Some(cached) = &self.cached_binary_path
                && std::fs::metadata(cached).is_ok_and(|m| m.is_file())
            {
                return Ok(cached.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = match requested {
            // `latest` tracks the newest published release. `pre_release` is an
            // exact-equality filter in Zed, not "also include pre-releases", so
            // `false` here means "the newest non-prerelease".
            Some(LATEST) => zed::latest_github_release(
                GITHUB_REPO,
                zed::GithubReleaseOptions {
                    require_assets: true,
                    pre_release: false,
                },
            )
            .map_err(|_| install_hint(format!("no release found for {BINARY_NAME}")))?,

            // An explicit tag, or otherwise the release this extension is pinned to.
            other => {
                let tag = match other {
                    Some(tag) => tag,
                    None => pinned_lsp_version(),
                };
                zed::github_release_by_tag_name(GITHUB_REPO, tag).map_err(|_| {
                    install_hint(format!("no release found for {GITHUB_REPO} with tag {tag}"))
                })?
            }
        };

        let (platform, arch) = zed::current_platform();
        // Already carries its own hint: the releases link `install_hint` adds
        // is a dead end when no asset exists for this target at all.
        let asset_name = release_asset_name(platform, arch)?;

        let bin_suffix = cached_binary_suffix(platform);

        // Version-tagged filename so we re-download on updates
        let binary_path = format!("{BINARY_NAME}-{}{bin_suffix}", release.version);

        if !std::fs::metadata(&binary_path).is_ok_and(|m| m.is_file()) {
            let asset = release
                .assets
                .iter()
                .find(|a| a.name == asset_name)
                .ok_or_else(|| {
                    install_hint(format!("no release asset found; expected: {asset_name}"))
                })?;

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

/// Every failure to obtain a binary ends up here, so the user gets a route
/// forward rather than just a reason it did not work.
fn install_hint(reason: String) -> String {
    format!(
        "{reason}. Download a binary from https://github.com/{GITHUB_REPO}/releases and set \
        `lsp.surrealql-lsp.binary.path` to it, or build from source: {}",
        source_build_hint()
    )
}
