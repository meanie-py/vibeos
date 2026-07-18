# vibeos wow64cpu.dll — native amd64 WOW64 CPU layer (Phase E). Exports exactly
# the BTCpu* contract the shared wow64.dll host layer resolves by name (the same
# surface arm64's libwow64fex.dll exports). amd64-only.
@ stdcall -arch=x86_64 BTCpuProcessInit()
@ stdcall -arch=x86_64 BTCpuThreadInit()
@ stdcall -arch=x86_64 BTCpuGetBopCode()
@ stdcall -arch=x86_64 BTCpuSimulate()
@ stdcall -arch=x86_64 BTCpuNotifyMapViewOfSection(ptr ptr ptr long long long)
@ stdcall -arch=x86_64 BTCpuSetContext(ptr ptr ptr ptr)
@ stdcall -arch=x86_64 BTCpuGetContext(ptr ptr ptr ptr)
