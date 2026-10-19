"""Two adapters at once, and the stack moving between them when one loses its link.

The Pi 5 carries its GEM and its Wi-Fi side by side; this is the same shape in QEMU,
where a link can be cut on demand. Two NICs, each on its own slirp network:

    wired  e1000 (8086:100E), which netstack drives itself    10.0.3.0/24  adapter 0
    ndis   e1000-82544gc (8086:100C), through the NDIS miniport 10.0.2.0/24  adapter 1

Run it on its own (it sets VIBEOS_QEMU_EXTRA_ARGS for the session's boot):

    pytest tests/qemu/test_multinic.py --arch amd64-uefi --timeout 900

Each adapter gets a lease of its own; cutting the wired link (QMP set_link) drops its
lease and makes adapter 1 the primary; restoring it brings adapter 0's lease back
without taking the stack's sockets back from adapter 1.
"""

from __future__ import annotations

import os
import time

from tests.qemu.vm import QemuVM

# Read by vm.py when the session's VM launches, which is after collection.
os.environ["VIBEOS_QEMU_EXTRA_ARGS"] = (
    "-nic user,id=wired,net=10.0.3.0/24,model=e1000 -nic user,id=ndis,model=e1000-82544gc"
)

LEASE_WIRED = "[NETSTACK] dhcp lease 10.0.3.15 on adapter 0"
LEASE_NDIS = "[NETSTACK] dhcp lease 10.0.2.15 on adapter 1"
WIRED_DOWN = "[NETSTACK] adapter 0: media disconnected"
WIRED_LOST = "[NETSTACK] dhcp lease lost on adapter 0"
WIRED_UP = "[NETSTACK] adapter 0: media connected"
PRIMARY_NDIS = "[NETSTACK] primary adapter: 1"
PRIMARY_WIRED = "[NETSTACK] primary adapter: 0"


def _log(vm: QemuVM) -> str:
    try:
        return vm.live_log_path.read_text(errors="replace")
    except OSError:
        return vm._boot_output


def _wait(vm: QemuVM, marker: str, count: int = 1, timeout: float = 180.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if _log(vm).count(marker) >= count:
            return
        time.sleep(0.5)
    tail = _log(vm)[-1500:]
    raise AssertionError(f"{marker!r} (x{count}) never appeared; serial tail:\n{tail}")


def test_two_adapters_and_failover(vm: QemuVM) -> None:
    _wait(vm, LEASE_WIRED)
    _wait(vm, LEASE_NDIS)

    # Counted from here: the fixture restores a snapshot taken at the desktop, so
    # anything after it in the guest's life can appear in the log twice.
    base = {m: _log(vm).count(m) for m in (WIRED_DOWN, WIRED_LOST, PRIMARY_NDIS, PRIMARY_WIRED)}
    vm.qmp.execute("set_link", name="wired", up=False)
    for m in (WIRED_DOWN, WIRED_LOST, PRIMARY_NDIS):
        _wait(vm, m, count=base[m] + 1, timeout=60)

    leases = _log(vm).count(LEASE_WIRED)
    up = _log(vm).count(WIRED_UP)
    vm.qmp.execute("set_link", name="wired", up=True)
    _wait(vm, WIRED_UP, count=up + 1, timeout=60)
    _wait(vm, LEASE_WIRED, count=leases + 1, timeout=120)
    assert _log(vm).count(PRIMARY_WIRED) == base[PRIMARY_WIRED], (
        "the stack's sockets moved back to adapter 0; they should stay on adapter 1"
    )
