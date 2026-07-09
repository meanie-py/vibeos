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
// The guest blob (64-bit):
//     48 c7 c0 2a 00 00 00   mov rax, 0x2A     ; 42
//     48 83 c0 07            add rax, 7        ; -> 49
//     f4                     hlt               ; EnableExitOnHLT -> ExecuteThread returns
// Success = guest RAX reads back as 0x31 (49). The arithmetic is deliberately
// two ops so a wrong result distinguishes "ran the blob" from "returned zero".

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

constexpr uint64_t GuestExpected = 0x31; // 42 + 7

// Hand-assembled x86-64: mov rax,0x2A ; add rax,7 ; hlt
const uint8_t GuestBlob[] = {
  0x48, 0xC7, 0xC0, 0x2A, 0x00, 0x00, 0x00, // mov rax, 0x2A
  0x48, 0x83, 0xC0, 0x07,                   // add rax, 7
  0xF4,                                     // hlt
};

// 64-bit long-mode code segment (GDT index 6<<3 == DEFAULT_USER_CS on x86-64).
constexpr size_t DefaultCS = FEXCore::Core::CPUState::DEFAULT_USER_CS;
FEXCore::Core::CPUState::gdt_segment gdt[32] {};
} // namespace

extern "C" int main(int /*argc*/, char** /*argv*/) {
  DbgPrint("[FEX-HELLO] rung-2 hello-compute harness start\n");

  // 64-bit guest.
  FEXCore::Config::Initialize();
  FEXCore::Config::Set(FEXCore::Config::CONFIG_IS64BIT_MODE, "1");

  const auto NtDll = GetModuleHandleA("ntdll.dll");
  const bool IsWine = NtDll && !!GetProcAddress(NtDll, "wine_get_version");
  auto HostFeatures = FEX::Windows::CPUFeatures::FetchHostFeatures(IsWine, FEXCore::HostFeatures::HostTypeEnum::Arm64ec);

  auto CTX = FEXCore::Context::Context::CreateNewContext(HostFeatures);

  auto SignalDelegation = FEX::DummyHandlers::CreateSignalDelegator();
  auto SyscallHandler = FEX::DummyHandlers::CreateSyscallHandler();
  CTX->SetSignalDelegator(SignalDelegation.get());
  CTX->SetSyscallHandler(SyscallHandler.get());

  if (!CTX->InitCore()) {
    DbgPrint("[FEX-HELLO] FAIL: InitCore returned false\n");
    return 1;
  }

  // Exit ExecuteThread when the guest hits HLT (instead of trapping).
  CTX->EnableExitOnHLT();

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
  const bool Pass = (GuestRAX == GuestExpected);
  DbgPrint("[FEX-HELLO] guest RAX = 0x%llx (expected 0x%llx) (%s)\n",
           (unsigned long long)GuestRAX, (unsigned long long)GuestExpected, Pass ? "PASS" : "FAIL");

  CTX->DestroyThread(Thread);
  DbgPrint("[FEX-HELLO] done\n");
  return Pass ? 0 : 1;
}
