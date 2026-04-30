# config/ Rust port debug status (2026-04-29)

## Where we are

On branch `wip-config-rust-port` at `2fe69a64` (Codex opaque_ptr sweep) +
this doc commit. Build is clean both arches, 0 unexpected ud2. Boot still
fails on i386, but the failure mode is no longer the LLVM captures(none)
ud2 cascade — it's a real translation issue plus an unstable CGU class.

## Bisection result

The smoke test bugcheck is `0x67 CONFIG_INITIALIZATION_FAILED
(0x1, 0xD, 0xC000003B=STATUS_OBJECT_PATH_SYNTAX_BAD, 0x0)`, which
corresponds to `cmsysini.rs:2257` — the bugcheck site after
`CmpInitializeHardwareConfiguration` returns failure.

By temporarily routing `ARM64_UART_TRACE` through `DbgPrint` (it's a
no-op on x86 in the current code) and adding traces inside
`CmpInitializeHardwareConfiguration` and `CmpInitializeRegistryNode` in
`cmconfig.rs`, the failure was bisected to:

- Both top-level `NtCreateKey` calls in `CmpInitializeHardwareConfiguration`
  (DeviceMap, Description) succeed.
- `CmpSetupConfigurationTree` enters; first iteration runs.
- Inside `CmpInitializeRegistryNode` first iter (SystemClass node):
  - `NtCreateKey` for the type key (`\Registry\Machine\Hardware\Description\System`) — OK
  - `NtSetValueKey` for `Component Information` — OK
  - `RtlAnsiStringToUnicodeString` for the identifier path — OK (NtSetValueKey for `Identifier` succeeds)
  - Configuration Data prep: `have data` → `copying` → `copied` → `writing IT/BN` → `IT/BN done` — all OK
  - **`NtSetValueKey` for `Configuration Data` is the failing call** — never returns either OK or FAILED trace.

## Two suspects

### 1. NtSetValueKey for `Configuration Data` (most likely real bug)

`NtSetValueKey` is in `config/ntapi.rs:895`. The Rust translation has
some structural differences from the C in `ntoskrnl/config/ntapi.c:859`,
notably:

- The `if !DataSize: Data = NULL` is moved to before the user-mode probe
  block (cosmetic — same effect for kernel-mode caller).
- The user-mode probe block contains an obviously wrong inner pattern:
  ```rust
  if NT_SUCCESS(Status) {     // always true here
      DataCopy = ExAllocatePoolWithTag(...);
      ...
      core::ptr::copy_nonoverlapping(Data, DataCopy, ...);
      if !NT_SUCCESS(Status) { ... }   // can never fire
      Data = DataCopy;
  } else {
      ObDereferenceObject(KeyObject as PVOID);
      return Status;
  }
  ```
  This is dead-code-only because PreviousMode is `KernelMode (0)` for
  the failing in-kernel call, so the whole `if PreviousMode != 0i8 && DataSize != 0`
  block is skipped. Not the bug.

- `ProbeAndCaptureUnicodeString` is called unconditionally (correct).
- The trailing-NUL strip loop, length validation, ReadOnly check, and
  PreSet/CmSetValueKey/PostSet sequence look structurally faithful to C.

The most likely bug isn't in `NtSetValueKey` proper — it's downstream
in `CmSetValueKey` or one of the registry-write callees. That's where
the audit needs to go next.

### 2. CGU partitioning instability (also present)

While bisecting, adding `ARM64_UART_TRACE` calls (which become real
`DbgPrint` calls in the temporary debug build) reshuffles LLVM's CGU
partitioning enough that the bug's exact symptom shifts:

- Without traces: BSOD `0x67 (1, 13, 0xC000003B, 0)` ~30s in.
- With traces past `Configuration Data`: hang at the trace line, no BSOD.

This is the same captures(none) class documented in `feedback_cgu1.md`.
The Codex opaque_ptr sweep mitigated it for the bulk of the kernel (0
unexpected ud2 on both arches), but the config/ paths going through
`CmSetValueKey` and into hive/cell code are still partition-sensitive.
Some additional `opaque_ptr` barriers, or moving particular hot
functions to `#[inline(never)]`, may be required.

## Next steps

1. Audit `CmSetValueKey` (in `cmkcbncb.rs` / `cmvalche.rs`) and the
   value-write path for the Configuration Data scenario specifically.
   The faulting input has Type=`REG_FULL_RESOURCE_DESCRIPTOR` and
   ~16-byte Data. Compare line-by-line against the C:
   - `ntoskrnl/config/cmkcbncb.c::CmSetValueKey`
   - `ntoskrnl/config/cmvalche.c` (value cell allocation and writing)
   - `sdk/lib/cmlib/cmindex.c` and `cmvalue.c` (cmlib helpers)

2. Once a candidate translation bug is found, before claiming the fix
   works also verify with another `opaque_ptr` sweep across the
   value-write path, since the captures(none) class is still active in
   this code.

3. To make tracing reliable (without changing CGU layout per build),
   route `ARM64_UART_TRACE` through `DbgPrint` permanently for x86 in
   one place (`config/mod.rs`) instead of editing each file. Then the
   `[CFG]` traces become a stable diagnostic surface.

## Memory notes consulted

- `feedback_cgu1.md` — LLVM captures(none) regression, CGU=256 + opaque_ptr workaround.
- `feedback_extern_c_abi.md` — extern "C" vs "system" ABI; not the immediate cause this time but a perennial cm_rust_shims.c concern.
- `feedback_translation_philosophy.md` — Rust is the goal; never revert .rs to C; never delete .rs without approval.
