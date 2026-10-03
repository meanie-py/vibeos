"""Guards on the record/replay command-line transform.

Every test here stands for a way a trace can be silently non-deterministic. That
is the failure mode worth paying for: a bad transform does not error, it produces
a recording that diverges halfway through a replay, and the divergence message
names an event number rather than the config that caused it. So the transform's
job is to REFUSE, and these tests hold it to that.
"""

from __future__ import annotations

import json
import struct
import subprocess
import time
from pathlib import Path

import pytest

from . import recordreplay as rr


def _base_cmd(tmp_path: Path) -> list[str]:
    """A cut-down arm64 command line with the shapes that actually appear:
    pflash firmware, an id'd `if=none` drive with a device pointing at it, and a
    bare `if=virtio` drive with no id at all."""
    (tmp_path / "QEMU_EFI.fd").write_bytes(b"\0" * 64)
    (tmp_path / "efivars.qcow2").write_bytes(b"\0" * 64)
    (tmp_path / "boot.img").write_bytes(b"\0" * 64)
    (tmp_path / "scratch.qcow2").write_bytes(b"\0" * 64)
    return [
        "qemu-system-aarch64",
        "-M",
        "virt",
        "-cpu",
        "cortex-a72",
        "-accel",
        "tcg,thread=multi",
        "-m",
        "1G",
        "-drive",
        f"file={tmp_path}/scratch.qcow2,format=qcow2,if=none,id=scratch",
        "-device",
        "virtio-blk-device,drive=scratch",
        "-drive",
        f"if=pflash,format=raw,file={tmp_path}/QEMU_EFI.fd,readonly=on",
        "-drive",
        f"if=pflash,format=qcow2,file={tmp_path}/efivars.qcow2",
        "-drive",
        f"file={tmp_path}/boot.img,format=raw,if=virtio,readonly=on,file.locking=off",
        "-device",
        "ramfb",
        "-display",
        "none",
    ]


def _drives(argv: list[str]) -> list[str]:
    return [argv[i + 1] for i, a in enumerate(argv) if a == "-drive"]


# --- the transform -------------------------------------------------------


def test_every_guest_drive_is_wrapped_in_blkreplay(tmp_path):
    """An unwrapped drive completes I/O in host time, so its completion lands in a
    different instruction slot on replay. One missed drive is enough to ruin a
    trace, and nothing reports it at record time."""
    out, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False)
    drives = [rr.parse_drive(d) for d in _drives(out)]
    guest = [d for d in drives if d.get("if") != "pflash"]
    backends = [d for d in guest if d.get("driver") != "blkreplay"]
    filters = [d for d in guest if d.get("driver") == "blkreplay"]
    assert len(filters) == 2, f"expected scratch + boot wrapped, got {filters}"
    # each filter names exactly one backend, and every backend is named
    assert {f["image"] for f in filters} == {b["id"] for b in backends}


def test_blkreplay_filter_keeps_the_id_the_device_refers_to(tmp_path):
    """`-device virtio-blk-device,drive=scratch` must keep resolving. If the filter
    took a new id and the backend kept `scratch`, QEMU would start fine and the
    device would be attached to the RAW image — i.e. an unrecorded drive, with no
    error anywhere."""
    out, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False)
    drives = [rr.parse_drive(d) for d in _drives(out)]
    by_id = {d["id"]: d for d in drives if "id" in d}
    assert by_id["scratch"]["driver"] == "blkreplay"
    assert by_id["scratch-img"]["file"].endswith("scratch.qcow2")
    assert "-device" in out and "virtio-blk-device,drive=scratch" in out


def test_pflash_is_repointed_but_not_wrapped(tmp_path):
    """pflash is memory-mapped, not an async block device — blkreplay is not a legal
    backend for it. It still has to come from the trace, or a replay reads firmware
    variables the recording never saw."""
    out, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False)
    pflash = [rr.parse_drive(d) for d in _drives(out) if "if=pflash" in d]
    assert len(pflash) == 2
    for d in pflash:
        assert d.get("driver") != "blkreplay"
        assert str(tmp_path / "t" / "media") in d["file"], d


def test_readonly_media_becomes_a_writable_overlay(tmp_path):
    """boot.img is mounted readonly in the normal harness. Under rr it must be
    writable-through-an-overlay: rrsnapshot needs somewhere to put the vmstate, and
    a readonly drive cannot take one."""
    out, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False)
    for d in (rr.parse_drive(x) for x in _drives(out)):
        assert "readonly" not in d, d


def test_mttcg_is_forced_to_single_threaded(tmp_path):
    """MTTCG has no recorded global instruction order across vCPU threads. arm64's
    config asks for thread=multi by default, so this is the live case, not a
    hypothetical."""
    out, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False)
    assert "tcg,thread=single" in out
    assert "tcg,thread=multi" not in out


def test_icount_is_appended_with_the_trace_file_and_snapshot(tmp_path):
    out, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False)
    ic = out[out.index("-icount") + 1]
    assert "rr=record" in ic
    assert f"rrfile={tmp_path / 't' / 'replay.bin'}" in ic
    assert "rrsnapshot=init" in ic
    assert ic.startswith(f"shift={rr.DEFAULT_ICOUNT_SHIFT},")
    assert "sleep" not in [o.split("=")[0] for o in ic.split(",")]


def test_replay_mode_differs_from_record_only_in_the_rr_field(tmp_path):
    """Everything else the guest can observe must be identical, or the recorded
    input stream is being fed to a different machine."""
    cmd = _base_cmd(tmp_path)
    rec, _ = rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)
    rep, _ = rr.transform(cmd, mode="replay", trace_dir=tmp_path / "t", stage=False)
    assert [a.replace("rr=record", "rr=replay") for a in rec] == rep


def test_sleep_off_is_not_set_in_either_mode(tmp_path):
    """It is the obvious fix for replay's wall-clock cost and it is broken on QEMU
    9.0.2 both ways: recording with it never leaves the UEFI firmware, replaying
    with it aborts in vmstate_load_state while loading the rrsnapshot. A/B'd on one
    trace with nothing else changed."""
    cmd = _base_cmd(tmp_path)
    for mode in ("record", "replay"):
        out, _ = rr.transform(cmd, mode=mode, trace_dir=tmp_path / "t", stage=False)
        # split into options, not a substring search: pytest's tmp_path is named
        # after the test, so "sleep" appears inside the rrfile PATH
        opts = [o.split("=")[0] for o in out[out.index("-icount") + 1].split(",")]
        assert "sleep" not in opts


# --- the refusals --------------------------------------------------------


def test_kvm_is_refused(tmp_path):
    """Hardware virtualization has no instruction counter. QEMU's own error for this
    is about icount, not about accel, so we say it in the terms the operator chose."""
    cmd = _base_cmd(tmp_path)
    cmd[cmd.index("tcg,thread=multi")] = "kvm"
    with pytest.raises(rr.RecordReplayError, match="TCG"):
        rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)


def test_multiple_vcpus_are_refused(tmp_path):
    cmd = [*_base_cmd(tmp_path), "-smp", "2"]
    with pytest.raises(rr.RecordReplayError, match="single-vCPU"):
        rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)


def test_smp_1_is_accepted_because_that_is_the_experiment(tmp_path):
    """`--smp 1` on the MP kernel is the UP:=smp1 collapse case — the thing the arm64
    bug lives in. Refusing all -smp would have refused the interesting boot."""
    cmd = [*_base_cmd(tmp_path), "-smp", "1"]
    out, _ = rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)
    assert out[out.index("-smp") + 1] == "1"


def test_a_nic_is_refused_rather_than_recorded_unfiltered(tmp_path):
    """Network input needs replay net filters we have not wired. Recording without
    them yields a trace that diverges the moment a packet arrives."""
    cmd = [*_base_cmd(tmp_path), "-netdev", "user,id=n0"]
    with pytest.raises(rr.RecordReplayError, match="network"):
        rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)


def test_existing_icount_is_refused(tmp_path):
    cmd = [*_base_cmd(tmp_path), "-icount", "shift=3"]
    with pytest.raises(rr.RecordReplayError, match="already carries -icount"):
        rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)


def test_a_command_line_with_no_drives_is_refused(tmp_path):
    with pytest.raises(rr.RecordReplayError, match="no -drive"):
        rr.transform(
            ["qemu-system-aarch64", "-M", "virt"],
            mode="record",
            trace_dir=tmp_path / "t",
            stage=False,
        )


# --- media lifecycle -----------------------------------------------------


def test_staging_makes_an_overlay_and_never_writes_the_original(tmp_path):
    """A raw ISO is 600MB; copying it per trace is wasteful, and writing to it would
    corrupt the media every other lane boots."""
    src = tmp_path / "boot.img"
    src.write_bytes(b"\0" * (1 << 20))
    before = src.read_bytes()
    dst = tmp_path / "t" / "media" / "boot.qcow2"
    rr.stage_image(src, dst, "raw")
    assert dst.is_file()
    assert dst.stat().st_size < src.stat().st_size  # overlay, not a copy
    assert src.read_bytes() == before


def test_qcow2_source_is_copied_not_backed(tmp_path):
    """A qcow2 backing file would leave the trace depending on a per-run harness file
    that the next boot overwrites — the trace would rot silently."""
    src = tmp_path / "efivars.qcow2"
    import subprocess

    subprocess.run(
        ["qemu-img", "create", "-f", "qcow2", str(src), "1M"], check=True, capture_output=True
    )
    dst = tmp_path / "t" / "media" / "efivars.qcow2"
    rr.stage_image(src, dst, "qcow2")
    info = json.loads(
        subprocess.run(
            ["qemu-img", "info", "--output=json", str(dst)],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    assert "backing-filename" not in info


def test_replay_resets_media_so_the_second_replay_matches_the_first(tmp_path):
    """Without this, replay #1 works and replay #2 starts from media that replay #1
    wrote to — a divergence that looks like rr being unreliable."""
    trace = tmp_path / "t"
    (trace / "media").mkdir(parents=True)
    (trace / "media" / "disk.qcow2").write_bytes(b"pristine")
    rr.snapshot_pristine(trace)
    (trace / "media" / "disk.qcow2").write_bytes(b"dirtied by a replay")
    rr.prepare_replay(trace)
    assert (trace / "media" / "disk.qcow2").read_bytes() == b"pristine"
    rr.prepare_replay(trace)
    assert (trace / "media" / "disk.qcow2").read_bytes() == b"pristine"


def test_replaying_an_incomplete_recording_says_so(tmp_path):
    trace = tmp_path / "t"
    trace.mkdir()
    with pytest.raises(rr.RecordReplayError, match="pristine"):
        rr.prepare_replay(trace)


def test_check_trace_flags_an_empty_replay_log(tmp_path):
    """QEMU flushes the rr log on exit, so a killed recording leaves a zero-byte file
    that looks like a trace until you try to replay it."""
    trace = tmp_path / "t"
    trace.mkdir()
    rr.write_manifest(trace, arch="arm64", argv=[], media={}, finalised=True)
    (trace / "replay.bin").write_bytes(b"")
    assert any("did not complete" in w for w in rr.check_trace(trace))


def test_a_qemu_version_change_warns_but_does_not_refuse(tmp_path, monkeypatch):
    """The distinction the marker exists for. A newer QEMU MIGHT still replay the
    trace, so that is advice; media the guest cannot read back is not."""
    trace = tmp_path / "t"
    trace.mkdir()
    rr.write_manifest(trace, arch="arm64", argv=[], media={}, finalised=True)
    m = json.loads((trace / "manifest.json").read_text())
    m["qemu_version"] = "QEMU emulator version 1.0.0"
    (trace / "manifest.json").write_text(json.dumps(m))
    (trace / "replay.bin").write_bytes(b"x")
    warnings = rr.check_trace(trace)
    assert warnings
    assert not any(rr.FATAL_MARKER in w for w in warnings)


def test_replay_refuses_a_trace_whose_media_moved_on(tmp_path, monkeypatch):
    """The run this prevents actually happened: a replay of a trace whose overlay
    source had been rebuilt printed the warning, carried on, opened the gdbstub,
    and then dropped the connection. A warning nobody can act on mid-run is not a
    warning."""
    from . import rrcmd

    src = tmp_path / "boot.img"
    src.write_bytes(b"as recorded")
    trace = tmp_path / "t"
    cmd = ["qemu-system-aarch64", "-drive", f"file={src},format=raw,if=virtio"]
    _out, media = rr.transform(cmd, mode="record", trace_dir=trace, stage=True)
    rr.write_manifest(trace, arch="arm64", argv=[], media=media, finalised=True)
    (trace / "replay.bin").write_bytes(b"x")
    rr.snapshot_pristine(trace)
    src.write_bytes(b"rebuilt since")

    called = []
    monkeypatch.setattr(rr, "prepare_replay", lambda t: called.append(t))

    class _Args:
        trace = str(tmp_path / "t")
        root = None
        gdb = False

    assert rrcmd.cmd_replay(_Args(), tmp_path) == 2
    assert not called, "refused replays must not even reset the media"


def test_check_trace_flags_a_qemu_version_change(tmp_path):
    """Device models change between QEMU releases; the trace does not. This is the
    most likely reason an old trace stops replaying, and it is invisible otherwise."""
    trace = tmp_path / "t"
    trace.mkdir()
    rr.write_manifest(trace, arch="arm64", argv=[], media={}, finalised=True)
    m = json.loads((trace / "manifest.json").read_text())
    m["qemu_version"] = "QEMU emulator version 1.0.0"
    (trace / "manifest.json").write_text(json.dumps(m))
    (trace / "replay.bin").write_bytes(b"x")
    assert any("device models differ" in w for w in rr.check_trace(trace))


# --- the env hook --------------------------------------------------------


def test_env_hook_is_off_unless_asked(monkeypatch):
    """Every existing boot path must be byte-identical when rr is not requested."""
    monkeypatch.delenv(rr.ENV_VAR, raising=False)
    assert rr.env_request() is None


def test_env_hook_rejects_a_malformed_request(monkeypatch):
    monkeypatch.setenv(rr.ENV_VAR, "/some/dir")
    with pytest.raises(rr.RecordReplayError):
        rr.env_request()
    monkeypatch.setenv(rr.ENV_VAR, "rewind:/some/dir")
    with pytest.raises(rr.RecordReplayError):
        rr.env_request()


def test_env_hook_parses_mode_and_dir(monkeypatch):
    monkeypatch.setenv(rr.ENV_VAR, "replay:/home/x/vibeos-rr/t1")
    assert rr.env_request() == ("replay", Path("/home/x/vibeos-rr/t1"))


def test_vm_applies_the_transform_only_when_asked(monkeypatch, tmp_path):
    """The hook lives in the launch path every boot goes through, so `off` has to
    mean untouched — not `mostly untouched`."""
    from ..vm import QemuVM

    monkeypatch.delenv(rr.ENV_VAR, raising=False)
    cmd = ["qemu-system-aarch64", "-M", "virt"]
    assert QemuVM._apply_record_replay(object.__new__(QemuVM), cmd) is cmd


def test_a_changed_device_set_is_refused_on_replay(tmp_path):
    """A trace recorded with one device set, replayed against another, aborts deep in
    the run with a message about event ordering — which reads as rr being flaky
    rather than as the config drift it is."""
    trace = tmp_path / "t"
    trace.mkdir()
    rec, _ = rr.transform(_base_cmd(tmp_path), mode="record", trace_dir=trace, stage=False)
    rr.write_manifest(trace, arch="arm64", argv=rec, media={})
    rr.assert_same_shape(trace, rec)  # identical: fine

    grown = [
        *_base_cmd(tmp_path),
        "-drive",
        f"file={tmp_path}/scratch.qcow2,format=qcow2,if=none,id=usbstor0",
    ]
    now, _ = rr.transform(grown, mode="replay", trace_dir=trace, stage=False)
    with pytest.raises(rr.RecordReplayError, match="does not match the recording"):
        rr.assert_same_shape(trace, now)


def test_pristine_is_frozen_after_the_recording_not_before(tmp_path, monkeypatch):
    """`rrsnapshot=init` writes the vmstate snapshot INTO the qcow2 during the
    recording. Media frozen before the boot do not contain it, so restoring them
    would leave every replay with no snapshot to load — and the failure surfaces as
    a replay that starts and then diverges, not as a missing-file error.

    So the launch-path hook must NOT freeze; the CLI verb freezes on the way out.
    """
    import inspect

    from ..vm import QemuVM

    src = inspect.getsource(QemuVM._apply_record_replay)
    assert "snapshot_pristine" not in src, (
        "the launch hook froze pristine/ — that captures media from BEFORE the "
        "recording wrote its init snapshot"
    )
    from . import rrcmd

    assert "snapshot_pristine" in inspect.getsource(rrcmd.cmd_record)


def test_media_are_named_after_their_source(tmp_path):
    """A media dir of rrdrive1/2/3 gives you no way to tell firmware from the
    ramdisk when you come back to a trace months later."""
    _out, media = rr.transform(
        _base_cmd(tmp_path), mode="record", trace_dir=tmp_path / "t", stage=False
    )
    assert set(media) == {"scratch.qcow2", "QEMU_EFI.qcow2", "efivars.qcow2", "boot.qcow2"}


def test_a_rebuilt_source_image_is_flagged_not_silently_replayed(tmp_path):
    """The ramdisk is kept as an overlay over `qemu/boot.img`, and every build
    rewrites that file. Without a digest the trace rots silently and the only
    symptom is a replay that diverges for no visible reason."""
    src = tmp_path / "boot.img"
    src.write_bytes(b"the media the recording saw")
    trace = tmp_path / "t"
    cmd = [
        "qemu-system-aarch64",
        "-drive",
        f"file={src},format=raw,if=virtio",
    ]
    _out, media = rr.transform(cmd, mode="record", trace_dir=trace, stage=True)
    rr.write_manifest(trace, arch="arm64", argv=[], media=media, finalised=True)
    (trace / "replay.bin").write_bytes(b"x")
    assert rr.check_trace(trace) == []

    src.write_bytes(b"the media after somebody rebuilt")
    changed = rr.check_trace(trace)
    assert any("has CHANGED" in w for w in changed)
    # FATAL, not advisory: the guest would read bytes the recording never saw, so
    # the replay is pointless and `rr replay` must refuse rather than burn a run.
    assert all(rr.FATAL_MARKER in w for w in changed)

    src.unlink()
    gone = rr.check_trace(trace)
    assert any("is GONE" in w for w in gone)
    assert all(rr.FATAL_MARKER in w for w in gone)


def test_a_copied_image_does_not_depend_on_its_source(tmp_path):
    """A qcow2 source is copied outright, so deleting the original is a non-event.
    Reporting it would train people to ignore the warning that matters."""
    import subprocess

    src = tmp_path / "efivars.qcow2"
    subprocess.run(
        ["qemu-img", "create", "-f", "qcow2", str(src), "1M"], check=True, capture_output=True
    )
    trace = tmp_path / "t"
    cmd = ["qemu-system-aarch64", "-drive", f"if=pflash,format=qcow2,file={src}"]
    _out, media = rr.transform(cmd, mode="record", trace_dir=trace, stage=True)
    rr.write_manifest(trace, arch="arm64", argv=[], media=media, finalised=True)
    (trace / "replay.bin").write_bytes(b"x")
    src.unlink()
    assert rr.check_trace(trace) == []


def test_verify_ignores_the_harness_banner_in_the_serial_log(tmp_path):
    """The live log opens with a harness-written header carrying the launcher pid and
    a wall-clock timestamp. Diffing it would make `verify` report a divergence on
    line 1 of every trace — discrediting the one check that proves rr works."""
    from .rrcmd import _guest_lines, _serial_divergence

    a = tmp_path / "a.log"
    b = tmp_path / "b.log"
    a.write_text(
        "# vibeos smoke live serial -- arm64 -- pid(launcher)=111 -- started Mon\n"
        "# tail -f /tmp/x\n\n[freeldr] MachInit done\nshell ready\n"
    )
    b.write_text(
        "# vibeos smoke live serial -- arm64 -- pid(launcher)=222 -- started Tue\n"
        "# tail -f /tmp/y\n\n[freeldr] MachInit done\nshell ready\n"
    )
    ga, gb = _guest_lines(a), _guest_lines(b)
    assert ga == ["[freeldr] MachInit done", "shell ready"]
    assert _serial_divergence(ga, gb) is None


def test_verify_reports_the_first_guest_line_that_differs(tmp_path):
    from .rrcmd import _guest_lines, _serial_divergence

    a = tmp_path / "a.log"
    b = tmp_path / "b.log"
    a.write_text("# banner\n[KE] one\n[KE] two\n[KE] three\n")
    b.write_text("# banner\n[KE] one\n[KE] TWO\n[KE] three\n")
    div = _serial_divergence(_guest_lines(a), _guest_lines(b))
    assert div is not None
    n, detail = div
    assert n == 2
    assert "[KE] two" in detail and "[KE] TWO" in detail


def test_the_replay_transcript_comes_from_the_run_that_just_happened(tmp_path):
    """A real false green this tool shipped for one iteration: the serial path was
    read from the manifest, which only a RECORD writes. A replay therefore graded
    the recording's own transcript against itself and printed VERIFIED while the
    replay had actually hung at a fifth of the recording's output.

    last-run.json is rewritten by every run, and the mode guard catches the case
    where the launch hook did not run at all.
    """
    from .rrcmd import _keep_serial

    trace = tmp_path / "t"
    trace.mkdir()
    rec = tmp_path / "rec.log"
    rep = tmp_path / "rep.log"
    rec.write_text("# banner\nfull boot\nshell ready\n")
    rep.write_text("# banner\nfull boot\n")

    rr.note_run(trace, mode="record", serial_log=str(rec))
    assert _keep_serial(trace, "record-serial.log", expect_mode="record") is not None

    rr.note_run(trace, mode="replay", serial_log=str(rep))
    kept = _keep_serial(trace, "replay-serial.log", expect_mode="replay")
    assert kept is not None
    assert kept.read_text() != (trace / "record-serial.log").read_text()


def test_keeping_serial_refuses_when_the_last_run_was_the_other_mode(tmp_path):
    from .rrcmd import _keep_serial

    trace = tmp_path / "t"
    trace.mkdir()
    log = tmp_path / "x.log"
    log.write_text("boot\n")
    rr.note_run(trace, mode="record", serial_log=str(log))
    assert _keep_serial(trace, "replay-serial.log", expect_mode="replay") is None


def test_last_run_missing_is_an_error_not_a_silent_pass(tmp_path):
    trace = tmp_path / "t"
    trace.mkdir()
    with pytest.raises(rr.RecordReplayError, match="launch hook did not run"):
        rr.last_run(trace)


def test_rr_runs_get_a_watchdog_budget_matched_to_how_slow_they_are(tmp_path, monkeypatch):
    """A replay waits out each recorded I/O completion instead of taking the host's,
    so it is quiet for longer than plain TCG ever is. The stock 120s silence
    watchdog aborted a perfectly healthy arm64 replay as HANG_TIMEOUT."""
    from ..vm import Arm64Config, QemuVM

    trace = tmp_path / "t"
    trace.mkdir()
    cmd = _base_cmd(tmp_path)
    rec, _ = rr.transform(cmd, mode="record", trace_dir=trace, stage=False)
    rr.write_manifest(trace, arch="arm64", argv=rec, media={})

    vm = object.__new__(QemuVM)
    vm._backend = "native"
    vm.live_log_path = tmp_path / "serial.log"

    vm.config = Arm64Config(boot_timeout=540.0, hang_silence_seconds=120.0)
    monkeypatch.setenv(rr.ENV_VAR, f"replay:{trace}")
    monkeypatch.setenv(rr.SHIFT_ENV_VAR, "3")
    vm._apply_record_replay(cmd)
    assert vm.config.boot_timeout >= 1800.0
    assert vm.config.hang_silence_seconds >= 600.0


def test_a_config_with_the_watchdog_off_stays_off(tmp_path, monkeypatch):
    """0 means disabled, not `unset` — raising it would ARM a watchdog the arch
    config deliberately turned off."""
    from ..vm import Arm64Config, QemuVM

    trace = tmp_path / "t"
    trace.mkdir()
    cmd = _base_cmd(tmp_path)
    rec, _ = rr.transform(cmd, mode="record", trace_dir=trace, stage=False)
    rr.write_manifest(trace, arch="arm64", argv=rec, media={})

    vm = object.__new__(QemuVM)
    vm._backend = "native"
    vm.live_log_path = tmp_path / "serial.log"

    vm.config = Arm64Config(boot_timeout=540.0, hang_silence_seconds=0.0)
    monkeypatch.setenv(rr.ENV_VAR, f"replay:{trace}")
    vm._apply_record_replay(cmd)
    assert vm.config.hang_silence_seconds == 0.0


def test_a_short_replay_is_not_reported_as_a_pass(tmp_path, monkeypatch):
    """A replay that stops halfway has proven nothing about the half it never
    reached. Calling that a pass is how a tool starts lying to you."""
    from . import rrcmd

    trace = tmp_path / "t"
    trace.mkdir()
    (trace / "record-serial.log").write_text("# banner\na\nb\nc\nd\n")
    replay_log = tmp_path / "replay.log"
    replay_log.write_text("# banner\na\nb\n")
    rr.note_run(trace, mode="replay", serial_log=str(replay_log))

    class _Args:
        trace = str(tmp_path / "t")
        root = None
        gdb = False

    # the boot itself is not under test; monkeypatch so the stub cannot leak
    monkeypatch.setattr(rrcmd, "cmd_replay", lambda args, repo: 0)
    assert rrcmd.cmd_verify(_Args(), tmp_path) != 0


# --- the REAL per-arch command lines -------------------------------------
#
# The fixtures above are hand-written and can drift from what the arch configs
# actually emit. These drive the transform with the genuine output of
# ArchConfig.qemu_args, so a config change that rr cannot handle fails here
# instead of halfway through someone's recording.


def _real_arch_args(name: str, work: Path) -> list[str] | None:
    """The real qemu_args for one arch config, or None if its media are absent.

    Skipping is correct rather than lazy: qemu_args copies firmware images that a
    fresh worktree does not have until it has built, and a missing OVMF is not
    an rr defect.
    """
    from .. import vm as vm_mod

    cfg = getattr(vm_mod, name)()
    work.mkdir(parents=True, exist_ok=True)  # qemu_args writes its firmware copies here
    try:
        return cfg.qemu_args(work)
    except (FileNotFoundError, OSError, subprocess.CalledProcessError):
        return None


def _demoted(name: str, args: list[str] | None) -> list[str] | None:
    """What the vm.py hook hands the transform: x86 demoted from KVM to TCG.

    Not a test convenience — it is the step that makes rr possible on x86 at all,
    and this test found that it was missing.
    """
    if args is None or name.startswith("Arm64"):
        return args
    from .. import backend as be

    return be.rewrite_accel(args, "tcg", "amd64" if "Amd64" in name else "i386")


@pytest.mark.parametrize("config", ["Amd64UefiConfig", "I386UefiConfig"])
def test_x86_configs_are_refused_for_ATAPI_rather_than_recorded_dead(config, tmp_path):
    """x86 boots its ISO from `ide-cd`, and QEMU 9.0.2's blkreplay stalls that path
    — the guest never leaves the OVMF screen clear. Measured by wrapping one drive
    at a time on amd64: scratch behind virtio-blk-pci wrapped gives 2050 serial
    lines in 100s (unwrapped: 1975); cd0 behind ide-cd wrapped gives ONE.

    So the refusal is the correct behaviour and this test pins it. The failure it
    prevents is the worst kind: leaving cd0 unwrapped would record a trace whose
    BOOT media replays non-deterministically, and nothing downstream of that could
    be trusted. When a QEMU that fixes blkreplay-over-ATAPI arrives, this test is
    the thing that should fail.
    """
    args = _demoted(config, _real_arch_args(config, tmp_path / "work"))
    if args is None:
        pytest.skip(f"{config} media not present in this worktree")
    with pytest.raises(rr.RecordReplayError, match="ATAPI"):
        rr.transform(args, mode="record", trace_dir=tmp_path / "t", stage=False)


@pytest.mark.parametrize("config", ["Arm64Config"])
def test_every_arch_config_transforms_coherently(config, tmp_path):
    """For each arch rr can actually record: every drive a DEVICE points at must end
    up as a blkreplay filter, and no guest-visible drive may go unwrapped.

    Attach a device to the raw backend instead of the filter and QEMU starts
    happily with that drive UNRECORDED — no error anywhere, and a trace that
    cannot replay.
    """
    args = _demoted(config, _real_arch_args(config, tmp_path / "work"))
    if args is None:
        pytest.skip(f"{config} media not present in this worktree")
    out, media = rr.transform(args, mode="record", trace_dir=tmp_path / "t", stage=False)

    drives = [rr.parse_drive(d) for d in _drives(out)]
    by_id = {d["id"]: d for d in drives if "id" in d}
    for arg in out:
        for tok in arg.split(","):
            if tok.startswith("drive="):
                ref = tok.split("=", 1)[1]
                assert ref in by_id, f"-device references drive={ref}, which no -drive defines"
                assert by_id[ref].get("driver") == "blkreplay", (
                    f"device is attached to {ref}, which is NOT the blkreplay filter — "
                    "the recording would capture nothing for that drive"
                )
    for d in drives:
        if d.get("if") == "pflash" or d.get("driver") == "blkreplay":
            continue
        assert d.get("id", "").endswith("-img"), f"unwrapped guest drive: {d}"
    assert media, "no media staged"


@pytest.mark.parametrize("config", ["Arm64Config"])
def test_no_arch_config_smuggles_in_a_host_path_we_cannot_pin(config, tmp_path):
    """Firmware from a distro package (amd64 boots /usr/share/OVMF/OVMF_CODE_4M.fd)
    is outside the repo and outside our control. rr keeps it as an overlay, so the
    trace depends on that file — which is fine, but only because the manifest
    digests it and `rr info` warns when it changes. This asserts the digest is
    actually taken for every non-qcow2 source."""
    args = _demoted(config, _real_arch_args(config, tmp_path / "work"))
    if args is None:
        pytest.skip(f"{config} media not present in this worktree")
    _out, media = rr.transform(args, mode="record", trace_dir=tmp_path / "t", stage=True)
    for name, info in media.items():
        if info["copied"]:
            continue
        assert info["sha256"], f"{name} is an overlay with no source digest — it can rot silently"


def test_the_hook_demotes_x86_to_tcg_itself(tmp_path, monkeypatch):
    """x86 boots `-accel kvm -cpu host` by default, and KVM has no instruction
    counter. If the hook did not demote, every `rr record --arch amd64` would die
    on the transform's refusal and the operator would be left to find the knob.
    `-cpu host` has to go with it — it is a KVM host-passthrough concept.
    """
    from ..vm import Amd64UefiConfig, QemuVM

    trace = tmp_path / "t"
    trace.mkdir()
    vm = object.__new__(QemuVM)
    vm._backend = "native"
    vm.live_log_path = tmp_path / "serial.log"
    vm.config = Amd64UefiConfig()

    cmd = [
        "qemu-system-x86_64",
        "-accel",
        "kvm",
        "-cpu",
        "host",
        "-drive",
        f"id=scratch,if=none,format=qcow2,file={tmp_path}/scratch.qcow2",
        # virtio, not the real config's ide-cd: this test is about the ACCELERATOR,
        # and an ide-cd would (correctly) be refused before the accel is visible.
        "-device",
        "virtio-blk-pci,drive=scratch",
    ]
    (tmp_path / "scratch.qcow2").write_bytes(b"\0" * 64)
    monkeypatch.setenv(rr.ENV_VAR, f"record:{trace}")
    out = vm._apply_record_replay(cmd)
    assert "kvm" not in out
    assert "tcg" in " ".join(out)
    assert "host" not in out, "-cpu host survived the demotion; TCG cannot honour it"


def test_arm64_keeps_its_cpu_model_through_the_hook(tmp_path, monkeypatch):
    """The demotion is an x86 rewrite. Applied to arm64 it would strip
    `-cpu cortex-a72`, leaving the guest on QEMU's default model — the same
    carve-out _build_windows_cmd already makes."""
    from ..vm import Arm64Config, QemuVM

    trace = tmp_path / "t"
    trace.mkdir()
    vm = object.__new__(QemuVM)
    vm._backend = "native"
    vm.live_log_path = tmp_path / "serial.log"
    vm.config = Arm64Config()

    cmd = _base_cmd(tmp_path)
    monkeypatch.setenv(rr.ENV_VAR, f"record:{trace}")
    out = vm._apply_record_replay(cmd)
    assert "cortex-a72" in out


# --- telling a WEDGED replay from a slow one -----------------------------


def test_trace_position_finds_the_process_reading_the_trace(tmp_path, monkeypatch):
    """The probe reads /proc for whoever holds replay.bin open. If it cannot find
    the reader it must return None, not 0 — 0 would read as `consumed nothing`,
    which is a verdict, and the absence of information is not one."""
    trace = tmp_path / "t"
    trace.mkdir()
    rrfile = trace / "replay.bin"
    rrfile.write_bytes(b"x" * 4096)
    resolve = Path.resolve

    def no_fd_resolution(path, *args, **kwargs):
        assert not str(path).startswith("/proc/"), "must not traverse unrelated fd targets"
        return resolve(path, *args, **kwargs)

    monkeypatch.setattr(Path, "resolve", no_fd_resolution)
    assert rr.trace_read_position(trace) is None
    with rrfile.open("rb") as f:
        f.read(1)
        # The KERNEL offset, so it moves in buffered chunks, not per byte read.
        # That is fine for the only question asked of it — is it moving at all,
        # over minutes — but it means the value is not a byte-accurate cursor.
        assert rr.trace_read_position(trace) not in (None, 0)


def test_a_frozen_read_position_is_reported_as_WEDGED(tmp_path):
    """The distinction this exists for: a replay that stops consuming recorded
    events is not going to finish, and telling the operator to be patient is the
    wrong answer. Measured on trace rec5 — 167,936 of 10,266,279 bytes, frozen
    across 200s at 111% CPU."""
    from .rrcmd import _TraceProgress

    trace = tmp_path / "t"
    trace.mkdir()
    (trace / "replay.bin").write_bytes(b"x" * 10_266_279)
    p = _TraceProgress(trace, period=0.01)
    p.max_pos = 167_936
    p.stalled_for = 200.0
    note = p.report()
    assert note is not None
    assert "WEDGED" in note
    assert "1.6%" in note
    assert "waiting" in note


def test_a_replay_that_keeps_consuming_is_not_called_wedged(tmp_path):
    from .rrcmd import _TraceProgress

    trace = tmp_path / "t"
    trace.mkdir()
    (trace / "replay.bin").write_bytes(b"x" * 1000)
    p = _TraceProgress(trace, period=0.01)
    p.max_pos, p.stalled_for = 900, 20.0
    note = p.report()
    assert note is not None
    assert "WEDGED" not in note
    assert "90.0%" in note


def test_no_progress_information_produces_no_claim(tmp_path):
    """On a platform without /proc, or a run where the sampler never caught the
    process, the tool must say nothing rather than invent a verdict."""
    from .rrcmd import _TraceProgress

    trace = tmp_path / "t"
    trace.mkdir()
    (trace / "replay.bin").write_bytes(b"x" * 1000)
    assert _TraceProgress(trace, period=0.01).report() is None


def test_a_virtio_backed_cd_would_be_allowed(tmp_path):
    """The refusal is on the DEVICE MODEL, not on "it is a CD". If the boot ISO is
    ever attached through virtio-blk, rr can record it — that is the measured
    difference, and it is the shape of the eventual x86 fix."""
    cmd = [
        "qemu-system-x86_64",
        "-drive",
        f"id=cd0,if=none,format=raw,file={tmp_path}/iso",
        "-device",
        "virtio-blk-pci,drive=cd0",
    ]
    (tmp_path / "iso").write_bytes(b"\0" * 64)
    out, _ = rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)
    assert any("driver=blkreplay" in a for a in out)


def test_scsi_cd_is_refused_too(tmp_path):
    """Same ATAPI command set, same block path. Listing only `ide-cd` would let the
    identical failure through the moment somebody changed the controller."""
    cmd = [
        "qemu-system-x86_64",
        "-drive",
        f"id=cd0,if=none,format=raw,file={tmp_path}/iso",
        "-device",
        "scsi-cd,drive=cd0",
    ]
    (tmp_path / "iso").write_bytes(b"\0" * 64)
    with pytest.raises(rr.RecordReplayError, match="ATAPI"):
        rr.transform(cmd, mode="record", trace_dir=tmp_path / "t", stage=False)


def test_the_gdb_flag_warns_how_to_stop_a_replay_correctly(tmp_path, monkeypatch, capsys):
    """`--gdb` was documented as giving reverse execution and never tested; the first
    real test aborted QEMU with `Playback shouldn't have to iowait`.

    The warning must name BOTH halves. Saying only "stopping aborts it" is what led
    to declaring the tool a camera rather than a debugger — replay-break stops a
    replay cleanly and gdb can then inspect the paused guest, verified. A warning
    that omits the working mechanism sends people away from it."""
    from . import rrcmd

    trace = tmp_path / "t"
    trace.mkdir()
    rr.write_manifest(trace, arch="arm64", argv=[], media={}, boot_cmd=["true"], finalised=True)
    (trace / "replay.bin").write_bytes(b"x")
    (trace / "media").mkdir()
    rr.snapshot_pristine(trace)
    # check_trace() shells out for the QEMU version; the boot itself is not under
    # test, so stub both with something that satisfies each caller.
    monkeypatch.setattr(
        rrcmd.subprocess,
        "run",
        lambda *a, **k: type("R", (), {"returncode": 0, "stdout": "", "stderr": ""})(),
    )
    monkeypatch.setattr(
        rr.subprocess,
        "run",
        lambda *a, **k: type("R", (), {"returncode": 0, "stdout": "", "stderr": ""})(),
    )

    class _Args:
        root = None
        gdb = True

        def __init__(self, where: Path) -> None:
            self.trace = str(where)

    rrcmd.cmd_replay(_Args(trace), tmp_path)
    err = capsys.readouterr().err
    assert "Playback shouldn't have to iowait" in err
    # It must name the failing operations specifically -- a breakpoint HIT aborts
    # too, which is the least obvious of them and the one that decides whether a
    # planned debugging session can work at all.
    assert "breakpoint hit" in err.lower()
    # and it must point at the mechanism that DOES work, not just the one that
    # does not: replay-break stops a replay cleanly and gdb can then inspect it.
    assert "replay-break" in err


def test_record_waits_for_the_trace_to_stop_growing(tmp_path):
    """`vibeos-cli conformance` returns while its QEMU is still shutting down, so
    the boot command returning is NOT the recording ending. Measured on a kmtest
    record: command returned 05:28:38, guest serial finished 05:29:23, trace
    finished 05:29:39. Finalising at the first froze pristine/ mid-record, kept a
    1,456-byte transcript of a 240,750-byte boot, and reported "trace 0MB" for
    10.6MB — three silent corruptions from one early return.
    """
    import threading

    from . import rrcmd

    trace = tmp_path / "t"
    trace.mkdir()
    rrbin = trace / "replay.bin"
    rrbin.write_bytes(b"")

    def grow():
        for n in range(1, 6):
            time.sleep(0.05)
            rrbin.write_bytes(b"x" * (n * 1000))

    threading.Thread(target=grow, daemon=True).start()
    size = rrcmd.await_trace_quiescence(trace, settle=0.4, limit=20.0)
    assert size == 5000, "returned before the trace stopped growing"


def test_quiescence_gives_up_loudly_rather_than_hanging(tmp_path, capsys):
    """A trace that never settles must not block a recording forever — but the
    caller has to be told the finalised trace may be truncated."""
    from . import rrcmd

    trace = tmp_path / "t"
    trace.mkdir()
    rrbin = trace / "replay.bin"
    rrbin.write_bytes(b"x")
    stop = False

    import threading

    def grow():
        n = 1
        while not stop and n < 10_000:
            n += 1
            rrbin.write_bytes(b"x" * n)
            time.sleep(0.02)

    t = threading.Thread(target=grow, daemon=True)
    t.start()
    rrcmd.await_trace_quiescence(trace, settle=5.0, limit=1.0)
    stop = True
    assert "still growing" in capsys.readouterr().err


# --- reclaiming disk ------------------------------------------------------


def _fake_trace(root: Path, name: str, size: int, age: float) -> Path:
    import os

    d = root / name
    (d / "media").mkdir(parents=True)
    rr.write_manifest(d, arch="arm64", argv=[], media={}, finalised=True)
    (d / "replay.bin").write_bytes(b"x" * size)
    os.utime(d, (age, age))
    return d


class _PruneArgs:
    root = None
    keep = 2
    name = None
    apply = False

    def __init__(self, root, **kw):
        self.root = str(root)
        for k, v in kw.items():
            setattr(self, k, v)


def test_prune_is_dry_run_unless_asked(tmp_path):
    """A trace costs a full boot to make. Deleting one should take the same
    deliberate second keystroke `vibeos-cli prune` asks for."""
    from . import rrcmd

    root = tmp_path / "traces"
    root.mkdir()
    for n, age in (("old", 1000), ("mid", 2000), ("new", 3000)):
        _fake_trace(root, n, 100, age)
    rrcmd.cmd_prune(_PruneArgs(root, keep=1), tmp_path)
    assert {d.name for d in root.iterdir()} == {"old", "mid", "new"}, "dry run deleted something"


def test_prune_keeps_the_NEWEST_n(tmp_path):
    """Newest-first: the trace you just made is the one you are about to replay."""
    from . import rrcmd

    root = tmp_path / "traces"
    root.mkdir()
    for n, age in (("old", 1000), ("mid", 2000), ("new", 3000)):
        _fake_trace(root, n, 100, age)
    rrcmd.cmd_prune(_PruneArgs(root, keep=2, apply=True), tmp_path)
    assert {d.name for d in root.iterdir()} == {"mid", "new"}


def test_prune_by_name_ignores_the_keep_count(tmp_path):
    """Deleting a KNOWN-BAD trace by name must work even when it is the newest —
    e.g. a recording finalised mid-write, which looks valid and is not."""
    from . import rrcmd

    root = tmp_path / "traces"
    root.mkdir()
    for n, age in (("good", 1000), ("truncated", 3000)):
        _fake_trace(root, n, 100, age)
    rrcmd.cmd_prune(_PruneArgs(root, keep=5, name=["truncated"], apply=True), tmp_path)
    assert {d.name for d in root.iterdir()} == {"good"}


def test_prune_never_touches_a_directory_that_is_not_a_trace(tmp_path):
    """The root is under the user's home. Anything without a manifest is not ours."""
    from . import rrcmd

    root = tmp_path / "traces"
    root.mkdir()
    _fake_trace(root, "atrace", 100, 1000)
    (root / "someone-elses-data").mkdir()
    rrcmd.cmd_prune(_PruneArgs(root, keep=0, apply=True), tmp_path)
    assert (root / "someone-elses-data").is_dir()
    assert not (root / "atrace").exists()


def test_a_recording_is_never_a_gate_boot(tmp_path):
    """Two traces are comparable only while what gets recorded is pinned.

    The wrong inference here is a short one with two true premises: arm64 gates
    run the WoW64 ladder, rr records arm64 boots, therefore traces now contain
    nine WoW64 guests. False — a recording is a `conformance` or `smoke` boot,
    never a `gate` boot. This test is what makes adding a gate-only behaviour to
    recordings a deliberate act rather than something that arrives by
    inheritance.
    """
    from .rrcmd import _boot_cmd

    for payload in ("kmtest", "none"):
        cmd = _boot_cmd(tmp_path, "arm64", payload, 1, False, 600)
        assert "gate" not in cmd
        assert "--ladder" not in cmd
        assert cmd[1] in ("conformance", "smoke")


def test_a_trace_from_before_the_finalisation_fix_is_refused(tmp_path):
    """Traces recorded before `rr record` waited for the trace to stop growing are
    TRUNCATED and look identical from the outside — same manifest, media, pristine,
    a plausible replay.bin. Replaying one aborts with "Missing character write
    event in the replay log", deterministically, having produced 56 of 3149 lines.

    The absence of the flag is the only way to recognise that generation, so it is
    fatal rather than advisory: there is nothing a replay of one can tell you.
    """
    trace = tmp_path / "t"
    trace.mkdir()
    rr.write_manifest(trace, arch="arm64", argv=[], media={})  # no finalised=
    (trace / "replay.bin").write_bytes(b"x")
    problems = rr.check_trace(trace)
    assert any("TRUNCATED" in w for w in problems)
    assert all(rr.FATAL_MARKER in w for w in problems if "TRUNCATED" in w)


def test_record_stamps_finalised_only_after_the_trace_settles(tmp_path):
    """The flag must be written by the same code path that waits, or it becomes a
    label rather than evidence."""
    import inspect

    from . import rrcmd

    src = inspect.getsource(rrcmd.cmd_record)
    wait_at = src.index("await_trace_quiescence")
    stamp_at = src.index("finalised = True")
    assert wait_at < stamp_at, "the flag is set before the wait — it would mean nothing"


def test_a_non_default_shift_warns_that_the_trace_will_not_replay(tmp_path, monkeypatch, capsys):
    """Only the default icount shift has ever produced a replayable trace.

    Measured on arm64, one boot, three shifts: 1 and 5 both stall at the first
    4,096-byte read buffer with 271 bytes of guest serial; 3 reaches desktop
    bringup. The RECORDING succeeds at all three, so the failure is invisible
    until someone replays — by which time the expensive part is already spent.
    """
    from . import rrcmd

    monkeypatch.setattr(
        rrcmd.subprocess,
        "run",
        lambda *a, **k: type("R", (), {"returncode": 0, "stdout": "", "stderr": ""})(),
    )
    monkeypatch.setattr(rrcmd, "await_trace_quiescence", lambda t: 0)

    class _Args:
        arch = "arm64"
        payload = "none"
        smp = 1
        name = "t"
        build = False
        timeout = 60
        root = str(tmp_path)
        shift = rr.DEFAULT_ICOUNT_SHIFT + 2

    rrcmd.cmd_record(_Args(), tmp_path)
    err = capsys.readouterr().err
    assert "REPLAY PROBABLY WILL NOT" in err
    assert "4,096" in err


def test_replay_refuses_a_different_HOST_side_shape(tmp_path):
    """Backends the guest cannot observe still change how far a replay gets.

    Measured on one trace: file-backed serial without QMP reached 1773 guest
    lines, the same with a QMP socket attached reached 23, and the recording's
    own chardev+QMP shape reached 1693. A replay that silently under-reaches by
    two orders of magnitude is worse than one that crashes — it reads as a
    finding about the guest.
    """
    trace = tmp_path / "t"
    trace.mkdir()
    base = _base_cmd(tmp_path)
    recorded = [
        "-chardev",
        "socket,id=ser0,path=/tmp/x/ser.sock,server=off,reconnect=1",
        "-serial",
        "chardev:ser0",
        *base,
    ]
    rec, _ = rr.transform(recorded, mode="record", trace_dir=trace, stage=False)
    rr.write_manifest(trace, arch="arm64", argv=rec, media={}, finalised=True)
    rr.assert_same_shape(trace, rec)  # identical: fine

    hand_rolled = ["-serial", f"file:{tmp_path}/out.log", *base]
    now, _ = rr.transform(hand_rolled, mode="replay", trace_dir=trace, stage=False)
    with pytest.raises(rr.RecordReplayError, match="HOST-SIDE shape"):
        rr.assert_same_shape(trace, now)


def test_host_shape_ignores_per_run_paths_and_ports(tmp_path):
    """The harness picks a fresh socket path every run, so comparing paths would
    refuse every legitimate replay. Only the backend KIND is compared."""
    a = ["-qmp", "unix:/tmp/ros-aaa/qmp.sock,server,nowait", "-serial", "chardev:ser0"]
    b = ["-qmp", "unix:/tmp/ros-zzz/qmp.sock,server,nowait", "-serial", "chardev:ser0"]
    assert rr._host_shape(a) == rr._host_shape(b)
    c = ["-qmp", "unix:/tmp/ros-aaa/qmp.sock,server,nowait", "-serial", "file:/tmp/o.log"]
    assert rr._host_shape(a) != rr._host_shape(c)


# --------------------------------------------------------------------------
# legal break icounts
# --------------------------------------------------------------------------


def _replay_log(tmp_path, events: bytes, version=rr._SUPPORTED_REPLAY_VERSION) -> Path:
    """A minimal replay.bin: the 12-byte header, then raw event bytes."""
    trace = tmp_path / "t"
    trace.mkdir()
    (trace / "replay.bin").write_bytes(struct.pack(">IQ", version, 0) + events)
    return trace


def _insn(n: int) -> bytes:
    return struct.pack(">BI", 0, n)


def test_boundaries_accumulate_instruction_chunks(tmp_path):
    trace = _replay_log(tmp_path, _insn(100) + _insn(50) + _insn(1))
    assert rr.instruction_boundaries(trace) == [100, 150, 151]


def test_boundaries_skip_over_the_other_event_types(tmp_path):
    """A mis-sized event desynchronises the walk, so every event we skip has to
    be skipped by exactly the right width. EVENT_CHAR_WRITE is the common one:
    235,577 of them in a single arm64 boot."""
    body = (
        _insn(10)
        + struct.pack(">BII", 22, 1, 2)  # EVENT_CHAR_WRITE
        + struct.pack(">BQ", 28, 0)  # EVENT_CLOCK_HOST
        + bytes([31])  # EVENT_CP_CLOCK_WARP_ACCOUNT, no payload
        + struct.pack(">BQ", 29, 0)  # EVENT_CLOCK_VIRTUAL_RT
        + _insn(7)
    )
    assert rr.instruction_boundaries(_replay_log(tmp_path, body)) == [10, 17]


def test_boundaries_handle_a_variable_length_random_event(tmp_path):
    body = _insn(3) + struct.pack(">BII", 27, 0, 5) + b"abcde" + _insn(4)
    assert rr.instruction_boundaries(_replay_log(tmp_path, body)) == [3, 7]


def test_boundaries_handle_an_async_bh_event(tmp_path):
    body = _insn(3) + struct.pack(">BBBQ", 8, 0, 1, 999) + _insn(4)
    assert rr.instruction_boundaries(_replay_log(tmp_path, body)) == [3, 7]


def test_boundaries_refuse_an_unknown_async_kind_rather_than_going_short(tmp_path):
    """The failure mode this guards is not a crash but a SHORT list: a walk that
    loses sync still returns plausible numbers, and a caller would then snap a
    break to an icount that aborts the replay."""
    body = _insn(3) + struct.pack(">BBB", 8, 4, 1) + _insn(4)
    with pytest.raises(rr.RecordReplayError, match="no known payload width"):
        rr.instruction_boundaries(_replay_log(tmp_path, body))


def test_boundaries_refuse_an_unknown_replay_version(tmp_path):
    trace = _replay_log(tmp_path, _insn(5), version=0xE02007)
    with pytest.raises(rr.RecordReplayError, match="only knows"):
        rr.instruction_boundaries(trace)


def test_a_truncated_trailing_event_ends_the_walk_without_inventing_a_boundary(tmp_path):
    """A recording killed mid-write leaves a partial event. Everything before it
    is still true, so the walk keeps those boundaries and stops."""
    trace = _replay_log(tmp_path, _insn(10) + b"\x00\x00\x01")
    assert rr.instruction_boundaries(trace) == [10]


def test_snap_goes_FORWARD_to_the_next_legal_icount(tmp_path):
    """Forward, never nearest: a caller asking to stop at N wants to have seen
    everything up to N, and a backward snap silently stops short of it."""
    trace = _replay_log(tmp_path, _insn(100) + _insn(100) + _insn(100))
    assert rr.snap_break_icount(trace, 100) == 100  # already legal
    assert rr.snap_break_icount(trace, 101) == 200
    assert rr.snap_break_icount(trace, 199) == 200
    assert rr.snap_break_icount(trace, 1) == 100


def test_snap_refuses_an_icount_past_the_end_of_the_recording(tmp_path):
    """A break past the end never fires, which looks exactly like a break that
    was never set — so it fails here instead."""
    trace = _replay_log(tmp_path, _insn(100))
    with pytest.raises(rr.RecordReplayError, match="past the end"):
        rr.snap_break_icount(trace, 101)


def test_boundaries_refuse_a_trace_with_no_recording(tmp_path):
    trace = tmp_path / "empty"
    trace.mkdir()
    with pytest.raises(rr.RecordReplayError, match="nothing was recorded"):
        rr.instruction_boundaries(trace)


def test_env_break_reads_the_requested_icount(monkeypatch):
    monkeypatch.delenv(rr.BREAK_ENV_VAR, raising=False)
    assert rr.env_break() is None
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "12345")
    assert rr.env_break() == 12345


def test_env_break_refuses_junk_rather_than_silently_not_breaking(monkeypatch):
    """A replay that quietly failed to arm its break looks exactly like a break
    that fired and was missed."""
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "not-a-number")
    with pytest.raises(rr.RecordReplayError, match="must be an integer"):
        rr.env_break()
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "-1")
    with pytest.raises(rr.RecordReplayError, match="must not be negative"):
        rr.env_break()


def test_arm_replay_break_snaps_and_arms_through_the_harness_qmp(tmp_path, monkeypatch, capsys):
    """The vm.py hook: a mid-chunk request must be snapped forward before it ever
    reaches QEMU, because arming it unsnapped aborts the whole replay."""
    from tests.qemu.vm import QemuVM

    trace = _replay_log(tmp_path, _insn(100) + _insn(100))
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "150")

    armed = []

    class FakeQmp:
        def execute(self, name, **kw):
            armed.append((name, kw))

    vm = QemuVM.__new__(QemuVM)  # no QEMU, no sockets — only the hook logic under test
    vm.on_qmp_ready = None
    vm._arm_replay_break(rr, trace)
    assert vm.on_qmp_ready is not None
    vm.on_qmp_ready(FakeQmp())
    assert armed == [("replay-break", {"icount": 200})]
    assert "mid-chunk" in capsys.readouterr().err


def test_arm_replay_break_is_a_no_op_when_nothing_asked_for_one(tmp_path, monkeypatch):
    from tests.qemu.vm import QemuVM

    trace = _replay_log(tmp_path, _insn(100))
    monkeypatch.delenv(rr.BREAK_ENV_VAR, raising=False)
    vm = QemuVM.__new__(QemuVM)
    vm.on_qmp_ready = None
    vm._arm_replay_break(rr, trace)
    assert vm.on_qmp_ready is None


def test_arm_replay_break_refuses_an_icount_past_the_end(tmp_path, monkeypatch):
    """Better to fail at arming than to boot a whole replay whose break can never
    fire and looks identical to one that was never set."""
    from tests.qemu.vm import QemuVM

    trace = _replay_log(tmp_path, _insn(100))
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "999999")
    vm = QemuVM.__new__(QemuVM)
    vm.on_qmp_ready = None
    with pytest.raises(rr.RecordReplayError, match="past the end"):
        vm._arm_replay_break(rr, trace)


def _shutdown(eid: int = 14) -> bytes:
    return bytes([eid])


def test_a_log_that_reaches_a_shutdown_event_is_provably_complete(tmp_path):
    trace = _replay_log(tmp_path, _insn(10) + _shutdown())
    assert rr.trace_is_provably_complete(trace)


def test_completeness_ignores_bytes_after_the_shutdown_event(tmp_path):
    """Replay stops at shutdown and never reads further. Real traces carry a
    trailing byte past it (clean1 has a 0x27), and QEMU's own replay-dump.py
    stops at the same place — so requiring EOF would fail a whole trace."""
    trace = _replay_log(tmp_path, _insn(10) + _shutdown() + b"\x27")
    assert rr.trace_is_provably_complete(trace)


def test_a_log_that_just_stops_is_not_complete(tmp_path):
    """The truncation case: events all the way to EOF, no shutdown."""
    trace = _replay_log(tmp_path, _insn(10) + _insn(10))
    assert not rr.trace_is_provably_complete(trace)


def test_a_log_cut_mid_event_is_not_complete(tmp_path):
    trace = _replay_log(tmp_path, _insn(10) + b"\x00\x00\x01")
    assert not rr.trace_is_provably_complete(trace)


def test_completeness_withholds_judgement_on_an_unknown_version(tmp_path):
    """An unknown version is a reason not to UPGRADE confidence, not a reason to
    refuse — so it returns False and the marker check stands on its own."""
    trace = _replay_log(tmp_path, _insn(10) + _shutdown(), version=0xE02007)
    assert not rr.trace_is_provably_complete(trace)


def test_an_unfinalised_but_provably_complete_trace_is_not_refused(tmp_path):
    """The point of the prover: the marker only covers traces recorded after it
    existed, so its absence is missing evidence, not evidence of truncation."""
    trace = _replay_log(tmp_path, _insn(10) + _shutdown())
    rr.write_manifest(trace, arch="arm64", argv=[], media={})  # no finalised key
    assert not any("finalised" in w for w in rr.check_trace(trace))

    other = tmp_path / "other"
    other.mkdir()
    stump = _replay_log(other, _insn(10) + _insn(10))
    rr.write_manifest(stump, arch="arm64", argv=[], media={})
    assert any("finalised" in w for w in rr.check_trace(stump))


def test_report_replay_break_says_whether_it_fired(tmp_path, monkeypatch):
    """A paused replay and a wedged one look identical from outside — frozen
    serial, busy QEMU — so the STOP event is the only thing that tells them
    apart."""
    from tests.qemu.qmp_client import QmpClient
    from tests.qemu.vm import QemuVM

    trace = _replay_log(tmp_path, _insn(100) + _insn(100))
    monkeypatch.setenv(rr.BREAK_ENV_VAR, "200")

    vm = QemuVM.__new__(QemuVM)
    vm.on_qmp_ready = None
    assert vm.report_replay_break() == ""  # nothing armed: silent

    vm._arm_replay_break(rr, trace)
    vm.qmp = QmpClient()
    assert "did NOT fire" in vm.report_replay_break()

    vm.qmp.events.append({"event": "STOP"})
    assert vm.report_replay_break() == "[rr] replay-break FIRED: guest stopped at icount 200"


def test_a_vm_with_no_break_armed_reports_nothing(tmp_path, monkeypatch):
    """smoke's triage calls report_replay_break() on every hang. It must be silent
    for ordinary boots, which is what lets it sit on the shared path."""
    from tests.qemu.vm import QemuVM

    monkeypatch.delenv(rr.BREAK_ENV_VAR, raising=False)
    vm = QemuVM.__new__(QemuVM)
    vm.on_qmp_ready = None
    assert vm.report_replay_break() == ""


# --- reading a wedge: which event did the replay stop on -------------------


def test_walk_events_reports_offset_event_and_running_icount(tmp_path):
    """The walker is what every trace question is built on, so its records have to
    carry all three facts: where the event starts, what it is, and the icount it
    leaves the guest at. A record missing the offset cannot answer a wedge."""
    body = _insn(10) + struct.pack(">BII", 22, 1, 2) + _insn(7)
    evs = list(rr.walk_events(_replay_log(tmp_path, body)))
    assert [(e.offset, e.eid, e.icount) for e in evs] == [
        (12, 0, 10),  # right after the 12-byte header
        (17, 22, 10),  # a non-instruction event does not move the icount
        (26, 0, 17),
    ]
    assert evs[1].name == "EVENT_CHAR_WRITE"


def test_boundaries_still_come_from_the_shared_walker(tmp_path):
    """instruction_boundaries is now a consumer of walk_events rather than its own
    decoder. Same answer, or the refactor changed behaviour."""
    body = _insn(100) + struct.pack(">BII", 22, 1, 2) + _insn(50)
    trace = _replay_log(tmp_path, body)
    assert rr.instruction_boundaries(trace) == [100, 150]
    assert [e.icount for e in rr.walk_events(trace) if e.eid == 0] == [100, 150]


def test_events_around_offset_splits_at_the_read_position(tmp_path):
    body = _insn(1) + _insn(2) + _insn(3) + _insn(4) + _insn(5)
    trace = _replay_log(tmp_path, body)
    # events start at 12, 17, 22, 27, 32; ask about a position inside the third
    before, after = rr.events_around_offset(trace, 24, before=2, after=2)
    assert [e.offset for e in before] == [17, 22]
    assert [e.offset for e in after] == [27, 32]


def test_events_around_offset_keeps_the_events_NEAREST_the_offset(tmp_path):
    """The window has to be the last N before, not the first N of the file —
    a wedge report built from the head of a 10MB log says nothing."""
    trace = _replay_log(tmp_path, b"".join(_insn(i) for i in range(1, 11)))
    before, _ = rr.events_around_offset(trace, 10_000, before=3, after=3)
    assert [e.icount for e in before] == [36, 45, 55]


def test_events_around_offset_handles_an_offset_past_the_end(tmp_path):
    """QEMU's read position is a buffered kernel offset, so it can legitimately
    sit at EOF while the replay is still working. That is not an error."""
    trace = _replay_log(tmp_path, _insn(1) + _insn(2))
    before, after = rr.events_around_offset(trace, 10_000)
    assert [e.icount for e in before] == [1, 3]
    assert after == []


def test_event_name_refuses_to_invent_a_name_it_does_not_know(tmp_path):
    """Only the ids whose meaning this decoder actually relies on get names. A
    plausible label on a merely-width-known event would read as evidence in a
    wedge report."""
    assert rr.event_name(0) == "EVENT_INSTRUCTION"
    assert rr.event_name(5).startswith("EVENT_ASYNC")
    assert rr.event_name(14).startswith("EVENT_SHUTDOWN")
    assert rr.event_name(33).startswith("EVENT_CHECKPOINT")
    assert rr.event_name(26) == "id 26"
    assert rr.event_name(29) == "id 29"
