# config/ Rust port debug status (2026-04-30 — session 3)

## Where we are

Branch `wip-config-rust-port`. **6 commits shipped** advancing boot from
- `BSOD 0x67 (config init step 13)` →
- `BSOD 0x6B (PsLocateSystemDll: ntdll.dll path not found)` →
- `i386: hangs at mountmgr.sys PnP enum (ASSERT in ExReleasePushLock)` →
- `amd64: hangs at uniata.sys PnP enum (same pushlock ASSERT)` →
- **i386 + amd64: BSOD 0x6B — cdrom.sys DriverEntry fails because wdf01000.sys
  was unloaded after its own DriverEntry returned STATUS_OBJECT_PATH_SYNTAX_BAD**

Smoke tests still **not** passing, but the kernel now runs **all 24 boot
driver DriverEntries**, completes PnP enumeration without hangs, and reaches
`IopReassignSystemRoot` / `PsLocateSystemDll`. The remaining gap is a single
WDF dependency chain.

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

## Open: cdrom.sys DriverEntry fails → BSOD 0x6B

After the fix above, the next blocker is:

- `wdf01000.sys` is loaded as a builtin and present in PsLoadedModuleList.
- Its `DriverEntry` (KMDF library DriverEntry in
  `sdk/lib/drivers/wdf/kmdf/src/dynamic/version/version.cpp:318`) returns
  `STATUS_OBJECT_PATH_SYNTAX_BAD` (0xC000003B) — likely from
  `FxLibraryCreateDevice` or `WdfRegisterLibrary`.
- `IopInitializeDriverModule` treats this as failure, calls
  `ObMakeTemporaryObject` + `ObDereferenceObject` on the driver object,
  and the LDR module entry gets unlinked from `PsLoadedModuleList`.
- Subsequently `cdrom.sys`'s DriverEntry calls `WdfVersionBind`
  (via `wdfdriverentry.c:FxDriverEntry`), which loads
  `\SystemRoot\system32\drivers\wdf01000.sys` via `MmLoadSystemImage`.
  The path-scan loop misses it (because `wdfldr.sys` is in the list with
  ARC-path naming, not `wdf01000.sys`), falls through to `ZwOpenFile`,
  which returns `STATUS_OBJECT_PATH_NOT_FOUND` because `\SystemRoot`
  doesn't resolve yet.
- `cdrom.sys` DriverEntry returns failure → no AddDevice → CdRomCount=0
  → `IopCreateArcNames` doesn't create `\ArcName\multi(0)disk(0)cdrom(96)`
  → `\SystemRoot` symlink resolution fails → `PsLocateSystemDll` fails
  with `STATUS_OBJECT_PATH_NOT_FOUND` → BUGCHECK 0x6B.

**Two paths forward:**

1. Fix the actual reason `wdf01000.sys`'s DriverEntry returns
   `STATUS_OBJECT_PATH_SYNTAX_BAD`. Add tracing to `FxLibraryCreateDevice`
   / `WdfRegisterLibrary` to identify which UNICODE_STRING is malformed.
2. Modify `IopInitializeDriverModule` to NOT unload the LDR module entry
   when DriverEntry fails (only drop the driver object). This would let
   cdrom.sys find wdf01000.sys later. This is closer to NT behavior but
   diverges from the existing `ObDereferenceObject` semantics.

This blocker is **pre-existing** — it was always there, but masked by
the earlier ASSERT hang. With the ASSERT fixed, it now surfaces.

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
