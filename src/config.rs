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

/// `cargo install --git` cannot build this server: its `build.rs` compiles the
/// tree-sitter grammar from a sibling `surrealql-tree-sitter` checkout, which a
/// bare git install does not provide. This is the setup the server's own README
/// documents for source builds.
pub fn source_build_hint() -> String {
    format!(
        "clone {GITHUB_REPO}, run `bash scripts/setup-grammar.sh` (or set \
        TREE_SITTER_SURREALQL_DIR to an existing surrealql-tree-sitter checkout), \
        then `cargo install --path .`"
    )
}
