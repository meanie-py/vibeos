# IO Manager — Disabled Rust Translations

Rust translations that exist on disk but aren't wired into the build, with
the diagnosed cause for each. **Re-probe with current toolchain before
treating any entry as a settled answer** — many siblings of these have
turned out to be tractable on re-test (most recently: 10 io captures(none)
workarounds, the `_c`-suffixed PnP shim family, `IopInitializeDriverModule`).

## Pattern

Every "Rust crashes / hangs" excuse documented here predates the workspace
`lto = false` + `codegen-units = 256` firewall, the `opaque_ptr` laundering
discipline, and `verify_abi.py`. Several siblings of these files were
re-probed recently and turned out to either (a) build clean with zero
source changes, or (b) have one underlying real bug (often an ABI
mismatch) that was being hidden by a C shim. Don't trust the historical
"category: translation bug" classifications below — they're best-guesses
from when the file was parked, not current diagnoses.

## Files

### `iomgr.rs` / `device.rs` — ENABLED

Both ship on all 3 arches. Smoke 7/7 on i386 + amd64; arm64 ntoskrnl
builds clean. The "LLVM captures(none) success-path elimination" pattern
that originally parked `device.rs` was fully resolved 2026-05-06 — see
shim-elimination doc for the chain of evidence.

### `irp.rs` (irp.c — 2027 lines)

Hangs on amd64 at last test. Three translation bugs were found and fixed
in the source (NT_ERROR macro defined as `status < 0` instead of
`(status >> 30) == 3`; KTHREAD offset error in KeInitializeApc;
PacketType constant wrong). `IopCompleteRequest` (~300 lines) likely has
more bugs. Worth a fresh re-probe given how many sibling files turned
out to need only ABI fixes. Status: parked, not actively investigated.

### `file.rs` (file.c — 4202 lines)

Crashes on at least one arch. Last investigated months ago. Large file
with complex SEH patterns and user-mode pointer validation. Could be a
real translation bug, could be ABI / captures(none) / latent flexible-array
indexing — hasn't been bisected with current tooling.

### `iofunc.rs` (iofunc.c — 4646 lines)

SMSS crash. Last investigated months ago. Largest IO file. Same
caveats as `file.rs`.

### `ramdisk.rs` (ramdisk.c — 307 lines)

Bugcheck 0x6B on ARM64; works on i386/amd64 (where it's effectively
never called). Two translation bugs already fixed
(`InitializeObjectAttributes`, `sprintf` format string null). The
remaining issue could be PE layout sensitivity (the historical hypothesis)
or could be something the pattern-suite would catch on re-probe.

### `driver.rs` (driver.c — 2314 lines)

Mostly Rust now (the file is huge and has many functions). One known
remaining piece: ARM64 Phase0 crash. amd64 + i386 work. Recent additions
include the `IopInitializeDriverModule` translation (2026-05-06).

### `arcname.rs` (arcname.c — 994 lines)

Two functions are Rust, two are C. Crashes on i386 if all 4 go Rust.
Historically attributed to "i386 PE section layout sensitivity" — but
that was the captures(none) era; worth re-probing.

### `devaction.rs` (devaction.c — 2750 lines)

Stays in C as a whole. Multiple historical attempts (delegating exported
fns to C shims, etc.) didn't fix it because C shims call back into Rust
internal functions. The "deep bugs in internal static function graph"
diagnosis is plausible but pre-dates the current debugging tools.

## Common bug patterns we've seen

These are the *families* of bugs that have shown up across the IO
translations. Note none of them are "LLVM is broken" — that excuse retired
in 2026-05.

1. **NT_ERROR vs NT_SUCCESS.** `NT_ERROR(x)` is `(x >> 30) == 3`, NOT
   `x < 0` (which is `!NT_SUCCESS(x)` and includes warnings).
2. **Bindgen anonymous union offsets.** `__bindgen_anon_1` can point to
   the wrong union member. KTHREAD::ApcState lives inside an anon union
   on every arch.
3. **Wrong enum constants.** Match arms using sequential ordinals instead
   of actual enum values (DeviceProperty, KeyValueInformationClass).
4. **Bindgen flexible-array indexing.** Bindgen renders trailing
   `Foo[]` as `Foo: [T; 1]`; indexing past 0 panics on bounds-check.
   Take the array base via `addr_of!` once, use `.add(i)` from there.
5. **`extern "C"` vs `extern "system"` ABI mismatch.** On i386, NTAPI
   functions are stdcall; declaring them `extern "C"` produces a cdecl
   call-site that blows the stack frame. amd64+arm64 link silently
   because NTAPI=cdecl there. Run `verify_abi.py` or `nm` first when ud2
   shows up after a function call.
6. **Wide string encoding.** `b"..."` byte strings used as UTF-16. Must
   be `[u16; N]` or use the `wstr!` macro.
7. **Static lifetime on inline arrays.** Inline `[u16; N]` arrays as
   temporaries → dangling pointers; declare as `static`.
8. ~~**LLVM captures(none).**~~ — CLOSED 2026-05-06. The workspace
   `lto = false` + `codegen-units = 256` firewall is sufficient. Every
   io captures(none) workaround we re-probed turned out to need no
   structural change.
