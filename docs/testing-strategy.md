# ReactOS Testing Strategy

## Current State

ReactOS has ~1,900 test files across ~1.16M LOC in `reactos/modules/rostests/`. All tests are
**in-target** -- they compile as PE executables and run inside a booted ReactOS instance in QEMU.
There is no host-side testing capability.

### Test Categories

| Category | Location | Tests | Framework |
|---|---|---|---|
| Kernel-mode (kmtest) | `rostests/kmtests/` | 78 tests | `kmtest_drv.sys` + `kmtest_.exe` |
| API tests | `rostests/apitests/` | 79 modules | Wine test.h |
| Wine compat tests | `rostests/winetests/` | 132 modules | Wine test.h |
| Win32 integration | `rostests/win32/` | 14 modules | Wine test.h |

### How Kernel Tests Run Today

1. `kmtest_.exe` (user-mode) installs `kmtest_drv.sys` as a kernel service via SCM
2. Sends IOCTLs to trigger individual test functions inside the driver
3. Results collected from shared `KMT_RESULTBUFFER` (success/failure/skip counters + log)
4. Some tests have split user/kernel design (`_user.c` + `_drv/` companion driver)

### Existing User-Space Emulation

The test framework has `KMT_EMULATE_KERNEL` mode (`kmt_platform.h`):
- `ExAllocatePool` -> `HeapAlloc(GetProcessHeap())`
- `ExFreePool` -> `HeapFree`
- `KeRaiseIrql` -> no-op
- `ok_irql()` -> disabled

A few tests opt in (`RtlUnicodeString.c`, `RtlStrSafe.c`). This is a band-aid on the test
code, not a structural separation in the kernel code itself.

## Goal: Host-Side Testing on macOS

Build and run kernel algorithm tests natively on the development machine (macOS/ARM64 or x86_64)
without booting QEMU. This gives:
- Sub-second test feedback (vs 60+ second QEMU boot)
- CI/CD integration
- Foundation for Rust+Kani/Loom formal verification (long-term)

## Decoupling Assessment

### GREEN -- Trivially Portable

Pure algorithms with no/minimal kernel dependencies. Can run on host with a thin shim header.

| Test | Algorithm | Kernel Dependency | Mitigation |
|---|---|---|---|
| RtlAvlTree | AVL tree insert/delete/lookup | ARM64 QEMU MMIO barrier (weak-linked) | `Arm64UartKernelBase = NULL` |
| RtlSplayTree | Splay tree operations | Same | Same |
| RtlUnicodeString | Unicode string search/compare | None | Already `KMT_EMULATE_KERNEL` |
| RtlStrSafe | Safe string formatting | None | Already `KMT_EMULATE_KERNEL` |
| RtlMemory | memcpy/memset/memcmp | IRQL raise (no-op in emulation) | Already handled |
| RtlIntSafe | Integer overflow checks | None | Pure arithmetic |
| ExDoubleList | Interlocked doubly-linked list | IRQL checks (compile-time disabled) | Already handled |
| ExSingleList | Interlocked singly-linked list | Same | Same |
| FsRtlExpression | Wildcard pattern matching | `ExAllocatePoolWithTag` fallback | -> `malloc` |
| FsRtlDissect | Path component splitting | None | Pure string parsing |
| FsRtlLegal | Legal character validation | None | Pure table lookup |
| RtlRangeList | Range list management | Pool allocation | -> `malloc` |

### YELLOW -- Moderate Stubbing Required

Core algorithm is portable but calls kernel allocation/synchronization at boundaries.

| Test | Algorithm | Stubs Needed |
|---|---|---|
| ExPools | Pool stress test | `ExAllocatePoolWithTag` -> `malloc` |
| ExResource | Shared/exclusive lock | IRQL, critical regions -> mutex |
| ExFastMutex | Fast mutex semantics | IRQL -> no-op, dispatcher -> pthread |
| ZwAllocateVirtualMemory | VM alloc edge cases | Syscall -> `mmap` mock |
| KeEvent, KeMutex | Sync object semantics | Dispatcher -> pthread_cond/mutex |
| FsRtlMcb | Map control block | Pool allocation |
| FsRtlTunnel | Tunnel cache | Pool + timer |

### RED -- Deeply Kernel-Coupled

Require live kernel state, hardware, or page tables. Not feasible without full kernel environment.

| Test | Why |
|---|---|
| MmMdl | Physical page frames, PFN database, kernel VA mapping |
| MmSection, ZwCreateSection, ZwMapViewOfSection | Section objects, demand paging |
| ObReference, ObTypes, ObHandle, ObSecurity | Live object manager namespace + handle table |
| KeIrql, KeSpinLock | Testing the actual IRQL/spinlock hardware mechanism |
| IoCreateFile, IoIrp, IoInterrupt | Full I/O manager + device stack |
| CcCopyRead/Write, CcMapData | Cache manager tied to Mm and Io |
| KeDpc, KeApc | DPC/APC queue delivery (interrupt-driven) |
| NpfsConnect/Create/ReadWrite | Named pipe filesystem driver |

## Why RED Tests Are Coupled (Code, Not Algorithm)

The algorithms inside RED subsystems are often simple. The coupling is an artifact of
1990s C coding style, not inherent complexity.

### Example: PFN List Manager (`ntoskrnl/mm/ARM3/pfnlist.c`)

`MiUnlinkPageFromList` is a **doubly-linked list unlink** using array indices instead of pointers:

```c
OldFlink = Pfn->u1.Flink;
OldBlink = Pfn->u2.Blink;
if (OldFlink != LIST_HEAD)
    MI_PFN_ELEMENT(OldFlink)->u2.Blink = OldBlink;  // MI_PFN_ELEMENT = &MmPfnDatabase[i]
else
    ListHead->Blink = OldBlink;
// ... symmetric for back link
ListHead->Total--;
```

What makes it "RED":
- `MI_PFN_ELEMENT(i)` = `&MmPfnDatabase[i]` -- global array allocated from physical memory at boot
- `MI_ASSERT_PFN_LOCK_HELD()` -- asserts spinlock held at DISPATCH_LEVEL
- `MmAvailablePages`, `MmTransitionSharedPages` -- global counters modified by multiple subsystems
- `MmPageLocationList[]` -- global dispatch table mapping page states to list heads

The algorithm itself (indexed linked list with state-based dispatch) is trivially testable.
The problem is there's no abstraction boundary between the algorithm and its kernel bindings.

### Example: Object Manager Ref Counting (`ntoskrnl/ob/obref.c`)

`ObReferenceObjectSafe` is a **lock-free CAS reference count increment**:

```c
do {
    OldValue = ObjectHeader->PointerCount;
    if (!OldValue) return FALSE;
    NewValue = InterlockedCompareExchangeSizeT(&ObjectHeader->PointerCount, OldValue + 1, OldValue);
} while (OldValue != NewValue);
return TRUE;
```

Pure concurrency algorithm. But coupled to:
- `OBJECT_TO_OBJECT_HEADER()` -- pointer arithmetic macro (header prepended before object body)
- `ObpDeferObjectDeletion` -- pushes to lock-free LIFO, then `ExQueueWorkItem` (kernel work queue)

### Example: Handle Table (`ntoskrnl/ex/handle.c`)

`ExpLookupHandleTableEntry` is a **3-level radix tree lookup** (flat for <512 handles, 2-level
for <512K, 3-level beyond). Pure data structure. But hardwired to `ExFreePoolWithTag`,
`PsReturnProcessPagedPoolQuota`, etc.

### Pattern Summary

| Coupling Type | Example | Kernel Part (20-30%) | Algorithm Part (70-80%) |
|---|---|---|---|
| Data structure + globals | PFN lists | `MmPfnDatabase` array, PFN spinlock | Indexed linked list ops |
| Concurrency + side effect | ObRef | `ExQueueWorkItem` for deferred delete | Lock-free CAS refcount |
| Multi-level lookup + alloc | Handle table | Pool allocator calls | 3-level radix tree |

## Long-Term: Rust + Formal Verification

The decoupling work for host-side testing is a prerequisite for:

1. **Rust rewrites of core algorithms** -- extract the pure algorithm, rewrite in Rust with
   the same semantics, FFI-bind back into the C kernel
2. **Kani verification** -- prove absence of panics, integer overflows, out-of-bounds access
   in the pure algorithm layer
3. **Loom verification** -- prove correctness of concurrent algorithms (PFN lock ordering,
   object ref counting, handle table concurrent access) under all possible thread interleavings

The GREEN tests are the starting point because they require zero decoupling work. The YELLOW
tests require a stub layer. The RED tests require actual refactoring of the kernel code to
separate algorithm from kernel bindings -- that's the medium-term project.

## Implementation Plan

### Phase 1: Host-Side GREEN Tests (Current)
- Build a macOS-native test harness using clang
- Provide shim headers for NT types (`NTSTATUS`, `UNICODE_STRING`, `ULONG`, etc.)
- Compile and run ~12 GREEN tests natively
- Validate against QEMU in-target results

### Phase 2: Host-Side YELLOW Tests
- Create stub library: `ExAllocatePoolWithTag` -> `malloc`, IRQL -> no-op, etc.
- Port ~7 additional tests
- Build mock for `ZwAllocateVirtualMemory` -> `mmap`

### Phase 3: Algorithm Extraction for RED Tests
- Refactor PFN list ops into pure functions taking array + indices
- Refactor object manager refcount into generic CAS primitives
- Refactor handle table into standalone radix tree
- Write new host-side tests for extracted algorithms

### Phase 4: Rust + Kani/Loom
- Rewrite extracted algorithms in Rust
- Kani proofs for safety properties
- Loom tests for concurrency correctness
- FFI bindings back into C kernel
