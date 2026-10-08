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
