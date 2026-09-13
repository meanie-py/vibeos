"""Guards on the record/replay command-line transform.

Every test here stands for a way a trace can be silently non-deterministic. That
is the failure mode worth paying for: a bad transform does not error, it produces
a recording that diverges halfway through a replay, and the divergence message
names an event number rather than the config that caused it. So the transform's
job is to REFUSE, and these tests hold it to that.
"""

from __future__ import annotations

import json
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
    rr.write_manifest(trace, arch="arm64", argv=[], media={})
    (trace / "replay.bin").write_bytes(b"")
    assert any("did not complete" in w for w in rr.check_trace(trace))


def test_check_trace_flags_a_qemu_version_change(tmp_path):
    """Device models change between QEMU releases; the trace does not. This is the
    most likely reason an old trace stops replaying, and it is invisible otherwise."""
    trace = tmp_path / "t"
    trace.mkdir()
    rr.write_manifest(trace, arch="arm64", argv=[], media={})
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
    rr.write_manifest(trace, arch="arm64", argv=[], media=media)
    (trace / "replay.bin").write_bytes(b"x")
    assert rr.check_trace(trace) == []

    src.write_bytes(b"the media after somebody rebuilt")
    assert any("has CHANGED" in w for w in rr.check_trace(trace))

    src.unlink()
    assert any("is GONE" in w for w in rr.check_trace(trace))


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
    rr.write_manifest(trace, arch="arm64", argv=[], media=media)
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
