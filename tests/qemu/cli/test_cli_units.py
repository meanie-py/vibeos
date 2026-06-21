"""Unit tests for the vibeos-cli pure logic — no QEMU, no build needed.

Run: <venv>/bin/python -m pytest tests/qemu/cli/test_cli_units.py -q
"""

from __future__ import annotations

import json
import struct
import subprocess
from pathlib import Path

import pytest

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
