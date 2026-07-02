"""Unit tests for the vibeos-cli pure logic — no QEMU, no build needed.

Run: <venv>/bin/python -m pytest tests/qemu/cli/test_cli_units.py -q
"""

from __future__ import annotations

import json
import os
import socket
import struct
import subprocess
import threading
import time
from pathlib import Path

import pytest

from tests.qemu import backend as be_mod
from tests.qemu import qmp_client, serial_client

from . import build as build_mod
from . import config as config_mod
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
    (repo / "sdk" / "lib" / "rust" / "target" / "x86_64-kernel").mkdir(parents=True)
    amd64_stem = provenance.workspace_cargo_target(repo, "amd64-uefi")
    assert amd64_stem is not None and amd64_stem.name == "x86_64-kernel"


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
    serial = "[EXPLORER] _tWinMain entered\nBugCheck 0xF4 (CRITICAL_OBJECT...)\n"
    r = triage.classify(serial, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "KNOWN_FLAKE"
    assert r["known_flake"] == "tWinMain_0xF4_race"
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
    # i386 nettest race: nettest spawned, explorer never reached _tWinMain
    nettest = "...\n[WINLOGON] nettest.exe created (PID 268)\n[NETSTACK] TDI self-test: skipped\n"
    r = triage.classify(nettest, reached=False, fail_marker=None, debugcon=None)
    assert r["classification"] == "KNOWN_FLAKE"
    assert r["known_flake"] == "i386_nettest_explorer_race"

    # the `none` guard: if explorer DID reach _tWinMain it's NOT this flake
    r2 = triage.classify(
        nettest + "[EXPLORER] _tWinMain entered\n", reached=False, fail_marker=None, debugcon=None
    )
    assert r2["classification"] == "HANG_TIMEOUT" and r2["known_flake"] is None

    # arm64 LpcExitThread race
    arm = "[PS] PspExitThread Image=System -> about to call LpcExitThread\n"
    r3 = triage.classify(arm, reached=False, fail_marker=None, debugcon=None)
    assert r3["classification"] == "KNOWN_FLAKE"
    assert r3["known_flake"] == "arm64_lpcexitthread_race"

    # a hang-stage flake signature must NOT match a bugcheck (when-gated)
    r4 = triage.classify(
        "BugCheck 0x7E\n[WINLOGON] nettest.exe created\n",
        reached=False,
        fail_marker=None,
        debugcon=None,
    )
    assert r4["classification"] == "BUGCHECK"  # bugcheck branch, hang sig ignored


# --- artifacts: summary.json is the machine-true verdict -------------------


def test_run_summary_json(tmp_path, monkeypatch):
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    # re-import to pick up env? ARTIFACT_ROOT is module-level; build dir explicitly.
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
    assert cachecfg.cache_enabled(SimpleNamespace(cache=None)) is False  # default off
    assert cachecfg.cache_enabled(SimpleNamespace(cache=True)) is True
    monkeypatch.setenv("VIBEOS_BUILD_CACHE", "1")
    assert cachecfg.cache_enabled(SimpleNamespace(cache=False)) is False  # --no-cache beats env
    assert cachecfg.cache_enabled(SimpleNamespace(cache=None)) is True  # env fallback


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
    # whpx: kvm->whpx (irqchip off) and ALSO drop -cpu host — it faults OVMF on FXSAVE in
    # early firmware, so whpx must fall back to qemu's default cpu model.
    assert be_mod.rewrite_accel(base, "whpx") == [
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
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    bd = tmp_path / "build-i386"
    bd.mkdir()
    k = hl_mod.DEFAULT_CPU_LEASES
    with hl_mod.build_lock(bd, "i386"):
        # the build holds the exclusive host → a boot can't get a permit
        assert hl_mod.acquire_cpu_lease("boot:arm64", leases=k, timeout=0.2) is None
    b = hl_mod.acquire_cpu_lease("boot:arm64", leases=k, timeout=0.2)
    assert b is not None  # released when the cm exited
    hl_mod.release(b)


def test_hostlock_cpu_lease_cm_acquires_and_releases(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    with hl_mod.cpu_lease("build:i386", leases=1) as fd:
        assert fd is not None
        assert hl_mod.acquire_cpu_lease("other", leases=1, timeout=0.2) is None  # held
    fd2 = hl_mod.acquire_cpu_lease("after", leases=1, timeout=0.2)
    assert fd2 is not None  # released when the cm exited
    hl_mod.release(fd2)


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
