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
import time

import pytest

from tests.qemu.vm import (
    Amd64UefiNetConfig,
    Arm64NetConfig,
    I386UefiNetConfig,
    QemuVM,
)

_NET_CONFIG_TYPES = (Amd64UefiNetConfig, I386UefiNetConfig, Arm64NetConfig)


def _net_config(vm: QemuVM) -> Amd64UefiNetConfig | I386UefiNetConfig | Arm64NetConfig:
    """vm.config as the net config it must be: only those forward the guest ports."""
    cfg = vm.config
    assert isinstance(cfg, _NET_CONFIG_TYPES), f"{type(cfg).__name__} forwards no guest ports"
    return cfg


def _wait_for_marker(vm: QemuVM, marker: str, timeout: float = 90.0) -> bool:
    """Poll the live serial log for a marker that may land AFTER boot completes.

    The kernel TDI self-test runs concurrently with boot and finishes its
    connect/send/receive round-trip late (after `shell ready`), so its markers
    are not reliably inside the static `_boot_output` capture window. The live
    log is appended for the VM's whole lifetime, so poll it instead.
    """
    if marker in vm._boot_output:
        return True
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            text = vm.live_log_path.read_text(errors="replace")
        except OSError:
            text = ""
        if marker in text:
            return True
        time.sleep(0.5)
    return False


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
MARK_TDI_DGRAM = "[NETSTACK] TDI dgram self-test: RECEIVE_DATAGRAM OK"
MARK_TDI_BCAST = "[NETSTACK] TDI dgram self-test: BROADCAST local delivery OK"
MARK_TDI_SRV_LISTEN = "[NETSTACK] TDI server: listening on :8888"
MARK_TDI_SRV_ECHO = b"TDI-SRV-ECHO"
MARK_USERMODE_TCP = "[NETTEST] usermode winsock TCP echo OK"
MARK_USERMODE_UDP = "[NETTEST] usermode winsock UDP DNS OK"
MARK_ICMP_IOCTL = "[NETSTACK] ICMP self-test: reply OK"
MARK_SET_IOCTL = "[NETSTACK] SET_INFO self-test: route program OK"
MARK_M10_CONNECTED = "[NETSTACK] M10: interrupt-driven RX connected"
MARK_M10_IRQ = "[NETSTACK] M10: first RX interrupt serviced"

_NET_CONFIGS = ("amd64-uefi-net", "i386-uefi-net", "arm64-net")


def _require_net_config(vm: QemuVM) -> None:
    if vm.config.arch not in _NET_CONFIGS:
        pytest.skip(f"net tests need a net-enabled config: --arch {' / '.join(_NET_CONFIGS)}")


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
    # net_host is 127.0.0.1 on native; on the windows-host backend it's the WSL
    # default-gateway IP (the Windows QEMU's 0.0.0.0 hostfwd isn't on WSL's loopback).
    with socket.create_connection(
        (vm.net_host, _net_config(vm).echo_hostfwd_port), timeout=10.0
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
    """Kernel self-test resolved an A record via the slirp DNS forwarder.

    POLLS the live log rather than reading the static `_boot_output` snapshot,
    and it is the only one of the netstack self-test markers that needs to.
    Measured on a healthy boot (~1000 lines of serial output):

        [NETSTACK] tcp echo ready    line 863    (~140 lines of headroom)
        [NETSTACK] dhcp lease        line 864    (~139)
        [NETSTACK] ping gateway OK   line 866    (~137)
        [NETSTACK] dns OK            line 981    (   22)   <-- ~2% margin
        shell ready                  line 1003   (the _boot_output cutoff)

    DNS is the LAST thing to land before boot completes, and it is the only
    marker in this suite whose latency depends on a resolver OUTSIDE the machine
    (slirp forwards 10.0.2.3 to the host's DNS). A 2% margin against an external
    round-trip is a race, not a contract: any perturbation of the boot — host
    load, a slow upstream resolver, a concurrent gate — eats it, and the test
    goes red for a reason that has nothing to do with the OS.

    That is exactly how it failed during the msvcrt wave-4 NULL regression
    (§9.64): the real bug perturbed the boot, DNS lost its 22-line margin, and
    `test_dns_resolve` fired alongside the genuine failure — sending the triage
    at a kernel-side marker that user-mode msvcrt cannot even reach.

    This is STRICTLY STRONGER than the snapshot assert, not weaker:
    `_wait_for_marker` checks `_boot_output` FIRST and returns immediately on a
    hit, so it is a superset — it can only ever convert a spurious FAIL into a
    PASS. The marker is still REQUIRED; a genuinely broken DNS still goes red,
    just after a bounded wait. The timeout is deliberately tight (30s, vs the
    90s default) so a DNS path that regresses into being *slow* is still caught.

    The other five netstack markers keep their snapshot asserts on purpose —
    they land with ~140 lines of headroom and are not at risk; polling them would
    add tolerance where none is needed.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_DNS, timeout=30.0), (
        "kernel DNS self-test never resolved an A record via the slirp forwarder "
        "(10.0.2.3 -> host resolver)"
    )


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
    will sit on): guest -> virtio TX -> slirp guestfwd -> host echo -> back. The
    connect happens at boot, but the engine retries on no-echo (the windows-host
    backend's shared-chardev guestfwd drops the FIRST connection's return), so the
    success can land slightly after the boot snapshot — poll the live log.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_OUT_SENT), "outbound TCP connect/send did not complete"
    assert _wait_for_marker(vm, MARK_OUT_RECV), "outbound TCP recv (echo back) did not complete"


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
    assert _wait_for_marker(vm, MARK_TDI_CONNECT), (
        "TDI CONNECT did not complete (dispatch/inbox/completion-bridge path)"
    )
    assert _wait_for_marker(vm, MARK_TDI_RT), (
        "TDI SEND/RECEIVE round-trip did not complete (MDL data path)"
    )


def test_icmp_echo_ioctl(vm: QemuVM) -> None:
    r"""M9(3): the ICMP echo (ping) IOCTL on \Device\Ip. The kernel self-test
    opens \Device\Ip and issues IOCTL_ICMP_ECHO_REQUEST (METHOD_BUFFERED) to the
    gateway, then validates the ICMP_ECHO_REPLY the worker writes back.

    This is the exact path usermode `ping` / iphlpapi's IcmpSendEcho2 ride on:
    dispatch parses the request → posts Cmd::IcmpEcho → the smoltcp worker emits
    an echo on the management ICMP socket → matches the reply by sequence number
    → fills ICMP_ECHO_REPLY (data at buffer end + Data back-pointer) → completes
    the pended IRP. The marker fires only on IP_SUCCESS with a reply count >= 1.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_ICMP_IOCTL, timeout=120), (
        "ICMP echo IOCTL round-trip did not complete (ping path: \\Device\\Ip "
        "IOCTL_ICMP_ECHO_REQUEST dispatch + cross-thread ICMP plumbing)"
    )


def test_set_info_ex_ioctl(vm: QemuVM) -> None:
    r"""M9(4): the management-plane SET IOCTL on \Device\Tcp. The kernel self-test
    opens \Device\Tcp and issues IOCTL_TCP_SET_INFORMATION_EX (METHOD_BUFFERED)
    carrying a default-route IPRouteEntry — the exact request iphlpapi's
    createIpForwardEntry / the usermode `route` command / dhcpcsvc emit.

    Proves the dispatch arm parses TCP_REQUEST_SET_INFORMATION_EX, posts
    Cmd::SetDefaultRoute to the worker (which reprograms the smoltcp default
    route), and acknowledges the IRP. The marker fires only on STATUS_SUCCESS.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_SET_IOCTL, timeout=120), (
        "SET_INFORMATION_EX route-program IOCTL did not complete "
        "(\\Device\\Tcp IOCTL_TCP_SET_INFORMATION_EX dispatch + worker route program)"
    )


def test_interrupt_driven_rx(vm: QemuVM) -> None:
    r"""M10: the virtio-net legacy INTx is connected to netstack's ISR and
    actually fires, on every arch. The ISR acks the device (reads+clears the ISR
    status, deasserts INTx) and queues a DPC; the DPC sets the worker's RX event;
    the worker wakes on that event instead of busy-polling.

    Two proofs: (1) the interrupt CONNECTED (IoConnectInterrupt +
    HalGetInterruptVector resolved a real vector for the manually-enumerated NIC;
    on arm64 the vector is the GIC INTID of the board's GPEX INTx routing);
    (2) it FIRED — the one-shot marker is emitted from the DPC, so it can only
    appear if a real device IRQ drove the ISR→DPC→event path (not the worker's
    fallback timeout).
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_M10_CONNECTED, timeout=90), (
        "virtio RX interrupt was not connected (IoConnectInterrupt / HalGetInterruptVector(PCIBus))"
    )
    assert _wait_for_marker(vm, MARK_M10_IRQ, timeout=90), (
        "RX interrupt never fired (ISR->DPC->event path) — worker is polling-only"
    )


def test_tdi_datagram_roundtrip(vm: QemuVM) -> None:
    r"""M5: the UDP/connectionless analogue of the TDI round-trip — a kernel TDI
    client drives netstack's own \Device\Udp through the EXACT path AFD uses for
    sendto/recvfrom (and the usermode DHCP/DNS services): open an ADDRESS object
    (TransportAddress EA), register a TDI_EVENT_RECEIVE_DATAGRAM handler, then
    TDI_SEND_DATAGRAM a real DNS query to slirp's DNS forwarder (10.0.2.3:53) and
    receive the reply via the datagram event-indication upcall.

    Proves the UDP send path + the connectionless completion bridge (smoltcp
    udp::Socket bind/send/recv) + the datagram upcall AFD relies on for recvfrom.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_TDI_DGRAM), (
        "TDI SEND_DATAGRAM/RECEIVE_DATAGRAM round-trip did not complete "
        "(UDP datagram path + datagram event-indication upcall)"
    )


def test_tdi_broadcast_local_delivery(vm: QemuVM) -> None:
    """A UDP broadcast this host sends reaches a local socket bound to that port.

    NDIS never loops a broadcast back to its sender, and smoltcp handles broadcast only
    on egress, so without the local delivery in tditrans.rs a second socket on the
    same host never sees it (NetBIOS, SSDP-style self-discovery). The self-test sends
    to the limited (255.255.255.255) and the directed (10.0.2.255) broadcast and needs
    both. Upstream tcpip fixed the same gap in b8ee7950af4.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_TDI_BCAST), (
        "a broadcast sent through TDI never reached the local listener bound to its port"
    )


def test_tdi_server_listen_accept(vm: QemuVM) -> None:
    r"""M8: the SERVER/inbound mirror of the TDI round-trip. netstack's kernel TDI
    server self-test (tdisrvtest.rs) binds :8888, posts a TDI_LISTEN that PENDS,
    and on a host connect ACCEPTs + echoes a marker back — the exact IRP path
    AfdListenSocket/AfdAccept use (no event-upcall subsystem; a pended listen IRP).

    The host connects through the second hostfwd, sends a probe, and reads the
    fixed echo marker. Proves the M8 listen/accept dispatch + completion bridge
    (TDI_LISTEN pends -> smoltcp finishes the handshake in place -> bridge
    completes the pended listen IRP) plus the post-accept receive-event + TDI_SEND
    on a server-accepted connection. This is the inbound counterpart to the
    outbound client (test_tdi_selftest_roundtrip).
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_TDI_SRV_LISTEN, timeout=120), (
        "netstack kernel TDI server never announced listening on :8888 "
        "(M8 TDI_LISTEN dispatch / bind path)"
    )
    payload = b"vibeos-tdi-server-probe\n"
    with socket.create_connection(
        (vm.net_host, _net_config(vm).tdi_srv_hostfwd_port), timeout=10.0
    ) as s:
        s.settimeout(15.0)
        s.sendall(payload)
        got = b""
        while MARK_TDI_SRV_ECHO not in got:
            chunk = s.recv(4096)
            if not chunk:
                break
            got += chunk
    assert MARK_TDI_SRV_ECHO in got, (
        "netstack TDI server accepted the connection but did not echo "
        f"(M8 accept -> receive-event -> TDI_SEND): got {got!r}"
    )


def test_usermode_winsock_roundtrip(vm: QemuVM) -> None:
    r"""M5 capstone (BLOCKED — see skip reason): a REAL usermode winsock process
    (nettest.exe, spawned by winlogon at desktop) drives the full stack from the
    top — ws2_32 -> msafd -> AFD -> \Device\Tcp|Udp -> netstack -> smoltcp -> slirp.
    Closes the loop the kernel self-tests open from the TDI side.
    """
    _require_net_config(vm)
    assert _wait_for_marker(vm, MARK_USERMODE_TCP, timeout=120), (
        "usermode winsock TCP round-trip did not complete (ws2_32 -> AFD -> netstack TCP path)"
    )
    assert _wait_for_marker(vm, MARK_USERMODE_UDP, timeout=120), (
        "usermode winsock UDP round-trip did not complete "
        "(ws2_32 -> AFD -> netstack UDP datagram path)"
    )
