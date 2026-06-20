
add_subdirectory(afd)
add_subdirectory(dd)
add_subdirectory(ndis)
add_subdirectory(ndisuio)
add_subdirectory(netio)
add_subdirectory(netstack)
# tcpip.sys (ip/ + lwip/, ~129K C) is REPLACED by netstack.sys (smoltcp, Rust) —
# the L4 net-stack rewrite. netstack owns all four TDI devices (\Device\Tcp/Udp/
# Ip/RawIp) behind the same TDI interface AFD/ws2_32 use, so nothing links the
# tcpip/ip/lwipcore targets and nothing references tcpip.sys in txtsetup/bootcd
# lists. M6: the C stack is dropped from the build. Source kept in-tree (revertible,
# and nettcpip.inf's Winsock2 catalog is still referenced from there). See
# docs/engineering/smoltcp-netstack-design.md.
# add_subdirectory(tcpip)
add_subdirectory(tdi)
