// SPDX-License-Identifier: MIT
#pragma once

#include <FEXCore/Utils/IntervalList.h>
#include <algorithm>
#include <thread>
#include <shared_mutex>


namespace FEX::Windows {
/**
 * @brief Emulates memory overcommit of reserved regions with exceptions
 */
class OvercommitTracker {
private:
  bool IsWine;
  FEXCore::IntervalList<uint64_t> OvercommitIntervals;
  std::shared_mutex OvercommitIntervalsMutex;

public:
  OvercommitTracker(bool IsWine)
    : IsWine {IsWine} {}

  void MarkRange(uint64_t Start, uint64_t Length) {
    std::unique_lock Lock {OvercommitIntervalsMutex};
    OvercommitIntervals.Insert({Start, Start + Length});
  }

  void UnmarkRange(uint64_t Start, uint64_t Length) {
    std::unique_lock Lock {OvercommitIntervalsMutex};
    OvercommitIntervals.Remove({Start, Start + Length});
  }

  bool HandleAccessViolation(uint64_t FaultAddress) {
    std::shared_lock Lock {OvercommitIntervalsMutex};
    auto Query = OvercommitIntervals.Query(FaultAddress);

    if (Query.Enclosed) {
      if (IsWine) {
        MEMORY_BASIC_INFORMATION Info;
        NtQueryVirtualMemory(NtCurrentProcess(), reinterpret_cast<void*>(FaultAddress), MemoryBasicInformation, &Info, sizeof(Info), nullptr);
        const auto CommitSize = reinterpret_cast<SIZE_T>(Info.BaseAddress) + Info.RegionSize - reinterpret_cast<SIZE_T>(Info.AllocationBase);
        VirtualAlloc(reinterpret_cast<void*>(Info.AllocationBase), CommitSize, MEM_COMMIT, PAGE_READWRITE);
      } else {
        static constexpr size_t MaxFaultCommitSize = 1024 * 64;
        const auto AlignedFaultAddress = reinterpret_cast<void*>(FaultAddress & FEXCore::Utils::FEX_PAGE_MASK);
        // VIBEOS PATCH (commit on touch): commit only pages still merely RESERVED, never
        // past the end of that run, and with the protection the range was reserved with.
        // A fault on a committed page is someone else's (a JIT guard page, an RWX write
        // trap), and committing over it would reset its protection; code buffers are
        // reserved RWX and must come up executable. A refused commit (the system is at its
        // commit limit) is not handled: the fault stands.
        MEMORY_BASIC_INFORMATION Info;
        if (NtQueryVirtualMemory(NtCurrentProcess(), AlignedFaultAddress, MemoryBasicInformation, &Info, sizeof(Info), nullptr) ||
            Info.State != MEM_RESERVE) {
          return false;
        }
        const size_t CommitSize = std::min({static_cast<size_t>(Query.Size), MaxFaultCommitSize, static_cast<size_t>(Info.RegionSize)});
        return VirtualAlloc(AlignedFaultAddress, CommitSize, MEM_COMMIT, Info.AllocationProtect ? Info.AllocationProtect : PAGE_READWRITE) !=
               nullptr;
      }
      return true;
    }
    return false;
  }
};
} // namespace FEX::Windows
