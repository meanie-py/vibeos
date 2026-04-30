# config/ Rust port debug status (2026-04-30 — session 3)

## Where we are

Branch `wip-config-rust-port`. **7 commits shipped** advancing boot from
- `BSOD 0x67 (config init step 13)` →
- `BSOD 0x6B (PsLocateSystemDll: ntdll.dll path not found)` →
- `i386: hangs at mountmgr.sys PnP enum (ASSERT in ExReleasePushLock)` →
- `amd64: hangs at uniata.sys PnP enum (same pushlock ASSERT)` →
- `BSOD 0x6B — cdrom.sys DriverEntry fails because wdf01000.sys's
  DriverEntry got a 0-length RegistryPath` →
- **BSOD 0x4C (FATAL_UNHANDLED_HARD_ERROR / STATUS_REGISTRY_CORRUPT)
  during SMSS user-mode hive load**

Smoke tests still **not** passing, but the kernel now runs **all 24 boot
driver DriverEntries**, completes PnP enumeration, creates ARC/SystemRoot
symlinks, and reaches **user mode** — ntdll loads, SMSS starts, parses
its config. The remaining failure is a registry hive whose on-disk header
reads as all zeros (signature 0x0 expected 0x66676572 'regf').

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

## Open: BUGCHECK 0x4C in SMSS hive load

```
(\sdk\lib\cmlib\hiveinit.c:62) Verify Hive Header failed:
    Signature: 0x0, expected 0x66676572; Major: 0x0, expected 0x1
    Minor: 0x0 expected to be >= 0x3; Type: 0x0, expected 0x0
    Format: 0x0, expected 0x1; Cluster: 0x0, expected 1
    Sequence: 0x0, expected 0x0; Checksum: 0x1, expected 0x0
The hive base header block needs to be RECOVERED
The hive has no log for header recovery
Registry hive couldn't be initialized, it's corrupt (hive 0xB11C7008)
*** BUGCHECK: 0x0000004C (0xC0000218, 0xF752AB94, 0x00000000, 0x00000000)
```

Hive header is all zeroes. This is **not** a config/cmapi.rs class of
bug — it's either a registry IO bug (ZwReadFile returning zero-filled
buffer instead of file content) or a livecd image issue (SAM /
SECURITY / DEFAULT hives missing or empty on the CD).

Likely investigation directions:

1. Identify which hive SMSS is loading via `CmLoadKey` (probably SAM,
   SECURITY, or a per-user hive).
2. Check whether the file exists in the livecd image and is non-empty.
3. If the file exists, instrument `CmpInitHiveFromFile` /
   `CmpFileRead` to confirm the data being read.
4. If the read is correct but verification fails, check
   `HvpVerifyHiveHeader` for translation issues (signature constant?).

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
