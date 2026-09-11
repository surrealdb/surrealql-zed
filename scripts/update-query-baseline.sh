#!/usr/bin/env bash
#
# Regenerate test/fixtures/query-baseline.txt against the grammar revision
# currently pinned in extension.toml. Run this after editing a query file or
# repinning the grammar, then commit the diff.
#
# Requires the `tree-sitter` CLI on PATH (the grammar repo generates with
# 0.26.8; `npm i -g tree-sitter-cli@0.26.8`).

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
grammar="$(mktemp -d)"
trap 'rm -rf "$grammar"' EXIT

"$root/scripts/fetch-grammar.sh" "$grammar"
python3 "$root/scripts/check-queries.py" "$grammar" --update
