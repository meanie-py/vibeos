> **HISTORICAL — config/ Rust-port debug log (2026-04-30); all bugs resolved, config is fully Rust, i386 boots to desktop. The 'uncommitted diagnostic traces' list below was long since cleaned up.**

# config/ Rust port debug status (2026-04-30 — session 3 final)

## Where we are

Branch `wip-config-rust-port`. **9 commits shipped** advancing boot from
- `BSOD 0x67 (config init step 13)` →
- `BSOD 0x6B (PsLocateSystemDll)` →
- `i386: hangs at mountmgr.sys PnP enum (ASSERT in ExReleasePushLock)` →
- `amd64: hangs at uniata.sys PnP enum` →
- `BSOD 0x6B — cdrom.sys DriverEntry fails because wdf01000.sys
  DriverEntry got 0-length RegistryPath` →
- `BSOD 0x4C — SMSS hive load reads all-zero header` →
- `BSOD 0x1E — strlen(0x30393733) crash at NtBuildLab pointer mistype` →
- **boot stalls in SMSS drive-letter assignment + KCB-lock ASSERTs** —
  no longer BSODs, the kernel runs through DUMMY_PSEH ASSERT prints.

Smoke tests still **not** passing, but the boot now reaches:
- ✅ All 24 boot driver `DriverEntry`s
- ✅ PnP enum complete, all critical drivers bind devices
- ✅ ARC/SystemRoot symlinks resolve to `\Device\CdRom0\reactos`
- ✅ ntdll loads, SMSS reaches user mode
- ✅ SMSS reads SECURITY/SOFTWARE/DEFAULT/SAM hives
- ✅ SmpConfigureSubSystems iterates all subsystem entries
- ✅ Pagefile init, version-data write
- ❌ Drive-letter assignment hangs with KCB-lock recursion ASSERTs
  (`cm_x.h:120 Owner != KeGetCurrentThread()`) and pushlock-not-held
  ASSERTs (`ex.h:1305 OldValue.Locked`).

Both i386 and amd64 reach this same point.

## Fixes shipped (session 1, prior)

1. `cmvalche.rs:144` — bindgen flexible-array indexing panic. (`a4c169b6`)
2. `cmconfig.rs` — `CmpMultifunctionTypes` zero-length array indexing. (`a4c169b6`)
3. `cmparse.rs::CmpDoOpen` — wrong `STATUS_REPARSE` constant. (`a4c169b6`)
4. `cmapi.rs::CmEnumerateKey` — registry-lock recursion leak (Quickie pattern). (`e3c3cd4f`)
5. `fstub`/`kdbg` — pool tag big-endian + STATUS_NO_MEDIA constant. (`d0efbaf8`)

## Fixes shipped (session 2, this commit)

**`cmapi.rs` — five wrong NTSTATUS hex literals from rustconv translation.**

The LLM that translated C → Rust transcribed status constants as raw hex
literals and got several WRONG (digits transposed or wrong constant
identity). All occurred in `CmpQueryKeyData` and related registry
read/write paths:

| Site | Wrong value | Correct C constant | Correct value |
|------|-------------|-------------------|----------------|
| `cmapi.rs:454` (`CmpQueryKeyData KeyBasicInformation`) | `0x8000000D` STATUS_PARTIAL_COPY | STATUS_BUFFER_TOO_SMALL | `0xC0000023` |
| `cmapi.rs:493` (`CmpQueryKeyData KeyNodeInformation`) | `0x8000000D` | STATUS_BUFFER_TOO_SMALL | `0xC0000023` |
| `cmapi.rs:564` (`CmpQueryKeyData KeyFullInformation`) | `0x8000000D` | STATUS_BUFFER_TOO_SMALL | `0xC0000023` |
| `cmapi.rs:248` (`CmpSetValueKeyNew` HvMarkCellDirty fail) | `0xC00001D5` (undefined) | STATUS_NO_LOG_SPACE | `0xC000017D` |
| `cmapi.rs:750` (`CmSetValueKey` Quickie HvMarkCellDirty fail) | `0xC00001D5` (undefined) | STATUS_NO_LOG_SPACE | `0xC000017D` |
| `cmapi.rs:1079` (`CmEnumerateValueKey` deleted check) | `0xC0000017` STATUS_NO_MEMORY | STATUS_KEY_DELETED | `0xC000017C` |
| `cmapi.rs:1560` (`CmFlushKey` HvSyncHive fail) | `0xC000010C` STATUS_NO_GUID_TRANSLATION | STATUS_REGISTRY_IO_FAILED | `0xC000014D` |

### How the bugs blocked PnP enumeration

The critical one was `CmpQueryKeyData`'s wrong `STATUS_BUFFER_TOO_SMALL`.
`ZwEnumerateKey(handle, idx, KeyBasicInformation, NULL, 0, &len)` is the
canonical "size query" pattern: caller passes 0-byte buffer, expects
`STATUS_BUFFER_TOO_SMALL` (0xC0000023) plus the required size. Then
caller allocates and re-queries.

With the wrong return value `STATUS_PARTIAL_COPY` (0x8000000D), all
callers hitting this path in PnP — specifically `IopInstallCriticalDevice`
in `pnpmgr.rs:386-598` enumerating the CriticalDeviceDatabase — fall
through to a `Index += 1; continue;` loop that never matches. Result: no
device ever finds its CriticalDeviceDatabase entry → no Service value
written to its Enum subkey → `PiCallDriverAddDevice` later returns
`STATUS_OBJECT_NAME_NOT_FOUND` from `RtlQueryRegistryValues(Service)` →
no driver bound → no AddDevice → no child PDOs enumerated.

This cascades all the way down: ROOT not bound to PCI → no PCI children
→ no IDE controller → no CDROM PDO → cdrom.sys's AddDevice never called
→ `IoGetConfigurationInformation()->CdRomCount = 0` → ARC name not
created → `\SystemRoot` doesn't resolve → ntdll.dll lookup fails →
BUGCHECK 0x6B.

### After these fixes — boot timeline

**i386:**
- All registry init ✓
- PnP root device tree creation ✓
- All 24 boot driver `DriverEntry` calls ✓
- `IopInstallCriticalDevice` finds matches in CriticalDeviceDatabase ✓
- `PiCallDriverAddDevice` calls `AddDevice` for PCI bus, ISAPNP bridge,
  partmgr, mountmgr, etc. ✓
- **NEW BLOCKER**: during `mountmgr.sys` PnP enum, an `ExReleasePushLock`
  is called when the lock isn't held → `ASSERT FAILED: OldValue.Locked`
  at `ex.h:1305`. Boot hangs (silently — confirmed by patching out the
  assert: hang persists).

**amd64:**
- Same progress through mountmgr ✓
- Resource conflict warnings (IRQ 0x9 0x9, ACPI bus number) — non-fatal
- `uniata.sys` `DriverEntry` ✓
- During PnP enum: `WARNING: PCI Slot Resource Assignment is FOOBAR`
  (`pcibus.c:816`)
- Then same `ASSERT FAILED: OldValue.Locked` → hang.

## Pushlock release without acquire — **FIXED** (commit `c2b7d938`)

The mountmgr/uniata pushlock-not-held ASSERT at `ex.h:1305` traced to
`EnlistKeyBodyWithKCB` calling `CmpReleaseKcbLock` on the SHARED-flag
branch when the caller had passed *EXCLUSIVE*. Two LLM-translation
typos in `cmparse.rs` had inverted the values of
`CMP_ENLIST_KCB_LOCKED_SHARED` (0x1) and `CMP_ENLIST_KCB_LOCKED_EXCLUSIVE`
(0x2):

| Site | Bug | Fix |
|------|-----|-----|
| `cmparse.rs:299` (`CmpDoCreateChild` local const) | `CMP_ENLIST_KCB_LOCKED_EXCLUSIVE = 0x1` | `= 0x2` |
| `cmparse.rs:771` (`CmpDoOpen` ternary) | `IsLockShared ? 0x2 : 0x1` (inverted) | `IsLockShared ? 0x1 : 0x2` |

Diagnosis path: instrumented `ExReleasePushLock` to print
`__builtin_return_address(0)` when `OldValue.Locked == 0`. Caller PC
0x8047D404 symbolicated to `EnlistKeyBodyWithKCB+0x54`, which is the
return site after the `if (Flags & CMP_ENLIST_KCB_LOCKED_SHARED)`
release path. The C source matches the Rust source semantically; the
mismatch was in the constant values themselves.

After the fix, all 24 boot drivers complete `IopInitializeBuiltinDriver`,
PnP enum runs without ASSERTs, and boot reaches
`IopInitializeSystemDrivers` / `IopReassignSystemRoot`.

## cdrom.sys DriverEntry → wdf01000.sys path bug — **FIXED** (commit `09b47bb4`)

Root cause was a **truncated translation** of `CmpQueryNameInformation`
in `cmapi.rs:1255`. Sonnet generated the size-query / overflow check
prologue and stopped, never emitting the second pass that copies the
parent KCB chain into the output buffer. Result: every
`ZwQueryKey(KeyHandle, KeyNameInformation, ...)` returned a buffer with
`NameLength=0` and an uninitialized Name array.

Most drivers ignore `RegistryPath` in `DriverEntry` and were unaffected.
`wdf01000.sys` was the first driver to actually use the path — its
`DriverEntry` calls `WdfRegisterLibrary -> LibraryCreate -> GetImageName`
which uses `RegistryPath` as input to `ZwOpenKey`, and a 0-length
`UNICODE_STRING` produced `STATUS_OBJECT_PATH_SYNTAX_BAD` (0xC000003B).
The driver object cleanup unlinked wdf01000 from PsLoadedModuleList, so
cdrom.sys's later `WdfVersionBind` couldn't find it via the module list,
fell through to `MmLoadSystemImage`'s ZwOpenFile path, and failed
because `\SystemRoot` wasn't yet resolvable. The chain cascaded all the
way to BUGCHECK 0x6B.

Fix: ported the missing copy loop from `cmapi.c:1560-1601`, with
`CmpCopyCompressedName` for compressed names and `core::ptr::copy_nonoverlapping`
for raw names; appends `\\` between segments and writes the final
`NameLength` to the caller's structure.

Diagnostic path: instrumented `IopInitializeDriverModule` to print the
`RegistryPath` being passed to each `DriverInit`. Output was
`Length=0 Buffer=...` for every driver, immediately revealing the bug
class (every ZwQueryKey for KeyNameInformation was broken).

After the fix, boot now reaches **user mode** — ntdll loads, SMSS
starts, parses its command line, and tries to load registry hives.

## BUGCHECK 0x4C in SMSS hive load — **FIXED** (commit `7b371043`)

The Rust translation passed `0x00000002` (FILE_WRITE_THROUGH) to
`ZwCreateFile` in `CmpOpenHiveFiles` where the C source uses
`FILE_SYNCHRONOUS_IO_NONALERT` (`0x00000020`). Without the synchronous
flag, `ZwReadFile` returned `STATUS_PENDING` and CmpFileRead's
`NT_SUCCESS(STATUS_PENDING) == TRUE` check returned without waiting,
leaving the buffer all-zero — which `HvpVerifyHiveHeader` then
rejected.

Diagnostic path: traced `CmpFileRead` to see Status=0x103
(STATUS_PENDING), info=0, first8=0 — instantly identifying the
async-vs-sync IO bug. Fixed both ZwCreateFile call sites
(cminit.rs:423 and :583).

After this fix, SMSS hive loads succeed, all subsystem config is
read, and the boot proceeds into the next stage where it hits
`BUGCHECK 0x1E` from `strlen(NtBuildLab)`.

## strlen on bad NtBuildLab pointer — **FIXED** (commit `c5d7debb`)

`CmpSetVersionData` calls `RtlInitAnsiString(&TempString, NtBuildLab)`
to convert the build-lab string to a UNICODE_STRING. The C source
declares `extern CHAR NtBuildLab[];` (an array — name decays to
pointer). The Rust translation declared
`pub static mut NtBuildLab: *const i8;` (a pointer — Rust loads the
first 4 bytes of NtBuildLab as the pointer value).

The compiler then passed `0x30393733` (ASCII "0973" — the first 4
bytes of the build version string) as the SourceString, leading to
`strlen` accessing a non-canonical kernel address and BUGCHECK 0x1E.

Diagnostic path: instrumented `RtlInitAnsiString` to print
SourceString and `__builtin_return_address(0)`. Output showed
`SourceString=30393733 caller=804492F2`. Symbolicated to
`CmpSetVersionData+0x262` and the disassembly clearly showed
`pushl $0x30393733` as an immediate — confirming the array-vs-pointer
type mistake.

Fix: declare `pub static NtBuildLab: i8;` (matching the C-array
semantics — we just need the symbol's address) and pass
`core::ptr::addr_of!(NtBuildLab) as *const i8`.

## KCB-lock recursion + pushlock-not-held ASSERTs — **FIXED** (commit pending)

The two ASSERTs were both downstream symptoms of one root cause: the
LLM that translated `cmparse.rs` produced a stub `COMPUTE_HASH_CHAR`
that omitted the `RtlUpcaseUnicodeChar` call required by the C macro
at `cm_x.h:31-32`:

```c
#define COMPUTE_HASH_CHAR(ConvKey, Char)                            \
    (37 * ConvKey + RtlUpcaseUnicodeChar(Char))
```

The buggy Rust:

```rust
unsafe fn COMPUTE_HASH_CHAR(key: ULONG, ch: WCHAR) -> ULONG {
    key.wrapping_mul(37).wrapping_add(ch as ULONG)   // <-- missing upcase
}
```

`cmkcbncb.rs` got the same macro right (uses `RtlUpcaseUnicodeChar`).
The two implementations therefore disagreed for every name with at
least one ASCII letter that case-folded — i.e. essentially every
registry path. `CmpComputeHashValue` (in `cmparse.rs`) populated the
HashCacheStack with the lower-case-derived ConvKeys, so
`CmpBuildAndLockKcbArray` locked the WRONG hash indices.
`CmpCreateKeyControlBlock` (in `cmkcbncb.rs`) then derived the CORRECT
upper-case ConvKey, hashed it to the right index, found that nothing
held that index's lock, and the `CMP_LOCK_HASHES_FOR_KCB` contract
violation cascaded:

1. `CmpReferenceKeyControlBlock(FoundKcb)` saw `RefCount==0` (the
   delayed-close case) and `Owner != current` (because the wrong
   index was the one we'd locked, not this one).
2. Fell through to `CmpConvertKcbSharedToExclusive`, which
   `CmpReleaseKcbLock`s a lock no one holds → `ex.h:1305`
   `OldValue.Locked` ASSERT.
3. With DUMMY_PSEH the ASSERT just prints and returns. The release
   was a no-op because the lock was already 0; the subsequent
   `CmpAcquireKcbLockExclusive` then succeeded and *did* set
   `Owner = current`.
4. Later, `CmpDereferenceKeyControlBlock` for that same KCB called
   `CmpAcquireKcbLockExclusive` again → Owner already us → `cm_x.h:120`
   ASSERT.

Diagnostic path: instrumented both ASSERTs to print
`__builtin_return_address(0/1)` to find the call sites
(`CmpReferenceKeyControlBlock+0x53` from `CmpCreateKeyControlBlock`,
`CmpDereferenceKeyControlBlock` from `CmpParseKey`). Then instrumented
`CmpReferenceKeyControlBlock`'s convert path to dump the lock value
and Owner pointer at the moment of entry — `lock_val=00000000
owner=00000000` confirmed nobody held the lock. Adding `[CrtKcb]` to
`CmpCreateKeyControlBlock` showed `Flags=2` (CMP_LOCK_HASHES_FOR_KCB)
and `RtlGetCallersAddress` resolved the chain to
`ObpLookupObjectName -> CmpParseKey -> CmpCreateKeyControlBlock`.
Comparing the two `COMPUTE_HASH_CHAR` definitions (`cmparse.rs:65`
versus `cmkcbncb.rs:89`) made the disagreement obvious.

Fix: align `cmparse.rs::COMPUTE_HASH_CHAR` with `cmkcbncb.rs` and the
C macro, calling `ntkrnl_sys::ddk::RtlUpcaseUnicodeChar` on the input
character.

After this one-line fix, **i386 boots to the desktop and all 7 i386
smoke tests pass**. amd64 also reaches `shell ready` with this fix in
place; its `[EXPLORER] _tWinMain entered` milestone failure is the
pre-existing taskbar regression documented in MEMORY.md (not related
to config/). ARM64's pre-existing baseline failure also stands.

## Diagnostic state currently in tree (uncommitted)

The following diagnostic traces were committed in `52be9e92` for the
prior debugging session and are still in the tree. They should be
removed once the next blocker is resolved:

- `iomgr/iomgr.rs` — `io_trace()` + `[IO]` traces.
- `iomgr/driver.rs` — `[BLT]` per-builtin-driver trace (the leading
  `[BLT2]` traces I added this session were reverted before commit).
- `iomgr/arcname.rs` — `[CDARC]` and `[REASSIGN]`.
- `pnpmgr/pnpinit.rs` — `[PNP]`.
- `config/cmsysini.rs` — `[CMGSDL]`, `[LOCK]`/`[UNLK]`.

The `[BLT]` per-driver trace and `[PNP]` traces remain useful for the
next blocker investigation; consider keeping them.

## Memory notes consulted

- `feedback_translation_philosophy.md` — never revert .rs to C; never
  delete .rs without approval.
- `feedback_flexible_array.md` — bindgen flexible arrays must use pointer
  arithmetic (still relevant, no new bugs in this session).
- `feedback_no_blind_revert.md` — don't `git checkout` to discard work.
