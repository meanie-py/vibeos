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

#include <cstdint>
#include <cstring>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winternl.h> // NTSTATUS WINAPIV DbgPrint(LPCSTR, ...) — vibeos routes to PL011 serial

namespace {
// Global-ctor tripwire: runs during CRT init, BEFORE main. If [FEX-HELLO-CTOR]
// prints but [FEX-HELLO] (main) does not, the wedge is in FEX's static init
// (global ctors) between here and main. If NEITHER prints, the wedge is earlier
// still — a dependency DllMain, or CRT __scrt bringup before global ctors.
struct StartupTripwire {
  StartupTripwire() { DbgPrint("[FEX-HELLO-CTOR] a global ctor ran (pre-main)\n"); }
} g_StartupTripwire;

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
  void MarkOvercommitRange(uint64_t Start, uint64_t Length) override {
    void* Ret = VirtualAlloc(reinterpret_cast<void*>(Start), Length, MEM_COMMIT, PAGE_READWRITE);
    DbgPrint("[FEX-HELLO] overcommit range %p +0x%llx -> commit %s\n", (void*)Start, (unsigned long long)Length, Ret ? "OK" : "FAILED");
  }
  void UnmarkOvercommitRange(uint64_t Start, uint64_t Length) override {}
};

constexpr uint64_t GuestExpected = 0x99;      // recovery routine's RAX
constexpr uint64_t GuestFaultPointRAX = 0x31; // 42 + 7, live in RAX at the AV
constexpr uint64_t GuestFaultOffset = 0xB;    // blob offset of the faulting mov
constexpr uint64_t GuestRecoveryOffset = 0x14; // blob offset of the recovery entry

const uint8_t GuestBlob[] = {
  0x48, 0xC7, 0xC0, 0x2A, 0x00, 0x00, 0x00,             // +0x00: mov rax, 0x2A
  0x48, 0x83, 0xC0, 0x07,                               // +0x07: add rax, 7
  0x48, 0x89, 0x04, 0x25, 0x00, 0x00, 0x00, 0x00,       // +0x0B: mov [0], rax — AV
  0xF4,                                                 // +0x13: hlt (skipped)
  0x48, 0xC7, 0xC0, 0x99, 0x00, 0x00, 0x00,             // +0x14: mov rax, 0x99
  0xF4,                                                 // +0x1B: hlt
};

// Shared with the VEH (single-threaded harness; written before ExecuteThread).
FEXCore::Context::Context* g_CTX;
FEXCore::Core::InternalThreadState* g_Thread;
FEXCore::SignalDelegator* g_SignalDelegator;
uint64_t g_GuestRIPBase;
int g_GuestFaultsHandled;

// First-chance VEH crash dumper. The rung-2 faults land inside FEXCore's
// runtime-emitted dispatcher/JIT code (no symbols possible), so on AV dump the
// faulting instruction words + full register file to serial, then continue the
// search so the normal kill path still runs. Kernel user-exception dispatch is
// proven working on vibeos arm64 ([UEXC-DISPATCH] reaches ntdll).
LONG WINAPI ProbeVeh(EXCEPTION_POINTERS* Info) {
  const auto* Rec = Info->ExceptionRecord;
  auto* Ctx = Info->ContextRecord;

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
  const bool Pass = (GuestRAX == GuestExpected) && (g_GuestFaultsHandled == 1);
  DbgPrint("[FEX-HELLO] guest RAX = 0x%llx (expected 0x%llx), guest faults round-tripped = %d (expected 1) (%s)\n",
           (unsigned long long)GuestRAX, (unsigned long long)GuestExpected, g_GuestFaultsHandled, Pass ? "PASS" : "FAIL");

  CTX->DestroyThread(Thread);
  DbgPrint("[FEX-HELLO] done\n");
  return Pass ? 0 : 1;
}
