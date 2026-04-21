# SurrealQL for Zed

[SurrealQL](https://surrealdb.com/docs/surrealql) language support for the [Zed](https://zed.dev) editor.

## Features

- Syntax highlighting for SurrealQL (`.surql` files)
- Bracket matching and auto-close
- Embedded JavaScript highlighting inside scripting functions
- Tree-sitter grammar via [`surrealdb/surrealql-tree-sitter`](https://github.com/surrealdb/surrealql-tree-sitter)

## Installation

Search for **SurrealQL** in Zed's extension marketplace (`zed: extensions`) and install.

## Usage

Open any `.surql` file and syntax highlighting is applied automatically.

Embedded JavaScript in scripting functions is highlighted as JavaScript:

```surql
DEFINE FUNCTION fn::greet($name: string) {
    return function($name) {
        return `Hello, ${arguments[0]}!`;
    };
};
```

## Development

### Prerequisites

- [Zed](https://zed.dev) (with extension development support)
- [Rust](https://rustup.rs) toolchain

### Local setup

```sh
git clone https://github.com/surrealdb/surrealql-zed
```

Then in Zed, open the command palette and run:

```
zed: install dev extension
```

Point it at the cloned directory.

### Grammar

The tree-sitter grammar is fetched automatically from [`surrealdb/surrealql-tree-sitter`](https://github.com/surrealdb/surrealql-tree-sitter) at the commit pinned in `extension.toml`. To update the grammar, change the `commit` SHA there.
