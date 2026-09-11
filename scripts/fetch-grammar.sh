#!/usr/bin/env bash
#
# Fetch the tree-sitter grammar at the revision pinned in extension.toml, the
# same way Zed does. Shared by CI and scripts/update-query-baseline.sh so the
# two cannot drift.
#
#     scripts/fetch-grammar.sh <destination>

set -euo pipefail

dest="${1:?usage: fetch-grammar.sh <destination>}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Read with a regex rather than tomllib, so this runs on the Python 3.9 that
# ships with macOS as well as on CI.
pin="$(cd "$root" && python3 - <<'PY'
import re
import sys

text = open("extension.toml").read()
section = re.search(
    r"^\[grammars\.surrealql\]\s*$(.*?)(?=^\[|\Z)", text, re.S | re.M
)
if section is None:
    sys.exit("extension.toml has no [grammars.surrealql] section")

fields = dict(re.findall(r"^(\w+)\s*=\s*['\"]([^'\"]+)", section.group(1), re.M))
missing = {"repository", "rev"} - fields.keys()
if missing:
    sys.exit(f"[grammars.surrealql] is missing {', '.join(sorted(missing))}")

print(fields["repository"], fields["rev"])
PY
)"
read -r repository rev <<<"$pin"

rm -rf "$dest"
mkdir -p "$dest"
git -C "$dest" init --quiet
git -C "$dest" remote add origin "$repository"

# Zed fetches grammars with `git fetch --depth 1 origin <rev>`, so this also
# proves the pinned revision is still reachable from the remote. A rev that
# only exists on a since-deleted branch fails here rather than on every new
# install.
git -C "$dest" fetch --quiet --depth 1 origin "$rev"
git -C "$dest" checkout --quiet "$rev"

echo "fetched $repository at $rev into $dest"
