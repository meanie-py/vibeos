"""`vibeos-cli stress` — the pure parts: knob rewriting, the matrix reading."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from tests.qemu import vm
from tests.qemu.cli import stress

X86 = ["qemu-system-x86_64", "-M", "pc", "-accel", "kvm", "-cpu", "host", "-smp", "2"]
ARM = ["qemu-system-aarch64", "-M", "virt", "-cpu", "cortex-a72", "-accel", "tcg,thread=multi"]


def test_no_knob_leaves_every_command_byte_identical(monkeypatch):
    for k in ("VIBEOS_QEMU_ICOUNT_SHIFT", "VIBEOS_TCG_THREAD", "VIBEOS_BOOT_TIMEOUT_SCALE"):
        monkeypatch.delenv(k, raising=False)
    assert vm.apply_stress_knobs(list(X86), "amd64-uefi") == X86
    assert vm.apply_stress_knobs(list(ARM), "arm64") == ARM
    assert vm.stress_timeout_scale() == 1.0


def test_icount_forces_tcg_single_thread_on_x86(monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_ICOUNT_SHIFT", "7")
    monkeypatch.delenv("VIBEOS_TCG_THREAD", raising=False)
    out = vm.apply_stress_knobs(list(X86), "amd64-uefi")
    assert "kvm" not in out and "host" not in out  # no instruction counter under KVM
    assert out[out.index("-accel") + 1] == "tcg,thread=single"
    assert out[-2:] == ["-icount", "shift=7,sleep=off"]
    assert "-smp" in out  # icount + -smp is legal under single-threaded TCG


def test_icount_rewrites_arm64_mttcg_but_keeps_its_cpu_model(monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_ICOUNT_SHIFT", "3")
    out = vm.apply_stress_knobs(list(ARM), "arm64")
    assert out[out.index("-cpu") + 1] == "cortex-a72"
    assert out[out.index("-accel") + 1] == "tcg,thread=single"
    assert out[-1] == "shift=3,sleep=off"


def test_thread_knob_alone_only_touches_tcg_accels(monkeypatch):
    monkeypatch.delenv("VIBEOS_QEMU_ICOUNT_SHIFT", raising=False)
    monkeypatch.setenv("VIBEOS_TCG_THREAD", "single")
    assert vm.apply_stress_knobs(list(X86), "amd64-uefi") == X86  # kvm: untouched
    out = vm.apply_stress_knobs(list(ARM), "arm64")
    assert out[out.index("-accel") + 1] == "tcg,thread=single"
    assert "-icount" not in out


def test_icount_refuses_to_stack_on_record_replay(monkeypatch):
    monkeypatch.setenv("VIBEOS_QEMU_ICOUNT_SHIFT", "3")
    with pytest.raises(ValueError, match="do not stack"):
        vm.apply_stress_knobs([*ARM, "-icount", "shift=3,rr=record,rrfile=x"], "arm64")


@pytest.mark.parametrize(
    "env,val",
    [
        ("VIBEOS_QEMU_ICOUNT_SHIFT", "11"),
        ("VIBEOS_TCG_THREAD", "dual"),
        ("VIBEOS_BOOT_TIMEOUT_SCALE", "0"),
    ],
)
def test_bad_knob_values_fail_loud(monkeypatch, env, val):
    monkeypatch.setenv(env, val)
    with pytest.raises(ValueError):
        {
            "VIBEOS_QEMU_ICOUNT_SHIFT": vm.stress_icount_shift,
            "VIBEOS_TCG_THREAD": vm.stress_tcg_thread,
            "VIBEOS_BOOT_TIMEOUT_SCALE": vm.stress_timeout_scale,
        }[env]()


def test_profiles_default_to_the_whole_matrix_and_reject_unknown():
    names = [p.name for p in stress.profiles(SimpleNamespace(profile="all"))]
    assert names == ["baseline", "slow-cpu", "lockstep", "starved-smp", "tight-clock"]
    only = stress.profiles(SimpleNamespace(profile="slow-cpu,lockstep", shift=4, smp=3))
    assert only[0].env == {"VIBEOS_QEMU_ICOUNT_SHIFT": "4"} and only[1].smp == 3
    with pytest.raises(SystemExit, match="unknown profile"):
        stress.profiles(SimpleNamespace(profile="hot-host"))


def _row(profile, ok, **kw):
    return {
        "profile": profile,
        "ok": ok,
        "duration_s": 60.0,
        "classification": "" if ok else "HANG_TIMEOUT",
        **kw,
    }


def test_diagnosis_reads_the_matrix():
    ran = ["baseline", "slow-cpu", "lockstep", "starved-smp", "tight-clock"]
    clean = [_row(p, True) for p in ran]
    assert "not it" in stress.diagnosis(clean)
    assert "bug" in stress.diagnosis([_row("baseline", False), *clean])
    assert "timeout" in stress.diagnosis([_row("slow-cpu", False), *clean])
    race = stress.diagnosis([_row("starved-smp", False), *clean])
    assert "race" in race and "lockstep" in race
    only_clock = stress.diagnosis([_row("tight-clock", False), *clean])
    assert "budget is the bug" in only_clock


def test_summary_counts_failure_families_per_profile():
    rows = [
        _row("lockstep", True),
        _row("lockstep", False, stop_code="0x0000000A", top_symbol="usbport.sys+0x30234"),
        _row("lockstep", False, stop_code="0x0000000A", top_symbol="usbport.sys+0x30234"),
    ]
    lines = stress.summarize(rows)
    assert lines[1].startswith("lockstep") and "   3    2" in lines[1]
    assert "usbport.sys+0x30234 x2" in lines[1]
