// SPDX-License-Identifier: MIT
//
// FEX rung-2 "hello compute" harness — vibeos FEX bring-up ladder
// (docs/engineering/fex-bringup-plan.md). A native arm64 Windows PE that drives
// the FEXCore Context API directly to JIT and execute a *static x86-64 blob* in
// process, with NO syscalls and NO WoW64 layer, then prints the computed guest
// result to the vibeos serial console via ntdll DbgPrint.
//
// This is the first proof that a freestanding FEXCore PE runs against *our*
// ntdll and that the JIT compiles + runs guest code on-box. It is deliberately
// modelled on FEX's own FEXOfflineCompiler (context/thread/GDT setup) and the
// WOW64 Module (guest-state readback), so the API usage matches upstream.
//
// The guest blob (64-bit) — rung 2 (compute) + rung 3 (fault round-trip):
//     48 c7 c0 2a 00 00 00   mov rax, 0x2A     ; 42
//     48 83 c0 07            add rax, 7        ; -> 49 (0x31)
//     48 89 04 25 00 00 00 00 mov [0], rax     ; rung 3: deliberate AV (write null)
//     f4                     hlt               ; never reached (VEH redirects)
//     48 c7 c0 99 00 00 00   mov rax, 0x99     ; +0x14: recovery entry
//     f4                     hlt               ; EnableExitOnHLT -> ExecuteThread returns
// The VEH catches the host-side AV inside JIT code, reconstructs the guest
// state (RIP must read back as blob+0xB with RAX already 0x31 — proving exact
// fault-point recovery), redirects guest RIP to the recovery entry, and resumes
// via the dispatcher's FillSRA entry. Success = guest RAX reads back as 0x99.

#include <FEXCore/Config/Config.h>
#include <FEXCore/Core/Context.h>
#include <FEXCore/Core/CoreState.h>
#include <FEXCore/Core/HostFeatures.h>
#include <FEXCore/Core/X86Enums.h>
#include <FEXCore/Debug/InternalThreadState.h>

#include <DummyHandlers.h>
#include <Common/HostFeatures.h>
#include <Common/CPUFeatures.h>
#include <FEXCore/Utils/AllocatorHooks.h>
#include <FEXCore/Utils/TypeDefines.h>

#include <optional>

#include <cstdint>
#include <cstring>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winternl.h> // NTSTATUS WINAPIV DbgPrint(LPCSTR, ...) — vibeos routes to PL011 serial
#include <Common/OvercommitTracker.h> // after the Windows headers it leans on

namespace {
// Global-ctor tripwire: runs during CRT init, BEFORE main. If [FEX-HELLO-CTOR]
// prints but [FEX-HELLO] (main) does not, the wedge is in FEX's static init
// (global ctors) between here and main. If NEITHER prints, the wedge is earlier
// still — a dependency DllMain, or CRT __scrt bringup before global ctors.
struct StartupTripwire {
  StartupTripwire() { DbgPrint("[FEX-HELLO-CTOR] a global ctor ran (pre-main)\n"); }
} g_StartupTripwire;

constexpr uint64_t GuestExpected = 0x5CA11ED; // written into guest RAX by the host syscall handler
constexpr uint64_t GuestFaultPointRAX = 0x31; // 42 + 7, live in RAX at the AV
constexpr uint64_t GuestFaultOffset = 0xB;    // blob offset of the faulting mov
constexpr uint64_t GuestRecoveryOffset = 0x14; // blob offset of the recovery entry
constexpr uint64_t GuestSyscallNum = 0x99;    // guest RAX at the syscall
constexpr uint64_t GuestSyscallArg = 0xC0FFEE; // guest RDI at the syscall

const uint8_t GuestBlob[] = {
  0x48, 0xC7, 0xC0, 0x2A, 0x00, 0x00, 0x00,             // +0x00: mov rax, 0x2A
  0x48, 0x83, 0xC0, 0x07,                               // +0x07: add rax, 7
  0x48, 0x89, 0x04, 0x25, 0x00, 0x00, 0x00, 0x00,       // +0x0B: mov [0], rax — AV (rung 3)
  0xF4,                                                 // +0x13: hlt (skipped)
  0x48, 0xC7, 0xC0, 0x99, 0x00, 0x00, 0x00,             // +0x14: mov rax, 0x99 (recovery)
  0x48, 0xC7, 0xC7, 0xEE, 0xFF, 0xC0, 0x00,             // +0x1B: mov rdi, 0xC0FFEE
  0x0F, 0x05,                                           // +0x22: syscall (rung 4)
  0xF4,                                                 // +0x24: hlt
};
constexpr uint64_t GuestSyscallOffset = 0x22;
constexpr size_t GuestSyscallInstSize = 2;

// Shared with the VEH (single-threaded harness; written before ExecuteThread).
FEXCore::Context::Context* g_CTX;
FEXCore::Core::InternalThreadState* g_Thread;
FEXCore::SignalDelegator* g_SignalDelegator;
// Commit on touch, as the real frontends do (WOW64/Module.cpp, wowx64fex): FEX's committed
// allocations and its lookup cache are reserved, and a fault commits the page. The vibeos
// kernel charges commit against a limit with no pagefile behind it, so committing the
// 272 MB lookup cache eagerly (what this harness used to do) is refused at 1 GB.
std::optional<FEX::Windows::OvercommitTracker> g_Overcommit;
uint64_t g_GuestRIPBase;
int g_GuestFaultsHandled;
int g_GuestSyscalls;

// FEXCore's LookupCache allocates its block-lookup tables (~264MB of VA in
// 64-bit mode: VirtualMemSize>>12*8 + 128MB CODE + L1) as MEM_RESERVE-only and
// then calls SyscallHandler->MarkOvercommitRange(), expecting the host layer to
// commit pages on first-touch AV (WOW64/ARM64EC modules install a fault handler
// wired to FEX::Windows::OvercommitTracker). DummyHandlers' MarkOvercommitRange
// is a no-op and no fault handler exists here, so the first FindBlock read into
// the reserved region was a clean C0000005 (the rung-2 data abort: ldr [x22,#8],
// x22 = the reserved cache base). Fix: eagerly MEM_COMMIT the marked range —
// commit is demand-paged charge, not physical backing, and this is byte-for-byte
// what FEX's own Wine path does (OvercommitTracker.h commits the whole region
// PAGE_READWRITE up front). Fault-driven overcommit is rung-3 territory.
class ProbeSyscallHandler final : public FEX::DummyHandlers::DummySyscallHandler {
public:
  ProbeSyscallHandler() {
    // Rung 4: OS_GENERIC is the wow64-style syscall ABI — the JIT spills ALL
    // guest registers to the CpuStateFrame before HandleSyscall and refills
    // after, so the handler reads/writes guest state directly and the C return
    // value is ignored (NORETURNEDRESULT). Same contract as WowSyscallHandler.
    OSABI = FEXCore::HLE::SyscallOSABI::OS_GENERIC;
  }

  // Rung 4: the guest executes `syscall` with a "number" in RAX and an arg in
  // RDI. State.rip holds the ADDRESS OF the syscall instruction (the JIT
  // reloads rip after this returns), so the handler must advance it past the
  // instruction itself — exactly like Wow64SystemServiceEx's dispatcher sets a
  // return RIP. Result goes into guest RAX by hand (OS_GENERIC ignores the
  // return value).
  uint64_t HandleSyscall(FEXCore::Core::CpuStateFrame* Frame, FEXCore::HLE::SyscallArguments* Args) override {
    auto& State = Frame->State;
    DbgPrint("[FEX-SYSCALL] guest syscall: num=0x%llx arg=0x%llx rip=%p\n",
             (unsigned long long)State.gregs[FEXCore::X86State::REG_RAX],
             (unsigned long long)State.gregs[FEXCore::X86State::REG_RDI], (void*)State.rip);
    g_GuestSyscalls++;
    State.gregs[FEXCore::X86State::REG_RAX] = GuestExpected;
    State.rip += GuestSyscallInstSize;
    return 0;
  }

  void MarkOvercommitRange(uint64_t Start, uint64_t Length) override {
    g_Overcommit->MarkRange(Start, Length);
    DbgPrint("[FEX-HELLO] overcommit range %p +0x%llx -> committed on touch\n", (void*)Start, (unsigned long long)Length);
  }
  void UnmarkOvercommitRange(uint64_t Start, uint64_t Length) override {
    g_Overcommit->UnmarkRange(Start, Length);
  }
};

// First-chance VEH crash dumper. The rung-2 faults land inside FEXCore's
// runtime-emitted dispatcher/JIT code (no symbols possible), so on AV dump the
// faulting instruction words + full register file to serial, then continue the
// search so the normal kill path still runs. Kernel user-exception dispatch is
// proven working on vibeos arm64 ([UEXC-DISPATCH] reaches ntdll).
LONG WINAPI ProbeVeh(EXCEPTION_POINTERS* Info) {
  const auto* Rec = Info->ExceptionRecord;
  auto* Ctx = Info->ContextRecord;

  // A first touch of reserved FEX memory: commit the page and retry.
  if (Rec->ExceptionCode == EXCEPTION_ACCESS_VIOLATION && Rec->NumberParameters > 1 && g_Overcommit &&
      g_Overcommit->HandleAccessViolation(Rec->ExceptionInformation[1])) {
    return EXCEPTION_CONTINUE_EXECUTION;
  }

  // Rung 3: a guest-code AV surfaces as a host AV with PC inside FEX's JIT code
  // buffer. Reconstruct the guest state exactly as wow64fex's
  // BTCpuResetToConsistentState does (WOW64/Module.cpp ReconstructThreadState),
  // redirect the guest to the recovery routine, and resume at the dispatcher's
  // FillSRA entry, which reloads SRA + RIP from the thread state.
  if (Rec->ExceptionCode == EXCEPTION_ACCESS_VIOLATION && g_CTX && g_Thread &&
      g_CTX->IsAddressInCodeBuffer(g_Thread, Ctx->Pc)) {
    const auto& Config = g_SignalDelegator->GetConfig();
    auto& State = g_Thread->CurrentFrame->State;

    State.rip = g_CTX->RestoreRIPFromHostPC(g_Thread, Ctx->Pc);
    for (size_t i = 0; i < Config.SRAGPRCount; i++) {
      State.gregs[i] = Ctx->X[Config.SRAGPRMapping[i]];
    }
    for (size_t i = 0; i < Config.SRAFPRCount; i++) {
      memcpy(State.xmm.sse.data[i], &Ctx->V[Config.SRAFPRMapping[i]], 16);
    }
    const uint32_t EFlags = g_CTX->ReconstructCompactedEFLAGS(g_Thread, true, Ctx->X, Ctx->Cpsr);
    g_CTX->SetFlagsFromCompactedEFLAGS(g_Thread, EFlags);

    const uint64_t FaultAddr = Rec->NumberParameters > 1 ? Rec->ExceptionInformation[1] : ~0ULL;
    DbgPrint("[FEX-FAULT] guest AV: RIP=%p (expect %p) RAX=0x%llx (expect 0x%llx) fault-addr=%p\n", (void*)State.rip,
             (void*)(g_GuestRIPBase + GuestFaultOffset), (unsigned long long)State.gregs[FEXCore::X86State::REG_RAX],
             (unsigned long long)GuestFaultPointRAX, (void*)FaultAddr);

    g_GuestFaultsHandled++;
    State.rip = g_GuestRIPBase + GuestRecoveryOffset;
    Ctx->Pc = Config.AbsoluteLoopTopAddressFillSRA;
    Ctx->X1 = 0; // ENTRY_FILL_SRA_SINGLE_INST_REG (TMP2=x1): zero = normal block execution
    return EXCEPTION_CONTINUE_EXECUTION;
  }
  DbgPrint("[FEX-VEH] code=%08lx addr=%p info0=%p info1=%p\n", Rec->ExceptionCode, Rec->ExceptionAddress,
           (void*)(Rec->NumberParameters > 0 ? Rec->ExceptionInformation[0] : ~0ULL),
           (void*)(Rec->NumberParameters > 1 ? Rec->ExceptionInformation[1] : ~0ULL));
  DbgPrint("[FEX-VEH] PC=%p SP=%p LR=%p FP=%p\n", (void*)Ctx->Pc, (void*)Ctx->Sp, (void*)Ctx->Lr, (void*)Ctx->Fp);
  for (int i = 0; i < 29; i += 4) {
    DbgPrint("[FEX-VEH] x%-2d=%016llx x%-2d=%016llx x%-2d=%016llx x%-2d=%016llx\n", i, (unsigned long long)Ctx->X[i], i + 1,
             (unsigned long long)(i + 1 < 29 ? Ctx->X[i + 1] : 0), i + 2, (unsigned long long)(i + 2 < 29 ? Ctx->X[i + 2] : 0), i + 3,
             (unsigned long long)(i + 3 < 29 ? Ctx->X[i + 3] : 0));
  }
  // 8 instruction words before + 8 after the PC (the PC page is executing, so
  // reading around it is safe modulo page edges — clamp to the PC's page).
  const uint64_t Page = Ctx->Pc & ~0xFFFULL;
  const uint64_t Lo = Ctx->Pc >= Page + 32 ? Ctx->Pc - 32 : Page;
  const auto* Insn = reinterpret_cast<const uint32_t*>(Lo);
  for (int i = 0; i < 16; i += 4) {
    DbgPrint("[FEX-VEH] %p: %08lx %08lx %08lx %08lx\n", (const void*)&Insn[i], Insn[i], Insn[i + 1], Insn[i + 2], Insn[i + 3]);
  }
  return EXCEPTION_CONTINUE_SEARCH;
}

// 64-bit long-mode code segment (GDT index 6<<3 == DEFAULT_USER_CS on x86-64).
constexpr size_t DefaultCS = FEXCore::Core::CPUState::DEFAULT_USER_CS;
FEXCore::Core::CPUState::gdt_segment gdt[32] {};
} // namespace

extern "C" int main(int /*argc*/, char** /*argv*/) {
  DbgPrint("[FEX-HELLO] rung-2 hello-compute harness start\n");

  // EL0 cache-maintenance tripwire (task #46): production Windows-on-ARM64
  // exposes `mrs CTR_EL0`, `dc cvau` and `dc zva` to user mode (SCTLR_EL1
  // UCI/UCT/DZE). vibeos inherited the reset-clear bits from firmware, so
  // these trapped as EC=0x18 -> STATUS_ILLEGAL_INSTRUCTION and killed
  // whichever FEX guest first took the rare inline-SMC-flush path (the
  // 2-in-13 startup-death flake). Executing all three here, every boot,
  // makes any regression deterministic: fexprobe dies right here with a
  // 0xC000001D exit instead of a random guest dying once a fortnight.
  {
    alignas(2048) static unsigned char ZvaBuf[2048]; // >= max DC ZVA block
    uint64_t Ctr;
    __asm__ __volatile__("mrs %0, ctr_el0" : "=r"(Ctr));
    __asm__ __volatile__("dc cvau, %0" ::"r"(&ZvaBuf[0]) : "memory");
    __asm__ __volatile__("dc zva, %0" ::"r"(&ZvaBuf[0]) : "memory");
    DbgPrint("[FEX-HELLO] EL0 cache ops OK (dc cvau/dc zva/CTR_EL0=%llx)\n",
             (unsigned long long)Ctr);
  }

  g_Overcommit.emplace(false);
  FEXCore::Allocator::LazyCommitTrack = [](void* Base, size_t Size, bool Mark) {
    if (Mark) {
      g_Overcommit->MarkRange(reinterpret_cast<uint64_t>(Base), Size);
    } else {
      g_Overcommit->UnmarkRange(reinterpret_cast<uint64_t>(Base), Size);
    }
  };
  AddVectoredExceptionHandler(1, ProbeVeh);

  // 64-bit guest.
  FEXCore::Config::Initialize();
  FEXCore::Config::Set(FEXCore::Config::CONFIG_IS64BIT_MODE, "1");

  const auto NtDll = GetModuleHandleA("ntdll.dll");
  const bool IsWine = NtDll && !!GetProcAddress(NtDll, "wine_get_version");
  auto HostFeatures = FEX::Windows::CPUFeatures::FetchHostFeatures(IsWine, FEXCore::HostFeatures::HostTypeEnum::Arm64ec);

  auto CTX = FEXCore::Context::Context::CreateNewContext(HostFeatures);

  auto SignalDelegation = FEX::DummyHandlers::CreateSignalDelegator();
  auto SyscallHandler = fextl::make_unique<ProbeSyscallHandler>();
  CTX->SetSignalDelegator(SignalDelegation.get());
  CTX->SetSyscallHandler(SyscallHandler.get());

  // Exit ExecuteThread when the guest hits HLT. MUST precede InitCore(): the
  // dispatcher is emitted inside InitCore and reads ExitOnHLTEnabled() at
  // emission time — enabling it after bakes the disabled path, where HLT is
  // lowered to a DELIBERATE null-load segfault ("Force a SIGSEGV by loading
  // zero", Dispatcher.cpp GuestSignal_SIGSEGV) for a signal delegator we don't
  // have. That was the rung-2 "dispatcher data abort": mov w1,#0; ldr x1,[x1]
  // with guest RAX already = 0x31 in x4.
  CTX->EnableExitOnHLT();

  if (!CTX->InitCore()) {
    DbgPrint("[FEX-HELLO] FAIL: InitCore returned false\n");
    return 1;
  }

  // Map an RWX buffer for the guest code (guest VA == host VA in-process).
  void* CodeMem = VirtualAlloc(nullptr, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
  void* StackMem = VirtualAlloc(nullptr, 0x10000, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
  if (!CodeMem || !StackMem) {
    DbgPrint("[FEX-HELLO] FAIL: VirtualAlloc code=%p stack=%p\n", CodeMem, StackMem);
    return 1;
  }
  memcpy(CodeMem, GuestBlob, sizeof(GuestBlob));

  const uint64_t GuestRIP = reinterpret_cast<uint64_t>(CodeMem);
  const uint64_t GuestRSP = reinterpret_cast<uint64_t>(StackMem) + 0x10000 - 0x100;

  auto Thread = CTX->CreateThread(GuestRIP, GuestRSP);

  g_CTX = CTX.get();
  g_Thread = Thread;
  g_SignalDelegator = SignalDelegation.get();
  g_GuestRIPBase = GuestRIP;

  // 64-bit long-mode GDT/segment setup (mirrors FEXOfflineCompiler).
  auto Frame = Thread->CurrentFrame;
  Frame->State.segment_arrays[FEXCore::Core::CPUState::SEGMENT_ARRAY_INDEX_GDT] = &gdt[0];
  Frame->State.segment_arrays[FEXCore::Core::CPUState::SEGMENT_ARRAY_INDEX_LDT] = &gdt[0];
  Frame->State.cs_idx = DefaultCS << 3;
  auto GDT = FEXCore::Core::CPUState::GetSegmentFromIndex(Frame->State, Frame->State.cs_idx);
  FEXCore::Core::CPUState::SetGDTBase(GDT, 0);
  FEXCore::Core::CPUState::SetGDTLimit(GDT, 0xFFFFFU);
  Frame->State.cs_cached = FEXCore::Core::CPUState::CalculateGDTBase(*GDT);
  GDT->L = 1; // Long mode = 64-bit
  GDT->D = 0;

  Frame->State.rip = GuestRIP;
  Frame->State.gregs[FEXCore::X86State::REG_RSP] = GuestRSP;

  DbgPrint("[FEX-HELLO] executing guest blob @ %p (rsp=%p)\n", (void*)GuestRIP, (void*)GuestRSP);
  CTX->ExecuteThread(Thread);

  const uint64_t GuestRAX = Frame->State.gregs[FEXCore::X86State::REG_RAX];
  const bool Pass = (GuestRAX == GuestExpected) && (g_GuestFaultsHandled == 1) && (g_GuestSyscalls == 1);
  DbgPrint("[FEX-HELLO] guest RAX = 0x%llx (expected 0x%llx), faults round-tripped = %d/1, syscalls thunked = %d/1 (%s)\n",
           (unsigned long long)GuestRAX, (unsigned long long)GuestExpected, g_GuestFaultsHandled, g_GuestSyscalls, Pass ? "PASS" : "FAIL");

  CTX->DestroyThread(Thread);

  // WOW64 Phase A load-test (docs/engineering/wow64-plan.md): with the ntdll
  // surface (synch family + RtlWow64* + NtAllocateVirtualMemoryEx/NtOpenKeyEx/
  // LdrGetDllFullName/RtlQPC) and the wow64.dll skeleton in place, the REAL
  // FEX CPU layer must fully resolve its imports. LoadLibrary runs its CRT
  // DllMain but NOT BTCpuProcessInit (called by wow64.dll in Phase B), so a
  // successful load + BTCpuGetBopCode lookup is exactly the Phase A milestone.
  HMODULE Wow64Fex = LoadLibraryW(L"libwow64fex.dll");
  if (Wow64Fex) {
    void* BopProc = (void*)GetProcAddress(Wow64Fex, "BTCpuGetBopCode");
    DbgPrint("[FEX-WOW64A] libwow64fex.dll loaded @ %p, BTCpuGetBopCode=%p (%s)\n", (void*)Wow64Fex, BopProc,
             BopProc ? "PASS" : "FAIL");
  } else {
    DbgPrint("[FEX-WOW64A] libwow64fex.dll load FAILED (gle=%lu)\n", GetLastError());
  }

  DbgPrint("[FEX-HELLO] done\n");
  return Pass ? 0 : 1;
}
