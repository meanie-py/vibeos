"""Positive/negative controls for verify_cohab_coverage — "an oracle needs an oracle".

This verifier now gates the merge train (gate.py `cohab:<arch>`), and it is exactly the
shape tools/test_scanner_controls.py exists to distrust: its healthy output is `0 findings`,
which is byte-identical to the output of a matcher that has quietly died. It cannot join that
harness — that one drives `check` lints, which are source-only and fixture-driven, while this
one reads object files — so its controls live here.

`_symbols` is the one impure edge (it shells out to llvm-nm), so it is stubbed and everything
above it runs for real: the freshness refusal, the provider/reference set algebra, the
decoration-spelling equivalence, and the `pub mod` scrape that decides what is even flipped.

The fixtures are the REAL msvcrt bugs, not shapes chosen because the code obviously handles
them: `MSVCRT__rotl64` shipped under a bare `_rotl64`, the eight `__stdcall` FDIV stubs under
`___adj_fdiv_m16i@4@4`, and `asm_sqrt` is the defined-but-dead symbol that must NOT be a
finding (CMake's `-Dasm_sqrt=sqrt` rewrites every call site but not the `__ASM_GLOBAL_FUNC`
label, so nothing anywhere references it).
"""

from __future__ import annotations

import importlib.util
import os
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "verify_cohab_coverage", REPO / "tools" / "verify_cohab_coverage.py"
)
assert _spec and _spec.loader
mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(mod)


ARCH = "i386"
OBJ_DIR = f"build-{ARCH}/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir"
SHIM_DIR = f"build-{ARCH}/dll/win32/msvcrt/CMakeFiles/msvcrt.dir"
ARCHIVE = f"build-{ARCH}/dll/win32/msvcrt/libmsvcrt_crt_cargo.a"


def _tree(tmp_path: Path, monkeypatch) -> Path:
    """A minimal repo whose shape is the one the tool navigates, ordered so the archive is
    NEWER than every .rs — otherwise _require_fresh (correctly) refuses to judge at all."""
    root = tmp_path
    monkeypatch.setattr(mod, "ROOT", root)
    (root / ".build-tools" / "llvm-mingw" / "bin").mkdir(parents=True)
    (root / ".build-tools" / "llvm-mingw" / "bin" / "llvm-nm").write_text("")

    src = root / "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt"
    src.mkdir(parents=True)
    (src / "mod.rs").write_text("pub mod math;\n// pub mod notflipped;\n")
    (src / "math.rs").write_text("// translated\n")
    for extra in ("sdk/lib/rust/msvcrt-crt-cargo/src", "sdk/lib/rust/cnt-rust/src"):
        (root / extra).mkdir(parents=True)

    (root / "dll/win32/msvcrt").mkdir(parents=True)
    (root / "dll/win32/msvcrt/msvcrt.spec").write_text("@ cdecl _rotl64(long long long)\n")

    for d in (OBJ_DIR, SHIM_DIR):
        (root / d).mkdir(parents=True)
    (root / OBJ_DIR / "math.c.obj").write_text("")
    (root / SHIM_DIR / "shim.c.obj").write_text("")
    (root / ARCHIVE).write_text("")  # written last => newest => fresh
    # ninja is the one other impure edge. The fixture models a FRESH tree -- every object
    # on disk is on the link line -- so it answers from the directories at call time; the
    # stale-object controls below replace it with a link line that leaves files out.
    monkeypatch.setattr(mod, "_link_inputs", _fresh_link_inputs)
    return root


def _fresh_link_inputs(build: Path, fam: dict) -> tuple[list[Path], list[Path]]:
    direct = sorted((build / SHIM_DIR.split("/", 1)[1]).rglob("*.obj"))
    members = sorted((build / OBJ_DIR.split("/", 1)[1]).rglob("*.obj"))
    return direct, members


def _stub_symbols(monkeypatch, table: dict[str, dict[str, list[tuple[str, str]]]]) -> None:
    """table[basename][mode] -> [(nm code, symbol)]. Anything unlisted has no symbols.

    The archive always gets one unrelated symbol on top of whatever the case asks for: a
    genuinely EMPTY provider set is the tool's "nothing links ahead" refusal, not a finding,
    and every real archive defines hundreds. Without this the positive controls would exercise
    the refusal path instead of the matcher.
    """
    merged = {k: dict(v) for k, v in table.items()}
    arch_entry = merged.setdefault("libmsvcrt_crt_cargo.a", {})
    arch_entry["defined"] = [*arch_entry.get("defined", []), ("T", "_msvcrt_rust_baseline")]

    def fake(_nm, obj, mode):
        return merged.get(Path(obj).name, {}).get(mode, [])

    monkeypatch.setattr(mod, "_symbols", fake)


def _run(root: Path) -> list[str]:
    return mod.check_family(mod.FAMILIES[0], ARCH, root / ".build-tools/llvm-mingw/bin/llvm-nm")


# --- positive controls: the three real msvcrt flips that were silently no-ops -------------


@pytest.mark.parametrize(
    "exported",
    [
        "_MSVCRT__rotl64",  # translated under a bare _rotl64 -> C copy stayed live
        "__adj_fdiv_m16i@4",  # #[export_name] STACKS with PE decoration -> ___adj_...@4@4
        "_invoke_matherr",  # CMake -Dmath_error=_invoke_matherr; source never spells it
    ],
)
def test_a_symbol_the_c_exports_and_the_rust_side_does_not_is_a_finding(
    tmp_path, monkeypatch, exported
):
    root = _tree(tmp_path, monkeypatch)
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("T", exported)], "undefined": []},
            # something still calls it, so the linker WILL go pull math.c.obj back in
            "shim.c.obj": {"defined": [], "undefined": [("U", exported)]},
        },
    )
    findings = _run(root)
    assert len(findings) == 1, findings
    assert exported in findings[0] and "math.rs" in findings[0]


def test_matcher_is_alive_across_every_provider_category(tmp_path, monkeypatch):
    """The negative control that matters: each of the three ways a symbol can legitimately
    already be resolved must silence the finding. Parsing the Rust source for `fn` names saw
    only the first of these and reported 81 false findings."""
    root = _tree(tmp_path, monkeypatch)
    for provider in ("libmsvcrt_crt_cargo.a", "shim.c.obj"):
        _stub_symbols(
            monkeypatch,
            {
                "math.c.obj": {"defined": [("T", "_MSVCRT__rotl64")], "undefined": []},
                provider: {"defined": [("T", "_MSVCRT__rotl64")], "undefined": []},
                "shim.c.obj": (
                    {"defined": [("T", "_MSVCRT__rotl64")], "undefined": []}
                    if provider == "shim.c.obj"
                    else {"defined": [], "undefined": [("U", "_MSVCRT__rotl64")]}
                ),
            },
        )
        assert _run(root) == [], f"{provider} provides it; that is not a gap"


def test_decoration_is_not_a_mismatch(tmp_path, monkeypatch):
    """i386 PE prepends `_` and __stdcall appends `@N`; amd64/arm64 do neither. Comparing a
    decorated name against an undecorated one is how a real match reads as missing."""
    root = _tree(tmp_path, monkeypatch)
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("T", "__adj_fdiv_m16i@4")], "undefined": []},
            "libmsvcrt_crt_cargo.a": {"defined": [("T", "_adj_fdiv_m16i")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "__adj_fdiv_m16i@4")]},
        },
    )
    assert _run(root) == []


# --- negative controls: shapes that must NOT be findings ----------------------------------


def test_defined_but_dead_is_not_a_finding(tmp_path, monkeypatch):
    """`asm_sqrt`/`asm_sqrtf`/`x87_asin`: CMake retargets every CALL SITE to libm but not the
    `__ASM_GLOBAL_FUNC` label (`#name` stringifies before macro expansion). No undefined
    reference exists anywhere, so the linker never goes looking and nothing need define it.
    Getting this wrong would mean an allowlist, and an allowlist rots."""
    root = _tree(tmp_path, monkeypatch)
    _stub_symbols(
        monkeypatch,
        {"math.c.obj": {"defined": [("T", "_asm_sqrt")], "undefined": []}},
    )
    assert _run(root) == []
    # Anchor it: an `== []` negative control passes just as well when the matcher is dead, so
    # prove the ONLY difference is the reference. Same fixture, one `U` added, must flip red.
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("T", "_asm_sqrt")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "_asm_sqrt")]},
        },
    )
    assert len(_run(root)) == 1


def test_file_local_symbols_cannot_pull_the_object_in(tmp_path, monkeypatch):
    root = _tree(tmp_path, monkeypatch)
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("t", "_helper"), ("d", "_table")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "_helper"), ("U", "_table")]},
        },
    )
    assert _run(root) == []
    # Anchored the same way: identical fixture, uppercase (global) codes, must flip red.
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("T", "_helper"), ("D", "_table")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "_helper"), ("U", "_table")]},
        },
    )
    assert len(_run(root)) == 2


def test_a_module_that_is_not_flipped_is_not_checked(tmp_path, monkeypatch):
    """`pub mod X;` IS the flip. A commented-out one means the C is still live and correct."""
    root = _tree(tmp_path, monkeypatch)
    (root / OBJ_DIR / "notflipped.c.obj").write_text("")
    _stub_symbols(
        monkeypatch,
        {
            "notflipped.c.obj": {"defined": [("T", "_gone")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "_gone")]},
        },
    )
    assert _run(root) == []
    assert mod._flipped_modules(mod.FAMILIES[0]) == ["math"]
    # Anchored: the same gap in the module that IS flipped is a finding, so the silence above
    # is the `pub mod` scrape doing its job and not the whole check having gone quiet.
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("T", "_gone")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "_gone")]},
        },
    )
    assert len(_run(root)) == 1


# --- refusals: the two ways this tool must decline to render a verdict ---------------------


def test_a_stale_object_on_disk_is_not_a_provider(tmp_path, monkeypatch):
    """The msvcrt shim split left stale copies in msvcrt.dir in every build dir configured
    before it. A directory glob counted them as providers; only the link line decides. Here
    the stale file defines exactly the symbol the flipped twin exports -- and it must still
    be a finding, because nothing links that file."""
    root = _tree(tmp_path, monkeypatch)
    (root / SHIM_DIR / "stale_old_shim.c.obj").write_text("")
    fresh = _fresh_link_inputs
    monkeypatch.setattr(
        mod,
        "_link_inputs",
        lambda build, fam: (
            [o for o in fresh(build, fam)[0] if o.name != "stale_old_shim.c.obj"],
            fresh(build, fam)[1],
        ),
    )
    _stub_symbols(
        monkeypatch,
        {
            "math.c.obj": {"defined": [("T", "_MSVCRT__rotl64")]},
            "stale_old_shim.c.obj": {"defined": [("T", "_MSVCRT__rotl64")]},
            "shim.c.obj": {"undefined": [("U", "_MSVCRT__rotl64")]},
        },
    )
    assert any("_MSVCRT__rotl64" in f for f in _run(root))


def test_a_stale_twin_that_links_into_nothing_is_not_checked(tmp_path, monkeypatch):
    """A flipped module's C object that is on disk but NOT a member of the static library
    (its source moved or stopped compiling) must not be judged -- it is not what links."""
    root = _tree(tmp_path, monkeypatch)
    (root / "dll/win32/msvcrt/math.c").write_text("")  # the source still exists
    monkeypatch.setattr(
        mod,
        "_link_inputs",
        lambda build, fam: ([build / SHIM_DIR.split("/", 1)[1] / "shim.c.obj"], []),
    )
    _stub_symbols(monkeypatch, {"math.c.obj": {"defined": [("T", "_MSVCRT__rotl64")]}})
    assert _run(root) == []


def test_a_stale_archive_is_refused_not_reported(tmp_path, monkeypatch):
    """Run against archives three days older than the tree, this tool reported 669 findings —
    all staleness — and "proved" that errno.c, file.c and time.c leave hundreds of symbols to
    the C. A verdict computed from a stale binary is worse than none: it looks identical."""
    root = _tree(tmp_path, monkeypatch)
    archive = root / ARCHIVE
    rs = root / "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/math.rs"
    # Explicit, not `touch()`: same-second writes land on an equal mtime and the freshness
    # test is a strict `>`, so a touch alone leaves the tree looking fresh and this control
    # silently exercises some other path.
    edited = archive.stat().st_mtime + 100
    os.utime(rs, (edited, edited))
    _stub_symbols(monkeypatch, {})
    with pytest.raises(SystemExit) as e:
        _run(root)
    assert "OLDER than" in str(e.value)


def test_nothing_linking_ahead_is_a_refusal_not_a_clean_sweep(tmp_path, monkeypatch):
    """With no Rust archive and no module objects, EVERY symbol looks missing. That is a false
    verdict, not a finding, and the difference is what a merge-queue rejection rests on."""
    root = _tree(tmp_path, monkeypatch)
    # Deliberately NOT _stub_symbols: that seeds a baseline archive symbol precisely so the
    # other cases don't land here. This case is the empty provider set itself.
    monkeypatch.setattr(
        mod,
        "_symbols",
        lambda _nm, obj, mode: (
            [("T", "_x")] if Path(obj).name == "math.c.obj" and mode == "defined" else []
        ),
    )
    with pytest.raises(SystemExit) as e:
        _run(root)
    assert "nothing links ahead" in str(e.value)


def test_check_over_zero_pairs_is_not_green(tmp_path, monkeypatch):
    """ "I compared nothing" must not read as "I found nothing". Rename a build dir, move
    msvcrt_static's obj dir, add an arch whose layout differs — each silently retires this
    check while still printing green, forever, with nobody to notice."""
    monkeypatch.setattr(mod, "ROOT", tmp_path)
    (tmp_path / ".build-tools/llvm-mingw/bin").mkdir(parents=True)
    (tmp_path / ".build-tools/llvm-mingw/bin/llvm-nm").write_text("")
    (tmp_path / f"build-{ARCH}").mkdir()  # exists, but holds no msvcrt objects
    monkeypatch.setattr(sys, "argv", ["verify_cohab_coverage.py", "--arch", ARCH, "--check"])
    assert mod.main() == 1
    # ...and without --check it stays advisory, so an interactive run still reports
    monkeypatch.setattr(sys, "argv", ["verify_cohab_coverage.py", "--arch", ARCH])
    assert mod.main() == 0


# --- the object LOOKUP itself, which used to fail open ------------------------------------


def _reflip(root: Path, mods: str) -> None:
    """Rewrite which modules are flipped, then re-stamp the archive as the newest thing in the
    tree. Without the second half every case below lands in the staleness refusal instead of
    the code under test — editing mod.rs is itself a source edit."""
    (root / "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/mod.rs").write_text(mods)
    newest = max(rs.stat().st_mtime for rs in (root / "sdk/lib/rust").rglob("*.rs"))
    os.utime(root / ARCHIVE, (newest + 10, newest + 10))


def test_a_module_named_differently_from_its_c_file_is_still_checked(tmp_path, monkeypatch):
    """`reactos/misc.c` cannot become `misc.rs` — that name is msvcrt's own Wine-derived
    misc.c — so the module is `reactos_misc` and the family's `objects` map is the only thing
    that pairs the two. Without the map the lookup misses and the module is checked against
    nothing at all."""
    root = _tree(tmp_path, monkeypatch)
    (root / OBJ_DIR / "reactos").mkdir()
    _reflip(root, "pub mod reactos_misc;\n")
    (root / OBJ_DIR / "reactos/misc.c.obj").write_text("")
    _stub_symbols(
        monkeypatch,
        {
            "misc.c.obj": {"defined": [("D", "__mbcasemap")], "undefined": []},
            "shim.c.obj": {"defined": [], "undefined": [("U", "__mbcasemap")]},
        },
    )
    findings = _run(root)
    assert len(findings) == 1, findings
    assert "reactos_misc.rs" in findings[0] and "reactos/misc.c.obj" in findings[0]


def test_a_module_whose_c_file_this_arch_does_not_compile_is_not_a_finding(tmp_path, monkeypatch):
    """`except_i386.c` exists in the tree but only i386 compiles it. On arm64 there is no
    object and nothing to check, which is a fact about the arch, not about the module."""
    root = _tree(tmp_path, monkeypatch)
    (root / "dll/win32/msvcrt/except_i386.c").write_text("/* the C source exists */\n")
    _reflip(root, "pub mod except_i386;\n")
    _stub_symbols(monkeypatch, {})
    assert _run(root) == []


def test_a_module_matching_no_c_file_anywhere_is_a_finding(tmp_path, monkeypatch):
    """The hole this closes. A missing object used to be a bare `continue`, so a renamed or
    mistyped module was compared against nothing and reported CLEAN — the same shape of
    silent pass as the `extern` parser bug this file exists to catch."""
    root = _tree(tmp_path, monkeypatch)
    _reflip(root, "pub mod raectos_misc;\n")
    _stub_symbols(monkeypatch, {})
    findings = _run(root)
    assert len(findings) == 1, findings
    assert "checked against nothing" in findings[0]


def test_a_declared_rust_only_module_is_not_a_finding(tmp_path, monkeypatch):
    """`cpp_rtti` is the RTTI half of cpp.c and `seh_i386` has no C original at all. "No
    object" is the truth about them rather than a lookup that failed, and saying so costs one
    line with a reason attached — which is what makes the case above safe to fail loudly."""
    root = _tree(tmp_path, monkeypatch)
    _reflip(root, "pub mod cpp_rtti;\n")
    _stub_symbols(monkeypatch, {})
    assert _run(root) == []
    assert mod.FAMILIES[0]["no_c_twin"]["cpp_rtti"], "an exemption without a reason is an allowlist"
