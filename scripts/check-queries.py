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

Two things are checked beyond the counts:

- the fixture parses with no ERROR or MISSING node, so a grammar pin that
  cannot parse ordinary SurrealQL is a red build;
- every `@name` a query file declares appears in the baseline, so a capture
  added later that matches nothing is reported rather than silently absent.

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

TREE_SITTER_HINT = (
    "tree-sitter not found on PATH. Install the version the grammar repo "
    "generates with: npm i -g tree-sitter-cli@0.26.8"
)

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
    """Resolve the grammar directory, with a clear message when it is wrong.

    This is not a security boundary. Every call below passes a list argv with
    shell=False, so nothing here is injectable, and in CI the path is the
    checkout this repo's own workflow just made. It earns its place by turning
    a wrong path into one sentence instead of seven identical tree-sitter
    failures.
    """
    grammar = pathlib.Path(raw).resolve()
    if not grammar.is_dir():
        raise SystemExit(f"{raw}: not a directory")
    if not (grammar / "grammar.js").is_file():
        raise SystemExit(f"{raw}: no grammar.js, is this a tree-sitter grammar?")
    return grammar


def tree_sitter(command, grammar, *args):
    """Run a tree-sitter subcommand. Never uses a shell."""
    try:
        return subprocess.run(
            ["tree-sitter", command, "-p", str(grammar), *args],
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        raise SystemExit(TREE_SITTER_HINT)


def parse_errors(grammar):
    """Count ERROR and MISSING nodes in the fixture's parse tree."""
    # `tree-sitter parse` exits non-zero when the tree has errors and still
    # prints the tree, so read both streams rather than branching on the code.
    result = tree_sitter("parse", grammar, str(FIXTURE))
    return len(re.findall(r"\b(?:ERROR|MISSING)\b", result.stdout + result.stderr))


def declared_captures(query):
    """The `@name`s a query file declares, ignoring `;` comments."""
    source = re.sub(r";.*", "", query.read_text())
    return set(re.findall(r"@([A-Za-z0-9_.]+)", source))


def collect(grammar):
    """Map each query file to its capture-name counts.

    Reports every query that fails to compile before giving up, so one run
    surfaces all of them rather than one per run.
    """
    counts = {}
    failed = False
    for query in sorted(QUERY_DIR.glob("*.scm")):
        result = tree_sitter("query", grammar, str(query), str(FIXTURE))
        if result.returncode != 0:
            print(
                f"::error file={query.relative_to(ROOT)}::does not compile: "
                f"{result.stderr.strip()}"
            )
            failed = True
            continue
        counts[query.name] = Counter(
            match.group(1)
            for line in result.stdout.splitlines()
            if (match := CAPTURE_RE.match(line))
        )
    return None if failed else counts


def unmatched_declarations(counts):
    """Capture names a query declares that the fixture never reaches.

    The baseline only stores nonzero counts, so without this a capture added
    later that matches nothing would be absent rather than reported.
    """
    unmatched = {}
    for query in sorted(QUERY_DIR.glob("*.scm")):
        missing = declared_captures(query) - set(counts.get(query.name, ()))
        if missing:
            unmatched[query.name] = sorted(missing)
    return unmatched


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


def report_unmatched(unmatched, level):
    for name, captures in unmatched.items():
        for capture in captures:
            print(
                f"::{level} file=languages/surql/{name}::@{capture} is declared "
                f"but never matched by {FIXTURE.name}; extend the fixture or "
                f"drop the pattern"
            )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("grammar")
    parser.add_argument("--update", action="store_true")
    args = parser.parse_args()

    grammar = resolve_grammar(args.grammar)
    errors = parse_errors(grammar)
    counts = collect(grammar)

    if args.update:
        if counts is None:
            return 1
        BASELINE.write_text(render(counts))
        total = sum(sum(c.values()) for c in counts.values())
        print(
            f"wrote {BASELINE.relative_to(ROOT)}: {total} captures, "
            f"{errors} parse errors"
        )
        if errors:
            print(
                "::warning::the fixture does not parse cleanly at this grammar "
                "revision; check-queries.py will fail until that is resolved"
            )
        report_unmatched(unmatched_declarations(counts), "warning")
        return 0

    failed = False

    # Reported before bailing on a failed collect, so a grammar pin that cannot
    # parse the fixture does not read as a query problem.
    if errors:
        print(
            f"::error file=extension.toml::the pinned grammar leaves {errors} "
            f"ERROR/MISSING nodes in {FIXTURE.relative_to(ROOT)}"
        )
        failed = True

    if counts is None:
        return 1

    unmatched = unmatched_declarations(counts)
    if unmatched:
        report_unmatched(unmatched, "error")
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
