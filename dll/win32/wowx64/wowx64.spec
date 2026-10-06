# wowx64.dll — the OS side of BTCpu64 (x86-64 guests on arm64). ntdll's process
# init calls WowX64LdrpInitialize for an amd64 main image on an arm64 host.
@ stdcall WowX64LdrpInitialize(ptr)
