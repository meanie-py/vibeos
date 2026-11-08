// SPDX-License-Identifier: MIT
// VIBEOS PATCH (uniprocessor TSO, tripwire options): this whole file. Both CPU layers (WOW64/Module.cpp
// and vibeos's tools/fex/wowx64fex) call it before creating their context.
#pragma once

#include <FEXCore/Config/Config.h>

#include <cstdlib>
#include <cwchar>
#include <string>
#include <vector>
#include <windef.h>
#include <winternl.h>

namespace FEX::Windows {
namespace UniprocessorTSODetail {
// Calls `Visit(Token, Length)` for each token of this boot's tripwire config (published by the
// kernel as HKLM\SYSTEM\CurrentControlSet\Control\VibeTripwire), until it returns true.
// Returns whether it did.
template<typename F>
inline bool ForEachTripwireToken(F&& Visit) {
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
  size_t Start = 0;
  for (size_t i = 0; i <= Count; i++) {
    const bool End = i == Count || Text[i] == L' ' || Text[i] == L';' || Text[i] == L'\n' || Text[i] == 0;
    if (!End) {
      continue;
    }
    if (i > Start && Visit(Text + Start, i - Start)) {
      return true;
    }
    if (i < Count && Text[i] == 0) {
      break;
    }
    Start = i + 1;
  }
  return false;
}

inline bool TripwireHasToken(const wchar_t* Token) {
  const size_t TokenLength = wcslen(Token);
  return ForEachTripwireToken(
    [&](const wchar_t* Text, size_t Length) { return Length == TokenLength && !wcsncmp(Text, Token, TokenLength); });
}

// A `FEX_<OPTION>=<value>` token other than FEX_TSO=1: a FEX config option, spelled as its
// environment variable.
inline bool IsOptionToken(const wchar_t* Text, size_t Length) {
  if (Length < 6 || wcsncmp(Text, L"FEX_", 4) || !wmemchr(Text, L'=', Length)) {
    return false;
  }
  return !(Length == 9 && !wcsncmp(Text, L"FEX_TSO=1", 9));
}
} // namespace UniprocessorTSODetail

// The environment FEX should load its config from: this boot's `FEX_<OPTION>=<value>` tripwire
// tokens, then `Environment`. FEX's environment layer lets a later entry win, so a variable the
// process really has beats the tripwire's. This is how a boot A/Bs any FEX option
// (`--tripwire FEX_MULTIBLOCK=0`) without a config file in the image. The strings live for the
// process, because FEX keeps views of them until it reloads the layer.
inline char** EnvironmentWithTripwireOptions(char** Environment) {
  static std::vector<std::string> Options;
  static std::vector<char*> Result;
  Options.clear();
  Result.clear();
  UniprocessorTSODetail::ForEachTripwireToken([](const wchar_t* Text, size_t Length) {
    if (UniprocessorTSODetail::IsOptionToken(Text, Length)) {
      Options.emplace_back(Text, Text + Length); // the token grammar is ASCII
    }
    return false;
  });
  for (auto& Option : Options) {
    Result.push_back(Option.data());
  }
  for (char** Entry = Environment; Entry && *Entry; Entry++) {
    Result.push_back(*Entry);
  }
  Result.push_back(nullptr);
  return Result.data();
}

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
  // An explicit FEX_TSOENABLED, in the environment or the tripwire, is a deliberate choice; keep it.
  if (getenv("FEX_TSOENABLED") || UniprocessorTSODetail::ForEachTripwireToken([](const wchar_t* Text, size_t Length) {
        return Length > 15 && !wcsncmp(Text, L"FEX_TSOENABLED=", 15);
      })) {
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
