# ACPICA: FFI-to-blob plan (keep Intel's lib as C, translate the glue to Rust)

> **STATUS: COMPLETE (2026-06-09).** All four phases done. The entire ReactOS
> ACPI driver glue (~9.9K LOC) is now Rust in `sdk/lib/rust/acpi-rust/`; Intel's
> ACPICA (`libacpica.a`, 88.8K LOC) is untouched C. The only remaining glue C is
> `acpi_printf_shim.c` — the **designed** C-variadic trampoline for `AcpiOsPrintf`
> (Rust cannot *define* a variadic `extern "C"` fn; it forwards to the Rust
> `AcpiOsVprintf`). Validated green ×3 (i386/amd64/arm64): cargo check + cohab
> link clean, `scan_ud2` 0 unexpected, `lint_kernel` clean, smoke boots to
> desktop. Commits: Phase 1 `f0cf39403e7`, Phase 2 `e312291b198`, Phase 3a
> `44696affc54`+`32cd8e730f1`, Phase 3b `20b051f7ee6`.

**Decision (user, 2026-06-09):** Do **not** translate Intel's ACPICA reference
implementation. Keep it as a C static library and FFI into it from Rust as
needed. Only the ReactOS-specific ACPI driver glue becomes Rust. (This is the L3
"FFI-to-blob" axis from the translation-banking roadmap.)

## Why this is clean already

`drivers/bus/acpi/CMakeLists.txt` already splits the component into two targets:

- **`acpica` static lib** — Intel's reference impl, **88,824 LOC** in upstream
  layout (`dispatcher/ events/ executer/ hardware/ namespace/ parser/
  resources/ tables/ utilities/`). Built with `ACPI_USE_LOCAL_CACHE;
  ACPI_USE_SYSTEM_INTTYPES`. **This stays C, untouched.**
- **`acpi` MODULE** — the ReactOS driver glue, **~9,984 LOC**, which
  `target_link_libraries(acpi … acpica …)`. **This is the Rust translate target.**

So the C-lib/Rust-glue boundary is the existing `acpica`↔`acpi` link edge. No
restructuring of the blob is required — we cohab-flip the `acpi` MODULE the same
way we did the other `.sys` drivers, and it keeps linking `libacpica.a`.

## The bidirectional FFI surface

ACPICA is designed around an "OS Services Layer" (OSL) precisely so it can be
hosted on any OS. That gives us two well-defined, header-pinned FFI surfaces:

### Inbound — ACPICA calls into us (the OSL): 46 `AcpiOs*` callbacks
Implemented today in `osl.c` (1273 LOC). In Rust these become
`#[no_mangle] extern "C"` fns; ACPICA (C) calls them. Signatures are fixed by
`acpica/include/acpiosxf.h`. The set:

```
AcpiOsInitialize/Terminate, AcpiOsGetRootPointer,
AcpiOsPredefinedOverride/TableOverride/PhysicalTableOverride,
AcpiOsMapMemory/UnmapMemory/GetPhysicalAddress,
AcpiOsAllocate/Free, AcpiOsReadable/Writable,
AcpiOsGetThreadId/Execute/Sleep/Stall/WaitEventsComplete,
AcpiOsCreateMutex/DeleteMutex/AcquireMutex/ReleaseMutex,
AcpiOsCreateSemaphore/DeleteSemaphore/WaitSemaphore/SignalSemaphore,
AcpiOsCreateLock/DeleteLock/AcquireLock/ReleaseLock,
AcpiOsReadPort/WritePort, AcpiOsReadMemory/WriteMemory,
AcpiOsReadPciConfiguration/WritePciConfiguration,
AcpiOsInstallInterruptHandler/RemoveInterruptHandler,
AcpiOsGetTimer/Signal/EnterSleep/GetLine,
AcpiOsPrintf/Vprintf/RedirectOutput/Breakpoint
```
`AcpiOsPrintf`/`AcpiOsVprintf` are C-variadic → keep a thin C-variadic shim
(same pattern as `dbgprint_variadic.c` / ntoskrnl `variadic_shims.c`), Rust does
the rest.

### Outbound — we call into ACPICA: 57 public `Acpi*` API fns
Used by the driver glue (main/pnp/power/buspdo/eval/interface/acpienum +
busmgr/*). Declared from `acpica/include/acpixf.h` (e.g. `AcpiInitializeSubsystem`,
`AcpiEnableSubsystem`, `AcpiInitializeObjects`, `AcpiLoadTables`,
`AcpiGetHandle`, `AcpiEvaluateObject`, `AcpiWalkNamespace`, `AcpiGetTable`,
`AcpiGetCurrentResources`, `AcpiInstallNotifyHandler`, …).

## Build approach (the substrate)

Mirror the `lsass-sys`/`cmlib` pattern: a focused **`acpica-sys` bindgen crate**
generated from a wrapper that pulls ACPICA's public headers
(`acpi.h` → `acpixf.h` + `actypes.h` + `acpiosxf.h` + `acexcep.h`) with
`-DACPI_USE_SYSTEM_INTTYPES -DACPI_USE_LOCAL_CACHE` and the same `-D_X86_/_AMD64_/
_ARM64_` per-arch defines. It provides, per arch:
- the ACPI types (`ACPI_STATUS`, `ACPI_HANDLE`, `ACPI_OBJECT`, `ACPI_TABLE_*`,
  `ACPI_BUFFER`, `ACPI_DEVICE_INFO`, `ACPI_RESOURCE`, the GPE/event types …),
- `extern "C"` decls for the 57 public `Acpi*` fns (outbound), and
- the exact callback typedefs so the Rust OSL impls (`#[no_mangle]`) match.

`acpica-sys` allowlist: `(ACPI_.*|Acpi.*)`. NDK/ntoskrnl context for the kernel
types the OSL touches (it calls `MmMapIoSpace`, `KeAcquireSpinLock`, `HalGetBusData`,
`ExAllocatePool`, etc. — those come from ntkrnl-sys, cast at the boundary).

## Phasing

1. **Substrate** — build `acpica-sys` (bindgen acpixf.h ×3 arches). No driver
   change yet; just `cargo check` that the decls compile.
2. **OSL first** — translate `osl.c` → `acpi-rust` (the 46 `AcpiOs*` impls +
   the variadic shim). This is the load-bearing, well-specified part. Cohab-flip:
   `acpi` MODULE links `libacpi_rust.a` providing the `AcpiOs*` symbols; ACPICA
   resolves them. Validate ACPICA still initializes.
3. **Driver glue** — main/pnp/power/buspdo/eval/interface/acpienum (~5K) then
   busmgr/* (~3.9K). rustconv bulk for the larger files (buspdo 2077, bus 1826)
   — NEEDS BUDGET; osl.c + small glue are hand-translatable.
4. **Validate** — acpi.sys is loaded by halacpi (PnP root enumeration). Confirm
   it's exercised in the QEMU smoke path (HAL flavor dependent); if not at the
   `shell ready` marker, link-clean ×3 is the bar, plus a targeted ACPI-init
   serial check.

## Cohab mechanics
Standard `.sys` MODULE cohab (like pl011/processr): `acpi` becomes
`add_library(acpi MODULE rust_cohab_stub.c)` + `set_module_type kernelmodedriver`
+ `target_link_libraries(acpi … acpica libacpi_rust.a …)`. **`acpica` C lib is
untouched.** Per-arch RUST_TARGET; i386 `-sse,-sse2`.

## Notes
- `acpi_new/` (uses the community **uACPI** lib, not Intel's) is unbuilt and out
  of scope per the "use Intel's lib" decision.
- ACPICA upstream is ANSI C with its own cache/inttypes shims already wired for
  ReactOS — no libc surprises; the OSL is the only OS coupling.
