# Rust Rewrite Roadmap

Tracking the incremental replacement of C code with Rust in ReactOS.
All Rust crates compile to PE-COFF static libraries and link into the C build.
Targets: `aarch64-pc-windows-gnullvm`, `x86_64-pc-windows-gnu`, `i686-pc-windows-gnu`.

## Completed (19 crates, ~5,030 lines of C replaced)

| Crate | C file(s) | Lines | Functions | Notes |
|-------|-----------|-------|-----------|-------|
| `reactos-slist` | slist.S (3 arches) | ~600 | 3 | Lock-free SList with LDXP/CMPXCHG16B/CMPXCHG8B |
| `reactos-bitmap` | bitmap.c | 957 | 18 | RtlBitmap operations |
| `reactos-avltable` | avltable.c + avlsupp.c | 790 | 12 | AVL generic table with NT callbacks |
| `reactos-compress` | compress.c | 439 | 8 | LZNT1 compression/decompression |
| `reactos-utf8` | utf8.c | 200 | 2 | UTF-8 ↔ UTF-16 conversion |
| `reactos-crc32` | crc32.c | 106 | 1 | CRC-32 checksum |
| `reactos-random` | random.c | 205 | 3 | LCG random number generators |
| `reactos-encode` | encode.c | 78 | 2 | Unicode string XOR encoding |
| `reactos-largeint` | largeint.c | 360 | 15 | 64-bit integer arithmetic |
| `reactos-interlck` | interlck.c | 26 | 1 | Atomic CAS64 via `AtomicI64` |
| `reactos-prefix` | prefix.c | 47 | 4 | UNIMPLEMENTED stubs (PfxInitialize etc.) |
| `reactos-luid` | luid.c | 64 | 3 | LUID copy/compare |
| `reactos-access` | access.c | 65 | 3 | Access mask bit checks + generic mapping |
| `reactos-byteswap` | byteswap.c | 97 | 3 | Endian swap (amd64+arm64; i386 keeps rtlswap.S) |
| `reactos-mem` | mem.c | 162 | 7 | CompareMemory, FillMemory, ZeroMemory (arm64+amd64; i386 keeps rtlmem.s) |
| `reactos-handle` | handle.c | 233 | 6 | Handle table with virtual memory backing |
| `reactos-time` | time.c | 423 | 10 | Date/time arithmetic, epoch conversions |
| `reactos-splaytree` | splaytree.c | 796 | 7 | Splay tree operations (pairs with generic table) |
| `reactos-generictable` | generictable.c | 613 | 13 | Splay-based generic table with NT callbacks |

## Phase 1: RTL Cleanup (~3,500 lines)

Finish all pure/light RTL files. No architectural changes needed.

### Batch A — Trivial (< 100 lines each) ✓ DONE

All 5 files ported. `byteswap.c` replaced for amd64+arm64 only (i386 keeps `rtlswap.S` for FASTCALL ABI).

### Batch B — Pure Algorithmic ✓ DONE

5 of 6 files ported (38 unit tests). `network.c` deferred — too many C library dependencies (variadic `RtlStringCchPrintfExW`, `towlower`, `wcstoul`, `RtlAnsiStringToUnicodeString`).

| File | Lines | Functions | Status |
|------|-------|-----------|--------|
| `mem.c` | 162 | 7 | ✓ Done (arm64+amd64; i386 keeps rtlmem.s) |
| `handle.c` | 233 | 6 | ✓ Done |
| `time.c` | 423 | 10 | ✓ Done |
| `splaytree.c` | 796 | 7 | ✓ Done |
| `generictable.c` | 613 | 13 | ✓ Done |
| `network.c` | 1035 | 17 | Deferred (heavy C deps) |

### Batch C — Light Dependencies

| File | Lines | Functions | Deps | Complexity |
|------|-------|-----------|------|------------|
| `dos8dot3.c` | 321 | 3 | NLS table | 8.3 filename generation |
| `rangelist.c` | 808 | 12 | Heap alloc | Range list management |
| `error.c` | 1559 | 9 | Table lookup | NTSTATUS → Win32 error mapping |

## Phase 2: Security Descriptors (~2,650 lines)

Surprisingly portable — stateless buffer manipulation, no syscalls, no globals.

| File | Lines | Functions | Pure % | Notes |
|------|-------|-----------|--------|-------|
| `sid.c` | 421 | 15 | ~90% | SID validation, comparison, formatting. Only `RtlValidSid` needs SEH. |
| `acl.c` | 1031 | 21 | ~70% | ACE insertion/deletion/enumeration. Stateless buffer ops. |
| `sd.c` | 1205 | 25 | ~60% | Owner/group/DACL/SACL getters/setters, absolute↔self-relative conversion. |

These form a dependency chain: **SID → ACL → SD**. Port in that order.
SEH is the only blocker, and with DUMMY_PSEH it's already dead code in our build.

## Phase 3: Crypto (~3,500 lines)

`sdk/lib/cryptlib/` — Pure algorithmic, zero OS dependencies.

| File | Lines | Functions | Notes |
|------|-------|-----------|-------|
| `rc4.c` | 67 | 2 | RC4 stream cipher |
| `sha1.c` | 199 | 3 | SHA-1 hash |
| `md4.c` | 247 | 3 | MD4 hash |
| `md5.c` | 257 | 3 | MD5 hash |
| `aes.c` | 1266 | 4 | AES block cipher |
| `des.c` | 1495 | 4 | DES/3DES block cipher |
| `mvAesAlg.c` | 317 | — | AES variant |

## Phase 4: Other SDK Libraries (~1,500 lines)

| Library | File | Lines | Notes |
|---------|------|-------|-------|
| `sdk/lib/skiplist/` | `skiplist.c` | 440 | Probabilistic search tree |
| `sdk/lib/tzlib/` | `tzlib.c` | 356 | Timezone offset calculation |
| `sdk/lib/inflib/` | `infcore.c` + `infget.c` | 1586 | .INF file parser |

## Phase 5: Kernel Infrastructure (~5,000 lines)

Higher risk, higher reward. Requires careful dependency analysis.

### FSRtl (ntoskrnl/fsrtl/) — Filesystem utilities

| File | Lines | Functions | Notes |
|------|-------|-----------|-------|
| `name.c` | 546 | 4 | Filename validation/wildcards |
| `tunnel.c` | 909 | 5 | Short name tunneling cache |
| `largemcb.c` | 1106 | 12 | VBN↔LBN mapping (uses AVL tables) |

### Registry Core (sdk/lib/cmlib/) — Hive management

| File | Lines | Functions | Notes |
|------|-------|-----------|-------|
| `cmvalue.c` | 606 | 6 | Registry value operations |
| `cmindex.c` | 1876 | 8 | Registry index tree (B-tree) |

## Phase 6: Heavy RTL — Kernel FFI Required

These files need a shared `reactos-ntffi` crate providing `extern "system"` declarations
for NT syscalls. The gate is building that abstraction layer.

### Tier A — Unlocked by VM + Sync FFI (~5,500 lines)

| File | Lines | Pure % | FFI Needed | Notes |
|------|-------|--------|------------|-------|
| `critical.c` | 888 | ~40% | `NtCreateEvent`, `NtWaitForKeyedEvent`, `NtReleaseKeyedEvent`, TEB/PEB | Fast-path spinlock is pure; slow-path needs event objects |
| `heap.c` | 4217 | ~40% | `ZwAllocateVirtualMemory`, `ZwFreeVirtualMemory`, PEB, heap locks | Free list search, block split/coalesce are pure; VM backing is not |
| `thread.c` | 352 | ~30% | `ZwAllocateVirtualMemory`, `ZwCreateThread`, `ZwProtectVirtualMemory` | Stack layout is pure; thread creation is syscall |

### Tier B — Unlocked by NLS + String Infrastructure (~5,900 lines)

| File | Lines | Pure % | FFI Needed | Notes |
|------|-------|--------|------------|-------|
| `unicode.c` | 2852 | ~60% | NLS codepage tables (globals), `RtlAllocateHeap` | String comparison/case conversion is pure once NLS tables provided |
| `path.c` | 3040 | ~50% | PEB current directory, `ZwQueryAttributesFile` | Path normalization, `.`/`..` collapsing, device name detection are pure |

### Tier C — Skip / Defer

| File | Lines | Reason |
|------|-------|--------|
| `actctx.c` | 6224 | Custom XML parser + PE resource loading + file I/O + TEB context stack. Enormous, tightly coupled. |
| `registry.c` | 1351 | Pure syscall wrapper (`ZwOpenKey`/`ZwQueryValueKey`). No meaningful algorithm to port. |
| `process.c` | 512 | 90% syscall glue (`ZwCreateProcess`, `ZwCreateSection`). No meaningful algorithm. |

### Kernel FFI Surface (shared by Tier A + B)

```
extern "system" {
    // Virtual Memory (heap.c, thread.c)
    fn ZwAllocateVirtualMemory(process: isize, base: *mut *mut u8, ...) -> i32;
    fn ZwFreeVirtualMemory(process: isize, base: *mut *mut u8, ...) -> i32;
    fn ZwProtectVirtualMemory(process: isize, base: *mut *mut u8, ...) -> i32;

    // Synchronization (critical.c)
    fn NtCreateEvent(handle: *mut isize, ...) -> i32;
    fn NtSetEvent(handle: isize, ...) -> i32;
    fn NtWaitForSingleObject(handle: isize, ...) -> i32;
    fn NtWaitForKeyedEvent(handle: isize, ...) -> i32;
    fn NtReleaseKeyedEvent(handle: isize, ...) -> i32;

    // Execution Context (critical.c, heap.c, path.c)
    fn NtCurrentTeb() -> *mut TEB;
    fn NtCurrentPeb() -> *mut PEB;
}
```

## Metrics

| Phase | Files | Lines of C | Status |
|-------|-------|-----------|--------|
| Done | 19 crates | ~5,030 | Complete |
| Phase 1 | ~4 files | ~3,700 | Next (Batch A+B done, Batch C remains) |
| Phase 2 | 3 files | ~2,650 | Planned |
| Phase 3 | ~7 files | ~3,500 | Planned |
| Phase 4 | ~4 files | ~1,500 | Planned |
| Phase 5 | ~5 files | ~5,000 | Planned |
| Phase 6 | ~8 files | ~17,500 | Future (needs kernel FFI) |
| **Total** | **~52 files** | **~36,000** | |

## Principles

1. **Same ABI** — `#[repr(C)]`, `extern "system"`, identical struct layouts
2. **Same symbols** — `#[no_mangle]`, callers never change
3. **Pure algorithm core** — testable, Kani-verifiable, no `unsafe`
4. **Thin FFI shell** — `unsafe` only at the boundary
5. **All 3 architectures** — every crate builds for arm64, amd64, i386
6. **Zero build failures** — maintained at all times
7. **Smoke tests pass** — all 3 arches boot after every change
