"""The inert-content predicate that both docs-only exemptions depend on.

Two callers grant a pass on it — the pre-push hook (no gate stamp needed) and `land`'s
lint preflights (nothing to lint). Every test here is about the UNSAFE direction: the
cost of wrongly gating a typo fix is minutes, and the cost of wrongly exempting live
source is an ungated tree on master.
"""

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from docs_only import INERT_SUFFIXES, docs_only, is_inert

REPO = Path(__file__).resolve().parents[1]


def test_plain_documentation_qualifies():
    assert docs_only(["docs/engineering/notes.md", "README.md", "a/b/c.txt"])


def test_a_dot_rs_UNDER_docs_does_not_qualify():
    """The hole the old `docs/**` rule had, and the reason this is suffix-based.

    `docs/**` judged where a file LIVES. This tree really does keep three `.rs` and four
    `.json` under `docs/`, so the old predicate called a Rust file docs-only. Nothing
    compiles those today — which is a fact about the tree this afternoon, not about the
    rule, and an exemption's whole job is to stay safe without being re-examined.
    """
    assert not docs_only(["docs/engineering/rtl-threadpool-raw-rustconv/threadpool.raw.rs"])
    assert not docs_only(["docs/engineering/rustconv-anchor-survey.json"])


def test_one_source_file_among_many_docs_gates_the_whole_delta():
    """Explicit owner instruction, and true even when the source change is a comment.

    Proving a comment-only edit is inert means parsing the language; a predicate that can
    be wrong in the unsafe direction is worth much less than one that occasionally makes
    you gate a typo fix.
    """
    assert not docs_only([f"docs/page{i}.md" for i in range(100)] + ["tools/x.py"])
    assert not docs_only(["README.md", "sdk/lib/rust/foo-rust/src/lib.rs"])


def test_an_empty_delta_is_not_docs_only():
    """ "Nothing changed" must not read as "safe to exempt" — a caller that failed to
    compute a delta would otherwise be handed the pass."""
    assert not docs_only([])
    assert not docs_only(["", "   "])


def test_config_and_extensionless_files_are_not_inert():
    for path in ("Makefile", "LICENSE", "Cargo.toml", "settings.yaml", "x.json", "d.svg"):
        assert not is_inert(path), f"{path} must not be treated as inert"


def test_suffix_match_is_case_insensitive_but_not_substring():
    assert is_inert("NOTES.MD") and is_inert("docs/A.Md")
    # the suffix must END the name: a file merely CONTAINING ".md" is not a document
    assert not is_inert("tools/mdparser.py")
    assert not is_inert("x.md.py")


def test_both_callers_use_THIS_predicate_and_not_a_private_copy():
    """The property that makes one definition worth having. If either caller reimplements
    it, they can disagree, and the hook disagreeing with the preflight is how an ungated
    tree reaches master while a lane sees green locally."""
    hook = (REPO / "tools" / "git-hooks" / "pre-push").read_text()
    land = (REPO / "tests" / "qemu" / "cli" / "land.py").read_text()
    for name, text in (("pre-push", hook), ("land.py", land)):
        assert "docs_only" in text, f"{name} must import the shared predicate"
        assert 'startswith("docs/")' not in text, f"{name} still has a private docs/ rule"


def test_the_hook_falls_back_to_NO_exemption_when_the_import_fails():
    """Fail closed. Run the hook's own fallback shape: an unimportable predicate must make
    every path non-inert, so the push demands a stamp rather than sailing through."""
    src = (REPO / "tools" / "git-hooks" / "pre-push").read_text()
    assert "except Exception" in src and "return False" in src, "fallback must exist"
    proc = subprocess.run(
        [
            sys.executable,
            "-c",
            "import sys\n"
            "sys.path.insert(0, '/nonexistent-so-the-import-fails')\n"
            "try:\n"
            "    from docs_only import is_inert\n"
            "except Exception:\n"
            "    def is_inert(_p):\n"
            "        return False\n"
            "print(is_inert('README.md'))\n",
        ],
        capture_output=True,
        text=True,
        cwd="/",
    )
    assert proc.stdout.strip() == "False", "a failed import must grant no exemption"


def test_the_inert_list_stays_small_and_deliberate():
    """A guard rail on the guard rail. Every suffix added here widens an exemption that
    skips a gate, so growing this list should be a decision someone makes on purpose."""
    assert {".md", ".txt", ".rst", ".png", ".jpg", ".jpeg", ".gif"} >= INERT_SUFFIXES
    assert all(s.startswith(".") and s.islower() for s in INERT_SUFFIXES)


# ── the QUEUE's use of it: skipping a whole gate, not just a lint ────────────────────────


def _repo_with_master(tmp_path):
    """A worktree with an origin/master to diff against, shaped like the integration tree."""
    r = tmp_path / "wt"
    r.mkdir()
    for a in (["init", "-q"], ["config", "user.email", "t@t"], ["config", "user.name", "t"]):
        subprocess.run(["git", *a], cwd=r, check=True, capture_output=True)
    (r / "seed.txt").write_text("x\n")
    # The predicate is imported from <wt>/tools, mirroring the real integration worktree.
    # It goes in the BASE commit: if it landed in the delta instead it would show up as a
    # non-inert path and every case here would "pass" for the wrong reason.
    (r / "tools").mkdir()
    (r / "tools" / "docs_only.py").write_text((REPO / "tools" / "docs_only.py").read_text())
    subprocess.run(["git", "add", "-A"], cwd=r, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "base"], cwd=r, check=True, capture_output=True)
    subprocess.run(
        ["git", "update-ref", "refs/remotes/origin/master", "HEAD"],
        cwd=r,
        check=True,
        capture_output=True,
    )
    return r


def _commit(repo, *paths):
    for rel in paths:
        f = repo / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text("content\n")
    subprocess.run(["git", "add", "-A"], cwd=repo, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "batch"], cwd=repo, check=True, capture_output=True)


def test_queue_skips_the_gate_for_an_all_documentation_batch(tmp_path):
    from tests.qemu.cli.queue import inert_only_batch

    wt = _repo_with_master(tmp_path)
    _commit(wt, "docs/a.md", "NOTES.txt")
    assert inert_only_batch(wt) == [], "a docs batch must not spend 3 builds and 4 boots"


def test_queue_gates_when_ONE_lane_in_the_batch_carries_code(tmp_path):
    """The unsafe direction, and the reason this is measured on the MERGED tree: three docs
    lanes plus one code lane is a code batch, and no single lane's own delta would say so."""
    from tests.qemu.cli.queue import inert_only_batch

    wt = _repo_with_master(tmp_path)
    _commit(wt, "docs/a.md", "docs/b.md", "docs/c.md", "sdk/lib/rust/x-rust/src/lib.rs")
    assert inert_only_batch(wt) == ["sdk/lib/rust/x-rust/src/lib.rs"]


def test_queue_gates_a_settled_lockfile_no_lane_declared(tmp_path):
    """`settle_cargo_locks` commits a rewritten `Cargo.lock` INTO the batch tip, so it appears
    in no lane's delta. Measuring against the merged tree is what catches it."""
    from tests.qemu.cli.queue import inert_only_batch

    wt = _repo_with_master(tmp_path)
    _commit(wt, "docs/a.md")
    _commit(wt, "Cargo.lock")
    assert inert_only_batch(wt) == ["Cargo.lock"]


def test_queue_gates_when_it_cannot_tell(tmp_path):
    """Fail closed, twice over: an empty delta and a missing predicate must both gate."""
    from tests.qemu.cli.queue import inert_only_batch

    wt = _repo_with_master(tmp_path)
    assert inert_only_batch(wt), "an empty delta is not a docs-only batch"
    _commit(wt, "docs/a.md")
    (wt / "tools" / "docs_only.py").unlink()
    assert inert_only_batch(wt), "no predicate must mean no exemption"


def test_the_hook_exempts_docs_WITHOUT_any_gate_stamp(tmp_path):
    """The bug that halted the train for two hours, pinned.

    The hook used to `fail("no fresh green full-gate stamp")` before it ever reached the
    docs-only exemption, which made the exemption unreachable on an empty stamp set. That
    was harmless only while the conductor gated every batch — a gate mints a stamp. The
    moment the queue learned to SKIP the gate for a documentation-only batch, the train
    produced no stamp, the hook refused the push it was about to exempt, and one lane sat
    queued for 122 minutes.

    The remote-master exemption needs no stamp by construction: every non-docs path in
    origin/master was gated when it landed. So the ordering is the whole fix, and this
    test is here to keep the early bail from being reinstated.
    """
    src = (REPO / "tools" / "git-hooks" / "pre-push").read_text()
    # Structural, not textual: every `if not valid_trees:` must be INDENTED, i.e. inside
    # the per-update loop that has already tried the exemption. A top-level one runs
    # before the loop and short-circuits it. Asserting on message wording instead would
    # pass the moment someone rewraps an f-string, which is how the first version of this
    # test managed to fail against the correct fix.
    tops = [
        ln
        for ln in src.splitlines()
        if ln.rstrip().endswith("if not valid_trees:") and not ln.startswith((" ", "\t"))
    ]
    assert not tops, (
        "a top-level `if not valid_trees:` bails before the docs-only exemption is "
        f"reached, so a documentation push with no fresh gate is refused: {tops}"
    )
