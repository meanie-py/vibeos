# wowx64.dll — the OS side of BTCpu64 (x86-64 guests on arm64). ntdll's process
# init calls WowX64LdrpInitialize for an amd64 main image on an arm64 host.
@ stdcall WowX64LdrpInitialize(ptr)
# ntdll's thread init hands it a thread a NATIVE process created in this guest
# process (CreateRemoteThread) whose start is not native code.
@ stdcall WowX64StartGuestThreadFromNative(ptr)
