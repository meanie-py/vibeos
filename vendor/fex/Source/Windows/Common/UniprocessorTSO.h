// SPDX-License-Identifier: MIT
// VIBEOS PATCH (uniprocessor TSO): this whole file. Both CPU layers (WOW64/Module.cpp
// and vibeos's tools/fex/wowx64fex) call it before creating their context.
#pragma once

#include <FEXCore/Config/Config.h>

#include <cstdlib>
#include <windef.h>
#include <winternl.h>

namespace FEX::Windows {
namespace UniprocessorTSODetail {
// Does this boot's tripwire config (published by the kernel as
// HKLM\SYSTEM\CurrentControlSet\Control\VibeTripwire) hold `Token`?
inline bool TripwireHasToken(const wchar_t* Token) {
  UNICODE_STRING KeyName, ValueName;
  RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control");
  RtlInitUnicodeString(&ValueName, L"VibeTripwire");
  OBJECT_ATTRIBUTES Attributes;
  InitializeObjectAttributes(&Attributes, &KeyName, OBJ_CASE_INSENSITIVE, nullptr, nullptr);
  HANDLE Key;
  if (NtOpenKeyEx(&Key, KEY_QUERY_VALUE, &Attributes, 0)) {
    return false;
  }
  alignas(8) unsigned char Buffer[sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 512] {};
  DWORD Length = 0;
  const NTSTATUS Status = NtQueryValueKey(Key, &ValueName, KeyValuePartialInformation, Buffer, sizeof(Buffer), &Length);
  NtClose(Key);
  const auto* Info = reinterpret_cast<const KEY_VALUE_PARTIAL_INFORMATION*>(Buffer);
  if (Status || Info->Type != REG_SZ) {
    return false;
  }
  // The kernel's grammar: tokens separated by space, semicolon or newline.
  const auto* Text = reinterpret_cast<const wchar_t*>(Info->Data);
  const size_t Count = Info->DataLength / sizeof(wchar_t);
  const size_t TokenLength = wcslen(Token);
  size_t Start = 0;
  for (size_t i = 0; i <= Count; i++) {
    const bool End = i == Count || Text[i] == L' ' || Text[i] == L';' || Text[i] == L'\n' || Text[i] == 0;
    if (!End) {
      continue;
    }
    if (i - Start == TokenLength && !wcsncmp(Text + Start, Token, TokenLength)) {
      return true;
    }
    if (i < Count && Text[i] == 0) {
      break;
    }
    Start = i + 1;
  }
  return false;
}
} // namespace UniprocessorTSODetail

// TSO emulation makes the JIT emit every guest load and store as an ordered
// (acquire/release) access, so that x86 code whose threads run on two cores at
// once sees x86's store ordering. With one processor no two threads run at once:
// a context switch is a full barrier on that core, and the code each thread runs
// observes its own accesses in program order. So the emulation buys nothing there,
// and costs on every memory access the guest makes. Turn it off when the system
// has one processor, unless the configuration says otherwise.
//
// Call after the config is loaded and before the context is created: the
// context reads TSOEnabled when it is constructed. Returns whether it turned
// the emulation off.
//
// The processor count is the PEB's, which the kernel fills from its active
// processor count. A processor added after the process started would not be
// seen; we have no processor hot-add.
inline bool DisableTSOOnUniprocessor() {
  // An explicit FEX_TSOENABLED in the environment is a deliberate choice; keep it.
  if (getenv("FEX_TSOENABLED")) {
    return false;
  }
  // The per-boot escape hatch, for A/B timing and for ruling this out.
  if (UniprocessorTSODetail::TripwireHasToken(L"FEX_TSO=1")) {
    return false;
  }
  if (reinterpret_cast<__TEB*>(NtCurrentTeb())->Peb->NumberOfProcessors != 1) {
    return false;
  }
  FEXCore::Config::Set(FEXCore::Config::CONFIG_TSOENABLED, "0");
  return true;
}
} // namespace FEX::Windows
