"""The hostile-QEMU profile (VIBEOS_QEMU_HOSTILE): hazard parsing, the injected
SPCR, the console UART it moves, and the -acpitable path the Windows QEMU needs."""

from pathlib import Path

import pytest

from .. import vm


def test_hazard_selection(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("VIBEOS_QEMU_HOSTILE", raising=False)
    assert vm.hostile_hazards() == set()
    monkeypatch.setenv("VIBEOS_QEMU_HOSTILE", "all")
    assert vm.hostile_hazards() == set(vm.HOSTILE_HAZARDS)
    monkeypatch.setenv("VIBEOS_QEMU_HOSTILE", "q35, serial")
    assert vm.hostile_hazards() == {"q35", "serial"}
    monkeypatch.setenv("VIBEOS_QEMU_HOSTILE", "q35,typo")
    with pytest.raises(ValueError, match="typo"):
        vm.hostile_hazards()


def test_spcr_is_a_valid_16550_table(tmp_path: Path) -> None:
    path = tmp_path / "spcr.bin"
    vm.make_spcr(path, 0x5000, 5)
    data = path.read_bytes()
    assert len(data) == 80 and data[:4] == b"SPCR" and sum(data) & 0xFF == 0
    assert data[36] == 0  # full 16550
    assert data[40] == 1 and int.from_bytes(data[44:52], "little") == 0x5000  # SystemIO GAS
    assert data[58] == 7  # 115200 baud


def test_console_uart_moves_only_under_the_serial_hazard(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("VIBEOS_QEMU_HOSTILE", raising=False)
    assert vm.console_uart_args("amd64-uefi", "ser0") == ["-serial", "chardev:ser0"]
    monkeypatch.setenv("VIBEOS_QEMU_HOSTILE", "serial")
    assert vm.console_uart_args("amd64-uefi", "ser0") == [
        "-device",
        "isa-serial,chardev=ser0,iobase=0x5000,irq=5",
    ]
    # Other configs keep COM1: the hazard is an amd64-uefi profile.
    assert vm.console_uart_args("i386-uefi", "ser0") == ["-serial", "chardev:ser0"]


def test_acpitable_path_has_no_drive_colon() -> None:
    args = ["-m", "6G", "-acpitable", "file=C:/temp/vibeos-qemu/run/spcr.bin", "-x", "file=C:/a"]
    out = vm._acpitable_relative(args)
    assert out[3] == "file=spcr.bin"
    assert out[5] == "file=C:/a"  # only -acpitable's value is rewritten


def test_arm64_hazards_reach_the_machine(monkeypatch: pytest.MonkeyPatch) -> None:
    cfg = vm.Arm64Config()
    monkeypatch.delenv("VIBEOS_QEMU_HOSTILE", raising=False)
    assert cfg.hostile_machine_args() == cfg.machine_args
    assert cfg.hostile_memory() == cfg.memory
    monkeypatch.setenv("VIBEOS_QEMU_HOSTILE", "a76,el2,mem")
    machine = cfg.hostile_machine_args()
    assert machine[machine.index("-M") + 1] == "virt,virtualization=on"
    assert machine[machine.index("-cpu") + 1] == "cortex-a76"
    assert cfg.hostile_memory() == "6G"
    # machine_args itself is never mutated: a later plain boot stays plain.
    plain = vm.Arm64Config().machine_args
    assert plain[plain.index("-cpu") + 1] == "cortex-a72"
