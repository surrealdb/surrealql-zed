use zed_extension_api as zed;

use crate::config::{BINARY_NAME, source_build_hint};

/// Asset names match the surrealql-language-server release CI uploads:
///   surrealql-language-server-macos-arm64
///   surrealql-language-server-linux-amd64
///   surrealql-language-server-linux-arm64
///   surrealql-language-server-windows-amd64.exe
///
/// No x86_64 macOS build is published. Rosetta 2 translates x86_64 to arm64,
/// not the reverse, so the arm64 asset cannot run on an Intel Mac: those fall
/// through to the unsupported-platform error rather than a broken download.
pub fn release_asset_name(platform: zed::Os, arch: zed::Architecture) -> Result<String, String> {
    let asset_name = match (platform, arch) {
        (zed::Os::Mac, zed::Architecture::Aarch64) => format!("{BINARY_NAME}-macos-arm64"),
        (zed::Os::Linux, zed::Architecture::X8664) => format!("{BINARY_NAME}-linux-amd64"),
        (zed::Os::Linux, zed::Architecture::Aarch64) => format!("{BINARY_NAME}-linux-arm64"),
        (zed::Os::Windows, zed::Architecture::X8664) => {
            format!("{BINARY_NAME}-windows-amd64.exe")
        }
        _ => return Err(unsupported_platform(platform, arch)),
    };

    Ok(asset_name)
}

/// The download paths attach an install hint pointing at the releases page.
/// That is a dead end here: reaching this arm means no release asset exists for
/// this target in the first place, so the only route open is a source build.
fn unsupported_platform(platform: zed::Os, arch: zed::Architecture) -> String {
    let platform = match platform {
        zed::Os::Mac => "macOS",
        zed::Os::Linux => "Linux",
        zed::Os::Windows => "Windows",
    };
    let arch = match arch {
        zed::Architecture::Aarch64 => "aarch64",
        zed::Architecture::X8664 => "x86_64",
        zed::Architecture::X86 => "x86",
    };

    format!(
        "{BINARY_NAME} publishes no prebuilt binary for {platform} {arch}. \
        Build one from source: {}",
        source_build_hint()
    )
}

pub fn cached_binary_suffix(platform: zed::Os) -> &'static str {
    if matches!(platform, zed::Os::Windows) {
        ".exe"
    } else {
        ""
    }
}
