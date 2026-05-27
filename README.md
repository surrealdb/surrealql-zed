# SurrealQL for Zed

[SurrealQL](https://surrealdb.com/docs/surrealql) language support for the [Zed](https://zed.dev) editor.

This extension targets parity with [`@surrealdb/lezer`](https://github.com/surrealdb/codemirror/tree/main/packages/lezer-surrealql) and [`@surrealdb/codemirror`](https://github.com/surrealdb/codemirror/tree/main/packages/codemirror-surrealql) for syntax highlighting and editor ergonomics. The tree-sitter grammar is pinned to the lezer-aligned [`tree-sitter-parity`](https://github.com/surrealdb/surrealql-tree-sitter/tree/tree-sitter-parity) branch of [`surrealdb/surrealql-tree-sitter`](https://github.com/surrealdb/surrealql-tree-sitter).

## Features

- Syntax highlighting aligned with `@surrealdb/lezer` (PascalCase tree-sitter nodes)
- Bracket matching, auto-close, and rainbow brackets
- Line comments (`--`, `#`, `//`) and block comments (`/* */`)
- Smart indentation inside `{}`, `[]`, and blocks
- Syntax-aware folding for objects, arrays, sets, and blocks (when supported by your Zed version)
- Embedded JavaScript highlighting inside scripting functions (`FunctionJs`)
- SurrealQL Language Server auto-download (`surrealql-language-server`)

## Installation

Search for **SurrealQL** in Zed's extension marketplace (`zed: extensions`) and install.

## Usage

Open any `.surql` or `.surrealql` file and syntax highlighting is applied automatically.

Embedded JavaScript in scripting functions is highlighted as JavaScript:

```surql
DEFINE FUNCTION fn::greet($name: string) {
    return function($name) {
        return `Hello, ${arguments[0]}!`;
    };
};
```

## Language server

The extension downloads the latest pre-release of [`surrealql-language-server`](https://github.com/surrealdb/surrealql-language-server) from GitHub when no local binary is found (`cargo install --git …` also works).

Supported platforms:

| Platform | Asset |
|----------|-------|
| macOS arm64 | `surrealql-language-server-macos-arm64` |
| macOS x86_64 | `surrealql-language-server-macos-arm64` (Rosetta 2) |
| Linux amd64 | `surrealql-language-server-linux-amd64` |
| Linux arm64 | `surrealql-language-server-linux-arm64` |
| Windows amd64 | `surrealql-language-server-windows-amd64.exe` |

## Parity scope

### Implemented (codemirror-surrealql / lezer-surrealql)

| Capability | Source |
|------------|--------|
| Full SurrealQL parse tree | `surrealql-tree-sitter` parity grammar |
| Syntax highlighting | `highlights.scm` ← `lezer-surrealql/src/highlight.js` |
| Embedded JS | `injections.scm` ← `parseMixed` on `FunctionJs` |
| Bracket / comment editor config | `config.toml` ← `surrealql.ts` `languageData` |
| Indentation | `indents.scm` ← `continuedIndent` on `Object` / `Array` |
| Folding | `folds.scm` ← `foldInside` on `Object` / `Array` / `Block` |

### Deferred (requires tree-sitter or LSP changes)

| Capability | Reason |
|------------|--------|
| Scoped parse tops (`permission`, `index`, `combined-results`, `syntax`) | Lezer exposes five `@top` rules; tree-sitter parity grammar has one entry point |
| Version linter (`surrealqlVersionLinter`) | Lezer `[since=]` / `[until=]` node props are not in tree-sitter; Zed has no tree-sitter diagnostic hook |
| Structural JS parse tree | Tree-sitter consumes JS bodies as an opaque token; highlighting is blob injection, not nested JS AST |

## Development

### Prerequisites

- [Zed](https://zed.dev) (with extension development support)
- [Rust](https://rustup.rs) toolchain with `wasm32-wasip2` target

### Local setup

```sh
git clone https://github.com/surrealdb/surrealql-zed
```

Then in Zed, open the command palette and run:

```
zed: install dev extension
```

Point it at the cloned directory.

### Updating locally

After changing extension files, tree-sitter queries, or the pinned grammar revision:

1. Run **`zed: install dev extension`** again and select this directory.
2. If the grammar revision changed, clear Zed’s cached checkout first (see below).
3. Reload any open `.surql` buffers (or restart Zed) to pick up grammar/LSP changes.

### Grammar

The tree-sitter grammar is fetched from [`surrealdb/surrealql-tree-sitter`](https://github.com/surrealdb/surrealql-tree-sitter) at the revision pinned in `extension.toml` (`rev` field). The current pin targets the [`tree-sitter-parity`](https://github.com/surrealdb/surrealql-tree-sitter/tree/tree-sitter-parity) branch, which includes the lezer-aligned PascalCase node names used by the query files in `languages/surql/`.

To bump the grammar:

1. Push your grammar change to `surrealql-tree-sitter` (run `bun run gen` there first so `src/parser.c` is committed).
2. Update `rev` in `extension.toml` to the new commit SHA.
3. Clear the cached grammar and reinstall the dev extension.

For unpublished grammar work, you can point at a local checkout instead:

```toml
[grammars.surrealql]
repository = 'file:///absolute/path/to/surrealql-tree-sitter'
rev = '<commit-sha-from-that-checkout>'
```

If dev install fails with `failed to compile grammar 'surrealql'`, delete the cached grammar checkout and retry:

```sh
rm -rf ~/Library/Application\ Support/Zed/extensions/installed/surrealdb-surrealql-zed-dev/grammars/surrealql
```

Then run `zed: install dev extension` again.

### Language server (diagnostics)

Syntax diagnostics come from `surrealql-language-server`. The extension prefers a binary on your `PATH` (for example from `cargo install --path ../surrealql-language-server --force`) and otherwise downloads the latest GitHub pre-release.

To test grammar fixes such as prefix `!` in `WHERE` clauses before a new LSP release is published:

```sh
cd ../surrealql-tree-sitter && bun run gen
cd ../surrealql-language-server && cargo install --path . --force
```

Ensure `~/.cargo/bin` (or wherever the binary lands) is on the PATH Zed inherits when it launches.

### Validating queries

With a checkout of `surrealql-tree-sitter` on the pinned revision:

```sh
cd ../surrealql-tree-sitter
./node_modules/.bin/tree-sitter query -p . ../surrealql-zed/languages/surql/highlights.scm ../surrealql-zed/test.surql
```

## License

Apache-2.0 — see [LICENSE](LICENSE).
