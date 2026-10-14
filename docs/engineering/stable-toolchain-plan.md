# Stable-everywhere toolchain — kill the nightly surface (i386 kernel retires via wow64)

**Lane:** `stable-toolchain` (worktree `vibeos-stable-toolchain`). **Opened:** 2026-07-12.
**Goal:** get the tree onto **stable rustc**. Two independent axes (see below); i386
*kernel* is the one cell that cannot follow, and it is deleted — not fixed — when the
wow64 lane makes 64-bit the sole supported kernel. **No `core` vendoring.**

> **Why no vendoring:** the sole genuine nightly-*target* requirement is **i386
> kernel-mode softfloat** (`rustc-abi: x86-softfloat`, target-spec-only, no `-C`
> equivalent, upstream-gated). That is *exactly coextensive with shipping a 32-bit
> kernel* — KTRAP_FRAME doesn't save xmm across i386 kernel entry, while usermode saves
> full FP state, so i386 *usermode* has no softfloat need. WoW64 is pure usermode on a
> 64-bit kernel — **zero 32-bit kernel-mode code**. So once wow64 lands, the i386 kernel
> is deleted, the softfloat requirement evaporates, and the remaining i386 usermode DLLs
> flip to the stable builtin `i686-pc-windows-gnu` triple (already wired in reactos-link
> as `rust_user_target`). Vendoring a softfloat `core` would be **perpetual** maintenance
> (re-vendor + full ud2/captures/sacred-primitives/offset audit each toolchain bump, and
> you own every miscompile in the rebuilt core — the class that produced the arm64
> outline-atomics BUGCHECK) — all for a target we're retiring.

---

## ⚠️ The 2026-07-12 survey CORRECTED three things the original plan got wrong

Recorded loudly because each one would send the next agent the wrong way:

1. **The nightly surface is TWO axes, not one.** The original plan only saw `-Z build-std`.
   It never mentioned **`RUSTC_BOOTSTRAP=1`**, which appeared in **37 CMakeLists** — the
   whole driver/user-DLL fleet. Those modules were *already* on builtin triples with the
   stable rustc binary and **never used build-std**; `RUSTC_BOOTSTRAP=1` (a stable-rustc
   escape hatch that unlocks nightly features) was the ONLY thing making them nightly.
2. **`c_variadic` was mis-scoped as "~12 x86-only driver entry points, out of scope."**
   Ground truth (from the compiler, not grep): **25 C-variadic DEFINITIONS across 12
   crates**, including user-mode DLLs on all arches (shell32, version). Grep undercounts —
   many signatures are multi-line. *Declaring* and *calling* a variadic extern fn is
   already stable; only **defining** one is gated. That distinction is what made the fix
   cheap.
3. **The "one correctness gate" (`abi-return-struct-as-int`) had an INVERTED premise and
   is DISCHARGED.** The plan said "every kernel `*.json` leaves it `false`." It is in fact
   **never `false` anywhere** in the 122 specs — `true` where present, absent otherwise.
   Empirically (IR-diffed across 13 struct shapes incl. `LARGE_INTEGER`/`LUID`/`COORD`/
   `IO_STATUS_BLOCK`): the flag is **inert on x86_64 and aarch64** (rustc's win64/aarch64
   callconv never reads it) and only live on **i686**, where our spec already **matches**
   the builtin and matches clang's C ABI. There are **215** by-value struct returns on the
   FFI boundary (many live — `MmGetPhysicalAddress`, `KeQueryPerformanceCounter`,
   `ScsiPortGetPhysicalAddress`, …), so the audit was worth running; it just blocks
   nothing. **Independent proof:** i386 boots today, and `MmGetPhysicalAddress` is on the
   boot path — had the i686 flag been wrong, it never could have.

---

## Axis A — nightly LANGUAGE features (`RUSTC_BOOTSTRAP=1`). **DONE.**

No target change ⇒ no ABI change ⇒ zero risk. This was the cheap 80%.

| Surface | Disposition |
|---|---|
| `link_llvm_intrinsics` | ✅ already dead (`code_addr!()`, landed w/ cswitchdep). |
| `linkage` | ✅ **deleted.** One vestigial `#![feature(linkage)]` in `hal-cargo/src/main.rs`, **zero** `#[linkage=…]` sites. Upstream marks it **`S-tracking-perma-unstable`** — it will *never* stabilize, so waiting was never an option. |
| 25 of 37 `RUSTC_BOOTSTRAP=1` | ✅ **deleted** — pure cargo-cult. Each verified by compiling the crate on plain stable *first* (afd, fastfat, cdfs, npfs, msfs, mup, fs_rec, ksecdd, ks, fltmgr, serial, atapi, buslogic, partmgr, ramdisk, hidclass, hidusb, kbdhid, mouhid, dxg, usbaudio, usbccgp, usbehci, usbohci, usbuhci). |
| 12 crates × `c_variadic` | ✅ **shimmed** — see below. |

### How `c_variadic` was killed (the pattern — reusable)

Stable Rust cannot do exactly two things: **DEFINE** a C-variadic fn (`va_start` is legal
only inside one) and **`va_arg`**. It *can* declare and call one. So:

* A per-module **`rust_varargs_shim.c`** supplies **only those two primitives**: one
  3-line variadic trampoline per function, plus six `vibeos_va_arg_*` pullers.
* The trampoline hands Rust a **POINTER to its `va_list`** (`(void*)&ap`) — never the
  `va_list` by value. That avoids assuming `va_list` is a single pointer (it is `char*` on
  all three of our Windows targets, but an array-of-struct on SysV x86-64), and it is what
  lets the mechanism be **unit-tested natively on the host**.
* All real logic — the printf formatters, the strsafe length/`NTSTATUS` semantics, the
  driver notification dispatch — **stays in Rust**, in a fixed-arity `Name_v` twin.
* `varargs-rust`'s `VaArgs::arg::<T>()` deliberately mirrors nightly's
  `VaListImpl::arg::<T>()`, so a translated formatter body ports by changing only its
  parameter **type** — not one line of logic.
* **Zero of the 184 call sites changed** (they already used stable `extern "C"` decls).

This is a shim for a *language gap*, not a Rust translation reverted to C (AGENTS.md
"Rust Is The Goal"). `VaArg` is implemented only for types that survive C's default
argument promotions, so `arg::<u8>()` — the classic `va_arg` footgun — will not compile.

**No-op debug fns took a different route — and this is NOT a style choice, it is load-bearing.**
`AhciDebugPrint` (storahci), `TRACE`/`WARN`/`FIXME` (version), `dprint_noop` (scsiport) and
`TracePrint` (cdrom, 962 call sites) were no-op variadic *fns* with Rust-only callers. They
became **`macro_rules!`** — zero cost, no C shim, and it *restores* the original shape (every
one of them was a C preprocessor macro that expands to **nothing**; Wine's TRACE/WARN/FIXME
and WPP's `#define TracePrint(x)` never even evaluate their arguments). storahci, version and
cdrom therefore gained **no C file at all**.

> ⚠️ **We learned this the hard way — see the field manual, "NEVER translate a C debug MACRO
> into a no-op `fn`".** cdrom's `TracePrint` was *first* given a C trampoline like the others.
> A trampoline is an **opaque** call, so LLVM can no longer fold the arguments of ~960 call
> sites — which uncloaked a pile of pre-existing §1 ABI mismatches (`_Name@N` stdcall decls
> for cdecl definitions) and detonated the **ud2 cascade**: cdrom.sys went 100 → 114 `ud2`
> sites and **i386 BUGCHECKed 0x1E / 0xC000001D at a `ud2` right after `calll _TracePrint`**.
> amd64 and arm64 booted to desktop, green, through the entire episode — **i386 was the only
> arch that told the truth.** Switching to a macro dropped it back to 100 `ud2` and i386
> booted. The uncovered ABI mismatches were fixed on their own merit (`autorun.rs`'s
> `extern "system"` block was mixing genuine NTAPI kernel exports with cdrom's own cdecl
> helpers), and the WPP constants mis-emitted as `extern "C" static` were fixed to `const`.

**Retirement:** `c_variadic` is FCP-complete with a merge disposition
([rust-lang/rust#155697](https://github.com/rust-lang/rust/pull/155697)), earliest stable
**1.99 (~2026-10-01)**. When it lands, delete every `rust_varargs_shim.c` + `varargs-rust`
and restore the variadic Rust defs. ⚠️ **But note the stabilization EXCLUDES custom
`target.json` targets** — so a `-cargo` cell can never use `c_variadic` while it is on a
JSON target. Another reason to finish Axis B.

### Two things the shim work turned up (owed / recorded, not silently dropped)

1. **`KDDEBUG` is already broken, independently of this lane.** `kdcom-serial-rust/src/kdcom.rs`
   and `kdgdb-rust/src/kdcom.rs` DEFINE a C-variadic `KdpDbgPrint` behind
   `#[cfg(feature = "KDDEBUG")]` — but neither crate has (or ever had) a
   `#![feature(c_variadic)]` gate, so enabling `KDDEBUG` fails to compile **even on
   nightly**. It stays latent only because the cargo feature is off by default. Untouched by
   this lane (`git diff` clean on both crates). When someone wants KDDEBUG back, it needs the
   `rust_varargs_shim.c` treatment — and note these are **`-cargo` cells on JSON targets**, so
   they can *never* use `c_variadic` even after it stabilizes (the stabilization excludes
   custom targets). The shim is their only path.
2. **The `TracePrint` fix exposed a translation-artifact class worth grepping for.**
   `cdrom-rust/src/mmc.rs` declared the WPP constants `TRACE_FLAG_IOCTL` / `TRACE_LEVEL_ERROR`
   as **`extern "C" static`** — but in C they are *compile-time constants* (`storswtr.h`), not
   linkable symbols. It stayed latent only because the old no-op Rust `TracePrint` let LLVM
   fold its arguments away; the moment `TracePrint` became a real extern call, they surfaced as
   `undefined symbol` link errors. Fixed to `const` (matching how sibling modules already do
   it). **The general shape — a C `#define`/enum constant mistranslated into an `extern static`
   — is invisible until something forces the argument to materialize.** Worth a sweep.

## Toolchain pin (was missing entirely — a live landmine)

There was **no `rust-toolchain.toml` anywhere**; `RUSTUP_TOOLCHAIN=nightly` floated to
whatever rustup last installed, and `reactos-link` called `rustup run **stable**` (the
floating channel). Meanwhile upstream had **already removed** the `VaListImpl` /
`.as_va_list()` API our 25 variadic defs were written against (it is `VaList` +
`next_arg::<T>()` now). The build was one `rustup update` from breaking.

* `rust-toolchain.toml` → **stable `1.92.0`** + the 3 Windows targets (the default for the
  whole driver/DLL fleet).
* `.rust-nightly` → **`nightly-2025-12-14`** (== the known-good `430d8297c`), single-sourced
  and read by `sdk/cmake/gcc.cmake`, `reactos-link`, `cargo_orchestrate.py`, `lints.py`,
  `verify_offsets_win32k.py`, and the 10 `build-and-test.sh` dev scripts. **Bump it there.**
* Pin what works; a version bump is its own deliberate lane (it must re-run the
  miscompile-tripwire battery — `scan_ud2`, `scan_captures`, `verify_sacred_primitives` —
  and 1.97 turns on v0 symbol mangling by default, which touches the symbolication tooling).
  We are on 1.92 while current stable is **1.97**.

---

## Axis B — `-Z build-std` + the 122 target JSONs. **DONE (2026-07-12).**

The 42 `*-cargo` link cells now build the **builtin triples on pinned stable**. `-Z build-std`
and the 80 amd64/arm64 target JSONs are **gone**; only i386 keeps a custom json + nightly.

### What shipped
- `gcc.cmake:add_cargo_module` selects per ARCH: i386 → `i686-kernel.json` + nightly +
  `-Z build-std` (since 2026-10-14 for the kernel kinds only; i386 user-mode cells take
  `i686-pc-windows-gnu` on stable); amd64 → `x86_64-pc-windows-gnu`; arm64 → `aarch64-pc-windows-gnullvm`.
  The engine hook already existed (`kernel_build_std = kernel && target.is_json()`), so a
  builtin target auto-drops build-std for the inner body build too.
- **80 target JSONs deleted** (42 `x86_64-kernel.json` + 38 `aarch64-kernel.json`). The drift
  they had accumulated (3 amd64 variants, 2 arm64) is gone with them.
- `reactos-link`: empty CRT **stub archives** on a preceding `-L` (walls off the builtin
  triples' `late-link-args` — `-lmsvcrt -lgcc -lkernel32 …`), `libmemcmp.a` linked LAST in the
  group as a pure fallback, and `-C force-frame-pointers=yes` restored on the body crates.
- Policy is mirrored in `tests/qemu/cli/lints.py`, `tools/cargo_orchestrate.py`, and the new
  `sdk/lib/rust/cargo-cell-args.sh` (sourced by the six dev helper scripts, so they stop
  carrying six private copies of it).

### What the plan got WRONG (all four watch-items were mis-predicted)
- **`.text` does NOT grow ~2.4× on the kernel.** ntoskrnl amd64 is **+0.4%** — flat — and its
  *file* shrank 7.5%. Most modules **shrank 16–29%**. The growth is real only for small modules
  that merely *touch* `core` (bootvid +115%, hal +77%): `x86_64-pc-windows-gnu` sets
  `function-sections: false`, so `--gc-sections` can't strip unused `core` functions and one
  reference drags a whole codegen unit in. build-std used to compile `core` with our
  `codegen-units=256`, which gave fine-grained DCE for free. Net ISO: **+0.2%**. Accepted —
  `-Z function-sections` is nightly-only, so it is not fixable on stable.
- **arm64 does not "lose `+outline-atomics`" — it GAINS them**, and it *shrinks* rather than
  grows: `aarch64-pc-windows-gnullvm` has `function-sections: true`, the mirror image of amd64,
  so gc actually bites. hal −85%, csrss −80%, bootvid −77%, ntoskrnl −2.9%.
- **`-C linker=` in RUSTFLAGS was the wrong lever.** It's a *config key*, so it belongs in
  `.cargo/config.toml`, where it COMPOSES with an env `RUSTFLAGS` instead of being clobbered by
  it (env RUSTFLAGS *replaces* config `rustflags` wholesale).
- **`-C link-self-contained=no` is amd64-only.** gnullvm rejects it outright at bin-link time
  ("not supported on this target") — it links no CRT objects to begin with.

### The bug this uncovered — §9.65, a CLOAK
Dropping build-std hung **both** arches at boot with an empty serial log. Root cause was NOT the
migration: `bootvid-rust` had defined its own `memcpy`/`memset` whose bodies were
`copy_nonoverlapping`/`write_bytes` — which LLVM lowers back into a **call to themselves**. They
compiled to a literal `jmp .`. It never fired because `compiler-builtins-mem` had been linking a
correct competing definition that won under `--allow-multiple-definition`. Removing build-std
removed the competitor and elected the broken twin.

**The generalisable lesson: removing a redundant definition is not "cleanup" — it elects a new
winner, and the new winner has never run.** (Same shape as Axis A's cdrom `TracePrint` cloak.)

Permanent detector: **`tools/scan_self_recursive.py`** (wired into the lint gate beside
`scan_ud2`) — flags any function whose first instruction jumps to its own entry. It immediately
found **30 more** pre-existing silent-hang landmines (23× a C `exp2()` folded into self-recursion
by the same LLVM idiom; classpnp's `KeRaiseIrql`/`KeLowerIrql` stubs; ftfd's `loop {}` panic
handler). Those are baselined in `tools/lint-baseline/scan_self_recursive.txt` — tracked and
**owed**, not accepted; the gate fails on anything NEW.

**i386 kernel: UNTOUCHED, by design.** It stays on `i686-kernel.json` + nightly, blocked on
`rustc-abi: x86-softfloat`. Re-confirmed 2026-07-12: compiling a **plain integer loop** (no
floats at all) for the builtin `i686-pc-windows-gnu` emits **19 xmm + 1 x87** instructions (LLVM
auto-vectorizes with SSE2) vs **zero** under `i686-kernel.json`. The i386 kernel does not
save/restore XMM across traps, so the builtin triple would silently scribble user-mode FPU state.
Retired by the wow64 lane, not fixed here.

## Definition of done
- ✅ Axis A: zero nightly *language* features; `RUSTC_BOOTSTRAP` gone from the tree.
- ✅ Toolchain pinned + single-sourced.
- ✅ ABI-return-struct audit discharged (proven inert on x86_64/aarch64 — only the 32-bit x86
  callconv reads it; empirically verified byte-identical IR).
- ✅ amd64 + arm64: no `*-cargo` cell uses `-Z build-std`; all on pinned stable.
- ✅ 80 amd64/arm64 target JSONs deleted; drift eliminated.
- ✅ ntoskrnl `.text` delta measured + recorded (amd64 +0.4%, arm64 −2.9% — kernel included, not
  deferred).
- ✅ i386 kernel untouched + labeled as wow64-retirement scope.
- ✅ ×3 gate green.

## Pointers
- Shim pattern: `sdk/lib/rust/varargs-rust/` (+ `rust_varargs_shim.c.template`);
  reference impl `drivers/usb/usbhub/`.
- reactos-link engine: `sdk/lib/rust/reactos-link/src/lib.rs` (`rust_user_target`,
  `is_json`/`kernel_build_std` gate, `Linker::new`, `ModuleKind`).
- Outer drive: `sdk/cmake/gcc.cmake` (`add_cargo_module`).
- Target spec: `sdk/lib/rust/*-cargo/i686-kernel.json`, kernel-kind cells only (the 80
  amd64/arm64 jsons were deleted on land; the i386 user-mode cells' on 2026-10-14).
- Cargo policy (keep the 4 copies in sync): `sdk/cmake/gcc.cmake:add_cargo_module` (production),
  `tests/qemu/cli/lints.py`, `tools/cargo_orchestrate.py`, `sdk/lib/rust/cargo-cell-args.sh`.
- Self-recursion detector: `tools/scan_self_recursive.py` + `tools/lint-baseline/scan_self_recursive.txt`.
- Full analysis: `../archive/cargo-target-model-builtin-vs-buildstd.md`,
  `../archive/execution-context-ownership-model.md` §"Toolchain reality (v3)".
- Roadmap ledger entry (retire on land): `docs/product/vibeos-roadmap.md` §Owed-work
  "Stable-everything toolchain program".

## Verify the current matrix
```bash
# amd64/arm64 must show ONLY the builtin triple + no -Z build-std; i386 only the json.
grep -hoE -- "--target [^ ]*\.json|--target (i686|x86_64)-pc-windows-gnu|--target aarch64-pc-windows-gnullvm|-Z build-std[^ ]*" \
  build-<arch>/build.ninja | sort | uniq -c
grep -rl RUSTC_BOOTSTRAP --include=CMakeLists.txt .   # must be empty
grep -rn '#!\[feature(' sdk/lib/rust --include=*.rs   # must be empty
```
