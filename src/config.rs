pub const BINARY_NAME: &str = "surrealql-language-server";
pub const GITHUB_REPO: &str = "surrealdb/surrealql-language-server";
pub const SERVER_ID: &str = "surrealql-lsp";

const PINNED_LSP_VERSION_RAW: &str = include_str!("../.lsp-version");

/// The `surrealql-language-server` release this extension is built against, and
/// the version downloaded when the user has not pinned one themselves.
/// Maintained by `.github/workflows/bump-lsp.yml`.
pub fn pinned_lsp_version() -> &'static str {
    PINNED_LSP_VERSION_RAW.trim()
}
