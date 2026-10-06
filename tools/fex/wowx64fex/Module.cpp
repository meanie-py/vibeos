// SPDX-License-Identifier: MIT
/*
 * libwowx64fex.dll — the vibeos BTCpu64 CPU layer: x86-64 guests on arm64,
 * via the vendored FEXCore in 64-bit mode. Fork B of
 * docs/engineering/wow64-x64-guest-design.md; the contract is
 * sdk/include/reactos/wowx64/btcpu64.h and the OS side is wowx64.dll.
 *
 * Built OUT OF TREE from vendor/fex, by linking its static archives (the design
 * doc's sub-option B1): FEX ships no third Windows plugin and vendor/fex stays
 * untouched. The shape is upstream's WOW64/Module.cpp, function for function,
 * with five differences — read those, the rest is deliberately the same:
 *
 *   1. CONFIG_IS64BIT_MODE = 1, and the code segment is long mode (L=1).
 *   2. Guest state crosses the contract as an x86-64 CONTEXT (WOWX64_CONTEXT),
 *      mapped exactly as ARM64EC/Module.cpp's LoadStateFromECContext does.
 *   3. A guest `syscall` is forwarded by register shuffle to the OS side
 *      (WOWX64_HOST_SERVICES::SystemService) — no Wow64SystemServiceEx and no
 *      32->64 thunking; see btcpu64.h for why the numbers already agree.
 *   4. Per-thread state hangs off TEB->WOW32Reserved, not TlsSlots[]: this is
 *      a NATIVE process, so its TLS slots belong to kernel32's TlsAlloc, while
 *      WOW32Reserved has no user in a non-WoW64 process (and the guest, which
 *      shares this TEB, does not touch it either).
 *   5. No wow64.dll: the import table is ntdll alone.
 *
 * GS: the guest shares the native TEB, so guest gs:[0x30] is the same TEB the
 * arm64 code sees and gs:[0x60] the same PEB. That is what makes the amd64
 * system DLLs' TEB/PEB accesses come out right with no translation.
 */

#include <FEXCore/fextl/fmt.h>
#include <FEXCore/Core/X86Enums.h>
#include <FEXCore/Core/SignalDelegator.h>
#include <FEXCore/Core/Context.h>
#include <FEXCore/Core/CoreState.h>
#include <FEXCore/Debug/InternalThreadState.h>
#include <FEXCore/HLE/SyscallHandler.h>
#include <FEXCore/Config/Config.h>
#include <FEXCore/Utils/Allocator.h>
#include <FEXCore/Utils/LogManager.h>
#include <FEXCore/Utils/Threads.h>
#include <FEXCore/Utils/Profiler.h>
#include <FEXCore/Utils/EnumUtils.h>
#include <FEXCore/Utils/ArchHelpers/Arm64.h>
#include <FEXCore/Utils/TypeDefines.h>

#include "Windows/Common/Allocator.h"
#include "Windows/Common/EnvironmentVariablesHandling.h"
#include "Windows/Common/FEXUnixLib.h"
#include "Common/CallRetStack.h"
#include "Common/JITGuardPage.h"
#include "Common/Config.h"
#include "Common/Exception.h"
#include "Common/TSOHandlerConfig.h"
#include "Common/ImageTracker.h"
#include "Common/InvalidationTracker.h"
#include "Common/OvercommitTracker.h"
#include "Common/CPUFeatures.h"
#include "Common/Logging.h"
#include "Common/Module.h"
#include "Common/CRT/CRT.h"
#include "Common/PortabilityInfo.h"
#include "Common/Handle.h"
#include "DummyHandlers.h"

#include "btcpu64.h"

#include <cstdint>
#include <atomic>
#include <mutex>
#include <optional>
#include <unordered_map>
#include <ntstatus.h>
#include <windef.h>
#include <winternl.h>

extern "C" {
NTSTATUS STDMETHODCALLTYPE BTCpu64ProcessInit(const WOWX64_HOST_SERVICES* Host);
NTSTATUS STDMETHODCALLTYPE BTCpu64ThreadInit();
NTSTATUS STDMETHODCALLTYPE BTCpu64GetContext(HANDLE Thread, HANDLE Process, void* Reserved, WOWX64_CONTEXT* Context);
NTSTATUS STDMETHODCALLTYPE BTCpu64SetContext(HANDLE Thread, HANDLE Process, void* Reserved, const WOWX64_CONTEXT* Context);
void STDMETHODCALLTYPE BTCpu64Simulate();
NTSTATUS STDMETHODCALLTYPE BTCpu64ResetToConsistentState(EXCEPTION_POINTERS* Ptrs);
NTSTATUS STDMETHODCALLTYPE BTCpu64NotifyMapViewOfSection(void* Unk1, void* Address, void* Unk2, SIZE_T Size, ULONG AllocType, ULONG Prot);
void STDMETHODCALLTYPE BTCpu64NotifyUnmapViewOfSection(void* Address, BOOL After, ULONG Status);
void STDMETHODCALLTYPE BTCpu64NotifyMemoryAlloc(void* Address, SIZE_T Size, ULONG Type, ULONG Prot, BOOL After, ULONG Status);
void STDMETHODCALLTYPE BTCpu64NotifyMemoryProtect(void* Address, SIZE_T Size, ULONG NewProt, BOOL After, ULONG Status);
void STDMETHODCALLTYPE BTCpu64NotifyMemoryFree(void* Address, SIZE_T Size, ULONG FreeType, BOOL After, ULONG Status);
void STDMETHODCALLTYPE BTCpu64FlushInstructionCache2(const void* Address, SIZE_T Size);
void STDMETHODCALLTYPE BTCpu64NotifyMemoryDirty(void* Address, SIZE_T Size);
void STDMETHODCALLTYPE BTCpu64NotifyReadFile(HANDLE Handle, void* Address, SIZE_T Size, BOOL After, NTSTATUS Status);
}

namespace ControlBits {
// When this is unset, a thread can be safely interrupted and have its context recovered
// IMPORTANT: This can only safely be written by the owning thread
static constexpr uint32_t IN_JIT {1U << 0};

// JIT entry polls this bit until it is unset, at which point CONTROL_IN_JIT will be set
static constexpr uint32_t PAUSED {1U << 1};
}; // namespace ControlBits

// Per-thread CPU-layer state; TEB->WOW32Reserved points here (difference 4).
struct ThreadData {
  CONTEXT* EntryContext {};
  std::atomic<uint32_t> ControlWord {};
  FEXCore::Core::InternalThreadState* ThreadState {};
  // See CachedCallRetSp in WOW64/Module.cpp: user callbacks return via a syscall
  // the JIT cannot see, unbalancing the call-ret stack.
  uint64_t CachedCallRetSp {};
};

struct FrontendThreadData {
  bool InLockedRWXRead {};
};

class X64SyscallHandler;

namespace {
fextl::unique_ptr<FEXCore::Context::Context> CTX;
fextl::unique_ptr<FEX::DummyHandlers::DummySignalDelegator> SignalDelegator;
fextl::unique_ptr<X64SyscallHandler> SyscallHandler;

std::optional<FEX::Windows::InvalidationTracker> InvalidationTracker;
std::optional<FEX::Windows::CPUFeatures> CPUFeatures;
std::optional<FEX::Windows::OvercommitTracker> OvercommitTracker;
std::optional<FEX::Windows::ImageTracker> ImageTracker;
std::optional<FEX::Windows::TSOHandlerConfig> HandlerConfig;

WOWX64_HOST_SERVICES Host {};

std::mutex ThreadCreationMutex;
// Map of TIDs to their FEX thread state, `ThreadCreationMutex` must be locked when accessing
std::unordered_map<DWORD, FEXCore::Core::InternalThreadState*> Threads;

ThreadData* GetThreadData() {
  return static_cast<ThreadData*>(reinterpret_cast<__TEB*>(NtCurrentTeb())->WOW32Reserved);
}

// M1 drives only the calling thread. Anything else must fail loudly, never
// silently act on the wrong thread's state.
bool IsCurrentThread(HANDLE Thread) {
  if (Thread == GetCurrentThread()) {
    return true;
  }
  THREAD_BASIC_INFORMATION Info;
  if (NtQueryInformationThread(Thread, ThreadBasicInformation, &Info, sizeof(Info), nullptr)) {
    return false;
  }
  return reinterpret_cast<uintptr_t>(Info.ClientId.UniqueThread) == GetCurrentThreadId();
}

bool IsDispatcherAddress(uint64_t Address) {
  const auto& Config = SignalDelegator->GetConfig();
  return Address >= Config.DispatcherBegin && Address < Config.DispatcherEnd;
}

bool IsAddressInJit(FEXCore::Core::InternalThreadState* Thread, uint64_t Address) {
  return IsDispatcherAddress(Address) || Thread->CTX->IsAddressInCodeBuffer(Thread, Address);
}

void HandleImageMap(uint64_t Address, bool MainImage = false) {
  fextl::string ModulePath = FEX::Windows::GetSectionFilePath(Address);
  fextl::string ModuleName = fextl::string {FEX::Windows::BaseName(ModulePath)};
  InvalidationTracker->HandleImageMap(ModuleName, Address);
  ImageTracker->HandleImageMap(ModulePath, Address, MainImage);
}

void HandleImageUnmap(uint64_t Address, uint64_t Size) {
  ImageTracker->HandleImageUnmap(Address, Size);
}
} // namespace

namespace Context {
// x86-64 CONTEXT -> FEX state. ARM64EC/Module.cpp's LoadStateFromECContext,
// which is plain register mapping with nothing hybrid-PE about it (difference 2).
void LoadStateFromContext(FEXCore::Core::InternalThreadState* Thread, const WOWX64_CONTEXT& Context) {
  auto& State = Thread->CurrentFrame->State;

  if ((Context.ContextFlags & WOWX64_CONTEXT_INTEGER) == WOWX64_CONTEXT_INTEGER) {
    State.gregs[FEXCore::X86State::REG_RAX] = Context.Rax;
    State.gregs[FEXCore::X86State::REG_RCX] = Context.Rcx;
    State.gregs[FEXCore::X86State::REG_RDX] = Context.Rdx;
    State.gregs[FEXCore::X86State::REG_RBX] = Context.Rbx;
    State.gregs[FEXCore::X86State::REG_RSI] = Context.Rsi;
    State.gregs[FEXCore::X86State::REG_RDI] = Context.Rdi;
    State.gregs[FEXCore::X86State::REG_R8] = Context.R8;
    State.gregs[FEXCore::X86State::REG_R9] = Context.R9;
    State.gregs[FEXCore::X86State::REG_R10] = Context.R10;
    State.gregs[FEXCore::X86State::REG_R11] = Context.R11;
    State.gregs[FEXCore::X86State::REG_R12] = Context.R12;
    State.gregs[FEXCore::X86State::REG_R13] = Context.R13;
    State.gregs[FEXCore::X86State::REG_R14] = Context.R14;
    State.gregs[FEXCore::X86State::REG_R15] = Context.R15;
  }

  if ((Context.ContextFlags & WOWX64_CONTEXT_CONTROL) == WOWX64_CONTEXT_CONTROL) {
    State.rip = Context.Rip;
    State.gregs[FEXCore::X86State::REG_RSP] = Context.Rsp;
    State.gregs[FEXCore::X86State::REG_RBP] = Context.Rbp;
    CTX->SetFlagsFromCompactedEFLAGS(Thread, Context.EFlags);
  }

  if ((Context.ContextFlags & WOWX64_CONTEXT_SEGMENTS) == WOWX64_CONTEXT_SEGMENTS) {
    State.es_idx = Context.SegEs;
    State.cs_idx = Context.SegCs;
    State.ss_idx = Context.SegSs;
    State.ds_idx = Context.SegDs;
    State.fs_idx = Context.SegFs;
    State.gs_idx = Context.SegGs;
  }
  // The bases are not selector-derived in long mode: GS is always this
  // thread's TEB and the rest are flat. Re-assert them on every load so no
  // context, whatever its flags, can leave GS pointing somewhere else.
  State.gs_cached = reinterpret_cast<uint64_t>(NtCurrentTeb());
  State.fs_cached = 0;
  State.es_cached = 0;
  State.cs_cached = 0;
  State.ss_cached = 0;
  State.ds_cached = 0;

  if ((Context.ContextFlags & WOWX64_CONTEXT_FLOATING_POINT) == WOWX64_CONTEXT_FLOATING_POINT) {
    CTX->SetXMMRegistersFromState(Thread, reinterpret_cast<const __uint128_t*>(Context.FltSave.XmmRegisters), nullptr);
    memcpy(State.mm, Context.FltSave.FloatRegisters, sizeof(State.mm));

    State.FCW = Context.FltSave.ControlWord;
    State.flags[FEXCore::X86State::X87FLAG_IE_LOC] = Context.FltSave.StatusWord & 1;
    State.flags[FEXCore::X86State::X87FLAG_C0_LOC] = (Context.FltSave.StatusWord >> 8) & 1;
    State.flags[FEXCore::X86State::X87FLAG_C1_LOC] = (Context.FltSave.StatusWord >> 9) & 1;
    State.flags[FEXCore::X86State::X87FLAG_C2_LOC] = (Context.FltSave.StatusWord >> 10) & 1;
    State.flags[FEXCore::X86State::X87FLAG_C3_LOC] = (Context.FltSave.StatusWord >> 14) & 1;
    State.flags[FEXCore::X86State::X87FLAG_TOP_LOC] = (Context.FltSave.StatusWord >> 11) & 0b111;
    State.AbridgedFTW = Context.FltSave.TagWord;
  }
}

// FEX state -> x86-64 CONTEXT (all of it; the caller's flags are not consulted).
void StoreContextFromState(FEXCore::Core::InternalThreadState* Thread, WOWX64_CONTEXT* Context) {
  auto& State = Thread->CurrentFrame->State;

  Context->ContextFlags = WOWX64_CONTEXT_ALL;
  Context->Rax = State.gregs[FEXCore::X86State::REG_RAX];
  Context->Rcx = State.gregs[FEXCore::X86State::REG_RCX];
  Context->Rdx = State.gregs[FEXCore::X86State::REG_RDX];
  Context->Rbx = State.gregs[FEXCore::X86State::REG_RBX];
  Context->Rsp = State.gregs[FEXCore::X86State::REG_RSP];
  Context->Rbp = State.gregs[FEXCore::X86State::REG_RBP];
  Context->Rsi = State.gregs[FEXCore::X86State::REG_RSI];
  Context->Rdi = State.gregs[FEXCore::X86State::REG_RDI];
  Context->R8 = State.gregs[FEXCore::X86State::REG_R8];
  Context->R9 = State.gregs[FEXCore::X86State::REG_R9];
  Context->R10 = State.gregs[FEXCore::X86State::REG_R10];
  Context->R11 = State.gregs[FEXCore::X86State::REG_R11];
  Context->R12 = State.gregs[FEXCore::X86State::REG_R12];
  Context->R13 = State.gregs[FEXCore::X86State::REG_R13];
  Context->R14 = State.gregs[FEXCore::X86State::REG_R14];
  Context->R15 = State.gregs[FEXCore::X86State::REG_R15];
  Context->Rip = State.rip;
  Context->EFlags = CTX->ReconstructCompactedEFLAGS(Thread, false, nullptr, 0);

  Context->SegEs = State.es_idx;
  Context->SegCs = State.cs_idx;
  Context->SegSs = State.ss_idx;
  Context->SegDs = State.ds_idx;
  Context->SegFs = State.fs_idx;
  Context->SegGs = State.gs_idx;

  CTX->ReconstructXMMRegisters(Thread, reinterpret_cast<__uint128_t*>(Context->FltSave.XmmRegisters), nullptr);
  memcpy(Context->FltSave.FloatRegisters, State.mm, sizeof(State.mm));
  Context->FltSave.ControlWord = State.FCW;
  Context->FltSave.StatusWord =
    (State.flags[FEXCore::X86State::X87FLAG_TOP_LOC] << 11) | (State.flags[FEXCore::X86State::X87FLAG_C0_LOC] << 8) |
    (State.flags[FEXCore::X86State::X87FLAG_C1_LOC] << 9) | (State.flags[FEXCore::X86State::X87FLAG_C2_LOC] << 10) |
    (State.flags[FEXCore::X86State::X87FLAG_C3_LOC] << 14) | State.flags[FEXCore::X86State::X87FLAG_IE_LOC];
  Context->FltSave.TagWord = State.AbridgedFTW;
}

void ReconstructThreadState(FEXCore::Core::InternalThreadState* Thread, CONTEXT* Context) {
  const auto& Config = SignalDelegator->GetConfig();
  auto& State = Thread->CurrentFrame->State;

  State.rip = CTX->RestoreRIPFromHostPC(Thread, Context->Pc);

  // Spill all SRA GPRs
  for (size_t i = 0; i < Config.SRAGPRCount; i++) {
    State.gregs[i] = Context->X[Config.SRAGPRMapping[i]];
  }

  // Spill all SRA FPRs
  for (size_t i = 0; i < Config.SRAFPRCount; i++) {
    memcpy(State.xmm.sse.data[i], &Context->V[Config.SRAFPRMapping[i]], sizeof(__uint128_t));
  }

  // Spill EFlags
  uint32_t EFlags = CTX->ReconstructCompactedEFLAGS(Thread, true, Context->X, Context->Cpsr);
  CTX->SetFlagsFromCompactedEFLAGS(Thread, EFlags);
}

bool HandleUnalignedAccess(FEXCore::Core::InternalThreadState* Thread, CONTEXT* Context) {
  if (!Thread->CTX->IsAddressInCodeBuffer(Thread, Context->Pc)) {
    return false;
  }

  const auto Result =
    FEXCore::ArchHelpers::Arm64::HandleUnalignedAccess(Thread, HandlerConfig->GetUnalignedHandlerType(), Context->Pc, &Context->X0);
  Context->Pc += Result.value_or(0);
  return Result.has_value();
}

void LockJITContext(ThreadData* TD) {
  uint32_t Expected = TD->ControlWord.load(), New;

  // Spin until PAUSED is unset, setting IN_JIT when that occurs
  do {
    Expected = Expected & ~ControlBits::PAUSED;
    New = Expected | ControlBits::IN_JIT;
  } while (!TD->ControlWord.compare_exchange_weak(Expected, New, std::memory_order::relaxed));
  std::atomic_signal_fence(std::memory_order::seq_cst);
}

void UnlockJITContext(ThreadData* TD) {
  std::atomic_signal_fence(std::memory_order::seq_cst);
  TD->ControlWord.fetch_and(~ControlBits::IN_JIT, std::memory_order::relaxed);
}

class ScopedJITContextLock {
  ThreadData* TD;

public:
  explicit ScopedJITContextLock(ThreadData* TD)
    : TD {TD} {
    LockJITContext(TD);
  }

  ~ScopedJITContextLock() {
    UnlockJITContext(TD);
  }
};
} // namespace Context

// Calls a 2-argument function `Func` setting the parent unwind frame information to the given SP and PC
__attribute__((naked)) extern "C" uint64_t WowX64SEHFrameTrampoline2Args(void* Arg0, void* Arg1, void* Func, uint64_t Sp, uint64_t Pc) {
  asm(".seh_proc WowX64SEHFrameTrampoline2Args;"
      "stp x3, x4, [sp, #-0x10]!;"
      ".seh_pushframe;"
      "stp x29, x30, [sp, #-0x10]!;"
      ".seh_save_fplr_x 16;"
      ".seh_endprologue;"
      "blr x2;"
      "ldp x29, x30, [sp], 0x20;"
      "ret;"
      ".seh_endproc;");
}

// Difference 3: a guest `syscall` is a register shuffle, not a thunk.
class X64SyscallHandler : public FEXCore::HLE::SyscallHandler, public FEXCore::Allocator::FEXAllocOperators {
public:
  X64SyscallHandler() {
    // OS_GENERIC: the JIT spills every guest register to the frame before this
    // runs and refills after, so guest state is read and written directly and
    // the return value is ignored.
    OSABI = FEXCore::HLE::SyscallOSABI::OS_GENERIC;
  }

  static uint64_t HandleSyscallImpl(FEXCore::Core::CpuStateFrame* Frame, FEXCore::HLE::SyscallArguments*) {
    auto& State = Frame->State;

    // amd64 ntdll's stubs are `mov r10, rcx; mov eax, N; syscall; ret`: the
    // first four arguments are in r10/rdx/r8/r9, the rest on the stack from
    // rsp+0x28. The stack ones are the OS side's to read -- it knows how many
    // each service takes, and reading a fixed count runs off the top of a
    // guest stack (contract version 2).
    uint64_t Args[WOWX64_REGISTER_ARGS];
    Args[0] = State.gregs[FEXCore::X86State::REG_R10];
    Args[1] = State.gregs[FEXCore::X86State::REG_RDX];
    Args[2] = State.gregs[FEXCore::X86State::REG_R8];
    Args[3] = State.gregs[FEXCore::X86State::REG_R9];
    const uint64_t GuestRsp = State.gregs[FEXCore::X86State::REG_RSP];

    // The JIT parks rip ON the syscall instruction and has already put the
    // return rip in RCX and RFLAGS in R11, as the instruction architecturally
    // does. Remember where we were so a service that installs a NEW context
    // (NtContinue, a callback return) is not then overwritten below.
    const uint64_t SyscallRip = State.rip;
    const uint32_t Service = static_cast<uint32_t>(State.gregs[FEXCore::X86State::REG_RAX]);

    auto* TD = GetThreadData();
    Context::UnlockJITContext(TD);
    const uint64_t Ret = Host.SystemService(Service, Args, GuestRsp);
    Context::LockJITContext(TD);

    if (State.rip == SyscallRip) {
      State.gregs[FEXCore::X86State::REG_RAX] = Ret;
      State.rip = State.gregs[FEXCore::X86State::REG_RCX];
    }

    // NORETURNEDRESULT: ignored, every register is reloaded from the frame.
    return 0;
  }

  uint64_t HandleSyscall(FEXCore::Core::CpuStateFrame* Frame, FEXCore::HLE::SyscallArguments* Args) override {
    auto* TD = GetThreadData();
    // Stash the context pointer on the stack: Simulate can be re-entered from
    // this syscall handler (a kernel callback), which would overwrite it.
    CONTEXT* EntryContext = TD->EntryContext;
    // Run the handler with unwind info naming Simulate as its caller.
    uint64_t Ret = WowX64SEHFrameTrampoline2Args(reinterpret_cast<void*>(Frame), reinterpret_cast<void*>(Args),
                                                 reinterpret_cast<void*>(&HandleSyscallImpl), EntryContext->Sp, EntryContext->Pc);
    TD->EntryContext = EntryContext;
    return Ret;
  }

  std::optional<FEXCore::ExecutableFileSectionInfo> LookupExecutableFileSection(FEXCore::Core::InternalThreadState*, uint64_t Address) override {
    return ImageTracker->LookupExecutableFileSection(Address);
  }

  void MarkGuestExecutableRange(FEXCore::Core::InternalThreadState* Thread, uint64_t Start, uint64_t Length) override {
    InvalidationTracker->ReprotectRWXIntervals(Start, Length);
  }

  void InvalidateGuestCodeRange(FEXCore::Core::InternalThreadState* Thread, uint64_t Start, uint64_t Length) override {
    InvalidationTracker->InvalidateAlignedInterval(Start, Length, false);
  }

  void MarkOvercommitRange(uint64_t Start, uint64_t Length) override {
    OvercommitTracker->MarkRange(Start, Length);
  }

  void UnmarkOvercommitRange(uint64_t Start, uint64_t Length) override {
    OvercommitTracker->UnmarkRange(Start, Length);
  }

  FEXCore::HLE::ExecutableRangeInfo QueryGuestExecutableRange(FEXCore::Core::InternalThreadState* Thread, uint64_t Address) override {
    return InvalidationTracker->QueryExecutableRange(Address);
  }
};

NTSTATUS BTCpu64ProcessInit(const WOWX64_HOST_SERVICES* Services) {
  // Fail loud on a contract mismatch: a short table read past its end would
  // call through garbage on the first guest syscall.
  if (!Services || Services->Size < sizeof(WOWX64_HOST_SERVICES) || Services->Version != WOWX64_CONTRACT_VERSION ||
      !Services->SystemService) {
    return STATUS_REVISION_MISMATCH;
  }
  Host = *Services;

  FEX::Windows::InitCRTProcess();
  const auto ExecutableName = FEX::Windows::BaseName(FEX::Windows::GetExecutableFilePath());
  FEX::Config::LoadConfig(fextl::string {ExecutableName}, _environ, FEX::ReadPortabilityInformation());
  FEXCore::Config::ReloadMetaLayer();
  FEX::Windows::Logging::Init();

  FEXCore::Config::Set(FEXCore::Config::CONFIG_INTERPRETER_INSTALLED, "0");
  FEXCore::Config::Set(FEXCore::Config::CONFIG_IS64BIT_MODE, "1"); // difference 1

  FEXCore::Profiler::Init("", "");

  SignalDelegator = fextl::make_unique<FEX::DummyHandlers::DummySignalDelegator>();
  SyscallHandler = fextl::make_unique<X64SyscallHandler>();
  const auto NtDll = GetModuleHandle("ntdll.dll");
  const bool IsWine = !!GetProcAddress(NtDll, "wine_get_version");
  OvercommitTracker.emplace(IsWine);

  FEX::Windows::Allocator::SetupHooks(NtDll);
  FEX::Windows::UnixLib::Init(NtDll);

  {
    // HostTypeEnum::Wow64 means "a Windows host: `int 0x2e` is a syscall".
    // Arm64ec would additionally turn on its CHPE-only `int 0x81` counter read,
    // which is precisely the hybrid-PE ABI this contract does not implement.
    auto HostFeatures = FEX::Windows::CPUFeatures::FetchHostFeatures(IsWine, FEXCore::HostFeatures::HostTypeEnum::Wow64);
    CTX = FEXCore::Context::Context::CreateNewContext(HostFeatures);
  }

  CTX->SetSignalDelegator(SignalDelegator.get());
  CTX->SetSyscallHandler(SyscallHandler.get());
  CTX->InitCore();
  HandlerConfig.emplace(*CTX);
  InvalidationTracker.emplace(*CTX, Threads);
  ImageTracker.emplace(*CTX, false);

  auto MainModule = reinterpret_cast<__TEB*>(NtCurrentTeb())->Peb->ImageBaseAddress;
  HandleImageMap(reinterpret_cast<uint64_t>(MainModule), true);

  CPUFeatures.emplace(*CTX);

  FEX::Windows::SetupEnvironmentVariableValues(NtDll);
  return STATUS_SUCCESS;
}

NTSTATUS BTCpu64ThreadInit() {
  std::scoped_lock Lock(ThreadCreationMutex);
  FEX::Windows::InitCRTThread();
  auto* Thread = CTX->CreateThread(0, 0);

  // Long-mode code segment at the amd64 user CS (0x33 = GDT index 6, RPL 3).
  auto Frame = Thread->CurrentFrame;
  auto NewSegments = new FEXCore::Core::CPUState::gdt_segment[32]();
  auto& GDT = NewSegments[WOWX64_SEG_CS >> 3];
  FEXCore::Core::CPUState::SetGDTBase(&GDT, 0);
  FEXCore::Core::CPUState::SetGDTLimit(&GDT, 0xF'FFFFU);
  GDT.L = 1; // L = Long Mode = 64-bit (difference 1)
  GDT.D = 0;

  Frame->State.segment_arrays[FEXCore::Core::CPUState::SEGMENT_ARRAY_INDEX_GDT] = &NewSegments[0];
  // TODO: LDTs are currently unsupported, mirror them to GDT.
  Frame->State.segment_arrays[FEXCore::Core::CPUState::SEGMENT_ARRAY_INDEX_LDT] = &NewSegments[0];

  Frame->State.cs_idx = WOWX64_SEG_CS;
  Frame->State.cs_cached = FEXCore::Core::CPUState::CalculateGDTBase(GDT);
  Frame->State.ss_idx = WOWX64_SEG_DS;
  Frame->State.ds_idx = WOWX64_SEG_DS;
  Frame->State.es_idx = WOWX64_SEG_DS;
  Frame->State.gs_cached = reinterpret_cast<uint64_t>(NtCurrentTeb());

  FEX::Windows::CallRetStack::InitializeThread(Thread);

  auto* TD = new ThreadData();
  TD->ThreadState = Thread;
  reinterpret_cast<__TEB*>(NtCurrentTeb())->WOW32Reserved = TD;

  Thread->FrontendPtr = new FrontendThreadData();

  Threads.emplace(GetCurrentThreadId(), Thread);
  return STATUS_SUCCESS;
}

NTSTATUS BTCpu64GetContext(HANDLE Thread, HANDLE Process, void* Reserved, WOWX64_CONTEXT* Context) {
  auto* TD = GetThreadData();
  if (!IsCurrentThread(Thread) || !TD) {
    return STATUS_NOT_IMPLEMENTED;
  }
  Context::StoreContextFromState(TD->ThreadState, Context);
  return STATUS_SUCCESS;
}

NTSTATUS BTCpu64SetContext(HANDLE Thread, HANDLE Process, void* Reserved, const WOWX64_CONTEXT* Context) {
  auto* TD = GetThreadData();
  if (!IsCurrentThread(Thread) || !TD) {
    return STATUS_NOT_IMPLEMENTED;
  }

  if (TD->CachedCallRetSp) {
    TD->ThreadState->CurrentFrame->State.callret_sp = TD->CachedCallRetSp;
  }
  // Copy first: the caller may pass state it is about to overwrite.
  const WOWX64_CONTEXT Tmp = *Context;
  Context::LoadStateFromContext(TD->ThreadState, Tmp);
  return STATUS_SUCCESS;
}

// .seh_pushframe doesn't restore the frame pointer, so if when unwinding from RtlCaptureContext an operation is used
// that sets SP from FP, the unwound SP value will be incorrect. Wrap RtlCaptureContext so the correct FP is immediately
// restored from the stack to prevent this. (Verbatim WOW64/Module.cpp BTCpuSimulate.)
__attribute__((naked)) void BTCpu64Simulate() {
  asm(".seh_proc BTCpu64Simulate;"
      "sub sp, sp, #0x390;"
      ".seh_stackalloc 0x390;"
      "stp x29, x30, [sp, #-0x10]!;"
      ".seh_save_fplr_x 16;"
      ".seh_endprologue;"
      "add x0, sp, #0x10;"
      "bl RtlCaptureContext;"
      "add x0, sp, #0x10;"
      "bl BTCpu64SimulateImpl;"
      "ldp x29, x30, [sp], 0x10;"
      "add sp, sp, #0x390;"
      "ret;"
      ".seh_endproc;");
}

extern "C" void BTCpu64SimulateImpl(CONTEXT* entry_context) {
  auto* TD = GetThreadData();
  TD->EntryContext = entry_context;
  TD->CachedCallRetSp = TD->ThreadState->CurrentFrame->State.callret_sp;

  Context::ScopedJITContextLock Lk {TD};
  CTX->ExecuteThread(TD->ThreadState);
}

// Returns true if exception dispatch should be halted and the execution context
// restored to Ptrs->ContextRecord. WOW64's BTCpuResetToConsistentStateImpl, minus
// the cross-thread suspend path (M1 drives one thread) and with the guest
// context in x86-64 terms.
static bool BTCpu64ResetToConsistentStateImpl(EXCEPTION_POINTERS* Ptrs) {
  auto* Context = Ptrs->ContextRecord;
  auto* Exception = Ptrs->ExceptionRecord;
  auto* TD = GetThreadData();
  auto* Thread = TD ? TD->ThreadState : nullptr;

  // Before BTCpu64ThreadInit there is no FEX state to consult, but FEX may
  // already be running (InitCore allocates) — keep on-demand commit working.
  // Same reasoning as the vibeos patch to WOW64/Module.cpp (vendor-patches/fex.md).
  if (!Thread) {
    return Exception->ExceptionCode == EXCEPTION_ACCESS_VIOLATION && OvercommitTracker &&
           OvercommitTracker->HandleAccessViolation(static_cast<uint64_t>(Exception->ExceptionInformation[1]));
  }

  if (Exception->ExceptionCode == EXCEPTION_ACCESS_VIOLATION) {
    const auto FaultAddress = static_cast<uint64_t>(Exception->ExceptionInformation[1]);

    if (FEX::Windows::CallRetStack::HandleAccessViolation(Thread, FaultAddress, Context->X25)) {
      return true;
    }

    if (OvercommitTracker && OvercommitTracker->HandleAccessViolation(FaultAddress)) {
      return true;
    }

    if (FEX::Windows::JITGuardPage::HandleJITGuardPage(Thread, reinterpret_cast<void*>(FaultAddress), Context->X,
                                                       reinterpret_cast<__uint128_t*>(Context->V), &Context->Pc)) {
      return true;
    }

    std::scoped_lock Lock(ThreadCreationMutex);
    if (InvalidationTracker->HandleRWXAccessViolation(Thread, Context->Pc, FaultAddress)) {
      if (CTX->IsAddressInCodeBuffer(Thread, Context->Pc) && !CTX->IsCurrentBlockSingleInst(Thread) &&
          CTX->IsAddressInCurrentBlock(Thread, FaultAddress & FEXCore::Utils::FEX_PAGE_MASK, FEXCore::Utils::FEX_PAGE_SIZE)) {
        // Inline SMC: reconstruct the pre-write state and single-step the write.
        Context::ReconstructThreadState(Thread, Context);
        const auto& Config = SignalDelegator->GetConfig();
        Context->Pc = Config.AbsoluteLoopTopAddressFillSRA;
        Context->X1 = 1; // ENTRY_FILL_SRA_SINGLE_INST_REG: force a single step
      }
      return true;
    }
  }

  if (!IsAddressInJit(Thread, Context->Pc)) {
    return false; // a host fault: not ours to touch
  }

  if (Exception->ExceptionCode == EXCEPTION_DATATYPE_MISALIGNMENT && Context::HandleUnalignedAccess(Thread, Context)) {
    return true;
  }

  // A genuine guest fault. Restate it in guest terms and hand it to the OS
  // side, which owns guest exception dispatch.
  if (!IsDispatcherAddress(Context->Pc)) {
    Context::ReconstructThreadState(Thread, Context);
  }
  auto& State = Thread->CurrentFrame->State;
  uint64_t Rip = State.rip;
  auto& Fault = Thread->CurrentFrame->SynchronousFaultData;
  *Exception = FEX::Windows::HandleGuestException(Fault, *Exception, Rip, State.gregs[FEXCore::X86State::REG_RAX],
                                                  State.gregs[FEXCore::X86State::REG_RCX]);
  State.rip = Rip;
  if (Exception->ExceptionCode == EXCEPTION_SINGLE_STEP) {
    CTX->SetFlagsFromCompactedEFLAGS(
      Thread, CTX->ReconstructCompactedEFLAGS(Thread, false, nullptr, 0) & ~(1U << FEXCore::X86State::RFLAG_TF_RAW_LOC));
  }
  Context::UnlockJITContext(TD);

  // Replace the host context with one captured before JIT entry so host code can unwind
  memcpy(Context, TD->EntryContext, sizeof(*Context));
  return false;
}

NTSTATUS BTCpu64ResetToConsistentState(EXCEPTION_POINTERS* Ptrs) {
  if (BTCpu64ResetToConsistentStateImpl(Ptrs)) {
    NtContinue(Ptrs->ContextRecord, FALSE);
  }
  return STATUS_SUCCESS;
}

void BTCpu64FlushInstructionCache2(const void* Address, SIZE_T Size) {
  std::scoped_lock Lock(ThreadCreationMutex);
  InvalidationTracker->InvalidateAlignedInterval(reinterpret_cast<uint64_t>(Address), static_cast<uint64_t>(Size), false);
}

// A kernel-side write into guest memory (NtWriteVirtualMemory): FEX never saw
// the store, so drop any code translated from the range. WOW64/Module.cpp's
// BTCpuNotifyMemoryDirty.
void BTCpu64NotifyMemoryDirty(void* Address, SIZE_T Size) {
  std::scoped_lock Lock(ThreadCreationMutex);
  InvalidationTracker->InvalidateAlignedInterval(reinterpret_cast<uint64_t>(Address), static_cast<uint64_t>(Size), false);
}

// NtReadFile into guest memory, bracketed: the kernel writes the buffer with
// FEX's SMC write-protection lifted and code invalidated for the range.
// WOW64/Module.cpp's BTCpuNotifyReadFile, with the per-thread flag reached
// through this module's ThreadData.
void BTCpu64NotifyReadFile(HANDLE Handle, void* Address, SIZE_T Size, BOOL After, NTSTATUS Status) {
  auto* TD = GetThreadData();
  if (!TD) {
    return;
  }
  auto& InLockedRWXRead = static_cast<FrontendThreadData*>(TD->ThreadState->FrontendPtr)->InLockedRWXRead;
  if (!After) {
    ThreadCreationMutex.lock();
    CTX->GetCodeInvalidationMutex().lock();
    if (InvalidationTracker->BeginUntrackedWriteLocked(reinterpret_cast<uint64_t>(Address), static_cast<uint64_t>(Size))) {
      InLockedRWXRead = true;
    } else {
      CTX->GetCodeInvalidationMutex().unlock();
      ThreadCreationMutex.unlock();
    }
  } else {
    if (InLockedRWXRead) {
      InLockedRWXRead = false;
      CTX->GetCodeInvalidationMutex().unlock();
      ThreadCreationMutex.unlock();
    }
  }
}

void BTCpu64NotifyMemoryAlloc(void* Address, SIZE_T Size, ULONG Type, ULONG Prot, BOOL After, ULONG Status) {
  if (!After) {
    ThreadCreationMutex.lock();
  } else {
    // MEM_RESET(_UNDO) ignores the passed permissions
    if (!Status && !(Type & (MEM_RESET | MEM_RESET_UNDO))) {
      InvalidationTracker->HandleMemoryProtectionNotification(reinterpret_cast<uint64_t>(Address), static_cast<uint64_t>(Size), Prot);
    }
    ThreadCreationMutex.unlock();
  }
}

void BTCpu64NotifyMemoryProtect(void* Address, SIZE_T Size, ULONG NewProt, BOOL After, ULONG Status) {
  if (!After) {
    ThreadCreationMutex.lock();
  } else {
    if (!Status) {
      InvalidationTracker->HandleMemoryProtectionNotification(reinterpret_cast<uint64_t>(Address), static_cast<uint64_t>(Size), NewProt);
    }
    ThreadCreationMutex.unlock();
  }
}

void BTCpu64NotifyMemoryFree(void* Address, SIZE_T Size, ULONG FreeType, BOOL After, ULONG Status) {
  if (!After) {
    ThreadCreationMutex.lock();
  } else {
    if (!Status) {
      InvalidationTracker->InvalidateAlignedInterval(reinterpret_cast<uint64_t>(Address), static_cast<uint64_t>(Size), true);
    }
    ThreadCreationMutex.unlock();
  }
}

NTSTATUS BTCpu64NotifyMapViewOfSection(void* Unk1, void* Address, void* Unk2, SIZE_T Size, ULONG AllocType, ULONG Prot) {
  std::scoped_lock Lock(ThreadCreationMutex);
  HandleImageMap(reinterpret_cast<uint64_t>(Address));
  return STATUS_SUCCESS;
}

void BTCpu64NotifyUnmapViewOfSection(void* Address, BOOL After, ULONG Status) {
  if (!After) {
    ThreadCreationMutex.lock();
    auto [Start, Size] = InvalidationTracker->InvalidateContainingSection(reinterpret_cast<uint64_t>(Address), true);
    if (Size) {
      HandleImageUnmap(Start, Size);
    }
  } else {
    ThreadCreationMutex.unlock();
  }
}
