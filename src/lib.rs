mod config;
mod extension;
mod lsp;

use extension::SurrealQLExtension;
use zed_extension_api as zed;

zed::register_extension!(SurrealQLExtension);
