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

# `mktemp -d` creates the directory, and fetch-grammar.sh refuses to `rm -rf` a
# directory that is not already a grammar checkout. Point it one level in, at a
# path that does not exist yet.
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

"$root/scripts/fetch-grammar.sh" "$workdir/grammar"
python3 "$root/scripts/check-queries.py" "$workdir/grammar" --update
