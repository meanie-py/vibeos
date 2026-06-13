"""QEMU network tests — the smoltcp netstack check ladder.

Part of the smoltcp netstack lane (docs/engineering/smoltcp-netstack-design.md).
Deliberately SEPARATE from boot smoke: smoke proves the desktop boots, this
proves the network stack works. Run it explicitly:

    pytest tests/qemu/test_net.py -v --arch amd64-uefi-net --timeout 600

The amd64-uefi-net config (vm.py:Amd64UefiNetConfig) is the real UEFI smoke
boot plus a virtio-net-pci NIC on a slirp user netdev:
    network 10.0.2.0/24 · gateway 10.0.2.2 · DNS 10.0.2.3 · built-in DHCP
    host TCP echo path: 127.0.0.1:<echo_hostfwd_port> -> guest :7777

Serial marker contract — emitted by the netstack-rust kernel self-test as the
milestones land. These strings are the API between the kernel self-test and
this harness; change them in both places or not at all.

    [NETSTACK] M1 raw frame RX/TX OK      M1: ARP round-trip with slirp gateway
    [NETSTACK] ping gateway OK            M2: ICMP echo to 10.0.2.2 answered
    [NETSTACK] tcp echo ready             M2: TCP echo listener up on :7777
    [NETSTACK] dhcp lease 10.0.2.x        M3: DHCPv4 lease obtained from slirp
    [NETSTACK] dns OK                     M3: A-record resolved via 10.0.2.3

M0-M3 all pass today. Milestones land behind xfail(strict=True) ratchets while
in progress, then flip to real asserts the moment the engine passes them — an
honest ratchet, never a weakened test. M4+ (TDI/AFD) checks get added the same
way.
"""

from __future__ import annotations

import socket

import pytest

from tests.qemu.vm import QemuVM

# Markers — keep in sync with the netstack-rust self-test (see module docstring).
MARK_M1_OK = "[NETSTACK] M1 raw frame RX/TX OK"
MARK_PING = "[NETSTACK] ping gateway OK"
MARK_ECHO_READY = "[NETSTACK] tcp echo ready"
MARK_DHCP = "[NETSTACK] dhcp lease 10.0.2."
MARK_DNS = "[NETSTACK] dns OK"
MARK_TDI_OWNED = "[NETSTACK] TDI dispatch ready (4 of 4 devices owned)"
MARK_OUT_SENT = "[NETSTACK] outbound TCP connected"
MARK_OUT_RECV = "[NETSTACK] outbound TCP echo received"
MARK_TDI_CONNECT = "[NETSTACK] TDI self-test: CONNECT OK"
MARK_TDI_RT = "[NETSTACK] TDI self-test: RECEIVE OK"

def _require_net_config(vm: QemuVM) -> None:
    if vm.config.arch != "amd64-uefi-net":
        pytest.skip("net tests need the net-enabled config: --arch amd64-uefi-net")


# --------------------------------------------------------------------------- #
# M0 gate — GREEN today: the NIC's presence must not break or stall boot.
# --------------------------------------------------------------------------- #

def test_boot_with_nic(vm: QemuVM) -> None:
    """Boot reaches the full desktop with a virtio-net-pci NIC attached.

    The vm fixture already failed the session if the boot marker never fired;
    this asserts the full milestone chain so a NIC-induced regression (PCI
    enumeration choking on VEN_1AF4, a driver wedging PnP, boot stalling on
    an idle NIC) is attributed to the net config, not mistaken for flake.
    """
    _require_net_config(vm)
    for milestone in vm.config.required_milestones:
        assert milestone in vm._boot_output, (
            f"milestone {milestone!r} missing from boot output with NIC attached"
        )


# --------------------------------------------------------------------------- #
# M1 gate — GREEN: raw virtio-net frame RX/TX, proven by an ARP round-trip with
# the slirp gateway (no smoltcp, no TDI). The reply crossing the RX virtqueue
# proves both directions of the device path work.
# --------------------------------------------------------------------------- #

def test_m1_raw_frame_roundtrip(vm: QemuVM) -> None:
    """netstack.sys DriverEntry self-test ARP'd 10.0.2.2 and got slirp's reply."""
    _require_net_config(vm)
    assert MARK_M1_OK in vm._boot_output, (
        "M1 self-test marker missing — virtio-net raw RX/TX did not complete"
    )


# --------------------------------------------------------------------------- #
# M2 — engine self-test: ICMP + TCP echo (no TDI, driven inside the kernel).
# --------------------------------------------------------------------------- #

def test_selftest_ping_gateway(vm: QemuVM) -> None:
    """Kernel self-test pinged the slirp gateway (10.0.2.2) and got a reply."""
    _require_net_config(vm)
    assert MARK_PING in vm._boot_output


def test_tcp_echo_roundtrip(vm: QemuVM) -> None:
    """Host connects through the hostfwd to the guest echo port; bytes bounce.

    This is the first HOST-VISIBLE proof of TX+RX: frames crossed the virtqueue
    both ways and smoltcp ran a real TCP handshake + data exchange.
    """
    _require_net_config(vm)
    assert MARK_ECHO_READY in vm._boot_output, "guest never announced the echo listener"
    payload = b"vibeos-netstack-echo-probe\n"
    with socket.create_connection(
        ("127.0.0.1", vm.config.echo_hostfwd_port), timeout=10.0
    ) as s:
        s.settimeout(10.0)
        s.sendall(payload)
        got = b""
        while len(got) < len(payload):
            chunk = s.recv(4096)
            if not chunk:
                break
            got += chunk
    assert got == payload, f"echo mismatch: sent {payload!r}, got {got!r}"


# --------------------------------------------------------------------------- #
# M3 — DHCPv4 lease + DNS through slirp's built-in servers.
# --------------------------------------------------------------------------- #

def test_dhcp_lease(vm: QemuVM) -> None:
    """Kernel self-test obtained a DHCPv4 lease from slirp (10.0.2.x)."""
    _require_net_config(vm)
    assert MARK_DHCP in vm._boot_output


def test_dns_resolve(vm: QemuVM) -> None:
    """Kernel self-test resolved an A record via the slirp DNS forwarder."""
    _require_net_config(vm)
    assert MARK_DNS in vm._boot_output


# --------------------------------------------------------------------------- #
# M4 foundation — netstack owns the TDI transport surface (\Device\Tcp etc.) that
# AFD/winsock open. The data path + completion bridge + a usermode winsock
# round-trip are the remaining M4 work.
# --------------------------------------------------------------------------- #

def test_tdi_devices_owned(vm: QemuVM) -> None:
    r"""netstack created + owns all four TDI devices (\Device\Tcp/Udp/Ip/RawIp)."""
    _require_net_config(vm)
    assert MARK_TDI_OWNED in vm._boot_output, (
        "netstack did not own the 4 TDI transport devices (tcpip collision?)"
    )


def test_outbound_tcp_client(vm: QemuVM) -> None:
    """The netstack engine connects OUT through the stack and round-trips data.

    Proves the smoltcp connect/send/recv client path (the path the TDI/AFD layer
    will sit on): guest -> virtio TX -> slirp guestfwd -> `cat` echo on the host
    -> back. The connect happens at boot (after DHCP), so the markers land in the
    boot output. A full client round-trip = connect + send + recv all work.
    """
    _require_net_config(vm)
    assert MARK_OUT_SENT in vm._boot_output, "outbound TCP connect/send did not complete"
    assert MARK_OUT_RECV in vm._boot_output, "outbound TCP recv (echo back) did not complete"


def test_tdi_selftest_roundtrip(vm: QemuVM) -> None:
    r"""M4: a kernel TDI client drives netstack's own \Device\Tcp through the
    EXACT path AFD/winsock uses — open an address object + a connection object
    (both via EA buffers: TransportAddress / ConnectionContext), ASSOCIATE,
    CONNECT, SEND, RECEIVE — and gets the echo back through smoltcp + slirp.

    This is the end-to-end proof of the TDI dispatch + message-passing inbox +
    smoltcp completion bridge. CONNECT completing proves the bridge (dispatch
    pends the IRP, the poll worker completes it on socket ESTABLISHED); RECEIVE
    OK proves the full SEND->host-echo->RECEIVE data path with real MDLs.
    """
    _require_net_config(vm)
    assert MARK_TDI_CONNECT in vm._boot_output, (
        "TDI CONNECT did not complete (dispatch/inbox/completion-bridge path)"
    )
    assert MARK_TDI_RT in vm._boot_output, (
        "TDI SEND/RECEIVE round-trip did not complete (MDL data path)"
    )
