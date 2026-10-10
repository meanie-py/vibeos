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


# --- --sample-pcs ---------------------------------------------------------------

_REGS = """CPU#0
RAX=0000000000000000 RBX=0000000000000001
RIP=fffff80000401010 RFL=00000246 [---Z-P-] CPL=0 II=0 A20=1 SMM=0 HLT=1
RBP=fffff80000900f00
CPU#1
RIP=fffff80000402008 RFL=00000202 [-------] CPL=0 II=0 A20=1 SMM=0 HLT=0
RBP=fffff80000901f00
"""


def test_a_sample_tells_a_halted_vcpu_from_a_spinning_one():
    from tests.qemu.cli import pcsample

    rows = pcsample.parse_sample(_REGS)
    assert [(r["cpu"], r["pc"], r["halted"]) for r in rows] == [
        (0, "fffff80000401010", True),
        (1, "fffff80000402008", False),
    ]
    assert rows[1]["fp"] == "fffff80000901f00"


def test_report_ranks_only_non_halted_samples_and_resolves_through_the_index(tmp_path):
    import json

    from tests.qemu.cli import pcsample

    (tmp_path / "qemu").mkdir()
    (tmp_path / "qemu" / "symbols-amd64-mp.tsv").write_text(
        "fffff80000401000\tKiIdleLoop\tidle.rs:1\nfffff80000402000\tKxAcquireSpinLock\tspin.rs:1\n"
    )
    rows = [
        {"t": 0, "cpu": 0, "pc": "fffff80000401010", "halted": True, "callers": []},
        {
            "t": 0,
            "cpu": 1,
            "pc": "fffff80000402008",
            "halted": False,
            "callers": ["fffff80000401020"],
        },
        {
            "t": 1,
            "cpu": 1,
            "pc": "fffff80000402010",
            "halted": False,
            "callers": ["fffff80000401020"],
        },
        {"t": 1, "cpu": 0, "pc": "00007ff600001000", "halted": False, "callers": []},
    ]
    samples = tmp_path / "s.jsonl"
    samples.write_text("".join(json.dumps(r) + "\n" for r in rows))
    out = "\n".join(pcsample.report(tmp_path, "amd64", samples, "loaded ntkrnlmp.exe"))
    assert "CPU#0: 2 samples, 50% halted" in out
    assert "66.7%  KxAcquireSpinLock" in out  # 2 of the 3 non-halted samples
    assert "33.3%  (user mode)" in out
    assert "KxAcquireSpinLock  <-  KiIdleLoop" in out  # no stack scan: falls back to the fp chain


def test_an_mp_boot_never_resolves_against_the_up_index(tmp_path):
    from tests.qemu.cli import pcsample

    (tmp_path / "qemu").mkdir()
    (tmp_path / "qemu" / "symbols-amd64.tsv").write_text("fffff80000401000\tWrongBinary\tx:1\n")
    samples = tmp_path / "s.jsonl"
    samples.write_text('{"t":0,"cpu":0,"pc":"fffff80000401010","halted":false,"callers":[]}\n')
    with pytest.raises(FileNotFoundError, match="-mp"):
        pcsample.report(tmp_path, "amd64", samples, "ntkrnlmp.exe")


def test_likely_caller_skips_the_pcs_own_function_and_stale_duplicates():
    from tests.qemu.cli import pcsample

    sym = {
        "fffff80000402008": "KxAcquireSpinLock+0x8",
        "fffff80000402100": "KxAcquireSpinLock+0x100",
        "fffff80000403000": "MiDeletePte+0x0",
        "fffff80000404000": "MmFreePages+0x0",
    }
    row = {
        "pc": "fffff80000402008",
        "stack": ["fffff80000402100", "fffff80000403000", "fffff80000403000", "fffff80000404000"],
    }
    assert pcsample._likely_caller(row, sym) == "MiDeletePte <- MmFreePages"


def test_stack_scan_keeps_kernel_half_words_in_the_monitors_real_format():
    from tests.qemu.cli import pcsample

    class _Qmp:
        def hmp(self, cmd: str) -> str:
            if not cmd.startswith("x "):
                return ""
            # verbatim from a 2026-10-09 hang-cpustate.txt
            return (
                "fffff88075627880: 0xfffff880756278b0 0xfffff80000227843\r\n"
                "fffff88075627890: 0xfffff88075627920 0x00000e66d59033ad\r\n"
            )

    words = pcsample._stack_words(_Qmp(), 1, "fffff88075627880")
    assert words == ["fffff880756278b0", "fffff80000227843", "fffff88075627920"]
