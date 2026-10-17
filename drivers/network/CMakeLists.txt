
add_subdirectory(afd)
add_subdirectory(dd)
add_subdirectory(ndis)
add_subdirectory(ndisuio)
add_subdirectory(netio)
add_subdirectory(netstack)
# tcpip.sys (ip/ + lwip/) was REPLACED by netstack.sys (smoltcp, Rust) — the L4
# net-stack rewrite. netstack owns all four TDI devices (\Device\Tcp/Udp/Ip/RawIp).
# The tcpip sources (and drivers/network/lan, the old stack's LAN-adapter layer,
# which had no CMakeLists and was unreachable) were DELETED 2026-07-18 after the
# replacement had been green for some time: ~189K lines, self-contained, nothing
# outside referenced them. media/inf/nettcpip.inf keeps the "Tcpip" SERVICE name
# (dhcpcsvc/ws2_32 read Services\Tcpip\Parameters by literal path) but now points
# ServiceBinary at netstack.sys. See docs/engineering/smoltcp-netstack-design.md.
add_subdirectory(tdi)
add_subdirectory(tdihelpers)
