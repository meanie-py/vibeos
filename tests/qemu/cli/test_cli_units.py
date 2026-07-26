"""Unit tests for the vibeos-cli pure logic — no QEMU, no build needed.

Run: <venv>/bin/python -m pytest tests/qemu/cli/test_cli_units.py -q
"""

from __future__ import annotations

import json
import os
import re
import socket
import struct
import subprocess
import threading
import time
from pathlib import Path

import pytest

from tests.qemu import backend as be_mod
from tests.qemu import qmp_client, serial_client
from tests.qemu import vm as vm_mod

from . import build as build_mod
from . import config as config_mod
from . import conformance as conformance_mod
from . import gdbengine as ge_mod
from . import lints, provenance, repro, triage
from .artifacts import Run, Stage

# --- provenance: git fingerprint + stamp round-trip + staleness ------------


def _git(repo: Path, *args: str) -> None:
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True)


@pytest.fixture
def tiny_repo(tmp_path: Path) -> Path:
    r = tmp_path / "repo"
    r.mkdir()
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t")
    _git(r, "config", "user.name", "t")
    (r / "a.txt").write_text("hello\n")
    _git(r, "add", "-A")
    _git(r, "commit", "-qm", "init")
    return r


def test_git_fingerprint_stable_then_changes(tiny_repo: Path):
    fp1 = provenance.git_fingerprint(tiny_repo)
    fp2 = provenance.git_fingerprint(tiny_repo)
    assert fp1 == fp2, "fingerprint must be deterministic for unchanged tree"
    (tiny_repo / "a.txt").write_text("changed\n")
    fp3 = provenance.git_fingerprint(tiny_repo)
    assert fp3 != fp1, "working-tree edit must change the fingerprint"


def test_git_fingerprint_immune_to_mtime(tiny_repo: Path):
    fp1 = provenance.git_fingerprint(tiny_repo)
    # touch bumps mtime but not content
    (tiny_repo / "a.txt").touch()
    assert provenance.git_fingerprint(tiny_repo) == fp1, "mtime churn must not change fingerprint"


# --- freeldr boot-entry selection by in-place ISO patch --------------------
#
# The MP pair (ntkrnlmp.exe + halmp.dll) has always shipped on the same media as
# the UP pair; only `DefaultOS=` inside freeldr.ini chose between them, and with
# TimeOut=0 there is no menu to choose at. Rewriting that one line in a COPY of
# the image replaces a cmake reconfigure + mkisofs repack (8m30s measured) with a
# 0.15s copy — so an MP gate leg costs one extra boot. Byte-level, so a plain
# bytes fixture exercises it exactly as a real ISO does.


def _fake_iso(default_os: bytes = b"LiveCD_Debug", smp_section: bool = True) -> bytes:
    body = b"\x00" * 64 + b"[FREELOADER]\r\nDefaultOS=" + default_os + b"\r\nTimeOut=0\r\n"
    if smp_section:
        body += b"[LiveCD_SMP]\r\nOptions=/KERNEL=ntkrnlmp.exe /HAL=halmp.dll\r\n"
    return body + b"\x00" * 64


def test_smp_iso_patch_is_equal_length_and_selects_the_mp_entry(tmp_path: Path):
    src = tmp_path / "livecd.iso"
    src.write_bytes(_fake_iso())
    out = vm_mod.smp_selected_iso(src, tmp_path / "work")
    assert out != src, "must not patch the build tree's media in place"
    patched = out.read_bytes()
    assert len(patched) == len(src.read_bytes()), "ISO9660 extents: length must not change"
    assert b"DefaultOS=LiveCD_SMP\r\n\r\n" in patched
    assert b"LiveCD_Debug" not in patched


def test_smp_iso_patch_is_idempotent(tmp_path: Path):
    """Media built with --smp-kernel already defaults to the MP entry; patching it
    again must be a no-op that returns the original path, not a needless copy."""
    src = tmp_path / "livecd.iso"
    src.write_bytes(_fake_iso(default_os=b"LiveCD_SMP"))
    assert vm_mod.smp_selected_iso(src, tmp_path / "work") == src


def test_smp_iso_patch_raises_rather_than_booting_up_media(tmp_path: Path):
    """Every failure path must raise. Silently returning the unpatched ISO would
    boot the UP pair on an --smp run — exactly the false green _smp_check exists
    to catch, and the reason this is not a best-effort helper."""
    work = tmp_path / "work"
    # no [LiveCD_SMP] section: media predates the halmp.dll twin
    a = tmp_path / "a.iso"
    a.write_bytes(_fake_iso(smp_section=False))
    with pytest.raises(RuntimeError, match="LiveCD_SMP"):
        vm_mod.smp_selected_iso(a, work)
    # two DefaultOS= lines: cannot tell which one freeldr reads
    b = tmp_path / "b.iso"
    b.write_bytes(_fake_iso() + b"DefaultOS=LiveCD\r\n")
    with pytest.raises(RuntimeError, match="exactly one"):
        vm_mod.smp_selected_iso(b, work)
    # value too short to overwrite in place
    c = tmp_path / "c.iso"
    c.write_bytes(_fake_iso(default_os=b"X"))
    with pytest.raises(RuntimeError, match="shorter"):
        vm_mod.smp_selected_iso(c, work)


def test_smp_boot_timeout_scales_the_clock_only_for_real_smp():
    """A UP boot and -smp 1 must keep the tuned budget untouched; only a genuinely
    multi-vCPU boot pays the MTTCG stretch."""
    assert vm_mod.smp_boot_timeout(180.0, None) == 180.0
    assert vm_mod.smp_boot_timeout(180.0, 1) == 180.0
    assert vm_mod.smp_boot_timeout(180.0, 2) == 360.0
    assert vm_mod.smp_boot_timeout(180.0, 4) == 720.0


def test_smp_boot_timeout_does_not_compound_across_boots(monkeypatch, tmp_path: Path):
    """Reusing one ArchConfig across VMs (a gate retry loop) must NOT multiply the
    budget again each time — the scale is always recomputed from the stashed UP
    value, so a wedge still fails at 2x, not 8x."""
    monkeypatch.setenv("VIBEOS_QEMU_SMP", "2")
    cfg = vm_mod.get_arch_config("arm64")
    up = cfg.boot_timeout
    for _ in range(3):
        vm_mod.QemuVM(cfg, tmp_path / "w")
    assert cfg.boot_timeout == up * 2


def _res(name: str, p: int = 10, f: int = 0, status: str = "OK") -> dict:
    return {"name": name, "status": status, "pass": p, "fail": f, "skip": 0, "exit": 0}


def test_write_baseline_refuses_to_delete_tests_from_the_floor():
    """The ratchet form of "never weaken a test to make it pass": --write-baseline
    replaces the floor wholesale, so a sweep cut short by a boot flake or an early
    BUGCHECK would silently remove every test it never reached."""
    base = {n: {"status": "OK", "pass": 10, "fail": 0} for n in ("A", "B", "C")}
    short = [_res("A"), _res("B")]  # C never launched
    msg = conformance_mod.baseline_coverage_refusal(short, base, force=False)
    assert msg and "REFUSED" in msg and "    C\n" in msg
    # a full sweep is fine
    assert (
        conformance_mod.baseline_coverage_refusal(
            [_res("A"), _res("B"), _res("C")], base, force=False
        )
        is None
    )
    # ...and --force-baseline is the deliberate retire-a-test escape hatch
    assert conformance_mod.baseline_coverage_refusal(short, base, force=True) is None


def test_write_baseline_announces_a_lowered_floor_but_allows_it():
    """Absorbing a known regression is sometimes deliberate (i386/arm64 both took
    ExHardError 30/0 -> 24/6 on purpose) — it just must never be silent."""
    base = {"ExHardError": {"status": "OK", "pass": 30, "fail": 0}}
    note = conformance_mod.baseline_lowering_note([_res("ExHardError", 24, 6)], base)
    assert note and "ExHardError: pass 30 -> 24, fail 0 -> 6" in note
    # unchanged rows say nothing, and an IMPROVEMENT is not a lowering
    assert conformance_mod.baseline_lowering_note([_res("ExHardError", 30, 0)], base) is None
    assert conformance_mod.baseline_lowering_note([_res("ExHardError", 31, 0)], base) is None


def test_run_raises_loudly_instead_of_returning_empty(tiny_repo: Path):
    """The BUILD_INCOMPLETE train-halter class: _run used to swallow BOTH a
    non-zero exit and any exception, returning "". That silently shifted the
    fingerprint (false RED) or emptied cargo_module_rels (false GREEN), and left
    no evidence either way. Any failure must now name the command, rc and stderr."""
    with pytest.raises(provenance.ProvenanceError) as ei:
        provenance._run(["git", "rev-parse", "--verify", "no-such-ref-xyz"], tiny_repo)
    msg = str(ei.value)
    assert "exited" in msg and "rev-parse" in msg, msg


def test_run_tolerates_declared_nonzero_codes(tiny_repo: Path):
    """`git grep` exits 1 for "no matches" — a real empty result, not a broken
    oracle. Without ok_codes the loud version would reject every clean repo."""
    assert provenance._run(["git", "grep", "-l", "zzz-absent"], tiny_repo, ok_codes=(0, 1)) == ""
    with pytest.raises(provenance.ProvenanceError):
        provenance._run(["git", "grep", "-l", "zzz-absent"], tiny_repo)


def test_check_staleness_reports_oracle_failure_rather_than_raising(tmp_path: Path, monkeypatch):
    """A broken oracle must be reported AS a broken oracle — not laundered into a
    confident "source changed since build", and not escaping as a traceback."""
    bd = tmp_path / "build-amd64"
    bd.mkdir()
    media = bd / "livecd.iso"
    media.write_bytes(b"iso")
    provenance.Stamp(
        arch="amd64",
        ts=1,
        git_fingerprint="gf",
        rust_src_hash="rs",
        boot_media="build-amd64/livecd.iso",
        boot_media_sha=provenance._sha256_file(media),
        cold=True,
    ).write(bd)
    monkeypatch.setattr(
        provenance,
        "git_fingerprint",
        lambda *a, **k: (_ for _ in ()).throw(provenance.ProvenanceError("git exploded")),
    )
    st = provenance.check_staleness(tmp_path, "amd64")
    assert not st.fresh
    assert any("provenance oracle FAILED" in r and "git exploded" in r for r in st.reasons), (
        st.reasons
    )


def test_src_delta_names_a_head_move(tiny_repo: Path):
    """ "build source changed" fires milliseconds after the same hash was stamped
    and never said WHICH input moved. A merge landing under the build is the
    interesting case and now says so by name."""
    old_head = provenance.git_head(tiny_repo)
    stamp = provenance.Stamp(
        arch="amd64",
        ts=1,
        git_fingerprint="gf",
        rust_src_hash="rs",
        boot_media="x",
        boot_media_sha="",
        cold=True,
        head_sha=old_head,
    )
    (tiny_repo / "b.txt").write_text("second\n")
    _git(tiny_repo, "add", "-A")
    _git(tiny_repo, "commit", "-qm", "second")
    delta = provenance._src_delta(tiny_repo, stamp)
    assert "HEAD MOVED SINCE THE BUILD" in delta, delta
    assert old_head[:12] in delta, delta


def test_src_delta_flags_a_phantom_change(tiny_repo: Path):
    """Fingerprint moved, HEAD did not, and nothing is modified => the ORACLE is
    the suspect, not the tree. Saying that out loud is the whole point: this is
    the shape that halted the merge train with no evidence."""
    stamp = provenance.Stamp(
        arch="amd64",
        ts=1,
        git_fingerprint="gf",
        rust_src_hash="rs",
        boot_media="x",
        boot_media_sha="",
        cold=True,
        head_sha=provenance.git_head(tiny_repo),
    )
    delta = provenance._src_delta(tiny_repo, stamp)
    assert "suspect the oracle" in delta, delta


def test_path_scoped_fingerprint(tiny_repo: Path):
    (tiny_repo / "src").mkdir()
    (tiny_repo / "src" / "x.rs").write_text("fn main(){}\n")
    _git(tiny_repo, "add", "-A")
    _git(tiny_repo, "commit", "-qm", "src")
    fp_src = provenance.git_fingerprint(tiny_repo, ["src"])
    (tiny_repo / "a.txt").write_text("doc change\n")  # outside src
    assert provenance.git_fingerprint(tiny_repo, ["src"]) == fp_src, (
        "non-src change must not move the src-scoped fingerprint"
    )


def test_stamp_roundtrip(tmp_path: Path):
    bd = tmp_path / "build-i386"
    bd.mkdir()
    s = provenance.Stamp(
        arch="i386",
        ts=123,
        git_fingerprint="gf",
        rust_src_hash="rh",
        boot_media="x/livecd.iso",
        boot_media_sha="bs",
        cold=True,
        contributing={"ntoskrnl.exe": "abc"},
        rust_targets=["t"],
    )
    s.write(bd)
    back = provenance.Stamp.read(bd)
    assert back is not None and back.cold is True and back.contributing == {"ntoskrnl.exe": "abc"}


@pytest.fixture
def staleness_repo(tmp_path: Path, monkeypatch) -> Path:
    """A git repo with a fake build-i386/livecd.iso + sdk/lib/rust source, so
    check_staleness() can be exercised against real make_stamp output."""
    r = tmp_path / "repo"
    (r / "sdk" / "lib" / "rust").mkdir(parents=True)
    (r / "build-i386").mkdir()
    (r / "docs").mkdir()
    (r / "sdk" / "lib" / "rust" / "x.rs").write_text("fn k(){}\n")
    (r / "docs" / "readme.md").write_text("hi\n")
    (r / "build-i386" / "livecd.iso").write_bytes(b"ISO-CONTENT-v1")
    # Model the real repo: build-*/ is gitignored. Without this the fixture's own STAMP
    # (written into build-i386/ after the commit) shows up as an untracked file and moves
    # every fingerprint between make_stamp and check_staleness — which is why these tests
    # never matched tier 1 and reached the tier-2 path at all. They passed only because the
    # oracle down there was blind and called everything "clean-diff (non-build change)".
    (r / ".gitignore").write_text("build-*/\n")
    # Non-Rust build inputs. rust_src_hash is scoped to sdk/lib/rust, so these are the
    # ones the deleted `ninja -n` oracle was supposed to cover and silently did not.
    (r / "dll" / "win32" / "thing").mkdir(parents=True)
    (r / "dll" / "win32" / "thing" / "thing.c").write_text("int f(void){return 1;}\n")
    (r / "dll" / "win32" / "thing" / "CMakeLists.txt").write_text("add_library(thing thing.c)\n")
    (r / "dll" / "win32" / "thing" / "thing.spec").write_text("@ stdcall f()\n")
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t")
    _git(r, "config", "user.name", "t")
    _git(r, "add", "-A")
    _git(r, "commit", "-qm", "init")
    return r


def test_staleness_fresh_cold(staleness_repo: Path):
    s = provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1)
    s.write(staleness_repo / "build-i386")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.fresh and not v.stale and not v.warm, v.reasons


def test_staleness_warm_flag(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=False, ts=1).write(
        staleness_repo / "build-i386"
    )
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.fresh and v.warm, "warm build must be fresh-but-flagged"


def test_staleness_rust_source_change_is_stale(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "sdk" / "lib" / "rust" / "x.rs").write_text("fn k(){ /*edit*/ }\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale and not v.fresh
    assert any("rust source changed" in r for r in v.reasons), v.reasons


def test_staleness_doc_change_is_clean_diff(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "docs" / "readme.md").write_text("changed doc\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.fresh, f"a doc-only change must not be stale: {v.reasons}"
    assert any("clean-diff" in r for r in v.reasons)


def test_staleness_media_missing(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "build-i386" / "livecd.iso").unlink()
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.media_missing and v.stale


def test_staleness_media_swapped_is_stale(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    # rebuild media out of band (content changes) without touching git
    (staleness_repo / "build-i386" / "livecd.iso").write_bytes(b"ISO-CONTENT-v2-different")
    # fingerprint still matches (no git change) -> tier-1 says fresh; this is the
    # documented limitation (media swapped with zero source change is rare). The
    # real guard is: a source change escalates and re-hashes media. Verify the
    # escalation path catches a swap WHEN source also moved:
    (staleness_repo / "sdk" / "lib" / "rust" / "x.rs").write_text("fn k(){ }\n// e\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale, v.reasons


def test_staleness_arm64_restage_trap(tmp_path: Path):
    """arm64-specific: ntoskrnl.exe newer than boot.img (ninja rebuilt the
    kernel but mkramdisk wasn't re-run) must be STALE even when rust source is
    unchanged — isolated here via a doc-only git change to trigger escalation."""
    import os

    r = tmp_path / "repo"
    (r / "sdk" / "lib" / "rust").mkdir(parents=True)
    (r / "docs").mkdir()
    (r / "qemu").mkdir()
    (r / "build-arm64" / "ntoskrnl").mkdir(parents=True)
    (r / "sdk" / "lib" / "rust" / "x.rs").write_text("fn k(){}\n")
    (r / "docs" / "readme.md").write_text("hi\n")
    (r / "qemu" / "boot.img").write_bytes(b"BOOT")
    (r / "build-arm64" / "ntoskrnl" / "ntoskrnl.exe").write_bytes(b"KERNEL")
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t")
    _git(r, "config", "user.name", "t")
    _git(r, "add", "-A")
    _git(r, "commit", "-qm", "init")

    provenance.make_stamp(r, "arm64", cold=True, ts=1).write(r / "build-arm64")
    # boot.img OLD, kernel NEW (the missed-restage condition)
    boot = r / "qemu" / "boot.img"
    kern = r / "build-arm64" / "ntoskrnl" / "ntoskrnl.exe"
    os.utime(boot, (1000, 1000))
    os.utime(kern, (2000, 2000))
    # doc-only change escalates past the tier-1 fingerprint (rust unchanged)
    (r / "docs" / "readme.md").write_text("changed\n")

    v = provenance.check_staleness(r, "arm64")
    assert v.stale, v.reasons
    assert any("re-staged" in reason for reason in v.reasons), v.reasons


@pytest.fixture
def arm64_cohab_repo(tmp_path: Path) -> Path:
    """An arm64 repo with one Rust-cohab driver (foo-rust -> foo.sys), staged via
    mkramdisk REQUIRED_FILES, plus a build-arm64 artifact + boot.img + stamp — the
    fixture for the staged-build cohab freshness net (advapi32/fastfat class)."""
    r = tmp_path / "repo"
    crate = r / "sdk" / "lib" / "rust" / "foo-rust"
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text("[package]\nname = 'foo-rust'\n")
    (crate / "src" / "lib.rs").write_text("// rust cohab\n")
    (r / "build-arm64" / "drivers" / "x" / "foo").mkdir(parents=True)
    (r / "build-arm64" / "drivers" / "x" / "foo" / "foo.sys").write_bytes(b"SYS")
    (r / "qemu").mkdir()
    (r / "qemu" / "mkramdisk.sh").write_text(
        'REQUIRED_FILES=(\n    "$BUILD_DIR/ntoskrnl/ntoskrnl.exe"\n'
        '    "$BUILD_DIR/drivers/x/foo/foo.sys"\n)\n'
    )
    (r / "qemu" / "boot.img").write_bytes(b"BOOT")
    (r / "docs").mkdir()
    (r / "docs" / "readme.md").write_text("hi\n")
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t")
    _git(r, "config", "user.name", "t")
    _git(r, "add", "-A")
    _git(r, "commit", "-qm", "init")
    provenance.make_stamp(r, "arm64", cold=True, ts=1).write(r / "build-arm64")
    return r


def test_mkramdisk_required_rels_parsed(arm64_cohab_repo: Path):
    rels = provenance.mkramdisk_required_rels(arm64_cohab_repo)
    assert "drivers/x/foo/foo.sys" in rels and "ntoskrnl/ntoskrnl.exe" in rels


def test_crate_for_stem_separator_normalization(arm64_cohab_repo: Path):
    # underscore'd binary stem (kernel32_vista) must find the hyphen crate name
    (arm64_cohab_repo / "sdk" / "lib" / "rust" / "k32-vista-rust").mkdir()
    (arm64_cohab_repo / "sdk" / "lib" / "rust" / "k32-vista-rust" / "Cargo.toml").write_text("x")
    assert provenance._crate_for_stem(arm64_cohab_repo, "k32_vista") is not None
    assert provenance._crate_for_stem(arm64_cohab_repo, "nonexistent") is None


def test_cohab_rels_maps_only_rust_binaries(arm64_cohab_repo: Path):
    rels = provenance.boot_set_cohab_rels(arm64_cohab_repo, "arm64")
    assert "drivers/x/foo/foo.sys" in rels  # has foo-rust crate
    assert "ntoskrnl/ntoskrnl.exe" not in rels  # no ntoskrnl-rust crate in this repo
    # x86 never stages — always empty (full ninja livecd can't ship stale cohab)
    assert provenance.boot_set_cohab_rels(arm64_cohab_repo, "i386") == []


def test_cohab_fresh_binary_not_stale(arm64_cohab_repo: Path):
    sysf = arm64_cohab_repo / "build-arm64" / "drivers" / "x" / "foo" / "foo.sys"
    rs = arm64_cohab_repo / "sdk" / "lib" / "rust" / "foo-rust" / "src" / "lib.rs"
    os.utime(rs, (1000, 1000))
    os.utime(sysf, (2000, 2000))  # binary newer than its .rs == freshly rebuilt
    assert provenance.cohab_staleness_reasons(arm64_cohab_repo, "arm64") == []


def test_cohab_stale_binary_flagged(arm64_cohab_repo: Path):
    sysf = arm64_cohab_repo / "build-arm64" / "drivers" / "x" / "foo" / "foo.sys"
    rs = arm64_cohab_repo / "sdk" / "lib" / "rust" / "foo-rust" / "src" / "lib.rs"
    os.utime(sysf, (1000, 1000))
    os.utime(rs, (2000, 2000))  # .rs newer == cohab flip not rebuilt
    reasons = provenance.cohab_staleness_reasons(arm64_cohab_repo, "arm64")
    assert any("foo.sys" in r and "stale" in r for r in reasons), reasons


def test_cohab_missing_artifact_flagged(arm64_cohab_repo: Path):
    (arm64_cohab_repo / "build-arm64" / "drivers" / "x" / "foo" / "foo.sys").unlink()
    reasons = provenance.cohab_staleness_reasons(arm64_cohab_repo, "arm64")
    assert any("foo.sys" in r and "absent" in r for r in reasons), reasons


def test_cohab_stale_overrides_tier1_fresh(arm64_cohab_repo: Path):
    """The crux: fingerprint matches the stamp (no source change since build), so
    tier-1 would short-circuit to FRESH — but the cohab binary was never rebuilt,
    so the net must still mark it stale (the exact advapi32 false-green)."""
    sysf = arm64_cohab_repo / "build-arm64" / "drivers" / "x" / "foo" / "foo.sys"
    rs = arm64_cohab_repo / "sdk" / "lib" / "rust" / "foo-rust" / "src" / "lib.rs"
    os.utime(sysf, (1000, 1000))
    os.utime(rs, (2000, 2000))
    v = provenance.check_staleness(arm64_cohab_repo, "arm64")
    assert v.stale and not v.fresh, v.reasons
    assert any("foo.sys" in r for r in v.reasons), v.reasons


@pytest.fixture
def cargo_link_repo(tmp_path: Path) -> Path:
    """An x86 repo with one cargo-LINKED module (bar-cargo -> bar/bar.exe on the
    livecd manifest) — the fixture for the cargo-link freshness net (the
    2026-07-04 stale-ntoskrnl-inside-a-fresh-ISO class)."""
    r = tmp_path / "repo"
    ws = r / "sdk" / "lib" / "rust"
    (ws / "bar-rust" / "src").mkdir(parents=True)
    (ws / "bar-rust" / "Cargo.toml").write_text("[package]\nname = 'bar-rust'\n")
    (ws / "bar-rust" / "src" / "lib.rs").write_text("// body\n")
    (r / "bar").mkdir()
    (r / "bar" / "CMakeLists.txt").write_text(
        "add_cargo_module(bar bar-cargo bar.exe\n    KIND kernelexe\n    BODY_CRATE bar-rust)\n"
        "add_cargo_kernel_dll(${_t} v-cargo ${_t}.dll)\n"  # variable output — must be skipped
    )
    (r / "build-amd64" / "bar").mkdir(parents=True)
    (r / "build-amd64" / "bar" / "bar.exe").write_bytes(b"PE")
    (r / "build-amd64" / "boot").mkdir()
    (r / "build-amd64" / "boot" / "livecd.cmake.lst").write_text(
        "reactos/system32/bar.exe=/tmp/vibeos/build-amd64/bar/bar.exe\n"
        "/tmp/vibeos/build-amd64/boot/empty\n"
    )
    (r / "build-amd64" / "livecd.iso").write_bytes(b"ISO")
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t")
    _git(r, "config", "user.name", "t")
    _git(r, "add", "-A")
    _git(r, "commit", "-qm", "init")
    provenance.make_stamp(r, "amd64", cold=True, ts=1).write(r / "build-amd64")
    return r


def test_cargo_module_rels_parsed(cargo_link_repo: Path):
    rels = provenance.cargo_module_rels(cargo_link_repo)
    assert rels == ["bar/bar.exe"]  # ${_t}.dll variable call site skipped


def test_cargo_link_fresh_binary_not_stale(cargo_link_repo: Path):
    exe = cargo_link_repo / "build-amd64" / "bar" / "bar.exe"
    crate = cargo_link_repo / "sdk" / "lib" / "rust" / "bar-rust"
    os.utime(crate / "src" / "lib.rs", (1000, 1000))
    os.utime(crate / "Cargo.toml", (1000, 1000))  # .toml is a workspace src too
    os.utime(exe, (2000, 2000))
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64") == []


def test_cargo_link_stale_binary_flagged_x86(cargo_link_repo: Path):
    """The crux of the 2026-07-04 incident: a fresh ISO around a stale cargo-linked
    PE, on x86 (which the cohab net exempts). Tier-1 fingerprint matches the stamp,
    so only this net can catch it."""
    exe = cargo_link_repo / "build-amd64" / "bar" / "bar.exe"
    rs = cargo_link_repo / "sdk" / "lib" / "rust" / "bar-rust" / "src" / "lib.rs"
    os.utime(exe, (1000, 1000))
    os.utime(rs, (2000, 2000))  # .rs newer == cargo link never re-ran
    reasons = provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64")
    assert any("bar.exe" in x and "stale" in x for x in reasons), reasons
    v = provenance.check_staleness(cargo_link_repo, "amd64")
    assert v.stale and not v.fresh, v.reasons


def test_cargo_link_scope_excludes_off_media_modules(cargo_link_repo: Path):
    """A cargo module NOT on the boot media (e.g. rosload on a UEFI-only arch)
    must not block smoke when it goes stale — scope is the CD manifest."""
    lst = cargo_link_repo / "build-amd64" / "boot" / "livecd.cmake.lst"
    lst.write_text("/tmp/vibeos/build-amd64/boot/empty\n")  # bar.exe not on CD
    exe = cargo_link_repo / "build-amd64" / "bar" / "bar.exe"
    rs = cargo_link_repo / "sdk" / "lib" / "rust" / "bar-rust" / "src" / "lib.rs"
    os.utime(exe, (1000, 1000))
    os.utime(rs, (2000, 2000))
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64") == []


def test_discover_rust_targets(tmp_path: Path):
    bd = tmp_path / "build-i386"
    (bd / "ntoskrnl" / "rust-kernel-target").mkdir(parents=True)
    (bd / "hal" / "halx86" / "rust-hal-target").mkdir(parents=True)
    (bd / "dll" / "ntdll" / "rust-target").mkdir(parents=True)
    (bd / "x").mkdir(parents=True)
    (bd / "x" / "libkernel_rust.a").write_text("")
    targets = provenance.discover_rust_targets(bd)
    assert len(targets) == 3, f"should find all 3 target dirs, got {targets}"
    assert len(provenance.discover_rust_archives(bd)) == 1


def test_workspace_cargo_target_per_arch(tmp_path: Path):
    # the cargo workspace cache lives OUTSIDE build-<arch>; discover_rust_targets
    # can't see it, so --clean needs this to be genuinely cold (§9.6b / task #41).
    repo = tmp_path
    assert provenance.workspace_cargo_target(repo, "i386") is None  # absent -> None
    stem = repo / "sdk" / "lib" / "rust" / "target" / "i686-kernel"
    stem.mkdir(parents=True)
    assert provenance.workspace_cargo_target(repo, "i386") == stem
    # arch alias + per-arch stem resolution
    (repo / "sdk" / "lib" / "rust" / "target" / "x86_64-pc-windows-gnu").mkdir(parents=True)
    amd64_stem = provenance.workspace_cargo_target(repo, "amd64-uefi")
    assert amd64_stem is not None and amd64_stem.name == "x86_64-pc-windows-gnu"


def test_discover_rust_module_outputs(tmp_path: Path):
    # the copied cargo-module outputs --clean must delete so ninja re-links
    # (§stale-cargo / task #41). Only existing files are returned.
    repo = tmp_path
    bd = repo / "build-i386"
    (bd / "ntoskrnl").mkdir(parents=True)
    (bd / "win32ss").mkdir(parents=True)
    (bd / "ntoskrnl" / "ntoskrnl.exe").write_text("k")
    (bd / "win32ss" / "win32k.sys").write_text("w")
    # kernel32.dll absent -> not returned (no false unlink target)
    outs = provenance.discover_rust_module_outputs(repo, "i386")
    names = {p.name for p in outs}
    assert names == {"ntoskrnl.exe", "win32k.sys"}, names
    assert all(p.is_file() for p in outs)


# --- triage classification --------------------------------------------------


def test_triage_rust_panic():
    serial = "boot...\n!!! [NTDLL] RUST PANIC HIT !!!\nPANIC LOC: ldr/utils.rs:412\n"
    r = triage.classify(serial, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "RUST_PANIC"
    assert r["panic_loc"] == "ldr/utils.rs:412"


def test_triage_bugcheck_and_known_flake():
    # tWinMain_0xF4_race was RETIRED 2026-07-03 (verify-gone by absence): a 0xF4 after
    # _tWinMain no longer matches a known flake, so it BLOCKS as a plain BUGCHECK — a
    # recurrence must surface, not be auto-retried.
    serial = "[EXPLORER] _tWinMain entered\nBugCheck 0xF4 (CRITICAL_OBJECT...)\n"
    r = triage.classify(serial, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "BUGCHECK"
    assert r["bugcheck"] == "0xF4"
    assert r["known_flake"] is None
    # a different bugcheck with no flake match stays BUGCHECK
    r2 = triage.classify("BugCheck 0x7E (foo)\n", reached=False, fail_marker=None, debugcon=None)
    assert r2["classification"] == "BUGCHECK" and r2["bugcheck"] == "0x7E"


def test_triage_hang_surfaces_tails():
    r = triage.classify(
        "line1\nline2\nstuck...\n", reached=False, fail_marker=None, debugcon="A\nB\nC\n"
    )
    assert r["classification"] == "HANG_TIMEOUT"
    assert r["known_flake"] is None  # unsignatured hang stays a blocking HANG_TIMEOUT
    assert any("debugcon" in s for s in r["salient"])


def test_triage_known_hang_flakes():
    # nettest_explorer_race is RETIRED (root fix: winlogon gates the nettest.exe
    # spawn on NIC presence, so plain smokes never spawn it). A hang whose serial
    # somehow still carries the nettest marker now classifies as a plain
    # HANG_TIMEOUT — no longer masked as an expected flake.
    nettest = "...\n[WINLOGON] nettest.exe created (PID 268)\n[NETSTACK] TDI self-test: skipped\n"
    r = triage.classify(nettest, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "HANG_TIMEOUT"
    assert r["known_flake"] is None

    # explorer reaching _tWinMain is a normal (non-flake) hang either way
    r2 = triage.classify(
        nettest + "[EXPLORER] _tWinMain entered\n", reached=False, fail_marker=None, debugcon=None
    )
    assert r2["classification"] == "HANG_TIMEOUT" and r2["known_flake"] is None

    # arm64_lpcexitthread_race is RETIRED (root cause = undersized arm64 kernel
    # stack, fixed by the 0xC000 bump). An arm64 exit-path hang with the
    # LpcExitThread trace but no nettest marker now classifies as a plain
    # HANG_TIMEOUT (honest, no longer masked as an expected flake).
    arm = "[PS] PspExitThread Image=System -> about to call LpcExitThread\n"
    r3 = triage.classify(arm, reached=False, fail_marker=None, debugcon=None)
    assert r3["classification"] == "HANG_TIMEOUT"
    assert r3["known_flake"] is None

    # a hang-stage flake signature must NOT match a bugcheck (when-gated)
    r4 = triage.classify(
        "BugCheck 0x7E\n[PS] ... LpcExitThread ...\n",
        reached=False,
        fail_marker=None,
        debugcon=None,
    )
    assert r4["classification"] == "BUGCHECK"  # bugcheck branch, hang sig ignored


def test_triage_arm64_bugcheck_banner():
    # ke/bug.c prints `*** BUGCHECK: 0xNN …` (the canonical Rust/arm64 banner);
    # until 2026-07-04 the regex only knew `BugCheck 0xNN` / `*** STOP`, so a
    # real arm64 0xF4 classified as HANG_TIMEOUT — which the old gate policy
    # would then have auto-retried as a "flake".
    serial = "*** BUGCHECK: 0x000000F4 (0x03, 0xfffffa00063809, 0xfffffa000638, 0xfffff8004)\n"
    r = triage.classify(serial, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "BUGCHECK"
    assert r["bugcheck"] == "0x000000F4"


def test_faulting_pc_prefers_bugcheck_over_earlier_pc_text():
    # THE REGRESSION. symbolicate_pc used `_PC.search()`, which takes the FIRST
    # `PC=`/`RIP=` anywhere in the log. A real arm64 conformance boot carried an
    # earlier usermode trace, so the NtCreateSection 0x50 kernel fault was
    # symbolicated from a 0x7FF... usermode address and answered
    # `KTRAP_FRAME_LENGTH+8794868291436`. Confident + wrong relocates the hunt.
    serial = (
        "[FC] n=1 EC=0x25 FAR=0x700000000000 PC=0x000007ffb70018cc SP=0xffff8000055ceb20\n"
        "[KMTEST-SWEEP] launch NtCreateSection\n"
        "*** BUGCHECK: 0x00000050 (0xffff800004544040, 0x0000000000000000,"
        " 0xffff8000048595e0, 0x0000000000000000)\n"
    )
    # 0x50 PAGE_FAULT_IN_NONPAGED_AREA: P3 is the address that referenced memory.
    assert triage.faulting_pc(serial) == "0xffff8000048595e0"


def test_faulting_pc_uses_p2_for_1e():
    # 0x1E KMODE_EXCEPTION_NOT_HANDLED: P2 is where the exception occurred.
    # Observed on arm64 IoCreateFile; P2 is a driver-range PC, P3 is not.
    serial = (
        "*** BUGCHECK: 0x0000001E (0xffffffffc0000005, 0xfffff800421c960c,"
        " 0xffff80000480aca0, 0x0000000000000000)\n"
    )
    assert triage.faulting_pc(serial) == "0xfffff800421c960c"


def test_faulting_pc_takes_the_LAST_bugcheck():
    # A probe boot can log more than one crash; triage the one it died on.
    serial = (
        "*** BUGCHECK: 0x0000001E (0x1, 0x1111111111111111, 0x2, 0x0)\n"
        "*** BUGCHECK: 0x0000001E (0x1, 0x2222222222222222, 0x2, 0x0)\n"
    )
    assert triage.faulting_pc(serial) == "0x2222222222222222"


def test_faulting_pc_skips_zero_parameter():
    # A zeroed parameter carries no address — fall back rather than report 0x0.
    serial = "PC=0xffff800004001234\n*** BUGCHECK: 0x00000050 (0xa, 0x0, 0x0, 0x0)\n"
    assert triage.faulting_pc(serial) == "0xffff800004001234"


def test_symbolicate_pc_refuses_an_absurd_offset(tmp_path, monkeypatch):
    # A match 8.79 TRILLION bytes into a symbol is not a near miss; it means the
    # address is outside every known range. Returning it as an answer is worse
    # than returning nothing.
    tool = tmp_path / "qemu" / "addr2sym.sh"
    tool.parent.mkdir(parents=True)
    tool.write_text("#!/bin/sh\necho 'deadbeef\tKTRAP_FRAME_LENGTH+8794868291436\t-'\n")
    serial = "*** BUGCHECK: 0x0000001E (0x1, 0xffff800004001234, 0x2, 0x0)\n"
    out = triage.symbolicate_pc(tmp_path, "arm64", serial)
    assert out is not None
    assert "no symbol" in out
    assert "KTRAP_FRAME_LENGTH" not in out


def test_symbolicate_pc_passes_reloc_for_i386_only(tmp_path):
    # The index holds FILE VAs, a bugcheck reports RUNTIME VAs. Without --reloc
    # every i386 kernel PC lands past the end of the index and the offset guard
    # (correctly) declines — so the fix that stopped it LYING also made it unable
    # to ANSWER until the reloc was wired through. arm64's base is unknown, so it
    # must NOT get one and must keep declining.
    tool = tmp_path / "qemu" / "addr2sym.sh"
    tool.parent.mkdir(parents=True)
    # Echo the argv so the test can assert what was passed.
    tool.write_text('#!/bin/sh\necho "ARGS:$*"\n')
    serial = "*** BUGCHECK: 0x0000001E (0x1, 0x805148fd, 0x2, 0x0)\n"

    out_i386 = triage.symbolicate_pc(tmp_path, "i386", serial)
    assert out_i386 is not None
    assert "--reloc 0x800c3000" in out_i386, out_i386

    out_arm64 = triage.symbolicate_pc(tmp_path, "arm64", serial)
    assert out_arm64 is not None
    assert "--reloc" not in out_arm64, out_arm64


def test_symbolicate_pc_keeps_a_plausible_offset(tmp_path):
    tool = tmp_path / "qemu" / "addr2sym.sh"
    tool.parent.mkdir(parents=True)
    tool.write_text("#!/bin/sh\necho 'deadbeef\tMiInsertStandbyListAtFront+40\t-'\n")
    serial = "*** BUGCHECK: 0x0000001E (0x1, 0xffff800004001234, 0x2, 0x0)\n"
    out = triage.symbolicate_pc(tmp_path, "arm64", serial)
    assert out is not None
    assert "MiInsertStandbyListAtFront+40" in out


def test_triage_kernel_panic_via_debugcon():
    # kernel-mode Rust panic reaches debugcon 0xE9 only (serial goes silent) —
    # must classify RUST_PANIC (never retried), not HANG_TIMEOUT (retryable)
    r = triage.classify(
        "[SMSS] _main entered\n",
        reached=False,
        fail_marker=None,
        debugcon=None,
        debugcon_raw="KMPSC...PANIC:mm/ARM3/pagfault.rs:412\n",
    )
    assert r["classification"] == "RUST_PANIC"
    assert r["panic_loc"] == "mm/ARM3/pagfault.rs:412"
    assert any("debugcon" in s for s in r["salient"])


def test_triage_census_confirms_wedge():
    serial = (
        "[SMSS] _main entered\n"
        "[CENSUS] 3 thread(s) GateWait >10s (scan #4, synctrace dropped=0)\n"
        "[CENSUS] STUCK services.exe pid=724 tid=804 GateWait verdict=HOLDER-STUCK\n"
        "[CENSUS] STUCK csrss.exe pid=512 tid=520 GateWait (no synctrace entry)\n"
    )
    r = triage.classify(serial, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "HANG_TIMEOUT"
    assert r["wedge_confirmed"] is True
    assert r["census_stuck"] == 2
    assert any("wedge CONFIRMED" in s for s in r["salient"])


def test_triage_wedge_evidence_counts():
    # real emission shapes from ke/lockdep.rs (Phase A SELF-DEADLOCK, Phase B ABBA);
    # the "[LOCKDEP] N violation(s) recorded" header and reverse-order context
    # lines are deliberately NOT counted as violations.
    serial = (
        "noise\n"
        "[CENSUS] STUCK a\n"
        "[LOCKDEP] 2 violation(s) recorded (thread-slot drops=0, edge drops=0, edges=6200):\n"
        "[LOCKDEP]   SELF-DEADLOCK #1 thread=0xfff WAIT(shared) on lock=0xabc"
        " already held(excl) site=0x123\n"
        "[LOCKDEP]   ABBA #2 thr=0xfff holds 0xaaa(excl) acquiring 0xbbb(excl) site=0x456\n"
        "[LOCKDEP]        reverse order seen: thr=0xeee took 0xbbb before 0xaaa site=0x789\n"
        "noise\n"
    )
    ev = triage.wedge_evidence(serial)
    assert ev["census_stuck"] == 1 and ev["lockdep"] == 2 and ev["cswitch"] == 0
    assert triage.wedge_evidence("clean boot\n") == {
        "census_stuck": 0,
        "lockdep": 0,
        "cswitch": 0,
        "lines": [],
    }
    # cswitchdep emission shapes (ke/cswitchdep.rs): a loud line and a census
    # entry are both violations; the "census:" counters line is context only.
    cs = (
        "[CSWITCH] HIGH_IRQL_AT_SWITCH old=0xfff new=0x0 expected=2 actual=15 pc=0x123\n"
        "[CSWITCH] census: switches=8000 violations=1 first_runs=40 no_record=12"
        " evictions=30 drops=0\n"
        "[CSWITCH]   #1 HIGH_IRQL_AT_SWITCH old=0xfff new=0x0 expected=2 actual=15 pc=0x123\n"
    )
    ev = triage.wedge_evidence(cs)
    assert ev["cswitch"] == 2 and ev["lockdep"] == 0


def test_triage_post_boot_check():
    # clean settle window → None
    assert triage.post_boot_check("[EXPLORER] _tWinMain entered\nshell ready\n") is None
    # BugCheck AFTER the marker (previously a structural false green)
    r = triage.post_boot_check("[EXPLORER] _tWinMain entered\nBugCheck 0xF4 (0x3, ...)\n")
    assert r is not None and r["classification"] == "BUGCHECK" and r["bugcheck"] == "0xF4"
    # kernel panic that only reached debugcon
    r2 = triage.post_boot_check("_tWinMain entered\n", debugcon_raw="PANIC:ke/wait.rs:99\n")
    assert r2 is not None and r2["classification"] == "RUST_PANIC"
    assert r2["panic_loc"] == "ke/wait.rs:99"
    # user-mode panic in the settle window
    r3 = triage.post_boot_check("_tWinMain entered\npanicked at 'oops', tray.rs:7\n")
    assert r3 is not None and r3["classification"] == "RUST_PANIC"


# --- artifacts: summary.json is the machine-true verdict -------------------


def test_run_summary_json(tmp_path, monkeypatch):
    # setenv would do NOTHING here — artifacts.ARTIFACT_ROOT is resolved at import
    # time, so this Run used to land in the live artifact tree and, under the
    # gate's check-tooling stage, planted a fake FAIL/BUGCHECK summary.json that
    # was twice mistaken for a real verdict. conftest's autouse
    # _artifact_root_is_never_the_real_one now pins the root for every test.
    run = Run("smoke", "i386", ts=1700000000)
    run.add(Stage("staleness", status="ok"))
    run.add(Stage("boot", status="fail", detail="BUGCHECK", salient=["BugCheck 0x7E"]))
    run.classification = "BUGCHECK"
    rc = run.finalize()
    assert rc == 1, "a fail stage must yield nonzero exit"
    summary = json.loads((run.dir / "summary.json").read_text())
    assert summary["verdict"] == "FAIL"
    assert summary["classification"] == "BUGCHECK"
    assert summary["arch"] == "i386"


# --- cargo-link guard: skips cleanly when the PoCs aren't present ----------


def test_cargo_link_guard_skips_when_pocs_absent(tmp_path, monkeypatch):
    """The guard must no-op (skip, never fail) when the cargo-link PoCs are
    absent — so it never blocks `check --strict` on a tree without them. At the
    default arch (i386) all three PoCs apply."""
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    run = Run("check", "i386", ts=1700000000)
    lints.run_cargo_link_guard(run, tmp_path)  # empty repo → no PoCs
    stages = {s.name: s for s in run.stages}
    for label in ("ntoskrnl", "ntdll", "ntdll_vista"):
        name = f"cargo_link:{label}:i386"
        assert name in stages, f"guard must emit a {name} stage"
        assert stages[name].status == "skip"
        assert "PoC absent" in stages[name].detail


def test_cargo_link_guard_amd64_runs_all_pocs(tmp_path, monkeypatch):
    """On amd64, all three cargo-link PoCs are attempted (and skip cleanly on an
    empty repo). ntoskrnl-cargo became arch-general in cdcfc9fed55 — it is no
    longer i386-only — so amd64 must exercise it too."""
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    run = Run("check", "amd64", ts=1700000000)
    lints.run_cargo_link_guard(run, tmp_path, "amd64")
    names = {s.name for s in run.stages}
    for label in ("ntoskrnl", "ntdll", "ntdll_vista"):
        assert f"cargo_link:{label}:amd64" in names, f"{label} PoC must run on amd64"


def test_cargo_link_guard_respects_arch_filter(tmp_path, monkeypatch):
    """The guard's per-PoC arch filter (`arch in arches`) must still gate: a PoC
    declared for i386 only is not attempted on amd64. Verified against a synthetic
    PoC table so the mechanism stays covered even though no real PoC is arch-limited
    today."""
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    monkeypatch.setattr(
        lints,
        "_CARGO_LINK_POCS",
        [
            ("i386only", "i386only-cargo", "i386only", ("i386",), 1_000),
            ("everywhere", "everywhere-cargo", "everywhere", ("i386", "amd64", "arm64"), 1_000),
        ],
    )
    run = Run("check", "amd64", ts=1700000000)
    lints.run_cargo_link_guard(run, tmp_path, "amd64")
    names = {s.name for s in run.stages}
    assert "cargo_link:i386only:amd64" not in names  # filtered out on amd64
    assert "cargo_link:everywhere:amd64" in names


# --- cachecfg: VIBEOS_ROOT resolution, env overlay, stats parsing ----------

from . import cachecfg  # noqa: E402


def test_vibeos_root_env_wins(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path / "custom"))
    assert cachecfg.vibeos_root(tmp_path / "repo") == (tmp_path / "custom").resolve()


def test_vibeos_root_defaults_to_repo_parent(monkeypatch):
    monkeypatch.delenv("VIBEOS_ROOT", raising=False)
    # main checkout and a sibling worktree must resolve to the SAME root → shared
    # CCACHE_BASEDIR → cross-worktree hits.
    parent = Path("/home/u/os-playground")
    assert cachecfg.vibeos_root(parent / "vibeos") == parent
    assert cachecfg.vibeos_root(parent / "vibeos-wt-x") == parent


def test_cache_enabled_flag_and_env(monkeypatch):
    from types import SimpleNamespace

    monkeypatch.delenv("VIBEOS_BUILD_CACHE", raising=False)
    assert cachecfg.cache_enabled(SimpleNamespace(cache=None)) is True  # mandatory default
    assert cachecfg.cache_enabled(SimpleNamespace(cache=True)) is True
    assert cachecfg.cache_enabled(SimpleNamespace(cache=False)) is False  # --no-cache opt-out
    monkeypatch.setenv("VIBEOS_BUILD_CACHE", "0")
    assert cachecfg.cache_enabled(SimpleNamespace(cache=None)) is False  # env opt-out
    assert cachecfg.cache_enabled(SimpleNamespace(cache=True)) is True  # --cache beats env
    monkeypatch.setenv("VIBEOS_BUILD_CACHE", "1")
    assert cachecfg.cache_enabled(SimpleNamespace(cache=False)) is False  # --no-cache beats env
    assert cachecfg.cache_enabled(SimpleNamespace(cache=None)) is True


def test_cache_env_overlay_shape(tmp_path):
    ce = cachecfg.CacheEnv(
        root=tmp_path,
        ccache_dir=tmp_path / "ccache",
        sccache_dir=tmp_path / "sccache",
        ccache_bin="/usr/bin/ccache",
        sccache_bin="/usr/local/bin/sccache",
    )
    e = ce.env()
    assert e["CCACHE_BASEDIR"] == str(tmp_path)  # basedir == worktree parent
    assert e["CCACHE_NOHASHDIR"] == "true"
    assert e["CCACHE_DIR"] == str(tmp_path / "ccache")
    assert e["RUSTC_WRAPPER"] == "/usr/local/bin/sccache"
    assert e["CARGO_INCREMENTAL"] == "0"
    assert e["SCCACHE_DIR"] == str(tmp_path / "sccache")
    # critical safety: we never set RUSTFLAGS (would clobber config.toml's
    # load-bearing i386 -sse,-sse2 kernel guard).
    assert "RUSTFLAGS" not in e


def test_cache_env_absent_tools(tmp_path):
    ce = cachecfg.CacheEnv(tmp_path, tmp_path / "c", tmp_path / "s", None, None)
    assert ce.env() == {}
    assert not ce.use_ccache and not ce.use_sccache


def test_parse_ccache_stats():
    txt = (
        "direct_cache_hit\t7\n"
        "preprocessed_cache_hit\t3\n"
        "cache_miss\t10\n"
        "primary_storage_hit\t99\n"  # must be IGNORED (not double-counted)
    )
    st = cachecfg.parse_ccache_stats(txt)
    assert st["hits"] == 10 and st["misses"] == 10
    assert abs(st["hit_rate"] - 0.5) < 1e-9


def test_parse_sccache_json_bucketed():
    txt = json.dumps(
        {
            "stats": {
                "compile_requests": 20,
                "cache_hits": {"counts": {"Rust": 12}},
                "cache_misses": {"counts": {"Rust": 8}},
            }
        }
    )
    st = cachecfg.parse_sccache_json(txt)
    assert st["requests"] == 20 and st["hits"] == 12 and st["misses"] == 8
    assert abs(st["hit_rate"] - 0.6) < 1e-9


def test_parse_sccache_json_scalar_and_garbage():
    st = cachecfg.parse_sccache_json(json.dumps({"stats": {"cache_hits": 5, "cache_misses": 0}}))
    assert st["hits"] == 5 and st["hit_rate"] == 1.0
    assert cachecfg.parse_sccache_json("not json") == {}


# --- per-arch QEMU concurrency sub-cap (serialize same-arch boots) ---------
import types  # noqa: E402

from .. import vm  # noqa: E402


def test_per_arch_cap_defaults(monkeypatch):
    for v in (
        "VIBEOS_QEMU_PER_ARCH_CAP",
        "VIBEOS_QEMU_CAP_I386",
        "VIBEOS_QEMU_CAP_AMD64",
        "VIBEOS_QEMU_CAP_ARM64",
    ):
        monkeypatch.delenv(v, raising=False)
    # KVM arches serialize (1); arm64 TCG uncapped (0). Aliases resolve to base.
    assert vm.per_arch_cap("i386") == 1
    assert vm.per_arch_cap("i386-uefi") == 1
    assert vm.per_arch_cap("amd64") == 1
    assert vm.per_arch_cap("amd64-uefi") == 1
    assert vm.per_arch_cap("arm64") == 0


def test_per_arch_cap_env_overrides(monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_PER_ARCH_CAP", "3")  # blanket KVM override
    assert vm.per_arch_cap("i386") == 3 and vm.per_arch_cap("amd64") == 3
    monkeypatch.setenv("VIBEOS_QEMU_CAP_I386", "1")  # per-arch beats blanket
    assert vm.per_arch_cap("i386") == 1 and vm.per_arch_cap("amd64") == 3
    monkeypatch.setenv("VIBEOS_QEMU_CAP_ARM64", "2")  # cap arm64 if TCG starves
    assert vm.per_arch_cap("arm64") == 2


def _bare_vm(arch: str):
    """An uninitialized QemuVM carrying only what the slot methods touch."""
    v = vm.QemuVM.__new__(vm.QemuVM)
    v.config = types.SimpleNamespace(arch=arch)  # type: ignore[assignment]
    v._slot_fd = None
    v._arch_slot_fd = None
    v._cpu_lease_fd = None
    return v


def test_arch_slot_serializes_same_arch(tmp_path, monkeypatch):
    monkeypatch.setattr(vm, "SLOT_DIR", tmp_path)
    monkeypatch.setattr(vm, "QEMU_QUEUE_TIMEOUT", 0.0)  # second boot fails fast
    for var in ("VIBEOS_QEMU_PER_ARCH_CAP", "VIBEOS_QEMU_CAP_I386"):
        monkeypatch.delenv(var, raising=False)

    first = _bare_vm("i386")
    first._acquire_arch_slot()
    assert first._arch_slot_fd is not None  # holds the single i386 slot

    # A second i386 boot can't get the cap-1 slot → queues → times out.
    second = _bare_vm("i386")
    with pytest.raises(TimeoutError):
        second._acquire_arch_slot()
    assert second._arch_slot_fd is None

    # Cross-arch is uncapped → arm64 acquires immediately alongside i386.
    other = _bare_vm("arm64")
    other._acquire_arch_slot()
    assert other._arch_slot_fd is None  # cap 0: no lock taken, no contention

    # Free the first → the queued arch is available again.
    first._release_slot()
    third = _bare_vm("i386")
    third._acquire_arch_slot()
    assert third._arch_slot_fd is not None
    third._release_slot()


# --- host CPU broker: boot-side lease wiring (vm.py) ------------------------
def test_boot_is_tcg_native_arm64_yes(monkeypatch):
    v = _bare_vm("arm64")
    v._backend = "native"
    assert v._boot_is_tcg() is True  # no aarch64 KVM in WSL2 → always TCG


def test_boot_is_tcg_native_x86_follows_kvm(monkeypatch):
    v = _bare_vm("amd64")
    v._backend = "native"
    monkeypatch.setattr(vm.Path, "exists", lambda self: True)  # /dev/kvm present
    assert v._boot_is_tcg() is False  # KVM → no CPU lease
    monkeypatch.setattr(vm.Path, "exists", lambda self: False)  # no /dev/kvm
    assert v._boot_is_tcg() is True  # QEMU degrades to TCG → CPU-heavy


def test_boot_is_tcg_windows_host_whpx_no(monkeypatch):
    v = _bare_vm("amd64")
    v._backend = "windows-host"
    v._win_accel = "whpx"
    assert v._boot_is_tcg() is False  # hardware virt on the Windows host → no lease
    v._win_accel = "tcg"
    assert v._boot_is_tcg() is True


def test_acquire_cpu_lease_tcg_takes_then_release(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    v = _bare_vm("arm64")
    v._backend = "native"
    v.work_dir = tmp_path
    v._acquire_cpu_lease()
    assert v._cpu_lease_fd is not None  # a TCG boot holds one CPU permit
    v._release_slot()  # _release_slot also drops the CPU lease
    assert v._cpu_lease_fd is None


def test_acquire_cpu_lease_kvm_takes_nothing(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    v = _bare_vm("amd64")
    v._backend = "native"
    v.work_dir = tmp_path
    monkeypatch.setattr(vm.Path, "exists", lambda self: True)  # /dev/kvm present → KVM
    v._acquire_cpu_lease()
    assert v._cpu_lease_fd is None  # KVM boot takes no CPU lease


def test_acquire_cpu_lease_noop_when_broker_disabled(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "0")
    v = _bare_vm("arm64")
    v._backend = "native"
    v.work_dir = tmp_path
    v._acquire_cpu_lease()
    assert v._cpu_lease_fd is None  # broker off → no lease even for a TCG boot


def test_acquire_cpu_lease_soft_fails_on_timeout(tmp_path, monkeypatch, capsys):
    """When no permit is free (an exclusive build holds the pool), the boot lease times out
    → warn + proceed UNLOCKED, never wedging the boot (the writer-takes-all soft-fail seam)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    monkeypatch.setattr(hl_mod, "acquire_cpu_lease", lambda *a, **k: None)  # simulate timeout
    v = _bare_vm("arm64")
    v._backend = "native"
    v.work_dir = tmp_path
    v._acquire_cpu_lease()
    assert v._cpu_lease_fd is None  # never wedged
    assert "proceeding UNLOCKED" in capsys.readouterr().out


# --- windows-host hostfwd-bind TOCTOU launch retry -------------------------
class _Reader:
    """Minimal proc.stderr stand-in (read() returns the captured bytes once)."""

    def __init__(self, data: bytes = b""):
        self._data = data

    def read(self) -> bytes:
        return self._data


class _FakeProc:
    """subprocess.Popen stand-in for _launch_qemu tests."""

    def __init__(self, stderr: bytes = b""):
        self.stderr = _Reader(stderr)


def _launch_vm(backend: str = "windows-host"):
    """A bare QemuVM carrying only what _launch_qemu touches."""
    v = vm.QemuVM.__new__(vm.QemuVM)
    v._backend = backend
    v._proc = None
    v._early_stderr = ""
    v._gdb = False
    return v


def test_launch_qemu_retries_hostfwd_bind_race(monkeypatch):
    v = _launch_vm("windows-host")
    monkeypatch.setattr(
        v,
        "_build_windows_cmd",
        lambda: ["qemu", "-netdev", "user,id=n,hostfwd=tcp:0.0.0.0:1234-:7777"],
    )
    procs = [_FakeProc(), _FakeProc(), _FakeProc()]
    launched: list = []

    def fake_popen(cmd, **kw):
        p = procs[len(launched)]
        launched.append(p)
        return p

    monkeypatch.setattr(vm.subprocess, "Popen", fake_popen)
    # first two attempts lose the port race, third launches clean (None = healthy)
    seq = iter([vm.HOSTFWD_BIND_ERROR + " 'tcp:0.0.0.0:1234'", vm.HOSTFWD_BIND_ERROR, None])
    monkeypatch.setattr(v, "_early_exit_stderr", lambda proc, budget=2.0: next(seq))
    discarded: list = []
    monkeypatch.setattr(v, "_discard_failed_launch", discarded.append)

    cmd = v._launch_qemu()
    assert "hostfwd=" in " ".join(cmd)
    assert v._proc is procs[2]  # the healthy (3rd) attempt
    assert discarded == [procs[0], procs[1]]  # first two torn down
    assert len(launched) == 3


def test_launch_qemu_healthy_first_try_no_retry(monkeypatch):
    v = _launch_vm("windows-host")
    monkeypatch.setattr(
        v, "_build_windows_cmd", lambda: ["qemu", "-netdev", "user,hostfwd=tcp:0.0.0.0:9-:9"]
    )
    proc = _FakeProc()
    launched: list = []
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: launched.append(proc) or proc)
    monkeypatch.setattr(v, "_early_exit_stderr", lambda p, budget=2.0: None)
    monkeypatch.setattr(
        v, "_discard_failed_launch", lambda p: pytest.fail("healthy launch must not discard")
    )
    v._launch_qemu()
    assert v._proc is proc
    assert len(launched) == 1


def test_launch_qemu_non_hostfwd_skips_poll(monkeypatch):
    # native (or any non-hostfwd) config: launch once, never poll for the bind race.
    v = _launch_vm("native")
    monkeypatch.setattr(v, "_build_native_cmd", lambda: ["qemu", "-M", "pc"])
    proc = _FakeProc()
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: proc)
    monkeypatch.setattr(
        v, "_early_exit_stderr", lambda *a, **k: pytest.fail("must not poll a non-hostfwd cmd")
    )
    cmd = v._launch_qemu()
    assert v._proc is proc
    assert cmd == ["qemu", "-M", "pc"]


def test_launch_qemu_non_bind_early_exit_is_not_retried(monkeypatch):
    # QEMU dies early for some OTHER reason → not the flake: keep the proc (the normal
    # start path surfaces it), stash stderr, do NOT retry.
    v = _launch_vm("windows-host")
    monkeypatch.setattr(v, "_build_windows_cmd", lambda: ["qemu", "hostfwd=tcp:0.0.0.0:1-:1"])
    proc = _FakeProc()
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: proc)
    monkeypatch.setattr(v, "_early_exit_stderr", lambda p, budget=2.0: "bad -drive: No such file")
    monkeypatch.setattr(v, "_discard_failed_launch", lambda p: pytest.fail("must not retry"))
    v._launch_qemu()
    assert v._proc is proc
    assert v._early_stderr == "bad -drive: No such file"


def test_launch_qemu_raises_after_exhausting_retries(monkeypatch):
    monkeypatch.setattr(vm, "HOSTFWD_LAUNCH_RETRIES", 2)
    v = _launch_vm("windows-host")
    monkeypatch.setattr(v, "_build_windows_cmd", lambda: ["qemu", "hostfwd=tcp:0.0.0.0:1-:1"])
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: _FakeProc())
    monkeypatch.setattr(v, "_early_exit_stderr", lambda p, budget=2.0: vm.HOSTFWD_BIND_ERROR)
    monkeypatch.setattr(v, "_discard_failed_launch", lambda p: None)
    with pytest.raises(RuntimeError, match="hostfwd-bind failed after 2"):
        v._launch_qemu()


def test_launch_qemu_retries_gdb_bind_race(monkeypatch):
    # A windows-host gdbstub binds a Windows-stack port with the same pick-then-bind
    # TOCTOU as a hostfwd → must be retried with a fresh port (no hostfwd present).
    v = _launch_vm("windows-host")
    v._gdb = True
    monkeypatch.setattr(v, "_build_windows_cmd", lambda: ["qemu", "-gdb", "tcp:0.0.0.0:1234"])
    procs = [_FakeProc(), _FakeProc()]
    launched: list = []

    def fake_popen(cmd, **kw):
        p = procs[len(launched)]
        launched.append(p)
        return p

    monkeypatch.setattr(vm.subprocess, "Popen", fake_popen)
    seq = iter([vm.GDB_BIND_ERROR + " on device 'tcp:0.0.0.0:1234'", None])
    monkeypatch.setattr(v, "_early_exit_stderr", lambda proc, budget=2.0: next(seq))
    discarded: list = []
    monkeypatch.setattr(v, "_discard_failed_launch", discarded.append)

    cmd = v._launch_qemu()
    assert "-gdb" in cmd
    assert v._proc is procs[1]  # healthy 2nd attempt
    assert discarded == [procs[0]]  # first (raced) torn down


def _gdb_vm(gdb: bool = False, wait: bool = False, backend: str = "native"):
    v = vm.QemuVM.__new__(vm.QemuVM)
    v._backend = backend
    v._gdb = gdb
    v._gdb_wait = wait
    v.gdb_port = None
    return v


def test_gdb_args_off_emits_nothing():
    assert _gdb_vm(gdb=False)._gdb_args(win_stack=False) == []
    assert _gdb_vm(gdb=False)._gdb_args(win_stack=True) == []  # even on the win path


def test_gdb_args_native_binds_loopback():
    v = _gdb_vm(gdb=True)
    a = v._gdb_args(win_stack=False)
    assert a[0] == "-gdb" and a[1].startswith("tcp:127.0.0.1:") and "-S" not in a
    assert v.gdb_port is not None and str(v.gdb_port) in a[1]


def test_gdb_args_wait_adds_dash_S():
    assert _gdb_vm(gdb=True, wait=True)._gdb_args(win_stack=False)[-1] == "-S"


def test_gdb_args_windows_binds_wildcard_on_windows_port(monkeypatch):
    # windows-host: QEMU.exe is the gdb SERVER → bind 0.0.0.0 on a WINDOWS-picked port.
    monkeypatch.setattr(be_mod, "windows_free_tcp_port", lambda: 54321)
    v = _gdb_vm(gdb=True, backend="windows-host")
    assert v._gdb_args(win_stack=True) == ["-gdb", "tcp:0.0.0.0:54321"]
    assert v.gdb_port == 54321


def test_gdb_host_native_vs_windows(monkeypatch):
    assert _gdb_vm(backend="native").gdb_host == "127.0.0.1"
    monkeypatch.setattr(be_mod, "gateway_ip", lambda: "172.26.176.1")
    assert _gdb_vm(backend="windows-host").gdb_host == "172.26.176.1"


# --- gdbengine: the Python gdb engine's pure logic (no QEMU) ---------------


def test_gdbengine_gdb_arch_map():
    assert ge_mod._gdb_arch("amd64") == "i386:x86-64"
    assert ge_mod._gdb_arch("i386") == "i386"
    assert ge_mod._gdb_arch("arm64") == "aarch64"


def test_gdbengine_expand_verb():
    assert ge_mod._expand_verb("bt 50") == "bt-tagged 50"
    assert ge_mod._expand_verb("bt") == "bt-tagged"  # no trailing space
    assert ge_mod._expand_verb("triage-bugcheck") == "triage-bugcheck"
    assert ge_mod._expand_verb("dump-thread 0xffff") == "dump-thread 0xffff"
    assert ge_mod._expand_verb("raw info registers rax") == "info registers rax"
    with pytest.raises(ValueError, match="unknown verb"):
        ge_mod._expand_verb("frobnicate")


def test_gdbengine_all_helper_files_exist():
    missing = [h for h in ge_mod.HELPER_FILES if not (ge_mod.GDB_HELPERS / h).is_file()]
    assert missing == [], f"missing gdb helpers: {missing}"
    # _helper_source_args emits a `source <path>` -ex pair per existing helper
    args = ge_mod._helper_source_args()
    assert args.count("-ex") == len(ge_mod.HELPER_FILES)


def test_gdbengine_manifest_roundtrip(tmp_path, monkeypatch):
    monkeypatch.setattr(ge_mod, "RUNS_DIR", tmp_path)
    run = ge_mod.DebugRun(
        run_id="amd64-xy",
        arch="amd64",
        backend="windows-host",
        gdb_host="172.26.176.1",
        gdb_port=54321,
        serial_log="/mnt/c/x/serial.log",
        kernel_image="/k/ntoskrnl.exe",
        win_uid="vibeos-dbg-1-2",
    )
    run.save()
    got = ge_mod.DebugRun.load("amd64-xy")
    assert got.gdb_port == 54321 and got.win_uid == "vibeos-dbg-1-2" and got.arch == "amd64"


def test_gdbengine_load_rejects_foreign_manifest(tmp_path, monkeypatch):
    # qemu/runs/ is shared with the bash toolkit, whose manifest lacks our fields.
    monkeypatch.setattr(ge_mod, "RUNS_DIR", tmp_path)
    (tmp_path / "bash-run").mkdir()
    (tmp_path / "bash-run" / "manifest.json").write_text('{"run_id":"bash-run","qemu_pid":9}')
    with pytest.raises(ValueError, match="not a gdbengine run manifest"):
        ge_mod.DebugRun.load("bash-run")


def test_gdbengine_status_uses_process_liveness_not_gdb_connect(monkeypatch):
    # Connecting to QEMU's gdbstub HALTS the guest — it wedged early boots (serial
    # stuck at 0, PC parked in OVMF). status() must report liveness via the Windows
    # process, never a socket connect to gdb_host:gdb_port. Guard against regressing.
    seen = {"uid": None}
    monkeypatch.setattr(ge_mod, "_win_alive", lambda uid: seen.__setitem__("uid", uid) or True)
    run = ge_mod.DebugRun(
        run_id="r",
        arch="amd64",
        backend="windows-host",
        gdb_host="1.2.3.4",
        gdb_port=9,
        serial_log="/does/not/exist",
        kernel_image="/k",
        win_uid="vibeos-dbg-1-9",
    )
    st = ge_mod.status(run)
    assert st["alive"] is True and seen["uid"] == "vibeos-dbg-1-9"
    assert "gdb_reachable" not in st  # the halting probe must stay gone
    assert st["serial_bytes"] == 0
    # tight-poll variant skips even the process query
    assert "alive" not in ge_mod.status(run, check_alive=False)


def test_gdbengine_run_until_builds_break_and_then(monkeypatch):
    # run_until must emit `break <sym>`, `bounded-continue <t>`, then each expanded verb.
    captured = {}

    def fake_run_gdb(run, verb_ex, *, timeout):
        captured["ex"] = verb_ex
        captured["timeout"] = timeout
        return "BREAKPOINT HIT"

    monkeypatch.setattr(ge_mod, "_run_gdb", fake_run_gdb)
    run = ge_mod.DebugRun.__new__(ge_mod.DebugRun)
    out = ge_mod.run_until(run, "KeBugCheckEx", timeout=30, then=["bt 10", "triage-bugcheck"])
    ex = captured["ex"]
    assert "break KeBugCheckEx" in ex and "bounded-continue 30" in ex
    assert "bt-tagged 10" in ex and "triage-bugcheck" in ex
    assert captured["timeout"] == 30 + 30  # in-gdb budget + teardown headroom
    assert out == "BREAKPOINT HIT"


def test_early_exit_stderr_reports_exit_vs_healthy():
    v = vm.QemuVM.__new__(vm.QemuVM)

    class Exited:
        stderr = _Reader(b"Could not set up host forwarding rule 'tcp:0.0.0.0:1234'")

        def poll(self):
            return 1

    out = v._early_exit_stderr(Exited(), budget=0.3)  # type: ignore[arg-type]
    assert "host forwarding" in (out or "")

    class Alive:
        stderr = None

        def poll(self):
            return None

    alive = v._early_exit_stderr(Alive(), budget=0.15)  # type: ignore[arg-type]
    assert alive is None


# --- prune: idle-worktree GC eligibility gates -----------------------------
from . import prune  # noqa: E402


def _classify(
    wt,
    current,
    *,
    live_qemu=None,
    active_build=None,
    allow_self=False,
    min_age_hours=0.0,
    now=1000.0,
    newest_mtime=None,
):
    return prune.classify(
        Path(wt),
        current=Path(current),
        live_qemu=live_qemu or set(),
        active_build=active_build or set(),
        allow_self=allow_self,
        min_age_hours=min_age_hours,
        now=now,
        newest_mtime=newest_mtime,
    )


def test_prune_gates_current_live_and_build():
    # current worktree is protected unless --self
    assert _classify("/wt/a", "/wt/a") == "current worktree (use --self)"
    assert _classify("/wt/a", "/wt/a", allow_self=True) is None
    # live QEMU and in-flight build are hard skips
    assert _classify("/wt/b", "/wt/a", live_qemu={Path("/wt/b")}) == "live QEMU running"
    assert _classify("/wt/b", "/wt/a", active_build={Path("/wt/b")}) == "build in flight"
    # otherwise prunable
    assert _classify("/wt/b", "/wt/a") is None


def test_prune_idle_hours_filter():
    now = 100_000.0
    # touched 1h ago, threshold 24h -> held
    assert (
        _classify("/wt/b", "/wt/a", min_age_hours=24, now=now, newest_mtime=now - 3600)
        == "touched < 24h ago"
    )
    # touched 48h ago, threshold 24h -> prunable
    assert (
        _classify("/wt/b", "/wt/a", min_age_hours=24, now=now, newest_mtime=now - 48 * 3600) is None
    )
    # threshold 0 disables the age gate even with a fresh mtime
    assert _classify("/wt/b", "/wt/a", min_age_hours=0, now=now, newest_mtime=now) is None


def test_prune_attribute_longest_match_wins():
    wts = [Path("/wt"), Path("/wt/nested")]
    assert prune._attribute("/wt/nested/build-i386/x.o", wts) == Path("/wt/nested")
    assert prune._attribute("/wt/build-i386/x.o", wts) == Path("/wt")
    assert prune._attribute("/elsewhere/x", wts) is None


# --- repro: PE parser + reproducibility compare (det-plan rung 0) ----------


def _mk_pe(timestamp: int, sections: list[tuple[str, bytes]], machine: int = 0x8664) -> bytes:
    """Build a minimal valid PE image (COFF header + section table + bodies, no optional
    header) for testing the determinism parser. Mirrors the layout parse_pe reads."""
    n = len(sections)
    pe_off = 0x40
    head = bytearray(b"\x00" * pe_off)
    head[0:2] = b"MZ"
    head[0x3C:0x40] = struct.pack("<I", pe_off)
    # COFF: machine, num_sections, timestamp, ptr_symtab, num_syms, size_opt, characteristics
    coff = b"PE\x00\x00" + struct.pack("<HHIIIHH", machine, n, timestamp, 0, 0, 0, 0)
    sec_table_off = pe_off + len(coff)
    cursor = sec_table_off + 40 * n
    headers = b""
    bodies = b""
    for name, body in sections:
        nm = name.encode("latin-1")[:8].ljust(8, b"\x00")
        # virtsize, virtaddr, rawsize, rawoff, then 12 zero bytes + characteristics
        headers += nm + struct.pack("<IIII", len(body), 0x1000, len(body), cursor)
        headers += b"\x00" * 12 + struct.pack("<I", 0)
        bodies += body
        cursor += len(body)
    return bytes(head) + coff + headers + bodies


def test_repro_parse_pe_fields():
    img = _mk_pe(0x12345678, [(".text", b"\xc3" * 16), (".data", b"\x00" * 8)])
    pe = repro.parse_pe(img)
    assert pe.machine == 0x8664
    assert pe.timestamp == 0x12345678
    assert [s.name for s in pe.sections] == [".text", ".data"]
    assert pe.sections[0].raw_size == 16


def test_repro_parse_pe_rejects_non_pe():
    with pytest.raises(ValueError):
        repro.parse_pe(b"not a pe at all" * 8)


def test_repro_compare_identical():
    img = _mk_pe(0xCAFEBABE, [(".text", b"\x90" * 32)])
    rep = repro.compare(img, img)
    assert rep.identical
    assert not rep.timestamp_differs
    assert rep.section_diffs == ()


def test_repro_compare_timestamp_only_is_the_leak_signature():
    # Two builds identical except the PE TimeDateStamp — the measured live leak. The
    # whole-file digest differs but NO section body does → localizes to the timestamp.
    a = _mk_pe(0x10000000, [(".text", b"\x90" * 32), (".rdata", b"abc")])
    b = _mk_pe(0x20000000, [(".text", b"\x90" * 32), (".rdata", b"abc")])
    rep = repro.compare(a, b)
    assert not rep.identical
    assert rep.timestamp_differs
    assert rep.section_diffs == ()


def test_repro_compare_localizes_changed_section():
    a = _mk_pe(0x10000000, [(".text", b"\x90" * 32), (".data", b"old")])
    b = _mk_pe(0x10000000, [(".text", b"\x90" * 32), (".data", b"new")])
    rep = repro.compare(a, b)
    assert not rep.identical
    assert not rep.timestamp_differs
    assert rep.section_diffs == (".data",)


def test_repro_verify_parser_wires_arch_and_dispatch():
    # rung 5: `repro verify --arch i386` routes to repro_mod.repro with repro_cmd=verify
    from .__main__ import build_parser

    ns = build_parser().parse_args(["repro", "verify", "--arch", "i386"])
    assert ns.repro_cmd == "verify"
    assert ns.arch == "i386"
    assert ns.fn is repro.repro


# --- build: SOURCE_DATE_EPOCH commit-date derivation (det-plan rung 2b) -----


def test_source_date_epoch_is_commit_ct(tiny_repo: Path):
    sde = build_mod.source_date_epoch(tiny_repo)
    assert sde is not None and sde.isdigit()
    # must equal the HEAD commit's %ct (deterministic per commit, not wall-clock)
    expected = subprocess.run(
        ["git", "log", "-1", "--format=%ct"],
        cwd=tiny_repo,
        capture_output=True,
        text=True,
    ).stdout.strip()
    assert sde == expected


def test_source_date_epoch_none_outside_git(tmp_path: Path):
    assert build_mod.source_date_epoch(tmp_path) is None


# --- config: layered defaults + local override + env (windows-host backend) -


def test_config_defaults_when_no_overrides(tmp_path: Path):
    cfg = config_mod.load(tmp_path, environ={})
    assert cfg.get("qemu.backend") == "windows-host"
    assert cfg.get("qemu.windows.accel") == "tcg"  # whpx is opt-in (doesn't boot x86 cleanly yet)
    assert cfg.get("nonexistent.key", "fallback") == "fallback"


def test_config_local_file_deep_merges(tmp_path: Path):
    (tmp_path / config_mod.LOCAL_FILENAME).write_text(
        json.dumps({"qemu": {"backend": "windows-host", "windows": {"accel": "whpx"}}})
    )
    cfg = config_mod.load(tmp_path, environ={})
    assert cfg.get("qemu.backend") == "windows-host"
    assert cfg.get("qemu.windows.accel") == "whpx"  # local override beats the tcg default
    # untouched sibling defaults survive the deep merge
    assert cfg.get("qemu.windows.host_addr") == "127.0.0.1"


def test_config_env_beats_local_and_defaults(tmp_path: Path):
    (tmp_path / config_mod.LOCAL_FILENAME).write_text(
        json.dumps({"qemu": {"backend": "windows-host"}})
    )
    cfg = config_mod.load(tmp_path, environ={"VIBEOS_QEMU_BACKEND": "native"})
    assert cfg.get("qemu.backend") == "native"  # env wins over local


def test_config_malformed_local_file_falls_back_to_defaults(tmp_path: Path):
    (tmp_path / config_mod.LOCAL_FILENAME).write_text("{ not valid json")
    cfg = config_mod.load(tmp_path, environ={})
    assert cfg.get("qemu.backend") == "windows-host"


# --- TCP transport (windows-host backend: WSL listens, QEMU connects) -------


def test_serial_client_tcp_drains_and_reads_marker():
    s = serial_client.SerialClient()
    port = s.listen_tcp("127.0.0.1", 0)
    assert port > 0

    def fake_qemu():
        c = socket.create_connection(("127.0.0.1", port), timeout=5)
        c.sendall(b"booting...\n[MARK] WORLD ready\n")
        time.sleep(0.3)
        c.close()

    t = threading.Thread(target=fake_qemu, daemon=True)
    t.start()
    s.accept(timeout=5)
    out = s.read_until("WORLD", timeout=5)
    assert "WORLD" in out
    s.close()
    t.join(timeout=5)


def test_qmp_client_tcp_handshake_and_execute():
    qmp = qmp_client.QmpClient()
    port = qmp.listen_tcp("127.0.0.1", 0)
    assert port > 0
    greeting = {"QMP": {"version": {"qemu": {"major": 11}}, "capabilities": []}}

    def fake_qemu():
        c = socket.create_connection(("127.0.0.1", port), timeout=5)
        c.sendall(json.dumps(greeting).encode() + b"\n")

        def read_line() -> bytes:
            buf = b""
            while b"\n" not in buf:
                buf += c.recv(4096)
            return buf

        read_line()  # qmp_capabilities
        c.sendall(json.dumps({"return": {}}).encode() + b"\n")
        read_line()  # query-status
        c.sendall(json.dumps({"return": {"status": "running"}}).encode() + b"\n")
        c.close()

    t = threading.Thread(target=fake_qemu, daemon=True)
    t.start()
    assert qmp.accept(timeout=5) == greeting
    assert qmp.execute("query-status")["return"]["status"] == "running"
    qmp.close()
    t.join(timeout=5)


# --- backend helpers (windows-host path translation / arg rewrite) ----------


def test_backend_win_path():
    assert be_mod.win_path("/mnt/c/temp/x/livecd.iso") == "C:/temp/x/livecd.iso"
    assert be_mod.win_path("/mnt/d/qemu/OVMF.fd") == "D:/qemu/OVMF.fd"
    with pytest.raises(ValueError):
        be_mod.win_path("/home/curi/not/under/mnt")


def test_backend_windows_qemu_bin():
    d = "/mnt/c/Program Files/qemu"
    # tcg keeps the per-arch exe
    assert be_mod.windows_qemu_bin("qemu-system-i386", d, "tcg").endswith("qemu-system-i386.exe")
    # whpx only exists in the x86_64 emulator → i386 guest uses it
    assert be_mod.windows_qemu_bin("qemu-system-i386", d, "whpx").endswith("qemu-system-x86_64.exe")
    assert be_mod.windows_qemu_bin("qemu-system-x86_64", d, "tcg").endswith(
        "qemu-system-x86_64.exe"
    )


def test_backend_rewrite_accel():
    base = ["-M", "pc", "-accel", "kvm", "-cpu", "host", "-m", "512M"]
    # tcg: kvm->tcg AND drop the paired -cpu host
    assert be_mod.rewrite_accel(base, "tcg") == ["-M", "pc", "-accel", "tcg", "-m", "512M"]
    # whpx: kvm->whpx with PER-ARCH kernel-irqchip (amd64 needs the Hyper-V in-kernel
    # APIC — QEMU's userspace APIC under irqchip=off zeroes guest CR8/IRQL on every VM
    # exit, bugchecking amd64; i386's PIC HAL needs irqchip=off) and ALSO drop -cpu
    # host — it faults OVMF on FXSAVE in early firmware, so whpx must fall back to
    # qemu's default cpu model.
    assert be_mod.rewrite_accel(base, "whpx", "amd64") == [
        "-M",
        "pc",
        "-accel",
        "whpx,kernel-irqchip=on",
        "-m",
        "512M",
    ]
    assert be_mod.rewrite_accel(base, "whpx", "i386-uefi") == [
        "-M",
        "pc",
        "-accel",
        "whpx,kernel-irqchip=off",
        "-m",
        "512M",
    ]


def test_backend_resolve_accel_per_arch():
    # x86 honors the configured value...
    assert be_mod.resolve_accel("i386", "whpx") == "whpx"
    assert be_mod.resolve_accel("amd64", "whpx") == "whpx"
    assert be_mod.resolve_accel("amd64", "tcg") == "tcg"
    # ...arm64 is FORCED to tcg regardless (whpx is x86-only; the cpu model must survive)
    assert be_mod.resolve_accel("arm64", "whpx") == "tcg"
    assert be_mod.resolve_accel("arm64-uefi", "whpx") == "tcg"


def test_backend_resolve_accel_probe_degrade():
    # probe says DEFINITELY absent → x86 whpx degrades to tcg (fail-soft)
    assert be_mod.resolve_accel("amd64", "whpx", whpx_ok=False) == "tcg"
    # probe present or inconclusive → trust the configured whpx (don't degrade on a flaky probe)
    assert be_mod.resolve_accel("amd64", "whpx", whpx_ok=True) == "whpx"
    assert be_mod.resolve_accel("amd64", "whpx", whpx_ok=None) == "whpx"
    # a probe result never upgrades an explicit tcg config
    assert be_mod.resolve_accel("amd64", "tcg", whpx_ok=True) == "tcg"


def test_backend_stage_media_paths_passthrough():
    # args without a file= token are returned unchanged (staging only touches file= paths)
    args = ["-M", "pc", "-vga", "std", "-display", "none", "-no-reboot"]
    assert be_mod.stage_media_paths(args, Path("/mnt/c/temp/vibeos-qemu")) == args


def test_backend_rewrite_netdev_for_windows():
    # the real amd64-uefi-net netdev: hostfwd 127.0.0.1 -> 0.0.0.0, guestfwd cmd:cat -> chardev
    args = [
        "-device",
        "virtio-net-pci,netdev=net0",
        "-netdev",
        "user,id=net0,hostfwd=tcp:127.0.0.1:54321-:7777,guestfwd=tcp:10.0.2.100:9999-cmd:cat",
    ]
    out, needs_echo = be_mod.rewrite_netdev_for_windows(args, "gf0")
    assert needs_echo is True
    assert "hostfwd=tcp:0.0.0.0:54321-:7777" in out[-1]
    assert "guestfwd=tcp:10.0.2.100:9999-chardev:gf0" in out[-1]
    assert "cmd:cat" not in out[-1]
    # no netdev with fwds -> no-op, no echo server needed (the native-config shape)
    plain = ["-M", "pc", "-accel", "tcg"]
    assert be_mod.rewrite_netdev_for_windows(plain, "gf0") == (plain, False)


def test_free_tcp_port_windows_picker_hook():
    # Native (no hook): a WSL-stack bind picks a real ephemeral port.
    assert vm._windows_port_picker is None
    p = vm._free_tcp_port()
    assert 1024 < p < 65536
    # windows-host: _free_tcp_port delegates to the injected picker (which, in the
    # real backend, picks a Windows-stack-free port so QEMU.exe's 0.0.0.0 hostfwd
    # binds — the WSL stack is a different stack under NAT). Mock it; restore after.
    try:
        vm._windows_port_picker = lambda: 49999
        assert vm._free_tcp_port() == 49999
    finally:
        vm._windows_port_picker = None
    # restored -> back to the WSL pick
    assert vm._windows_port_picker is None
    assert 1024 < vm._free_tcp_port() < 65536


def test_backend_stage_media_paths_arm64_drives():
    # arm64 boot media rides on pflash (QEMU_EFI.fd + efivars) and a virtio -drive
    # whose value carries a trailing `file.locking=off`. Staging must rewrite each
    # real `file=/abs` to its Windows path while leaving `file.locking=off` (no `=/`)
    # and every non-file token untouched. Fake /mnt paths that don't exist take the
    # win_path branch (no 293 MB copy).
    staging = Path("/mnt/c/temp/vibeos-qemu/run-x")
    args = [
        "-drive",
        "if=pflash,format=raw,file=/mnt/c/fake/QEMU_EFI.fd,readonly=on",
        "-drive",
        "if=pflash,format=qcow2,file=/mnt/c/fake/efivars.qcow2",
        "-drive",
        "file=/mnt/c/fake/boot.img,format=raw,if=virtio,readonly=on,file.locking=off",
        "-device",
        "ramfb",
        "-display",
        "none",
    ]
    out = be_mod.stage_media_paths(args, staging)
    assert out[1] == "if=pflash,format=raw,file=C:/fake/QEMU_EFI.fd,readonly=on"
    assert out[3] == "if=pflash,format=qcow2,file=C:/fake/efivars.qcow2"
    # the virtio drive: file= rewritten, file.locking=off preserved verbatim
    assert out[5] == "file=C:/fake/boot.img,format=raw,if=virtio,readonly=on,file.locking=off"
    assert out[7] == "ramfb" and out[9] == "none"


def test_backend_rewrite_accel_would_break_arm64():
    # WHY _build_windows_cmd skips rewrite_accel for arm64: rewrite_accel treats any
    # `-cpu` as the KVM-only `-cpu host` and DROPS it under tcg — but arm64 `virt`
    # needs the real `cortex-a72` model. It also flattens `tcg,thread=multi` -> `tcg`.
    # Both are why arm64 keeps its machine args untouched on the windows-host backend.
    arm64 = ["-M", "virt", "-cpu", "cortex-a72", "-accel", "tcg,thread=multi"]
    out = be_mod.rewrite_accel(arm64, "tcg")
    assert "cortex-a72" not in out  # the model loss we avoid by NOT calling it
    assert out == ["-M", "virt", "-accel", "tcg"]


# --- host broker: cross-worktree CPU-heavy locks ---------------------------
from . import hostlock as hl_mod  # noqa: E402


def test_hostlock_cpu_lease_serializes(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    fd1 = hl_mod.acquire_cpu_lease("build:i386", leases=1, timeout=0.0)
    assert fd1 is not None
    # 1 permit held → a second CPU-heavy op (e.g. a TCG boot) is serialized
    assert hl_mod.acquire_cpu_lease("boot:arm64", leases=1, timeout=0.3) is None
    hl_mod.release(fd1)
    fd3 = hl_mod.acquire_cpu_lease("build:amd64", leases=1, timeout=0.5)
    assert fd3 is not None  # freed on release
    hl_mod.release(fd3)


def test_hostlock_cpu_lease_pool_of_two(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    a = hl_mod.acquire_cpu_lease("x", leases=2, timeout=0.0)
    b = hl_mod.acquire_cpu_lease("y", leases=2, timeout=0.0)
    assert a is not None and b is not None  # both permits available
    assert hl_mod.acquire_cpu_lease("z", leases=2, timeout=0.3) is None  # pool exhausted
    hl_mod.release(a)
    hl_mod.release(b)


# --- FIFO admission (fairness) ---------------------------------------------
# The bug these cover: permit acquisition is order-blind, so a late arrival can take the
# permits a long-waiting lane is queued for. A build starved 23 minutes this way on
# 2026-07-24 while the permits changed hands between two OTHER worktrees.


def _fake_ticket(tmp_path, pool: str, *, ns: int, pid: int, need: int):
    """Plant a ticket held by a LIVE flock (a stand-in for another worktree's waiter).
    Returns the held fd — closing it makes the ticket an orphan, as a death would."""
    import fcntl as _f
    import os as _o

    qd = tmp_path / f"queue-{pool}"
    qd.mkdir(parents=True, exist_ok=True)
    p = qd / f"{ns:020d}-{pid}-{need}"
    fd = _o.open(str(p), _o.O_RDWR | _o.O_CREAT, 0o644)
    _f.flock(fd, _f.LOCK_EX | _f.LOCK_NB)
    return fd


def test_hostlock_fifo_waiter_ahead_blocks_barging(tmp_path, monkeypatch):
    """A free permit is NOT taken by a latecomer when an earlier arrival is owed it."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    # Someone arrived first (ns=1) and needs both permits of a 2-permit pool.
    ahead = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=999999, need=2)
    try:
        # Pool is entirely free, but we arrive second → we must not barge.
        assert hl_mod.acquire_cpu_share("late", n=1, leases=2, timeout=0.3) is None
    finally:
        os.close(ahead)  # the earlier waiter goes away
    # Its ticket is now an orphan; the next acquirer reaps it and proceeds.
    fds = hl_mod.acquire_cpu_share("late", n=1, leases=2, timeout=1.0)
    assert fds is not None
    hl_mod.release_all(fds)


def test_hostlock_fifo_head_is_never_blocked_by_itself(tmp_path, monkeypatch):
    """Head-of-queue always attempts — the queue must not deadlock the only waiter."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    fds = hl_mod.acquire_cpu_share("solo", n=2, leases=2, timeout=1.0)
    assert fds is not None
    hl_mod.release_all(fds)
    # Ticket cleaned up on the way out — no residue to starve the next caller.
    assert hl_mod.queue_status("cpuheavy") == []


def test_hostlock_fifo_backfill_when_head_not_delayed(tmp_path, monkeypatch):
    """A latecomer MAY use permits nobody ahead of it is owed (throughput preserved)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    ahead = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=999999, need=1)
    try:
        # 4 permits, one earlier waiter owed 1 → 3 spare, so our 2 fit without delaying it.
        fds = hl_mod.acquire_cpu_share("late", n=2, leases=4, timeout=1.0)
        assert fds is not None
        hl_mod.release_all(fds)
    finally:
        os.close(ahead)


def test_hostlock_fifo_orphan_ticket_is_reaped(tmp_path, monkeypatch):
    """A waiter that DIES must not wedge the queue forever (kernel drops its flock)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    dead = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=999999, need=4)
    os.close(dead)  # simulate the owner dying mid-wait
    assert hl_mod.queue_status("cpuheavy") == []  # not counted as a live waiter
    fds = hl_mod.acquire_cpu_share("survivor", n=2, leases=4, timeout=1.0)
    assert fds is not None
    hl_mod.release_all(fds)
    assert not list((tmp_path / "queue-cpuheavy").iterdir())  # orphan actually deleted


def test_hostlock_fifo_can_be_disabled(tmp_path, monkeypatch):
    """VIBEOS_HOSTLOCK_FAIR=0 restores the old order-blind behavior (escape hatch)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOSTLOCK_FAIR", "0")
    ahead = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=999999, need=2)
    try:
        fds = hl_mod.acquire_cpu_share("barger", n=1, leases=2, timeout=0.5)
        assert fds is not None  # barges past the earlier waiter, as before
        hl_mod.release_all(fds)
    finally:
        os.close(ahead)


def test_hostlock_queue_status_reports_waiters(tmp_path, monkeypatch):
    """Starvation must be VISIBLE — it used to read as 'my build is slow' (§9.84)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    a = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=111, need=2)
    b = _fake_ticket(tmp_path, "cpuheavy", ns=2, pid=222, need=1)
    try:
        q = hl_mod.queue_status("cpuheavy")
        assert [w["pid"] for w in q] == ["111", "222"]  # oldest arrival first
        assert [w["need"] for w in q] == [2, 1]
        rendered = hl_mod.render_status(hl_mod.status())
        assert "queued" in rendered and "NEXT" in rendered and "pid=111" in rendered
    finally:
        os.close(a)
        os.close(b)


def test_hostlock_build_dir_same_blocks_different_ok(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    d1 = tmp_path / "build-amd64"
    d2 = tmp_path / "build-arm64"
    d1.mkdir()
    d2.mkdir()
    f1 = hl_mod.acquire_build_dir(d1, timeout=0.0)
    assert f1 is not None
    assert hl_mod.acquire_build_dir(d1, timeout=0.3) is None  # same dir → corruption guard
    f2 = hl_mod.acquire_build_dir(d2, timeout=0.0)
    assert f2 is not None  # a different dir is independent
    hl_mod.release(f1)
    hl_mod.release(f2)


def test_hostlock_disabled_is_noop(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "0")
    assert hl_mod.enabled() is False
    with hl_mod.cpu_lease("build:i386", leases=1) as fd:
        assert fd is None  # context manager is a no-op when disabled
    with hl_mod.cpu_exclusive("build:i386") as fds:
        assert fds is None
    with hl_mod.build_dir_lock(tmp_path / "build-i386") as fd:
        assert fd is None
    with hl_mod.build_lock(tmp_path / "build-i386", "i386"):
        pass  # no-op grant, must not raise


def test_hostlock_exclusive_build_excludes_boots(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    fds = hl_mod.acquire_cpu_exclusive("build:i386", leases=4, timeout=0.0)
    assert fds is not None and len(fds) == 4  # a build grabs the WHOLE pool
    assert hl_mod.acquire_cpu_lease("boot:arm64", leases=4, timeout=0.3) is None  # boots excluded
    hl_mod.release_all(fds)
    b = hl_mod.acquire_cpu_lease("boot:arm64", leases=4, timeout=0.3)
    assert b is not None  # freed → a boot runs
    hl_mod.release(b)


def test_hostlock_boots_parallelize_but_block_a_build(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    # pool of 2: two TCG boots run in parallel...
    b1 = hl_mod.acquire_cpu_lease("boot:arm64", leases=2, timeout=0.0)
    b2 = hl_mod.acquire_cpu_lease("boot:arm64", leases=2, timeout=0.0)
    assert b1 is not None and b2 is not None
    assert hl_mod.acquire_cpu_lease("boot:arm64", leases=2, timeout=0.2) is None  # pool full
    # ...but a build (needs ALL permits) can't start while any boot holds one
    assert hl_mod.acquire_cpu_exclusive("build:i386", leases=2, timeout=0.3) is None
    hl_mod.release(b1)
    hl_mod.release(b2)
    fds = hl_mod.acquire_cpu_exclusive("build:i386", leases=2, timeout=0.3)
    assert fds is not None and len(fds) == 2  # boots gone → build goes exclusive
    hl_mod.release_all(fds)


def test_hostlock_build_lock_cm_excludes_boot(tmp_path, monkeypatch):
    # With VIBEOS_MAX_CONCURRENT_BUILDS=1 the original writer-takes-all grant is
    # restored: one build → exclusive host → boots excluded.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    monkeypatch.setenv("VIBEOS_MAX_CONCURRENT_BUILDS", "1")
    bd = tmp_path / "build-i386"
    bd.mkdir()
    k = hl_mod.DEFAULT_CPU_LEASES
    with hl_mod.build_lock(bd, "i386"):
        # the build holds the exclusive host → a boot can't get a permit
        assert hl_mod.acquire_cpu_lease("boot:arm64", leases=k, timeout=0.2) is None
    b = hl_mod.acquire_cpu_lease("boot:arm64", leases=k, timeout=0.2)
    assert b is not None  # released when the cm exited
    hl_mod.release(b)


def test_hostlock_two_builds_coexist_third_queues(tmp_path, monkeypatch):
    # Default policy (max 2 builds): each build takes ceil(4/2)=2 permits, so two
    # builds run concurrently and a third can't get a share.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    monkeypatch.delenv("VIBEOS_MAX_CONCURRENT_BUILDS", raising=False)
    assert hl_mod.build_permits(4) == 2
    a = hl_mod.acquire_cpu_share("build:i386", n=2, leases=4, timeout=0.0)
    b = hl_mod.acquire_cpu_share("build:amd64", n=2, leases=4, timeout=0.0)
    assert a is not None and b is not None  # two builds coexist
    # pool exhausted: a third build AND a native-TCG boot are both excluded
    assert hl_mod.acquire_cpu_share("build:arm64", n=2, leases=4, timeout=0.3) is None
    assert hl_mod.acquire_cpu_lease("boot:arm64", leases=4, timeout=0.2) is None
    hl_mod.release_all(a)
    hl_mod.release_all(b)


def test_hostlock_one_build_leaves_boot_room(tmp_path, monkeypatch):
    # One build (2 of 4 permits) leaves 2 permits: native-TCG boots run alongside.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    a = hl_mod.acquire_cpu_share("build:i386", n=2, leases=4, timeout=0.0)
    assert a is not None
    b1 = hl_mod.acquire_cpu_lease("boot:arm64", leases=4, timeout=0.2)
    b2 = hl_mod.acquire_cpu_lease("boot:arm64", leases=4, timeout=0.2)
    assert b1 is not None and b2 is not None
    assert hl_mod.acquire_cpu_lease("boot:arm64", leases=4, timeout=0.2) is None
    hl_mod.release(b1)
    hl_mod.release(b2)
    hl_mod.release_all(a)


def test_hostlock_share_shortfall_releases_all(tmp_path, monkeypatch):
    # No hold-and-wait: a share acquirer that can't get its full n holds NOTHING
    # while waiting (deadlock-freedom invariant).
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    held = [
        hl_mod.acquire_cpu_lease("boot:a", leases=4, timeout=0.0),
        hl_mod.acquire_cpu_lease("boot:b", leases=4, timeout=0.0),
        hl_mod.acquire_cpu_lease("boot:c", leases=4, timeout=0.0),
    ]
    assert all(f is not None for f in held)
    # only 1 free permit → a 2-share times out...
    assert hl_mod.acquire_cpu_share("build:i386", n=2, leases=4, timeout=0.3) is None
    # ...and held nothing while failing: the free permit is still immediately grabbable
    b = hl_mod.acquire_cpu_lease("boot:d", leases=4, timeout=0.0)
    assert b is not None
    hl_mod.release(b)
    for f in held:
        hl_mod.release(f)


def test_hostlock_cpu_lease_cm_acquires_and_releases(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    with hl_mod.cpu_lease("build:i386", leases=1) as fd:
        assert fd is not None
        assert hl_mod.acquire_cpu_lease("other", leases=1, timeout=0.2) is None  # held
    fd2 = hl_mod.acquire_cpu_lease("after", leases=1, timeout=0.2)
    assert fd2 is not None  # released when the cm exited
    hl_mod.release(fd2)


def test_hostlock_announce_pair_on_queued_acquire(tmp_path, monkeypatch, capsys):
    # The §9.84 starvation classifier depends on this exact invariant: a queued
    # acquire that eventually succeeds prints BOTH announce lines; one that times
    # out prints only WAIT; an uncontended acquire prints neither.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    holder = hl_mod.acquire_cpu_lease("boot:a", leases=1, timeout=0.0)
    assert holder is not None
    # timed-out queue → WAIT only
    assert hl_mod.acquire_cpu_lease("boot:b", leases=1, timeout=0.8) is None
    out = capsys.readouterr().out
    assert hl_mod.WAIT_ANNOUNCE in out and hl_mod.ACQUIRED_ANNOUNCE not in out
    assert hl_mod.looks_starved(out)
    # queued-then-granted → WAIT + ACQUIRED (holder released mid-wait by a thread)
    threading.Timer(0.8, hl_mod.release, args=(holder,)).start()
    fd = hl_mod.acquire_cpu_lease("boot:c", leases=1, timeout=10.0)
    assert fd is not None
    out = capsys.readouterr().out
    assert hl_mod.WAIT_ANNOUNCE in out and hl_mod.ACQUIRED_ANNOUNCE in out
    assert not hl_mod.looks_starved(out)
    hl_mod.release(fd)
    # uncontended → silent
    fd2 = hl_mod.acquire_cpu_lease("boot:d", leases=1, timeout=0.5)
    assert fd2 is not None
    out = capsys.readouterr().out
    assert hl_mod.WAIT_ANNOUNCE not in out and hl_mod.ACQUIRED_ANNOUNCE not in out
    assert not hl_mod.looks_starved(out)
    hl_mod.release(fd2)


def test_hostlock_looks_starved_classifier():
    assert hl_mod.looks_starved(f"setup\n{hl_mod.WAIT_ANNOUNCE} (boot:amd64-uefi-net)...\n")
    assert not hl_mod.looks_starved(
        f"{hl_mod.WAIT_ANNOUNCE} (boot:amd64-uefi-net)...\n"
        f"{hl_mod.ACQUIRED_ANNOUNCE} (boot:amd64-uefi-net) after 42s queued\n"
        "2 failed, 13 passed"
    )
    assert not hl_mod.looks_starved("15 errors in 600.39s")  # no queue announced


def test_gate_host_starved_is_retryable():
    # gate retries HOST_STARVED: its retry phase runs after the gate's own lanes
    # released their leases, so self-inflicted contention clears; an external
    # holder keeps the retry starved and the final red says HOST_STARVED.
    from . import gate as gate_mod

    assert "HOST_STARVED" in gate_mod.RETRYABLE
    assert gate_mod._retryable("HOST_STARVED", {}) is True


def test_hostlock_status_reports_live_holder(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    fd = hl_mod.acquire_cpu_lease("build:i386", leases=1, timeout=0.0)
    st = hl_mod.status()
    assert st["cpu_leases_held"] == 1
    held = [e for e in st["cpuheavy"] if e["held"]]
    assert len(held) == 1
    assert held[0]["holder"]["kind"] == "build:i386"
    assert held[0]["holder_alive"] is True  # this very process holds it
    hl_mod.release(fd)
    assert hl_mod.status()["cpu_leases_held"] == 0


def test_hostlock_render_status_no_dir(tmp_path, monkeypatch):
    # status() before any lock dir exists must still carry the lease keys + render
    # without KeyError (regression: render_status hit a missing cpu_leases_held).
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "does-not-exist")
    st = hl_mod.status()
    assert st["cpu_leases_held"] == 0 and st["cpu_leases_total"] >= 1
    out = hl_mod.render_status(st)
    assert "host broker" in out and "FREE" in out


# --- canon: canonical build path detection, configure carry, toolchain heal ----

from . import canon as canon_mod  # noqa: E402


def test_canon_cmake_home_and_is_canonical(tmp_path, monkeypatch):
    monkeypatch.delenv("VIBEOS_CANON", raising=False)
    bd = tmp_path / "build-i386"
    bd.mkdir()
    assert canon_mod.cmake_home(bd) is None
    assert canon_mod.is_canonical(bd) is False
    (bd / "CMakeCache.txt").write_text(
        "ARCH:STRING=i386\nCMAKE_HOME_DIRECTORY:INTERNAL=/tmp/vibeos\n"
    )
    assert canon_mod.cmake_home(bd) == "/tmp/vibeos"
    assert canon_mod.is_canonical(bd) is True
    monkeypatch.setenv("VIBEOS_CANON", "/somewhere/else")
    assert canon_mod.is_canonical(bd) is False


def test_canon_carried_options(tmp_path):
    bd = tmp_path / "build-i386"
    bd.mkdir()
    assert canon_mod.carried_options(bd) == []
    (bd / "CMakeCache.txt").write_text(
        "CMAKE_BUILD_TYPE:STRING=Debug\nREACTOS_TEST_KERNEL:BOOL=TRUE\n"
        "SARCH:STRING=\nOARCH:STRING=pentium\nUNRELATED:BOOL=ON\n"
    )
    opts = canon_mod.carried_options(bd)
    assert "-DCMAKE_BUILD_TYPE=Debug" in opts
    assert "-DREACTOS_TEST_KERNEL=TRUE" in opts
    assert "-DOARCH=pentium" in opts
    assert not any(o.startswith("-DSARCH") for o in opts)  # empty value dropped
    assert not any("UNRELATED" in o for o in opts)


def test_canon_configure_cmd_shape(monkeypatch):
    monkeypatch.delenv("VIBEOS_CANON", raising=False)
    cmd = canon_mod.configure_cmd("i386", ["-DREACTOS_TEST_KERNEL=TRUE"])
    assert cmd[0] == "cmake" and cmd[-1] == "/tmp/vibeos"
    assert "-DARCH:STRING=i386" in cmd
    assert "-DENABLE_CCACHE=1" in cmd
    # carried opts come AFTER the Debug default so they win (cmake last -D wins)
    assert cmd.index("-DREACTOS_TEST_KERNEL=TRUE") > cmd.index("-DCMAKE_BUILD_TYPE=Debug")


def test_canon_wrap_argv(tmp_path):
    argv = canon_mod.wrap(tmp_path, ["ninja", "livecd"])
    assert argv[0] == "bash"
    assert argv[1].endswith("tools/canon-run.sh")
    assert argv[2:] == ["ninja", "livecd"]


def test_canon_ensure_toolchain_heals_relative_symlink(tmp_path, monkeypatch):
    parent = tmp_path
    repo = parent / "vibeos"
    (repo / ".build-tools").mkdir(parents=True)
    tc = parent / "llvm-mingw-20250114"
    tc.mkdir()
    monkeypatch.setenv("VIBEOS_ROOT", str(parent))
    link = repo / ".build-tools" / "llvm-mingw"
    # historical RELATIVE symlink — dangles under the canonical mount
    link.symlink_to("../../llvm-mingw-20250114")
    assert canon_mod.ensure_toolchain(repo) is True
    assert str(link.readlink()).startswith("/")  # now absolute
    assert link.resolve() == tc.resolve()


def test_canon_ensure_toolchain_fresh_worktree(tmp_path, monkeypatch):
    parent = tmp_path
    repo = parent / "vibeos-wt-x"
    repo.mkdir()
    tc = parent / "llvm-mingw-20250114"
    tc.mkdir()
    monkeypatch.setenv("VIBEOS_ROOT", str(parent))
    assert canon_mod.ensure_toolchain(repo) is True
    assert (repo / ".build-tools" / "llvm-mingw").resolve() == tc.resolve()


def test_canon_ensure_toolchain_missing(tmp_path, monkeypatch):
    repo = tmp_path / "vibeos"
    repo.mkdir()
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path))
    assert canon_mod.ensure_toolchain(repo) is False


# --- gate: parallel driver retry/verdict logic (no QEMU) -------------------

from argparse import Namespace  # noqa: E402

from . import gate as gate_mod  # noqa: E402


def _gate_args(**kw):
    # no_lint=True by default so retry/verdict tests don't invoke the real
    # per-arch strict-lint subprocess; the lint-gate tests pass no_lint=False.
    base = dict(
        arches="i386,amd64,arm64",
        retries=2,
        no_build=True,
        clean=False,
        no_net=False,
        no_lint=True,
    )
    base.update(kw)
    return Namespace(**base)


def _fake_smoke(mapping):
    """Return a stand-in for gate._smoke_once. `mapping[arch]` is a list of
    per-attempt verdicts, e.g. ['KNOWN_FLAKE', 'OK']. Attempt index derives from
    the root dir suffix (a1 = first pass, r1/r2 = retries). A HANG_TIMEOUT can
    carry its smoke characterization as a suffix: 'HANG_TIMEOUT:slow' (serial
    still flowing — contention-shaped), 'HANG_TIMEOUT:wedged' (silent), or
    'HANG_TIMEOUT:census' (kernel census/lockdep self-confirmed the wedge)."""
    calls: dict[str, int] = {}

    def fake(repo, arch, root, extra):
        seq = mapping[arch]
        name = root.name  # smoke-<arch>-a1 / smoke-<arch>-r1 ...
        if name.endswith("-a1"):
            idx = 0
        else:
            idx = 1 + int(name.rsplit("-r", 1)[1]) - 1
        idx = min(idx, len(seq) - 1)
        calls[f"{arch}:{name}"] = calls.get(f"{arch}:{name}", 0) + 1
        verdict = seq[idx]
        fields: dict = {}
        if ":" in verdict:
            verdict, kind = verdict.split(":", 1)
            if kind == "census":
                fields.update(hang_kind="wedged", wedge_confirmed=True, census_stuck=3)
            else:
                fields["hang_kind"] = kind
        ok = verdict in ("OK", "WARN")
        classification = "" if ok else verdict
        return {
            "arch": arch,
            "verdict": verdict if ok else "FAIL",
            "classification": classification,
            "ok": ok,
            "retryable": gate_mod._retryable(classification, fields),
            "hang_kind": fields.get("hang_kind"),
            "wedge_confirmed": bool(fields.get("wedge_confirmed")),
            "census_stuck": fields.get("census_stuck", 0),
            "seconds": 0.01,
            "run_dir": str(root),
            "flake_rate": None,
            "screenshot": None,
            "salient": "" if ok else f"{verdict} detail",
        }

    fake.calls = calls
    return fake


def test_gate_all_green_no_retries(tmp_path, monkeypatch):
    monkeypatch.setattr(gate_mod, "ARTIFACT_ROOT_UNUSED", None, raising=False)
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(), tmp_path)
    assert rc == 0
    # no retry dirs requested
    assert not any("-r" in k for k in fake.calls), fake.calls


def test_gate_flake_recovers_on_retry(tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["KNOWN_FLAKE", "OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(), tmp_path)
    assert rc == 0, "a documented flake that goes green on retry must pass the gate"
    # amd64 was retried
    assert any(k.startswith("amd64:") and "-r" in k for k in fake.calls), fake.calls


def test_gate_bugcheck_not_retried(tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["BUGCHECK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(), tmp_path)
    assert rc == 1, "a real regression must fail the gate"
    # a non-retryable classification must NOT trigger retries (would hide the bug)
    assert not any(k.startswith("amd64:") and "-r" in k for k in fake.calls), fake.calls


def test_gate_flake_stays_red_if_all_retries_fail(tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["KNOWN_FLAKE"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(retries=2), tmp_path)
    assert rc == 1, "a flake that never recovers must still fail (hides nothing)"
    # it DID retry (2 parallel attempts)
    retry_calls = [k for k in fake.calls if k.startswith("amd64:") and "-r" in k]
    assert len(retry_calls) == 2, fake.calls


def test_gate_retries_zero_disables_retry(tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["KNOWN_FLAKE", "OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(arches="i386", retries=0), tmp_path)
    assert rc == 1
    assert not any("-r" in k for k in fake.calls), fake.calls


def test_gate_slow_hang_timeout_is_retried(tmp_path, monkeypatch):
    # serial still flowing at deadline = contention-shaped → retryable
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["HANG_TIMEOUT:slow", "OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(), tmp_path)
    assert rc == 0
    assert any(k.startswith("amd64:") and "-r" in k for k in fake.calls), fake.calls


def test_gate_wedged_hang_timeout_not_retried(tmp_path, monkeypatch):
    # serial went silent — a wedge is a regression signature, never a retry.
    # (The old blanket HANG_TIMEOUT retry let a ~50% service wedge pass a
    # 3-attempt gate ~87% of the time.)
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["HANG_TIMEOUT:wedged", "OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(), tmp_path)
    assert rc == 1, "a wedged hang must fail the gate, not be retried away"
    assert not any(k.startswith("amd64:") and "-r" in k for k in fake.calls), fake.calls


def test_gate_census_confirmed_wedge_not_retried(tmp_path, monkeypatch):
    # the kernel itself reported stuck threads — definitely not contention
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"amd64": ["HANG_TIMEOUT:census", "OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(arches="amd64"), tmp_path)
    assert rc == 1
    assert not any("-r" in k for k in fake.calls), fake.calls


def test_gate_launch_fail_is_retried(tmp_path, monkeypatch):
    # a QEMU/interop launch hiccup is an environment failure, not a regression
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"amd64": ["LAUNCH_FAIL", "OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(arches="amd64"), tmp_path)
    assert rc == 0, "an env launch failure that greens on retry must pass"
    assert any("-r" in k for k in fake.calls), fake.calls


def test_retryable_policy_unit():
    assert gate_mod._retryable("KNOWN_FLAKE", {})
    assert gate_mod._retryable("FLAKY", {})
    assert gate_mod._retryable("LAUNCH_FAIL", {})
    assert gate_mod._retryable("HANG_TIMEOUT", {"hang_kind": "slow"})
    assert not gate_mod._retryable("HANG_TIMEOUT", {})  # uncharacterized blocks
    assert not gate_mod._retryable("HANG_TIMEOUT", {"hang_kind": "wedged"})
    assert not gate_mod._retryable("HANG_TIMEOUT", {"hang_kind": "slow", "wedge_confirmed": True})
    assert not gate_mod._retryable("BUGCHECK", {})
    assert not gate_mod._retryable("RUST_PANIC", {})
    assert not gate_mod._retryable("SHELL_NOT_READY", {})
    assert not gate_mod._retryable("STALE_MEDIA", {})


# --- gate: BLOCKING lint gate (T2 item 1 — enforce, don't just detect) ------


def _spy_smoke_ok(seen):
    def fake(repo, arch, root, extra):
        seen.append(arch)
        return {
            "arch": arch,
            "verdict": "OK",
            "classification": "",
            "ok": True,
            "retryable": False,
            "hang_kind": None,
            "wedge_confirmed": False,
            "census_stuck": 0,
            "seconds": 0.01,
            "run_dir": str(root),
            "flake_rate": None,
            "screenshot": None,
            "salient": "",
        }

    return fake


def _lint_once_stub(ok: bool):
    """Stand-in for gate._lint_once (which shells `check --strict --arch X`)."""

    def stub(repo, arch, root):
        return {
            "arch": arch,
            "ok": ok,
            "seconds": 0.01,
            "salient": "" if ok else f"{arch}: NEW lint findings",
            "exit": 0 if ok else 1,
        }

    return stub


def _tooling_once_stub(ok: bool):
    """Stand-in for gate._tooling_once (which shells `vibeos-cli check-tooling`)."""

    def stub(repo, root):
        return {
            "ok": ok,
            "seconds": 0.01,
            "salient": "" if ok else "python tooling findings",
            "exit": 0 if ok else 1,
        }

    return stub


def _difftest_once_stub(ok: bool):
    """Stand-in for gate._difftest_once (which shells `vibeos-cli difftest`)."""

    def stub(repo, root):
        return {
            "ok": ok,
            "seconds": 0.01,
            "salient": "" if ok else "RtlFoo fields a=1",
            "exit": 0 if ok else 1,
        }

    return stub


def test_gate_lint_failure_blocks_stamp_but_smokes_run(tmp_path, monkeypatch):
    """A NEW lint finding fails the gate and blocks the push stamp; smokes still
    run so the matrix stays informative (the landed T3 policy)."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    seen: list = []
    monkeypatch.setattr(gate_mod, "_smoke_once", _spy_smoke_ok(seen))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_difftest_once", _difftest_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_lint_once", _lint_once_stub(ok=False))
    stamped: list = []
    monkeypatch.setattr(gate_mod, "_write_gate_stamp", lambda *a, **k: stamped.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=False), tmp_path)
    assert rc == 1, "a lint regression must fail the gate"
    assert not stamped, "no push stamp on a lint failure"
    assert seen, "smokes still run under the T3 policy (informative matrix)"


def test_gate_lint_pass_proceeds_and_stamps(tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    monkeypatch.setattr(gate_mod, "_lint_once", _lint_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_difftest_once", _difftest_once_stub(ok=True))
    stamped: list = []
    monkeypatch.setattr(gate_mod, "_write_gate_stamp", lambda *a, **k: stamped.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=False), tmp_path)
    assert rc == 0
    assert fake.calls, "a passing lint gate must proceed to smokes"
    assert stamped, "a full green gate (lints + smokes) mints the push stamp"


def test_gate_tooling_failure_blocks_stamp(tmp_path, monkeypatch):
    """check-tooling (ruff/pyright/pytest over OUR python) gates the push stamp.

    Regression test for the hole that let master ship `check-tooling` red on two
    consecutive landings: the gate ran `check --strict` per arch but never ran
    check-tooling, so nothing between the author and the stamp saw Python
    regressions.
    """
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    monkeypatch.setattr(gate_mod, "_lint_once", _lint_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(ok=False))
    monkeypatch.setattr(gate_mod, "_difftest_once", _difftest_once_stub(ok=True))
    stamped: list = []
    monkeypatch.setattr(gate_mod, "_write_gate_stamp", lambda *a, **k: stamped.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=False), tmp_path)
    assert rc == 1, "a python tooling regression must fail the gate"
    assert not stamped, "no push stamp when check-tooling is red"
    assert fake.calls, "smokes still run so the matrix stays informative"


def test_gate_difftest_failure_blocks_stamp(tmp_path, monkeypatch):
    """A rust-vs-C divergence fails the gate and blocks the push stamp.

    Same hole check-tooling had, one layer down: the difftests were reachable
    only by typing `cargo test` in a crate dir, so `rtl-difftest` sat 2/87 red
    on master with nothing between the author and the stamp ever running it.
    One of those two was a real divergence in eleven `Rtl*String` converters.
    """
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    monkeypatch.setattr(gate_mod, "_lint_once", _lint_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_difftest_once", _difftest_once_stub(ok=False))
    stamped: list = []
    monkeypatch.setattr(gate_mod, "_write_gate_stamp", lambda *a, **k: stamped.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=False), tmp_path)
    assert rc == 1, "a rust-vs-C divergence must fail the gate"
    assert not stamped, "no push stamp when a difftest is red"
    assert fake.calls, "smokes still run so the matrix stays informative"


def test_gate_no_lint_skips_lint_phase(tmp_path, monkeypatch):
    """--no-lint bypasses the strict lint gate entirely (escape hatch)."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    called: list = []
    tooled: list = []
    diffed: list = []
    monkeypatch.setattr(gate_mod, "_lint_once", lambda *a, **k: called.append(1))
    monkeypatch.setattr(gate_mod, "_tooling_once", lambda *a, **k: tooled.append(1))
    monkeypatch.setattr(gate_mod, "_difftest_once", lambda *a, **k: diffed.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=True), tmp_path)
    assert rc == 0
    assert not called, "--no-lint must not invoke the lint subprocess"
    assert not tooled, "--no-lint must not invoke check-tooling either"
    assert not diffed, "--no-lint must not invoke the difftests either"


# --- check --update-baseline: per-tool scoping (T2 item 1b) -----------------

from . import debugcmd as debugcmd_mod  # noqa: E402


def test_check_update_baseline_per_tool_scopes(tmp_path, monkeypatch):
    """--update-baseline lint_kernel re-blesses ONLY that tool (not a blanket
    re-bless that would silently accept unrelated drift)."""
    cap: dict = {}

    def _cap(repo, only=None):
        cap["only"] = only
        return {"lint_kernel": {"count": 3, "added": 1, "removed": 0}}

    monkeypatch.setattr(lints, "lint_tool_names", lambda repo: {"lint_kernel", "verify_abi"})
    monkeypatch.setattr(lints, "write_baselines", _cap)
    args = Namespace(update_baseline="lint_kernel", arch=None, strict=False)
    assert debugcmd_mod.check(args, tmp_path) == 0
    assert cap["only"] == {"lint_kernel"}


def test_check_update_baseline_blanket_is_none(tmp_path, monkeypatch):
    cap: dict = {}

    def _cap(repo, only=None):
        cap["only"] = only
        return {}

    monkeypatch.setattr(lints, "write_baselines", _cap)
    args = Namespace(update_baseline="__ALL__", arch=None, strict=False)
    assert debugcmd_mod.check(args, tmp_path) == 0
    assert cap["only"] is None  # blanket rewrites every baseline


def test_check_update_baseline_rejects_unknown_tool(tmp_path, monkeypatch):
    called: list = []
    monkeypatch.setattr(lints, "lint_tool_names", lambda repo: {"lint_kernel"})
    monkeypatch.setattr(lints, "write_baselines", lambda *a, **k: called.append(1) or {})
    args = Namespace(update_baseline="bogus_tool", arch=None, strict=False)
    assert debugcmd_mod.check(args, tmp_path) == 2
    assert not called, "an unknown tool name must not rewrite any baseline"


def test_write_baselines_only_filter(tmp_path, monkeypatch):
    """write_baselines(only={...}) rewrites just those tools' files."""
    bdir = tmp_path / lints.BASELINE_DIR
    bdir.mkdir(parents=True)
    (bdir / "toolA.txt").write_text("old-A\n")
    (bdir / "toolB.txt").write_text("old-B\n")
    monkeypatch.setattr(
        lints,
        "source_lint_specs",
        lambda repo: [("toolA", ["true"], "stdout"), ("toolB", ["true"], "stdout")],
    )

    def fake_run(cmd, cwd, capture_output, text, timeout):
        import subprocess as _sp

        return _sp.CompletedProcess(cmd, 0, "new-line\n", "")

    monkeypatch.setattr(lints.subprocess, "run", fake_run)
    out = lints.write_baselines(tmp_path, only={"toolA"})
    assert set(out) == {"toolA"}
    assert (bdir / "toolA.txt").read_text().strip() == "new-line"
    assert (bdir / "toolB.txt").read_text().strip() == "old-B"  # untouched
    assert out["toolA"] == {"count": 1, "added": 1, "removed": 1}


# --- smoke._watch: early-hang watchdog + net opt-in (no QEMU) ---------------

from . import smoke as smoke_mod  # noqa: E402


class _FakeSerial:
    """Emits the given chunks once, then stays silent (returns "")."""

    def __init__(self, chunks):
        self._chunks = list(chunks)
        self._i = 0

    def read_available(self, timeout=0.5):
        if self._i < len(self._chunks):
            c = self._chunks[self._i]
            self._i += 1
            return c
        time.sleep(0.02)  # mimic a quiet poll
        return ""


class _FakeVM:
    def __init__(self, chunks, hang_silence):
        self.serial = _FakeSerial(chunks)
        self.config = type("Cfg", (), {"hang_silence_seconds": hang_silence})()


def test_watch_early_hang_aborts_on_silence():
    # boot emits once, then goes silent → watchdog trips well before `timeout`
    vm = _FakeVM(["[SMSS] boot\n"], hang_silence=0.1)
    t0 = time.monotonic()
    outcome, _buf, diag = smoke_mod._watch(vm, until="NEVER", fail_on=None, timeout=30)  # type: ignore[arg-type]
    assert outcome == "timeout"
    assert diag["early_hang"] is True
    assert diag["silent_s"] >= 0.1
    assert time.monotonic() - t0 < 5, "must fail FAST, not wait the full timeout"


def test_watch_reaches_marker_before_hang():
    vm = _FakeVM(["boot\n", "[EXPLORER] _tWinMain entered\n"], hang_silence=0.1)
    outcome, _buf, _diag = smoke_mod._watch(
        vm,  # type: ignore[arg-type]
        until="_tWinMain entered",
        fail_on=None,
        timeout=30,
    )
    assert outcome == "reached"


def test_watch_no_early_hang_when_disabled():
    # hang_silence=0 disables the watchdog → runs to the (short) deadline
    vm = _FakeVM(["x\n"], hang_silence=0.0)
    outcome, _buf, diag = smoke_mod._watch(vm, until="NEVER", fail_on=None, timeout=0.3)  # type: ignore[arg-type]
    assert outcome == "timeout"
    assert diag["early_hang"] is False


def test_settle_watch_keeps_serial_and_sees_milestone():
    # settle serial is KEPT (for the post-marker health scan) and the milestone
    # is detected — previously the drain discarded everything it read.
    vm = _FakeVM(["taskbar building...\n", "[WIN32K] shell ready\n"], hang_silence=0.0)
    text, seen = smoke_mod._settle_watch(vm, 0.15, "shell ready", grace=0.1)  # type: ignore[arg-type]
    assert "shell ready" in text
    assert seen is True


def test_settle_watch_milestone_missing_after_grace():
    vm = _FakeVM(["taskbar building...\n"], hang_silence=0.0)
    t0 = time.monotonic()
    text, seen = smoke_mod._settle_watch(vm, 0.1, "shell ready", grace=0.2)  # type: ignore[arg-type]
    assert seen is False
    assert "taskbar" in text
    assert time.monotonic() - t0 < 5


def test_settle_watch_no_milestone_required():
    vm = _FakeVM(["whatever\n"], hang_silence=0.0)
    text, seen = smoke_mod._settle_watch(vm, 0.1, None)  # type: ignore[arg-type]
    assert seen is True and "whatever" in text


# --- hostlock: build(cpuheavy) vs boot(bootcpu) pools are DECOUPLED --------


def test_hostlock_build_exclusive_does_not_block_windows_host_boot(tmp_path, monkeypatch):
    # A cold build grabs the WHOLE cpuheavy pool (exclusive host). A windows-host boot
    # draws from the SEPARATE bootcpu pool, so it must NOT be blocked by that build.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    build_fds = hl_mod.acquire_cpu_exclusive("build:i386", leases=4, timeout=0.0)
    assert build_fds is not None and len(build_fds) == 4  # build owns all cpuheavy
    boot_fd = hl_mod.acquire_boot_lease("boot:amd64", leases=4, timeout=0.3)
    assert boot_fd is not None, "windows-host boot must run alongside an exclusive build"
    hl_mod.release(boot_fd)
    hl_mod.release_all(build_fds)


def test_hostlock_boot_pool_throttles_boot_vs_boot(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    a = hl_mod.acquire_boot_lease("boot:i386", leases=2, timeout=0.0)
    b = hl_mod.acquire_boot_lease("boot:amd64", leases=2, timeout=0.0)
    assert a is not None and b is not None
    assert hl_mod.acquire_boot_lease("boot:arm64", leases=2, timeout=0.3) is None  # pool full
    hl_mod.release(a)
    hl_mod.release(b)


def test_hostlock_status_reports_both_pools(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    st = hl_mod.status()
    assert "bootcpu" in st and "boot_leases_total" in st
    # render must not crash and should mention both pools
    body = hl_mod.render_status(st)
    assert "build(cpuheavy)" in body and "boot(windows-host)" in body


def test_ninja_jobs_caps_and_reserves(monkeypatch):
    monkeypatch.setattr(build_mod.os, "cpu_count", lambda: 32)
    monkeypatch.setenv("VIBEOS_BUILD_CORE_RESERVE", "8")
    assert build_mod._ninja_jobs() == ["-j", "24"]
    monkeypatch.setenv("VIBEOS_BUILD_CORE_RESERVE", "0")
    assert build_mod._ninja_jobs() == []  # uncapped
    monkeypatch.setenv("VIBEOS_BUILD_CORE_RESERVE", "8")
    monkeypatch.setattr(build_mod.os, "cpu_count", lambda: 4)
    assert build_mod._ninja_jobs() == []  # tiny host: don't cap below the reserve


# --- lints: summary/tally lines are not findings (merge-magnet fix) ---------


def test_finding_lines_excludes_count_summaries():
    out = "\n".join(
        [
            "Spec catalog: 1571 exports",
            "Bindgen catalog: 2921 functions",
            "  C_MACRO: 94",
            "  FLEX_ARRAY: 66",
            "  TOTAL: 160 issues across 2348 files",
            "SUMMARY: HIGH=0  MEDIUM=0  LOW(likely-FP)=10  shown=0",
            "  -> MISMATCH             : 0",
            "  matched to a C array    : 25",
            "  [C_MACRO] sdk/lib/rust/cc-rust/src/cacheman.rs:38: real finding",
            "      - ARITY: rust 0 params vs C 5",  # detail line: KEPT (real content)
        ]
    )
    got = lints._finding_lines(out)
    assert "  [C_MACRO] sdk/lib/rust/cc-rust/src/cacheman.rs:38: real finding" in got
    assert "      - ARITY: rust 0 params vs C 5" in got
    assert len(got) == 2  # every count-bearing tally line dropped


# --- build: isolation preflight — in-tree cargo hardlinks are benign ---------


def _mk_cargo_tree(repo, nlinks_extra, escape_dir=None):
    """A fake worktree with a cargo target dir whose deps file has extra
    hardlinks (cargo-incremental style). Optionally add one link OUTSIDE."""
    ws = repo / "sdk" / "lib" / "rust" / "target"
    deps = ws / "x86_64-pc-windows-gnu" / "debug" / "deps"
    deps.mkdir(parents=True)
    (ws / "CACHEDIR.TAG").write_text("Signature: 8a477f597d28d172789f06886806bc55")
    f = deps / "libfoo.rmeta"
    f.write_text("meta")
    for i in range(nlinks_extra):
        d = ws / "x86_64-pc-windows-gnu" / "debug" / "incremental" / f"s-{i}"
        d.mkdir(parents=True)
        os.link(f, d / "metadata.rmeta")
    if escape_dir is not None:
        escape_dir.mkdir(parents=True, exist_ok=True)
        os.link(f, escape_dir / "libfoo.rmeta")
    return f


def test_isolation_preflight_allows_in_tree_incremental_links(tmp_path):
    # cargo incremental: deps + 2 session dirs = 3 links, ALL in-tree → benign
    # (the 2026-07-07 gate FP: nlink>2 alone is not the dedup signature).
    repo = tmp_path / "wt"
    repo.mkdir()
    _mk_cargo_tree(repo, nlinks_extra=2)
    run = build_mod.Run("build", "amd64", ts=1700000000)
    assert build_mod._isolation_preflight(run, repo, repo / "build-amd64") is True


def test_isolation_preflight_blocks_cross_worktree_links(tmp_path):
    # one link escaping the worktree = the dedup-incident signature → refuse.
    repo = tmp_path / "wt"
    repo.mkdir()
    _mk_cargo_tree(repo, nlinks_extra=1, escape_dir=tmp_path / "other-wt")
    run = build_mod.Run("build", "amd64", ts=1700000000)
    assert build_mod._isolation_preflight(run, repo, repo / "build-amd64") is False
    assert run.classification == "ISOLATION_FAIL"


# --- conformance verb (quest M4) -------------------------------------------
def _conf_repo_root() -> Path:
    # tests/qemu/cli/test_cli_units.py -> repo root is parents[3]
    return Path(__file__).resolve().parents[3]


def test_conformance_arch_normalization():
    m = conformance_mod._PYTEST_ARCH
    assert m.get("amd64", "amd64") == "amd64-uefi"
    assert m.get("i386", "i386") == "i386-uefi"
    assert m.get("arm64", "arm64") == "arm64"  # arm64 is itself
    assert m.get("amd64-uefi", "amd64-uefi") == "amd64-uefi"  # suffixed passes through


def test_conformance_parser_detects_crash_regression(tmp_path):
    parser = conformance_mod._load_parser(_conf_repo_root())
    log = (
        "[KMTEST-SWEEP] launch KeEvent\n"
        "[KMTEST:END KeEvent pass=42733 fail=0 skip=0]\n"
        "[KMTEST-SWEEP] exit KeEvent code=0\n"
        "[KMTEST-SWEEP] launch NpfsFileInfo\n"
        "*** BUGCHECK: 0x19\n"
    )
    results, done, bc = parser.parse(log)
    assert [r["status"] for r in results] == ["OK", "CRASH"]
    assert bc == "0x19" and done is None
    bp = tmp_path / "base.txt"
    bp.write_text("KeEvent OK pass=42733 fail=0 skip=0\nNpfsFileInfo OK pass=1 fail=0 skip=0\n")
    regs = parser.compare(results, parser.load_baseline(bp))
    assert any("NpfsFileInfo" in r and "CRASH" in r for r in regs)


def test_conformance_rostests_preflight(tmp_path):
    # no build dir -> None (unknown); either option ON -> True; both off/absent -> False.
    assert conformance_mod._rostests_enabled(tmp_path, "amd64") is None
    bd = tmp_path / "build-amd64"
    bd.mkdir()
    (bd / "CMakeCache.txt").write_text("FOO:BOOL=ON\nENABLE_ROSTESTS:BOOL=ON\n")
    assert conformance_mod._rostests_enabled(tmp_path, "amd64") is True
    # the fleet-default config: parent OFF but KMTESTS ON, parent listed FIRST
    # (the first-match bug this guards against).
    (bd / "CMakeCache.txt").write_text(
        "ENABLE_ROSTESTS:BOOL=OFF\nENABLE_ROSTESTS_KMTESTS:BOOL=ON\n"
    )
    assert conformance_mod._rostests_enabled(tmp_path, "amd64") is True
    (bd / "CMakeCache.txt").write_text(
        "ENABLE_ROSTESTS:BOOL=OFF\nENABLE_ROSTESTS_KMTESTS:BOOL=OFF\n"
    )
    assert conformance_mod._rostests_enabled(tmp_path, "amd64") is False
    (bd / "CMakeCache.txt").write_text("FOO:BOOL=ON\n")
    assert conformance_mod._rostests_enabled(tmp_path, "amd64") is False


# --- cohab staleness: the mtime chain (.rs -> cargo .a -> copied .a -> binary) -----------
#
# These pin BOTH directions, because the bug they encode was a checker that looked healthy:
# the obvious rule (binary must be newer than the newest .rs) false-RED the arm64 gate on a
# codegen-neutral edit, and the obvious replacement (`ninja -n`) was structurally blind and
# passed the benign case while missing a real stale cohab. Field manual §9.81.


@pytest.fixture
def cohab_repo(tmp_path: Path, monkeypatch) -> Path:
    """A fake arm64 tree with the full staticlib-cohab chain for one crate."""
    r = tmp_path / "repo"
    crate = r / "sdk" / "lib" / "rust" / "widget-rust" / "src"
    crate.mkdir(parents=True)
    (crate.parent / "Cargo.toml").write_text("[package]\nname='widget-rust'\n")
    (crate / "lib.rs").write_text("// v1\n")
    mod = r / "build-arm64" / "drivers" / "widget"
    cargo_out = mod / "rust-widget-target" / "aarch64-pc-windows-gnullvm" / "release"
    cargo_out.mkdir(parents=True)
    (r / "qemu").mkdir(parents=True)
    (r / "qemu" / "mkramdisk.sh").write_text('RMC "$BUILD_DIR/drivers/widget/widget.sys" ::/w\n')
    for p, t in (
        (crate / "lib.rs", 1000),
        (cargo_out / "libwidget_rust.a", 2000),
        (mod / "libwidget_rust.a", 2000),
        (mod / "widget.sys", 2000),
    ):
        p.parent.mkdir(parents=True, exist_ok=True)
        if not p.exists():
            p.write_bytes(b"x")
        os.utime(p, (t, t))
    return r


def _cohab(repo: Path) -> list[str]:
    return [x for x in provenance.cohab_staleness_reasons(repo, "arm64") if "widget" in x]


def test_cohab_clean_tree(cohab_repo: Path):
    assert _cohab(cohab_repo) == []


def test_cohab_codegen_neutral_edit_is_NOT_stale(cohab_repo: Path):
    """THE false-RED. The .rs is newer than the binary, but cargo re-ran and emitted an
    identical .a, so copy_if_different left the copy (and hence the link) alone. Nothing is
    stale — the binary genuinely reflects the source."""
    src = cohab_repo / "sdk/lib/rust/widget-rust/src/lib.rs"
    os.utime(src, (3000, 3000))  # edited AFTER the binary was linked
    cargo_a = next((cohab_repo / "build-arm64").rglob("rust-widget-target/**/libwidget_rust.a"))
    os.utime(cargo_a, (4000, 4000))  # cargo re-ran...
    # ...and the copy + binary were NOT touched, because the bytes did not change.
    assert _cohab(cohab_repo) == []


def test_cohab_crate_not_rebuilt_is_stale(cohab_repo: Path):
    """Rule A: the .rs changed and cargo never ran => the crate is not in this arch's DAG."""
    os.utime(cohab_repo / "sdk/lib/rust/widget-rust/src/lib.rs", (5000, 5000))
    r = _cohab(cohab_repo)
    assert r and "cargo's own output" in r[0], r


def test_cohab_binary_not_relinked_is_stale(cohab_repo: Path):
    """Rule B: the staticlib really changed but the .sys was never relinked (the gdi32 class
    that this whole check exists for)."""
    mod = cohab_repo / "build-arm64" / "drivers" / "widget"
    os.utime(mod / "libwidget_rust.a", (6000, 6000))  # copy updated => bytes DID change
    r = _cohab(cohab_repo)
    assert r and "never relinked" in r[0], r


def test_cohab_cargo_output_missing_is_stale(cohab_repo: Path):
    """The crate has a staged staticlib but was never compiled for this arch."""
    next((cohab_repo / "build-arm64").rglob("rust-widget-target/**/libwidget_rust.a")).unlink()
    os.utime(cohab_repo / "sdk/lib/rust/widget-rust/src/lib.rs", (5000, 5000))
    r = _cohab(cohab_repo)
    assert r and "missing" in r[0], r


# --- tier-2: non-Rust build inputs (the deleted `ninja -n` oracle's job) -----------------
#
# The regression these pin is a FALSE GREEN, not a crash: `_ninja_has_work` returned False
# for every target on every arch (CONFIGURE_DEPENDS globs make -n stop at the regen edge),
# and rust_src_hash only covers sdk/lib/rust — so editing a tracked, compiled .c and running
# smoke classified it as "clean-diff (non-build change)", called the tree FRESH, and booted
# the stale iso. Field manual §9.81b.


def test_staleness_c_source_change_is_stale(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "dll" / "win32" / "thing" / "thing.c").write_text("int f(void){return 2;}\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale and not v.fresh, v.reasons
    assert any("build source changed" in r for r in v.reasons), v.reasons


def test_staleness_lint_baseline_change_is_not_stale(staleness_repo: Path):
    """A lint baseline is accepted-findings bookkeeping, not a build input.

    Counterpart to the .c/CMakeLists tests above: those pin that build-relevant edits
    must NOT read as fresh; this pins the opposite error. tools/lint-baseline/*.txt is
    only ever read by lints -- never compiled, never staged -- so counting it made the
    documented build -> `check --update-baseline` -> smoke loop demand a second, no-op
    rebuild every single time. The exclusion is deliberately narrow; the rest of tools/
    carries real compiled sources, which the next test pins.

    The file must be COMMITTED before the stamp: git_fingerprint folds untracked paths
    in via `git status --porcelain`, which reports the path and not the content, so a
    test that edits an untracked file passes no matter what the pathspec says.
    """
    bl = staleness_repo / "tools" / "lint-baseline"
    bl.mkdir(parents=True, exist_ok=True)
    (bl / "some_scanner.txt").write_text("accepted finding A\n")
    _git(staleness_repo, "add", "-A")
    _git(staleness_repo, "commit", "-qm", "baseline")
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (bl / "some_scanner.txt").write_text("accepted finding A\naccepted finding B\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.fresh and not v.stale, v.reasons


def test_staleness_other_tools_source_still_stale(staleness_repo: Path):
    """tools/ at large stays in the fingerprint -- it carries compiled sources."""
    t = staleness_repo / "tools" / "libschily"
    t.mkdir(parents=True, exist_ok=True)
    (t / "helper.c").write_text("int h(void){return 1;}\n")
    _git(staleness_repo, "add", "-A")
    _git(staleness_repo, "commit", "-qm", "tools src")
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (t / "helper.c").write_text("int h(void){return 2;}\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale, v.reasons
    assert any("build source changed" in r for r in v.reasons), v.reasons


def test_staleness_cmake_change_is_stale(staleness_repo: Path):
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "dll" / "win32" / "thing" / "CMakeLists.txt").write_text("# retarget\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale, v.reasons


def test_staleness_spec_change_is_stale(staleness_repo: Path):
    """A .spec is neither Rust nor C, and it changes the DLL's export table."""
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "dll" / "win32" / "thing" / "thing.spec").write_text("@ stdcall g()\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale, v.reasons


def test_staleness_untracked_c_file_is_stale(staleness_repo: Path):
    """A brand-new, never-committed source still changes the build."""
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "dll" / "win32" / "thing" / "extra.c").write_text("int g(void){return 0;}\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale, v.reasons


def test_staleness_old_stamp_without_build_hash_is_loud_not_clean(staleness_repo: Path):
    """A stamp written before build_src_hash existed cannot answer the question. It must
    say so, not read as clean — an absent field defaulting to "" that compares equal to
    nothing is the same silent-guess shape this whole fix is about."""
    s = provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1)
    s.build_src_hash = ""  # simulate a pre-existing stamp
    s.write(staleness_repo / "build-i386")
    (staleness_repo / "docs" / "readme.md").write_text("force tier 2\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.stale, v.reasons
    assert any("predates build-source fingerprinting" in r for r in v.reasons), v.reasons


def test_staleness_doc_change_still_clean_diff(staleness_repo: Path):
    """The must-NOT-fire half: docs are excluded, so a docs-only delta stays FRESH. The
    gate's docs-only fast path depends on this, and an over-broad build-relevant set would
    quietly turn every doc edit into a full rebuild."""
    provenance.make_stamp(staleness_repo, "i386", cold=True, ts=1).write(
        staleness_repo / "build-i386"
    )
    (staleness_repo / "docs" / "readme.md").write_text("changed doc\n")
    (staleness_repo / "notes.md").write_text("top-level markdown\n")
    v = provenance.check_staleness(staleness_repo, "i386")
    assert v.fresh and not v.stale, v.reasons


# --- land: the serialized land critical section (kills the gate→push race) --

from . import land as land_mod  # noqa: E402


def test_boot_leases_scale_with_host_capacity(monkeypatch):
    """The bootcpu pool was a flat 4 — EXACTLY one full gate's boot fan-out (3 arches +
    net), so one gating lane saturated the fleet. It now derives from the host."""
    for cpus, want in ((24, 8), (32, 10), (64, 12), (12, 4), (4, 4), (0, 4)):
        monkeypatch.setattr(os, "cpu_count", lambda c=cpus: c)
        assert hl_mod._derive_boot_leases() == want, cpus


def test_land_lock_is_exclusive(tmp_path, monkeypatch):
    """Two lanes must never be inside fetch→merge→gate→push at once — that window IS the
    race, and the whole point is that master cannot move under a gating lane."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    with hl_mod.land_lock("land:a", timeout=1.0):
        with pytest.raises(hl_mod.LandLockBusy) as ei, hl_mod.land_lock("land:b", timeout=0.3):
            pass
        # The error must NAME the blocker: "who is landing" is the first thing you ask.
        assert "holder" in str(ei.value) and str(os.getpid()) in str(ei.value)
    # Released on exit → the next lane gets in.
    with hl_mod.land_lock("land:c", timeout=1.0):
        pass


def test_land_lock_is_fifo(tmp_path, monkeypatch):
    """A patient lander must not lose to a latecomer (the LIFO degeneration §hostlock)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    ahead = _fake_ticket(tmp_path, "land", ns=1, pid=999999, need=1)
    try:
        with pytest.raises(hl_mod.LandLockBusy), hl_mod.land_lock("land:latecomer", timeout=0.3):
            pass
    finally:
        os.close(ahead)  # earlier waiter dies → its ticket is an orphan
    with hl_mod.land_lock("land:latecomer", timeout=1.0):
        pass


def test_land_lock_never_silently_degrades(tmp_path, monkeypatch):
    """The CPU pools warn-and-proceed on timeout; this one MUST NOT — proceeding unlocked
    reopens the exact race it exists to close (fail loud, never silently degrade)."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    with (
        hl_mod.land_lock("land:holder", timeout=1.0),
        pytest.raises(hl_mod.LandLockBusy),
        hl_mod.land_lock("land:other", timeout=0.2),
    ):
        raise AssertionError("must never reach the body")


def test_land_lock_opt_out(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_LAND_LOCK", "0")
    with (
        hl_mod.land_lock("land:a", timeout=0.2),
        hl_mod.land_lock("land:b", timeout=0.2),  # no-op, no exclusion
    ):
        pass


def test_host_status_shows_who_is_landing(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    with hl_mod.land_lock("land:shell32-com", timeout=1.0):
        h = hl_mod.land_holder()
        assert h.get("kind") == "land:shell32-com" and h.get("pid") == os.getpid()
        rendered = hl_mod.render_status(hl_mod.status())
        assert "LANDING" in rendered and "land:shell32-com" in rendered
    assert hl_mod.land_holder() == {}


# --- land: stamp acceptance mirrors the pre-push hook ----------------------


def _write_stamp(repo: Path, tree: str, *, age_s: float = 0, arches=("i386", "amd64", "arm64")):
    d = repo / ".git" / "vibeos-gate-stamps"
    d.mkdir(parents=True, exist_ok=True)
    (d / f"{tree}.json").write_text(
        json.dumps({"tree": tree, "time": time.time() - age_s, "arches": list(arches)})
    )


def test_fresh_full_stamp_accepts_exact_tree(tiny_repo: Path):
    tree = subprocess.run(
        ["git", "rev-parse", "HEAD^{tree}"], cwd=tiny_repo, capture_output=True, text=True
    ).stdout.strip()
    _write_stamp(tiny_repo, tree)
    assert land_mod.fresh_full_stamp(tiny_repo, tree) is not None


def test_fresh_full_stamp_rejects_stale_partial_and_other_tree(tiny_repo: Path):
    tree = subprocess.run(
        ["git", "rev-parse", "HEAD^{tree}"], cwd=tiny_repo, capture_output=True, text=True
    ).stdout.strip()
    _write_stamp(tiny_repo, tree, age_s=land_mod.STAMP_MAX_AGE_S + 60)
    assert land_mod.fresh_full_stamp(tiny_repo, tree) is None, "24h-stale stamp accepted"
    _write_stamp(tiny_repo, tree, arches=("i386", "amd64"))
    assert land_mod.fresh_full_stamp(tiny_repo, tree) is None, "partial-arch stamp accepted"
    _write_stamp(tiny_repo, tree)
    assert land_mod.fresh_full_stamp(tiny_repo, "0" * 40) is None, "wrong-tree stamp accepted"


# --- land: preflight + full flow -------------------------------------------


def test_land_preflight_refuses_dirty_tree(tiny_repo: Path):
    (tiny_repo / "a.txt").write_text("uncommitted\n")
    with pytest.raises(land_mod.LandAbort) as ei:
        land_mod._preflight(tiny_repo)
    assert "uncommitted" in str(ei.value)


def test_land_preflight_refuses_detached_head(tiny_repo: Path):
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=tiny_repo, capture_output=True, text=True
    ).stdout.strip()
    subprocess.run(["git", "checkout", "-q", head], cwd=tiny_repo, capture_output=True)
    with pytest.raises(land_mod.LandAbort) as ei:
        land_mod._preflight(tiny_repo)
    assert "detached HEAD" in str(ei.value)


@pytest.fixture
def lane_repo(tmp_path: Path):
    """A clone with an `origin` whose master has ALREADY moved ahead — i.e. exactly the
    state that used to void a lane's gate stamp and force a re-gate."""
    origin = tmp_path / "origin.git"
    subprocess.run(["git", "init", "-q", "--bare", str(origin)], check=True, capture_output=True)
    seed = tmp_path / "seed"
    _git_init_commit(seed, origin)
    lane = tmp_path / "lane"
    subprocess.run(["git", "clone", "-q", str(origin), str(lane)], check=True, capture_output=True)
    for k, v in (("user.email", "t@t"), ("user.name", "t")):
        subprocess.run(["git", "config", k, v], cwd=lane, check=True, capture_output=True)
    # our lane branch, one commit
    subprocess.run(["git", "checkout", "-qb", "mylane"], cwd=lane, check=True, capture_output=True)
    (lane / "mine.txt").write_text("lane work\n")
    subprocess.run(["git", "add", "-A"], cwd=lane, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "lane work"], cwd=lane, check=True, capture_output=True)
    # ANOTHER lane lands first
    (seed / "theirs.txt").write_text("their work\n")
    subprocess.run(["git", "add", "-A"], cwd=seed, check=True, capture_output=True)
    subprocess.run(
        ["git", "commit", "-qm", "their work"], cwd=seed, check=True, capture_output=True
    )
    subprocess.run(
        ["git", "push", "-q", "origin", "master"], cwd=seed, check=True, capture_output=True
    )
    return lane, origin


def _git_init_commit(path: Path, origin: Path) -> None:
    path.mkdir()
    for a in (
        ["init", "-q", "-b", "master"],
        ["config", "user.email", "t@t"],
        ["config", "user.name", "t"],
    ):
        subprocess.run(["git", *a], cwd=path, check=True, capture_output=True)
    (path / "base.txt").write_text("base\n")
    subprocess.run(["git", "add", "-A"], cwd=path, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "base"], cwd=path, check=True, capture_output=True)
    subprocess.run(
        ["git", "remote", "add", "origin", str(origin)], cwd=path, check=True, capture_output=True
    )
    subprocess.run(
        ["git", "push", "-q", "-u", "origin", "master"], cwd=path, check=True, capture_output=True
    )


class _Args:
    def __init__(self, **kw):
        self.retries = 2
        self.clean = False
        self.no_push = False
        self.dry_run = False
        self.queue_timeout = 5
        # `land` defaults to the merge-QUEUE path; these direct-path tests opt out
        # explicitly (and test_land_default_path_submits_to_the_queue covers the default).
        self.direct = True
        self.drain = False
        self.wait = False
        self.__dict__.update(kw)


def test_land_merges_master_then_gates_then_pushes_in_that_order(lane_repo, tmp_path, monkeypatch):
    """The whole mechanism in one test: land merges the CURRENT origin/master, gates the
    RESULT, and pushes — all inside the lock, so the gated tree is the pushed tree."""
    lane, origin = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    order: list[str] = []

    def fake_gate(repo, run, args):
        order.append("gate")
        # the tree being gated must already contain origin/master
        remote = subprocess.run(
            ["git", "rev-parse", "origin/master"], cwd=repo, capture_output=True, text=True
        ).stdout.strip()
        assert (
            subprocess.run(
                ["git", "merge-base", "--is-ancestor", remote, "HEAD"], cwd=repo
            ).returncode
            == 0
        ), "gated a tree that does not contain origin/master — the race is back"

    monkeypatch.setattr(land_mod, "_run_gate", fake_gate)
    real_push = land_mod._push

    def spy_push(repo, run, head):
        order.append("push")
        return real_push(repo, run, head)

    monkeypatch.setattr(land_mod, "_push", spy_push)
    assert land_mod.land(_Args(), lane) == 0, "land failed"
    assert order == ["gate", "push"]
    # origin/master now IS our lane tip, reached by fast-forward.
    landed = subprocess.run(
        ["git", "rev-parse", "master"], cwd=origin, capture_output=True, text=True
    ).stdout.strip()
    mine = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=lane, capture_output=True, text=True
    ).stdout.strip()
    assert landed == mine
    # ...and the local `master` ref was NOT force-moved from this worktree (the phantom
    # staged-revert footgun): only origin advanced.
    local_master = subprocess.run(
        ["git", "rev-parse", "master"], cwd=lane, capture_output=True, text=True
    ).stdout.strip()
    assert local_master != mine


def test_land_skips_the_gate_when_a_fresh_stamp_already_covers_the_tree(
    lane_repo, tmp_path, monkeypatch
):
    """The fast path: you gated, nobody raced you → landing costs seconds, not a re-gate."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    # bring the lane up to date ourselves, then stamp the resulting tree
    subprocess.run(["git", "fetch", "-q", "origin"], cwd=lane, check=True, capture_output=True)
    subprocess.run(
        ["git", "merge", "-q", "--no-edit", "origin/master"],
        cwd=lane,
        check=True,
        capture_output=True,
    )
    tree = subprocess.run(
        ["git", "rev-parse", "HEAD^{tree}"], cwd=lane, capture_output=True, text=True
    ).stdout.strip()
    _write_stamp(lane, tree)
    called = []
    monkeypatch.setattr(land_mod, "_run_gate", lambda *a: called.append(1))
    assert land_mod.land(_Args(), lane) == 0
    assert called == [], "re-gated a tree that already had a fresh green stamp"


def test_land_aborts_on_conflicting_merge_without_holding_the_lock_hostage(
    lane_repo, tmp_path, monkeypatch
):
    """A conflict is the lane's problem to resolve — don't sit on the fleet's land lock
    while a human thinks, and don't leave a half-merged tree behind."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    # make our lane touch the same file the other lane added, with different content
    (lane / "theirs.txt").write_text("conflicting\n")
    subprocess.run(["git", "add", "-A"], cwd=lane, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "conflict"], cwd=lane, check=True, capture_output=True)
    monkeypatch.setattr(land_mod, "_run_gate", lambda *a: pytest.fail("gate ran on a conflict"))
    assert land_mod.land(_Args(), lane) != 0
    assert hl_mod.land_holder() == {}, "land lock still held after abort"
    assert not (lane / ".git" / "MERGE_HEAD").exists(), "left a half-merged tree"


# --- gate: the build phase submits arches concurrently (broker does the limiting) --


def test_gate_builds_are_submitted_concurrently(tmp_path, monkeypatch):
    """The gate used to build arches in a SERIAL loop, which was stricter than the broker it
    delegates to (2 concurrent builds are sanctioned fleet-wide) and made the build phase
    sum-of-arches. It matters more now that `land` serializes gates, so gate wall-clock IS
    the landing throughput limit. Here: all three must be in flight together."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    live = 0
    peak = 0
    lock = threading.Lock()
    barrier = threading.Barrier(3, timeout=10)

    def fake_build(repo, run, arch, args):
        nonlocal live, peak
        with lock:
            live += 1
            peak = max(peak, live)
        barrier.wait()  # deadlocks (and fails) if the phase is serial
        with lock:
            live -= 1
        return {"arch": arch, "ok": True, "exit": 0, "seconds": 0.01}

    monkeypatch.setattr(gate_mod, "_build_once", fake_build)
    monkeypatch.setattr(
        gate_mod, "_smoke_once", _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    )
    monkeypatch.setattr(
        gate_mod,
        "_net_once",
        lambda repo, root: {
            "arch": "net",
            "verdict": "OK",
            "classification": "",
            "ok": True,
            "retryable": False,
            "seconds": 0.01,
            "run_dir": str(root),
            "salient": "",
        },
    )
    rc = gate_mod.gate(_gate_args(no_build=False), tmp_path)
    assert rc == 0
    assert peak == 3, f"builds did not overlap (peak={peak})"


def test_gate_failed_build_skips_that_arch_but_others_still_smoke(tmp_path, monkeypatch):
    """A build failure is fatal for ITS arch only — the rest must still build+smoke so the
    matrix is complete, and the failed arch must never be smoked on stale media."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(
        gate_mod,
        "_build_once",
        lambda repo, run, arch, args: {
            "arch": arch,
            "ok": arch != "arm64",
            "exit": 0 if arch != "arm64" else 1,
            "seconds": 0.01,
        },
    )
    fake = _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(no_build=False, no_net=True), tmp_path)
    assert rc == 1, "a build failure must fail the gate"
    assert not any(k.startswith("arm64:") for k in fake.calls), fake.calls
    assert any(k.startswith("i386:") for k in fake.calls), fake.calls
    assert any(k.startswith("amd64:") for k in fake.calls), fake.calls


def test_gate_refuses_to_stamp_a_tree_it_did_not_gate(tmp_path, monkeypatch, tiny_repo):
    """The stamp is a claim about what was VALIDATED. It used to read HEAD's tree at stamp
    time, so a commit landed during a 20-minute gate got a credential for content nothing
    had built or booted."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(
        gate_mod, "_smoke_once", _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    )

    def moved_tree(repo):
        # gate start reports one tree; the repo's real HEAD tree is a different one
        return ("0" * 40, False)

    monkeypatch.setattr(gate_mod, "_tree_state", moved_tree)
    rc = gate_mod.gate(_gate_args(no_net=True), tiny_repo)
    assert rc == 0, "the gate itself is still green"
    stamps = list((tiny_repo / ".git" / "vibeos-gate-stamps").glob("*.json"))
    assert stamps == [], "stamped a tree the gate never validated"


def test_gate_stamps_the_tree_it_gated(tmp_path, monkeypatch, tiny_repo):
    """The must-pass half: an unchanged clean checkout still mints the credential."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(
        gate_mod, "_smoke_once", _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    )
    rc = gate_mod.gate(_gate_args(no_net=True), tiny_repo)
    assert rc == 0
    tree = subprocess.run(
        ["git", "rev-parse", "HEAD^{tree}"], cwd=tiny_repo, capture_output=True, text=True
    ).stdout.strip()
    assert (tiny_repo / ".git" / "vibeos-gate-stamps" / f"{tree}.json").is_file()


def test_gate_refuses_to_stamp_when_dirty_at_start(tmp_path, monkeypatch, tiny_repo):
    """Gating a dirty tree validates content no commit hashes to — certify nothing."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(
        gate_mod, "_smoke_once", _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    )
    real = gate_mod._tree_state(tiny_repo)
    monkeypatch.setattr(gate_mod, "_tree_state", lambda repo: (real[0], True))
    gate_mod.gate(_gate_args(no_net=True), tiny_repo)
    assert not list((tiny_repo / ".git" / "vibeos-gate-stamps").glob("*.json"))


# --- lint baseline: line DRIFT is not a new finding, but a new OCCURRENCE is --


def test_lint_baseline_ignores_pure_line_drift():
    """Inserting a comment above a baselined finding moved it 90→93 and turned the gate RED
    with '1 new' (2026-07-25). Nothing an author can do about that except re-bless the
    baseline — which launders whatever else drifted in the same pass."""
    base = {"qemu/lib/common.sh:90:17: warning: SLOT_INDEX appears unused. [SC2034]"}
    cur = {"qemu/lib/common.sh:93:17: warning: SLOT_INDEX appears unused. [SC2034]"}
    assert lints._new_findings(cur, base) == []


def test_lint_baseline_still_flags_a_new_occurrence_of_a_baselined_message():
    """The multiset half: the SAME message twice where the baseline recorded it once is a
    genuinely new site and must stay red. A plain set-of-keys diff would mask it."""
    base = {"a.sh:10:1: warning: X unused. [SC2034]"}
    cur = {
        "a.sh:12:1: warning: X unused. [SC2034]",
        "a.sh:40:1: warning: X unused. [SC2034]",
    }
    new = lints._new_findings(cur, base)
    assert len(new) == 1, new


def test_lint_baseline_still_flags_a_new_file_and_a_new_message():
    base = {"a.sh:10:1: warning: X unused. [SC2034]"}
    assert lints._new_findings({"b.sh:10:1: warning: X unused. [SC2034]"}, base) != []
    assert lints._new_findings({"a.sh:10:1: warning: Y unused. [SC2034]"}, base) != []


def test_lint_baseline_ignores_drift_in_lint_kernels_embedded_line_number():
    """lint_kernel carries the line number INSIDE the message, so the `file:line:` stripper
    above never applied to it and it kept drifting (2026-07-25: adding 4-line banners to 12
    files reported 35 "new" findings that were the same 35 sites). The scanner prints a file
    header, then indented `L <n> Fn(): ...` lines."""
    base = {"    L 309 RtlpCheckDeviceName(): *NameInvalid = ..."}
    cur = {"    L 313 RtlpCheckDeviceName(): *NameInvalid = ..."}
    assert lints._new_findings(cur, base) == []


def test_lint_baseline_still_flags_an_extra_embedded_line_occurrence():
    """The multiset half again — drift immunity must not swallow a genuinely new site."""
    base = {"    L 309 RtlpCheckDeviceName(): *NameInvalid = ..."}
    cur = {
        "    L 313 RtlpCheckDeviceName(): *NameInvalid = ...",
        "    L 480 RtlpCheckDeviceName(): *NameInvalid = ...",
    }
    assert len(lints._new_findings(cur, base)) == 1
    # ...and a different function at a drifted line is still new.
    assert lints._new_findings({"    L 313 RtlpOther(): *NameInvalid = ..."}, base) != []


def test_lint_baseline_ignores_drift_behind_a_leading_class_tag():
    """verify_ffi and scan_magic_offsets print `[TAG] path:line: msg`. _LOCATION is
    start-anchored, so it never saw the path and 205 baseline lines were silently NOT
    drift-immune — a one-line comment added to fastfat's extern block moved an unrelated
    finding 177 -> 178 and the merge queue rejected the whole batch (2026-07-25)."""
    base = {"  [C_MACRO] a/b.rs:177: IoGetCurrentIrpStackLocation: is a C macro"}
    cur = {"  [C_MACRO] a/b.rs:178: IoGetCurrentIrpStackLocation: is a C macro"}
    assert lints._new_findings(cur, base) == []
    # The tag is part of the identity: same site, different class, is a NEW finding.
    other_class = {"  [FLEX_ARRAY] a/b.rs:177: IoGetCurrentIrpStackLocation: is a C macro"}
    assert lints._new_findings(other_class, base) != []
    # ...and the multiset half still holds.
    extra = {"  [C_MACRO] a/b.rs:900: IoGetCurrentIrpStackLocation: is a C macro"}
    assert len(lints._new_findings(cur | extra, base)) == 1


def test_lint_baseline_ignores_drift_in_a_trailing_parenthesized_location():
    """verify_signatures puts the location last: `[MEDIUM] NtSetLdtEntries  (path:210)`."""
    base = {"[MEDIUM] NtSetLdtEntries  (sdk/lib/rust/x/stubs.rs:210)"}
    drifted = {"[MEDIUM] NtSetLdtEntries  (sdk/lib/rust/x/stubs.rs:255)"}
    assert lints._new_findings(drifted, base) == []
    assert lints._new_findings({"[MEDIUM] NtOther  (sdk/lib/rust/x/stubs.rs:210)"}, base) != []
    reclassified = {"[HIGH] NtSetLdtEntries  (sdk/lib/rust/x/stubs.rs:210)"}
    assert lints._new_findings(reclassified, base) != []


def test_every_baseline_line_is_immune_to_line_drift():
    """The GUARD over the three fixes above. Each was found the same way — a lane's unrelated
    edit shifted a line, the gate went red, and someone had to re-bless a baseline (which
    launders whatever else drifted in the same pass). The failure is invisible at authorship
    time: a new scanner with a novel output shape looks fine until it costs another lane a
    gate. So assert the PROPERTY directly over every baseline we actually ship — perturbing a
    finding's line number must never change its key. A scanner whose format this cannot parse
    fails check-tooling here, where its author is standing, instead of a stranger's merge."""
    # A location we are confident is a location: a real source extension, then digits.
    loc = re.compile(r"(?P<f>[\w./+-]+\.(?:rs|c|h|py|md|txt|cmake|spec|toml|sh)):(?P<n>\d+)")
    baselines = sorted((_conf_repo_root() / "tools" / "lint-baseline").glob("*.txt"))
    assert baselines, "no lint baselines found — wrong REPO root?"
    blind: list[str] = []
    for path in baselines:
        for line in path.read_text().splitlines():
            m = loc.search(line)
            if not line.strip() or not m:
                continue
            drifted = line[: m.start("n")] + str(int(m.group("n")) + 1000) + line[m.end("n") :]
            if lints._finding_key(line) != lints._finding_key(drifted):
                blind.append(f"{path.name}: {line[:110]}")
    assert not blind, (
        f"{len(blind)} baseline finding(s) carry a line number that _finding_key does not "
        f"strip, so any edit above them reads as NEW:\n  " + "\n  ".join(blind[:10])
    )


def test_lint_baseline_compares_prefixless_findings_verbatim():
    """Tools that don't emit `file:line:` (verify_* MISMATCH lines) keep exact matching."""
    base = {"MISMATCH: NtFoo stdcall vs cdecl"}
    assert lints._new_findings({"MISMATCH: NtFoo stdcall vs cdecl"}, base) == []
    assert lints._new_findings({"MISMATCH: NtBar stdcall vs cdecl"}, base) != []


# --- the merge queue: submit / batch / split-attribution / reject channel ---

from . import queue as q_mod  # noqa: E402


@pytest.fixture
def queue_repo(tmp_path: Path, monkeypatch):
    """A clone with an `origin` and two lanes ready to submit."""
    origin = tmp_path / "origin.git"
    subprocess.run(["git", "init", "-q", "--bare", str(origin)], check=True, capture_output=True)
    seed = tmp_path / "seed"
    _git_init_commit(seed, origin)
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path / "root"))
    return seed, origin


def _commit(repo: Path, name: str, body: str = "x\n") -> str:
    (repo / name).write_text(body)
    subprocess.run(["git", "add", "-A"], cwd=repo, check=True, capture_output=True)
    subprocess.run(
        ["git", "commit", "-qm", f"add {name}"], cwd=repo, check=True, capture_output=True
    )
    return subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=repo, capture_output=True, text=True
    ).stdout.strip()


def test_queue_submit_pins_the_tip_with_a_ref(queue_repo):
    """The record must never be visible before the ref: a conductor that reads a submission
    whose objects can be GC'd would fail to land work that was accepted."""
    repo, _ = queue_repo
    tip = _commit(repo, "mine.txt")
    sub = q_mod.submit(repo)
    assert sub.tip == tip
    assert (
        subprocess.run(
            ["git", "rev-parse", "--verify", sub.ref], cwd=repo, capture_output=True, text=True
        ).stdout.strip()
        == tip
    )
    assert [s.seq for s in q_mod.pending(repo)] == [sub.seq]


def test_queue_is_fifo_by_arrival(queue_repo):
    repo, _ = queue_repo
    _commit(repo, "a.txt")
    first = q_mod.submit(repo, lane="lane-a")
    _commit(repo, "b.txt")
    second = q_mod.submit(repo, lane="lane-b")
    assert [s.seq for s in q_mod.pending(repo)] == [first.seq, second.seq]


def test_queue_resolve_always_leaves_a_verdict(queue_repo):
    """A submission leaving the queue WITHOUT a result is the one unrecoverable outcome —
    the agent can never learn what happened to its work."""
    repo, _ = queue_repo
    _commit(repo, "a.txt")
    sub = q_mod.submit(repo)
    q_mod.resolve(repo, sub, outcome="rejected", reason="nope", classification="GATE_RED")
    assert q_mod.pending(repo) == []
    recs = q_mod.results(repo)
    assert len(recs) == 1 and recs[0]["outcome"] == "rejected" and recs[0]["seq"] == sub.seq
    # the ref is released too, so the objects are collectable
    assert not q_mod.git_ok(repo, "rev-parse", "--verify", sub.ref)


def test_queue_conflicting_submission_is_bounced_not_resolved(queue_repo, monkeypatch):
    """A conflict is the lane's intent to reconcile, never the conductor's to guess. The rest
    of the batch must still proceed."""
    repo, _origin = queue_repo
    # lane A lands "shared.txt" on master directly
    _commit(repo, "shared.txt", "master version\n")
    subprocess.run(
        ["git", "push", "-q", "origin", "master"], cwd=repo, check=True, capture_output=True
    )
    # a conflicting lane: same file, different content, branched from before
    subprocess.run(
        ["git", "checkout", "-q", "-b", "conflicting", "HEAD~1"],
        cwd=repo,
        check=True,
        capture_output=True,
    )
    _commit(repo, "shared.txt", "lane version\n")
    bad = q_mod.submit(repo, lane="conflicting")
    # an innocent lane touching a different file
    subprocess.run(
        ["git", "checkout", "-q", "-b", "innocent", "master"],
        cwd=repo,
        check=True,
        capture_output=True,
    )
    _commit(repo, "other.txt")
    good = q_mod.submit(repo, lane="innocent")

    wt = repo  # gate in-place for the unit test; the real conductor uses its own worktree
    logs: list[str] = []
    res = q_mod.build_batch(repo, wt, [bad, good], logs.append)
    assert [s.seq for s in res.conflicted] == [bad.seq]
    assert [s.seq for s in res.merged] == [good.seq]
    assert any("CONFLICT" in m for m in logs)
    assert not (repo / ".git" / "MERGE_HEAD").exists(), "left the tree mid-merge"


def test_queue_red_batch_splits_and_spares_the_innocent(queue_repo, monkeypatch):
    """The whole point of attribution: one bad submission must not reject its neighbours.
    Gate says red iff the batch contains the culprit."""
    repo, _ = queue_repo
    subs = []
    for lane in ("a", "b", "c", "d"):
        subprocess.run(
            ["git", "checkout", "-q", "-b", f"l{lane}", "master"],
            cwd=repo,
            check=True,
            capture_output=True,
        )
        _commit(repo, f"{lane}.txt")
        subs.append(q_mod.submit(repo, lane=f"lane-{lane}"))
    culprit = subs[2]

    gated: list[tuple[str, ...]] = []

    def fake_build(repo_, wt_, batch, log):
        return q_mod.BatchResult(ok=False, tip="t" * 40, merged=list(batch), conflicted=[])

    def fake_gate(wt_, log, retries=2):
        # red exactly when the culprit is in the current subset
        subset = gated[-1]
        ok = culprit.lane not in subset
        return ok, "" if ok else "BUGCHECK", "/tmp/art", "smoke:amd64: BUGCHECK 0x50"

    def tracking_build(repo_, wt_, batch, log):
        gated.append(tuple(s.lane for s in batch))
        return fake_build(repo_, wt_, batch, log)

    monkeypatch.setattr(q_mod, "build_batch", tracking_build)
    monkeypatch.setattr(q_mod, "gate_batch", fake_gate)
    monkeypatch.setattr(q_mod, "git", lambda *a, **k: "")
    monkeypatch.setattr(q_mod, "git_ok", lambda *a, **k: True)
    # This scenario is "the base is fine, one lane is bad" — see
    # test_queue_red_base_rejects_nobody for the other half.
    monkeypatch.setattr(q_mod.BaseCheck, "is_green", lambda *a, **k: True)

    q_mod.land_subset(repo, repo, subs, lambda m: None)
    by_lane = {r["lane"]: r for r in q_mod.results(repo, limit=50)}
    assert by_lane["lane-c"]["outcome"] == "rejected", by_lane["lane-c"]
    assert by_lane["lane-c"]["classification"] == "BUGCHECK"
    assert by_lane["lane-c"]["artifact"] == "/tmp/art"
    for lane in ("lane-a", "lane-b", "lane-d"):
        assert by_lane[lane]["outcome"] == "landed", (lane, by_lane[lane])
    assert q_mod.pending(repo) == [], "every submission must leave the queue with a verdict"


def test_queue_green_batch_lands_everything_in_one_gate(queue_repo, monkeypatch):
    repo, _ = queue_repo
    subs = []
    for lane in ("a", "b", "c"):
        subprocess.run(
            ["git", "checkout", "-q", "-b", f"g{lane}", "master"],
            cwd=repo,
            check=True,
            capture_output=True,
        )
        _commit(repo, f"g{lane}.txt")
        subs.append(q_mod.submit(repo, lane=f"lane-{lane}"))
    gates = []
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="f" * 40, merged=list(b)),
    )
    monkeypatch.setattr(
        q_mod, "gate_batch", lambda w, log, retries=2: (gates.append(1), True, "", "", "")[1:]
    )
    monkeypatch.setattr(q_mod, "git", lambda *a, **k: "")
    monkeypatch.setattr(q_mod, "git_ok", lambda *a, **k: True)
    q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert len(gates) == 1, "3 landings must cost ONE gate"
    assert all(r["outcome"] == "landed" for r in q_mod.results(repo, limit=50))


def test_queue_conductor_election_is_the_land_lock(queue_repo, monkeypatch):
    """No daemon: whoever finds the lock free trains. A second agent must NOT block."""
    repo, _ = queue_repo
    monkeypatch.setattr(q_mod, "drain", lambda *a, **k: 0)
    with hl_mod.land_lock("someone-else", timeout=1.0):
        assert q_mod.try_become_conductor(repo, lambda m: None) is False
    assert q_mod.try_become_conductor(repo, lambda m: None) is True


def test_queue_already_landed_submission_needs_no_gate(queue_repo):
    """Free landing: a tip already in master (it rode someone else's batch) must be resolved,
    not re-gated."""
    repo, _ = queue_repo
    tip = _commit(repo, "landed.txt")
    subprocess.run(
        ["git", "push", "-q", "origin", "master"], cwd=repo, check=True, capture_output=True
    )
    subprocess.run(["git", "fetch", "-q", "origin"], cwd=repo, check=True, capture_output=True)
    sub = q_mod.submit(repo, tip=tip)
    assert q_mod.already_landed(repo, sub) is True


def test_land_default_path_submits_to_the_queue(lane_repo, tmp_path, monkeypatch):
    """`land` with no flags must go through the merge queue, not the direct path — the direct
    path is the opt-out (`--direct`), because a fleet landing directly is the serialization
    this exists to remove. And it must spawn the conductor DETACHED: the elected agent was the
    last one still blocked on a gate (which is why every dev drain went through a hand-rolled
    nohup wrapper)."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    monkeypatch.setattr(land_mod, "_land_direct", lambda *a: pytest.fail("took the direct path"))
    monkeypatch.setattr(
        q_mod, "drain", lambda *a, **k: pytest.fail("conducted INLINE — must be detached")
    )
    spawned = []
    monkeypatch.setattr(land_mod, "_spawn_detached", lambda argv, cwd, log: spawned.append(argv))
    rc = land_mod.land(_Args(direct=False), lane)
    assert rc == 0
    assert len(spawned) == 1, spawned
    assert spawned[0][1:] == ["land", "--drain", "--foreground"], spawned[0]
    assert q_mod.pending(lane), "the submission itself must be queued"


def test_land_foreground_conducts_inline(lane_repo, tmp_path, monkeypatch):
    """--foreground is the escape hatch for watching a batch (and is what the detached
    conductor runs as its own child)."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    monkeypatch.setattr(
        land_mod, "_spawn_detached", lambda *a: pytest.fail("spawned despite --foreground")
    )
    drained = []
    monkeypatch.setattr(q_mod, "drain", lambda repo, log, **k: drained.append(1) or 0)
    assert land_mod.land(_Args(direct=False, foreground=True), lane) == 0
    assert drained == [1]


def test_prune_never_reclaims_the_integration_tree(tmp_path):
    """It is idle between batches — exactly prune's target shape — but its warm caches are
    what make a batch gate in minutes, and EVERY landing in the fleet would pay for a cold
    one."""
    wt = tmp_path / q_mod.INTEGRATION_DIRNAME
    reason = prune.classify(
        wt,
        current=tmp_path / "other",
        live_qemu=set(),
        active_build=set(),
        allow_self=False,
        min_age_hours=0,
        now=0.0,
        newest_mtime=None,
    )
    assert reason and "integration" in reason
    # ...but --self from inside it still allows a deliberate reclaim
    assert (
        prune.classify(
            wt,
            current=wt,
            live_qemu=set(),
            active_build=set(),
            allow_self=True,
            min_age_hours=0,
            now=0.0,
            newest_mtime=None,
        )
        is None
    )


def _safe_git(monkeypatch):
    """Patch out only the MUTATING git verbs. Blanket-patching `queue.git` also breaks
    `queue_dir()` (it resolves --git-common-dir), which silently points pending/results at
    the wrong directory and makes every queue assertion vacuously true — that bit me once."""
    real = q_mod.git
    mutating = {"push", "reset", "clean", "fetch", "merge", "update-ref", "worktree"}

    def safe(repo_, *args, **kw):
        if args and args[0] in mutating:
            return ""
        return real(repo_, *args, **kw)

    monkeypatch.setattr(q_mod, "git", safe)
    monkeypatch.setattr(q_mod, "git_ok", lambda *a, **k: True)


def _two_queued(repo, prefix):
    subs = []
    for lane in ("a", "b"):
        subprocess.run(
            ["git", "checkout", "-q", "-b", f"{prefix}{lane}", "master"],
            cwd=repo,
            check=True,
            capture_output=True,
        )
        _commit(repo, f"{prefix}{lane}.txt")
        subs.append(q_mod.submit(repo, lane=f"lane-{lane}"))
    return subs


def test_queue_red_base_rejects_nobody_and_halts_the_train(queue_repo, monkeypatch):
    """The first REAL batch got this wrong: mainline was red (fltmgr-rust failed to compile
    for i386), the train split down to one submission per half and rejected BOTH — one of
    them a docs-only change, which cannot break a build. A queue that blames the base's
    breakage on whoever happens to be in the batch is worse than no queue."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "rb")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="e" * 40, merged=list(b)),
    )
    # EVERYTHING is red, base included — the honest verdict is "not their fault".
    monkeypatch.setattr(
        q_mod, "gate_batch", lambda w, log, retries=2: (False, "BUILD_FAIL", "/tmp/a", "ninja")
    )
    with pytest.raises(q_mod.BaseRed) as ei:
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert "origin/master itself fails the gate" in str(ei.value)
    assert q_mod.results(repo, limit=20) == [], "rejected a lane for mainline's breakage"
    assert len(q_mod.pending(repo)) == 2, "submissions must stay QUEUED — never judged"


def test_queue_red_batch_with_green_base_rejects_the_guilty(queue_repo, monkeypatch):
    """The mirror: base green ⇒ the submissions really are at fault and must be rejected."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "gb")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="d" * 40, merged=list(b)),
    )
    monkeypatch.setattr(
        q_mod, "gate_batch", lambda w, log, retries=2: (False, "BUGCHECK", "/tmp/a", "boom")
    )
    monkeypatch.setattr(q_mod.BaseCheck, "is_green", lambda *a, **k: True)
    q_mod.land_subset(repo, repo, subs, lambda m: None)
    recs = q_mod.results(repo, limit=20)
    assert len(recs) == 2 and all(r["outcome"] == "rejected" for r in recs), recs
    assert all(r["classification"] == "BUGCHECK" for r in recs)


def test_queue_base_check_is_memoized(queue_repo, monkeypatch):
    """A red batch must gate the base ONCE, not once per half — the base cannot change while
    the conductor holds the land lock, and a full gate is minutes."""
    repo, _ = queue_repo
    _safe_git(monkeypatch)
    calls = []
    monkeypatch.setattr(
        q_mod,
        "gate_batch",
        lambda w, log, retries=2: (calls.append(1), (True, "", "", ""))[1],
    )
    base = q_mod.BaseCheck()
    assert base.is_green(repo, repo, lambda m: None, retries=0) is True
    assert base.is_green(repo, repo, lambda m: None, retries=0) is True
    assert len(calls) == 1, f"base gated {len(calls)}x"


def test_queue_inconclusive_gate_requeues_instead_of_rejecting(queue_repo, monkeypatch):
    """HOST_STARVED / LAUNCH_FAIL / a gate that died without a verdict say NOTHING about the
    code. Rejecting a lane for one sends it hunting a bug that does not exist — the same error
    as blaming a red base, which is why both halt instead."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "inc")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="c" * 40, merged=list(b)),
    )
    gates = []
    monkeypatch.setattr(
        q_mod,
        "gate_batch",
        lambda w, log, retries=2: (gates.append(1), (False, "HOST_STARVED", "/tmp/a", "queued"))[1],
    )
    with pytest.raises(q_mod.Inconclusive):
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert q_mod.results(repo, limit=20) == [], "rejected a lane for an environment failure"
    assert len(q_mod.pending(repo)) == 2
    assert len(gates) == 1, "must NOT split — splitting re-discovers that the host is busy"


def test_queue_empty_classification_is_inconclusive(queue_repo, monkeypatch):
    """A gate that dies without writing a verdict is the LEAST conclusive outcome there is."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "noc")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="b" * 40, merged=list(b)),
    )
    monkeypatch.setattr(q_mod, "gate_batch", lambda w, log, retries=2: (False, "", "", ""))
    with pytest.raises(q_mod.Inconclusive):
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert len(q_mod.pending(repo)) == 2


def test_queue_rejection_names_what_it_was_gated_against(queue_repo, monkeypatch):
    """When an earlier half of the batch has already landed, 'red on this submission alone' is
    misleading — the lane may be green against the master it branched from. The verdict has to
    name the neighbour, or the lane hunts a phantom bug in its own diff."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "ctx")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="a" * 40, merged=list(b)),
    )
    # full batch red; first half green (lands); second half red (genuinely rejected)
    seen = []

    def gate(w, log, retries=2):
        seen.append(1)
        ok = len(seen) == 2  # 1st = full batch (red), 2nd = first half (green), 3rd = second half
        return (True, "", "", "") if ok else (False, "BUGCHECK", "/tmp/a", "boom")

    monkeypatch.setattr(q_mod, "gate_batch", gate)
    monkeypatch.setattr(q_mod.BaseCheck, "is_green", lambda *a, **k: True)
    q_mod.land_subset(repo, repo, subs, lambda m: None)
    rej = [r for r in q_mod.results(repo, limit=20) if r["outcome"] == "rejected"]
    landed = [r for r in q_mod.results(repo, limit=20) if r["outcome"] == "landed"]
    assert len(landed) == 1 and len(rej) == 1, q_mod.results(repo, limit=20)
    assert "gated against master" in rej[0]["reason"]
    assert landed[0]["lane"] in rej[0]["reason"], "must name the neighbour that landed first"


def test_queue_batch_gate_is_cold_and_collects_all_failures(monkeypatch, tmp_path):
    """Warm builds hid THREE simultaneous mainline breaks: a crate that isn't recompiled can't
    fail, and a .sys that isn't relinked can't fail to link. The batch gate must therefore be
    cold AND use -k0, or it keeps certifying a tree that does not build from scratch."""
    seen = {}

    def _fake_run(cmd, **kw):
        seen["cmd"] = cmd
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(q_mod.subprocess, "run", _fake_run)
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    q_mod.gate_batch(tmp_path, lambda m: None, retries=0)
    assert "--clean" in seen["cmd"], seen["cmd"]
    assert "--keep-going" in seen["cmd"], seen["cmd"]


def test_queue_batch_gate_cold_can_be_opted_out(monkeypatch, tmp_path):
    monkeypatch.setenv("VIBEOS_QUEUE_CLEAN", "0")
    seen = {}

    def _fake_run(cmd, **kw):
        seen["cmd"] = cmd
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(q_mod.subprocess, "run", _fake_run)
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    q_mod.gate_batch(tmp_path, lambda m: None, retries=0)
    assert "--clean" not in seen["cmd"]


def test_build_keep_going_maps_to_ninja_k0():
    """-k0 only for verdict builds: local iteration wants fail-fast, since you just broke it."""
    from argparse import Namespace

    assert build_mod._ninja_keep_going(Namespace(keep_going=True)) == ["-k0"]
    assert build_mod._ninja_keep_going(Namespace(keep_going=False)) == []
    assert build_mod._ninja_keep_going(None) == []


def test_queue_submit_is_idempotent_per_tip(queue_repo):
    """Submitting the same (lane, tip) twice must be a no-op — the first live batch contained
    the same canary tip twice because only the CLI path deduped, not submit() itself."""
    repo, _ = queue_repo
    _commit(repo, "dup.txt")
    a = q_mod.submit(repo, lane="lane-x")
    b = q_mod.submit(repo, lane="lane-x")
    assert a.seq == b.seq
    assert len(q_mod.pending(repo)) == 1


def test_queue_submit_supersedes_this_lanes_older_tip(queue_repo):
    """A lane's ancestor tip must not ride along beside its own descendant: harmless to merge,
    but a split can REJECT the ancestor while the descendant lands — a verdict about a commit
    the lane already moved past."""
    repo, _ = queue_repo
    old_tip = _commit(repo, "one.txt")
    first = q_mod.submit(repo, lane="lane-y")
    new_tip = _commit(repo, "two.txt")
    second = q_mod.submit(repo, lane="lane-y")
    assert [s.seq for s in q_mod.pending(repo)] == [second.seq], "ancestor still queued"
    rec = next(r for r in q_mod.results(repo, limit=10) if r["seq"] == first.seq)
    assert rec["outcome"] == "superseded" and new_tip[:12] in rec["reason"]
    assert old_tip != new_tip


def test_queue_submit_keeps_a_DIVERGENT_tip_from_the_same_lane(queue_repo):
    """Only an ANCESTOR is superseded. A rewritten/divergent tip is a different proposal and
    must stay queued on its own merits."""
    repo, _ = queue_repo
    subprocess.run(["git", "checkout", "-q", "-b", "div1", "master"], cwd=repo, capture_output=True)
    _commit(repo, "d1.txt")
    first = q_mod.submit(repo, lane="lane-z")
    subprocess.run(["git", "checkout", "-q", "-b", "div2", "master"], cwd=repo, capture_output=True)
    _commit(repo, "d2.txt")
    second = q_mod.submit(repo, lane="lane-z")
    assert {s.seq for s in q_mod.pending(repo)} == {first.seq, second.seq}


def test_queue_conductor_can_wait_for_its_turn(queue_repo, monkeypatch):
    """The orphan window: a conductor that gives up instantly leaves a submission with nobody
    to drain it (seen within minutes of shipping — a pre-queue lane held the lock via the direct
    path). A DETACHED conductor has no reason to give up, so it must be able to wait."""
    repo, _ = queue_repo
    drained = []
    monkeypatch.setattr(q_mod, "drain", lambda *a, **k: drained.append(1) or 0)
    # lock held by "someone else" → a 0-timeout conductor bows out, a waiting one would block
    with hl_mod.land_lock("other-lane", timeout=1.0):
        assert q_mod.try_become_conductor(repo, lambda m: None, timeout=0.0) is False
        assert q_mod.try_become_conductor(repo, lambda m: None, timeout=0.2) is False
        assert drained == []
    assert q_mod.try_become_conductor(repo, lambda m: None, timeout=1.0) is True
    assert drained == [1]


def test_land_drain_waits_but_submit_path_does_not(lane_repo, tmp_path, monkeypatch):
    """`--drain` (the detached child) waits for the lock; the inline-from-submit convenience
    path must not, because there the agent's own turn is on the line."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    seen = []
    monkeypatch.setattr(
        q_mod,
        "try_become_conductor",
        lambda repo, log, retries=2, timeout=0.0: seen.append(timeout) or True,
    )
    land_mod.land(_Args(direct=False, drain=True, foreground=True), lane)
    assert seen[-1] == hl_mod.LAND_LOCK_TIMEOUT, seen
    land_mod.land(_Args(direct=False, foreground=True), lane)
    assert seen[-1] == 0.0, seen


def test_gate_recovers_the_child_builds_own_classification(tmp_path: Path):
    """A build failure the child fully understood was reaching the merge queue as `""`.

    queue.INCONCLUSIVE contains `""` on purpose — it means "the gate died without writing a
    verdict", the least conclusive outcome there is. So dropping the child's classification
    made every named build failure indistinguishable from a host hiccup: the train halted,
    rejected nobody, and re-gated the whole batch. Five times on 2026-07-25, each a full
    3-build/4-boot cycle.
    """
    from . import gate as gate_mod

    (tmp_path / "summary.json").write_text(json.dumps({"classification": "BUILD_INCOMPLETE"}))
    stdout = f"build arm64  —  FAIL: BUILD_INCOMPLETE\nfull log:  {tmp_path}/full.log\n"
    assert gate_mod._child_classification(stdout) == "BUILD_INCOMPLETE"


def test_gate_classification_recovery_fails_safe(tmp_path: Path):
    """Every failure path must yield "" — the old behaviour — never an exception. This runs
    inside the gate's build loop, so raising here would take down a landing."""
    from . import gate as gate_mod

    assert gate_mod._child_classification("") == ""
    assert gate_mod._child_classification("no artifact line at all") == ""
    assert gate_mod._child_classification("full log:  /definitely/not/here/full.log") == ""
    (tmp_path / "summary.json").write_text("{not json")
    assert gate_mod._child_classification(f"full log:  {tmp_path}/full.log") == ""


def test_build_incomplete_does_not_reject_a_lane():
    """It means ninja exited 0 but a cohab PE is older than its staticlib — and the stale set
    was DIFFERENT on all 5 occurrences (kernel32 / version / shell32+usbport /
    advapi32+kernel32+ws2_32+mouhid). A real missing dependency edge is not random, so it says
    nothing about the submission and must not bounce a lane until it is root-caused."""
    from . import queue as queue_mod

    assert "BUILD_INCOMPLETE" in queue_mod.INCONCLUSIVE
    # ...while genuinely conclusive reds still reject. DIFFTEST_FAIL belongs here:
    # a rust-vs-C divergence is deterministic and reproduces off any host, so it
    # is evidence about the submission, not about the fleet.
    for conclusive in ("BUILD_FAIL", "LINT_FAIL", "TOOLING_FAIL", "DIFFTEST_FAIL"):
        assert conclusive not in queue_mod.INCONCLUSIVE


def test_archive_serial_copies_into_the_run_dir(tmp_path: Path):
    """The point of the copy: the live path is a per-arch FIXED name that the next
    boot truncates, so summary.json must point at a snapshot, not at that name."""
    from .artifacts import archive_serial

    live = tmp_path / "amd64-uefi_smoke_live.log"
    live.write_text("[POOL] entry=ffff prev=2 size=4\nBUGCHECK 0x19\n")
    run_dir = tmp_path / "run"
    run_dir.mkdir()
    extra: dict = {}
    archive_serial(run_dir, live, extra)

    assert extra["serial_log"] == str(run_dir / "serial.log")
    assert extra["live_log"] == str(live)
    assert "BUGCHECK 0x19" in (run_dir / "serial.log").read_text()

    # The whole bug, reproduced: the next boot of this arch truncates the live log.
    live.write_text("")
    assert "BUGCHECK 0x19" in (run_dir / "serial.log").read_text()


def test_archive_serial_never_fails_a_run(tmp_path: Path):
    """Build-only failures produce no serial at all; that is normal, not an error,
    and this helper runs inside finalize() where raising would lose the summary."""
    from .artifacts import archive_serial

    run_dir = tmp_path / "run"
    run_dir.mkdir()

    extra: dict = {}
    archive_serial(run_dir, None, extra)
    assert extra == {}

    extra = {}
    archive_serial(run_dir, tmp_path / "never-booted.log", extra)
    assert "serial_log" not in extra
    assert not (run_dir / "serial.log").exists()


# --- vibeos-cli difftest: the host rust-vs-C harnesses ----------------------


def test_difftest_discovers_crates_from_the_workspace(tmp_path: Path):
    """Membership comes from the manifest, never a hardcoded list — a list is how
    a harness goes missing when someone adds one."""
    from . import difftest as difftest_mod

    ws = tmp_path / "sdk" / "lib" / "rust"
    ws.mkdir(parents=True)
    (ws / "Cargo.toml").write_text(
        '[workspace]\nmembers = [\n  "cnt-rust",\n  "cnt-difftest",\n'
        '  "msvcrt-difftest",\n  "rtl-difftest",\n  "ntdll-sys",\n]\n'
    )
    assert difftest_mod.discover(tmp_path) == ["cnt-difftest", "msvcrt-difftest", "rtl-difftest"]


def test_difftest_discovers_a_newly_added_harness(tmp_path: Path):
    """The whole point of discovery: a crate nobody told the CLI about is covered
    the moment it joins the workspace."""
    from . import difftest as difftest_mod

    ws = tmp_path / "sdk" / "lib" / "rust"
    ws.mkdir(parents=True)
    (ws / "Cargo.toml").write_text('[workspace]\nmembers = ["shell32-difftest", "cnt-rust"]\n')
    assert difftest_mod.discover(tmp_path) == ["shell32-difftest"]


def test_difftest_counts_cases_across_harness_dialects():
    """Each crate reports its count differently; the gate needs one number."""
    from . import difftest as difftest_mod

    assert difftest_mod.graded("difftest result: ok. 81 passed; 0 failed") == 81
    assert difftest_mod.graded("  160 mbcs cases passed") == 160
    assert difftest_mod.graded("test result: ok. 87 passed; 0 failed; 0 ignored") == 87


def test_difftest_a_harness_that_graded_nothing_is_a_FAILURE(tmp_path, monkeypatch):
    """cargo exits 0 for a suite that compared nothing, and every crate emits
    `test result: ok. 0 passed` for its empty lib/doc-test targets. Trusting the
    exit code alone would report green over a harness whose cases all vanished —
    the vacuous pass, which is worse than no harness because green stops anyone
    looking again."""
    from types import SimpleNamespace

    from . import artifacts
    from . import difftest as difftest_mod

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    run = artifacts.Run("difftest", "host")
    monkeypatch.setattr(
        run,
        "run",
        lambda *a, **k: SimpleNamespace(
            returncode=0, stdout="test result: ok. 0 passed; 0 failed", stderr=""
        ),
    )
    assert difftest_mod.run_difftests(tmp_path, run, ["cnt-difftest"]) is False
    assert run.stages[0].status == "fail"
    assert "NOTHING" in run.stages[0].detail


def test_difftest_a_real_harness_run_is_green(tmp_path, monkeypatch):
    from types import SimpleNamespace

    from . import artifacts
    from . import difftest as difftest_mod

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    run = artifacts.Run("difftest", "host")
    monkeypatch.setattr(
        run,
        "run",
        lambda *a, **k: SimpleNamespace(
            returncode=0, stdout="difftest result: ok. 81 passed; 0 failed", stderr=""
        ),
    )
    assert difftest_mod.run_difftests(tmp_path, run, ["cnt-difftest"]) is True
    assert run.stages[0].detail == "81 cases"


def test_difftest_runs_one_cargo_invocation_per_crate(tmp_path, monkeypatch):
    """NOT one `cargo test -p a -p b`. cnt-difftest pulls cnt-rust with the `cabi`
    feature and msvcrt-difftest deliberately does not; unified into one build,
    cnt-rust's 208 #[no_mangle] shims collide with msvcrt-crt-rust's wrappers
    (`duplicate symbol: mbtowc`) and nothing links."""
    from types import SimpleNamespace

    from . import artifacts
    from . import difftest as difftest_mod

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    run = artifacts.Run("difftest", "host")
    seen: list[list[str]] = []

    def _spy(cmd, **kw):
        seen.append(cmd)
        return SimpleNamespace(returncode=0, stdout="difftest result: ok. 3 passed", stderr="")

    monkeypatch.setattr(run, "run", _spy)
    difftest_mod.run_difftests(tmp_path, run, ["cnt-difftest", "msvcrt-difftest"])
    assert len(seen) == 2, "each harness gets its own cargo invocation"
    for cmd in seen:
        assert cmd.count("-p") == 1, f"one crate per invocation, got {cmd}"


def test_difftest_salient_keeps_the_assertion_not_the_linker_line():
    """A failed harness must surface the two values that disagreed."""
    from . import difftest as difftest_mod

    out = (
        "     Running tests/diff.rs\n"
        "test diff_rtl_uni_to_str ... FAILED\n"
        "thread 'diff_rtl_uni_to_str' panicked at rtl-difftest/tests/diff.rs:901:17:\n"
        "assertion `left == right` failed: RtlUnicodeStringToAnsiString fields a=1\n"
        "  left: (6, 7, false)\n"
        "  right: (6, 7, true)\n"
    )
    got = difftest_mod._salient(out)
    assert any("RtlUnicodeStringToAnsiString" in ln for ln in got)
    assert any(ln.startswith("left:") for ln in got)


def test_faulting_module_offset_beats_a_guessed_reloc():
    """The bugcheck banner resolves the module itself, so the RVA is right there.
    arm64/amd64 have no _RELOC_BY_ARCH entry, so before this every one of their
    bugchecks declined to symbolicate — including the -smp 1 0x1E that turned out
    to be a fault on the FIRST store into the trap frame."""
    serial = (
        "*** BUGCHECK: 0x0000001E (0xffffffffc0000005, 0xfffff80042193788, "
        "0xffff8000048e8e50, 0x0000000000000000)\n"
        "***   P2 = ntoskrnl.exe+0x3788\n"
    )
    assert triage.faulting_module_offset(serial) == ("ntoskrnl.exe", 0x3788)
    # a log with no resolved-module line falls back rather than inventing one
    assert triage.faulting_module_offset("*** BUGCHECK: 0x0000007E (0,0,0,0)\n") is None


def test_faulting_module_offset_takes_the_LAST_bugcheck():
    """Same rule as faulting_pc: the crash being triaged is the one the boot died
    on, not the first the log happens to mention."""
    serial = "***   P2 = ntoskrnl.exe+0x1111\n***   P2 = ntoskrnl.exe+0x2222\n"
    assert triage.faulting_module_offset(serial) == ("ntoskrnl.exe", 0x2222)


# --- the gate must never reject a lane without saying why -------------------


def test_smoke_stage_that_dies_before_a_verdict_keeps_its_evidence(tmp_path, monkeypatch):
    """A smoke that aborts before writing summary.json used to be recorded as a
    bare `smoke:arm64 FAIL` with an empty salient, an empty run dir, and the
    child's console thrown away -- which is exactly how a lane got rejected with
    nothing to read. Name the condition, keep the output, carry the exit code."""

    class _Proc:
        returncode = 3
        stdout = "staleness: refusing to boot\n"
        stderr = "fatal: media older than the tree\n"

    monkeypatch.setattr(gate_mod.subprocess, "run", lambda *a, **k: _Proc())
    root = tmp_path / "smoke-arm64-a1"
    r = gate_mod._smoke_once(tmp_path, "arm64", root, [])

    assert r["ok"] is False
    assert r["classification"] == "SMOKE_NO_SUMMARY"
    assert r["exit"] == 3
    # the console survives on disk AND the last line is quoted in the salient
    assert (root / "smoke-console.log").read_text().count("staleness") == 1
    assert "media older than the tree" in r["salient"]
    assert "exited 3" in r["salient"]


def test_smoke_stage_with_a_real_verdict_is_untouched(tmp_path, monkeypatch):
    """The no-summary path must not swallow ordinary failures: a smoke that DID
    render a verdict keeps that verdict and its own classification."""

    class _Proc:
        returncode = 1
        stdout = "boot failed\n"
        stderr = ""

    monkeypatch.setattr(gate_mod.subprocess, "run", lambda *a, **k: _Proc())
    root = tmp_path / "smoke-amd64-a1"
    (root / "20260101-000000-smoke-amd64").mkdir(parents=True)
    (root / "20260101-000000-smoke-amd64" / "summary.json").write_text(
        json.dumps({"verdict": "FAIL", "classification": "BUGCHECK", "stages": []})
    )
    r = gate_mod._smoke_once(tmp_path, "amd64", root, [])
    assert r["classification"] == "BUGCHECK"
    assert r["exit"] == 1


# --- an optional ninja pass can invalidate the main pass's output -----------


def test_tooling_and_difftest_keep_their_console_and_never_report_a_bare_exit(
    tmp_path, monkeypatch
):
    """A gate stage that can turn the whole train red must leave something to
    read. TOOLING_FAIL did not: its dir held only artifacts the pytest suites
    happened to drop there, the gate's full.log never sees the subprocess, and
    the salient is scraped by a marker search that misses whenever the tool does
    not print the marker — ruff and pyright do not, and a traceback goes to
    stderr. The train sat red with a bare 'exit 1' and nowhere to look."""

    class _Proc:
        returncode = 1
        stdout = "ruff: 3 errors\nsomething went wrong\n"
        stderr = "Traceback (most recent call last):\nAssertionError: boom\n"

    monkeypatch.setattr(gate_mod.subprocess, "run", lambda *a, **k: _Proc())

    for fn, logname in (
        (gate_mod._tooling_once, "check-tooling-console.log"),
        (gate_mod._difftest_once, "difftest-console.log"),
    ):
        root = tmp_path / logname
        r = fn(tmp_path, root)
        assert r["ok"] is False and r["exit"] == 1
        kept = (root / logname).read_text()
        assert "ruff: 3 errors" in kept and "AssertionError: boom" in kept
        # no marker in stdout -> the salient falls back to the child's last words
        assert "exit 1" in r["salient"] and "AssertionError: boom" in r["salient"]


def test_summary_records_which_worktree_the_run_belongs_to(tmp_path):
    """VIBEOS_ARTIFACT_ROOT defaults to /tmp/vibeos-cli for EVERY worktree, so
    every lane's run dirs interleave in one directory and picking "the newest
    build-arm64 run" hands you whichever lane started last. That mis-attribution
    already produced a wrong status report. The owner belongs in the artifact."""
    repo = tmp_path / "vibeos-smp"
    repo.mkdir()
    run = Run("build", "arm64", ts=1700000000)
    run.add(Stage("ninja", status="ok"))
    run.finalize(repo)
    summary = json.loads((run.dir / "summary.json").read_text())
    assert summary["worktree"] == str(repo.resolve())

    # ...and a run finalized without a repo says so rather than inventing one
    run2 = Run("build", "arm64", ts=1700000001)
    run2.finalize()
    assert json.loads((run2.dir / "summary.json").read_text())["worktree"] is None


def test_ninja_did_work_counts_edges_not_just_dirtiness():
    """`relinked N` is the interesting number: a re-affirm that keeps relinking
    the same cell every build is a wiring bug, not a hiccup it absorbed."""
    assert build_mod._ninja_did_work("ninja: no work to do.\n") == 0
    assert (
        build_mod._ninja_did_work(
            "[1/3] Building kernel-rust\n[2/3] Linking ntoskrnl.exe\n[3/3] done\n"
        )
        == 3
    )
    # progress lines only count at the start of a line -- a compiler diagnostic
    # that happens to quote one must not inflate the count
    assert build_mod._ninja_did_work("note: see [1/3] in the log\n") == 0


def test_ninja_did_work_does_not_count_the_glob_recheck_pseudo_edge():
    """VERBATIM output from a real no-op re-affirm that still reported
    "relinked 1 target(s)". `[0/n]` is the CONFIGURE_DEPENDS glob re-check, a
    status line, not a completed edge -- real edges start at [1/n]. Counting it
    made the number always >=1, which kills the signal in the direction that
    HIDES the wiring bug the count exists to expose."""
    real = "[0/2] Re-checking globbed directories...\nninja: no work to do.\n"
    assert build_mod._ninja_did_work(real) == 0
    # "no work to do" is authoritative even if progress lines preceded it
    assert build_mod._ninja_did_work("[1/2] Building x\nninja: no work to do.\n") == 0
    # a glob re-check followed by REAL work still counts only the real edges
    assert (
        build_mod._ninja_did_work(
            "[0/2] Re-checking globbed directories...\n[1/1] Linking ntoskrnl.exe\n"
        )
        == 1
    )
