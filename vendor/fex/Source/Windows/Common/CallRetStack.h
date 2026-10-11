// SPDX-License-Identifier: MIT
#pragma once

#include <algorithm>

#include <FEXCore/Core/Context.h>
#include <FEXCore/Utils/Allocator.h>
#include <FEXCore/Utils/LogManager.h>
#include <FEXCore/Debug/InternalThreadState.h>

namespace FEX::Windows::CallRetStack {
struct CallRetStackInfo {
  uint64_t AllocationBase;
  uint64_t AllocationEnd;
  uint64_t DefaultLocation;
};

CallRetStackInfo GetInfoThread(FEXCore::Core::InternalThreadState* Thread) {
  uint64_t Base = reinterpret_cast<uint64_t>(Thread->CallRetStackBase);
  // Leave some room from the base for the default location to allow for underflows without constant exceptions
  return {Base - FEXCore::Utils::FEX_PAGE_SIZE, Base + FEXCore::Core::InternalThreadState::CALLRET_STACK_SIZE + FEXCore::Utils::FEX_PAGE_SIZE,
          Base + FEXCore::Core::InternalThreadState::CALLRET_STACK_SIZE / 4};
}

void InitializeThread(FEXCore::Core::InternalThreadState* Thread) {
  // Allocate the call-ret stack with guard pages on both sides
  const void* CallRetStackAlloc = ::VirtualAlloc(nullptr, FEXCore::Core::InternalThreadState::CALLRET_STACK_SIZE + 2 * FEXCore::Utils::FEX_PAGE_SIZE,
                                                 MEM_RESERVE | MEM_TOP_DOWN, PAGE_NOACCESS);

  FEXCore::Allocator::VirtualName("FEXMem_CallRetStacks", CallRetStackAlloc,
                                  FEXCore::Core::InternalThreadState::CALLRET_STACK_SIZE + 2 * FEXCore::Utils::FEX_PAGE_SIZE);
  FEXCore::Allocator::VirtualTHPControl(CallRetStackAlloc, FEXCore::Core::InternalThreadState::CALLRET_STACK_SIZE + 2 * FEXCore::Utils::FEX_PAGE_SIZE,
                                        FEXCore::Allocator::THPControl::Disable);

  Thread->CallRetStackBase = reinterpret_cast<void*>(reinterpret_cast<uintptr_t>(CallRetStackAlloc) + FEXCore::Utils::FEX_PAGE_SIZE);
  // VIBEOS PATCH (commit on touch): left reserved; HandleAccessViolation commits each page
  // the stack reaches. 4 MB a thread committed up front, and recommitted at every zeroing,
  // was most of what a FEX process charged against vibeos's pagefile-less commit limit.

  Thread->CurrentFrame->State.callret_sp = GetInfoThread(Thread).DefaultLocation;
}

void DestroyThread(FEXCore::Core::InternalThreadState* Thread) {
  auto CallRetStackInfo = GetInfoThread(Thread);
  ::VirtualFree(reinterpret_cast<void*>(CallRetStackInfo.AllocationBase), 0, MEM_RELEASE);
}

bool HandleAccessViolation(FEXCore::Core::InternalThreadState* Thread, uint64_t Address, uint64_t& CallRetSPReg) {
  auto CallRetStackInfo = GetInfoThread(Thread);
  // VIBEOS PATCH (commit on touch): a page of the stack proper that is only reserved (never
  // touched, or zeroed since) is committed and the access retried. The guard pages either
  // side stay reserved, so a run off either end is still an imbalance below.
  const uint64_t StackBase = reinterpret_cast<uint64_t>(Thread->CallRetStackBase);
  const uint64_t StackEnd = StackBase + FEXCore::Core::InternalThreadState::CALLRET_STACK_SIZE;
  if (Address >= StackBase && Address < StackEnd) {
    void* Page = reinterpret_cast<void*>(Address & FEXCore::Utils::FEX_PAGE_MASK);
    MEMORY_BASIC_INFORMATION Info;
    if (::VirtualQuery(Page, &Info, sizeof(Info)) && Info.State == MEM_RESERVE) {
      static constexpr uint64_t MaxFaultCommitSize = 1024 * 64;
      const uint64_t Size = std::min({MaxFaultCommitSize, StackEnd - reinterpret_cast<uint64_t>(Page), static_cast<uint64_t>(Info.RegionSize)});
      if (::VirtualAlloc(Page, Size, MEM_COMMIT, PAGE_READWRITE)) {
        return true;
      }
    }
  }
  if (Address >= CallRetStackInfo.AllocationBase && Address < CallRetStackInfo.AllocationEnd) {
    LogMan::Msg::DFmt("Call-ret stack inbalance: {:X}", Address);
    CallRetSPReg = CallRetStackInfo.DefaultLocation;
    return true;
  }
  return false;
}
} // namespace FEX::Windows::CallRetStack
