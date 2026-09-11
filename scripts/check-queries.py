#!/usr/bin/env python3
"""Check every query file against the pinned grammar, by capture counts.

`tree-sitter query --quiet` only proves a query compiles. A query that
compiles but matches nothing compiles just as happily, so a pattern that
goes stale against a new grammar revision stays green. This compares the
captures each query actually produces over test/fixtures/highlight-sample.surql
against a committed baseline, and fails on any drift.

Counts are per capture name, not per file: a file total can hold steady
while `@keyword` quietly becomes `@variable`, and that is exactly the
regression worth catching.

Also asserts the fixture parses with no ERROR or MISSING nodes, which turns
a grammar pin that cannot parse ordinary SurrealQL into a red build.

    scripts/check-queries.py <grammar-dir>
    scripts/check-queries.py <grammar-dir> --update
"""

import argparse
import pathlib
import re
import subprocess
import sys
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parent.parent
QUERY_DIR = ROOT / "languages" / "surql"
FIXTURE = ROOT / "test" / "fixtures" / "highlight-sample.surql"
BASELINE = ROOT / "test" / "fixtures" / "query-baseline.txt"

# tree-sitter prints two shapes, and dropping the second silently undercounts
# every capture that spans more than one line:
#
#     capture: 3 - punctuation.bracket, start: (0, 0), end: (0, 1), text: `[`
#     capture: comment.block, start: (11, 0), end: (12, 21)
CAPTURE_RE = re.compile(r"^\s*capture: (?:\d+ - )?([A-Za-z0-9_.]+),")

HEADER = """\
# Capture counts per query file, produced by scripts/check-queries.py over
# test/fixtures/highlight-sample.surql against the grammar revision pinned in
# extension.toml.
#
# Regenerate with scripts/update-query-baseline.sh and commit the diff. A
# change here should always be explained by a grammar repin or a query edit;
# an unexplained one is the regression this file exists to catch.
#
# <query file> <capture name> <count>
"""


def resolve_grammar(raw):
    """Validate the caller's grammar directory before it reaches a subprocess.

    Everything below shells out to `tree-sitter -p <grammar>`, so this is the
    one place untrusted input enters. Resolve it, require a real directory that
    actually holds a tree-sitter grammar, and pass the resolved path onward.
    """
    grammar = pathlib.Path(raw).resolve()
    if not grammar.is_dir():
        raise SystemExit(f"{raw}: not a directory")
    if not (grammar / "grammar.js").is_file():
        raise SystemExit(f"{raw}: no grammar.js, is this a tree-sitter grammar?")
    return grammar


def tree_sitter(command, grammar, *args):
    """Run a tree-sitter subcommand. Never uses a shell."""
    return subprocess.run(
        ["tree-sitter", command, "-p", str(grammar), *args],
        capture_output=True,
        text=True,
        shell=False,
    )


def parse_errors(grammar):
    """Count ERROR and MISSING nodes in the fixture's parse tree."""
    # `tree-sitter parse` exits non-zero when the tree has errors and still
    # prints the tree, so read both streams rather than branching on the code.
    result = tree_sitter("parse", grammar, str(FIXTURE))
    return len(re.findall(r"\b(?:ERROR|MISSING)\b", result.stdout + result.stderr))


def collect(grammar):
    """Map each query file to its capture-name counts."""
    counts = {}
    for query in sorted(QUERY_DIR.glob("*.scm")):
        result = tree_sitter("query", grammar, str(query), str(FIXTURE))
        if result.returncode != 0:
            print(
                f"::error file={query.relative_to(ROOT)}::does not compile: "
                f"{result.stderr.strip()}"
            )
            return None
        captures = Counter(
            match.group(1)
            for line in result.stdout.splitlines()
            if (match := CAPTURE_RE.match(line))
        )
        counts[query.name] = captures
    return counts


def render(counts):
    lines = [HEADER]
    for name, captures in sorted(counts.items()):
        for capture, count in sorted(captures.items()):
            lines.append(f"{name} {capture} {count}\n")
    return "".join(lines)


def load_baseline():
    counts = {}
    for line in BASELINE.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        name, capture, count = line.split()
        counts.setdefault(name, Counter())[capture] = int(count)
    return counts


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("grammar")
    parser.add_argument("--update", action="store_true")
    args = parser.parse_args()

    grammar = resolve_grammar(args.grammar)
    errors = parse_errors(grammar)
    counts = collect(grammar)
    if counts is None:
        return 1

    if args.update:
        BASELINE.write_text(render(counts))
        total = sum(sum(c.values()) for c in counts.values())
        print(f"wrote {BASELINE.relative_to(ROOT)}: {total} captures, {errors} parse errors")
        if errors:
            print(
                "::warning::the fixture does not parse cleanly at this grammar "
                "revision; check-queries.py will fail until that is resolved"
            )
        return 0

    failed = False

    if errors:
        print(
            f"::error file=extension.toml::the pinned grammar leaves {errors} "
            f"ERROR/MISSING nodes in {FIXTURE.relative_to(ROOT)}"
        )
        failed = True

    expected = load_baseline()
    for name in sorted(set(expected) | set(counts)):
        want = expected.get(name, Counter())
        got = counts.get(name, Counter())
        if want == got:
            print(f"ok   {name} ({sum(got.values())} captures)")
            continue
        failed = True
        for capture in sorted(set(want) | set(got)):
            if want.get(capture, 0) != got.get(capture, 0):
                print(
                    f"::error file=languages/surql/{name}::@{capture}: "
                    f"expected {want.get(capture, 0)}, got {got.get(capture, 0)}"
                )

    if failed:
        print(
            "::notice::if this drift is intended, run "
            "scripts/update-query-baseline.sh and commit the result"
        )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
