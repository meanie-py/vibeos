/*
 * wow64probe — WOW64 Phase B harness (docs/engineering/wow64-plan.md).
 *
 * fexprobe proved FEXCore's Context API runs x86 blobs on vibeos arm64.
 * This probe moves one layer up: it drives the REAL vendored CPU layer
 * (libwow64fex.dll) through its BTCpu* contract, standing in for the 64-bit
 * loader + wow64.dll bootstrap that Phase C builds into the OS. It fakes, in
 * one process, everything the real WoW64 process bootstrap would provide:
 *
 *   - LdrSystemDllInitBlock.ntdll_handle -> a mapped i386 ntdll.dll
 *   - TEB->TlsSlots[WOW64_TLS_WOW64INFO=10] -> a WOW64INFO
 *   - TEB->TlsSlots[WOW64_TLS_CPURESERVED=1] -> WOW64_CPURESERVED + WOW64_CONTEXT
 *   - guest code + stack below 4GB (ZeroBits-mask allocations)
 *
 * then calls BTCpuProcessInit / BTCpuThreadInit / BTCpuSimulate and verifies a
 * guest x86 syscall round-trips through wow64.dll's Wow64SystemServiceEx probe
 * services (ECHO -> guest -> VERDICT). PASS exits 0x600D via NtTerminateProcess
 * from inside the VERDICT thunk — BTCpuSimulate never returns.
 *
 * Native arm64 PE, NO FEXCore link — the point is exercising the .dll contract.
 * All output via DbgPrint -> serial, [FEX-WOW64B] prefix.
 */

#include <windows.h>
#include <cstdint>
#include <cstring>

extern "C" {
ULONG WINAPI DbgPrint(const char* Format, ...);
LONG WINAPI NtAllocateVirtualMemory(HANDLE Process, PVOID* BaseAddress, ULONG_PTR ZeroBits,
                                    SIZE_T* RegionSize, ULONG AllocationType, ULONG Protect);
LONG WINAPI NtCreateSection(HANDLE* SectionHandle, ACCESS_MASK DesiredAccess, void* ObjectAttributes,
                            PLARGE_INTEGER MaximumSize, ULONG SectionPageProtection,
                            ULONG AllocationAttributes, HANDLE FileHandle);
LONG WINAPI NtMapViewOfSection(HANDLE SectionHandle, HANDLE ProcessHandle, PVOID* BaseAddress,
                               ULONG_PTR ZeroBits, SIZE_T CommitSize, PLARGE_INTEGER SectionOffset,
                               SIZE_T* ViewSize, ULONG InheritDisposition, ULONG AllocationType,
                               ULONG Win32Protect);
}

#define PROBE(fmt, ...) DbgPrint("[FEX-WOW64B] " fmt "\n", ##__VA_ARGS__)

namespace {

// Crash oracle for the BTCpu* bring-up (fexprobe's ProbeVeh methodology one
// layer up): on any exception, dump the full register file + a stack window
// so a serial log + the dll base line is enough to symbolize offline.
uintptr_t g_FexBase;
size_t g_FexSize;

// The CPU layer's exception entry — in a real WoW64 process ntdll/wow64.dll
// route every exception here BEFORE user dispatch (it services LookupCache
// overcommit faults, SMC, suspend interrupts, and guest-fault reconstruction,
// NtContinue-ing itself when handled). The probe stands in for that plumbing.
NTSTATUS (*g_ResetToConsistentState)(EXCEPTION_POINTERS*);

LONG WINAPI ProbeVeh(EXCEPTION_POINTERS* Info)
{
    auto* Rec = Info->ExceptionRecord;
    auto* Ctx = Info->ContextRecord;

    if (g_ResetToConsistentState) {
        g_ResetToConsistentState(Info); // NtContinues (no return) if handled
    }
    PROBE("VEH: code=%08lx addr=%p  (libwow64fex base=%p size=%zx)",
          Rec->ExceptionCode, Rec->ExceptionAddress, reinterpret_cast<void*>(g_FexBase), g_FexSize);
    PROBE("VEH: Pc=%p Lr=%p Sp=%p Fp=%p", reinterpret_cast<void*>(Ctx->Pc),
          reinterpret_cast<void*>(Ctx->Lr), reinterpret_cast<void*>(Ctx->Sp),
          reinterpret_cast<void*>(Ctx->Fp));
    for (int i = 0; i < 28; i += 4) {
        PROBE("VEH: X%02d %016llx %016llx %016llx %016llx", i,
              Ctx->X[i], Ctx->X[i + 1], Ctx->X[i + 2], Ctx->X[i + 3]);
    }
    const auto* Stack = reinterpret_cast<const uint64_t*>(Ctx->Sp);
    for (int i = 0; i < 48; i += 4) {
        PROBE("VEH: sp+%03x %016llx %016llx %016llx %016llx", i * 8,
              Stack[i], Stack[i + 1], Stack[i + 2], Stack[i + 3]);
    }
    return EXCEPTION_CONTINUE_SEARCH;
}

} // namespace

namespace {

// ---- Contract constants (vendor/fex/Source/Windows/include/winternl.h +
// ---- WOW64/Module.cpp; our ntdll wow64shim.rs must agree) -----------------
constexpr size_t kTebTlsSlotsOffset = 0x1480;   // TEB64->TlsSlots[]
constexpr size_t kTlsSlotWow64Info = 10;        // WOW64_TLS_WOW64INFO
constexpr size_t kTlsSlotCpuReserved = 1;       // WOW64_TLS_CPURESERVED
constexpr size_t kInitBlockNtdllHandleOff = 72; // SYSTEM_DLL_INIT_BLOCK.ntdll_handle
constexpr ULONG_PTR kZeroBitsBelow2G = 0x7FFFFFFF; // mask form: alloc <2GB
constexpr uint32_t kEchoService = 0xF00D;       // wow64.c WOW64B_SVC_ECHO
constexpr uint32_t kVerdictService = 0xD00E;    // wow64.c WOW64B_SVC_VERDICT
constexpr USHORT kMachineI386 = 0x014C;

// FEX winternl.h WOW64INFO — normally lives after the PEB32; TlsSlots[10]
// points at it. Only CpuFlags (written by BTCpuProcessInit) and the machine
// fields matter for this probe.
struct Wow64Info {
    ULONG NativeSystemPageSize;
    ULONG CpuFlags;
    ULONG Wow64ExecuteFlags;
    ULONG Unknown;
    ULONGLONG SectionHandle;
    ULONGLONG CrossProcessWorkList;
    USHORT NativeMachineType;
    USHORT EmulatedMachineType;
};

PVOID* TlsSlots()
{
    return reinterpret_cast<PVOID*>(reinterpret_cast<char*>(NtCurrentTeb()) + kTebTlsSlotsOffset);
}

// Allocate below 2GB via the ZeroBits mask form — the same call shape
// BTCpuProcessInit uses for the BOP page, so this doubles as the kernel
// readiness check flagged in wow64-plan.md Phase B.
void* AllocBelow2G(SIZE_T Size, ULONG Protect, const char* What)
{
    void* Base = nullptr;
    LONG Status = NtAllocateVirtualMemory(GetCurrentProcess(), &Base, kZeroBitsBelow2G, &Size,
                                          MEM_RESERVE | MEM_COMMIT, Protect);
    if (Status < 0) {
        PROBE("%s: ZeroBits-mask alloc FAILED status=%08lx — kernel mask semantics missing?", What, Status);
        return nullptr;
    }
    if (reinterpret_cast<uintptr_t>(Base) >= 0x80000000u) {
        PROBE("%s: ZeroBits-mask alloc landed HIGH at %p — kernel ignored the mask", What, Base);
        return nullptr;
    }
    PROBE("%s: %zu bytes at %p (below 2GB, ZeroBits mask honored)", What, Size, Base);
    return Base;
}

// Map the staged 32-bit ntdll. SEC_IMAGE is what the real loader will need
// (Phase C); report its verdict either way and fall back to a plain data
// mapping — wow64fex only parses PE headers off this base at Phase-B depth.
void* MapNtdll32(const wchar_t* Path)
{
    HANDLE File = CreateFileW(Path, GENERIC_READ | GENERIC_EXECUTE, FILE_SHARE_READ, nullptr,
                              OPEN_EXISTING, 0, nullptr);
    if (File == INVALID_HANDLE_VALUE) {
        PROBE("i386 ntdll: CreateFileW(%S) FAILED err=%lu — not staged?", Path, GetLastError());
        return nullptr;
    }

    void* Base = nullptr;
    HANDLE Section = nullptr;
    LONG Status = NtCreateSection(&Section, SECTION_MAP_READ | SECTION_MAP_EXECUTE | SECTION_QUERY,
                                  nullptr, nullptr, PAGE_EXECUTE, SEC_IMAGE, File);
    if (Status >= 0) {
        SIZE_T ViewSize = 0;
        Status = NtMapViewOfSection(Section, GetCurrentProcess(), &Base, 0, 0, nullptr, &ViewSize,
                                    1 /* ViewShare */, 0, PAGE_EXECUTE);
        if (Status >= 0) {
            PROBE("i386 ntdll: SEC_IMAGE mapped at %p size=%zx (kernel accepts i386 images — Phase C ready)",
                  Base, ViewSize);
            CloseHandle(File);
            return Base;
        }
        PROBE("i386 ntdll: SEC_IMAGE map FAILED status=%08lx", Status);
        CloseHandle(Section);
    } else {
        PROBE("i386 ntdll: NtCreateSection(SEC_IMAGE) FAILED status=%08lx (Phase C kernel work owed)", Status);
    }

    // Data-mapping fallback: headers parse at base; good enough for the
    // Phase-B HandleImageMap bookkeeping.
    HANDLE Mapping = CreateFileMappingW(File, nullptr, PAGE_READONLY, 0, 0, nullptr);
    CloseHandle(File);
    if (!Mapping) {
        PROBE("i386 ntdll: data-mapping fallback FAILED err=%lu", GetLastError());
        return nullptr;
    }
    Base = MapViewOfFile(Mapping, FILE_MAP_READ, 0, 0, 0);
    CloseHandle(Mapping);
    PROBE("i386 ntdll: data-mapped (fallback) at %p", Base);
    return Base;
}

// The guest program, hand-assembled 32-bit x86. Stack effect mirrors a real
// 32-bit ntdll syscall stub: caller pushes args then calls a stub whose own
// `call edx` return slot is what HandleSyscallImpl pops; args are read from
// guest ESP+8. ECHO's result comes back in EAX; the guest hands it to
// VERDICT, which terminates the process from the host side.
size_t EmitGuestBlob(uint8_t* Code, uint32_t BopAddress)
{
    uint8_t* P = Code;
    auto Emit8 = [&](uint8_t V) { *P++ = V; };
    auto Emit32 = [&](uint32_t V) { memcpy(P, &V, 4); P += 4; };

    // Stub lives after the main flow; two call sites need its offset — emit
    // main flow with rel32 placeholders, patch after.
    uint8_t* CallSite1;
    uint8_t* CallSite2;

    Emit8(0x68); Emit32(0x33333333);        // push 0x33333333   (arg3)
    Emit8(0x68); Emit32(0x22222222);        // push 0x22222222   (arg2)
    Emit8(0x68); Emit32(0x11111111);        // push 0x11111111   (arg1)
    Emit8(0xB8); Emit32(kEchoService);      // mov eax, ECHO
    Emit8(0xE8); CallSite1 = P; Emit32(0);  // call stub
    Emit8(0x83); Emit8(0xC4); Emit8(0x0C);  // add esp, 12       (drop args)
    Emit8(0x50);                            // push eax          (echo result -> arg1)
    Emit8(0xB8); Emit32(kVerdictService);   // mov eax, VERDICT
    Emit8(0xE8); CallSite2 = P; Emit32(0);  // call stub (never returns)
    Emit8(0xF4);                            // hlt (unreachable)

    uint8_t* Stub = P;
    Emit8(0xBA); Emit32(BopAddress);        // mov edx, BOP
    Emit8(0xFF); Emit8(0xD2);               // call edx
    Emit8(0xC3);                            // ret

    const auto Patch = [&](uint8_t* Site) {
        uint32_t Rel = static_cast<uint32_t>(Stub - (Site + 4));
        memcpy(Site, &Rel, 4);
    };
    Patch(CallSite1);
    Patch(CallSite2);
    return static_cast<size_t>(P - Code);
}

} // namespace

int main()
{
    PROBE("wow64probe start — driving libwow64fex.dll BTCpu contract");

    // -- Kernel readiness: the ZeroBits mask form BTCpuProcessInit depends on.
    uint8_t* GuestCode = static_cast<uint8_t*>(AllocBelow2G(0x1000, PAGE_EXECUTE_READWRITE, "guest code"));
    uint8_t* GuestStack = static_cast<uint8_t*>(AllocBelow2G(0x10000, PAGE_READWRITE, "guest stack"));
    if (!GuestCode || !GuestStack) {
        PROBE("FAIL: guest memory unavailable");
        return 0xBAD;
    }

    // -- 32-bit ntdll for LdrSystemDllInitBlock.ntdll_handle.
    wchar_t NtdllPath[MAX_PATH];
    GetWindowsDirectoryW(NtdllPath, MAX_PATH - 32);
    wcscat(NtdllPath, L"\\syswow64\\ntdll.dll");
    void* Ntdll32 = MapNtdll32(NtdllPath);
    if (!Ntdll32) {
        PROBE("FAIL: no i386 ntdll mapping");
        return 0xBAD;
    }

    auto* InitBlock = reinterpret_cast<uint8_t*>(
        GetProcAddress(GetModuleHandleW(L"ntdll.dll"), "LdrSystemDllInitBlock"));
    if (!InitBlock) {
        PROBE("FAIL: LdrSystemDllInitBlock not exported from ntdll");
        return 0xBAD;
    }
    *reinterpret_cast<uint64_t*>(InitBlock + kInitBlockNtdllHandleOff) = reinterpret_cast<uint64_t>(Ntdll32);
    PROBE("LdrSystemDllInitBlock at %p, ntdll_handle=%p", InitBlock, Ntdll32);

    // -- WOW64INFO the CPU layer reads/writes through TlsSlots[10].
    static Wow64Info Info = {};
    Info.NativeSystemPageSize = 0x1000;
    Info.NativeMachineType = 0xAA64;  // IMAGE_FILE_MACHINE_ARM64
    Info.EmulatedMachineType = kMachineI386;
    TlsSlots()[kTlsSlotWow64Info] = &Info;

    // Logging::Init falls back to fopen("%LOCALAPPDATA%\\fex-<pid>.log") and
    // formats the getenv result unchecked — NULL in a winlogon-spawned
    // process. Give it a real value (the dll's mini-CRT snapshots the PEB env
    // during BTCpuProcessInit, so this is visible to it).
    if (GetEnvironmentVariableA("LOCALAPPDATA", nullptr, 0) == 0) {
        SetEnvironmentVariableA("LOCALAPPDATA", "C:\\ReactOS\\Temp");
        PROBE("env: LOCALAPPDATA defaulted (Logging::Init formats it unchecked)");
    }

    // -- Load the CPU layer and initialize the process.
    HMODULE Fex = LoadLibraryW(L"libwow64fex.dll");
    if (!Fex) {
        PROBE("FAIL: LoadLibraryW(libwow64fex.dll) err=%lu", GetLastError());
        return 0xBAD;
    }
    auto ProcessInit = reinterpret_cast<void (*)()>(GetProcAddress(Fex, "BTCpuProcessInit"));
    auto ThreadInit = reinterpret_cast<void (*)()>(GetProcAddress(Fex, "BTCpuThreadInit"));
    auto GetBopCode = reinterpret_cast<void* (*)()>(GetProcAddress(Fex, "BTCpuGetBopCode"));
    auto Simulate = reinterpret_cast<void (*)()>(GetProcAddress(Fex, "BTCpuSimulate"));
    g_ResetToConsistentState = reinterpret_cast<NTSTATUS (*)(EXCEPTION_POINTERS*)>(
        GetProcAddress(Fex, "BTCpuResetToConsistentState"));
    if (!ProcessInit || !ThreadInit || !GetBopCode || !Simulate || !g_ResetToConsistentState) {
        PROBE("FAIL: BTCpu* exports missing (%p %p %p %p %p)", ProcessInit, ThreadInit, GetBopCode,
              Simulate, g_ResetToConsistentState);
        return 0xBAD;
    }

    g_FexBase = reinterpret_cast<uintptr_t>(Fex);
    auto* Nt = reinterpret_cast<const IMAGE_NT_HEADERS*>(
        reinterpret_cast<const char*>(Fex) + reinterpret_cast<const IMAGE_DOS_HEADER*>(Fex)->e_lfanew);
    g_FexSize = Nt->OptionalHeader.SizeOfImage;
    AddVectoredExceptionHandler(1, ProbeVeh);
    // Logging::Init formats getenv("LOCALAPPDATA") unchecked when
    // __wine_dbg_output is absent — record what the dll will see.
    char EnvBuf[MAX_PATH] = "(unset)";
    GetEnvironmentVariableA("LOCALAPPDATA", EnvBuf, sizeof(EnvBuf));
    PROBE("libwow64fex.dll base=%p size=%zx LOCALAPPDATA=%s; calling BTCpuProcessInit...",
          Fex, g_FexSize, EnvBuf);
    ProcessInit();
    void* Bop = GetBopCode();
    PROBE("BTCpuProcessInit OK — BOP page at %p (CpuFlags now %lx)", Bop, Info.CpuFlags);
    if (reinterpret_cast<uintptr_t>(Bop) >= 0x80000000u) {
        PROBE("FAIL: BOP page above 2GB — guest can't reach it");
        return 0xBAD;
    }

    // -- Guest program + CPU area, then hand the thread to the emulator.
    size_t BlobLen = EmitGuestBlob(GuestCode, static_cast<uint32_t>(reinterpret_cast<uintptr_t>(Bop)));
    PROBE("guest blob: %zu bytes at %p, stack top %p", BlobLen, GuestCode, GuestStack + 0x10000 - 64);

    // WOW64_CPURESERVED{Flags,Machine} + 16-aligned WOW64_CONTEXT — layout our
    // ntdll RtlWow64GetCurrentCpuArea (wow64shim.rs) resolves.
    alignas(16) static uint8_t CpuArea[16 + sizeof(WOW64_CONTEXT)] = {};
    *reinterpret_cast<USHORT*>(CpuArea + 2) = kMachineI386; // WOW64_CPURESERVED.Machine
    auto* Ctx = reinterpret_cast<WOW64_CONTEXT*>(CpuArea + 16);
    Ctx->ContextFlags = WOW64_CONTEXT_FULL | WOW64_CONTEXT_EXTENDED_REGISTERS;
    Ctx->Eip = static_cast<DWORD>(reinterpret_cast<uintptr_t>(GuestCode));
    Ctx->Esp = static_cast<DWORD>(reinterpret_cast<uintptr_t>(GuestStack)) + 0x10000 - 64;
    Ctx->EFlags = 0x202;
    Ctx->SegCs = 0x23; // index 4 == wow64fex DefaultWow64CS
    Ctx->SegSs = 0x2B;
    Ctx->SegDs = 0x2B;
    Ctx->SegEs = 0x2B;
    Ctx->SegFs = 0x53;
    Ctx->SegGs = 0x2B;
    auto* XSave = reinterpret_cast<uint8_t*>(Ctx->ExtendedRegisters);
    *reinterpret_cast<uint16_t*>(XSave + 0x00) = 0x027F;     // FCW
    *reinterpret_cast<uint32_t*>(XSave + 0x18) = 0x00001F80; // MXCSR
    TlsSlots()[kTlsSlotCpuReserved] = CpuArea;

    PROBE("calling BTCpuThreadInit...");
    ThreadInit();

    PROBE("entering BTCpuSimulate (eip=%08lx) — PASS arrives via wow64.dll VERDICT service", Ctx->Eip);
    Simulate();

    PROBE("FAIL: BTCpuSimulate returned");
    return 0xBAD;
}
