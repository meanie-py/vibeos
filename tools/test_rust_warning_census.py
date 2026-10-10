"""The warnings census: lint attribution, tiering, and coverage honesty."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import rust_warning_census as c

# The real shape, from a vibeos-cli build log: only the FIRST occurrence of a lint in a
# crate carries its `#[warn(..)]` note.
LOG = """\
   Compiling kernel32-rust v0.1.0 (/tmp/vibeos/sdk/lib/rust/kernel32-rust)
warning: comparison is useless due to type limits
    --> kernel32-rust/src/client/path.rs:1606:28
     |
1606 |                         if Result > 0xFFFFFFFF {
     |                            ^^^^^^^^^^^^^^^^^^^
     |
     = note: `#[warn(unused_comparisons)]` on by default

warning: comparison is useless due to type limits
    --> kernel32-rust/src/client/path.rs:1669:32
     |
1669 | ...                   if Result > 0xFFFFFFFF {
     |                          ^^^^^^^^^^^^^^^^^^^

warning: path statement with no effect
   --> kernel32-rust/src/client/loader.rs:141:5
    |
141 |     UNIMPLEMENTED;
    |     ^^^^^^^^^^^^^^
    |
    = note: `#[warn(path_statements)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `FOO`
 --> kernel32-rust/src/a.rs:1:5
  |
1 | use x::FOO;
  |     ^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused imports: `BAR`, `BAZ`, and `QUX`
 --> kernel32-rust/src/b.rs:2:5
  |
2 | use x::{BAR, BAZ, QUX};
  |         ^^^

warning: `kernel32-rust` (lib) generated 5 warnings
"""


def _sites(text: str = LOG) -> list[c.Site]:
    sites, _ = c.parse_log(text)
    c.attribute(sites)
    return sites


def test_an_unannotated_block_takes_the_lint_its_template_was_taught():
    """rustc names the lint once per crate; the second `Result > 0xFFFFFFFF` has no note
    and must still be counted as unused_comparisons, not dropped or left unnamed."""
    got = [(s.path.rsplit("/", 1)[-1], s.line, s.lint) for s in _sites()]
    assert ("path.rs", 1669, "unused_comparisons") in got


def test_a_group_note_names_the_specific_lint_not_the_group():
    """`#[warn(path_statements)]` (part of `#[warn(unused)]`) is path_statements --
    which is LOGIC -- not `unused`, which would read as noise."""
    s = next(s for s in _sites() if s.line == 141)
    assert s.lint == "path_statements"
    assert c.tier_of(s.lint) == "LOGIC"


def test_a_list_and_a_plural_fold_into_one_template():
    """`unused imports: `A`, `B`, and `C`` is the same shape as `unused import: `D``."""
    s = next(s for s in _sites() if s.path.endswith("b.rs"))
    assert s.lint == "unused_imports"


def test_the_crate_summary_is_a_claim_not_a_site():
    sites, claimed = c.parse_log(LOG)
    assert claimed == {"kernel32-rust": 5}
    assert all("generated" not in s.msg for s in sites)
    assert len(sites) == 5


def test_a_template_taught_two_lints_attributes_neither():
    """Guessing between two lints a template was taught would be a confident wrong
    answer, so the site stays unattributed -- and therefore UNCLASSIFIED."""
    log = (
        "warning: same words\n --> a-rust/src/x.rs:1:1\n  |\n"
        "  = note: `#[warn(dead_code)]` on by default\n\n"
        "warning: same words\n --> b-rust/src/x.rs:1:1\n  |\n"
        "  = note: `#[warn(invalid_value)]` on by default\n\n"
        "warning: same words\n --> c-rust/src/x.rs:1:1\n  |\n"
    )
    sites = _sites(log)
    assert next(s for s in sites if s.path.startswith("c-rust")).lint is None


def test_an_unknown_lint_is_unclassified_never_noise():
    """A rustc upgrade brings new lints unranked; defaulting one to harmless is how a
    real one would hide."""
    assert c.tier_of("some_lint_from_rustc_2027") == "UNCLASSIFIED"


def test_the_tiers_are_disjoint():
    assert not (c.UB & c.LOGIC) and not (c.UB & c.NOISE) and not (c.LOGIC & c.NOISE)


def test_the_same_site_on_two_arches_is_one_site_seen_twice():
    a, b = _sites(), _sites()
    merged = c.merge({"i386": a, "arm64": b})
    assert len(merged) == len(a)
    assert all(s.arches == {"i386", "arm64"} for s in merged)


def test_the_arch_is_read_from_the_run_dir():
    log = Path("/tmp/vibeos-cli/20261008-011558-ngiwyrng-build-arm64/full.log")
    assert c.arch_of(log) == "arm64"


def _diag(path: str, line: int, lint: str, msg: str) -> str:
    import json

    return json.dumps(
        {
            "$message_type": "diagnostic",
            "message": msg,
            "code": {"code": lint, "explanation": None},
            "level": "warning",
            "spans": [
                {
                    "file_name": path,
                    "line_start": line,
                    "column_start": 5,
                    "is_primary": True,
                    "text": [{"text": "    x = 0;"}],
                }
            ],
            "children": [],
        }
    )


def _unit(target: Path, crate: str, h: str, deps: list[str], output: list[str] | None):
    fp = target / ".fingerprint" / f"{crate}-{h}"
    fp.mkdir(parents=True)
    lib = crate.replace("-", "_")
    (fp / f"lib-{lib}.json").write_text("{}")
    if output is not None:
        (fp / f"output-lib-{lib}").write_text("\n".join(output) + "\n")
    (target / "deps").mkdir(exist_ok=True)
    (target / "deps" / f"{lib}-{h}.d").write_text(
        f"/tmp/vibeos/build-i386/x/deps/{lib}-{h}.d: {' '.join(deps)}\n\n"
    )
    (fp / "invoked.timestamp").write_text("")
    return fp


def test_a_build_dir_reports_fresh_units_and_refuses_edited_ones(tmp_path):
    import os

    repo = tmp_path / "repo"
    src = repo / "sdk" / "lib" / "rust"
    for f in ("good-rust/src/lib.rs", "clean-rust/src/lib.rs", "edited-rust/src/lib.rs"):
        (src / f).parent.mkdir(parents=True, exist_ok=True)
        (src / f).write_text("")
        os.utime(src / f, (1000, 1000))
    target = repo / "build-i386" / "t" / "i686" / "release"
    good = _unit(
        target,
        "good-rust",
        "0" * 16,
        ["good-rust/src/lib.rs"],
        [
            _diag(
                "good-rust/src/lib.rs",
                7,
                "unused_assignments",
                "value assigned to `x` is never read",
            )
        ],
    )
    clean = _unit(target, "clean-rust", "1" * 16, ["clean-rust/src/lib.rs"], None)
    edited = _unit(target, "edited-rust", "2" * 16, ["edited-rust/src/lib.rs"], [])
    for fp in (good, clean, edited):
        os.utime(fp / "invoked.timestamp", (2000, 2000))
    os.utime(src / "edited-rust/src/lib.rs", (3000, 3000))  # edited after its build

    units = {u.crate: u for u in c.read_build_dir(repo / "build-i386", repo)}
    assert [(s.path, s.line, s.lint) for s in units["good-rust"].sites] == [
        ("good-rust/src/lib.rs", 7, "unused_assignments")
    ]
    assert units["clean-rust"].sites == [] and not units["clean-rust"].stale
    fresh, stale = c.coverage(list(units.values()))
    assert fresh == {"good-rust", "clean-rust"}
    assert "changed since the build" in stale["edited-rust"]
    assert c.main([str(repo / "build-i386")]) == 3  # refuses a total missing a crate
    os.utime(src / "edited-rust/src/lib.rs", (1000, 1000))
    assert c.main([str(repo / "build-i386")]) == 0
