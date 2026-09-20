"""Unit tests for the vibeos-cli pure logic — no QEMU, no build needed.

Run: <venv>/bin/python -m pytest tests/qemu/cli/test_cli_units.py -q
"""

from __future__ import annotations

import contextlib
import io
import json
import os
import re
import socket
import struct
import subprocess
import threading
import time
from pathlib import Path
from typing import Any, ClassVar, cast

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


def test_git_fingerprint_survives_a_non_utf8_file(tiny_repo: Path):
    """A latin-1 file in the diff must not take the CLI down.

    The Microsoft DDK EULA under drivers/storage/ide/atapi carried a
    French-Canadian warranty clause in latin-1. Deleting it put 0xe9 into
    `git diff HEAD`, and the strict decode in `_run` raised UnicodeDecodeError
    from inside make_stamp — so `vibeos-cli build` and every `land` behind it
    died, with a traceback pointing at the CLI rather than at the file."""
    latin1 = tiny_repo / "eula.txt"
    latin1.write_bytes("garantie limit\xe9e\n".encode("latin-1"))
    fp_untracked = provenance.git_fingerprint(tiny_repo)
    assert len(fp_untracked) == 64

    subprocess.run(["git", "add", "eula.txt"], cwd=tiny_repo, check=True)
    subprocess.run(
        ["git", "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-m", "eula"],
        cwd=tiny_repo,
        check=True,
        capture_output=True,
    )
    latin1.unlink()
    fp_deleted = provenance.git_fingerprint(tiny_repo)
    assert len(fp_deleted) == 64

    # And byte-true, not lossily decoded: two distinct undecodable contents
    # must not collapse onto one U+FFFD run and thus one fingerprint.
    latin1.write_bytes(b"\xe9\xe9")
    fp_a = provenance.git_fingerprint(tiny_repo)
    latin1.write_bytes(b"\xe9\xea")
    assert provenance.git_fingerprint(tiny_repo) != fp_a


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


def _fake_boot_img(default_os: bytes = b"ReactOS_Arm64_UP") -> bytes:
    """arm64 qemu/boot.img: FAT, not ISO9660, but the ini's bytes sit in it just as
    literally.

    Two decoys, both taken from the real image rather than invented:
      - the uefildr blob carries a bare `DefaultOS` token, which is why the needle
        keeps the `=`;
      - the shipped ini's own comment about keeping the entry names equal-length
        says `DefaultOS=` mid-line, which is why the search is anchored to a line
        start. That one is not hypothetical: it took the patcher down on the first
        real boot.img built with the two-entry ini, and the guard raised instead of
        poking the comment.
    """
    return (
        b"\x00" * 32
        + b"uefildr: DefaultOS\x00"
        + b"[FREELOADER]\r\nDefaultOS="
        + default_os
        + b"\r\nTimeOut=0\r\n"
        + b"; flipping DefaultOS= between them is an equal-length poke\r\n"
        + b"[ReactOS_Arm64_UP]\r\nOptions=/SOS /RDPATH=\\ramdisk.img\r\n"
        + b"[ReactOS_Arm64_MP]\r\nOptions=/KERNEL=ntkrnlmp.exe /HAL=halmp.dll\r\n"
        + b"\x00" * 32
    )


def test_smp_patch_selects_the_arm64_mp_entry_with_no_padding(tmp_path: Path):
    """arm64 was the last arch getting its MP pair from a different BUILD. Its two
    entry names are equal length on purpose, so the poke needs no `\\r\\n;` filler."""
    src = tmp_path / "boot.img"
    src.write_bytes(_fake_boot_img())
    out = vm_mod.smp_selected_iso(src, tmp_path / "work")
    assert out != src, "must not patch the build tree's media in place"
    patched = out.read_bytes()
    assert len(patched) == len(src.read_bytes())
    assert b"DefaultOS=ReactOS_Arm64_MP\r\n" in patched
    assert b"DefaultOS=ReactOS_Arm64_UP" not in patched
    # the [ReactOS_Arm64_UP] SECTION must survive; only the selector moves
    assert b"[ReactOS_Arm64_UP]" in patched
    # equal-length swap => the ONLY difference is the one byte U -> M.
    # strict= is the assertion, not lint appeasement: a length change would make
    # zip truncate and still report "1 differing byte", passing a test whose whole
    # point is that the poke did not resize the image.
    assert sum(x != y for x, y in zip(src.read_bytes(), patched, strict=True)) == 1


def test_smp_patch_is_idempotent_on_arm64_media(tmp_path: Path):
    src = tmp_path / "boot.img"
    src.write_bytes(_fake_boot_img(default_os=b"ReactOS_Arm64_MP"))
    assert vm_mod.smp_selected_iso(src, tmp_path / "work") == src


def _stub_mcopy(monkeypatch, ini_text: str = "[FREELOADER]\n/SOS /RDPATH=x\n"):
    """Stand in for mcopy: extraction writes the ini out, write-back is a no-op.

    The real transform needs a FAT image; what these tests are about is the
    CACHING, so the FAT plumbing is stubbed and the calls are recorded.
    """
    calls: list[list[str]] = []

    def fake_run(cmd, **kwargs):
        calls.append(list(cmd))
        # `mcopy -o -i IMG ::/freeldr.ini OUT` — the extraction direction
        if cmd[-2].startswith("::"):
            Path(cmd[-1]).write_text(ini_text)
        return subprocess.CompletedProcess(cmd, 0, b"", b"")

    monkeypatch.setattr(vm_mod.subprocess, "run", fake_run)
    return calls


def test_testkernel_boot_img_is_built_once_and_reused(tmp_path: Path, monkeypatch):
    """This copied 280 MiB on every construction of the config. The transform is
    deterministic, so every copy held identical bytes. The second call must reuse."""
    src = tmp_path / "boot.img"
    src.write_bytes(b"BOOTIMG" * 100)
    work = tmp_path / "work"
    work.mkdir()
    calls = _stub_mcopy(monkeypatch)

    first = vm_mod.testkernel_boot_img(src, work)
    assert first.exists()
    assert first != src, "must not mutate the build tree's media"
    built = len(calls)
    assert built > 0

    second = vm_mod.testkernel_boot_img(src, work)
    assert second == first
    assert len(calls) == built, "second boot rebuilt the image instead of reusing it"


def test_testkernel_boot_img_rekeys_when_the_source_changes(tmp_path: Path, monkeypatch):
    """A rebuild re-stages boot.img. Reusing the old variant would boot stale
    media, which is the one failure worse than the copying it replaces."""
    src = tmp_path / "boot.img"
    src.write_bytes(b"OLD")
    work = tmp_path / "work"
    work.mkdir()
    _stub_mcopy(monkeypatch)

    old = vm_mod.testkernel_boot_img(src, work)
    src.write_bytes(b"NEWER-AND-LONGER")
    os.utime(src, (1, 1))
    new = vm_mod.testkernel_boot_img(src, work)

    assert new != old, "a re-staged boot.img must produce a new cache key"
    assert not old.exists(), "the stale variant should be pruned"


def test_testkernel_boot_img_never_writes_the_shared_file_in_place(tmp_path: Path, monkeypatch):
    """AGENTS.md records SIGBUS from a tool rewriting a shared inode under a
    sibling's mapping. Building via a temp + os.replace means an already-open
    boot keeps a valid inode, so guard that the staging path is what gets
    written and the shared name only ever appears via replace."""
    src = tmp_path / "boot.img"
    src.write_bytes(b"BOOTIMG")
    work = tmp_path / "work"
    work.mkdir()
    _stub_mcopy(monkeypatch)

    replaced: list[tuple[str, str]] = []
    real_replace = os.replace
    monkeypatch.setattr(
        vm_mod.os,
        "replace",
        lambda a, b: (replaced.append((str(a), str(b))), real_replace(a, b))[1],
    )

    out = vm_mod.testkernel_boot_img(src, work)
    assert len(replaced) == 1, "the shared image must appear via exactly one atomic replace"
    staged, final = replaced[0]
    assert str(out) == final
    assert "staging" in staged, "content must be built at a private path first"


def test_mp_entry_names_are_equal_length_to_the_defaults_they_replace():
    """The in-place poke can pad a shorter replacement but never a longer one, and
    arm64 relies on an exact-length swap. Guard the table itself so renaming one
    half of a pair fails here instead of at boot."""
    for shipped, mp in vm_mod._MP_ENTRY_FOR.items():
        assert len(mp) <= len(shipped), f"{mp!r} longer than {shipped!r}: cannot patch in place"
        if len(mp) != len(shipped):
            assert len(shipped) - len(mp) != 1, "1-byte remainder cannot terminate the line"


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
    # two DefaultOS= LINES: cannot tell which one freeldr reads. (Line-initial on
    # purpose — a mid-line mention is a comment and must NOT count; see
    # _fake_boot_img, where exactly that distinction was a real bug.)
    b = tmp_path / "b.iso"
    b.write_bytes(_fake_iso() + b"\r\nDefaultOS=LiveCD\r\n")
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


def _fmt(r: dict, flaky=()) -> str:
    # Mirrors parser.fmt's signature: merge_baseline_rows hands it the `assert?`
    # marks the row already carries, so a re-seed cannot delete them.
    line = f"{r['name']} {r['status']} pass={r['pass']} fail={r['fail']} skip={r['skip']}"
    for a in sorted(set(r.get("asserts") or ()) | set(flaky)):
        line += f"\n    assert{'?' if a in set(flaky) else ''} {a}"
    return line


_BASELINE_FIXTURE = (
    "# kmtest conformance baseline — parse_kmtest_serial.py format\n"
    "# arch=i386\n"
    "FsRtlLegal OK pass=100 fail=32 skip=0\n"
    "IoDeviceObject OK pass=185 fail=4 skip=0\n"
    "KeGuardedMutex OK pass=200 fail=87 skip=0\n"
)


def test_only_reseeds_the_named_rows_and_leaves_the_flaky_one_banked():
    """The whole point of --only: IoDeviceObject is bistable on all three arches,
    so a wholesale rewrite freezes whichever sample this boot happened to produce.
    Re-seeding the deterministic rows must not touch it."""
    results = [
        _res("FsRtlLegal", 132, 0),
        _res("IoDeviceObject", 184, 5),  # the unlucky side of the flake
        _res("KeGuardedMutex", 287, 0),
    ]
    text, refusal = conformance_mod.merge_baseline_rows(
        _BASELINE_FIXTURE, results, ["FsRtlLegal", "KeGuardedMutex"], _fmt
    )
    assert refusal is None
    assert text is not None
    assert "FsRtlLegal OK pass=132 fail=0" in text
    assert "KeGuardedMutex OK pass=287 fail=0" in text
    # untouched — still the banked sample, NOT this boot's 184/5
    assert "IoDeviceObject OK pass=185 fail=4 skip=0" in text
    # comments and ordering survive
    assert text.startswith("# kmtest conformance baseline")
    assert [ln.split()[0] for ln in text.splitlines() if not ln.startswith("#")] == [
        "FsRtlLegal",
        "IoDeviceObject",
        "KeGuardedMutex",
    ]


def test_only_refuses_a_test_this_sweep_never_produced():
    """Naming a test the sweep never launched must not silently no-op — that
    reads as "re-seeded" while the stale row stays on the floor."""
    text, refusal = conformance_mod.merge_baseline_rows(
        _BASELINE_FIXTURE, [_res("FsRtlLegal", 132, 0)], ["FsRtlLegal", "KeSpinLock"], _fmt
    )
    assert text is None
    assert refusal and "REFUSED" in refusal and "    KeSpinLock\n" in refusal


def test_only_appends_a_row_the_baseline_does_not_carry_yet(capsys):
    """An --only name that is new to the baseline is an addition, not a re-seed.
    Appending raises the floor, which is fine — growing the file silently is not."""
    text, refusal = conformance_mod.merge_baseline_rows(
        _BASELINE_FIXTURE, [_res("CmSecurity", 900, 2)], ["CmSecurity"], _fmt
    )
    assert refusal is None
    assert text is not None
    assert text.rstrip().endswith("CmSecurity OK pass=900 fail=2 skip=0")
    assert "did not carry" in capsys.readouterr().out


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
    # A cohab crate is one the workspace actually BUILDS; `parked-rust` is the
    # translated-but-not-yet-flipped shape and is deliberately absent from members.
    (r / "sdk" / "lib" / "rust" / "Cargo.toml").write_text(
        '[workspace]\nmembers = [\n    "foo-rust",\n    "k32-vista-rust",\n]\n'
    )
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


def test_crate_for_stem_ignores_a_parked_non_member_crate(arm64_cohab_repo: Path):
    """A translated-but-NOT-YET-FLIPPED crate is not cohab, and treating it as one
    halted the merge train on 2026-08-04: msv1_0-rust was committed and parked out
    of `members` pending its grind, dll/win32/msv1_0 still built from C, and the
    staleness net compared the fresh .rs against the C-built msv1_0.dll and reported
    `cohab flip not rebuilt` for a flip that had not happened. No rebuild clears
    that, so the gate could render no verdict at all."""
    rust = arm64_cohab_repo / "sdk" / "lib" / "rust"
    (rust / "parked-rust").mkdir()
    (rust / "parked-rust" / "Cargo.toml").write_text("[package]\nname = 'parked-rust'\n")
    provenance._workspace_members.cache_clear()
    assert provenance._crate_for_stem(arm64_cohab_repo, "parked") is None
    # …and it stays out of the boot-set mapping the staleness net walks.
    assert not any(
        "parked" in rel for rel in provenance.boot_set_cohab_rels(arm64_cohab_repo, "arm64")
    )


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
    (r / "build-amd64" / "bar" / "bar.exe.d").write_text(
        "/tmp/vibeos/build-amd64/bar/bar.exe: "
        "/tmp/vibeos/sdk/lib/rust/bar-rust/src/lib.rs "
        "/tmp/vibeos/sdk/lib/rust/bar-rust/Cargo.toml\n"
    )
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


@pytest.mark.parametrize("dependency", ["src/lib.rs", "Cargo.toml"])
def test_cargo_link_only_own_closure(cargo_link_repo: Path, dependency: str):
    test_cargo_link_fresh_binary_not_stale(cargo_link_repo)
    unrelated = cargo_link_repo / "sdk/lib/rust/unrelated.rs"
    unrelated.write_text("// unrelated")
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64") == []
    own = cargo_link_repo / "sdk/lib/rust/bar-rust" / dependency
    os.utime(own, (3000, 3000))
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64")


@pytest.mark.parametrize("rule", [None, "wrong: input\n", "/tmp/vibeos/build-amd64/bar/bar.exe: "])
def test_cargo_link_missing_evidence_fails(cargo_link_repo: Path, rule: str | None):
    depfile = cargo_link_repo / "build-amd64/bar/bar.exe.d"
    if rule is None:
        depfile.unlink()
    else:
        depfile.write_text(rule)
    with pytest.raises(provenance.ProvenanceError, match="cannot verify cargo link inputs"):
        provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64")


def test_cargo_link_depfile_spaces_and_missing_input(cargo_link_repo: Path):
    test_cargo_link_fresh_binary_not_stale(cargo_link_repo)
    asset = cargo_link_repo / "asset with spaces"
    asset.write_text("data")
    os.utime(asset, (1000, 1000))
    depfile = cargo_link_repo / "build-amd64/bar/bar.exe.d"
    depfile.write_text(
        depfile.read_text().rstrip() + " " + chr(92) + "\n /tmp/vibeos/asset\\ with\\ spaces\n"
    )
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64") == []
    asset.unlink()
    with pytest.raises(provenance.ProvenanceError, match="missing dependency"):
        provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64")


def test_cargo_link_directory_watch(cargo_link_repo: Path):
    test_cargo_link_fresh_binary_not_stale(cargo_link_repo)
    src = cargo_link_repo / "sdk/lib/rust/bar-rust/src"
    os.utime(src, (1000, 1000))
    depfile = cargo_link_repo / "build-amd64/bar/bar.exe.d"
    depfile.write_text(
        "/tmp/vibeos/build-amd64/bar/bar.exe: /tmp/vibeos/sdk/lib/rust/bar-rust/src\n"
    )
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64") == []
    os.utime(src / "lib.rs", (3000, 3000))
    assert provenance.cargo_link_staleness_reasons(cargo_link_repo, "amd64")


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


_E3_SERIAL = (
    "[EXPLORER] _tWinMain entered\nshell ready\n"
    "*** BUGCHECK: 0x000000E3 (0xfffffa0006344870, 0xfffffa00061fcc00, "
    "0x0000000000000000, 0x0000000000000003)\n"
)


def test_triage_known_flake_after_boot_marker_is_still_a_known_flake():
    """A documented flake does not stop being one by firing after the marker.

    post_boot_check() used to hardcode known_flake=None, so the SAME arm64 ERESOURCE
    0xE3 was auto-retried before the marker and rejected a green-gated batch out of
    the merge queue after it. Both paths must agree.
    """
    post = triage.post_boot_check(_E3_SERIAL)
    assert post is not None
    assert post["classification"] == "KNOWN_FLAKE"
    assert post["known_flake"] == "arm64_eresource_notowned_0xe3"

    pre = triage.classify(_E3_SERIAL, reached=False, fail_marker=None, debugcon=None)
    assert pre["classification"] == post["classification"]
    assert pre["known_flake"] == post["known_flake"]


def test_triage_unsignatured_bugcheck_after_marker_still_blocks():
    """The other half: a post-marker crash with no signature must NOT be excused."""
    serial = "[EXPLORER] _tWinMain entered\n*** BUGCHECK: 0x0000007E (0x1, 0x2, 0x3, 0x4)\n"
    r = triage.post_boot_check(serial)
    assert r is not None
    assert r["classification"] == "BUGCHECK"
    assert r["known_flake"] is None


def test_triage_flake_note_is_optional():
    """`note` is optional metadata; a matching entry without one must not KeyError."""
    entry = {"name": "t", "when": "bugcheck", "code": "0x1", "any": []}
    assert triage._flake_note(entry) == []
    assert triage._flake_note(None) == []
    assert triage._flake_note({**entry, "note": "hi"}) == ["hi"]


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


# `info registers -a` as QEMU 9 actually prints it for the arm64 `virt` machine.
_ARM64_REGS = """\
=== info status ===
VM status: running

=== info cpus ===
* CPU #0: thread_id=41521
  CPU #1: thread_id=41522

=== info registers -a ===

CPU#0
 PC=ffff800004001234 X00=0000000000000001 X01=ffffa00006122c00
X02=0000000000000000 X03=0000000000000001
PSTATE=204003c5 --C- EL1h

CPU#1
 PC=ffff8000040567ac X00=0000000000000000 X01=0000000000000000
PSTATE=604003c5 -ZC- EL1h
"""


def test_cpu_pcs_credits_each_vcpu_with_its_own_pc():
    # The whole point is telling one wedged CPU from another. A parser that
    # searched the blob globally would report CPU#0's PC twice and quietly erase
    # the difference between "one CPU spinning" and "both stuck in the same place".
    assert triage.cpu_pcs(_ARM64_REGS) == [
        (0, "ffff800004001234"),
        (1, "ffff8000040567ac"),
    ]


def test_cpu_pcs_reads_x86_program_counters_too():
    # amd64 says RIP=, i386 says EIP=, arm64 says PC= — and none of the three is
    # 0x-prefixed, which is why this cannot reuse triage._PC.
    assert triage.cpu_pcs("CPU#0\nRIP=fffff80000401000 RFL=00000246\n") == [(0, "fffff80000401000")]
    assert triage.cpu_pcs("CPU#0\nEIP=805148fd EFL=00000246\n") == [(0, "805148fd")]


def test_cpu_pcs_survives_a_dump_with_no_registers():
    # capture_cpu_state() never raises, so it can hand back "<unavailable: ...>"
    # instead of a register block. That must yield no PCs, not an exception.
    assert triage.cpu_pcs("=== info registers -a ===\n<unavailable: OSError()>\n") == []


def test_symbolicate_cpu_state_names_the_function_each_cpu_is_parked_in(tmp_path):
    tool = tmp_path / "qemu" / "addr2sym.sh"
    tool.parent.mkdir(parents=True)
    tool.write_text("#!/bin/sh\necho 'x\tKxAcquireSpinLock+24\t-'\n")
    out = triage.symbolicate_cpu_state(tmp_path, "arm64", _ARM64_REGS, mp=True)
    assert out == [
        "CPU#0 0xffff800004001234 -> x\tKxAcquireSpinLock+24\t-",
        "CPU#1 0xffff8000040567ac -> x\tKxAcquireSpinLock+24\t-",
    ]


def test_symbolicate_cpu_state_reports_the_raw_pc_when_the_index_is_missing(tmp_path):
    # No addr2sym.sh in this tree. A bare address still localizes a hang far
    # better than the nothing we had before, so it must not be swallowed.
    out = triage.symbolicate_cpu_state(tmp_path, "arm64", _ARM64_REGS)
    assert out == [
        "CPU#0 0xffff800004001234 -> (unresolved)",
        "CPU#1 0xffff8000040567ac -> (unresolved)",
    ]


def test_symbolicate_cpu_state_resolves_mp_addresses_against_the_MP_index(tmp_path):
    # ntkrnlmp.exe is a different binary from ntoskrnl.exe; resolving an MP hang
    # against the UP index names the wrong function and does it silently.
    tool = tmp_path / "qemu" / "addr2sym.sh"
    tool.parent.mkdir(parents=True)
    tool.write_text('#!/bin/sh\necho "ARGS:$*"\n')
    mp = triage.symbolicate_cpu_state(tmp_path, "arm64", _ARM64_REGS, mp=True)
    assert "--mp" in mp[0], mp[0]
    up = triage.symbolicate_cpu_state(tmp_path, "arm64", _ARM64_REGS)
    assert "--mp" not in up[0], up[0]


def test_symbolication_uses_this_boot_kernel_base(tmp_path):
    tool = tmp_path / "qemu" / "addr2sym.sh"
    tool.parent.mkdir(parents=True)
    tool.write_text('#!/bin/sh\necho "ARGS:$*"\n')
    serial = (
        "ntkrnlmp.exe\n"
        "[KRNLBASE] ntoskrnl runtime base=0xFFFFF80042190000\n"
        "*** BUGCHECK: 0x0000001E (0x1, 0xfffff80042205e7c, 0x2, 0x0)\n"
    )
    fault = triage.symbolicate_pc(tmp_path, "arm64", serial)
    assert fault is not None
    assert "--reloc 0xfffff80041d90000" in fault
    assert "--mp" in fault
    cpus = triage.symbolicate_cpu_state(tmp_path, "arm64", _ARM64_REGS, serial=serial, mp=True)
    assert all("--reloc 0xfffff80041d90000" in line for line in cpus)


def test_kernel_relocation_prefers_latest_boot_and_preserves_legacy_behavior():
    assert triage.kernel_relocation("arm64", "") is None
    assert triage.kernel_relocation("i386", "") == "0x800c3000"
    assert (
        triage.kernel_relocation("amd64", "[KRNLBASE] ntoskrnl runtime base=fffff80000400000\n")
        == "0xfffff80000000000"
    )
    serial = (
        "[KRNLBASE] ntoskrnl runtime base=0xFFFFF80042190000\n"
        "[KRNLBASE] ntoskrnl runtime base=0xFFFFF80042290000\n"
    )
    assert triage.kernel_relocation("arm64", serial) == "0xfffff80041e90000"


def test_cpu_state_prefers_the_capture_taken_at_the_deadline(tmp_path):
    # wait_for_boot samples the instant the boot times out. Re-sampling later
    # would agree on a true hang but disagree on a merely-slow boot, which by
    # then has moved on — reporting the wrong place as "where it was stuck".
    from . import smoke as smoke_mod

    class _StubVm:
        hang_cpu_state = "CPU#0\nPC=ffff800004001234\n"

        def capture_cpu_state(self):
            raise AssertionError("must not re-sample when the deadline capture exists")

    path, lines = smoke_mod._cpu_state(
        _StubVm(),  # type: ignore[arg-type]
        tmp_path,
        tmp_path,
        "arm64",
        "",
    )
    assert lines == ["CPU#0 0xffff800004001234 -> (unresolved)"]
    assert path and Path(path).read_text().startswith("CPU#0")


def test_cpu_state_falls_back_to_a_live_sample(tmp_path):
    from . import smoke as smoke_mod

    class _StubVm:
        hang_cpu_state = ""

        def capture_cpu_state(self):
            return "CPU#0\nPC=ffff8000040567ac\n"

    _path, lines = smoke_mod._cpu_state(
        _StubVm(),  # type: ignore[arg-type]
        tmp_path,
        tmp_path,
        "arm64",
        "",
    )
    assert lines == ["CPU#0 0xffff8000040567ac -> (unresolved)"]


def _hang_vm(work: Path, artifact: Path | None, capture):
    """A QemuVM with only what save_hang_state touches — no QEMU, no __init__."""
    v = vm_mod.QemuVM.__new__(vm_mod.QemuVM)
    v.work_dir = work
    v.artifact_dir = artifact
    v.hang_cpu_state = ""
    v.config = vm_mod.ArchConfig(
        arch="arm64", qemu_bin="x", machine_args=[], memory="1G", boot_marker="m"
    )
    v.capture_cpu_state = capture  # type: ignore[method-assign]
    return v


@pytest.fixture
def _no_resample_wait(monkeypatch):
    monkeypatch.setattr(vm_mod, "HANG_RESAMPLE_SECONDS", 0.0)


def test_save_hang_state_also_lands_in_the_durable_artifact_dir(tmp_path, _no_resample_wait):
    # The bug this closes: work_dir under the pytest `vm` fixture is a tmp_path
    # that pytest prunes to the last 3 runs, so a capture written only there is
    # deleted by unrelated later runs. Callers owning a durable run dir set
    # VIBEOS_ARTIFACT_DIR and must get their own copy.
    work, art = tmp_path / "w", tmp_path / "run"
    work.mkdir()
    v = _hang_vm(work, art, lambda: "CPU#0\nPC=ffff800004001234\n")
    v.save_hang_state("sweep drain")

    for d in (work, art):
        body = (d / "hang-cpustate.txt").read_text()
        assert "PC=ffff800004001234" in body
        # The phase is what tells a bare file in a run dir WHICH stall it is.
        assert "sweep drain" in body
        assert "arch=arm64" in body


def test_save_hang_state_separates_a_frozen_box_from_a_blocked_one(tmp_path, _no_resample_wait):
    # The whole point of two samples: a dead clock and a thread blocked forever
    # both park every vCPU in the idle loop and are indistinguishable in ONE
    # dump. Only "did the PC move" tells them apart, and it has to be decided
    # while the guest is alive, so the artifact must carry the verdict.
    frozen = _hang_vm(tmp_path / "a", None, lambda: "CPU#0\nPC=ffff800004001234\n")
    frozen.save_hang_state("sweep drain")
    assert "FROZEN" in (tmp_path / "a" / "hang-cpustate.txt").read_text()

    pcs = iter(["CPU#0\nPC=ffff800004001234\n", "CPU#0\nPC=ffff8000040099f0\n"])
    moving = _hang_vm(tmp_path / "b", None, lambda: next(pcs))
    moving.save_hang_state("sweep drain")
    assert "EXECUTING" in (tmp_path / "b" / "hang-cpustate.txt").read_text()


def test_save_hang_state_names_a_kernel_fault_loop_at_a_user_pc(tmp_path, _no_resample_wait):
    # The third reading of "the PC never moved": a same-EL instruction abort
    # the handler keeps resolving as spurious, eret'ing straight back to the
    # same PC. Two samples cannot separate that from a dead clock — but the
    # arm64 register dump can: kernel EL (PSTATE M != 0) with a PC in the user
    # half. Five real captures said FROZEN before this existed.
    dump = "CPU#0\n PC=0000000000010000 X00=0\nPSTATE=0000000020000345 --C- EL1h\n"
    v = _hang_vm(tmp_path / "loop", None, lambda: dump)
    v.save_hang_state("boot-wait timeout")
    body = (tmp_path / "loop" / "hang-cpustate.txt").read_text()
    assert "FAULT-LOOP" in body and "CPU#0 pc=0x0000000000010000" in body
    # The plain readings must not be promoted: a kernel PC at EL1 is FROZEN...
    idle = "CPU#0\n PC=fffff80042192824 X00=0\nPSTATE=0000000020000345 --C- EL1h\n"
    v = _hang_vm(tmp_path / "idle", None, lambda: idle)
    v.save_hang_state("boot-wait timeout")
    assert "FAULT-LOOP" not in (tmp_path / "idle" / "hang-cpustate.txt").read_text()
    # ...and a user PC at EL0 is a user thread, which is fine.
    user = "CPU#0\n PC=0000000000010000 X00=0\nPSTATE=0000000060000000 -ZC- EL0t\n"
    v = _hang_vm(tmp_path / "user", None, lambda: user)
    v.save_hang_state("boot-wait timeout")
    assert "FAULT-LOOP" not in (tmp_path / "user" / "hang-cpustate.txt").read_text()


def test_capture_cpu_state_disassembles_at_every_pc():
    # A register dump says WHERE a vCPU is; only the code there says what it is
    # doing (wfi / b . / an unfetchable page). One `x /6i` per vCPU, at that
    # vCPU's own PC, after the register dump it was parsed from.
    v = vm_mod.QemuVM.__new__(vm_mod.QemuVM)
    seen = []

    class _Qmp:
        def hmp(self, cmd):
            seen.append(cmd)
            if cmd == "info registers -a":
                return "CPU#0\n PC=fffff80042192824\nCPU#1\n PC=0000000000010000\n"
            if cmd.startswith("x /6i"):
                return f"disasm for {cmd}"
            return "ok"

    v.qmp = _Qmp()  # type: ignore[assignment]
    out = v.capture_cpu_state()
    assert seen[-2:] == ["x /6i 0xfffff80042192824", "x /6i 0x0000000000010000"]
    assert "=== CPU#1 x /6i 0x0000000000010000 ===" in out


def test_capture_cpu_state_reads_the_memory_under_every_stack_pointer():
    # The decisive test for a wedge parked on a stack access: can the MONITOR
    # read the page, using the guest's own translation? Unreadable means the
    # page is gone and the guest is re-faulting on it forever; readable means
    # the fault is not a translation fault. Nothing inside the guest can answer
    # that, because the guest is what is stuck.
    v = vm_mod.QemuVM.__new__(vm_mod.QemuVM)
    seen = []

    class _Qmp:
        def hmp(self, cmd):
            seen.append(cmd)
            if cmd == "info registers -a":
                return " PC=fffff8004219280c  SP=ffff800004a26a00\nCPU#0 header above\n"
            return "ok"

    # One CPU, banner first so the slicer credits it.
    class _Qmp2(_Qmp):
        def hmp(self, cmd):
            seen.append(cmd)
            if cmd == "info registers -a":
                return "CPU#0\n PC=fffff8004219280c X00=0\n SP=ffff800004a26a00\n"
            return "ok"

    v.qmp = _Qmp2()  # type: ignore[assignment]
    out = v.capture_cpu_state()
    assert "x /23gx 0xffff800004a26a00" in seen
    assert "=== CPU#0 stack x /23gx 0xffff800004a26a00 ===" in out
    # The stack read must come AFTER the register dump it was parsed from.
    assert seen.index("info registers -a") < seen.index("x /23gx 0xffff800004a26a00")


def test_save_hang_state_survives_a_dead_monitor(tmp_path, _no_resample_wait):
    # A diagnostic that raises on an already-failing run destroys the failure it
    # was there to explain, so a dead QMP must degrade to a note in the file —
    # and must NOT report FROZEN, which two empty samples would otherwise imply.
    def _boom():
        raise RuntimeError("qmp gone")

    v = _hang_vm(tmp_path, None, _boom)
    v.save_hang_state("boot-wait timeout")
    body = (tmp_path / "hang-cpustate.txt").read_text()
    assert "capture failed" in body
    assert "UNKNOWN" in body and "FROZEN" not in body


def test_a_drained_out_payload_captures_the_hang_before_it_fails():
    # The gap this closes: a sweep that BOOTS and then wedges never enters
    # wait_for_boot's TimeoutError, so it had no capture at all — and by the time
    # the assert has failed and the fixture torn the VM down, the vCPU PCs are
    # unrecoverable. The capture has to happen here, before the assert.
    from tests.qemu import test_payload as payload_mod

    calls = []

    class _StubVm:
        _boot_output = "booted fine, then nothing\n"

        def serial_output(self, timeout=1.0):
            return ""

        def save_hang_state(self, phase):
            calls.append(phase)
            return ""

    spec = {"test": {"launch_marker": "[KMTEST-SWEEP] done", "launch_drain_secs": 2}}
    with pytest.raises(AssertionError):
        payload_mod.test_payload_launched(_StubVm(), "arm64", "kmtest", spec)  # type: ignore[arg-type]
    assert calls and "kmtest" in calls[0]


def test_a_crashed_payload_does_not_capture_a_hang():
    # A BUGCHECK already carries its own faulting PC on serial. Capturing here
    # would resample a dead guest and file it as if it were a silent wedge.
    from tests.qemu import test_payload as payload_mod

    calls = []

    class _StubVm:
        _boot_output = "*** BUGCHECK 0x1E\n"

        def serial_output(self, timeout=1.0):
            return ""

        def save_hang_state(self, phase):
            calls.append(phase)
            return ""

    spec = {"test": {"launch_marker": "[KMTEST-SWEEP] done", "launch_drain_secs": 2}}
    with pytest.raises(AssertionError):
        payload_mod.test_payload_launched(_StubVm(), "arm64", "kmtest", spec)  # type: ignore[arg-type]
    assert calls == []


def test_conformance_points_the_sweep_at_its_run_dir():
    # conformance shells out to pytest, so the run dir can only reach the VM
    # through the environment. If this wiring is dropped, hang captures silently
    # go back to the prunable pytest tmp_path and the run dir holds serial only.
    src = Path(conformance_mod.__file__).read_text()
    assert 'env["VIBEOS_ARTIFACT_DIR"] = str(run_dir)' in src


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


# --- command timing and shipping Cargo target coverage ------------------


@pytest.mark.parametrize("failure", [None, "timeout", "missing"])
def test_command_timings_survive_failures(monkeypatch, failure):
    from . import artifacts

    ticks = iter([10.0, 12.0, 15.5, 16.0])
    monkeypatch.setattr(artifacts.time, "monotonic", lambda: next(ticks))

    def execute(*args, **kwargs):
        if failure == "timeout":
            raise subprocess.TimeoutExpired(["fixture"], 1)
        if failure == "missing":
            raise FileNotFoundError("fixture")
        return subprocess.CompletedProcess(["fixture"], 0, "", "")

    monkeypatch.setattr(artifacts.subprocess, "run", execute)
    run = Run("build", "amd64", ts=1700000000)
    cp = run.run(["fixture"], stage="ninja", timeout=1)
    run.finalize()
    expected = {
        "stage": "ninja",
        "start_seconds": 2.0,
        "seconds": 3.5,
        "exit": {None: 0, "timeout": 124, "missing": 127}[failure],
    }
    summary = json.loads((run.dir / "summary.json").read_text())
    assert summary["command_timings"] == [expected]
    journal = json.loads(run.commands_log.read_text())
    assert all(journal[key] == value for key, value in expected.items())
    assert cp.returncode == expected["exit"]


def test_rust_source_closure_tracks_shared_dependencies(tmp_path):
    """Exercise the shipping CMake helper, not a Python imitation of it."""
    workspace = tmp_path / "workspace"
    for name, deps in (
        ("leaf", '[dependencies]\nshared = { path = "../shared" }'),
        ("shared", ""),
        ("unrelated", ""),
    ):
        crate = workspace / name
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text(f'[package]\nname = "{name}"\n{deps}\n')
        (crate / "src/lib.rs").write_text("")
        (crate / "build.rs").write_text("fn main() {}")
    helper = Path(__file__).resolve().parents[3] / "sdk/cmake/rust-crate-deps.cmake"
    output = tmp_path / "sources.txt"
    script = tmp_path / "fixture.cmake"
    script.write_text(
        "cmake_minimum_required(VERSION 3.16)\n"
        f'include("{helper}")\nrust_crate_sources(sources "{workspace}" leaf)\n'
        f'file(WRITE "{output}" "${{sources}}")\n'
    )
    subprocess.run(["cmake", "-P", str(script)], check=True, capture_output=True, text=True)
    sources = set(output.read_text().split(";"))
    assert sources == {
        str(workspace / crate / source)
        for crate in ("leaf", "shared")
        for source in ("src/lib.rs", "Cargo.toml", "build.rs")
    }


def test_former_cargo_pocs_are_shipping_cmake_targets():
    """The old off-path link probes duplicate modules built and smoked by the gate."""
    repo = Path(__file__).resolve().parents[3]
    for relative, module, crate in (
        ("ntoskrnl/CMakeLists.txt", "ntoskrnl", "ntoskrnl-cargo"),
        ("dll/ntdll/CMakeLists.txt", "ntdll", "ntdll-cargo"),
        ("dll/ntdll/nt_0600/CMakeLists.txt", "ntdll_vista", "ntdll_vista-cargo"),
    ):
        source = (repo / relative).read_text()
        assert f"add_cargo_module({module} {crate}" in source


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


def test_vibeos_root_escapes_harness_subagent_worktree(monkeypatch):
    monkeypatch.delenv("VIBEOS_ROOT", raising=False)
    # A subagent worktree at <lane>/.claude/worktrees/<name> must still resolve to the
    # FLEET parent, not to its own private dir -- taking the literal parent built a 10G
    # ccache+sccache pair under vibeos-fex/.claude/worktrees/ that nothing could hit,
    # and misplaced the integration worktree and the ledger alongside it.
    parent = Path("/home/u/os-playground")
    lane = parent / "vibeos-fex"
    assert cachecfg.vibeos_root(lane / ".claude" / "worktrees" / "fex-m4") == parent
    # nested subagents: cut at the FIRST .claude, not the last
    deep = lane / ".claude" / "worktrees" / "a" / ".claude" / "worktrees" / "b"
    assert cachecfg.vibeos_root(deep) == parent


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


def test_sccache_non_cacheable_is_not_a_miss():
    stats = cachecfg.parse_sccache_json(
        json.dumps(
            {
                "stats": {
                    "compile_requests": 7,
                    "requests_not_cacheable": 5,
                    "not_cached": {"crate-type": 5},
                    "cache_errors": 1,
                    "compilations": 2,
                    "cache_hits": 1,
                    "cache_misses": 1,
                }
            }
        )
    )
    assert stats["non_cacheable"] == 5
    assert stats["non_cacheable_reasons"] == {"crate-type": 5}
    assert stats["cache_errors"] == 1
    assert stats["compilations"] == 2
    assert stats["hit_rate"] == 0.5


def test_sccache_stats_use_compilers_canonical_context(tmp_path, monkeypatch):
    cache = cachecfg.CacheEnv(tmp_path, tmp_path / "cc", tmp_path / "sc", None, "sccache")
    run = Run("build", "amd64", ts=1700000000)
    calls = []

    def execute(cmd, **kwargs):
        calls.append((cmd, kwargs["cwd"]))
        return subprocess.CompletedProcess(cmd, 0, '{"stats": {}}', "")

    monkeypatch.setattr(run, "run", execute)

    def wrap(cmd):
        return ["canonical-wrapper", *cmd]

    env = build_mod._setup_cache(run, tmp_path, tmp_path / "build-amd64", cache, wrap=wrap)
    assert env is not None
    build_mod._collect_cache_stats(run, cache, env, repo=tmp_path, wrap=wrap)
    assert calls == [
        (["canonical-wrapper", "sccache", "--show-stats", "--stats-format=json"], tmp_path),
    ]
    assert "not per-build" in run.extra["cache_stats_scope"]


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
    second = _bare_vm("i386-uefi-net")
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


def _launch_vm(backend: str, tmp_path: Path):
    """A bare QemuVM carrying only what _launch_qemu touches."""
    v = vm.QemuVM.__new__(vm.QemuVM)
    v._backend = backend
    v.live_log_path = tmp_path / "serial.log"
    v.config = vm_mod.get_arch_config("amd64-uefi")
    v._proc = None
    v._early_stderr = ""
    v._gdb = False
    return v


def test_launch_qemu_retries_hostfwd_bind_race(monkeypatch, tmp_path):
    v = _launch_vm("windows-host", tmp_path)
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


def test_launch_qemu_healthy_first_try_no_retry(monkeypatch, tmp_path):
    v = _launch_vm("windows-host", tmp_path)
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


def test_launch_qemu_non_hostfwd_skips_poll(monkeypatch, tmp_path):
    # native (or any non-hostfwd) config: launch once, never poll for the bind race.
    v = _launch_vm("native", tmp_path)
    monkeypatch.setattr(v, "_build_native_cmd", lambda: ["qemu", "-M", "pc"])
    proc = _FakeProc()
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: proc)
    monkeypatch.setattr(
        v, "_early_exit_stderr", lambda *a, **k: pytest.fail("must not poll a non-hostfwd cmd")
    )
    cmd = v._launch_qemu()
    assert v._proc is proc
    assert cmd == ["qemu", "-M", "pc"]


def test_launch_qemu_non_bind_early_exit_is_not_retried(monkeypatch, tmp_path):
    # QEMU dies early for some OTHER reason → not the flake: keep the proc (the normal
    # start path surfaces it), stash stderr, do NOT retry.
    v = _launch_vm("windows-host", tmp_path)
    monkeypatch.setattr(v, "_build_windows_cmd", lambda: ["qemu", "hostfwd=tcp:0.0.0.0:1-:1"])
    proc = _FakeProc()
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: proc)
    monkeypatch.setattr(v, "_early_exit_stderr", lambda p, budget=2.0: "bad -drive: No such file")
    monkeypatch.setattr(v, "_discard_failed_launch", lambda p: pytest.fail("must not retry"))
    v._launch_qemu()
    assert v._proc is proc
    assert v._early_stderr == "bad -drive: No such file"


def test_launch_qemu_raises_after_exhausting_retries(monkeypatch, tmp_path):
    monkeypatch.setattr(vm, "HOSTFWD_LAUNCH_RETRIES", 2)
    v = _launch_vm("windows-host", tmp_path)
    monkeypatch.setattr(v, "_build_windows_cmd", lambda: ["qemu", "hostfwd=tcp:0.0.0.0:1-:1"])
    monkeypatch.setattr(vm.subprocess, "Popen", lambda cmd, **kw: _FakeProc())
    monkeypatch.setattr(v, "_early_exit_stderr", lambda p, budget=2.0: vm.HOSTFWD_BIND_ERROR)
    monkeypatch.setattr(v, "_discard_failed_launch", lambda p: None)
    with pytest.raises(RuntimeError, match="hostfwd-bind failed after 2"):
        v._launch_qemu()


def test_launch_qemu_retries_gdb_bind_race(monkeypatch, tmp_path):
    # A windows-host gdbstub binds a Windows-stack port with the same pick-then-bind
    # TOCTOU as a hostfwd → must be retried with a fresh port (no hostfwd present).
    v = _launch_vm("windows-host", tmp_path)
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


@pytest.mark.parametrize("smp", [None, "1", "4"])
def test_gdbengine_smp_media_symbols_and_cpu_count_agree(tmp_path, monkeypatch, smp):
    from types import SimpleNamespace

    if smp is None:
        monkeypatch.delenv("VIBEOS_QEMU_SMP", raising=False)
    else:
        monkeypatch.setenv("VIBEOS_QEMU_SMP", smp)
    monkeypatch.setattr(ge_mod, "PROJECT_DIR", tmp_path)
    monkeypatch.setattr(ge_mod.subprocess, "run", lambda *a, **kw: None)
    monkeypatch.setattr(be_mod, "stage_media_paths", lambda args, staging: args)
    monkeypatch.setattr(be_mod, "windows_free_tcp_port", lambda: 12345)
    monkeypatch.setattr(be_mod, "windows_qemu_bin", lambda *a: "qemu-system-aarch64.exe")
    monkeypatch.setattr(be_mod, "win_path", str)
    monkeypatch.setattr(be_mod, "gateway_ip", lambda: "127.0.0.1")
    selected = []

    def qemu_args(work):
        selected.append(vm_mod.smp_count())
        return []

    config = SimpleNamespace(qemu_bin="qemu-system-aarch64", qemu_args=qemu_args)
    cmd, _ = ge_mod._build_windows_cmd(
        "arm64", config, {"qemu_dir": "qemu"}, tmp_path, tmp_path, gdb_wait=True
    )
    assert selected == [int(smp) if smp else None]
    if smp is None:
        assert "-smp" not in cmd
        assert ge_mod._kernel_image("arm64") == tmp_path / "build-arm64/ntoskrnl/ntoskrnl.exe"
    else:
        assert cmd[cmd.index("-smp") + 1] == smp
        assert (
            ge_mod._kernel_image("arm64") == tmp_path / "build-arm64/ntoskrnl/ntkrnlmp/ntkrnlmp.exe"
        )


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


@pytest.mark.parametrize("copy_fails", [False, True])
def test_gdbengine_stop_preserves_serial_before_cleaning_staging(tmp_path, monkeypatch, copy_fails):
    monkeypatch.setattr(ge_mod, "RUNS_DIR", tmp_path / "runs")
    staging = tmp_path / "staging"
    staging.mkdir()
    serial = staging / "serial.log"
    serial.write_text("boot evidence\n")
    run = ge_mod.DebugRun(
        run_id="arm64-archive",
        arch="arm64",
        backend="native",
        gdb_host="127.0.0.1",
        gdb_port=1234,
        serial_log=str(serial),
        kernel_image="/k/ntkrnlmp.exe",
        staging=str(staging),
    )
    run.save()
    if copy_fails:

        def fail_copy(*args, **kwargs):
            raise OSError("disk full")

        monkeypatch.setattr(ge_mod.shutil, "copy2", fail_copy)
        with pytest.raises(OSError, match="disk full"):
            ge_mod.stop(run)
        assert serial.read_text() == "boot evidence\n"
    else:
        ge_mod.stop(run)
        assert not staging.exists()
        archived = Path(ge_mod.DebugRun.load(run.run_id).serial_log)
        assert archived.read_text() == "boot evidence\n"
        ge_mod.stop(run)  # repeated cleanup must preserve the archive too
        assert archived.read_text() == "boot evidence\n"


def test_gdbengine_load_rejects_foreign_manifest(tmp_path, monkeypatch):
    # qemu/runs/ is shared with the bash toolkit, whose manifest lacks our fields.
    monkeypatch.setattr(ge_mod, "RUNS_DIR", tmp_path)
    (tmp_path / "bash-run").mkdir()
    (tmp_path / "bash-run" / "manifest.json").write_text('{"run_id":"bash-run","qemu_pid":9}')
    with pytest.raises(ValueError, match="not a gdbengine run manifest"):
        ge_mod.DebugRun.load("bash-run")


@pytest.mark.parametrize(
    "result",
    [None, {"hit": False, "checks": 0}, {"hit": "no", "checks": 2}, {"hit": False, "checks": 2}],
)
def test_gdbengine_probe_rejects_silent_script_failure(tmp_path, monkeypatch, result):
    monkeypatch.setattr(ge_mod, "RUNS_DIR", tmp_path / "runs")
    run = ge_mod.DebugRun("probe", "arm64", "windows-host", "host", 1234, "serial", "kernel")
    run.save()
    script = tmp_path / "watch.py"
    script.write_text("# probe\n")

    def fake_gdb(*args, **kwargs):
        if result is not None:
            (run.dir / "probe-result.json").write_text(json.dumps(result))
        return subprocess.CompletedProcess([], 0)

    monkeypatch.setattr(ge_mod.subprocess, "run", fake_gdb)
    if result == {"hit": False, "checks": 2}:
        assert ge_mod.probe(run, script, timeout=1) == result
    else:
        with pytest.raises(RuntimeError):
            ge_mod.probe(run, script, timeout=1)


@pytest.mark.parametrize("outcome", ["hit", "error", "cancel"])
def test_gdbengine_probe_soak_stops_on_hit_or_error_and_cleans_up(tmp_path, monkeypatch, outcome):
    from types import SimpleNamespace

    monkeypatch.setattr(ge_mod, "RUNS_DIR", tmp_path / "runs")
    kernel = tmp_path / "kernel"
    kernel.write_bytes(b"frozen kernel")
    (tmp_path / "qemu").mkdir()
    (tmp_path / "qemu" / "boot.img").write_bytes(b"media")
    script = tmp_path / "watch.py"
    script.write_text("# probe\n")
    monkeypatch.setattr(ge_mod, "_kernel_image", lambda arch: kernel)
    monkeypatch.setattr(ge_mod, "_cfg", lambda repo: ("windows-host", {}))
    runs = []
    stopped = []

    def fake_start(*args, **kwargs):
        assert kwargs["gdb_wait"]
        run = ge_mod.DebugRun(str(len(runs)), "arm64", "windows-host", "host", 1234, "s", "k")
        run.save()
        runs.append(run)
        return run

    def fake_probe(*args, **kwargs):
        if outcome == "cancel":
            raise KeyboardInterrupt
        if outcome == "error":
            raise RuntimeError("no coverage")
        return {"hit": len(runs) == 2, "checks": 1}

    monkeypatch.setattr(ge_mod, "start", fake_start)
    monkeypatch.setattr(ge_mod, "probe", fake_probe)
    monkeypatch.setattr(ge_mod, "stop", lambda run: stopped.append(run.run_id))
    args = SimpleNamespace(arch="arm64", script=str(script), repeat=8, timeout=1)
    assert ge_mod.cmd_probe(args, tmp_path) == {"hit": 0, "error": 2, "cancel": 130}[outcome]
    assert stopped == (["0", "1"] if outcome == "hit" else ["0"])
    last = json.loads((runs[-1].dir / "probe-summary.json").read_text())
    assert last["verdict"] == {"hit": "HIT", "error": "PROBE_ERROR", "cancel": "CANCELLED"}[outcome]


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


def test_early_exit_stderr_reports_exit_vs_healthy(tmp_path):
    v = vm.QemuVM.__new__(vm.QemuVM)
    v._stderr_path = tmp_path / "stderr.log"
    (tmp_path / "stderr.log").write_text("Could not set up host forwarding rule 'tcp:0.0.0.0:1234'")

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
    out, ids = be_mod.rewrite_netdev_for_windows(args)
    assert ids == ["gf0"]
    assert "hostfwd=tcp:0.0.0.0:54321-:7777" in out[-1]
    assert "guestfwd=tcp:10.0.2.100:9999-chardev:gf0" in out[-1]
    assert "cmd:cat" not in out[-1]
    # no netdev with fwds -> no-op, no echo server needed (the native-config shape)
    plain = ["-M", "pc", "-accel", "tcg"]
    assert be_mod.rewrite_netdev_for_windows(plain) == (plain, [])


def test_every_guestfwd_gets_its_OWN_chardev():
    """One chardev is one byte stream, so two guestfwds sharing an id merge their
    traffic — three consumers on a shared :9999 is what wedged the merge queue for a
    day (vm.GUESTFWD_ECHO_PORTS). The rewrite must hand out one id per forward."""
    netdev = "user,id=net0,hostfwd=tcp:127.0.0.1:54321-:7777" + "".join(
        f",guestfwd=tcp:10.0.2.100:{port}-cmd:cat" for port in (9997, 9998, 9999)
    )
    out, ids = be_mod.rewrite_netdev_for_windows(["-netdev", netdev])
    assert ids == ["gf0", "gf1", "gf2"]
    assert len(set(ids)) == len(ids)
    for port, cid in zip((9997, 9998, 9999), ids, strict=True):
        assert f"guestfwd=tcp:10.0.2.100:{port}-chardev:{cid}" in out[-1]
    assert "cmd:cat" not in out[-1]


def test_the_three_echo_consumers_never_share_a_port():
    """The guest hardcodes these; a duplicate here is a silently stolen echo, not a
    build error. Kept as a set-size check so adding a consumer forces a new port."""
    from tests.qemu import vm as vm_mod

    ports = vm_mod.GUESTFWD_ECHO_PORTS
    assert len(set(ports.values())) == len(ports), f"duplicate guestfwd echo port: {ports}"
    # Each port must appear exactly once in the emitted netdev options.
    opts = vm_mod._guestfwd_echo_opts("10.0.2.100")
    for port in ports.values():
        assert opts.count(f":{port}-cmd:cat") == 1


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


@pytest.fixture(autouse=True)
def _isolate_host_policy(tmp_path_factory, monkeypatch):
    """No unit test may read THIS BOX's real build policy.

    POLICY_DIR is durable and host-global by design, so unlike the old /tmp lock tree
    it is always present -- which means a test that patches only HOSTLOCK_DIR silently
    asserts against whatever capacity the machine happens to be running. That is how
    test_ninja_jobs_caps_and_reserves broke when the trial went active. Autouse so the
    whole class is closed, not just the 49 sites that exist today.
    """
    monkeypatch.setattr(hl_mod, "POLICY_DIR", tmp_path_factory.mktemp("host-policy"))


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


def test_hostlock_fifo_respects_existing_permit_holders(tmp_path, monkeypatch):
    """Capacity is not availability: an active build owns half the pool."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    held = hl_mod.acquire_cpu_share("running", n=2, leases=4, timeout=1.0)
    assert held is not None
    ahead = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=999999, need=2)
    late = None
    try:
        # The only free pair is owed to the older waiter, even though both
        # waiters' requests fit within the pool's TOTAL capacity.
        late = hl_mod.acquire_cpu_share("late", n=2, leases=4, timeout=0.1)
        assert late is None
    finally:
        hl_mod.release_all(late)
        os.close(ahead)
        hl_mod.release_all(held)


def test_hostlock_fifo_admission_does_not_serialize_running_work(tmp_path, monkeypatch):
    """Once admitted, a holder's remaining capacity is available to the next build."""
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    first = hl_mod.acquire_cpu_share("first", n=1, leases=4, timeout=1.0)
    assert first is not None
    second = None
    try:
        second = hl_mod.acquire_cpu_share("second", n=2, leases=4, timeout=1.0)
        assert second is not None  # first still owns its permit
    finally:
        hl_mod.release_all(second)
        hl_mod.release_all(first)


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


def test_hostlock_shadow_priority_ages_without_enforcement(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    waiters = [
        {"pid": "lane", "job_class": "interactive", "waited_s": 50, "clock": "monotonic"},
        {"pid": "train", "job_class": "integration", "waited_s": 1, "clock": "monotonic"},
    ]
    proposal = hl_mod.admission_shadow(waiters)
    assert proposal["proposed"] == ["train", "lane"]
    assert proposal["changed"] and not proposal["enforced"]
    waiters[0]["waited_s"] = 122
    assert hl_mod.admission_shadow(waiters)["proposed"] == ["lane", "train"]
    waiters[0]["job_class"] = "background"
    assert hl_mod.admission_shadow(waiters)["proposed"] == ["lane", "train"]
    waiters[0]["clock"] = "legacy-wall"
    assert "proposed" not in hl_mod.admission_shadow(waiters)
    (tmp_path / "admission-shadow.disabled").touch()
    assert hl_mod.admission_shadow([])["mode"] == "disabled"


def test_hostlock_ticket_metadata_and_shadow_leave_fifo_intact(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOSTLOCK_FAIR", "1")
    monkeypatch.setenv("VIBEOS_JOB_CLASS", "interactive")
    first = hl_mod._ticket_take("cpuheavy", 2)
    assert first is not None
    monkeypatch.setenv("VIBEOS_JOB_CLASS", "integration")
    second = hl_mod._ticket_take("cpuheavy", 2)
    assert second is not None
    try:
        q = hl_mod.queue_status("cpuheavy")
        assert [w["job_class"] for w in q] == ["interactive", "integration"]
        assert all(w["clock"] == "monotonic" for w in q)
        monkeypatch.setattr(hl_mod.time, "time", lambda: -1000000)
        assert all(w["waited_s"] >= 0 for w in hl_mod.queue_status("cpuheavy"))
        assert hl_mod.admission_shadow(q)["changed"]
        assert hl_mod._may_attempt("cpuheavy", first[1], 2, 4)
        assert not hl_mod._may_attempt("cpuheavy", second[1], 2, 4)
        # Status must not reap or mutate live tickets, even with shadow reporting.
        before = [p.read_bytes() for _, p in (first, second)]
        hl_mod.status()
        assert before == [p.read_bytes() for _, p in (first, second)]
    finally:
        hl_mod._ticket_drop(first)
        hl_mod._ticket_drop(second)


@pytest.mark.parametrize(
    "raw", ["[]", "null", "{", '{"queued_monotonic": NaN}', '{"queued_monotonic": true}']
)
def test_hostlock_shadow_malformed_ticket_is_unknown(tmp_path, monkeypatch, raw):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    fd = _fake_ticket(tmp_path, "cpuheavy", ns=1, pid=111, need=2)
    try:
        os.write(fd, raw.encode())
        q = hl_mod.queue_status("cpuheavy")
        assert q[0]["clock"] == "legacy-wall"
        assert "proposed" not in hl_mod.admission_shadow(q)
    finally:
        os.close(fd)
    assert hl_mod.queue_status("cpuheavy") == []


def test_hostlock_acquisition_records_wait_and_class(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_JOB_CLASS", "integration")
    fds = hl_mod.acquire_cpu_share("build:amd64", n=2, leases=2, timeout=0)
    assert fds is not None
    try:
        holders = hl_mod.status()["cpuheavy"]
        assert all(h["holder"]["job_class"] == "integration" for h in holders)
        assert all(h["holder"]["wait_seconds"] >= 0 for h in holders)
        assert '"enforced": false' in capsys.readouterr().out
    finally:
        hl_mod.release_all(fds)


def test_hostlock_build_admission_timings(tmp_path, monkeypatch):
    from contextlib import contextmanager

    now = [0.0]
    monkeypatch.setattr(hl_mod.time, "monotonic", lambda: now[0])
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")

    @contextmanager
    def cpu(*args, **kw):
        now[0] += 7
        yield [1, 2]

    @contextmanager
    def directory(*args, **kw):
        now[0] += 3
        yield 3

    monkeypatch.setattr(hl_mod, "cpu_share", cpu)
    monkeypatch.setattr(hl_mod, "build_dir_lock", directory)
    with hl_mod.build_lock(tmp_path, "amd64") as timing:
        assert timing["cpu_wait_s"] == 0
        assert timing["builddir_wait_s"] == 3
        assert not timing["cpu_granted"] and timing["builddir_granted"]
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "0")
    with hl_mod.build_lock(tmp_path, "amd64") as timing:
        assert timing["scheduler"] == "host-jobs"
        assert timing["builddir_granted"]


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
        assert fd is not None  # correctness lock survives CPU throttling opt-out
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


def test_hostlock_build_coordinator_reserves_no_cpu(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    bd = tmp_path / "build-i386"
    bd.mkdir()
    with hl_mod.build_lock(bd, "i386"):
        assert not hl_mod.status()["cpu_leases_held"]
        assert hl_mod.acquire_build_dir(bd, timeout=0) is None
        boot = hl_mod.acquire_cpu_lease("boot:arm64", leases=1, timeout=0)
        assert boot is not None
        hl_mod.release(boot)


def test_hostlock_two_builds_coexist_third_queues(tmp_path, monkeypatch):
    # Explicit two-build budget: each build takes ceil(4/2)=2 permits, so two
    # builds run concurrently and a third can't get a share.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOST_BROKER", "1")
    monkeypatch.setenv("VIBEOS_MAX_CONCURRENT_BUILDS", "2")
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
    # timed-out queue → WAIT only. The budget is seconds, not milliseconds, because
    # WAIT is printed AFTER the first failed attempt and AFTER the deadline check:
    # a process descheduled past its own deadline before that print returns None
    # having announced nothing, and the assert below fires on a lane that did
    # nothing wrong. This test runs during a merge-queue drain -- three builds and
    # four boots deep -- so "surely 0.8s is enough" is exactly the assumption the
    # host is busiest disproving.
    assert hl_mod.acquire_cpu_lease("boot:b", leases=1, timeout=3.0) is None
    out = capsys.readouterr().out
    assert hl_mod.WAIT_ANNOUNCE in out and hl_mod.ACQUIRED_ANNOUNCE not in out
    assert hl_mod.looks_starved(out)

    # queued-then-granted → WAIT + ACQUIRED.
    #
    # Release the holder only once the waiter has ACTUALLY announced, instead of
    # racing it on a timer. The old form was `Timer(0.8, release, holder)` followed
    # by the acquire, which assumed the acquire would observe the lease as held
    # within 0.8s. Under drain load it did not: the timer fired first, the very
    # first attempt succeeded, and the captured output was a lone admission-shadow
    # line with no WAIT at all -- TOOLING_FAIL, and an innocent lane bounced out of
    # the merge queue for a full gate cycle (3 builds + 4 boots).
    #
    # Note WAIT is not printed until an attempt has already FAILED, so waiting for
    # the ticket to appear in queue-cpuheavy would not be enough either; the only
    # sound signal is the announcement itself. redirect_stdout is process-wide
    # rather than per-thread, which is fine here only because the main thread does
    # not print inside this window.
    buf = io.StringIO()
    got: dict[str, int | None] = {}

    def _queued_acquire() -> None:
        with contextlib.redirect_stdout(buf):
            got["fd"] = hl_mod.acquire_cpu_lease("boot:c", leases=1, timeout=30.0)

    t = threading.Thread(target=_queued_acquire, daemon=True)
    t.start()
    deadline = time.monotonic() + 20.0
    while hl_mod.WAIT_ANNOUNCE not in buf.getvalue():
        assert time.monotonic() < deadline, f"waiter never announced: {buf.getvalue()!r}"
        time.sleep(0.01)
    hl_mod.release(holder)
    t.join(30.0)
    assert not t.is_alive(), "queued acquire never returned after the holder released"
    fd = got["fd"]
    assert fd is not None
    out = buf.getvalue()
    capsys.readouterr()
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


def test_canon_ensure_toolchain_heals_archiver_materialised_dir(tmp_path, monkeypatch):
    """A restore can leave a real EMPTY dir where the symlink was; heal it.

    Rebuilding this box on 2026-09-14 from a 7-Zip extraction of the WSL image
    produced exactly that in all 8 worktrees — the archiver materialised each
    symlink as an empty directory. `readlink()` then raised EINVAL, so the one
    function whose job is provisioning the toolchain died on the state it most
    needed to repair, and every build would have failed pointing at an empty dir.
    """
    parent = tmp_path
    repo = parent / "vibeos"
    link = repo / ".build-tools" / "llvm-mingw"
    link.mkdir(parents=True)  # the archiver's artifact: a real, empty directory
    tc = parent / "llvm-mingw-20250114"
    tc.mkdir()
    monkeypatch.setenv("VIBEOS_ROOT", str(parent))
    assert canon_mod.ensure_toolchain(repo) is True
    assert link.is_symlink()
    assert link.resolve() == tc.resolve()


def test_canon_ensure_toolchain_keeps_a_populated_real_directory(tmp_path, monkeypatch):
    """Only an EMPTY dir is the archiver artifact. A populated one is somebody's
    real machine-local toolchain — report it usable, never replace it."""
    repo = tmp_path / "vibeos"
    link = repo / ".build-tools" / "llvm-mingw"
    (link / "bin").mkdir(parents=True)
    (link / "bin" / "clang").write_text("#!/bin/sh\n")
    monkeypatch.setenv("VIBEOS_ROOT", str(tmp_path))
    assert canon_mod.ensure_toolchain(repo) is True
    assert not link.is_symlink()
    assert (link / "bin" / "clang").exists()


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


def test_gate_retries_stop_at_the_first_green(tmp_path, monkeypatch):
    # Attempts run serially, so a lane that recovers on r1 must never launch r2.
    # Firing all attempts up front spends a whole extra boot on every recovery.
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["KNOWN_FLAKE", "OK", "OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    rc = gate_mod.gate(_gate_args(retries=2), tmp_path)
    assert rc == 0
    retry_calls = [k for k in fake.calls if k.startswith("amd64:") and "-r" in k]
    assert retry_calls == ["amd64:smoke-amd64-r1"], retry_calls


def test_gate_a_lanes_retries_never_run_concurrently(tmp_path, monkeypatch):
    # THE point of serial retries. NET_FLAKE is *defined* as "failed under
    # concurrent-smoke load" and HOST_STARVED as "never got a slot", so running a
    # lane's attempts against each other recreates the cause and falsifies
    # nothing — two amd64-uefi-net VMs in contention with only each other. This
    # cost a gate and then a land rejection on 2026-08-08.
    import threading

    live = {"now": 0, "max": 0}
    lock = threading.Lock()
    inner = _fake_smoke({"i386": ["OK"], "amd64": ["KNOWN_FLAKE"], "arm64": ["OK"]})

    def counting(repo, arch, root, extra):
        if arch == "amd64":
            with lock:
                live["now"] += 1
                live["max"] = max(live["max"], live["now"])
            time.sleep(0.05)
            with lock:
                live["now"] -= 1
        return inner(repo, arch, root, extra)

    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(gate_mod, "_smoke_once", counting)
    gate_mod.gate(_gate_args(retries=3), tmp_path)
    # Assert the retries HAPPENED first: the first pass alone also yields max==1,
    # so without this the concurrency assertion below passes vacuously.
    amd64_calls = [k for k in inner.calls if k.startswith("amd64:")]
    assert len(amd64_calls) == 4, amd64_calls  # 1 first pass + 3 serial retries
    assert live["max"] == 1, f"a lane ran {live['max']} attempts at once"


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


def _cohab_once_stub(ok: bool):
    """Stand-in for gate._cohab_once (which shells tools/verify_cohab_coverage.py)."""

    def stub(repo, arch, root):
        return {
            "arch": arch,
            "ok": ok,
            "seconds": 0.01,
            "salient": "" if ok else "math.rs: [i386] nothing defines `__adj_fdiv_m16i@4`",
            "exit": 0 if ok else 1,
        }

    return stub


def test_gate_cohab_gap_blocks_stamp(tmp_path, monkeypatch):
    """A cohab coverage gap fails the gate and blocks the push stamp.

    This is the stage that exists because every OTHER stage is structurally blind to it:
    `cargo check` never codegens, so `global_asm!` is never assembled and `#[export_name]`
    never becomes a symbol; the build links; the smoke boots a green desktop. Eight msvcrt
    stdcall stubs shipped as `___adj_fdiv_m16i@4@4` and the flip was a no-op that nothing
    in the pipeline could see. A green boot is not evidence that the Rust is the thing
    running.
    """
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    fake = _fake_smoke({"i386": ["OK"], "amd64": ["OK"], "arm64": ["OK"]})
    monkeypatch.setattr(gate_mod, "_smoke_once", fake)
    monkeypatch.setattr(
        gate_mod,
        "_build_once",
        lambda repo, run, arch, args: {"arch": arch, "ok": True, "exit": 0, "seconds": 0.01},
    )
    monkeypatch.setattr(gate_mod, "_lint_once", _lint_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_difftest_once", _difftest_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_cohab_once", _cohab_once_stub(ok=False))
    stamped: list = []
    monkeypatch.setattr(gate_mod, "_write_gate_stamp", lambda *a, **k: stamped.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=False, no_build=False), tmp_path)
    assert rc == 1, "a cohab coverage gap must fail the gate"
    assert not stamped, "no push stamp when a flipped module is not actually flipped"
    assert fake.calls, "smokes still run so the matrix stays informative"


def test_gate_cohab_needs_both_a_build_and_the_lint_opt_in(tmp_path, monkeypatch):
    """The stage judges artifacts THIS run produced, so it runs only when both opt-ins hold.

    Under --no-build it would read a stale carry-over archive as a red (the verifier refuses
    anything older than the tree) — a false rejection on the merge train is more expensive
    than the check is worth, and `gate --no-build` exists precisely to re-smoke without
    rebuilding.
    """
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(
        gate_mod, "_smoke_once", _fake_smoke({a: ["OK"] for a in ("i386", "amd64", "arm64")})
    )
    monkeypatch.setattr(gate_mod, "_lint_once", _lint_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(ok=True))
    monkeypatch.setattr(gate_mod, "_difftest_once", _difftest_once_stub(ok=True))
    monkeypatch.setattr(
        gate_mod,
        "_build_once",
        lambda repo, run, arch, args: {"arch": arch, "ok": True, "exit": 0, "seconds": 0.01},
    )
    ran: list = []
    monkeypatch.setattr(gate_mod, "_cohab_once", lambda *a, **k: ran.append(1))
    for kw in ({"no_lint": False, "no_build": True}, {"no_lint": True, "no_build": False}):
        assert gate_mod.gate(_gate_args(**kw), tmp_path) == 0
        assert not ran, f"cohab must not run with {kw}"


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


@pytest.mark.parametrize("selection", ["", ",", "typo", "toolA,typo"])
def test_partial_check_rejects_bad_selection(tmp_path, monkeypatch, selection):
    monkeypatch.setattr(lints, "lint_tool_names", lambda repo: {"toolA"})
    monkeypatch.setattr(lints, "run_source_lints", lambda *a, **k: pytest.fail("must not execute"))
    assert debugcmd_mod.check(Namespace(only=selection), tmp_path) == 2


def test_partial_check_cannot_rewrite_baselines(tmp_path, monkeypatch):
    monkeypatch.setattr(lints, "write_baselines", lambda *a, **k: pytest.fail("must not write"))
    assert debugcmd_mod.check(Namespace(only="toolA", update_baseline="__ALL__"), tmp_path) == 2


@pytest.mark.parametrize("fail", [False, True])
def test_partial_check_runs_only_selected_and_reports_scope(tmp_path, monkeypatch, fail):
    specs = [(name, [name], "stdout") for name in ("toolA", "toolB")]
    monkeypatch.setattr(lints, "source_lint_specs", lambda repo: specs)
    called = []

    def stage(run, repo, name, cmd, *, strict, findings_from, cp=None):
        # `cp` mirrors the real _lint_stage: run_source_lints executes the
        # scanners concurrently and hands the finished process to the classifier.
        called.append((name, strict))
        run.add(Stage(name, status="fail" if fail else "ok"))

    monkeypatch.setattr(lints, "_lint_stage", stage)
    monkeypatch.setattr(lints, "run_binary_lints", lambda *a, **k: pytest.fail("not selected"))
    monkeypatch.setattr(
        lints, "run_committed_drift_guard", lambda *a, **k: pytest.fail("not selected")
    )
    rc = debugcmd_mod.check(Namespace(only=" toolB,toolB ", strict=True, arch="amd64"), tmp_path)
    assert rc == int(fail)
    assert called == [("toolB", True)]
    summaries = list((tmp_path / "artifacts").glob("*-check-partial-amd64/summary.json"))
    assert len(summaries) == 1
    summary = json.loads(summaries[0].read_text())
    assert summary["command"] == "check-partial"
    assert summary["coverage"] == "partial"
    assert summary["selected_scanners"] == ["toolB"]
    assert summary["omitted_scanners"] == ["toolA"]
    assert summary["verdict"] == ("FAIL" if fail else "WARN")


def test_source_lints_run_concurrently_but_report_in_spec_order(tmp_path, monkeypatch):
    """Scanners overlap; the artifact does not reorder.

    Both halves matter. Concurrency is the whole point (the suite was 48% of a
    build, serial, on a 24-core host), but stage ORDER is what the baseline diff
    and every reader of summary.json depend on, so the two are asserted together
    -- a future change that parallelises harder must not quietly shuffle stages.
    """
    import threading
    import time

    specs = [(f"s{i}", ["sleep-stub", str(i)], "stdout") for i in range(6)]
    monkeypatch.setattr(lints, "source_lint_specs", lambda repo: specs)

    live = 0
    peak = 0
    lock = threading.Lock()

    def fake_run(self, cmd, cwd=None, env=None, stage=None, timeout=None):
        nonlocal live, peak
        with lock:
            live += 1
            peak = max(peak, live)
        time.sleep(0.05)
        with lock:
            live -= 1
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(Run, "run", fake_run)
    monkeypatch.setattr(lints, "_baseline", lambda repo, name: [])
    run = Run("check", "amd64")
    lints.run_source_lints(run, tmp_path, strict=True)

    assert peak > 1, "scanners did not overlap"
    assert [st.name for st in run.stages] == [n for n, _, _ in specs]


def test_lint_jobs_serial_when_pinned_to_one(tmp_path, monkeypatch):
    """VIBEOS_LINT_JOBS=1 is the escape hatch back to the old behaviour."""
    monkeypatch.setenv("VIBEOS_LINT_JOBS", "1")
    assert lints.lint_jobs() == 1
    monkeypatch.setenv("VIBEOS_LINT_JOBS", "not-a-number")
    with pytest.raises(ValueError):
        lints.lint_jobs()


def test_source_lint_api_rejects_unknown_selection(tmp_path, monkeypatch):
    monkeypatch.setattr(lints, "source_lint_specs", lambda repo: [("a", ["a"], "stdout")])
    monkeypatch.setattr(lints, "_lint_stage", lambda *a, **k: pytest.fail("must not execute"))
    with pytest.raises(ValueError):
        lints.run_source_lints(Run("check", "amd64"), tmp_path, strict=True, only={"typo"})


@pytest.mark.parametrize("scope", ["source", "arch", None])
@pytest.mark.parametrize("fail", [False, True])
def test_check_partitions_preserve_full_coverage(tmp_path, monkeypatch, scope, fail):
    seen = []

    def source(run, repo, *, strict):
        assert strict
        seen.append("source")
        run.add(Stage("source", status="fail" if fail else "ok"))

    def guard(name):
        def execute(run, repo, arch, **kwargs):
            assert arch == "amd64"
            if name == "binary":
                assert kwargs["strict"]
            seen.append(name)
            run.add(Stage(name, status="fail" if fail else "ok"))

        return execute

    monkeypatch.setattr(lints, "run_source_lints", source)
    monkeypatch.setattr(lints, "run_committed_drift_guard", guard("drift"))
    monkeypatch.setattr(lints, "run_binary_lints", guard("binary"))
    assert debugcmd_mod.check(Namespace(scope=scope, strict=True, arch="amd64"), tmp_path) == int(
        fail
    )
    assert (
        seen
        == {
            "source": ["source"],
            "arch": ["drift", "binary"],
            None: ["source", "drift", "binary"],
        }[scope]
    )
    command = f"check-{scope}" if scope else "check"
    summary = json.loads(
        next((tmp_path / "artifacts").glob(f"*-{command}-amd64/summary.json")).read_text()
    )
    assert summary["verdict"] == ("FAIL" if fail else "WARN" if scope else "OK")
    assert summary.get("coverage") == ("partial" if scope else None)


@pytest.mark.parametrize(
    "options",
    [
        {"scope": "bogus", "strict": True},
        {"scope": "source"},
        {"scope": "arch", "strict": True},
        {"scope": "source", "strict": True, "only": "a"},
        {"scope": "arch", "strict": True, "arch": "amd64", "update_baseline": "__ALL__"},
    ],
)
def test_check_partition_rejects_ambiguous_or_incomplete_request(tmp_path, monkeypatch, options):
    monkeypatch.setattr(lints, "run_source_lints", lambda *a, **k: pytest.fail("must not scan"))
    monkeypatch.setattr(lints, "write_baselines", lambda *a, **k: pytest.fail("must not write"))
    assert debugcmd_mod.check(Namespace(**options), tmp_path) == 2


@pytest.mark.parametrize("failed", [None, "source-early", "source-deferred", "i386"])
def test_gate_runs_source_once_after_build_matrix_and_requires_every_partition(
    tmp_path, monkeypatch, failed
):
    built = set()
    seen = []
    stamped = []
    import threading

    early_started = threading.Event()
    builds_done = threading.Event()

    def build(repo, run, arch, args):
        assert early_started.wait(5), "early source checks must not wait for builds"
        built.add(arch)
        if len(built) == 3:
            builds_done.set()
        return {"arch": arch, "ok": True, "exit": 0, "seconds": 0.01}

    def lint(repo, arch, root):
        if arch == "source-early":
            assert not built
            early_started.set()
            assert builds_done.wait(5), "builds must overlap the early partition"
        elif arch == "source-deferred":
            assert built == {"i386", "amd64", "arm64"}
        else:
            assert arch in built
        seen.append(arch)
        return _lint_once_stub(arch != failed)(repo, arch, root)

    monkeypatch.setattr(gate_mod, "_build_once", build)
    monkeypatch.setattr(gate_mod, "_lint_once", lint)
    monkeypatch.setattr(gate_mod, "_cohab_once", _cohab_once_stub(True))
    monkeypatch.setattr(gate_mod, "_tooling_once", _tooling_once_stub(True))

    def difftest(repo, root):
        assert {"i386", "amd64"} <= built, "host oracles require generated x86 inputs"
        return _difftest_once_stub(True)(repo, root)

    monkeypatch.setattr(gate_mod, "_difftest_once", difftest)
    monkeypatch.setattr(gate_mod, "_smoke_once", _spy_smoke_ok([]))
    monkeypatch.setattr(gate_mod, "_write_gate_stamp", lambda *a, **k: stamped.append(1))
    rc = gate_mod.gate(_gate_args(no_lint=False, no_build=False, no_net=True), tmp_path)
    assert sorted(seen) == ["amd64", "arm64", "i386", "source-deferred", "source-early"]
    assert bool(stamped) == (failed is None)
    assert rc == int(failed is not None)


@pytest.mark.parametrize("arch", ["source", "source-early", "source-deferred", "amd64"])
def test_gate_child_requests_explicit_strict_partition(tmp_path, monkeypatch, arch):
    calls = []

    def execute(cmd, **kwargs):
        calls.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(gate_mod.subprocess, "run", execute)
    if arch in ("source-early", "source-deferred"):
        monkeypatch.setattr(
            gate_mod,
            "_read_summary",
            lambda root: {
                "checked_source_checks": [
                    s[0] for s in lints.source_partition_specs(tmp_path, arch)
                ]
            },
        )
    assert gate_mod._lint_once(tmp_path, arch, tmp_path / "logs")["ok"]
    suffix = [arch] if arch.startswith("source") else ["arch", "--arch", arch]
    assert calls == [[str(tmp_path / "vibeos-cli"), "check", "--strict", "--scope", *suffix]]


def test_source_partition_inventory_is_disjoint_complete_and_conservative(tmp_path, monkeypatch):
    original = lints.source_lint_specs(tmp_path)
    early = lints.source_partition_specs(tmp_path, "source-early")
    deferred = lints.source_partition_specs(tmp_path, "source-deferred")
    assert set(s[0] for s in early).isdisjoint(s[0] for s in deferred)
    assert sorted(early + deferred) == sorted(original)
    assert {s[0] for s in early} == lints.EARLY_SOURCE_CHECKS
    for mutation in ([*original, original[0]], [s for s in original if s[0] != early[0][0]]):
        monkeypatch.setattr(lints, "source_lint_specs", lambda repo, specs=mutation: specs)
        with pytest.raises(ValueError, match="inventory"):
            lints.source_partition_specs(tmp_path, "source-early")
    monkeypatch.setattr(
        lints, "source_lint_specs", lambda repo: [*original, ("future", ["future"], "stdout")]
    )
    assert "future" in {s[0] for s in lints.source_partition_specs(tmp_path, "source-deferred")}


@pytest.mark.parametrize("scope", ["source-early", "source-deferred"])
@pytest.mark.parametrize("drop", [False, True])
def test_source_partition_checks_executed_inventory(tmp_path, monkeypatch, scope, drop):
    def execute(run, repo, name, cmd, **kwargs):
        if not drop:
            run.add(Stage(name, status="ok"))

    monkeypatch.setattr(lints, "_lint_stage", execute)
    assert debugcmd_mod.check(Namespace(scope=scope, strict=True), tmp_path) == int(drop)


@pytest.mark.parametrize("inventory", [None, [], ["wrong"], ["scan_octal_literals"]])
def test_gate_rejects_missing_or_partial_source_inventory(tmp_path, monkeypatch, inventory):
    monkeypatch.setattr(
        gate_mod.subprocess, "run", lambda *a, **k: subprocess.CompletedProcess([], 0, "", "")
    )
    monkeypatch.setattr(
        gate_mod, "_read_summary", lambda root: {"checked_source_checks": inventory}
    )
    assert not gate_mod._lint_once(tmp_path, "source-early", tmp_path / "logs")["ok"]


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
    assert "legacy/native(cpuheavy)" in body and "boot(windows-host)" in body


def test_ninja_jobs_caps_and_reserves(monkeypatch, tmp_path):
    # Point HOSTLOCK_DIR at an empty dir so `_ninja_jobs` sees NO host build policy.
    # Without this the test reads the real one and asserts against whatever capacity
    # the box happens to be running -- it passed only while no policy was installed,
    # then failed the moment the three-build trial went active (`-j 6`, not `-j 24`).
    # The VIBEOS_BUILD_CORE_RESERVE path below is the no-policy fallback, so that is
    # the state it has to be exercised in.
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
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


def _canonize_probe(monkeypatch, tmp_path, *, cache: str | None, canonical: bool):
    """Drive _maybe_canonize with cmake stubbed out, returning (cmds, build_dir).

    `cache=None` models the case that actually bites: a build dir this worktree
    has never configured. Both cmake options gating the kmtest corpus default
    OFF, so nothing but an explicit -D on the fresh configure can turn them on.
    """
    repo = tmp_path / "wt"
    build_dir = repo / "build-i386"
    build_dir.mkdir(parents=True, exist_ok=True)  # one test probes twice
    if cache is not None:
        (build_dir / "CMakeCache.txt").write_text(cache)

    (build_dir / "build.ninja").touch()
    cmds: list[list[str]] = []

    class _Cp:
        returncode, stdout, stderr = 0, "", ""

    run = build_mod.Run("build", "i386", ts=1700000000)
    monkeypatch.setattr(run, "run", lambda cmd, **kw: (cmds.append(cmd), _Cp())[1])
    monkeypatch.setattr(build_mod.canon, "is_canonical", lambda _bd: canonical)
    monkeypatch.setattr(build_mod.canon, "available", lambda: True)
    monkeypatch.setattr(build_mod.canon, "ensure_toolchain", lambda _r: True)
    monkeypatch.setattr(build_mod.canon, "wrap", lambda _r, cmd: cmd)
    monkeypatch.setattr(build_mod.canon, "carried_options", lambda _bd: [])
    return cmds, run, repo, build_dir


def test_incomplete_canonical_configure_is_recovered(monkeypatch, tmp_path):
    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache="", canonical=True)
    (bd / "build.ninja").unlink()

    def configure(cmd, **kwargs):
        cmds.append(cmd)
        (bd / "build.ninja").touch()
        return type("Result", (), {"returncode": 0})()

    monkeypatch.setattr(run, "run", configure)
    build_mod._maybe_canonize(run, repo, bd, "i386", {})
    assert cmds == [["cmake", "."]]
    assert (bd / "build.ninja").is_file()


def test_sticky_kmtests_rides_the_fresh_configure(monkeypatch, tmp_path):
    # The fresh-worktree hole: no cache => carried_options carries nothing, so the
    # opt-in has to be named on the configure itself or it is dropped on exactly
    # the one run that cannot detect it (2026-08-02 — a full media built with no
    # kmtest corpus, surfacing as `[KMTEST-SWEEP] launch FAILED` minutes later).
    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache=None, canonical=False)
    build_mod._maybe_canonize(run, repo, bd, "i386", {}, sticky_on=("ENABLE_ROSTESTS_KMTESTS",))
    assert any("-DENABLE_ROSTESTS_KMTESTS=ON" in c for c in cmds), cmds


def test_sticky_kmtests_reconfigures_a_dir_that_lacks_it(monkeypatch, tmp_path):
    cmds, run, repo, bd = _canonize_probe(
        monkeypatch, tmp_path, cache="ENABLE_ROSTESTS_KMTESTS:BOOL=OFF\n", canonical=True
    )
    build_mod._maybe_canonize(run, repo, bd, "i386", {}, sticky_on=("ENABLE_ROSTESTS_KMTESTS",))
    assert cmds == [["cmake", "-DENABLE_ROSTESTS_KMTESTS=ON", "."]], cmds


def test_sticky_kmtests_is_one_directional_unlike_instr_flags(monkeypatch, tmp_path):
    # Sticky means a later PLAIN build must not flip it back OFF — the opposite of
    # --lockdep, which is reconciled every build so a gate can't inherit it. An ON
    # cache with nothing requested must produce no reconfigure at all.
    on = "ENABLE_ROSTESTS_KMTESTS:BOOL=ON\n"
    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache=on, canonical=True)
    build_mod._maybe_canonize(run, repo, bd, "i386", {}, sticky_on=())
    assert cmds == [], "a plain build reconfigured a sticky flag it was not asked about"

    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache=on, canonical=True)
    build_mod._maybe_canonize(run, repo, bd, "i386", {}, sticky_on=("ENABLE_ROSTESTS_KMTESTS",))
    assert cmds == [], "already ON should be a no-op, not a redundant reconfigure"


def test_rostests_is_reconciled_off_unlike_kmtests(monkeypatch, tmp_path):
    # --rostests is the OPPOSITE discipline from --kmtests and the pair is the
    # point: ENABLE_ROSTESTS adds ~128 executables no default build wants, so a
    # plain build must deterministically turn it back OFF. Sticky would mean one
    # opt-in quietly bloats every later i386 build in the worktree.
    on = "ENABLE_ROSTESTS:BOOL=ON\n"
    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache=on, canonical=True)
    build_mod._maybe_canonize(run, repo, bd, "i386", {"ENABLE_ROSTESTS": False})
    assert cmds == [["cmake", "-DENABLE_ROSTESTS=OFF", "."]], cmds


def test_rostests_turns_on_when_asked(monkeypatch, tmp_path):
    off = "ENABLE_ROSTESTS:BOOL=OFF\n"
    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache=off, canonical=True)
    build_mod._maybe_canonize(run, repo, bd, "i386", {"ENABLE_ROSTESTS": True})
    assert cmds == [["cmake", "-DENABLE_ROSTESTS=ON", "."]], cmds


def test_rostests_already_on_and_asked_is_a_no_op(monkeypatch, tmp_path):
    on = "ENABLE_ROSTESTS:BOOL=ON\n"
    cmds, run, repo, bd = _canonize_probe(monkeypatch, tmp_path, cache=on, canonical=True)
    build_mod._maybe_canonize(run, repo, bd, "i386", {"ENABLE_ROSTESTS": True})
    assert cmds == [], "already ON and requested should not reconfigure"


def test_rostests_flag_is_wired_from_the_real_parser_to_the_cmake_option():
    # Guard the WIRING, not just the mechanism. build.py reads the arg through
    # getattr(args, "rostests", False), which silently yields False for a name
    # that does not exist -- so a rename on either side disables the flag with
    # no error anywhere. Parse with the REAL parser and assert both directions.
    from tests.qemu.cli.__main__ import build_parser

    args = build_parser().parse_args(["build", "--arch", "i386", "--rostests"])
    assert bool(getattr(args, "rostests", False)) is True
    plain = build_parser().parse_args(["build", "--arch", "i386"])
    assert bool(getattr(plain, "rostests", False)) is False

    src = (Path(__file__).parent / "build.py").read_text()
    assert '"ENABLE_ROSTESTS": bool(getattr(args, "rostests", False))' in src, (
        "build.py no longer maps the parsed --rostests arg onto ENABLE_ROSTESTS"
    )


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


def test_conformance_asks_the_build_for_the_kmtest_corpus(monkeypatch, tmp_path):
    """conformance must pass --kmtests, and must never opt out to the UP kernel.

    Both cmake options gating the corpus default OFF, so a build dir this worktree
    just created has no tests in it; the old preflight only refused when a cache
    existed and said OFF, and returned None — sailing through — when there was no
    cache at all. The arm64 half is the tail of the MP-media work: every media now
    carries both kernel/HAL pairs and the MP one is the shipped default, so a
    media re-roll buys nothing. The UP half is the live assertion now: `--up-kernel`
    must never appear, because conformance is one of the automated paths that
    exists to test what we SHIP, and UP is a dead branch.

    Nothing here may depend on the ambient checkout being provisioned: the merge
    queue gates in a fresh worktree that legitimately lacks the gitignored payload
    exes (see the note in test_worktree_provisioning_preflight).
    """
    from types import SimpleNamespace

    seen: list[list[str]] = []
    monkeypatch.setattr(conformance_mod, "_worktree_provisioning_gaps", lambda *_a: [])
    monkeypatch.setattr(conformance_mod, "_load_parser", lambda _r: object())
    monkeypatch.setattr(conformance_mod, "_run_dir", lambda _a: tmp_path)
    monkeypatch.setattr(
        conformance_mod.subprocess,
        "run",
        lambda cmd, **kw: (seen.append(cmd), SimpleNamespace(returncode=1))[1],
    )
    for arch in ("i386", "arm64"):
        seen.clear()
        args = SimpleNamespace(
            arch=arch, smp=2, no_build=False, write_baseline=False, only=None, timeout=None
        )
        assert conformance_mod._conformance_once(args, tmp_path) == 1  # bailed on our stub
        assert seen and "--kmtests" in seen[0], seen
        assert "--up-kernel" not in seen[0], f"{arch}: conformance must not test the UP kernel"


def test_worktree_provisioning_preflight(tmp_path):
    gaps = conformance_mod._worktree_provisioning_gaps(tmp_path, "arm64")
    blob = "\n".join(gaps)
    # An empty tree lacks all three, and each finding must carry its own fix.
    assert "llvm-mingw" in blob and "kmtest-sweep-arm64.exe" in blob
    assert "QEMU_EFI.fd" in blob and "efivars.fd" in blob
    assert blob.count("fix:") == len(gaps)
    # Payload binaries are per-commit generated artifacts: BUILD them. Telling
    # anyone to copy them from a sibling worktree is the fleet-isolation trap.
    assert "build.sh arm64" in blob
    assert "cp " not in blob.split("QEMU_EFI.fd")[0]
    # A PROVISIONED tree must stay silent, or the preflight would refuse every
    # ordinary run. Build that tree here rather than asserting against the
    # ambient checkout: the merge queue gates in a FRESH worktree, which
    # legitimately lacks the gitignored payload exes, so pointing this at
    # _conf_repo_root() made the assertion a property of whoever ran it — it
    # passed locally and rejected the lane in the integration tree.
    ok = tmp_path / "provisioned"
    (ok / ".build-tools" / "llvm-mingw").mkdir(parents=True)
    (ok / "tools" / "payloads" / conformance_mod.SUITE).mkdir(parents=True)
    (
        ok
        / "tools"
        / "payloads"
        / conformance_mod.SUITE
        / f"{conformance_mod.SUITE}-sweep-arm64.exe"
    ).write_bytes(b"")
    (ok / "qemu").mkdir(parents=True)
    (ok / "qemu" / "QEMU_EFI.fd").write_bytes(b"")
    (ok / "qemu" / "efivars.fd").write_bytes(b"")
    assert conformance_mod._worktree_provisioning_gaps(ok, "arm64") == []


def _drive_soak(monkeypatch, launched_per_round, repeat=20):
    """Run conformance()'s repeat loop with a stubbed sweep, returning how many
    rounds it actually ran. `launched_per_round` supplies each round's launched
    count (last value repeats)."""
    calls = {"n": 0}
    monkeypatch.setattr(conformance_mod, "_ROUNDS", [], raising=False)

    def fake_once(args, repo):
        i = calls["n"]
        calls["n"] += 1
        n = launched_per_round[min(i, len(launched_per_round) - 1)]
        conformance_mod._ROUNDS.append({"shape": "x", "launched": n})
        return 0 if n else 1

    monkeypatch.setattr(conformance_mod, "_conformance_once", fake_once)

    class A:
        arch = "arm64"
        no_build = False
        repeat = 0

    a = A()
    a.repeat = repeat
    conformance_mod.conformance(a, Path("/nonexistent"))
    return calls["n"]


def test_soak_aborts_when_the_media_launches_NOTHING(monkeypatch):
    # An unprovisioned worktree banked 38 launched=0 rounds into the SHARED
    # ledger in about a minute. Every further round is noise in someone else's
    # denominator, so the loop must stop rather than run its full repeat count.
    ran = _drive_soak(monkeypatch, [0], repeat=20)
    assert ran == 3, f"expected abort after 3 consecutive no-launch rounds, ran {ran}"


def test_soak_does_NOT_abort_on_isolated_no_launch_rounds(monkeypatch):
    # launched=0 is also a GENUINE face — the smp corpus records 13 of them — so
    # a lone one must still be recorded and the soak must continue. Only a
    # consecutive run means the media itself is broken.
    ran = _drive_soak(monkeypatch, [0, 91, 0, 91, 0, 91, 91, 91], repeat=8)
    assert ran == 8, f"isolated no-launch rounds must not end the soak, ran {ran}"


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


def test_lint_baseline_ignores_verify_signatures_coverage_tallies():
    """verify_signatures prints two header tallies and a '── coverage ──' block of counters.
    They were baselined as if they were findings, so adding ONE #[no_mangle] fn moved
    'exported Rust fns', 'exported fns checked' and 'no C proto (internal)' by one each and
    the merge queue rejected the batch as LINT_FAIL "3 new" (2026-07-31) — for a commit whose
    only sin was exporting a new function, which is what a translation lane does all day."""

    def keys(text: str) -> set[str]:
        return {lints._finding_key(ln) for ln in lints._finding_lines(text)}

    before = """\
  C prototypes: 5034 FunctionDecls, 4536 typedefs
  exported Rust fns: 3359 (deduped twins)

── coverage ──
  exported fns checked : 3359
  matched to a C proto : 2238
  no C proto (internal): 1121
  signature OK         : 2229
  mismatches           : 9  (3 shown, 6 LOW-suppressed)
"""
    after = before.replace("3359", "3360").replace("1121", "1122")
    assert lints._new_findings(keys(after), keys(before)) == []
    # Every COUNT-BEARING line must be filtered, not merely the three that moved — the
    # others (matched to a C proto, signature OK, mismatches) drift the moment anyone
    # fixes or adds a signature, and would bounce that lane instead. The static
    # '── coverage ──' banner may stay: it carries no number, so it never drifts.
    assert keys(before) == {"── coverage ──"}


def test_lint_baseline_still_flags_a_real_verify_signatures_mismatch():
    """The other half: the tally filter must not swallow the per-site blocks underneath it.
    A filter broad enough to catch any '<label>: <number>' would have — which is why
    _SUMMARY_LINE enumerates verify_signatures' labels instead of generalising."""
    finding = """\
[HIGH] KiThreadStartup  (sdk/lib/rust/x/thrdini.rs:98)
    rust: fn KiThreadStartup() -> ()
    C   : void KiThreadStartup(PKSYSTEM_ROUTINE, PKSTART_ROUTINE, PVOID, BOOLEAN, KTRAP_FRAME)
      - ARITY: rust 0 params vs C 5
      - param#2 u32 vs ULONG_PTR(unsigned long long): width 4 vs 8 [MEDIUM]
"""
    kept = {lints._finding_key(ln) for ln in lints._finding_lines(finding)}
    assert len(kept) == 5, kept
    assert lints._new_findings(kept, set()) != []


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


def test_queue_is_fifo_even_when_the_wall_clock_steps_BACKWARDS(monkeypatch, queue_repo):
    """seq's numeric half is CLOCK_REALTIME, which steps. Observed live on WSL2 under
    parallel load: a lane got a timestamp 2.59 SECONDS earlier than one already queued,
    so it sorted first and jumped the queue. This pins the fix deterministically —
    test_queue_is_fifo_by_arrival only caught it about 1 run in 80, which is a flake
    report, not a control."""
    repo, _ = queue_repo
    real = q_mod.time.time_ns()
    _commit(repo, "a.txt")
    monkeypatch.setattr(q_mod.time, "time_ns", lambda: real)
    first = q_mod.submit(repo, lane="lane-a")
    # The clock jumps 2.59s into the past, exactly as the live failure did.
    _commit(repo, "b.txt")
    monkeypatch.setattr(q_mod.time, "time_ns", lambda: real - 2_593_682_747)
    second = q_mod.submit(repo, lane="lane-b")

    assert second.seq > first.seq, "a later arrival must never sort before an earlier one"
    assert [s.seq for s in q_mod.pending(repo)] == [first.seq, second.seq]


def test_queue_seq_returns_to_real_time_once_the_queue_drains(monkeypatch, queue_repo):
    """The monotonic bump scans PENDING only, so a clock excursion cannot pin seqs
    forever — drain the queue and the next submission is back on the real clock. That
    self-healing is the reason results are deliberately not scanned."""
    repo, _ = queue_repo
    real = q_mod.time.time_ns()
    _commit(repo, "a.txt")
    # A wild forward jump (a decade) while one submission is queued.
    monkeypatch.setattr(q_mod.time, "time_ns", lambda: real + 10 * 365 * 86_400 * 10**9)
    stuck = q_mod.submit(repo, lane="lane-a")
    q_mod.resolve(repo, stuck, outcome="landed", reason="done")
    assert q_mod.pending(repo) == []

    monkeypatch.setattr(q_mod.time, "time_ns", lambda: real)
    _commit(repo, "b.txt")
    after = q_mod.submit(repo, lane="lane-b")
    assert int(after.seq.split("-", 1)[0]) == real


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
        return q_mod.GateVerdict(
            ok, "" if ok else "BUGCHECK", "/tmp/art", "smoke:amd64: BUGCHECK 0x50"
        )

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


def _stamp_scenario(repo, monkeypatch, *, lanes=("a", "b"), dirt="", stages=None, locks=()):
    """One submitted batch, with the integration tree's dirt and the gate's stamp verdict
    dialled in. Returns (submissions, gates, pushes) — `gates`/`pushes` record what the drain
    actually spent, which is the point of every assertion below."""
    subs = []
    for lane in lanes:
        subprocess.run(
            ["git", "checkout", "-q", "-b", f"s{lane}", "master"],
            cwd=repo,
            check=True,
            capture_output=True,
        )
        _commit(repo, f"s{lane}.txt")
        subs.append(q_mod.submit(repo, lane=f"lane-{lane}"))
    gates: list[int] = []
    pushes: list[tuple] = []

    # Delegate anything not explicitly faked: `pending()`/`results()` locate the queue through
    # `rev-parse --git-common-dir`, so a blanket `lambda: ""` would make every "who is still
    # queued?" assertion below pass vacuously against an empty directory.
    real_git = q_mod.git

    def fake_git(_repo, *args, **kw):
        if args[:1] == ("status",):
            return dirt
        if args[:1] == ("push",):
            pushes.append(args)
            return ""
        if args[0] in ("commit", "add", "fetch", "reset", "clean", "merge"):
            return ""
        if args[:2] == ("rev-parse", "HEAD"):
            return "n" * 40
        return real_git(_repo, *args, **kw)

    monkeypatch.setattr(q_mod, "git", fake_git)
    monkeypatch.setattr(q_mod, "git_ok", lambda *a, **k: True)
    monkeypatch.setattr(q_mod, "settle_cargo_locks", lambda wt, log: list(locks))
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="f" * 40, merged=list(b)),
    )
    monkeypatch.setattr(
        q_mod,
        "gate_batch",
        lambda w, log, retries=2: (
            gates.append(1),
            q_mod.GateVerdict(True, "", "/tmp/art", "", stages or {}),
        )[1],
    )
    return subs, gates, pushes


def test_queue_dirty_integration_tree_halts_before_paying_for_a_gate(queue_repo, monkeypatch):
    """The wedge this replaces: three conductors each burned a full COLD gate (3 builds +
    4 boots) and then failed identically at the push, because a dirty tree mints no stamp and
    re-gating can never fix it. The dirt is knowable in ~0.2s, BEFORE the gate."""
    repo, _ = queue_repo
    subs, gates, pushes = _stamp_scenario(repo, monkeypatch, dirt=" M sdk/lib/rust/Cargo.lock\n")
    with pytest.raises(q_mod.Inconclusive) as e:
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert "Cargo.lock" in str(e.value), "the error must NAME the path, not just say 'dirty'"
    assert gates == [], "a 30-minute gate must not be spent on a tree that cannot be stamped"
    assert pushes == []
    assert len(q_mod.pending(repo)) == len(subs), "nobody is at fault — all stay QUEUED"
    assert q_mod.results(repo, limit=50) == [], "and nobody is rejected"


def test_queue_clean_tree_still_gates_and_lands(queue_repo, monkeypatch):
    """The must-NOT-fire half: the preflight is a guard, not a new way to refuse work."""
    repo, _ = queue_repo
    subs, gates, pushes = _stamp_scenario(repo, monkeypatch, stages={"stamp": "ok"})
    q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert len(gates) == 1 and len(pushes) == 1
    assert all(r["outcome"] == "landed" for r in q_mod.results(repo, limit=50))


def test_queue_settles_a_build_rewritten_lock_into_the_tip_it_gates(queue_repo, monkeypatch):
    """Settling BEFORE the gate is what makes it safe: the tree that is gated is then the tree
    that is pushed, so the stamp never certifies something other than what booted."""
    repo, _ = queue_repo
    subs, gates, pushes = _stamp_scenario(
        repo, monkeypatch, stages={"stamp": "ok"}, locks=["sdk/lib/rust/Cargo.lock"]
    )
    q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert len(gates) == 1, "settling is a preflight, not a reason to re-gate"
    assert pushes and pushes[0][2].startswith("n" * 40), (
        "the push must carry the SETTLED tip, not the pre-settle one — otherwise the "
        "regenerated lock is gated and then thrown away"
    )
    assert all(r["outcome"] == "landed" for r in q_mod.results(repo, limit=50))


def test_queue_green_gate_that_minted_no_stamp_halts_instead_of_pushing(queue_repo, monkeypatch):
    """The backstop, for a cause the preflight did not anticipate. Without it the only signal
    is one `warn` on the last line of an all-green summary, and the failure resurfaces 30
    minutes later as a bare pre-push rejection the conductor dies on."""
    repo, _ = queue_repo
    subs, _gates, pushes = _stamp_scenario(repo, monkeypatch, stages={"stamp": "warn"})
    with pytest.raises(q_mod.Inconclusive) as e:
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert "stamp" in str(e.value)
    assert pushes == [], "a push that the hook will refuse must not be attempted"
    assert len(q_mod.pending(repo)) == len(subs)
    assert q_mod.results(repo, limit=50) == [], "a missing stamp is nobody's fault to be judged"


def test_queue_unparsed_gate_summary_does_not_block_the_push(queue_repo, monkeypatch):
    """REJECT ONLY ON EVIDENCE, applied to the backstop itself: an empty `stages` map means
    the summary could not be read, which is not proof that no stamp was minted."""
    repo, _ = queue_repo
    subs, _gates, pushes = _stamp_scenario(repo, monkeypatch, stages={})
    q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert len(pushes) == 1
    assert all(r["outcome"] == "landed" for r in q_mod.results(repo, limit=50))


def test_queue_refused_push_halts_the_train_instead_of_escaping(queue_repo, monkeypatch):
    """A push that cannot happen must leave the submissions QUEUED and halt like every other
    no-verdict, not escape as an unhandled error that resolves nobody."""
    repo, _ = queue_repo
    subs, _gates, _pushes = _stamp_scenario(repo, monkeypatch, stages={"stamp": "ok"})
    real_git = q_mod.git

    def refusing_git(_repo, *args, **kw):
        if args[:1] == ("push",):
            raise q_mod.QueueError("git push failed (1): pre-push: no fresh gate stamp")
        return real_git(_repo, *args, **kw)

    monkeypatch.setattr(q_mod, "git", refusing_git)
    with pytest.raises(q_mod.Inconclusive) as e:
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert "no fresh gate stamp" in str(e.value), "the git error must survive into the message"
    assert len(q_mod.pending(repo)) == len(subs)


def test_dirty_paths_keeps_the_first_path_intact(monkeypatch):
    """`git()` strips its whole stdout, so the first porcelain line arrives WITHOUT the leading
    space the rest keep. Slicing a fixed `[3:]` therefore eats a character of exactly one
    path — and a guard that names `ests/qemu/...` is a guard nobody can act on."""
    monkeypatch.setattr(
        q_mod,
        "git",
        lambda r, *a, **k: (
            "M  tests/qemu/cli/queue.py\n M sdk/lib/rust/Cargo.lock\nR  old/name.rs -> new/name.rs"
        ),
    )
    assert q_mod.dirty_paths(Path("/nowhere")) == [
        "tests/qemu/cli/queue.py",
        "sdk/lib/rust/Cargo.lock",
        "old/name.rs -> new/name.rs",
    ]


def test_settle_cargo_locks_reports_a_cargo_failure_rather_than_reading_as_clean(monkeypatch):
    """A preflight that silently skips is worse than none: it reads as 'checked, and clean'."""
    monkeypatch.setattr(q_mod, "git", lambda r, *a, **k: "sdk/lib/rust/Cargo.lock\n")
    monkeypatch.setattr(
        q_mod.subprocess,
        "run",
        lambda *a, **k: subprocess.CompletedProcess(a, 101, "", "error: no such subcommand"),
    )
    logs: list[str] = []
    assert q_mod.settle_cargo_locks(Path("/nowhere"), logs.append) == []
    assert any("could not resolve" in m for m in logs), logs


@pytest.mark.parametrize(
    ("exc", "want_class", "want_detail"),
    [
        (q_mod.BaseRed("fltmgr-rust fails to compile for i386"), "BASE_RED", "mainline itself"),
        (q_mod.Inconclusive("dirty: sdk/lib/rust/Cargo.lock"), "NO_VERDICT", "NO VERDICT"),
    ],
)
def test_conduct_does_not_report_every_no_verdict_as_a_red_mainline(
    queue_repo, tmp_path, monkeypatch, exc, want_class, want_detail
):
    """BaseRed is one of several ways a train renders no verdict. Reporting them all as
    "mainline itself is red" sends the lane hunting a mainline break that does not exist —
    the misattribution BaseCheck exists to prevent, committed by the reporter instead."""
    repo, _ = queue_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")

    def boom(*a, **k):
        raise exc

    monkeypatch.setattr(q_mod, "try_become_conductor", boom)
    run = Run("land", "queue")
    conducted, note = land_mod._conduct(repo, run, _Args(foreground=True, drain=True))
    assert conducted
    assert note == ("base-red" if want_class == "BASE_RED" else "no-verdict")
    assert run.classification == want_class
    stage = next(s for s in run.stages if s.name == "conduct")
    assert want_detail in stage.detail, stage.detail
    assert str(exc) in " ".join(stage.salient), "the reason must survive into the report"


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
        q_mod,
        "gate_batch",
        lambda w, log, retries=2: (gates.append(1), q_mod.GateVerdict(True, "", "", ""))[1],
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


# --- land: the check-tooling preflight -------------------------------------
#
# The conductor runs check-tooling as part of the batched gate, so a lint failure there
# bounces the WHOLE train for 3 builds + 4 boots of fleet load. On 2026-07-26 that cost
# was paid for one line 101 characters long. These cover the preflight that moves the
# identical verdict into the lane, where it costs ~40s.


def _fake_cli(repo: Path, *, rc: int, out: str = "") -> None:
    """Plant a stand-in `vibeos-cli` so the preflight runs its REAL subprocess path."""
    p = repo / "vibeos-cli"
    p.write_text(f"#!/bin/sh\ncat <<'XEOF'\n{out}\nXEOF\nexit {rc}\n")
    p.chmod(0o755)


def test_lint_preflight_blocks_a_red_check_tooling(tiny_repo: Path, tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    _fake_cli(tiny_repo, rc=1, out="  ✗ ruff:tools/*.py\n\nFAIL — 1/15 checks failed:")
    with pytest.raises(land_mod.LandAbort) as ei:
        land_mod._lint_preflight(tiny_repo, Run("land", "t"), _Args())
    # the failing check must be NAMED — "exit 1" alone sends you back to re-run it by hand
    assert "ruff:tools/*.py" in str(ei.value)


def test_lint_preflight_KEEPS_the_output_it_captured(tiny_repo: Path, tmp_path, monkeypatch):
    """The child writes no artifact dir, so this capture is the only copy of the failing
    test's own output — check-tooling prints it AFTER the summary line. A preflight that
    reports the ✗ and drops the rest leaves a lane nothing to debug when the failure does
    not reproduce, which is exactly what happened on 2026-09-06."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    detail = "=== pytest:tools/test_x.py ===\nE   assert 1 == 2"
    _fake_cli(tiny_repo, rc=1, out=f"  ✗ pytest:tools/test_x.py\n\nFAIL — 1/28:\n{detail}")
    run = Run("land", "t")
    with pytest.raises(land_mod.LandAbort) as ei:
        land_mod._lint_preflight(tiny_repo, run, _Args())
    log = run.dir / "preflight-check-tooling.log"
    assert log.exists(), "the captured output must survive the abort"
    assert "assert 1 == 2" in log.read_text()
    assert str(log) in str(ei.value), "and the abort must say where it is"


def test_lint_preflight_passes_a_green_check_tooling(tiny_repo: Path, tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    _fake_cli(tiny_repo, rc=0, out="OK — all 15 tooling checks passed.")
    run = Run("land", "t")
    land_mod._lint_preflight(tiny_repo, run, _Args())  # must not raise
    assert any(s.name == "lint" and s.status == "ok" for s in run.stages)


def test_lint_preflight_fails_OPEN_when_it_cannot_run(tiny_repo: Path, tmp_path, monkeypatch):
    """A broken preflight must never wedge the fleet's ability to land. No vibeos-cli here
    at all, so the subprocess raises — that has to warn and continue, NOT abort. Only a
    check that actually ran and returned nonzero may block."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    assert not (tiny_repo / "vibeos-cli").exists()
    run = Run("land", "t")
    land_mod._lint_preflight(tiny_repo, run, _Args())  # must not raise
    assert any(s.name == "lint" and s.status == "warn" for s in run.stages)


def test_lint_preflight_honours_skip_lint(tiny_repo: Path, tmp_path, monkeypatch):
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    _fake_cli(tiny_repo, rc=1, out="✗ everything")
    run = Run("land", "t")
    land_mod._lint_preflight(tiny_repo, run, _Args(skip_lint=True))  # must not raise
    assert any(s.name == "lint" and s.status == "skip" for s in run.stages)


def test_red_lint_keeps_the_lane_OUT_of_the_queue(lane_repo, tmp_path, monkeypatch):
    """The point of the whole thing: a red lane must not reach the batch at all. Checking
    that _lint_preflight raises is not enough — what matters is that nothing was queued,
    because a submission is what costs the fleet a gate."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    monkeypatch.setattr(
        land_mod, "_spawn_detached", lambda *a: pytest.fail("conducted despite a red lint")
    )
    _fake_cli(lane, rc=1, out="  ✗ pyright:tests/qemu/cli")
    subprocess.run(["git", "add", "-A"], cwd=lane, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "cli"], cwd=lane, check=True, capture_output=True)
    assert land_mod.land(_Args(direct=False), lane) != 0, "a red lint must fail the land"
    assert not q_mod.pending(lane), "a lane that fails its own lint reached the queue"


def test_green_lint_still_reaches_the_queue(lane_repo, tmp_path, monkeypatch):
    """The negative control for the test above — proves it is the LINT VERDICT gating the
    submission and not the fake CLI merely existing."""
    lane, _ = lane_repo
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path / "locks")
    monkeypatch.setattr(land_mod, "_spawn_detached", lambda argv, cwd, log: None)
    _fake_cli(lane, rc=0, out="OK")
    subprocess.run(["git", "add", "-A"], cwd=lane, check=True, capture_output=True)
    subprocess.run(["git", "commit", "-qm", "cli"], cwd=lane, check=True, capture_output=True)
    assert land_mod.land(_Args(direct=False), lane) == 0
    assert q_mod.pending(lane), "a green lane must still be queued"


def test_a_killed_submit_is_REPORTED_not_silently_absent(lane_repo, tmp_path, monkeypatch):
    """A `land` killed mid-preflight leaves the lane unregistered with no error.

    That is not hypothetical: the host filled its disk and the harness killed five background
    commands in one session, reporting each as low memory. One was a `land`. `land-status`
    simply did not list the lane, which is indistinguishable from "the conductor has not
    reached me yet" — so the agent waited on a queue position it never had. Asking everyone to
    remember to double-check is the rule-instead-of-mechanism the repo forbids; leave a marker
    and let the tool say it.
    """
    lane, _ = lane_repo
    head = q_mod.git(lane, "rev-parse", "HEAD")
    attempt = q_mod.begin_attempt(lane, q_mod.lane_id(lane), head)
    assert attempt.is_file()
    # Nothing is queued and the marker's process is alive (it is us), so this is an IN-FLIGHT
    # submit, not an abandoned one — it must stay quiet, or it cries wolf at every land.
    assert q_mod.abandoned_attempts(lane) == [], "a live submit must not report as abandoned"
    # Now the kill: rewrite the marker with a pid that cannot be running.
    rec = json.loads(attempt.read_text())
    rec["pid"] = 2**22  # above every /proc/sys/kernel/pid_max in practice
    attempt.write_text(json.dumps(rec))
    found = q_mod.abandoned_attempts(lane)
    assert len(found) == 1 and found[0]["tip"] == head

    # and the agent has to SEE it, which is the entire point
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        land_mod.cmd_land_status(_Args(mine=True), lane)
    printed = out.getvalue()
    assert "NOT QUEUED" in printed, printed
    assert "re-run" in printed, "must say what to do, not just that something is wrong"


def test_a_malformed_marker_REPORTS_rather_than_going_quiet(lane_repo, tmp_path):
    """`os.kill(-1, 0)` SUCCEEDS -- it is the broadcast form -- so a record with a missing or
    unparsable pid used to read as "alive" and suppress itself forever, which is the exact
    silent omission this feature exists to remove. A marker truncated by the disk filling
    produces one, and that is the very scenario that motivated the feature.
    """
    lane, _ = lane_repo
    attempt = q_mod.begin_attempt(lane, q_mod.lane_id(lane), q_mod.git(lane, "rev-parse", "HEAD"))
    for bad in ({"lane": q_mod.lane_id(lane), "tip": "abc"}, {"pid": "not-an-int"}, {"pid": -1}):
        rec = {"lane": q_mod.lane_id(lane), "tip": "abc", "started_at": time.time(), **bad}
        attempt.write_text(json.dumps(rec))
        assert q_mod.abandoned_attempts(lane), f"a marker with pid={bad} went silent"


def test_a_RECYCLED_pid_does_not_suppress_the_marker(lane_repo, tmp_path):
    """A pid alive but started AFTER the marker was written belongs to something else.

    The host churns thousands of build processes, so reuse is ordinary; treating the new
    occupant as our submit would suppress the marker permanently. Fails toward reporting, like
    the malformed case.
    """
    lane, _ = lane_repo
    attempt = q_mod.begin_attempt(lane, q_mod.lane_id(lane), q_mod.git(lane, "rev-parse", "HEAD"))
    rec = json.loads(attempt.read_text())
    # our own pid is certainly alive, but claim the marker predates the epoch
    rec["started_at"] = 1.0
    attempt.write_text(json.dumps(rec))
    assert q_mod.abandoned_attempts(lane), "a recycled pid must not suppress the marker"
    # negative control: with an honest started_at the same live pid IS our submit, and quiet
    rec["started_at"] = time.time()
    attempt.write_text(json.dumps(rec))
    assert q_mod.abandoned_attempts(lane) == [], "a genuinely in-flight submit must stay quiet"


def test_an_enqueued_lane_is_not_reported_as_abandoned(lane_repo, tmp_path, monkeypatch):
    """The negative control. A marker left behind by a submit that SUCCEEDED must not be
    reported — otherwise the warning appears on every successful land and gets tuned out,
    which is how a true signal becomes noise."""
    lane, _ = lane_repo
    head = q_mod.git(lane, "rev-parse", "HEAD")
    attempt = q_mod.begin_attempt(lane, q_mod.lane_id(lane), head)
    rec = json.loads(attempt.read_text())
    rec["pid"] = 2**22
    attempt.write_text(json.dumps(rec))
    q_mod.submit(lane, tip=head)
    assert q_mod.abandoned_attempts(lane) == [], "a queued lane is not an abandoned submit"


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
        q_mod,
        "gate_batch",
        lambda w, log, retries=2: q_mod.GateVerdict(False, "BUILD_FAIL", "/tmp/a", "ninja"),
    )
    with pytest.raises(q_mod.BaseRed) as ei:
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert "origin/master itself fails the gate" in str(ei.value)
    assert q_mod.results(repo, limit=20) == [], "rejected a lane for mainline's breakage"
    assert len(q_mod.pending(repo)) == 2, "submissions must stay QUEUED — never judged"


def test_queue_red_base_still_convicts_a_fault_the_base_passes(queue_repo, monkeypatch):
    """A red base is an alibi only for the failures it EXPLAINS.

    2026-07-31, a train wedged 19 hours: the batch failed `lint:{i386,amd64,arm64}` while the
    base-alone control failed only `smoke:net` — a contention flake (field manual §9.84) that
    says nothing about whether a lane added lint findings. The old rule read "base red ⇒ nobody
    at fault" and requeued everyone, so three real verify_signatures regressions went
    unreported while every other lane queued behind them. Both summaries were already on disk.
    """
    repo, _ = queue_repo
    subs = _two_queued(repo, "rbg")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="d" * 40, merged=list(b)),
    )
    seen = []

    def gate(w, log, retries=2):
        seen.append(1)
        # The base gate is whichever run follows a lone-submission red; it fails ONLY on net.
        if len(seen) > 2:
            return q_mod.GateVerdict(
                False,
                "NET_FLAKE",
                "/tmp/base",
                "smoke:net: NET_FLAKE",
                {"lint:i386": "ok", "smoke:net": "fail"},
            )
        return q_mod.GateVerdict(
            False,
            "LINT_FAIL",
            "/tmp/a",
            "lint:i386: 3 new",
            {"lint:i386": "fail", "smoke:net": "fail"},
        )

    monkeypatch.setattr(q_mod, "gate_batch", gate)
    with pytest.raises(q_mod.BaseRed) as ei:
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    # The train still halts — nothing can land on a red base — but the lint fault is ATTRIBUTED.
    rej = [r for r in q_mod.results(repo, limit=20) if r["outcome"] == "rejected"]
    assert len(rej) == 1, f"the lint fault must be attributed, not requeued: {rej}"
    assert "lint:i386" in rej[0]["reason"]
    assert "PASS on origin/master alone" in rej[0]["reason"]
    assert "not yours" in rej[0]["reason"], "must still say the base is separately broken"
    assert "nothing can land on a red base" in str(ei.value)


def test_queue_red_base_does_not_convict_a_SMOKE_fault_it_happens_to_pass(queue_repo, monkeypatch):
    """The guard on the rule above. A smoke stage is timing-sensitive, so "the base passed it
    at some other moment" is NOT evidence — convicting on that would re-create the false
    conviction the base check exists to prevent."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "rbs")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="9" * 40, merged=list(b)),
    )
    seen = []

    def gate(w, log, retries=2):
        seen.append(1)
        if len(seen) > 2:
            return q_mod.GateVerdict(
                False,
                "NET_FLAKE",
                "/tmp/base",
                "smoke:net",
                {"smoke:amd64": "ok", "smoke:net": "fail"},
            )
        return q_mod.GateVerdict(
            False, "BUGCHECK", "/tmp/a", "smoke:amd64", {"smoke:amd64": "fail", "smoke:net": "fail"}
        )

    monkeypatch.setattr(q_mod, "gate_batch", gate)
    with pytest.raises(q_mod.BaseRed):
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert q_mod.results(repo, limit=20) == [], "a smoke stage is not deterministic evidence"
    assert len(q_mod.pending(repo)) == 2


def test_queue_red_base_does_not_convict_a_stage_the_base_never_ran(queue_repo, monkeypatch):
    """The second guard: a base gate that halts early never reaches `lint`, and absence of a
    result is not a pass. Convicting by omission is how an early-failing base would frame
    every lane behind it."""
    repo, _ = queue_repo
    subs = _two_queued(repo, "rbo")
    _safe_git(monkeypatch)
    monkeypatch.setattr(
        q_mod,
        "build_batch",
        lambda r, w, b, log: q_mod.BatchResult(ok=False, tip="8" * 40, merged=list(b)),
    )
    seen = []

    def gate(w, log, retries=2):
        seen.append(1)
        if len(seen) > 2:
            # base died in the build — `lint:i386` has no result at all
            return q_mod.GateVerdict(
                False, "BUILD_FAIL", "/tmp/base", "build:i386", {"build:i386": "fail"}
            )
        return q_mod.GateVerdict(
            False, "LINT_FAIL", "/tmp/a", "lint:i386", {"build:i386": "ok", "lint:i386": "fail"}
        )

    monkeypatch.setattr(q_mod, "gate_batch", gate)
    with pytest.raises(q_mod.BaseRed):
        q_mod.land_subset(repo, repo, subs, lambda m: None)
    assert q_mod.results(repo, limit=20) == [], "convicted a lane for a stage the base never ran"


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
        q_mod,
        "gate_batch",
        lambda w, log, retries=2: q_mod.GateVerdict(False, "BUGCHECK", "/tmp/a", "boom"),
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
        lambda w, log, retries=2: (calls.append(1), q_mod.GateVerdict(True, "", "", ""))[1],
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
        lambda w, log, retries=2: (
            gates.append(1),
            q_mod.GateVerdict(False, "HOST_STARVED", "/tmp/a", "queued"),
        )[1],
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
    monkeypatch.setattr(
        q_mod, "gate_batch", lambda w, log, retries=2: q_mod.GateVerdict(False, "", "", "")
    )
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
        return (
            q_mod.GateVerdict(True, "", "", "")
            if ok
            else q_mod.GateVerdict(False, "BUGCHECK", "/tmp/a", "boom")
        )

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
        seen["env"] = kw.get("env", {})
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(q_mod.subprocess, "run", _fake_run)
    monkeypatch.setenv("VIBEOS_ARTIFACT_ROOT", str(tmp_path))
    q_mod.gate_batch(tmp_path, lambda m: None, retries=0)
    assert "--clean" in seen["cmd"], seen["cmd"]
    assert "--keep-going" in seen["cmd"], seen["cmd"]
    assert seen["env"]["VIBEOS_JOB_CLASS"] == "integration"


def test_direct_land_gate_tags_integration(tmp_path):
    from argparse import Namespace

    class Recorder(Run):
        def run(self, cmd, **kw):
            assert kw["env"]["VIBEOS_JOB_CLASS"] == "integration"
            assert "PATH" in kw["env"]
            return subprocess.CompletedProcess(cmd, 0)

        def add(self, stage):
            assert stage.status == "ok"

    land_mod._run_gate(tmp_path, Recorder("test-direct", "all"), Namespace(clean=True, retries=0))


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


def test_difftest_discovers_every_generated_refstub_artifact(tmp_path: Path):
    """The set has grown from three artifacts to seven in two months, once per
    reference TU. `refstub/msvcrt_<stem>_protos.h` IS the registration, so a new
    reference TU is covered the moment its header lands — a hardcoded list is how
    the eighth arrives uncovered."""
    from . import difftest as difftest_mod

    refstub = tmp_path / "sdk" / "lib" / "rust" / "msvcrt-difftest" / "refstub"
    refstub.mkdir(parents=True)
    for name in ("msvcrt_file_protos.h", "msvcrt_time_protos.h", "msvcrt_shim_decls.h"):
        (refstub / name).write_text("")

    labels = [label for label, _ in difftest_mod.generated_checks(tmp_path)]
    assert labels == ["consts", "protos:file", "protos:time"], labels

    # …and the newcomer, without touching this module.
    (refstub / "msvcrt_process_protos.h").write_text("")
    labels = [label for label, _ in difftest_mod.generated_checks(tmp_path)]
    assert "protos:process" in labels
    # The argv must name the SOURCE the header was derived from, not the header.
    argv = dict(difftest_mod.generated_checks(tmp_path))["protos:process"]
    assert argv[-2:] == ["process.c", "--check"], argv


def test_difftest_reads_a_crates_target_triple_from_its_manifest(tmp_path: Path):
    """A harness that only exists in a 32-bit binary declares its triple, and the driver
    reads it. Building msvcrt-seh-difftest for the host is not a failure — every `#[cfg]`
    in it compiles to nothing and it grades zero rows, which the run would then have to
    catch downstream instead of never happening."""
    from . import difftest as difftest_mod

    ws = tmp_path / "sdk" / "lib" / "rust"
    (ws / "plain-difftest").mkdir(parents=True)
    (ws / "plain-difftest" / "Cargo.toml").write_text(
        '[package]\nname = "plain-difftest"\n\n[dependencies]\n'
    )
    (ws / "msvcrt-seh-difftest").mkdir(parents=True)
    (ws / "msvcrt-seh-difftest" / "Cargo.toml").write_text(
        '[package]\nname = "msvcrt-seh-difftest"\n\n'
        '[package.metadata.difftest]\ntarget = "i686-unknown-linux-gnu"\n\n'
        '[dependencies]\ntarget = "not this one"\n'
    )

    assert difftest_mod.crate_target(tmp_path, "plain-difftest") is None
    assert difftest_mod.crate_target(tmp_path, "absent-difftest") is None
    # …and the key must come from the metadata table, not from whatever `target =` line
    # happens to appear first in the manifest.
    assert difftest_mod.crate_target(tmp_path, "msvcrt-seh-difftest") == "i686-unknown-linux-gnu"


def test_difftest_covers_the_seh_layout_guardian(tmp_path: Path):
    """The i386 SEH layout header is generated from the tree's own PSDK headers, so it goes
    stale without anyone editing it — and a stale offset there regrades a structure walk
    rather than failing."""
    from . import difftest as difftest_mod

    refstub = tmp_path / "sdk" / "lib" / "rust" / "msvcrt-difftest" / "refstub"
    refstub.mkdir(parents=True)
    assert "layout:seh-i386" not in dict(difftest_mod.generated_checks(tmp_path))

    seh = tmp_path / "sdk" / "lib" / "rust" / "msvcrt-seh-difftest" / "refstub"
    seh.mkdir(parents=True)
    (seh / "seh_layout.h").write_text("")
    argv = dict(difftest_mod.generated_checks(tmp_path))["layout:seh-i386"]
    assert argv[-1] == "--check"
    assert argv[-2].endswith("gen_msvcrt_seh_layout.py")


def test_difftest_a_stale_generated_header_fails_the_run(tmp_path, monkeypatch):
    """A stale refstub header does not make one row wrong — it makes every row over
    that TU suspect, and the row that finally reaches it reports a DIVERGENCE in the
    Rust rather than "your oracle is stale". So the drift check is its own stage and
    its own classification."""
    from types import SimpleNamespace

    from . import artifacts
    from . import difftest as difftest_mod

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    refstub = tmp_path / "sdk" / "lib" / "rust" / "msvcrt-difftest" / "refstub"
    refstub.mkdir(parents=True)
    (refstub / "msvcrt_time_protos.h").write_text("")

    run = artifacts.Run("difftest", "host")
    monkeypatch.setattr(
        run,
        "run",
        lambda *a, **k: SimpleNamespace(
            returncode=1,
            stdout="DRIFT: refstub/msvcrt_time_protos.h is stale or missing\n",
            stderr="",
        ),
    )
    assert difftest_mod.check_generated(tmp_path, run) is False
    assert all(s.status == "fail" for s in run.stages)
    assert any("DRIFT" in line for s in run.stages for line in s.salient)


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


def test_cohab_stage_reports_a_finding_and_a_refusal_differently(tmp_path, monkeypatch):
    """The cohab stage has TWO distinct reds and they need different first lines.

    A FINDING is on stdout and names the .rs whose flip is a no-op — that is the one worth
    quoting verbatim. A REFUSAL (stale archive, build layout moved) is on stderr and means
    the tool declined to judge at all; scraping stdout for it yields nothing, and a stage
    that can turn the merge train red must never report a bare 'exit 1'.
    """
    finding = (
        "sdk/lib/rust/msvcrt-crt-rust/src/msvcrt/math.rs: [i386] nothing that links ahead "
        "of msvcrt_static.dir defines `__adj_fdiv_m16i@4`"
    )

    class _Finding:
        returncode = 1
        stdout = finding + "\n"
        stderr = "verify_cohab_coverage: 1 finding(s); 1 family/arch pair(s)\n"

    class _Refusal:
        returncode = 1
        stdout = ""
        stderr = "verify_cohab_coverage: libmsvcrt_crt_cargo.a for i386 is OLDER than math.rs\n"

    monkeypatch.setattr(gate_mod.subprocess, "run", lambda *a, **k: _Finding())
    r = gate_mod._cohab_once(tmp_path, "i386", tmp_path / "cohab-i386")
    assert r["ok"] is False and r["arch"] == "i386"
    assert r["salient"] == finding
    assert finding in (tmp_path / "cohab-i386" / "cohab-console.log").read_text()

    monkeypatch.setattr(gate_mod.subprocess, "run", lambda *a, **k: _Refusal())
    r = gate_mod._cohab_once(tmp_path, "i386", tmp_path / "cohab-refuse")
    assert r["ok"] is False
    assert "exit 1" in r["salient"] and "is OLDER than" in r["salient"]


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


def test_link_classifier_does_not_mistake_duplicate_symbol_WARNINGS_for_the_failure():
    """This tree links with --allow-multiple-definition, so ld.lld emits
    `warning: duplicate symbol` on every build. The classifier matched bare
    "duplicate symbol"/"ld.lld:" before the rustc branch, so a compile error that
    merely CO-OCCURRED with those warnings was reported as LINK_FAIL with three
    warnings as its salient — and the real cause (error[E0308] in census.rs)
    appeared nowhere in the summary. Verbatim shapes from that build."""
    warns_plus_rustc = (
        "ld.lld: warning: duplicate symbol: __udivti3\n"
        "ld.lld: warning: duplicate symbol: __umodti3\n"
        "error[E0308]: mismatched types\n"
        "error: could not compile `ntoskrnl-rust` (lib) due to 2 previous errors\n"
    )
    cls, salient = build_mod._classify_build_failure(warns_plus_rustc, "")
    assert cls == "CARGO_CHECK_FAIL"
    assert any("E0308" in s for s in salient), salient

    # a REAL undefined symbol still classifies as a link failure
    real_undef = (
        "ld.lld: warning: duplicate symbol: __udivti3\n"
        "ld.lld: error: undefined symbol: MmArmAccessFault\n"
    )
    assert build_mod._classify_build_failure(real_undef, "")[0] == "LINK_FAIL"
    # and so does a genuine duplicate-symbol ERROR
    assert (
        build_mod._classify_build_failure("ld.lld: error: duplicate symbol: foo", "")[0]
        == "LINK_FAIL"
    )


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


# --- WoW64 probe-ladder verdict check (smoke --ladder) ----------------------
# The ladder runs AFTER the desktop marker, so the ordinary smoke stops watching
# before it starts; these pin the parser that makes `--ladder` able to see it.


def _ladder_log(overrides: dict | None = None, drop: set | None = None) -> str:
    """A serial transcript with every ladder guest reporting PASS, minus `drop`,
    with `overrides` applied. Line shape is verbatim from a real amd64 boot."""
    from tests.qemu.cli import ladder as ladder_mod

    out = []
    for guest, code in ladder_mod.LADDER.items():
        if drop and guest in drop:
            continue
        code = (overrides or {}).get(guest, code)
        out.append(f"[WINLOGON] {guest}.exe exit = 0x{code:04x} (0x600D = PASS)")
    return "\n".join(out) + "\n"


def test_ladder_all_pass_is_clean():
    from tests.qemu.cli import ladder as ladder_mod

    assert ladder_mod.check(_ladder_log()) is None
    # Derived from the table, not hardcoded: this assertion was "9/9" and the
    # tenth guest broke it while the fixture -- which builds itself FROM the
    # table -- was already correct. A count literal here tests the literal.
    n = len(ladder_mod.LADDER)
    assert ladder_mod.summary(_ladder_log()).startswith(f"{n}/{n} guests PASS")


def test_ladder_gui32_pass_code_is_600e_not_600d():
    """gui32 is the GDI/draw rung and reports its own verdict code. Asserting a
    blanket 0x600D would make the check permanently red for a healthy boot --
    which is how a gate gets disabled."""
    from tests.qemu.cli import ladder as ladder_mod

    assert ladder_mod.LADDER["gui32"] == 0x600E
    bad = ladder_mod.check(_ladder_log({"gui32": 0x600D}))
    assert bad is not None and "gui32" in bad["salient"][0]


def test_ladder_catches_the_selfmod32_amd64_regression():
    """The real E19 finding: selfmod32 exited 0xBAD9 on amd64 for a day because
    its rung-5 fixture writes to a read-only volume there, and no gate could
    see it."""
    from tests.qemu.cli import ladder as ladder_mod

    res = ladder_mod.check(_ladder_log({"selfmod32": 0xBAD9}))
    assert res is not None
    assert res["classification"] == "LADDER_FAIL"
    assert any("selfmod32" in s and "0xbad9" in s for s in res["salient"])


def test_ladder_missing_guest_is_a_failure_not_a_pass():
    """A guest that crashes or hangs emits NO verdict line. Treating absence as
    success is the exact failure mode this check exists to prevent."""
    from tests.qemu.cli import ladder as ladder_mod

    res = ladder_mod.check(_ladder_log(drop={"thread32"}))
    assert res is not None and res["classification"] == "LADDER_FAIL"
    assert any("thread32" in s and "never reported" in s for s in res["salient"])


def test_ladder_nothing_staged_reports_the_build_recipe_not_a_regression():
    """Zero verdicts means the media has no guests staged (arm64 without
    VIBEOS_FEX=1). That must read as 'you built it wrong', not 'the OS broke'."""
    from tests.qemu.cli import ladder as ladder_mod

    res = ladder_mod.check("[EXPLORER] _tWinMain entered\nshell ready\n")
    assert res is not None and res["classification"] == "LADDER_NOT_STAGED"
    assert any("VIBEOS_FEX=1" in s for s in res["salient"])


def test_ladder_extra_arm64_guests_are_reported_but_not_required():
    """arm64 also stages fexprobe/wow64probe. They must not be required (that
    would make the table 'whatever arm64 happens to stage' instead of the
    cross-arch parity claim) but they should be visible in the summary."""
    from tests.qemu.cli import ladder as ladder_mod

    text = _ladder_log() + "[WINLOGON] wow64probe.exe exit = 0x600d\n"
    assert ladder_mod.check(text) is None
    assert "wow64probe" in ladder_mod.summary(text)


def test_ladder_takes_the_last_verdict_per_guest():
    from tests.qemu.cli import ladder as ladder_mod

    text = "[WINLOGON] hello32.exe exit = 0xbad1\n" + _ladder_log()
    assert ladder_mod.verdicts(text)["hello32"] == 0x600D


# --- probe presence in the MEDIA (the 9-hour blind soak, 2026-07-26) --------


def _fake_media(tmp_path, rel: str, body: bytes):
    p = tmp_path / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(body)
    return p


def test_probe_present_in_media_reports_nothing_missing(tmp_path):
    from tests.qemu.cli import provenance

    _fake_media(
        tmp_path, "build-i386/livecd.iso", b"\x00" * 100 + b"[MMGUARD] hi\x00" + b"\xff" * 50
    )
    assert provenance.probes_missing_from_media(tmp_path, "i386", ["[MMGUARD]"]) == []


def test_probe_absent_from_media_is_reported_with_the_mtime_and_the_stakes(tmp_path):
    """The exact 2026-07-26 failure: the literal is in the module but not the
    image, and the old check looked at the module."""
    from tests.qemu.cli import provenance

    _fake_media(tmp_path, "build-i386/livecd.iso", b"a boot image with no probe in it")
    reasons = provenance.probes_missing_from_media(tmp_path, "i386", ["[KSTACK]"])
    assert len(reasons) == 1
    assert "[KSTACK]" in reasons[0]
    assert "0 times" in reasons[0]
    # It must say WHY this matters, not just that a count was zero.
    assert "silence would prove nothing" in reasons[0]


def test_arm64_checks_the_ramdisk_too_not_just_the_outer_image(tmp_path):
    """arm64's kernel lives in ramdisk.img; checking only boot.img would repeat
    the same right-check-wrong-artifact mistake one level down."""
    from tests.qemu.cli import provenance

    _fake_media(tmp_path, "qemu/boot.img", b"[MMGUARD] present here")
    _fake_media(tmp_path, "qemu/ramdisk.img", b"but absent in the ramdisk")
    reasons = provenance.probes_missing_from_media(tmp_path, "arm64", ["[MMGUARD]"])
    assert len(reasons) == 1
    assert "ramdisk.img" in reasons[0]


def test_probe_straddling_a_read_boundary_is_still_found(tmp_path):
    """Streaming with no overlap would miss a literal split across chunks."""
    from tests.qemu.cli import provenance

    chunk = 8 << 20
    body = b"\x00" * (chunk - 4) + b"[MMGUARD]" + b"\x00" * 16
    _fake_media(tmp_path, "build-i386/livecd.iso", body)
    assert provenance.probes_missing_from_media(tmp_path, "i386", ["[MMGUARD]"]) == []


def test_missing_media_file_is_named_rather_than_silently_passing(tmp_path):
    from tests.qemu.cli import provenance

    reasons = provenance.probes_missing_from_media(tmp_path, "i386", ["[MMGUARD]"])
    assert reasons and "does not exist" in reasons[0]


def test_no_literals_requested_is_a_no_op_not_a_failure(tmp_path):
    from tests.qemu.cli import provenance

    assert provenance.probes_missing_from_media(tmp_path, "i386", []) == []
    assert provenance.probes_missing_from_media(tmp_path, "i386", ["  "]) == []


def test_rglob_stable_survives_a_directory_vanishing_mid_walk(tmp_path, monkeypatch):
    """A parallel gate's `--clean` build deletes a sibling arch's cargo target dir while
    another arch's smoke is walking it. pathlib raises FileNotFoundError from inside the
    generator; the walk must be redone, not abandoned half-finished."""
    from tests.qemu.cli import provenance

    (tmp_path / "aarch64-pc-windows-gnullvm").mkdir()
    (tmp_path / "aarch64-pc-windows-gnullvm" / "libfoo_rust.a").write_text("x")

    real_rglob = Path.rglob
    calls = {"n": 0}

    def flaky(self, pattern):
        calls["n"] += 1
        if calls["n"] == 1:
            # First walk races the concurrent build and dies part-way through.
            yield from ()
            raise FileNotFoundError(2, "No such file or directory", str(tmp_path / "i686-kernel"))
        yield from real_rglob(self, pattern)

    monkeypatch.setattr(Path, "rglob", flaky)
    found = provenance.rglob_stable(tmp_path, "libfoo_rust.a")

    assert calls["n"] == 2, "the racing walk must be redone, not swallowed"
    assert [p.name for p in found] == ["libfoo_rust.a"]


def test_rglob_stable_reraises_when_the_tree_is_genuinely_broken(tmp_path, monkeypatch):
    """Retries are for a transient race. A directory that is ALWAYS unreadable is a real
    fault, and returning a silently-short list would read as a staleness verdict."""
    from tests.qemu.cli import provenance

    def always_fails(self, pattern):
        yield from ()
        raise PermissionError(13, "Permission denied", str(tmp_path))

    monkeypatch.setattr(Path, "rglob", always_fails)
    with pytest.raises(OSError):
        provenance.rglob_stable(tmp_path, "libfoo_rust.a")


# --- scan_boot_media_drift: the two boot media must stage the same CM hives ------

_CMAKE_FIXTURE = """
    add_custom_command(
        OUTPUT ${CMAKE_BINARY_DIR}/boot/bootdata/system
               ${CMAKE_BINARY_DIR}/boot/bootdata/sam
        COMMAND native-mkhive -h:SYSTEM,SAM -d:${CMAKE_BINARY_DIR}/boot/bootdata ${_inf}
        DEPENDS native-mkhive ${_inf})

    add_cd_file(
        FILE ${CMAKE_BINARY_DIR}/boot/bootdata/system
             ${CMAKE_BINARY_DIR}/boot/bootdata/sam
        TARGET livecd_hives
        DESTINATION reactos/system32/config
        FOR livecd)

    add_cd_file(
        FILE ${CMAKE_BINARY_DIR}/boot/bootdata/BCD
        TARGET bcd_hive
        DESTINATION efi/boot
        FOR bootcd regtest livecd)
"""


def _ramdisk_fixture(copied, required):
    return (
        "REQUIRED_FILES=(\n"
        + "".join(f'    "$BUILD_DIR/boot/bootdata/{h}"\n' for h in required)
        + ")\n\n"
        + "".join(
            f'RMC "$BUILD_DIR/boot/bootdata/{h}"   ::/reactos/system32/config/{h}\n' for h in copied
        )
    )


def _run_drift(tmp_path, copied, required):
    repo = Path(__file__).resolve().parents[3]
    cmake = tmp_path / "CMakeMacros.cmake"
    cmake.write_text(_CMAKE_FIXTURE)
    ramdisk = tmp_path / "mkramdisk.sh"
    ramdisk.write_text(_ramdisk_fixture(copied, required))
    return subprocess.run(
        [
            "python3",
            str(repo / "tools" / "scan_boot_media_drift.py"),
            "--cmake",
            str(cmake),
            "--ramdisk",
            str(ramdisk),
        ],
        capture_output=True,
        text=True,
    )


def test_boot_media_drift_passes_when_both_media_agree(tmp_path):
    r = _run_drift(tmp_path, copied=["system", "sam"], required=["system", "sam"])
    assert r.returncode == 0, r.stdout
    assert "findings=0" in r.stdout


def test_boot_media_drift_catches_a_hive_missing_from_the_ramdisk(tmp_path):
    """The #71 bug exactly: `sam` on the livecd, absent from the ramdisk. The kernel
    fabricates an empty volatile hive rather than failing, so nothing else can see it."""
    r = _run_drift(tmp_path, copied=["system"], required=["system"])
    assert r.returncode == 1
    assert "[RAMDISK_MISSING]" in r.stdout
    assert "'sam'" in r.stdout


def test_boot_media_drift_catches_a_copied_but_unguarded_hive(tmp_path):
    """Copied without a REQUIRED_FILES entry: if the build stops emitting it, the copy
    stages nothing and we are back to a fabricated hive with no error anywhere."""
    r = _run_drift(tmp_path, copied=["system", "sam"], required=["system"])
    assert r.returncode == 1
    assert "[RAMDISK_UNGUARDED]" in r.stdout
    assert "'sam'" in r.stdout


def test_boot_media_drift_ignores_hives_staged_outside_the_config_dir(tmp_path):
    """BCD goes to efi/boot for the EFI boot manager — it is not a \\REGISTRY\\MACHINE
    hive and the ramdisk is not an ESP, so demanding it would be a false positive."""
    r = _run_drift(tmp_path, copied=["system", "sam"], required=["system", "sam"])
    assert r.returncode == 0
    assert "BCD" not in r.stdout


def test_boot_media_drift_fails_loudly_if_it_can_no_longer_find_the_reference_set(tmp_path):
    """A parser that silently matches nothing would pass every tree forever. If the cmake
    layout moves, this must fail rather than go quietly vacuous."""
    cmake = tmp_path / "CMakeMacros.cmake"
    cmake.write_text("# the hive blocks moved somewhere else\n")
    ramdisk = tmp_path / "mkramdisk.sh"
    ramdisk.write_text(_ramdisk_fixture([], []))
    repo = Path(__file__).resolve().parents[3]
    r = subprocess.run(
        [
            "python3",
            str(repo / "tools" / "scan_boot_media_drift.py"),
            "--cmake",
            str(cmake),
            "--ramdisk",
            str(ramdisk),
        ],
        capture_output=True,
        text=True,
    )
    assert r.returncode == 1
    assert "[NO_REFERENCE]" in r.stdout


def test_check_registers_the_vendored_registry_stage():
    """`vibeos-cli check` must run tools/vendored.py.

    The registry answers "which C is NOT ours to translate" — the denominator under
    every progress number in this project. Its container ratchet only ratchets if
    something runs it, and a tool nobody runs decays into a liability (see the
    difftest driver's docstring for the version of this lesson that cost us two red
    harnesses on master).
    """
    from . import lints as lints_mod

    repo = Path(__file__).resolve().parents[3]
    specs = lints_mod.source_lint_specs(repo)
    names = [s[0] for s in specs]
    assert "vendored_registry" in names
    argv = next(s[1] for s in specs if s[0] == "vendored_registry")
    assert argv != ["MISSING"], "tools/vendored.py is not where the stage expects it"


def test_vendored_registry_stage_is_silent_on_stdout_when_green():
    """stdout IS the findings channel for a `stdout`-mode lint stage.

    Any non-tally line there counts as a finding, so a chatty success message would
    park this stage permanently red. The pass condition is an empty stdout.
    """
    repo = Path(__file__).resolve().parents[3]
    r = subprocess.run(
        ["python3", str(repo / "tools" / "vendored.py")],
        capture_output=True,
        text=True,
        cwd=str(repo),
    )
    assert r.returncode == 0, r.stdout + r.stderr
    assert r.stdout == "", f"stdout must be empty when green, got: {r.stdout!r}"
    assert "OK" in r.stderr


# --- the ratchet gates on assertion IDENTITIES, not counts (#78) --------------

_SWEEP = """\
[KMTEST-SWEEP] launch IoDeviceObject
IoDeviceObject_drv.c:198: Test failed: Status = 0xc0000034, expected 0x00000000
IoDeviceObject_drv.c:206: Test failed: Ptr = FFFF800004952878, expected 0
[KMTEST:END IoDeviceObject pass=185 fail=2 skip=0]
[KMTEST-SWEEP] exit IoDeviceObject code=0
[KMTEST-SWEEP] done 1 tests
"""


def _parser():
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "parse_kmtest_serial",
        Path(__file__).resolve().parents[3] / "tools" / "conformance" / "parse_kmtest_serial.py",
    )
    assert spec is not None and spec.loader is not None
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def test_failing_assertions_are_parsed_and_round_trip_through_the_baseline():
    """file:line is the identity. The message is deliberately NOT part of it —
    it carries addresses and NTSTATUS values that differ every boot."""
    P = _parser()
    results, _done, _bug = P.parse(_SWEEP)
    assert [r["name"] for r in results] == ["IoDeviceObject"]
    assert results[0]["asserts"] == {"IoDeviceObject_drv.c:198", "IoDeviceObject_drv.c:206"}
    text = P.fmt(results[0])
    assert "    assert IoDeviceObject_drv.c:198" in text
    base = _round_trip(P, text)
    assert base["IoDeviceObject"]["asserts"] == results[0]["asserts"]
    assert P.compare(results, base) == []


def _round_trip(P, text):
    import tempfile

    fn = tempfile.mktemp()
    Path(fn).write_text("# hdr\n" + text + "\n")
    return P.load_baseline(fn)


def test_offsetting_changes_are_caught_though_the_counts_never_move():
    """THE point of #78. One assertion starts failing and another stops, so
    pass/fail/skip are byte-identical and the count ratchet reports nothing.
    Measured on 595 archived arm64 sweeps: 33 of the 37 count-signatures shared
    by more than one run cover runs whose failing assertions differ."""
    P = _parser()
    banked, _, _ = P.parse(_SWEEP)
    base = _round_trip(P, P.fmt(banked[0]))
    swapped = _SWEEP.replace("IoDeviceObject_drv.c:206", "IoDeviceObject_drv.c:301")
    now, _, _ = P.parse(swapped)
    assert now[0]["pass"] == banked[0]["pass"] and now[0]["fail"] == banked[0]["fail"]
    # count-only floor (a row not yet seeded with identities) sees nothing
    countonly = {
        "IoDeviceObject": {k: v for k, v in base["IoDeviceObject"].items() if k != "asserts"}
    }
    assert P.compare(now, countonly) == []
    # identity floor names it
    assert P.compare(now, base) == [
        "IoDeviceObject: NEW failing assertion IoDeviceObject_drv.c:301"
    ]


def test_a_bistable_assertion_marked_with_a_question_mark_is_not_a_regression():
    """IoDeviceObject really is bistable on all three arches. Without the mark
    every soak round reports a regression or an improvement, forever."""
    P = _parser()
    results, _, _ = P.parse(_SWEEP)
    base = _round_trip(P, P.fmt(results[0], flaky={"IoDeviceObject_drv.c:206"}))
    assert base["IoDeviceObject"]["flaky"] == {"IoDeviceObject_drv.c:206"}
    # it fired this time: known, so silent
    assert P.compare(results, base) == []
    # and it did not fire this time: still silent, and still on the floor
    quiet = _SWEEP.replace(
        "IoDeviceObject_drv.c:206: Test failed: Ptr = FFFF800004952878, expected 0\n", ""
    ).replace("fail=2", "fail=1")
    now, _, _ = P.parse(quiet)
    assert P.compare(now, base) == []


def test_reseeding_a_row_keeps_its_bistable_marks():
    """A wholesale re-seed from one boot must not delete an `assert?` — that
    banks whichever face this boot showed and hands the next run a false
    regression. Same failure --only exists to prevent, one level down."""
    prev = (
        "# hdr\n"
        "IoDeviceObject OK pass=185 fail=2 skip=0\n"
        "    assert IoDeviceObject_drv.c:198\n"
        "    assert? IoDeviceObject_drv.c:206\n"
    )
    assert conformance_mod._flaky_of(prev, "IoDeviceObject") == {"IoDeviceObject_drv.c:206"}
    # this boot did NOT show the bistable one
    res = [
        {
            "name": "IoDeviceObject",
            "status": "OK",
            "pass": 185,
            "fail": 1,
            "skip": 0,
            "asserts": {"IoDeviceObject_drv.c:198"},
        }
    ]
    text, refusal = conformance_mod.merge_baseline_rows(prev, res, ["IoDeviceObject"], _fmt)
    assert refusal is None and text is not None
    assert "    assert? IoDeviceObject_drv.c:206" in text, "the flake mark was dropped"


def test_an_unseeded_row_still_gates_on_the_fail_count():
    """Identity seeding is per-arch and incremental. A row with no `assert`
    lines must keep its old count gate rather than silently gating nothing."""
    P = _parser()
    base = _round_trip(P, "IoDeviceObject OK pass=185 fail=2 skip=0")
    assert base["IoDeviceObject"].get("asserts") is None
    worse = _SWEEP.replace("fail=2", "fail=3")
    now, _, _ = P.parse(worse)
    assert P.compare(now, base) == ["IoDeviceObject: fail 2 -> 3 (grew)"]


def test_reseeding_one_row_leaves_every_other_rows_identities_alone():
    """--only replaces the named row's whole assert block and nothing else. An
    indented `assert` line belongs to the row above it, not to a test named
    "assert" — getting that wrong would rewrite the file into nonsense."""
    prev = (
        "# hdr\n"
        "FsRtlLegal OK pass=100 fail=1 skip=0\n"
        "    assert FsRtlLegal.c:11\n"
        "IoDeviceObject OK pass=185 fail=2 skip=0\n"
        "    assert IoDeviceObject_drv.c:198\n"
        "    assert? IoDeviceObject_drv.c:206\n"
    )
    res = [
        {"name": "FsRtlLegal", "status": "OK", "pass": 132, "fail": 0, "skip": 0, "asserts": set()},
        {
            "name": "IoDeviceObject",
            "status": "OK",
            "pass": 185,
            "fail": 1,
            "skip": 0,
            "asserts": {"IoDeviceObject_drv.c:198"},
        },
    ]
    text, refusal = conformance_mod.merge_baseline_rows(prev, res, ["FsRtlLegal"], _fmt)
    assert refusal is None and text is not None
    # the re-seeded row drops the assertion that no longer fails
    assert "FsRtlLegal.c:11" not in text
    # the untouched row keeps BOTH of its identities, mark included
    assert "    assert IoDeviceObject_drv.c:198" in text
    assert "    assert? IoDeviceObject_drv.c:206" in text


def test_seed_asserts_adds_identities_without_touching_the_counts():
    """--seed-asserts upgrades a count-only floor in place. Re-rolling the counts
    from the soak is exactly what a wholesale --write-baseline does wrong."""
    P = _parser()
    prev = "# hdr\nExPools OK pass=100 fail=2 skip=0 exit=0\n"
    rounds = [
        [
            {
                "name": "ExPools",
                "status": "OK",
                "pass": 7,
                "fail": 2,
                "skip": 0,
                "asserts": {"ExPools.c:119", "ExPools.c:176"},
            }
        ],
        [
            {
                "name": "ExPools",
                "status": "OK",
                "pass": 7,
                "fail": 2,
                "skip": 0,
                "asserts": {"ExPools.c:119", "ExPools.c:176"},
            }
        ],
        [
            {
                "name": "ExPools",
                "status": "OK",
                "pass": 7,
                "fail": 1,
                "skip": 0,
                "asserts": {"ExPools.c:119"},
            }
        ],
    ]
    text, report = conformance_mod.seed_asserts_into_baseline(prev, rounds, P)
    # the banked counts survive verbatim
    assert "ExPools OK pass=100 fail=2 skip=0 exit=0" in text
    # unanimous -> assert; seen in 2 of 3 -> assert?
    assert "    assert ExPools.c:119" in text
    assert "    assert? ExPools.c:176" in text
    assert "1 stable + 1 bistable" in report


def test_seed_asserts_ignores_rounds_where_the_test_never_completed():
    """A round that CRASHed has no assertions for that test — it says nothing
    about which of them fail. Counting it as "did not fail" would mark every
    stable failure bistable and permanently weaken the floor from one bad boot."""
    P = _parser()
    prev = "# hdr\nExPools OK pass=100 fail=2 skip=0\n"
    good = {
        "name": "ExPools",
        "status": "OK",
        "pass": 7,
        "fail": 1,
        "skip": 0,
        "asserts": {"ExPools.c:119"},
    }
    dead = {"name": "ExPools", "status": "CRASH", "pass": 0, "fail": 0, "skip": 0, "asserts": set()}
    text, _report = conformance_mod.seed_asserts_into_baseline(
        prev, [[good], [dead], [good], [good]], P
    )
    assert "    assert ExPools.c:119" in text, "the crashed round diluted unanimity"
    assert "assert?" not in text


def test_seed_asserts_will_not_add_a_row_the_floor_does_not_carry():
    """Adding a test to the floor is --write-baseline's job: it has the coverage
    refusal in front of it, and this does not."""
    P = _parser()
    prev = "# hdr\nExPools OK pass=100 fail=2 skip=0\n"
    rounds = [
        [
            {
                "name": "NewTest",
                "status": "OK",
                "pass": 1,
                "fail": 1,
                "skip": 0,
                "asserts": {"NewTest.c:5"},
            }
        ]
    ] * 3
    text, report = conformance_mod.seed_asserts_into_baseline(prev, rounds, P)
    assert "NewTest" not in text.replace("NewTest.c", "")
    assert "not on the floor" in report


# ---------------------------------------------------------------------------
# conformance ratchet wired into `gate` (2026-09-06)
# ---------------------------------------------------------------------------
# The ratchet was report-only for its whole life: it computed a correct verdict
# and nothing was obliged to act on it. These pin the three things that make it
# a real gate rather than a louder log line.


def test_conformance_lane_is_retryable_so_one_bistable_cannot_reject_a_batch():
    """A first-attempt ratchet miss must classify as a RETRYABLE flake.

    The identity floor marks the bistables it knows about (`assert?`), but the
    corpus is not closed — an intermittent that never fired during seeding is
    indistinguishable from a regression on any single sweep. The merge queue
    gates ONCE for N lanes, so a non-retryable miss would reject a whole batch
    of innocent lanes on a coin flip. A real regression still fails the retry.
    """
    assert "CONFORMANCE_FLAKE" in gate_mod.RETRYABLE


def test_conformance_lane_skips_loudly_when_the_arch_has_no_floor(tmp_path):
    """No baseline means nothing to ratchet against. It must SKIP, not silently
    pass — a stage that cannot fail must not read as one that held."""
    r = gate_mod._conformance_once(tmp_path, "i386", tmp_path / "conf")
    assert r["verdict"] == "SKIP"
    assert r["ok"] is True
    assert "no baseline" in r["salient"]


def test_conformance_lane_reports_the_failing_identities_not_the_whole_table(tmp_path, monkeypatch):
    """The sweep prints a per-test table; only the NEW failing assertions belong
    in the gate summary, or the real signal drowns."""
    base = tmp_path / "tools" / "conformance-baseline" / "amd64"
    base.mkdir(parents=True)
    (base / "kmtest.txt").write_text("IoDeviceObject 12 0\n")

    class _CP:
        returncode = 1
        stdout = (
            "  IoDeviceObject   pass=11 fail=1\n"
            "  RATCHET REGRESSIONS (1):\n"
            "    IoDeviceObject: NEW failing assertion IoDeviceObject_drv.c:206\n"
        )
        stderr = ""

    monkeypatch.setattr(gate_mod.subprocess, "run", lambda *a, **k: _CP())
    r = gate_mod._conformance_once(tmp_path, "amd64", tmp_path / "conf")
    assert r["ok"] is False
    assert r["classification"] == "CONFORMANCE_FLAKE"
    assert r["retryable"] is True
    assert "IoDeviceObject_drv.c:206" in r["salient"]
    assert "pass=11" not in r["salient"], "the per-test table must not reach the summary"


# ---------------------------------------------------------------------------
# the kmtest payload provisioning scar (2026-09-06)
# ---------------------------------------------------------------------------
# Wiring the conformance lanes made the gate build stamp VIBEOS_PAYLOAD=kmtest
# unconditionally. kmtest-sweep-<arch>.exe is GITIGNORED, so the change worked in
# the worktree that had months of conformance runs behind it and broke the first
# fresh tree it met — the merge queue's integration worktree — as a BUILD_FAIL
# attributed to whichever innocent lanes shared the batch.


def test_kmtest_payload_is_built_when_missing_not_assumed_present(tmp_path, monkeypatch):
    """A fresh worktree has sweep.c and not the exe. The gate must BUILD it."""
    sh = tmp_path / "tools" / "payloads" / "kmtest"
    sh.mkdir(parents=True)
    (sh / "build.sh").write_text("#!/bin/bash\n")
    (tmp_path / ".build-tools" / "llvm-mingw").mkdir(parents=True)

    calls = []

    def _fake_run(cmd, **kw):
        calls.append(cmd)
        (sh / f"kmtest-sweep-{cmd[2]}.exe").write_text("x")
        return subprocess.CompletedProcess(cmd, 0, "", "")

    monkeypatch.setattr(gate_mod.subprocess, "run", _fake_run)
    ok, why = gate_mod._ensure_kmtest_payload(tmp_path, ["i386", "amd64"])
    assert ok and why == ""
    assert [c[2] for c in calls] == ["i386", "amd64"], "must build each missing arch"


def test_kmtest_payload_failure_disables_the_ratchet_it_does_not_break_the_build(tmp_path):
    """No toolchain means we cannot build the exe — and must NOT stamp the payload
    anyway, which is what failed the livecd media step for the whole batch."""
    sh = tmp_path / "tools" / "payloads" / "kmtest"
    sh.mkdir(parents=True)
    (sh / "build.sh").write_text("#!/bin/bash\n")
    ok, why = gate_mod._ensure_kmtest_payload(tmp_path, ["i386"])
    assert ok is False
    assert "llvm-mingw" in why, f"must name the missing toolchain, got {why!r}"


def test_kmtest_payload_already_present_runs_nothing(tmp_path, monkeypatch):
    sh = tmp_path / "tools" / "payloads" / "kmtest"
    sh.mkdir(parents=True)
    (sh / "build.sh").write_text("#!/bin/bash\n")
    (sh / "kmtest-sweep-arm64.exe").write_text("x")

    def _boom(*a, **k):
        raise AssertionError("must not shell out when the payload is already there")

    monkeypatch.setattr(gate_mod.subprocess, "run", _boom)
    assert gate_mod._ensure_kmtest_payload(tmp_path, ["arm64"]) == (True, "")


def test_cli_main_prints_the_exit_code_even_when_a_pipeline_swallows_it(capsys):
    """The number a `| tail` hides must still be IN the output it shows."""
    from tests.qemu.cli.__main__ import cli_main

    rc = cli_main(["nosuchcommand"])
    out = capsys.readouterr().out
    assert rc == 2, "argparse usage errors must come back as a value, not escape as SystemExit"
    assert out.strip().splitlines()[-1] == "vibeos-cli: exit=2"


def test_cli_main_reports_a_crash_rather_than_leaving_the_reader_a_clean_tail(capsys):
    """A traceback goes to stderr; a pipe reading stdout must still see a nonzero code."""
    import tests.qemu.cli.__main__ as cli

    def boom(_argv=None):
        raise RuntimeError("kaboom")

    original = cli._cli_exit_code
    cli._cli_exit_code = boom
    try:
        with pytest.raises(RuntimeError):
            cli.cli_main([])
    finally:
        cli._cli_exit_code = original
    assert capsys.readouterr().out.strip().splitlines()[-1] == "vibeos-cli: exit=1"


def test_no_baselined_unpruned_walk_runs_in_the_early_partition() -> None:
    """The property that matters is narrower than the one the ratchet gates, and nothing held it.

    `scan_unpruned_walk` gates "no NEW unpruned walk anywhere" and carries the latent ones in a
    baseline while they burn down. But an unpruned walk is only a live hazard when its scanner
    runs CONCURRENTLY with the builds — that is the whole history: six audited scanners were
    correct for years and became reachable the day `source-early` moved them to t=0, without a
    line of their code changing. Today no baselined file is in EARLY_SOURCE_CHECKS, so the real
    invariant holds — but it holds by accident of which sites happened to be migrated first.

    The ratchet cannot protect it. Promoting an already-baselined scanner into the early
    partition arms it and adds no NEW row, so the lint stays green through exactly the move
    that re-creates the original incident. This is the guard for that move: migrate the walk to
    tools/source_walk.py FIRST, then promote it.
    """
    from tests.qemu.cli.lints import EARLY_SOURCE_CHECKS

    repo = Path(__file__).resolve().parents[3]
    baseline = repo / "tools" / "lint-baseline" / "scan_unpruned_walk.txt"
    if not baseline.is_file():
        pytest.skip("no scan_unpruned_walk baseline in this worktree")
    rows = [ln for ln in baseline.read_text().splitlines() if ln.strip() and not ln.startswith("#")]
    assert rows, "empty baseline — this guard would be measuring nothing"
    latent = {Path(ln.split(":", 1)[0]).stem for ln in rows}
    armed = sorted(latent & set(EARLY_SOURCE_CHECKS))
    assert not armed, (
        "these scanners run beside the builds AND still have an unpruned walk on the "
        f"baseline, so the walk is live: {armed}. Migrate to tools/source_walk.py "
        "source_files() before promoting a scanner into EARLY_SOURCE_CHECKS — a baselined "
        "row does not fire the ratchet, so this move is silent."
    )


def test_every_tools_test_file_is_actually_collected_by_something() -> None:
    """A test suite nobody runs is worse than no suite: it reads as coverage.

    Three separate files had sat unrun in `tools/` before anyone noticed, and the newest of
    them was the oracle for the checker that gates the merge train. The registry is an
    explicit list on purpose — a suite may legitimately need a build, or be too slow for the
    tooling gate — so this does not auto-collect. It just makes leaving one out a DECISION
    somebody has to write down.
    """
    from tests.qemu.cli.tooling import _PYTEST_SUITES

    repo = Path(__file__).resolve().parents[3]
    registered = {t if p == "." else f"{p}/{t}" for p, _pp, t in _PYTEST_SUITES}
    exempt: dict[str, str] = {}  # path -> why; keep the reason with the exemption
    found = {f"tools/{p.name}" for p in (repo / "tools").glob("test_*.py")}
    assert found, "no tools/test_*.py found at all — this guard is measuring nothing"
    missing = sorted(found - registered - set(exempt))
    assert not missing, (
        "these tools/ suites are collected by nothing; add them to _PYTEST_SUITES "
        f"or give each an `exempt` entry saying why: {missing}"
    )


def test_same_second_runs_keep_separate_artifacts(tmp_path, monkeypatch):
    from concurrent.futures import ThreadPoolExecutor

    from . import artifacts, conformance

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path)
    monkeypatch.setattr(artifacts.time, "time", lambda: 1700000000)

    def create(i):
        directory = Run("conformance", "arm64").dir if i % 2 else conformance._run_dir("arm64")
        (directory / "summary.json").write_text(json.dumps({"owner": i}))
        return directory

    with ThreadPoolExecutor(max_workers=8) as pool:
        directories = list(pool.map(create, range(32)))
    assert len(set(directories)) == 32
    assert all(d.name.endswith("-conformance-arm64") for d in directories)
    assert [json.loads((d / "summary.json").read_text())["owner"] for d in directories] == list(
        range(32)
    )


# --- MP-default beta: the two flakes the fleet agreed to eat, and the evidence
# trail that has to survive them. Both halves are pinned here because both have
# already failed silently once: a signature that was too broad masked real bugs
# for months (retired arm64_lpcexitthread_race), and three captures were lost to
# /tmp retention when nothing preserved them.


def _flake_names():
    from .triage import KNOWN_FLAKES

    return {f["name"] for f in KNOWN_FLAKES}


def test_mp_beta_flake_signatures_are_registered():
    assert {"arm64_smp39_zero_pc_fetch_abort", "i386_mp_win32k_av_0x1e"} <= _flake_names()


def test_smp39_signature_needs_the_WHOLE_shape_not_just_the_esr():
    """The SEH tests print an ESR on PURPOSE and pass.

    `[UEXC] ESR=0x92000047` is a deliberate, passing user-mode SEH probe, and an
    earlier draft that keyed on ESR alone would have swallowed unrelated reds.
    A kernel fault with a NON-zero PC is also not #39.
    """
    from .triage import _match_known_flake

    real = (
        "[KEXC] Code=0xFFFFFFFFC0000005[KEXC] PC=0x0000000000000000"
        "[KEXC] Spsr=0x0000000020000345[KEXC] ESR=0x0000000086000007\n"
        "*** BUGCHECK: 0x0000001E (0xffffffffc0000005, 0x0000000000000000, 0x0, 0x0)\n"
    )
    hit = _match_known_flake(real, "0x0000001E", "bugcheck")
    assert hit is not None and hit["name"] == "arm64_smp39_zero_pc_fetch_abort"

    # A passing SEH probe plus an UNRELATED 0x1E must still block.
    seh_plus_other = (
        "[UEXC] ESR=0x0000000092000047[SEH-USER] hw-fault: caught=1 (PASS)\n"
        "*** BUGCHECK: 0x0000001E (0xffffffffc0000005, 0xfffff80042190000, 0x0, 0x0)\n"
    )
    assert _match_known_flake(seh_plus_other, "0x0000001E", "bugcheck") is None

    # Same fault class, NON-zero PC -> a different bug, must not be masked.
    nonzero_pc = real.replace("PC=0x0000000000000000", "PC=0xFFFF8000050E08A0")
    assert _match_known_flake(nonzero_pc, "0x0000001E", "bugcheck") is None


def test_win32k_signature_is_pinned_to_one_rva():
    """A different win32k fault is a different bug and must still go red."""
    from .triage import _match_known_flake

    mine = "*** BUGCHECK: 0x0000001E (0xc0000005, ...)\n***   P2 = win32k.sys+0x65EE1\n"
    other = "*** BUGCHECK: 0x0000001E (0xc0000005, ...)\n***   P2 = win32k.sys+0x12345\n"
    hit = _match_known_flake(mine, "0x0000001E", "bugcheck")
    assert hit is not None and hit["name"] == "i386_mp_win32k_av_0x1e"
    assert _match_known_flake(other, "0x0000001E", "bugcheck") is None


def test_failed_boot_preserves_serial_and_pins_the_kernel(tmp_path, monkeypatch):
    """Preserving a verdict without the serial and the matching symbol index is
    how a flake becomes unreadable lore. Both are part of the contract."""
    from . import artifacts

    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    repo = tmp_path / "repo"
    (repo / "build-arm64" / "ntoskrnl" / "ntkrnlmp").mkdir(parents=True)
    (repo / "build-arm64" / "ntoskrnl" / "ntkrnlmp" / "ntkrnlmp.exe").write_bytes(b"KERNEL")
    (repo / "qemu").mkdir(parents=True)
    (repo / "qemu" / "symbols-arm64-mp.tsv").write_text("0000\tKiFoo\t-\n")

    run = tmp_path / "20260912-000000-abcd-smoke-arm64"
    run.mkdir()
    (run / "arm64_serial.log").write_text("[FLDR] load ntkrnlmp.exe\n*** BUGCHECK: 0x0000001E\n")
    (run / "summary.json").write_text("{}")

    kept = artifacts.preserve_failure(run, "arm64", "BUGCHECK 0x1E", repo)
    assert kept, "a red boot must leave evidence behind"
    dst = Path(kept)
    assert (dst / "arm64_serial.log").is_file(), "the serial IS the evidence"
    manifest = json.loads((dst / "manifest.json").read_text())
    assert manifest["mp_kernel"] is True
    # Pinned by content hash, so the serial stays symbolicable after a rebuild.
    kdir = Path(manifest["kernels_dir"])
    assert (kdir / "ntkrnlmp.exe").read_bytes() == b"KERNEL"
    assert (kdir / "symbols-arm64-mp.tsv").is_file()


def test_preserving_the_same_build_twice_copies_the_kernel_once(tmp_path, monkeypatch):
    """Content-addressed on purpose: a 1.7% flake across a busy fleet must not
    grow the farm by a kernel image per occurrence."""
    from . import artifacts

    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    repo = tmp_path / "repo"
    (repo / "build-arm64" / "ntoskrnl" / "ntkrnlmp").mkdir(parents=True)
    (repo / "build-arm64" / "ntoskrnl" / "ntkrnlmp" / "ntkrnlmp.exe").write_bytes(b"KERNEL")

    seen = []
    for i in range(3):
        run = tmp_path / f"run{i}"
        run.mkdir()
        (run / "arm64_serial.log").write_text("ntkrnlmp.exe\n")
        kept = artifacts.preserve_failure(run, "arm64", "BUGCHECK 0x1E", repo)
        assert kept
        seen.append(json.loads((Path(kept) / "manifest.json").read_text())["kernels_dir"])

    assert len(set(seen)) == 1, "same build must resolve to one parked copy"
    assert len(list((tmp_path / "farm" / "kernels").iterdir())) == 1
    assert len(list((tmp_path / "farm" / "fails" / "auto").iterdir())) == 3


def test_a_non_boot_run_preserves_nothing(tmp_path, monkeypatch):
    """finalize() runs for `build` and `check` too; those have no serial and
    must not create empty farm entries."""
    from . import artifacts

    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    run = tmp_path / "build-run"
    run.mkdir()
    (run / "full.log").write_text("link error\n")
    assert artifacts.preserve_failure(run, "arm64", "BUILD_FAIL", None) is None
    assert not (tmp_path / "farm").exists()


def test_a_red_run_actually_reaches_the_preserve_path(tmp_path, monkeypatch):
    """finalize() must CALL preserve_failure, not merely be able to.

    The unit tests above prove the helper works in isolation; this one pins the
    wiring, because the two failure modes are indistinguishable from outside. An
    unwired preserve path produces an empty fails/auto/ directory, and an empty
    fails/auto/ is exactly what a healthy fleet produces too -- so the bug would
    present as good news and stay hidden until someone needed a capture.
    """
    from . import artifacts

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    seen = {}

    def spy(run_dir, arch, classification, repo):
        seen["called"] = (arch, classification)
        return str(tmp_path / "farm" / "fails" / "auto" / "x")

    monkeypatch.setattr(artifacts, "preserve_failure", spy)

    run = artifacts.Run("smoke", "arm64")
    run.verdict = "FAIL"
    run.classification = "BUGCHECK 0x1E"
    run.finalize(None)
    assert seen.get("called") == ("arm64", "BUGCHECK 0x1E"), "a red boot bypassed preservation"
    assert json.loads((run.dir / "summary.json").read_text())["preserved"]


def test_a_green_run_preserves_nothing(tmp_path, monkeypatch):
    """Preserving on success would bury the real failures in noise."""
    from . import artifacts

    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    monkeypatch.setattr(artifacts, "FARM_DIR", tmp_path / "farm")
    called = []
    monkeypatch.setattr(artifacts, "preserve_failure", lambda *a: called.append(a))

    run = artifacts.Run("smoke", "arm64")
    run.verdict = "OK"
    run.finalize(None)
    assert not called


def test_default_boot_now_asserts_the_mp_kernel(monkeypatch):
    """Since the MP promotion the DEFAULT media boots ntkrnlmp.exe, so a plain
    smoke must catch a silent revert to UP.

    A boot that reaches the desktop on the wrong kernel is indistinguishable
    from a correct one — screenshot included — so if this check only ran under
    --smp, a regression flipping DefaultOS= back would be green everywhere.
    """
    from . import smoke as smoke_mod

    monkeypatch.delenv("VIBEOS_QEMU_SMP", raising=False)
    up = "[FLDR] load \\reactos\\ntoskrnl.exe\n[FLDR] load \\reactos\\hal.dll\n"
    bad = smoke_mod._smp_check(up, "arm64")
    assert bad and bad["classification"] == "UP_KERNEL_ON_SMP_RUN"
    assert "the default boot" in bad["salient"][0], bad["salient"]

    mp = "[FLDR] load \\reactos\\ntkrnlmp.exe\n[FLDR] load \\reactos\\halmp.dll\n"
    assert smoke_mod._smp_check(mp, "arm64") is None


def test_default_boot_does_not_demand_ap_liveness(monkeypatch):
    """One CPU has no APs; the liveness check must not fire on a default boot."""
    from . import smoke as smoke_mod

    monkeypatch.delenv("VIBEOS_QEMU_SMP", raising=False)
    mp_one_cpu = "[FLDR] load \\reactos\\ntkrnlmp.exe\n[FLDR] load \\reactos\\halmp.dll\n"
    assert smoke_mod._smp_check(mp_one_cpu, "amd64") is None


def test_mp_kernel_on_a_up_hal_is_still_caught_on_a_default_boot(monkeypatch):
    """The MP kernel on the UP HAL boots to a desktop and silently runs 1 CPU —
    arm64 has no MISMATCHED_HAL handshake to catch it."""
    from . import smoke as smoke_mod

    monkeypatch.delenv("VIBEOS_QEMU_SMP", raising=False)
    text = "[FLDR] load \\reactos\\ntkrnlmp.exe\n[FLDR] load \\reactos\\hal.dll\n"
    bad = smoke_mod._smp_check(text, "arm64")
    assert bad and bad["classification"] == "SMP_UP_HAL"


def test_the_up_kernel_is_opt_in_only_and_nothing_automated_opts_in():
    """UP is a dead branch: no automated path may select it.

    The polarity was inverted on 2026-09-12 -- `--smp-kernel` (opt IN to MP) became
    `--up-kernel` (opt OUT to UP), so the flavor a gate/smoke/conformance run gets
    is MP unless a human deliberately asks otherwise. This pins both halves,
    because the failure is silent in both directions: a run that boots the wrong
    kernel reaches the desktop and looks identical to a correct one, and the old
    flag name surviving anywhere would be a no-op that reads as still working.
    """
    import subprocess
    import sys

    repo = Path(__file__).resolve().parents[3]
    # The opt-out exists and is spelled --up-kernel.
    help_text = subprocess.run(
        [sys.executable, "-m", "tests.qemu.cli", "build", "--help"],
        cwd=repo,
        capture_output=True,
        text=True,
        timeout=120,
    ).stdout
    assert "--up-kernel" in help_text, help_text[:400]
    assert "--smp-kernel" not in help_text, "the inverted flag name survived"

    # And no source file still reads the retired knob (docs may discuss history).
    # Assembled at runtime so this file does not match its OWN pattern -- spelling
    # the names literally made the guard report itself and nothing else.
    pat = "smp" + "_kernel|VIBEOS_SMP_" + "DEFAULT_BOOT"
    stale = subprocess.run(
        ["git", "-C", str(repo), "grep", "-lE", pat, "--", "*.py", "*.sh", "*.cmake", "*.txt"],
        capture_output=True,
        text=True,
        timeout=60,
    ).stdout.split()
    # gcc.cmake keeps a historical note about the FORMER env var; it reads nothing.
    stale = [f for f in stale if f != "sdk/cmake/gcc.cmake"]
    assert not stale, f"these still reference the retired MP opt-in knob: {stale}"


def test_ledger_records_the_kernel_it_OBSERVED_not_the_flag():
    """`mp_kernel` must come from the serial, never from `--smp`.

    It was `bool(smp)`, which was accurate only while `--smp N` was the sole way
    to get MP media. Since the MP promotion a DEFAULT boot runs ntkrnlmp.exe with
    smp unset, so the flag-derived value recorded every default MP round as UP --
    and `mp_kernel` is the field every flake-rate query filters on, so the damage
    is silent and retroactive: the rounds still happen, they just stop counting.

    The same inference-as-observation bug already cost an A/B once (see the note
    beside the field), which is why this is pinned rather than left to review.
    """
    import inspect

    from . import conformance as conf

    src = inspect.getsource(conf)
    assert '"mp_kernel": "ntkrnlmp.exe" in serial_text' in src, (
        "mp_kernel must be read off the boot's serial"
    )
    assert '"mp_kernel": bool(smp)' not in src, "mp_kernel regressed to the flag"


def test_fatal_scanner_diagnostic_cannot_be_baselined(tmp_path, monkeypatch):
    cp = subprocess.CompletedProcess(
        ["scanner"], 1, "header already baselined\n", "FATAL: missing generated headers\n"
    )
    monkeypatch.setattr(lints, "_baseline", lambda *_: {"header already baselined"})
    run = Run("check", "all")
    stage = lints._lint_stage(run, tmp_path, "scanner", ["scanner"], strict=True, cp=cp)
    assert stage.status == "fail"
    assert run.classification == "SCANNER_ERROR"


def test_build_source_lint_is_opt_in():
    from .__main__ import build_parser

    parser = build_parser()
    assert not parser.parse_args(["build", "--arch", "amd64"]).check
    assert parser.parse_args(["build", "--arch", "amd64", "--check"]).check
    assert not parser.parse_args(["build", "--arch", "amd64", "--no-check"]).check


def test_hostlock_large_host_admits_the_full_arch_matrix(tmp_path, monkeypatch):
    monkeypatch.setattr(hl_mod, "HOSTLOCK_DIR", tmp_path)
    monkeypatch.setattr(os, "sched_getaffinity", lambda pid: set(range(24)))
    monkeypatch.delenv("VIBEOS_MAX_CONCURRENT_BUILDS", raising=False)
    assert hl_mod.build_permits(4) == 1


def test_clean_discovers_isolated_cargo_links_only_for_requested_arch(tmp_path):
    root = tmp_path / "sdk/lib/rust/target"
    x86 = root / "sample-cargo-amd64-cargo-link/x86_64-pc-windows-gnu"
    arm = root / "sample-cargo-arm64-cargo-link/aarch64-pc-windows-gnullvm"
    x86.mkdir(parents=True)
    arm.mkdir(parents=True)
    assert provenance.workspace_cargo_targets(tmp_path, "amd64") == [x86]
    assert provenance.workspace_cargo_targets(tmp_path, "arm64") == [arm]


def test_kill_fails_when_explicit_pids_match_no_tracked_vm(tmp_path, monkeypatch, capsys):
    """`vibeos-cli kill <pid>` used to report OK when the pid matched nothing.

    Measured: a QEMU launched outside start.sh/vm.py has no lockfile, so it is
    invisible to running_vms(); two kill calls 29 minutes apart both reported
    success while it held 94% of a core. A sweep matching nothing is fine --
    `--stale` with no orphans is the normal case -- but a caller who NAMED
    processes and had none stopped must not be told it worked.
    """
    from . import killvm

    monkeypatch.setattr(killvm, "running_vms", lambda: [])

    class _A:
        pids: ClassVar[list[int]] = [4242]
        arch = None

    assert killvm.cmd_kill(_A(), tmp_path) != 0
    assert "NOT killed" in capsys.readouterr().out


def test_kill_still_succeeds_when_a_SWEEP_matches_nothing(tmp_path, monkeypatch):
    """The other half of the distinction: `--stale` with no orphans is success."""
    from . import killvm

    monkeypatch.setattr(killvm, "running_vms", lambda: [])

    class _A:
        pids: ClassVar[list[int]] = []
        arch = None
        stale = True

    assert killvm.cmd_kill(_A(), tmp_path) == 0


def test_kill_reports_a_zombie_as_exited_not_as_unkillable(tmp_path, monkeypatch):
    """os.kill(pid, 0) SUCCEEDS on a zombie — the pid is still in the table — so
    the liveness check read a QEMU that had already died on the SIGTERM as alive
    and escalated to a "STILL ALIVE after SIGTERM+SIGKILL — unkillable (D state?)"
    report. That is a worse lie than the silent success the escalation was added
    to fix: it sends the reader hunting a process that does not exist."""
    from tests.qemu.cli import killvm

    proc = tmp_path / "4242"
    proc.mkdir()
    (proc / "status").write_text("Name:\tqemu\nState:\tZ (zombie)\nPid:\t4242\n")
    monkeypatch.setattr(killvm, "Path", lambda p="": tmp_path if str(p) == "/proc" else Path(p))
    assert killvm._pid_is_zombie(4242)

    (proc / "status").write_text("Name:\tqemu\nState:\tR (running)\nPid:\t4242\n")
    assert not killvm._pid_is_zombie(4242)


def test_zombie_check_is_false_where_proc_is_absent(tmp_path, monkeypatch):
    """No /proc: fall back to the pid check, which is what it did before."""
    from tests.qemu.cli import killvm

    monkeypatch.setattr(killvm, "Path", lambda p="": tmp_path / "nope")
    assert not killvm._pid_is_zombie(1)


def test_qmp_client_keeps_events_instead_of_discarding_them():
    """_recv_json must skip events to find a reply, and used to drop them on the
    way past. That threw away STOP, which is the only notification QEMU sends when
    a replay-break fires — leaving a paused replay indistinguishable from a wedged
    one, since both show a frozen serial log and a busy CPU."""
    from tests.qemu.qmp_client import QmpClient

    c = QmpClient()
    feed = [
        {"event": "RESUME"},
        {"return": {}},
        {"event": "STOP", "timestamp": {"seconds": 7}},
        {"return": {"icount": 42}},
    ]
    c._recv_one_json = lambda: feed.pop(0)  # type: ignore[method-assign]

    assert c._recv_json() == {"return": {}}
    assert c._recv_json() == {"return": {"icount": 42}}
    assert [e["event"] for e in c.events] == ["RESUME", "STOP"]
    assert c.saw_event("STOP") == {"event": "STOP", "timestamp": {"seconds": 7}}
    assert c.saw_event("SHUTDOWN") is None


def test_qmp_event_retention_is_bounded():
    """A long boot emits plenty; this is a diagnostic tail, not a log."""
    from tests.qemu.qmp_client import QmpClient

    c = QmpClient()
    feed = [{"event": f"E{i}"} for i in range(300)] + [{"return": {}}]
    c._recv_one_json = lambda: feed.pop(0)  # type: ignore[method-assign]
    c._recv_json()
    assert len(c.events) == 256
    assert c.saw_event("E299") is not None
    assert c.saw_event("E0") is None  # fell off the tail


@pytest.mark.parametrize("arch", ["amd64-uefi-net", "i386-uefi-net", "amd64-test"])
def test_per_arch_cap_includes_variants(arch, monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_PER_ARCH_CAP", "2")
    monkeypatch.delenv("VIBEOS_QEMU_CAP_AMD64", raising=False)
    monkeypatch.delenv("VIBEOS_QEMU_CAP_I386", raising=False)
    assert vm_mod.per_arch_cap(arch) == 2


@pytest.mark.parametrize(
    "arch,available,expected",
    [
        ("amd64-uefi", True, "whpx"),
        ("amd64-uefi", False, "tcg"),
        ("arm64", True, "tcg"),
    ],
)
def test_start_resolves_accel_before_admission(arch, available, expected, monkeypatch):
    v = _bare_vm(arch)
    v._backend = "windows-host"
    v._win = {"accel": "whpx"}
    v._win_accel = "tcg"
    monkeypatch.setattr(be_mod, "whpx_available", lambda: available)
    observations = []
    monkeypatch.setattr(v, "_acquire_slot", lambda: observations.append(v._win_accel))
    monkeypatch.setattr(v, "_acquire_cpu_lease", lambda: observations.append(v._boot_is_tcg()))
    monkeypatch.setattr(vm_mod, "launch_lock", contextlib.nullcontext)
    monkeypatch.setattr(v, "_prepare_launch", lambda: None)
    monkeypatch.setattr(v, "_start_with_ports", lambda: None)
    v.start()
    assert observations == [expected, expected == "tcg"]


def test_boot_eof_preserves_serial_and_reports_process_exit(monkeypatch, tmp_path):
    from types import SimpleNamespace

    v = _bare_vm("amd64")
    v.config = vm_mod.get_arch_config("amd64-uefi")

    def eof(*args, **kwargs):
        raise ConnectionError("Serial socket closed")

    monkeypatch.setattr(
        v,
        "serial",
        SimpleNamespace(read_until=eof, partial_output="firmware breadcrumb"),
        raising=False,
    )
    monkeypatch.setattr(
        v,
        "_proc",
        SimpleNamespace(poll=lambda: 1, wait=lambda **_: 1, returncode=1),
        raising=False,
    )
    v._stderr_path = tmp_path / "stderr.log"
    (tmp_path / "stderr.log").write_text("WHPX fatal error")
    v._early_stderr = ""
    with pytest.raises(RuntimeError, match="QEMU exited with code 1") as error:
        v.wait_for_boot()
    assert "WHPX fatal error" in str(error.value)
    assert v._boot_output == "firmware breadcrumb"


def test_qemu_stderr_cannot_fill_a_pipe_and_stall(tmp_path, monkeypatch):
    import sys

    v = _launch_vm("native", tmp_path)
    monkeypatch.setattr(
        v,
        "_build_native_cmd",
        lambda: [sys.executable, "-c", "import sys; sys.stderr.write('x' * 131072)"],
    )
    v._launch_qemu()
    assert v._proc is not None
    try:
        assert v._proc.wait(timeout=5) == 0
        assert (tmp_path / "amd64-uefi_qemu.stderr.log").stat().st_size == 131072
        assert v._read_qemu_stderr() == "x" * 65536
    finally:
        if v._proc.poll() is None:
            v._proc.kill()
            v._proc.wait()


def test_serial_eof_retains_partial_output():
    client = serial_client.SerialClient()
    client._buf = "firmware reached the loader"
    client._sock_dead = True
    with pytest.raises(ConnectionError):
        client.read_until("shell ready", timeout=0.1)
    assert client.partial_output == "firmware reached the loader"


# --- land: the SOURCE-lint preflight ---------------------------------------
#
# Added 2026-09-19 after one lane ate two LINT_FAIL rejections for a one-line
# `scan_unpruned_walk` fix. `_lint_preflight` is named "lint" but only ever ran
# check-tooling; the conductor also runs `check --strict --scope source-early|
# source-deferred`, and plain `./vibeos-cli check` does NOT run those in strict mode, so
# a lane can be green locally and rejected after the full queue wait. These are pure
# source checks — no build, no boot — so they belong before the cost, not after it.


def _src_lint_run(tmp_path: Path):
    """A real `Run` with its artifact dir redirected — the preflight takes a Run, and a
    stand-in class would only prove the stand-in's interface."""
    run = Run("land", "host")
    run.dir = tmp_path
    run.stages = []
    return run


class _SrcLintArgs:
    skip_lint = False


def _stub_src_lint_once(land_mod_, monkeypatch, results: dict):
    import tests.qemu.cli.gate as gate_mod

    monkeypatch.setattr(gate_mod, "_lint_once", lambda repo, part, root: results[part])


def _src_lint_result(ok: bool, classification: str = "", salient: str = "", exit_code: int = 0):
    return {
        "arch": "source-deferred",
        "ok": ok,
        "seconds": 1.0,
        "salient": salient,
        "exit": exit_code,
        "classification": classification,
    }


def test_source_lint_preflight_passes_when_both_partitions_are_clean(tmp_path, monkeypatch):
    run = _src_lint_run(tmp_path)
    _stub_src_lint_once(
        land_mod,
        monkeypatch,
        {"source-early": _src_lint_result(True), "source-deferred": _src_lint_result(True)},
    )
    land_mod._source_lint_preflight(tmp_path, run, _SrcLintArgs())
    assert [s.status for s in run.stages] == ["ok", "ok"]


def test_source_lint_preflight_blocks_on_real_new_findings(tmp_path, monkeypatch):
    """The case that cost two rejections: a genuine NEW finding must stop the submission."""
    run = _src_lint_run(tmp_path)
    _stub_src_lint_once(
        land_mod,
        monkeypatch,
        {
            "source-early": _src_lint_result(True),
            "source-deferred": _src_lint_result(
                False,
                classification="LINT_FAIL",
                salient="✗ scan_unpruned_walk(2 new)",
                exit_code=1,
            ),
        },
    )
    with pytest.raises(land_mod.LandAbort) as ei:
        land_mod._source_lint_preflight(tmp_path, run, _SrcLintArgs())
    msg = str(ei.value)
    assert "scan_unpruned_walk" in msg
    # It must name the reproduction, because a green plain `check` is the trap here.
    assert "--strict --scope source-deferred" in msg


def test_source_lint_preflight_fails_OPEN_on_a_scanner_error(tmp_path, monkeypatch):
    """NEVER MORE STRICT THAN THE GATE. source-deferred holds verify_signatures and
    gen_ctype_tables, which hard-fail without this worktree's build-<arch> — a fresh
    lane legitimately has none, and the conductor's tree does. A partition that could
    not RUN here must warn, not block, or this preflight wedges landing."""
    run = _src_lint_run(tmp_path)
    _stub_src_lint_once(
        land_mod,
        monkeypatch,
        {
            "source-early": _src_lint_result(True),
            "source-deferred": _src_lint_result(
                False, classification="SCANNER_ERROR", salient="no build-amd64", exit_code=1
            ),
        },
    )
    land_mod._source_lint_preflight(tmp_path, run, _SrcLintArgs())  # must NOT raise
    assert [s.status for s in run.stages] == ["ok", "warn"]


def test_source_lint_preflight_honours_skip_lint(tmp_path, monkeypatch):
    run = _src_lint_run(tmp_path)

    def _boom(*a, **kw):
        raise AssertionError("must not run under --skip-lint")

    import tests.qemu.cli.gate as gate_mod

    monkeypatch.setattr(gate_mod, "_lint_once", _boom)
    args = _SrcLintArgs()
    args.skip_lint = True
    land_mod._source_lint_preflight(tmp_path, run, args)
    assert run.stages == []


def _with_origin_master(repo: Path, *paths: str) -> None:
    """Give `repo` an origin/master and a commit on top touching exactly `paths`.

    The shared predicate is installed into the BASE, because `_docs_only_vs_master` loads
    it from `<repo>/tools/docs_only.py` by path. In the base, not the delta: a predicate
    committed alongside the change would itself be a non-inert path.
    """
    tools = repo / "tools"
    tools.mkdir(parents=True, exist_ok=True)
    real = Path(__file__).resolve().parents[3] / "tools" / "docs_only.py"
    (tools / "docs_only.py").write_text(real.read_text())
    _git(repo, "add", "-A")
    _git(repo, "commit", "-qm", "install the shared docs-only predicate")
    _git(repo, "update-ref", "refs/remotes/origin/master", "HEAD")
    for rel in paths:
        target = repo / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("content\n")
    _git(repo, "add", "-A")
    _git(repo, "commit", "-qm", "lane work")


def test_lint_preflight_SKIPS_a_docs_only_delta(tiny_repo: Path, tmp_path, monkeypatch):
    """A Markdown-only change cannot fail a Python linter, so it must not pay for one.

    This is not a style preference: the commit that recorded the orphan inventory was
    refused by `test_verify_ucrtbase_cohab.py` over a stale `build-i386`, for a delta of
    one `.md` file. The pre-push hook has had this exemption since it was written; the
    preflight did not. Safe because the CONDUCTOR still lints the merged batch — this
    declines to lint a lane that has no source, it does not let source through unlinted.
    """
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    _with_origin_master(tiny_repo, "docs/engineering/note.md", "README.md")
    _fake_cli(tiny_repo, rc=1, out="✗ everything")  # red, and must still not block
    run = Run("land", "t")
    land_mod._lint_preflight(tiny_repo, run, _Args())  # must not raise
    assert any(s.name == "lint" and s.status == "skip" for s in run.stages)


def test_lint_preflight_does_NOT_skip_when_one_source_file_rides_along(
    tiny_repo: Path, tmp_path, monkeypatch
):
    """The dangerous direction. A delta that is *mostly* docs is not docs-only, and one
    `.py` beside forty `.md` must still be linted — otherwise the exemption becomes a way
    to smuggle source past the cheap check."""
    monkeypatch.setattr("tests.qemu.cli.artifacts.ARTIFACT_ROOT", tmp_path / "art")
    _with_origin_master(tiny_repo, "docs/a.md", "docs/b.md", "tools/thing.py")
    _fake_cli(tiny_repo, rc=1, out="  ✗ ruff:tools/thing.py\n\nFAIL — 1/15 checks failed:")
    with pytest.raises(land_mod.LandAbort):
        land_mod._lint_preflight(tiny_repo, Run("land", "t"), _Args())


def test_docs_only_predicate_fails_CLOSED_without_origin_master(tiny_repo: Path):
    """No origin/master (a fresh clone, a renamed remote) must mean "lint normally", never
    "skip". A predicate that cannot answer has to say False."""
    assert land_mod._docs_only_vs_master(tiny_repo) is False


def test_docs_only_predicate_ignores_what_master_did_meanwhile(tiny_repo: Path):
    """Three-dot, so a lane merely BEHIND master is judged on what IT changed. Two-dot
    would drag every landed source file into this lane's delta and never skip."""
    _with_origin_master(tiny_repo, "docs/mine.md")
    # the module-level `_git` helper returns None; capture the sha ourselves
    lane = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=tiny_repo, capture_output=True, text=True, check=True
    ).stdout.strip()
    _git(tiny_repo, "checkout", "-q", "refs/remotes/origin/master")
    (tiny_repo / "tools" / "other.py").parent.mkdir(parents=True, exist_ok=True)
    (tiny_repo / "tools" / "other.py").write_text("x = 1\n")
    _git(tiny_repo, "add", "-A")
    _git(tiny_repo, "commit", "-qm", "someone else's source landed")
    _git(tiny_repo, "update-ref", "refs/remotes/origin/master", "HEAD")
    _git(tiny_repo, "checkout", "-q", lane)
    assert land_mod._docs_only_vs_master(tiny_repo) is True


# --- launch retry (host-side socket flake, 2026-09-20) ----------------------


class _FlakyVM:
    """Stands in for QemuVM: fails its first `fail_times` starts, then works."""

    def __init__(self, fail_times: int):
        self._left = fail_times
        self.launch_timings: dict[str, float] = {}
        self.starts = 0
        self.stops = 0

    def start(self):
        self.starts += 1
        if self._left > 0:
            self._left -= 1
            raise TimeoutError("QEMU did not connect to serial socket within 15.0s")

    def stop(self):
        self.stops += 1


def _retry_probe(monkeypatch, failures: int, attempts: int = 3):
    """Drive _launch_with_retry against a VM that fails `failures` times."""
    from tests.qemu.cli import smoke as smoke_mod

    monkeypatch.setattr(smoke_mod, "_LAUNCH_ATTEMPTS", attempts)
    monkeypatch.setattr(smoke_mod, "_LAUNCH_RETRY_SECONDS", 0)
    budget = {"n": failures}
    built: list[_FlakyVM] = []
    logs: list[str] = []

    def make_vm():
        fail = 1 if budget["n"] > 0 else 0
        budget["n"] -= 1
        v = _FlakyVM(fail)
        built.append(v)
        return v

    timings: dict[str, float] = {}
    # _FlakyVM is a stand-in, not a QemuVM -- the helper only ever touches
    # start/stop/launch_timings, which is exactly what makes it testable here.
    vm, n, exc = smoke_mod._launch_with_retry(
        cast("Any", make_vm), timings, logs.append, "boot 1/5"
    )
    return built, logs, timings, vm, n, exc


def test_launch_retry_recovers_from_a_host_side_flake(monkeypatch):
    # The incident: one boot of five died on "QEMU did not connect to serial
    # socket within 15.0s" before a single guest instruction ran, and landed as
    # a red flake beside real verdicts.
    built, logs, timings, _vm, n, exc = _retry_probe(monkeypatch, failures=1)
    assert len(built) == 2
    assert exc is None, "a single host-side flake should not fail the boot"
    assert n == 2
    assert timings["launch_attempts"] == 2, "a recovered launch must still be recorded"
    assert any("host-side flake" in m for m in logs)


def test_launch_retry_builds_a_FRESH_vm_each_attempt(monkeypatch):
    # Load-bearing: the port and serial socket path are chosen in
    # QemuVM.__init__, so retrying start() on the SAME instance would retry onto
    # the same bad port and could never recover.
    built, _logs, _timings, _vm, n, exc = _retry_probe(monkeypatch, failures=2)
    assert exc is None and n == 3
    assert len(built) == 3, "each attempt must construct a new VM"
    assert all(v.starts == 1 for v in built), "no instance may be start()ed twice"
    assert built[0].stops == 1 and built[1].stops == 1, "failed VMs must be torn down"


def test_launch_retry_gives_up_and_reports_the_real_failure(monkeypatch):
    # The retry must not make a broken launch quieter or slower-to-diagnose.
    built, _logs, timings, _vm, n, exc = _retry_probe(monkeypatch, failures=99)
    assert isinstance(exc, TimeoutError)
    assert n == 3 and len(built) == 3
    assert "launch_attempts" not in timings, "never claim success on a failed launch"


def test_launch_retry_is_a_no_op_on_a_healthy_launch(monkeypatch):
    built, logs, timings, _vm, n, exc = _retry_probe(monkeypatch, failures=0)
    assert exc is None and n == 1 and len(built) == 1
    assert logs == [], "a clean launch must stay silent"
    assert "launch_attempts" not in timings


def test_launch_attempts_stays_small_enough_not_to_slow_a_broken_launch():
    from tests.qemu.cli import smoke as smoke_mod

    assert 2 <= smoke_mod._LAUNCH_ATTEMPTS <= 3
    assert smoke_mod._LAUNCH_RETRY_SECONDS <= 5


def test_launch_failure_still_reports_LAUNCH_FAIL_with_the_attempt_count():
    import inspect

    from tests.qemu.cli import smoke as smoke_mod

    src = inspect.getsource(smoke_mod._boot_vm)
    assert '"classification": "LAUNCH_FAIL"' in src
    assert '"launch_attempts": attempts' in src
    assert "launch attempts, all failed" in src
