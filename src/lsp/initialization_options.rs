use zed_extension_api::settings::LspSettings;
use zed_extension_api::Worktree;

use serde_json::{json, Value};

use crate::config::SERVER_ID;

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

pub fn resolve_initialization_options(worktree: &Worktree) -> Value {
    let user_options = LspSettings::for_worktree(SERVER_ID, worktree)
        .ok()
        .and_then(|s| s.initialization_options);

    let mut options = default_initialization_options();
    if let Some(user_options) = user_options {
        merge(&mut options, &user_options);
    }

    options
}
