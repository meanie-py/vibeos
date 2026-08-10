"""One C++ TU needs several translate runs, and they all derive the same output path.

`--free-functions` covers a TU's free helpers; `--cpp-com --class X` covers one class. Both
name their output after the TU's stem, so the second run used to overwrite the first with no
diagnostic at all -- it printed "Assembled to ..." and looked like a success. That happened on
folders/CNetFolder.cpp: the CNetFolderEnum assembly was replaced by the free-function assembly,
and nothing said so. The paid artifacts survive in .responses/, so the cost is time, not money,
but the failure is silent and the file it leaves behind is missing symbols the caller needs.
"""

from __future__ import annotations

import pytest

from rustconv.assembler import (
    ASSEMBLY_MARKER,
    AssemblyClobberError,
    _c_source_for_slice,
    _guard_assembly_target,
)


def _write(tmp_path, name, selector, body="pub fn f() {}\n"):
    p = tmp_path / name
    head = f"{ASSEMBLY_MARKER}{selector}\n" if selector else ""
    p.write_text(head + body)
    return p


def test_refuses_a_different_slice_of_the_same_tu(tmp_path):
    p = _write(tmp_path, "CNetFolder.rs", "class:CNetFolderEnum")
    with pytest.raises(AssemblyClobberError) as exc:
        _guard_assembly_target(p, "free-functions")
    msg = str(exc.value)
    assert "class:CNetFolderEnum" in msg
    assert "free-functions" in msg
    # The message has to say what to DO, or it just relocates the confusion.
    assert "-o" in msg


def test_allows_rerunning_the_same_slice(tmp_path):
    p = _write(tmp_path, "CNetFolder.rs", "class:CNetFolderEnum")
    _guard_assembly_target(p, "class:CNetFolderEnum")


def test_allows_a_fresh_path(tmp_path):
    _guard_assembly_target(tmp_path / "does_not_exist.rs", "free-functions")


def test_refuses_a_partial_slice_over_an_unmarked_file(tmp_path):
    """An unmarked file is the DANGEROUS case, not the safe one.

    dialogs/drvdefext.rs held a fully translated CDrvDefExt class with no marker, and a
    --free-functions run replaced all 1,742 lines of it. Nothing can tell from the outside
    what an unmarked file covers, so a partial slice has to ask.
    """
    p = _write(tmp_path, "drvdefext.rs", selector="")
    with pytest.raises(AssemblyClobberError) as exc:
        _guard_assembly_target(p, "free-functions")
    assert "-o" in str(exc.value)


def test_whole_tu_may_overwrite_an_unmarked_file(tmp_path):
    """A whole-TU run is not a partial slice, so it cannot drop part of the TU."""
    p = _write(tmp_path, "legacy.rs", selector="")
    _guard_assembly_target(p, "whole-tu")


def test_no_selector_disables_the_guard(tmp_path):
    """Targets that do not pass a selector keep their old behaviour rather than half-guarding."""
    p = _write(tmp_path, "CNetFolder.rs", "class:CNetFolderEnum")
    _guard_assembly_target(p, "")


# ── the embedded C block is a LINT ORACLE, not decoration ──────────────────────────────────


class _Fn:
    def __init__(self, name, line_start, line_end, cpp_class=None):
        self.name = name
        self.line_start = line_start
        self.line_end = line_end
        self.cpp_class = cpp_class


class _Scan:
    def __init__(self, functions):
        self.functions = functions


C_SRC = "\n".join(
    [
        "#include <precomp.h>",  # 1  file scope
        "static int g_state;",  # 2  file scope
        "void kept(void) {",  # 3
        "    x &= ~KEPT_FLAG;",  # 4
        "}",  # 5
        "void owned_elsewhere(void) {",  # 6
        "    rgfInOut &= ~SFGAO_VALIDATE;",  # 7
        "}",  # 8
    ]
)
SCAN = _Scan([_Fn("kept", 3, 5), _Fn("owned_elsewhere", 6, 8)])


def test_partial_slice_drops_c_owned_by_a_sibling_module():
    """This is the bug: CNetFolder.rs was reported as DROPPING a mask living in net_folder.rs."""
    out, elided = _c_source_for_slice(C_SRC, SCAN, {"kept"}, "free-functions")
    assert "KEPT_FLAG" in out
    assert "SFGAO_VALIDATE" not in out
    assert elided == 3
    # File scope belongs to every slice -- dropping it would lose the includes and statics.
    assert "#include <precomp.h>" in out
    assert "static int g_state;" in out


def test_whole_tu_embeds_everything():
    out, elided = _c_source_for_slice(C_SRC, SCAN, {"kept", "owned_elsewhere"}, "whole-tu")
    assert out == C_SRC
    assert elided == 0


def test_a_cpp_method_worker_is_matched_by_its_prefixed_name():
    scan = _Scan([_Fn("Method", 3, 5, cpp_class="CFoo"), _Fn("owned_elsewhere", 6, 8)])
    out, elided = _c_source_for_slice(C_SRC, scan, {"CFoo_Method"}, "class:CFoo")
    assert "KEPT_FLAG" in out
    assert "SFGAO_VALIDATE" not in out
    assert elided == 3


def test_no_match_falls_back_to_the_whole_source():
    """Eliding everything would be worse than eliding nothing -- degrade to today's behaviour."""
    out, elided = _c_source_for_slice(C_SRC, SCAN, {"not_a_function_here"}, "free-functions")
    assert out == C_SRC
    assert elided == 0


# ---------------------------------------------------------------------------
# A whole-TU run over INTEGRATED code
# ---------------------------------------------------------------------------


def _git_repo(tmp_path):
    import subprocess

    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    subprocess.run(["git", "config", "user.email", "t@t"], cwd=tmp_path, check=True)
    subprocess.run(["git", "config", "user.name", "t"], cwd=tmp_path, check=True)
    return tmp_path


def test_whole_tu_refuses_to_overwrite_committed_integrated_code(tmp_path):
    """The measured loss: a re-roll reverts every repair made after integration.

    browser-rust's re-translation re-forked `extern "system"` into `#[cfg]` ABI twins, put
    `&ServiceStatus` back on a `static mut`, and renamed `MIDL_user_allocate` to the C's
    lowercase spelling — the marshaller binds the capitalised one. The model only ever sees
    the C, so none of those repairs can survive a re-emit, and none of them looks wrong in
    the output.
    """
    import subprocess

    import pytest

    from rustconv.assembler import AssemblyClobberError, _guard_assembly_target

    repo = _git_repo(tmp_path)
    out = repo / "shipped.rs"
    out.write_text("// rustconv-assembly: whole-tu\npub fn repaired() {}\n")
    subprocess.run(["git", "add", "shipped.rs"], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "integrate"], cwd=repo, check=True)

    with pytest.raises(AssemblyClobberError, match="INTEGRATED code"):
        _guard_assembly_target(out, "whole-tu")
    # The escape hatch is explicit, never implied.
    _guard_assembly_target(out, "whole-tu", reassemble=True)


def test_an_in_flight_grind_is_still_free_to_reassemble(tmp_path):
    """Initial translation re-assembles the same TU repeatedly; only COMMITTED code is shielded."""
    import subprocess

    from rustconv.assembler import _guard_assembly_target

    repo = _git_repo(tmp_path)
    untracked = repo / "fresh.rs"
    untracked.write_text("// rustconv-assembly: whole-tu\npub fn a() {}\n")
    _guard_assembly_target(untracked, "whole-tu")  # untracked -> allowed

    dirty = repo / "dirty.rs"
    dirty.write_text("// rustconv-assembly: whole-tu\npub fn b() {}\n")
    subprocess.run(["git", "add", "dirty.rs"], cwd=repo, check=True)
    subprocess.run(["git", "commit", "-qm", "wip"], cwd=repo, check=True)
    dirty.write_text("// rustconv-assembly: whole-tu\npub fn b() { /* edited */ }\n")
    _guard_assembly_target(dirty, "whole-tu")  # already modified -> mid-grind, allowed
