# VibeOS Translation & Kernel Debugging Field Manual

Symptom-indexed reference for C→Rust translation regressions **and** live kernel crashes
(BUGCHECK, hang, heap corruption) in the ntoskrnl/win32k/HAL/freeldr port. **When a flip
crashes, hangs, links undefined, or smokes red, find the symptom here BEFORE instrumenting** —
most of these cost a multi-hour session the first time, and the root cause is rarely where the
crash points.

Entries are **Symptom → Cause → Fix**, one incident compressed to its reusable core. The
commit-by-commit narrative lives in `git log`; this file holds only what you need to recognise
the bug again.

**Three meta-rules that recur in every section:**
- **A green i386 does NOT exonerate a flip**, and a green 64-bit does not exonerate i386.
  Most single-arch deterministic crashes are real bugs the other arches luck-mask. Build and
  boot all three.
- **Check ABI first** (1 second: `nm`, `verify_abi.py`). A `ud2` after a call or a folded
  success path is `extern "C"` vs `extern "system"` far more often than anything exotic.
- **`cargo check` green ≠ `cargo build` green ≠ link-clean ≠ runtime-correct.** Each layer
  catches what the previous cannot. Integrate, link all three arches, and smoke.

## Contents

**Translation bug classes** · [ABI, FFI & calling convention](#abi-ffi--calling-convention--the-1-i386-crash-class)
· [LLVM folds](#llvm-folds-ud2-capturesnone-and-the-cgu-lottery)
· [bindgen & layout](#bindgen-struct-layout--literal-width)
· [Constants & match arms](#constants-invented-guessed-and-swallowed-match-arms)
· [Dropped & degraded output](#dropped-degraded--silently-rewritten-translation-output)
· [SEH & PSEH](#seh-pseh--exception-handling)

**Trusting your tools** · [Stale artifacts & false greens](#stale-artifacts--false-greens)
· [Your oracle is lying](#your-oracle-is-lying-harnesses-lints--generators)
· [Cohab flip & cold link](#cohab-flip--cold-link-mechanics)

**Subsystems** · [Mm, PFN & paging](#mm-pfn--paging)
· [SMP](#smp-what-a-second-cpu-unmasks)
· [arm64 SEH & bring-up](#arm64-seh-unwind--bring-up)
· [WoW64 / FEX](#wow64--fex-3264-divergences)
· [win32k, GDI & input](#win32k-gdi-input--desktop)
· [Per-arch reference](#per-arch-quick-reference)

**Hunting a live crash** · [Crash-hunting toolkit](#crash-hunting-toolkit)
· [Harness flakes & contention](#harness-flakes-contention--host-starvation)
· [Repo hygiene](#working-in-this-repo-without-breaking-a-sibling-lane)
· [Still open](#still-open)

## Adding an entry — read this first

This file is edited by many lanes at once, so **placement is the anti-conflict protocol**:

1. **Put the entry in the section its symptom belongs to** — never at end-of-file. Appending
   at EOF is what made every lane's merge conflict; two lanes writing to different sections
   merge cleanly.
2. **Do NOT invent a number.** Entries are identified by their heading. The `[§9.N]` tags on
   older entries are *frozen historical IDs*, kept only so pre-existing cross-references still
   resolve by grep. Sequential numbering caused five same-number collisions in one week
   (a number is only safe once merged, and lanes don't merge before writing) — the counter is
   retired. `tools/scan_doc_sections.py --lint` fails the gate if a new one appears.
3. **Merge into an existing entry when it's the same class.** A new face of a known bug belongs
   as a line on that entry, not as a new one.
4. **Cite by name, not by number.** In code comments, state the lesson in the comment itself;
   reference this manual only when deliberately deferring detail. ~473 of the old `(§9.N)`
   citations in the tree add nothing the surrounding comment doesn't already say, and they rot.

---

## ABI, FFI & calling convention — the #1 i386 crash class

On i386 `extern "C"` is cdecl and `extern "system"` is stdcall/NTAPI: different decoration
*and* different stack cleanup. On amd64/arm64 they coincide, so these bugs link clean and boot
fine on 64-bit and detonate only on i386 — or, for return/argument-width variants, the reverse.

**Ground truth for a symbol's convention, in priority order:** the C header (`NTAPI`/`WINAPI`/
`__stdcall` ⇒ stdcall ⇒ `extern "system"`; plain C runtime ⇒ cdecl ⇒ `extern "C"`), then the
`.spec` (`@ fastcall` is real and surprising — `ExReleaseResourceLite`, `Iof*`, `Obf*`), then
the linker (`undefined _Name` ⇒ it wants stdcall; `undefined _Name@N` ⇒ it wants cdecl).
Prefer importing from `ntkrnl_sys::ddk` over hand-declaring: bindgen carries the real per-arch
convention, including fastcall. **Never trust a header that omits `NTAPI`** — `internal/mm.h`
declares `MmDeleteVirtualMapping` without it and the symbol is stdcall anyway `[§9.107]`.

### The five ways a convention mismatch manifests
The same root produces five distinct symptoms; knowing which you're looking at saves the hunt.
1. **`ud2` after the call / folded success path.** The callsite's ABI contradiction lets LLVM
   prove the continuation unreachable → `unreachable` → BUGCHECK 0x1E `0xC000001D`. Cascades:
   one classpnp sweep folded ~46 functions from 76 decl↔def mismatches, and the cohab link
   reported **0 undefined** because lld fuzzy-matches `_F` ↔ `_F@N`.
2. **Link failure** — `undefined symbol: _Name` while the kernel exports `_Name@N` (or vice
   versa). The honest, easy case.
3. **Silent ESP drift, no fold, no link error** `[§internal-esp]`. Two ABIs that "agree enough"
   emit a real call that leaks arg_size bytes per call. Harmless in a single-call caller
   (absorbed at the frame-pointer epilogue) but a *loop* corrupts its own ESP-relative locals:
   kernel32's `TranslateStdHandle` (cdecl def, `extern "system"` decl) made
   `WaitForMultipleObjectsEx` read garbage handles → `STATUS_INVALID_HANDLE` → black screen.
   `WaitForSingleObject` (one call) survived; the per-handle loop did not.
4. **The call DIVERGES — even at 0 args.** `lsasrv`'s `DsSetupInit` (`VOID Foo(VOID)`, cdecl)
   was declared in an `extern "system"` block; the `_DsSetupInit@0` reference *resolved to
   something* via auto-import fixup, so the link was clean, and the call jumped to a bad
   address — swallowed by DUMMY_PSEH into an i386-only lsass init hang, no BUGCHECK, no panic.
   The bug is in symbol *resolution*, so arg count is irrelevant.
   **Variant with NO ud2 and no link error** `[§9.30]`: LLVM can resolve the same UB by
   **silently deleting the calling branch**. i386 booted to a fully working desktop with the
   desktop *icons* simply missing — no crash, no error, a code path that just wasn't there.
   **"A code path missing on one arch only, with zero errors" is the signature of a UB fold**,
   and it is why `tools/scan_cc_mismatch.py` exists.
5. **Silent corruption from the wrong flavour** — blanket-flipping a callee block to
   `extern "system"` breaks the fastcall members (`IofCallDriver`, `IofCompleteRequest`,
   `ObfDereferenceObject`, `ExReleaseResourceLite`): stdcall passes on the stack, fastcall in
   ecx/edx. No link error, just wrong data.

**Detection.** `tools/scan_cc_mismatch.py` (same-crate decl-vs-def), `verify_abi.py` (extern
blocks vs `ntoskrnl.spec` + bindgen), `verify_ffi.py` (workspace-wide),
`tools/scan_ir_mismatch.py` (catches the fold *before* it folds, from `.ll`). Refined
fold-truth model `[§9.102]`: **CC mismatch folds against both defs and declarations**;
**signature mismatch (arity / int width / void-vs-value) folds only against a definition**;
**variadic-call vs fixed-arity def folds.** Align decls by rewriting to the def's IR shape
using always-resolving tokens — do not "simplify" the def to match a decl `[§9.107]`.

**The blind spot all the spec-based checkers share:** an *internal*, non-exported helper is
invisible to spec/bindgen cross-checks, and if it is declared only once (wrongly) the
"appears under both ABIs" scan misses it too. The scan that catches these is **decl-ABI vs
def-ABI per symbol, over the crate's own symbols.**

### Wrong-direction fix: aligning the DEF to a lying header `[§9.107]`
Resolving a `scan_ir_mismatch` "def X vs decl system" by changing the **definition** to cdecl
is backwards when the header simply omits `NTAPI`. i386-only BUGCHECK 0x1E follows, and the
same symbol was the source of the one live `KNOWN_CGU_UD2`. **Align to the definition's real
ABI and the `.spec`, never to a header that could be lying.**

### A WRONG decoration can be load-bearing `[§9.125]`
A cold i386 build fails linking a driver on `___debugbreak@0`, undefined *and defined in the
same archive*. "Fixing" it to also emit the cdecl name made it worse: aliasing links the object
carrying the label, and that object carries kernel-only code → four *more* undefined symbols in
ksecdd. **Before fixing a decoration mismatch, ask what else lives in that object.** Correct
fix: define it locally per-arch as an `#[inline(always)]` emitting `int3` / `brk #0xf000`.
This was the *fourth* distinct break found by successive cold builds of one tree, each hidden
behind the previous — see "cold-build breakage comes in LAYERS" under stale artifacts `[§9.126]`,
which reached the same conclusion independently from mainline's side.

### Phantom symbols — declaring something that is not a symbol
A C **macro** or **FORCEINLINE** has no out-of-line symbol; `extern`-declaring it demands a
symbol that exists nowhere. It may "resolve" by accident and break later when an unrelated
change stops the caller being dead-stripped.
- `IsEqualGUID` is a *macro* (`!memcmp(...)`) in non-C++ guiddef.h. It only ever linked because
  `wmi-rust` happened to export a same-named function; on i386 the decorations diverged.
  **Fix:** implement the macro inline in Rust.
- **`KeGetCurrentIrql` is a real export on i386 but a FORCEINLINE on amd64/arm64** — the
  inverse of the usual, so it links on i386 and dangles on 64-bit. Import the per-arch inline
  explicitly (`use ntkrnl_sys::ke::KeGetCurrentIrql;`); a glob re-export does **not** beat a
  local `extern` decl, and any LLM-emitted local `extern` decl of a DDK FORCEINLINE must be
  deleted. Same class: `Io{Get,Set}*IrpStackLocation`, `IoMarkIrpPending`,
  `IoSetCancelRoutine`, `IoSetCompletionRoutine`, `RtlInitEmpty{Ansi,Unicode}String`,
  `ExAllocatePoolZero/Uninitialized`, `NtCurrentTeb`, `RtlGetProcessHeap`, `RtlULongMult`,
  the `NtUserx*` wrappers, the strsafe `StringCch*` family, the `GDIOBJ_*` family.
- **`SharedUserData`** is a C macro for a fixed VA, not a symbol (amd64 `KeQuerySystemTime`
  inline references it). Provide a crate-local `#[no_mangle] pub static mut` at
  `0xFFFFF78000000000`.
- **Before writing `extern { fn Foo }`, confirm `Foo` is a real out-of-line symbol** (`nm`, or
  grep for a definition), not a header macro.

### A typedef that is ALREADY a pointer, given a second star
The star lives in a header, so the model sees `_locale_t locale` as a bare type name and adds
the indirection the parameter appears to lack — `locale: *mut _locale_t`, i.e. a
`**localeinfo_struct` where the caller passes a `*localeinfo_struct`. On msvcrt's `mbcs.c` it
did this in **73 of 77** locale-taking signatures.

**The error is invisible at the ABI.** Both spellings are one machine pointer, so the signature
links and the call passes identical bits; only the body's dereferences break, and only *some*
of those are loud. It was caught here solely because `(*locale).mbcinfo` failed to typecheck —
put the same mistake at a parameter the body merely forwards and it compiles, links, ships, and
reads one indirection too deep forever. The same shape appears on **locals**
(`let mbcinfo: *mut pthreadmbcinfo`), usually with a `.cast()` bolted on to make it build.

**Fix:** `rustconv`'s `signature_fidelity` pass strips the invented star wherever the C declares
the parameter with no `*`, `typedefs.py` proves that name a plain pointer from the TU's own
include closure, and the Rust is exactly one star on *that same name*. Nothing else is touched:
the same file had 46 parameters where the model's spelling merely disagreed with a mechanical
mapping and the model was **right** (`size_t`/`usize`, `threadmbcinfo *`/`pthreadmbcinfo`).
Locals are not covered — grep `\*mut <ptr typedef>` after a flip.

### Per-arch provider differs, so a single-arch link check misses it
`KeRaiseIrql`/`KeLowerIrql` are provided by **ntoskrnl on i386** (plus fastcall
`@KfRaiseIrql@4`), by **hal on arm64** (halstubs), and by **neither on amd64** — so routing a
cohab driver through the tier1 `*Internal` wrappers links on i386/arm64 and fails on amd64.
Use the self-contained CR8 asm (`KfRaiseIrql_amd64`/`KeLowerIrql_amd64`) there.
`ExRaiseStatus` is exported only under that name, not as `RtlRaiseStatus` (the ntkrnl-sys
`#[link_name]` alias resolves only *inside* the kernel image). arm64 lacks exported
`KeGetCurrentPrcb`. **Pre-flight every cohab `.sys` link on all three arches** — cargo-check is
blind to all of it.

### Width mismatches: return, argument, and static
Same family, three positions, all silent on the arch where the widths happen to agree.
- **Return.** A `-> u32` decl over a `BOOLEAN`(u8) definition: the callee sets only `AL`, the
  caller reads all of `EAX` → false-"true" on non-`xor eax,eax` paths, nondeterministic (reads
  as a flaky boot). Real damage: `RtlEqualUnicodeString` false match snapped ftfd.dll against
  usbd.sys exports (BUGCHECK 0x4C); cdfs `FlagOn` freed a stack `IoContext` → 0x19
  BAD_POOL_HEADER. **Declare booleans `u8`.** When callers disagree and you own the impl, fix
  the *definition* to return zero-extended `u32` (forces `movzbl`, valid for both). A fn
  DEFINED `-> i32` with BOOLEAN callers is safe (AL is the low byte).
- **u32 def vs i64 decl.** i386 returns i64 in **EDX:EAX**; a u32 callee leaves EDX garbage.
  cdfs `SectorTruncate` returned `0x1_00000000`, which flowed through CcMapData into a spurious
  `>= FileSize` → zero-filled page → 0x1E STATUS_DISK_CORRUPT_ERROR.
- **Argument.** An `i64` param declared `u32` (or swapped order) picks up 4 garbage stack bytes
  or shifts every later arg — cdfs got an Mcb `ByteCount` of `0x80000000_00000800`.
- **`extern static` width drift.** `MmThrottleTop` declared `u64`, defined `u32`, permanently
  tripped the Cc write throttle → `WriteFile` on a FAT volume never returned, with every probe
  in the FS write path showing nothing wrong `[§9.36-throttle]`.
- **Detector:** scan def-vs-decl per symbol for ANY return/param width mismatch, not just
  narrow-vs-wide. Benign aliases to ignore: NTSTATUS=i32, ULONG=u32, LONGLONG=i64, BOOLEAN=u8,
  and pointer typedefs.

### Argument COUNT mismatches — the one clashing-extern bucket that is a real bug
Same family as the widths above, one position further out: the decl and the definition
disagree on how many parameters there are. `clashing_extern_declarations` carries ~1,458
findings, and the 2026-07-17 triage sorted them — pointer-name and const/mut differences are
ABI-identical noise, scalar widths are mostly benign return-register width, conventions are an
i386-only hazard needing the C prototype to adjudicate — but it called a **different parameter
count** "the highest-value needles" and left it unworked. It was right; `tools/scan_extern_arity.py`
now gates it.

classpnp declares and calls `ScsiGetNextSenseDescriptorByType(start, start_len, types,
type_count, out_type, out_buf, out_len)` against a shim defined `(buf, len, ty, found: *mut u8)`.
The 4th argument binds the caller's `typeList.len() as ULONG` — the literal **2** — to `found`,
whose body is `if !found.is_null() { *found = 0; }`: a store to **address 0x2** at
DISPATCH_LEVEL. It has never fired only because the block sits behind
`IsDescriptorSenseDataFormat()` and our virtual devices answer fixed-format sense; a real drive
returning descriptor sense bugchecks. Four siblings had the same shape less violently
(`ScsiGetSenseDescriptor` returned a truthy *pointer* as a `BOOLEAN`, so callers took the
"descriptor found" branch and read out-params the stub never wrote), plus cdrom's
`PORT_ALLOCATED_SENSE` (called with 2, defined with 1) and classpnp's `EventDescCreate` (8 vs 0).
A `VOID` function declared `-> BOOLEAN` is the same bug in the return position: fastfat's
`FatEvaluateNameCase` fed an undefined return register into `r != 0 || CreateLfn != 0`, making
"does this rename need LFN dirents?" effectively random.

**Why nothing else caught it.** cdecl means the CALLER cleans the stack, so a wrong argument
count neither corrupts the stack nor fails to link; rustc never compares the two signatures
because the declaration is all the call site can see; and `verify_ffi`'s arity check compares
against `ntoskrnl.spec`, which does not describe a crate-local shim. Only the crate's own decl
against the crate's own definition sees it.

**Three things that are NOT this bug**, baselined with reasons rather than "fixed": an
**address-only forward decl** (`fn BaseThreadStartup();` exists so kernel32 can write
`BaseThreadStartup as ULONG_PTR` into a new thread's `Rip` — never called through, so its arity
claims nothing; same for freeldr's `PcFindPciBios`); a **uniform syscall-table shape** (the SSDT
spells every entry `(_: usize, ...)` on purpose); and a decl in a file no `mod` chain reaches —
dead code makes no calls, which is why freeldr's parked `directui.rs` drops out on its own.

**Two parse traps that produced confidently wrong answers first**, both now in the GOOD control
fixture so a regression surfaces as a false positive: count against comment-MASKED text
(`RaiseOnError: u8, // BOOLEAN` contributes its own commas and `>`s), and **neutralise `->`
before depth-tracking `<`/`>`** — `Option<unsafe extern "system" fn(usize) -> usize>` drives the
depth negative at the arrow, after which every top-level comma is invisible. That one mistake
reported **3,790** phantom findings against a true count of 15, and the identical trap bit
`crate_repair`'s argument-index counter the same night. If you are counting top-level commas in
Rust, assume `->` is there to ruin it.

### A `transmute` at a callback-install site erases the CALLING CONVENTION
Rust will not unify `extern "system" fn` with a literal `extern "stdcall" fn` (i386) or
`extern "C" fn` (amd64/arm64) in a function-POINTER type, even though the ABI is identical on
that arch. bindgen writes the literal, translations write the portable `system`, so assigning
a translated callback into a bindgen slot is an E0308 — and the reflex fix is to transmute the
address:

```rust
macro_rules! cb { ($f:expr) => { core::mem::transmute($f as usize) } }   // buslogic
```

That silences the *type*, which is the whole type — including the convention. buslogic's six
SCSI miniport callbacks went into `PHW_*` slots this way while defined `extern "C"`; scsiport
dereferences them as stdcall, so on i386 neither side pops the arguments and 8 bytes leak per
SRB dispatch. The `cb!` comment even states the premise ("the bit pattern is identical; only
the nominal Rust type differs") — true for the case in front of the author, and false the
moment the definition's convention drifts. Same shape in cdrom via an explicit
`transmute::<unsafe extern "C" fn(..), _>` into a WDF `EvtCleanupCallback`.

**Fix the slot type, not the assignment.** `tools/rustconv/normalize_island_abi.py` spells
callback slots `extern "system"` across a crate's three islands, projecting from the i386
island — the *only* one that still records which callbacks are stdcall, because 64-bit has one
convention and bindgen flattens every `NTAPI` to `extern "C"` (11,809 `stdcall` blocks on
i386, ZERO on either twin). Then `Some(f)` assigns directly and rustc checks the convention:
reverting one buslogic definition to `extern "C"` now fails with `expected "system" fn, found
"C" fn` instead of shipping. Do NOT blanket-rewrite `extern "C" fn` → `system` on i386 — 34
i386 callback slots are genuinely cdecl and that promotion is the same bug pointing the other
way.

Census of the remaining exposure, so it is not re-derived: ~500 transmute-shaped callback
installs survive, led by dxg-rust (92), win32k-rust `ntddraw_dxeng` (42), cdfs `cdinit` (33),
winsrv's console frontends (~90 across tui/gui/serial), hal `bussupp`/`dma` (42), cdrom (20),
scsiport (15). Normalizing the islands tree-wide is a separate batch: it surfaced 86 E0308s in
ntoskrnl alone, every one a real convention-spelling mismatch that was previously invisible.

### Aggregates, arrays and unions across the FFI boundary
- **`i64` substituted for a by-value `_LARGE_INTEGER`** — i386 cdecl lowers an 8-byte
  aggregate differently from a scalar `i64`; amd64 hides it. Mirror the union type exactly and
  pass `*ptr`, not `(*ptr).QuadPart`.
- **A Rust `[T; N]` passed to a C-variadic is passed BY VALUE** — arrays do not decay. `%s`
  then consumes the first word as the pointer: i386 uefildr wedged in `strnlen(0x6f636553 /*
  "Seco" */, SIZE_MAX)`. Latent under TCG (unmapped reads return 0, scan ends instantly) and a
  multi-hour "hang" under WHPX/KVM, where every probe is a VM exit. amd64 accidentally works
  (Win64 passes >8-byte aggregates by reference). **`.as_ptr()` every array into a variadic.**
- **`extern CCHAR Foo[N]` declared as `*const i8`** — Rust reads the first bytes of the buffer
  *as* the pointer (`"mult"` = 0x746C756D). Mirror the array type, or use a C accessor.
- **Cross-DLL DATA import indirection.** Under `raw-dylib` the `__imp_` load already yields the
  symbol's value, so an extra `*` reads the array's first bytes as a pointer → AV. Declare the
  same indirection ntoskrnl actually exports; the `*const *const` form in `fsrtlfuncs.h` exists
  only under `__CREATE_NTOS_DATA_IMPORT_ALIAS` and must not be copied. Conversely, a cohab
  driver referencing kernel **data** (`KeLoaderBlock`) needs `raw-dylib` on i386/arm64 (PE data
  imports need an `__imp_` thunk); amd64 tolerates a plain extern static and hides it. Declare
  it crate-locally — don't change shared `globals.rs`, the kernel needs the local form.
- **`[T; 0]` extern-array decl** `[§9.35]` — Rust bounds-checks the *declared* length, so the C unsized-
  extern idiom panics at index 0. Declare the real per-arch size.

### i386 no-SSE float ABI: an `f32`/`f64` value cannot cross a fn boundary
i686 usermode crates build `-sse,-sse2`, so rustc rejects any float **passed by value** — and
it checks at the *definition*, so `#[inline(always)]` does not help. Fine on no-SSE i386:
float math inside a body (x87), float pointers, float struct fields. Two fixes:
- **Internal C float helper → `macro_rules!`, not a fn** (`GDI_ROUND`, `_fmul`, `_lrintf`).
  Keep the macro's internal width matching the C — C does transforms in `double`; narrowing to
  f32 silently loses precision.
- **Exported fn taking FLOAT by value → cfg-split the signature.** i386 stdcall passes a
  4-byte FLOAT on the stack, bit-identical to a `u32`; amd64/arm64 pass it in a vector register
  and need the native `f32`. A plain `f32` won't compile on i386; a plain `u32` corrupts the
  others (`SetMiterLimit`).

### i386 symbol lookup and asm aliasing
- **Three decoration schemes**: cdecl `_name`, stdcall `_name@N`, fastcall `@name@N`. A grep
  for the bare name finds nothing. Try all four forms or use `qemu/sym2addr.sh`.
- **`extern "fastcall"` is i386-only** — cfg-split every fastcall block and every
  `#[no_mangle] extern "fastcall"` export (`@Name@N` decoration is load-bearing for the cohab
  link). One private `_impl` + two thin cfg'd wrappers avoids duplicating bodies.
- **A `global_asm!` `.set _X, _X@0` alias must live in the crate that defines the target** —
  GAS evaluates it at assembly time, so across rlibs it emits an undefined reference instead of
  an alias.
- **LLVM COFF-i386 rejects `push offset <label>`** ("unsupported relocation type"); the
  `mov reg, offset` encoding works. Worse, bare `push <label>` is a silent **memory** push.
  Materialise into a provably-dead scratch register, then push it.
- **Verify an asm translation by disasm-diffing against the original `.obj`**, not by smoke —
  SEH/setjmp/CRT paths are barely exercised by a boot. `objdump -d` both (don't extract from a
  *thin* archive; objdump the on-disk `.obj`). Any diff beyond annotated reloc workarounds is
  real.

### Cohab link plumbing
- **A new rlib's `#[no_mangle]` symbols are absent from the staticlib** unless reached: add the
  path dep + `extern crate` + a `#[used]` force-reference block. A Cargo dependency alone is
  not a reachability path.
- **Cross-crate `#[inline(always)]` with an `asm!` body doesn't fire at opt-level=1** — rustc
  falls back to a call against a name that has no symbol. Replicate the two-line asm in the
  consuming crate.
- **A cohab flip can drop a `.spec` export that was an obj-level ALIAS** in the flipped `.c` —
  the name appears in no `.c` file, only the `.spec`. gdi32's `GdiEntry16` aliased
  `DdSwapTextureHandles`. Re-provide it as a forwarding `#[no_mangle] extern "system" fn`,
  all-arch. Recurs across `GdiEntry1..15`.
- **The driver's OWN spec exports must be `extern "system"`** — `spec2def` emits
  `@ stdcall Foo`, so the linker wants `_Foo@N`, but translated bodies default to
  `extern "C"`. 22 Flt + 37 SSPI functions needed flipping.
- **One driver can legitimately hold two conventions** — ksecdd's vendored tomcrypt is cdecl
  while ReactOS's own `MD5Init`/`MD5Update`/`MD5Final` are NTAPI stdcall, in the same file.
  Split by *which library provides the symbol*, never one-block-one-ABI.
- **lld i386 cascades red herrings** — one real undefined makes it report unrelated `Ki*`/
  `Hal*` undefineds that vanish once the real one is fixed. Fix the obvious ones and re-link
  before chasing the weird ones.
- **`git core.ignorecase`**: a `Lib.c` → `Lib.rs` module that case-folds onto `lib.rs` blocks
  `git add` ("will not add file alias"). Rename the module.

### Assorted ABI-adjacent traps
- **LLM appends `as u64` to every SIZE_T/ULONG_PTR** → ~50 i386-only E0308s. Use `as _` or
  `usize`; never hardcode a width for an arch-width field.
- **amd64 `cpuid` 32-bit `xchg ebx` dance zero-extends and wipes the upper 32 bits of rbx**,
  which is callee-saved and may hold a caller's pointer → BUGCHECK 0x50 at a truncated
  address. Use 64-bit `xchg` on amd64; cast at the return boundary, not inside the asm.
- **`PROCESS_ALL_ACCESS` is `0x001F0FFF` on NT 5.x**, not the Vista `0x001FFFFF` in
  `ntkrnl_sys::constants` — the mismatch against C-built access checks hung i386 SMSS.
  `THREAD_ALL_ACCESS` already matches.
- **`HIGH_LEVEL` is 31 on i386 but 15 on amd64/arm64**; amd64 maps IRQL to CR8 (4 valid bits),
  so `mov cr8, 0x1F` is a #GP at boot with an unreliable reported Rip. Disassemble the suspect
  HAL fn and look for `mov cr8, <imm > 15>` — trust the instruction stream over the
  symbolicator for control-register #GPs.
- **c2rust union/bitfield access vs `__BindgenUnionField`** — the repo's bindgen renders every
  union as the legacy accessor-struct, so c2rust's native `reg.All = x` / `reg.Bits.Flag()` /
  `Reg { All: 0 }` all fail (E0599/E0609/E0063). Mechanical: init → `zeroed()`, read →
  `.as_ref()`, write → `*x.as_mut() =`. Nested anon names differ (`.c2rust_unnamed` →
  `.__bindgen_anon_1`), and a member bindgen *flattened* drops the prefix entirely. A native
  bindgen `union` (ddk `_LARGE_INTEGER`) IS direct-access — grep the substrate to tell which.
- **bindgen prefixes enum constants with the enum tag** (`_SCSI_NOTIFICATION_TYPE_RequestComplete`)
  and wraps long `: <type> = N;` onto the next line. Automate: grep the substrate for
  `pub const [A-Za-z0-9_]*<name>:` (anchor the trailing colon so the wrap doesn't break it) and
  emit `pub use ... as <bare>;` — type-free `pub use` beats re-declaring a typed const.
- **Two bindgen `*-sys` crates cannot both be glob-imported** (E0659 on `BOOL`/`DWORD`/`HWND`).
  Pick one for the type bulk; declare the other's functions in local extern blocks using the
  chosen crate's types. Same for `ntkrnl_sys::base::*` + `ntkrnl_sys::ddk::*` — keep the `ddk`
  glob and import base items explicitly.
- **`src/main.rs` is auto-detected as a cargo BIN target** — a module from `main.c` collides
  with the convention and every sibling import "fails to resolve". Set `autobins = false`.
- **i386 FAST_IO / NTAPI callback slots are `extern "stdcall"`** — don't name a raw fn type;
  transmute the `extern "system"` fn item straight to the bindgen `Option<...>` slot type,
  which carries the correct per-arch ABI.

---

## LLVM folds: ud2, captures(none), and the CGU lottery

> **2026-07-05 status:** the volatile-blind attribute-deduction root is **fixed in the shipping
> toolchains** (LLVM 21.1.3+: volatile stores block `readnone captures(none)`; plain out-writes
> deduce fold-proof `writeonly initializes(...)`). A post-opt IR audit of the amd64 kernel found
> **zero live folds**. Layout-robustness is proven empirically: `VIBEOS_LAYOUT_CANARY=1`
> (451 symbols relocated) booted 5/5, where the 2026-06 tree deterministically 0x50'd. The
> **ABI-mismatch → ud2 cascade above is toolchain-independent and fully live.** Keep
> `tools/audit-captures.sh <arch>` in the loop after every rustc/LLVM bump; the entries below
> matter the day a bump regresses it.

### Triage order for any named-function `ud2` `[§9.103]`
1. `scan_ir_mismatch` the crate — CC/signature mismatch is the overwhelming majority.
2. Then the **provable-null-deref fold**, which survives every CC fix and is *not* a mismatch:
   in release builds LLVM sees through a same-crate callee, proves a deref is null, and folds.
   Two sources — **DUMMY_PSEH eager filter evaluation** (cdfs `CdPerformVerify`) and
   **unguarded C null-blindness** (win32k `UserRedrawDesktop`), where the C "works" only
   because nothing ever passes null.
3. Only then captures(none).

### The i386 driver-fleet ud2-bomb class `[§9.99]`
Any same-LLVM-module disagreement about one symbol folds and **passes the linker**. `scan_ud2`
was ntoskrnl-only, so a whole driver fleet carried live bombs. The five shapes:
`#[link_name]` redirects that don't match; a local shim def vs the bindgen decl class; a
`#[no_mangle]` with **no** `extern "…"` (raw symbol, Rust ABI); fastcall decls over cdecl defs
(the kernel exports `Interlocked*` FASTCALL on i386); variadic decls over a non-variadic def.
`cdrom.sys` carried **100** ud2 sites, all on paths boot never hits — which is exactly why
master was green.

### Making a no-op opaque UNCLOAKS everything hiding behind the fold
A C debug macro that expands to nothing, translated as a no-op **`fn`**, is trivially inlinable
— LLVM folds it away *with every argument expression*, and with them any relocation reachable
only through those arguments. Give it an out-of-line body (e.g. a C variadic trampoline) and
the latents materialise at once: undefined symbols for ABI-mismatched callees, `extern static`
decls for what are C `#define`s, and a fresh ud2 cascade. cdrom went 100 → 114 ud2, one on a
live path → i386 BUGCHECK 0x1E `0xC000001D` after `calll _TracePrint`. **The errors point at
innocent code.** Fix: make it a `macro_rules!` (the faithful shape — C never evaluates the
args), consume args by value (`let _ = $arg;`; `&$arg` is E0793 on a packed field), and do
**not** type-bind the parameters, because the per-module extern decls it replaces routinely
disagree with each other. Then fix the ABI mismatches you uncovered — they are real.

### NEVER hand-write `memcpy`/`memset`/`memmove`/`memcmp` in Rust `[§9.65]`
**It cannot be done correctly.** LLVM folds any byte-copy loop back into a call to the function
itself → **infinite self-recursion**. The PC parks on a literal `jmp .` and the serial log is
empty. `ntoskrnl.exe` exports all of them on every arch — **import them, don't define them.**

This is the other **cloak** bug: `-Z build-std-features=compiler-builtins-mem` had been linking
the real implementations ahead of a broken hand-written one in bootvid **for months**, so
dropping `-Z build-std` for the builtin triples made both amd64 and arm64 hang at boot with a
completely empty serial log. A real defect held harmless by a link-order accident — exactly the
same shape as the no-op-fn fold above, and the reason a "no functional change" toolchain edit
can detonate.

### captures(none)-class hardening that is still worth doing
- **Stale/hoisted load through a `static mut` table across a foreign call** — a by-handle
  lookup wrongly rejects an object; a `DbgPrint` between the reads "fixes" it. `read_volatile`
  at the reads, `write_volatile` at the paired writes. Proven pre-emptive step for any win32k
  handle/object/refcount table; boot-time single-threaded init never exercises it.
- **MMIO needs volatile too.** A type error on `write_volatile(cfg.add(0), WIDE_CONST)` gets
  "fixed" as `*cfg.offset(0) = WIDE_CONST as u8`, silently dropping volatile → the driver
  enumerates garbage. **Cast the value, keep volatile.** After any MMIO-touching driver, diff
  `grep -c volatile` before/after.
- **Shared clock reads in spin-waits need volatile** — a `do {} while (t0 == t1)` calibration
  loop never sees the tick change because LLVM hoists the non-volatile load out. i386 was
  immune (it called the real export). Any fixed-VA shared page used as a polling condition.
- **Non-volatile seqlock loops on KUSER_SHARED_DATA are the arm64 `b .` spin class** `[§9.106]`
  — the KSYSTEM_TIME capture loop with plain loads becomes an infinite self-branch (kernel32
  time functions, `EngInitializeSafeSemaphore`). The writer protocol (High1, Low, High2) is the
  same class. Grep any `High1Time`/`High2Time`/SharedUserData access for volatility.
- **`#[no_mangle]` alone does not stop inlining** — it preserves the symbol; LLVM can still
  inline the body into a same-CGU caller. The `#[inline(never)]` set on `shim_exports.rs` is
  defence-in-depth, not redundancy.
- **Don't wrap a *written* `#[no_mangle] pub static` in `Volatile<T>`** — a non-`mut` static
  can land in `.rdata` and page-fault on write despite interior mutability. Keep `static mut`
  and use volatile accessors.
- **Re-probe old C-shim workarounds before keeping them.** Most dissolved under the
  `lto=false` + `codegen-units=256` firewall; their comments are stale. Must-stays: amd64
  `KiSaveXState`/`KiRestoreXState` (KVM bug), i386 `SepInitTokensShim` (fat-LTO ud2).
- **Trace the full consumer chain before believing "load-bearing tracing"** — a capture chain
  whose only reader has no callers is dead; delete writer, reader and callers together.

### The CGU lottery
`codegen-units=256` partitioning is **not deterministic across workspace shape**, so a body
containing a parked ud2 can move onto a live caller's path with no source change.
- **Adding a workspace member repartitions unrelated crates** → a "crp regression" that is
  really a reshuffle. Reproduce on the real tree, deep-nuked (all five `rust-*-target/` plus
  every artifact — conftest nukes only one of five) before accepting the claim.
- **Parallel cold builds → false HANG at `[freeldr] Switching stack...`** with zero kernel
  serial. That line precedes the jump to `KiSystemStartup`, a `KNOWN_CGU_UD2` function; under
  memory pressure from three concurrent cold cargo builds the partitioner lands a ud2 on the
  early-boot path. **Build cold one arch at a time** (smokes may run parallel). Escalate only
  if it reproduces solo and deterministically.
- **A monolithic CGU causes lld `X was replaced`** (distinct from the benign
  `--allow-multiple-definition` warning): all ~100 CRT shims in one module = one object, so
  referencing any symbol drags strong defs that collide with ntoskrnl imports. C had one
  `.obj` per function. Fix: wrap each generated shim in its own `mod`, restoring per-function
  CGU granularity. A 1-second relink "FAIL" is a relink against a **stale** `.a` — `ar` never
  prunes.
- **`-C no-prepopulate-passes` is the class-guard for tiny one-shim-per-CGU objects** — i386
  always carried it; amd64/arm64 (and any cargo driver that scrubs RUSTFLAGS) did not, which is
  why `libcntpr` CRT shims misbehaved *only* on 64-bit (memset dead-store-eliminated into a
  guard-only spin; `_strnicmp` collapsed to always-equal). Scope it to the offending crate,
  not the whole kernel body.
---

## bindgen, struct layout & literal width

Bugs that compile and link clean but corrupt memory or strings at runtime, because the Rust
mirror, literal width, or field offset silently disagrees with C.

### Never hand-roll a shadow struct for a type bindgen already emits
The single highest-yield rule in this section. Hand-rolled mirrors drift; bindgen's are
size-verified. Real damage:
- **`#[repr(C, packed)]` on a naturally-aligned C struct** mislocates every field after the
  first alignment hole. `TDI_ADDRESS_IP { USHORT sin_port; ULONG in_addr; }` has 2 bytes of
  padding, so `in_addr` is at offset 4; a packed mirror puts it at 2 and a connect to
  10.0.2.100 parses as 2.100.0.0. The kernel's parser was right all along.
- **Five+ hand-rolled `KUSER_SHARED_DATA` shadows with fantasy offsets** → BUGCHECK 0x4C on the
  first `-smp 2` boot.
- **A per-file mini-mirror placing a field at offset 0** clobbers the real struct;
  `type X = c_void` makes `size_of::<X>() == 0` → a zero-byte allocation.
- **Pin every mirror you must keep** with an arch-gated `const _: () = assert!(size_of::<T>()
  == 0xNNNN);` matching the C's `C_ASSERT` (basesrv `BASE_STATIC_SERVER_DATA` == 0x1AC8 on
  i386). Never feed a helper a pointer to a private mirror — pass
  `addr_of_mut!((*real).Field)`.
- **`FX_SAVE_AREA`** `[§9.25]`: the C `_FXSAVE_FORMAT` is **520** bytes, not 512 (trailing member); four
  hand-rolled copies modelled it as 512 → garbage `Cr0NpxState` → 0x7F GP in
  `KiFlushNPXState`.
- **A dropped term in a hand-derived `sizeof` `#define`** under-allocates and overlaps the
  extension layout; **a dropped `FIELD_OFFSET(T, Array[N])` subscript scaling** under-allocates
  the heap block.

### bindgen drops things silently — know what
- **`__attribute__((aligned(N)))`.** Losing `__ALIGNED(16)` on `EX_PUSH_LOCK_WAIT_BLOCK` `[§9.24]` broke
  the pushlock word's 4 flag bits (which live in the low bits of the block *pointer*) → i386
  lost-wakeup wedge, "BAD gate", and BSOD roulette from one root. Fix with a
  `#[repr(C, align(16))]` newtype. **Sweep for any C type carrying an alignment attribute.**
- **Nested `_HEAD` structs flatten into `__bindgen_padding_0`.** `THRDESKHEAD`/`THROBJHEAD`
  lose `head.h` (offset 0 → `addr_of!` cast) and `head.pti` (a padding index). Derive the
  offset from the real field sequence and the target's alignment — **do not blanket-apply
  `2*usize`**: head-pti is `2 * size_of::<usize>()` (8/16), but `PROCESSINFO.W32PF_flags`
  after `{PEPROCESS; DWORD}` has no tail padding and is `size_of::<usize>() + 4` (8/12). A
  wrong formula gave 12 instead of 16 on 64-bit → dword-swapped garbage pointer → amd64 AV.
  Anonymous members land in `__bindgen_padding_N` too (BRUSHBODY, PERUSERSERVERINFO).
- **Multichar `#define` pool tags** (`'BdfA'`) are implementation-defined and skipped — re-emit
  them with a `const fn fourcc`. Note the byte order: C `'BdfA'` is
  `(B<<24)|(d<<16)|(f<<8)|A`, i.e. big-endian, which is why `verify_constants` skips `TAG_*`
  (Rust `from_le_bytes` disagrees) and why tag drift must be reviewed by hand.
- **A wide-string `#define` lowers to a NARROW `&[u8;N]`** (the `L` is dropped). Casting it
  `as *const u16` reinterprets ASCII byte-pairs as UTF-16 → `STATUS_OBJECT_PATH_SYNTAX_BAD`,
  "Cannot open winsta". Build a real UTF-16 twin via a `const fn`. The sibling bug is
  `size_of_val()` on that narrow const returning N instead of N*2 → an undersized
  UNICODE_STRING `MaximumLength` → truncation → 0xc000021a → BUGCHECK 0x4C. Use the wide twin
  in **both** the append and the size math.
- **`size_of_val` on a `__BindgenUnionField` marker returns 0** — it is a zero-sized accessor.

### Indexing a bindgen flexible array past [0] — the panic class
bindgen models C `T arr[ANYSIZE_ARRAY]`/`arr[1]` as `[T; 1]`, so Rust `arr[i]` for i≥1
bounds-checks and panic-aborts, while the C intentionally over-allocates. **Always
`arr.as_mut_ptr().add(i)`** (for a `static mut`, `FOO.as_mut_ptr().add(i)` — `addr_of_mut!(FOO)
.add(i)` strides by the whole array). For a `[T; 0]` extern the panic is unconditional.
Three faces, all seen live:
- **Kernel:** enumeration processes [0] then dies; csrss silently dies at i=1 → BUGCHECK 0x4C.
  Watch `DEVICE_RELATIONS.Objects`, `CM_RESOURCE_LIST`, `IO_RESOURCE_REQUIREMENTS_LIST`,
  `KEY_VALUE_FULL_INFORMATION`, `CsrRootProcess.ServerData`, and
  `FILE_FULL_EA_INFORMATION.EaName` (`[CHAR;1]`, whose NUL is read at `EaName[EaNameLength]`).
- **Usermode DLL, LOUD `RUST PANIC` on serial:** the same shape recurs where the *crate's own*
  `[T;1]` typedef models the flex member — `ws2help` `WSH_HANDLE_TABLE.SearchTables`,
  `netapi32` `MIB_IPADDRTABLE.table`. Deterministic on the flipping tree but the gate
  classifies it `NET_FLAKE`; the differential tell is that **only** the flipping worktree fails
  net. The inconsistency is intra-file — sibling functions already used pointer arithmetic.
- **SILENT thread hang, message on debugcon not serial:** `IoCheckEaBufferValidity` panicked;
  the kernel panic handler writes `PANIC:<file>:<line>` to **debugcon port 0xe9**, then
  `loop {}`. QEMU's timer still preempts the spinning thread, so the box stays alive and it
  reads as one operation wedging forever. **When a single op wedges but the system is live,
  check the debugcon capture for `PANIC:` before suspecting a lock.**

### Wide/narrow literal sizing
- **A hand-encoded `[u16; N]` one element short** silently drops the LAST character to make
  room for the NUL. `SERVICES_KEY [u16; 33]` spelled `…\Service` (no `s`), so `RCreateServiceW`
  opened a nonexistent parent key — invisible to boot, found only by a conformance sweep. The
  intent comment above the array says the right string, so review slides past it.
  **`tools/scan_wide_literals.py`** (gate-wired, empty baseline) decodes every
  `static/const NAME: [u16; N]` and diffs it against the `// L"..."` comment above. Two design
  lessons from its first run reporting *20 findings of which 19 were its own gaps*:
  **the oracle is prose, so it must not over-read prose** (skip any intent containing `...`;
  take only a contiguous `//` run, since a blank line means the comment isn't this literal's
  spec), and **there is no blanket "must end in NUL" rule** (UNICODE_STRING Buffers
  legitimately have none — fire only when the comment itself claims NUL-termination, and strip
  trailing NULs on both sides). It reports coverage (`85/782 had an intent comment`) because
  "0 findings" over 0 comparisons is a green that means nothing.
- **A wide literal sized to the exact character count** drops the NUL, so `swprintf`/
  `RtlQueryRegistryValues` scan off the end into adjacent garbage → 0x1E in `wcsnlen`. Harden
  the builder so it cannot recur: `const fn wlit<const N>(b:&[u8]) -> [u16;N] { assert!(N >
  b.len(), "room for NUL"); … }` turns every too-tight call into a build error. uniata had 6
  of 52 sites undersized.
- **`b"..."` has alignment 1**, so `b"C\0S\0…".as_ptr() as *mut u16` can land at an odd address
  → amd64-only 0x1E `STATUS_DATATYPE_MISALIGNMENT` when `ProbeForRead` enforces WCHAR
  alignment. Declare a real `static [u16; N]` (alignment 2 ⇒ always even).
- **`PWCHAR HalName = L"..."` is a pointer VARIABLE, not the bytes.** As a `[u16; N]` the
  symbol sits at the buffer start, so consumers read `'A','C','P','I'` as the pointer →
  triple-fault before any trace point. Emit storage + a `#[no_mangle] pub static mut ptr`.
  Fixed-size `T[N]` arrays keep the array shape.
- **An ANSI byte string cast to a WIDE pointer** makes every lookup silently fall back to
  defaults. **Hand-encoded UTF-16 has three distinct mis-encodings** — swept workspace-wide.

### Bitfields and unions
- **A C `:31` bitfield read as a full DWORD** lets the high flag bit corrupt the value into a
  wild pointer/offset. **A per-file hand-rolled struct view reading a bitfield at offset 0**
  silently disables the feature it gates.
- **Hand-rolled accessors over a union's byte view with two fields TRANSPOSED** fails only
  3 boots in 4 — a transposition is a *value* bug, so it only shows when the two differ.
- **`OriginalPte` is triple-duty in this port** (proto-template protection for active pages,
  and `RmapListHead` for ROS PFNs, which overlay it in the C union). A one-slot offset shift in
  hand-rolled `NextLRU`/`PreviousLRU`/`RmapListHead` accessors (they missed an embedded `Wsle`)
  put LRU links, WS-lock bits and rmap heads on the wrong words — see the Mm section.
- **win32k's flipped files invented thousands of inconsistent signatures and layouts**, so
  cross-file ABI disagreement was systemic; the fix was a deterministic ABI overwrite pass, not
  per-site repair. **A redundant `*-sys` crate that re-bindgens types the parent master header
  already pulls** produces mass E0659 ambiguity.
- **A crate with NO island is invisible to `verify_ffi`.** Its oracle is `ntoskrnl.spec` *and*
  `bindgen_<arch>.rs`; with no island there is nothing to compare against, so every hand-written
  extern block passes unchecked and the gate stays green. shell32-rust carried ~2.7K lines of
  hand-written substrate for this reason and appears in NONE of the 40 crates the
  clashing-extern passes converged — four real ABI disagreements survived in it until a semantic
  declaration comparison went looking. "The lint is green" means nothing if the crate is not in
  the lint's input set; check that before trusting it.

### Adopting an island in a crate that already has hand-written substrate
Measure first, and expect most of the hand-written surface to be redundant: on shell32,
**73 of 90** items in `types.rs` and **117 of 156** in `shellcom.rs` were already provided by the
island or `ntdll-sys` — including a `LARGE_INTEGER` hand-written as `{ QuadPart: i64 }` (a
cartoon of a union) and the PIDL on-disk layouts, where a wrong offset corrupts data on disk.
Compare every hand-written name against the island *and* the sibling `-sys` crates before
deleting; the ones that survive are the ones bindgen structurally cannot emit — pointer-valued
macros (`NULL`, `INVALID_HANDLE_VALUE`, `HKEY_CLASSES_ROOT`), cast-expression constants
(`((HRESULT)0x80004005L)`), and function-like macros. Keep those in `<stem>_extra.rs`, which
`gen_driver_types.sh` re-exports through the generated dispatch so regeneration cannot eat them.

Two traps specific to this move:
- **The island owns DATA LAYOUTS; com-sys / your vtable module owns COM ABI.** bindgen renders
  COM interfaces from a C++ header as classes, while the translation calls through the C
  `lpVtbl` idiom — let the island emit them and every call becomes `E0609: no field lpVtbl`
  (~73 on shell32). Blocklist the interfaces with `EXTRA_BL_ITEMS` *and* re-export them with
  `EXTRA_RAW_LINES`: blocklisting removes the definition, not the references, so without the
  re-export the island fails E0412 on its own output.
- **A `-x c++` cell emits broken aliases for class-template member typedefs** — bindgen
  flattens them to non-generic aliases over unbound parameters (`pub type
  ATL_CComEnum__CComEnumBase = u8<Base, T, Copy>;`). Blocklist by the **C++ qualified name**,
  not bindgen's flattened Rust spelling. Do not "fix" this by dropping every alias carrying a
  `<`: on shell32 that rule would have deleted 82 legitimate `Option<fn-ptr>` callbacks.

Also: `profile_derive` mirrors each TU's include prologue verbatim, so a **quoted** include
copied from a TU in a subdirectory (`#include "CFSFolder.h"` in `folders/CDesktopFolder.cpp`)
breaks — a quoted include resolves against the *including file's* directory and the generated
wrapper lives in `/tmp`. Use the path-qualified spelling the module's own `precomp.h` uses.

### Cross-DLL and shared-section layout
Cross-DLL reads returning garbage *past* the writer's own init, with a hang ~7 DLLs later in an
unrelated consumer, is a broken `#[repr(C)]` mirror of a shared struct: scrambled field order,
opaque `[u8;N]` blobs instead of real nested types, or missing fields. A known constant showing
where a length should be (0x409, the US LCID) is the tell. Rebuild in exact C field order using
real size-asserted bindgen nested types. Related: a hardcoded SD minimum length broke arm64
`CsrServerInitialization`; a consumed field reading as a handle-shaped value (0x3D0, near boot
handles like 0x3DC) means a bit-test never matches after a cohab flip.

---

## Constants: invented, guessed, and swallowed match arms

**This is the single largest translation bug class in the repo, and the least visible.** rustc
sees a valid integer; the linker is happy; the code is often read only once the kernel is
already dying. Every constant that reaches Rust must come from the header, deterministically —
never from the model.

### The LLM invents plausible values `[§9.113, §9.113a, §9.113b]`
`KeBugCheck(SYMBOL)` came back as `KeBugCheck(0xNN)` and **61 of ~115 translated stop codes
were wrong**. Review slides right off it: the numbers are the right magnitude, valid codes, and
internally consistent. The consequence is worse than a mislabel — **a wrong stop code relocates
the HUNT** `[§9.124]`: "arm64 BUGCHECK 0x3 INVALID_AFFINITY_SET" was really
`RESOURCE_NOT_OWNED`, and two long-standing bugs were being chased in the wrong subsystem for
weeks. Related: `mm/section.rs` declared a wrong value that was a real behavioural bug, not
just a label.
- **Fix deterministically:** `ntkrnl_sys::bugcodes::NAME` from `tools/gen_bugcodes.py`;
  `tools/rustconv/rustconv/verify_constants.py` (`--enums`, `--fix`) against a header catalog;
  `tools/scan_bugcheck_codes.py` (empty baseline, gate-wired).
- **`tools/scan_const_contradictions.py` is oracle-free and the cheapest detector** — it asks
  only whether the tree contradicts *itself*, so it needs no catalog.
- **NEGATIVE result worth keeping:** do **not** auto-sweep all 2216 headers. It yields noise.
  False-positive classes both tools must handle: bindgen-generated files are correct *by
  construction*; `#[cfg]`-gated declarations are the Rust spelling of the header's `#ifdef`;
  arch-suffixed paths must bucket per arch; and one header can define a name twice under NTDDI
  gates (`pstypes.h`, `THREAD_ALL_ACCESS`).
- **Two self-incriminating comment shapes to grep for:** a comment naming the right symbol next
  to a numeric literal, and a "value assumed / approximate" note.
- **After fixing a class of bad data, re-audit everything derived FROM it.**

### Locally re-declared enum members with GUESSED values (SILENT)
The translator re-declares an enum member it needs as a local `const NAME: T = <value>` and
guesses — often the intuitive-but-wrong value, or a *different* enum's similarly-named member.
Each file guesses independently, so setter and reader disagree **while both compile and each is
self-consistent**. One 2026-07-04 sweep found 41:
- `AdjustUnwait = 3` (header: **1**) made `KiUnwaitThread` stamp a value nothing matched — the
  unwait priority-boost/quantum-recharge block was **dead code for every thread wakeup** since
  the translation landed.
- Boost/Unwait swapped so `KeSetEventBoostPriority` took the wrong branch (over-boost to ~15).
- `ThreadedDpcObject = 7` collided with `GateObject`; `EventPairObject = 2` = `MutantObject`;
  `SynchronizationTimer = 0` = `NotificationTimer`.
- **`NotificationEvent`/`SynchronizationEvent` swapped in four files** — an event created with
  the wrong type wakes ONE waiter instead of ALL: a lost-wakeup landmine.
- The LPC message-type window shifted (live validation on the csrss CreateProcess reply path);
  `MEMORY_PRIORITY_FOREGROUND` writer(0)/reader(2) disagreement killed the foreground boost.

**Anything `verify_constants --enums` resolves to exactly ONE header value is a strong real-bug
signal** (the ambiguity rule already drops multi-enum names). Two deliberate blind spots to
review by hand: `TAG_*` pool tags (byte order, above) and untagged inline literals at call
sites (`KeInitializeEvent(e, 1, 0)` with no comment) — `scan_inline_flag_literals` only catches
the commented ones. **Prefer bindgen enums over local consts in new translations.**

### The same-named constant with a DIFFERENT value per driver
Cross-driver value bleed: cdfs and fastfat both define a WAIT flag, with different values. A
"known" constant is only known *within its module*.

### An arch-INVARIANT constant, wrong in exactly ONE arch's private copy
`ke/arm64/kiinit.rs` declared `PRCB_BUILD_UNIPROCESSOR = 1` / `PRCB_BUILD_DEBUG = 2`. Every
`ndk/*/ketypes.h` says DEBUG=1, UNIPROCESSOR=2 — **the two were swapped**, and i386 and amd64
had them right. So arm64 published a `Prcb.BuildType` meaning the exact opposite of what the
source reads like: the UP kernel announced "checked MP" and the MP kernel announced "free MP".

Everything downstream believed it. `KmtIsCheckedBuild` was FALSE for the entire arm64 kmtest
corpus, so every test silently took its *free-build* path; and because a free build is allowed
to call `KeAcquireSpinLockAtDpcLevel` at PASSIVE (real NT compiles those `ASSERT`s out), the
KeSpinLock test did exactly that and our ungated assert answered with BUGCHECK 0x9. It read as
an SMP bug for a week — the MP kernel is simply where the smp-gated asserts live.

**Why the lint was blind, and this is the transferable part.** `scan_const_contradictions` finds
one name carrying two values, but buckets **by arch** — and the values were 1/1/2 across
i386/amd64/arm64, so no single bucket disagreed with itself. *A constant that is arch-invariant
in the C headers but wrong in exactly one arch's private copy is structurally invisible to an
arch-bucketed scanner.* Two consequences: when a constant family is defined identically in all
three `ndk/<arch>/` headers, a cross-arch disagreement between our copies is a **bug, not
variance**; and per-arch translations of one shared header are the place to diff our copies
against each other, not just against C.

**A swapped PAIR also survives every sanity check that looks at one name at a time** — both
values are present, both are plausible, both appear in the header. Only the *pairing* is wrong.
Diff the whole family in one read.

### A hand-typed bit-flag literal that names the wrong member `[§9.86]`
`DC_vPrepareDCsForBlit` tested `fs & 0x04` (**DC_CANCELED**) where the C tests
**DC_DIRTY_RAO (0x10)** — so every GDI blit drew through a stale clip, and an app caching
`GetDC` blitted into a rectangle belonging to a previously destroyed window. Symptom looked
like an intermittent render bug and the lane was parked as "clip disproven", because probes had
measured the vis region and `prgnRao` *correctly* at update time. **A hand-typed bit-mask
literal is the flag-cousin of the magic struct offset — treat it as wrong until proven right.**
The constants were always available via the bindgen glob; the literals were pure debris.
**Corollary: when a pipeline has N derived stages, probe the LAST consumer, not the producers.**

### Flavor-variant constants are their own drop-class `[§9.97]`
`SYNCH_LEVEL` differs between UP and MP builds, and the translation froze the UP value in
**five independent sites** → `-smp 2` `KeIrql` kmtest failures. Single source of truth per
crate, runtime-reachable for drivers. Same family: any constant whose C value depends on a
build flavour.

A **sixth** site surfaced on arm64 in 2026-07: the `aarch64 + feature="smp"` arm still read
`DISPATCH_LEVEL`, justified by a comment claiming "arm64's ndk has no SMP variant". It has one —
`ndk/arm64/ketypes.h` carries the same `#ifndef CONFIG_SMP` pair as i386 and amd64. The comment
was written from memory, propagated into three files, and then into the lane's quest doc as a
recorded blocker. **When a comment asserts what a header says, open the header** — this is the
same failure that once turned an untested inference about netapi32's MIDL wrappers into a
blocker that deferred the work twice.

### An undefined constant in a `match` arm becomes an irrefutable CATCH-ALL
**The most dangerous shape in this section**, because it is not an error: an unresolved name in
pattern position is a fresh *binding*, so it matches everything and **silently kills every
later arm**. It has shipped a BUGCHECK 0x1E, killed USB enumeration, and caused an OS-wide
`LCMapString` failure (a `switch` whose case labels became an or-pattern that swallowed a
neighbour).

**Upstream cause: an incomplete bindgen island** `[§9.112]`. A constant missing from the island
is exactly the one a dispatcher matches on. **An island's completeness is a correctness
property, not a convenience.** Prophylactic: allowlist the header that *owns* any constant a
`switch` dispatches on; run
`python -m rustconv.profile_derive <module> --emit report` — its ERROR findings are blocking.
Note constants that live only in a *secondary* `-sys` island need an explicit `use`
(rtl8139's `CmResourceTypePort` is in `hal_bindgen_*.rs`) — allowlisting the header does
nothing.

**The oracle history here is instructive and worth not repeating** `[§9.112a, §9.112b]`:
1. The checker reported ✓ 0 while **273 live catch-alls sat in the tree** — two scope defects:
   it scanned `ntoskrnl-rust/src` only (a hardcoded default), and it resolved constants
   *workspace-wide*, so a name defined in any crate counted as resolvable from everywhere.
   **"Has a glob import" is not evidence a name resolves.**
2. Then the 273 turned out to be **mostly the tool's own false positives** — unharvested
   `pub use … as Alias` re-exports; an attribute on the same line hiding the import
   (`#[allow(unused_imports)] use …`); an enum-variant regex that ate its own delimiter; and
   dead files scanned as live. **Net: 273 reported → 88 rustc-confirmed → 0 live.**
3. **`unreachable_patterns` is the oracle that actually holds** — validate a lint of this kind
   against rustc, not against itself. rustc also caught a class the tool could not see at all
   (a leading-underscore binding).
4. **Timing trap:** `verify_match_consts` runs at BUILD, not at check — and `cargo check
   --workspace` is not the build, which is exactly where this class hides.

**Fixing an arm is a real behaviour change, not a cleanup** — restoring a constant makes
previously-dead code live. Two transcriber rules `[§9.112c]`: **position-dependent enums differ
between headers** (`SCSI_NOTIFICATION_TYPE`), and **never define a value-alias next to its
sibling** (`IRP_MJ_SCSI` == `IRP_MJ_INTERNAL_DEVICE_CONTROL` — two names, one value, and a
`match` cannot hold both). When widening a scrutinee: **no narrow siblings in the match → widen
it** (`match (*req).EntryIndex as u32`); **siblings already narrow → join them, don't widen.**

### Deterministic prevention: SEAL the facts `[§9.117]`
`rustconv translate --seal-facts --seal-calls` takes constants and call occurrences **out of
the LLM's hands entirely** — a dropped or invented one then fails loudly instead of silently.
It had been gated off for no technical reason, at a cost of **13,503 unprotected constant
occurrences and 7,002 unprotected call occurrences**. Three value families, and the original
`[A-Z][A-Z0-9_]+` regex covered only the first: SHOUTY names; **CamelCase enum members**
(`NonPagedPool`, `KernelMode`, `Executive`); and **mixed-case object-like `#define`s**
(`NERR_Success`, `CmResourceTypePort`), which are sealed only when the classifier can prove the
definition is a constant. **Always pass both flags for plain C.** Note the interaction: sealing
restores constant *names* into `match` patterns, so island completeness matters **more** once
you seal, not less.

**Being the first caller of it surfaced three defects at once** `[§9.127]` — every one of which
would have fired for whoever followed the documented loop:
1. **Unsealing was gated on the COM path.** Sealing applied whenever the flags were set, but the
   only unseal call site lived inside the shell32/COM branch, so a sealed plain-C translation
   emitted raw `__RUSTCONV_FACT_nnnn__` tokens into the assembled module. **Sealing is a lexical,
   language-neutral transform, so unsealing has to be too.**
2. **A forward `extern` declaration counted as a call.** The envelope demands each token exactly
   once, and the prompt *orders* a second occurrence ("emit a forward `extern` block above the
   function"), so every sealed callee was named twice and every worker failed. Declaration spans
   are now excluded **from both sides**: a declaration neither creates a duplicate nor satisfies
   the requirement, so declaring a callee you never call still fails loudly.
3. **The resume cache banked un-unsealed Rust.** It guarded empty output but not sealed output, so
   a failed unseal cached its tokens and replayed them **for $0 forever**. Now refused on `put`
   and a MISS on `get`, so a poisoned cache self-heals.
4. **Not every call-shaped thing lowers to a call.** Same root as (2), new source. The seal set is
   `dep.callees`, and the call graph is lexical, so a **function-like macro** (`LOWORD`,
   `IS_INTRESOURCE`) and a **C++ functional-style cast** (`PPICK_ICON_CONTEXT(lParam)`) enter it as
   callees. Neither survives lowering — a macro expands inline, a cast becomes `as` — so the
   envelope demands a call that structurally cannot exist and the worker is rejected. In
   `EnumPickIconResourceProc`, **3 of the 4 sealed "calls" were not calls**. Diagnosis rule:
   *two different models failing the same function identically is a tool bug, not model
   capability* — deepseek and sonnet both failed iconcache's `SIC_IconAppend`/`SIC_Initialize`,
   which is what redirected the search from the model to the seal.
   **The general statement: the envelope assumes the Rust lowering preserves a 1:1 call-site
   correspondence with the C, and every C++ construct that must be RESTRUCTURED breaks that
   assumption.** Three independent triggers seen in one batch, so expect more: function-like
   macros and functional-style casts (above); `goto`-cleanup (Rust has no `goto`, so the cleanup
   block is duplicated and its tokens fire "not exactly once"); and ATL smart-pointer bodies
   (`CComPtr<IShellFolder>`, where `BrFolder_NewFolder` lost **all 15** tokens at once). The
   FACT envelope fails the same way — `BrFolder_OnContextMenu`/`OnInitDialog` dropped constants
   for the same reason — so this is not call-specific. Note the blast radius: a rejected worker
   means the TU assembles **no output at all**, so one bad function costs the whole file until
   re-run. In this batch that was **7 of 12 completed TUs** (shlexec, shlfileop, dialogs,
   brfolder, iconcache, drive, shlfolder) — the dominant cost of the wave, not a tail case.
   `_EXCLUDED` is a hand-maintained denylist and cannot scale to the SDK's macro surface; the fix
   is a positive authority test (seal only a callee provably a real function). **Three candidate
   authorities were tested and all three are dead ends** — check before reaching for them again:
   `catalog.functions` is target-internal (247 entries for shell32, holding neither the macros nor
   `SendMessageW`/`HeapAlloc`, so it would suppress sealing entirely); `scan.defines` is
   this-file-only and never sees an SDK macro; and `resolver.unresolved` lumps macros and real
   externs together (282 entries, `LOWORD` and `SendMessageW` side by side). The authority has to
   come from the **preprocessor's include closure** — a function-like `#define NAME(` visible to
   the TU.

**RESOLVED 2026-07-25 — all 7 blocked TUs assemble.** The authority is now
`rustconv/macro_authority.py`: it replays the module's own `compile_commands.json` entry with
`-E -dM` and takes every `#define NAME(`, unioned over built arches, and HARD FAILS with
`./vibeos-cli build --arch i386` when no compile command exists — "seal everything anyway" is the
degradation it exists to end. `_EXCLUDED` survives only as a floor for `sizeof` (a keyword, not a
macro). Four more causes surfaced once the macro noise cleared, and the split matters because two
of them were **ours**, not the model's:

- **A C lexer was masking Rust output.** `_code_mask`/`_masked_cpp` existed in three byte-identical
  private copies and every one treated `'cleanup:` as an opening char literal, blanking the text to
  the next apostrophe. A faithful `SHFileOperationA` lowering (three `goto cleanup` → three
  `'cleanup: loop`) had two call tokens fall inside the blanked regions and was reported
  "missing" — the model had written them. Now one shared `rustconv/lex_mask.py`, where `'` +
  identifier + `'` is a literal (C multi-character constants included) and `'` + identifier +
  anything else is a Rust label or lifetime. This masker also feeds shell32 COM worker
  normalization in four more places, so any `&'a` in model output was corrupting those too.
- **`goto` has no 1:1 lowering.** Rust has no `goto`, so every lowering duplicates code, and *how
  much* is not something the translation picks consistently: three consecutive paid responses gave
  the label's tail inlined 4×, the same 3×, and a nested cascade duplicating the whole remainder.
  A count-based budget (`#goto + 1`) was tried and the third shape walked through it, so the rule
  is positional — from the earlier of the first `goto` and its target label, a token may repeat.
  Only the DUPLICATE half is relaxed; `missing` and `invented` still fire.
- **The contract told the model a lie.** "Do not duplicate" is false for a `goto` body, and the
  models obeyed it: needing the cleanup tail on three paths and forbidden to repeat a token, they
  MINTED NEW NUMBERS past the end of the list (`dde_connect` invented three, `SIC_IconAppend` two).
  The contract now says repetition is expected when the body has `goto`, and never invent a number.
- **A type is not a value.** The FACT seal was sealing symbols that are types, where Rust is
  entitled to a different spelling: the parameter types of a body-local `extern RET Callee(TYPE);`
  prototype (`InvokeOpenWith`, dead on four consecutive attempts) and the template argument of a
  C++ named cast, `const_cast<LPWSTR>(strchrW(...))` (`dde_connect`). Both now skipped — the same
  judgement the scanner already made for a C-style `(TYPE *)value` cast, and the same judgement
  `seal_calls` already made about the callee name in that identical `extern` construct.

**Retry, because the blast radius is the whole file.** What survives the deterministic fixes is
sampling noise on the biggest bodies, and one rejected worker still assembles *nothing*.
`--retry-rejects N` re-rolls a rejected worker under identical acceptance checks (each retry is
billed); `iconcache` passed on attempt 3 and `shlexec` on attempt 2. Rejected replies are also
kept now, at `.responses/<key>/__rejected__/<fn>.rs` — never replayed, never assembled. Before
that the tool printed the verdict and dropped the evidence, which is the wrong half to keep: the
verdict says the envelope broke, the reply says how. Every diagnosis above came from those files.

> **Changing a seal rule no longer re-bills anything** (this reverses the earlier warning here to
> never reword the contract). The contract's token enumeration and the envelope PROSE are now
> normalized out of the resume-cache key: the enumeration is fully derived from (body, seal rules)
> and the body is already in the key, and the envelopes are DETECTORS whose verdict is binary — a
> banked entry passed, and its names were restored by rustconv at unseal time, not written by the
> model. What each worker actually had hidden from it is stored separately and compared
> DIRECTIONALLY: an entry sealed at least as tightly as this run wants is a hit, a looser one is a
> miss. Genuine capability blocks (the `--local-raii` API) still demand an exact match, which is
> the `CDrvDefExt` stale-replay bug and must not regress. Excluding one macro would otherwise have
> re-billed 6,800 lines of accepted, already-unsealed Rust; instead shlfileop replayed 65 of 73
> functions for $0. `translate --dry-run` now prints `Resume cache: N banked, M to buy`, so a
> cache invalidation is visible BEFORE it charges rather than after.

**A capability that has never been exercised is not a capability.** Grepping the ledger for
`seal_facts=True` on any non-shell32 module returned zero rows, and that single query predicted
all three bugs above.

---

## Dropped, degraded & silently-rewritten translation output

The model does not only get things wrong — it omits them. Every entry here compiles.

### Whole functions vanish
- **A parse failure in the SCANNER deletes a function from the plan, and every downstream check
  passes** `[§9.78]`. `#define CONST const` (winnt.h) is unknown to tree-sitter-c, so ws2_32
  shipped a 34-file, ~14 KLOC translation **with no `bind`, `connect` or `sendto`**. A dropped
  export is invisible to the scanner (it never saw it), to cargo (nothing references it), and
  to the link (the C original still provides it during cohab). **The oracle must come from
  outside the pipeline** — `tools/verify_lost_exports.py` audits the `.spec` against the Rust.
  Two shapes it must tolerate: a prototype with no body (ReactOS puts declarations at column 0
  too) and assembly implementations (`KiUserExceptionDispatcher` lives in `dispatch.S`).
- **The model silently skips a function entirely** (no file emitted), and **the `.responses`
  assembly glob drops 3-digit files** — a plain lexicographic glob loses `100+`. **Assert the
  assembled function count equals the `.responses` file count.**
- **Split-TU translation covers only the first chunk.** **Scanners skip macro-wrapped, SAL,
  callback and preprocessor-conditional definitions.** **Mid-size win32k leaves lost 25–50% of
  their functions.**
- **A stitcher comment-collapse drops whole bodies into a `//` line.** **A dropped function
  name becomes a bare tuple no-op.**
- **`verify_lost_exports` cannot see a ONE-LINE `#[no_mangle]`** — its regex is multi-line.
  **Diff the `.spec` export list against the assembled `#[no_mangle]` set before building.**

### An unexpanded Win32 macro silently shrinks the translation plan `[§9.128]`
`EXTERN_C` is `#define EXTERN_C extern "C"` (C++) / `extern` (C). tree-sitter knows neither, so it
parses the macro as the return type and the real return type becomes a second type-identifier.
Whether the function survives then depends on tree-sitter's **error recovery**, whose repair cost
is sensitive to token **length** — with a pointer return type the body is recovered at ≤9
characters and **lost at ≥10**:

    EXTERN_C FOOBAR*          -> extracted
    EXTERN_C FILEDESCRIPTOR*  -> lost

That is why one file reported exactly 2 of 66 bodies unextractable, both pointer returns with long
names, while every `VOID`/`BOOL` stub beside them parsed.

**Expand the macro; never blank it.** Blanking looks equivalent and is strictly worse: with no
linkage specifier the declaration becomes ambiguous and yields **zero** function-definition nodes
*and an empty unextracted list* — a silent drop, the exact outcome the scanner exists to prevent.
Measured: blanking recovered 1 of 3 bodies and reported 0 unextracted; expanding recovered 3 of 3,
with a 500-file differential showing 499 identical, 1 gained, 0 lost. Same family as the `CONST`
and `FAR`/`NEAR` normalisers. **When a scanner gap looks length- or name-dependent, suspect error
recovery and fix the INPUT, not the recovery.**

### Bodies that are present but hollow
- **Stubbed/degraded bodies** — constant-return or pass-through. **A variadic function degraded
  to a non-formatting stub.** **`RTL_CONSTANT_STRING` global UNICODE_STRINGs stubbed to
  empty.** **A cfg-gated stub left empty** → NULL thread-start 0xF4.
- **A cohab `rust_cohab_stub.c` that stubs a MACRO is a silent no-op, not a link error**
  `[§9.53]`. `SEH_TRY_EXCEPT` never called `try_fn(ctx)`, so fastfat directory enumeration
  returned **empty with STATUS_SUCCESS** → arm64 `FindFirstFile` failed for every FAT
  directory and Ctrl+Esc opened no start menu. `try_return` was a no-op the same way. Single-file
  opens worked (different path), which is the whole tell. **A stubbed macro cannot fail loudly
  — audit every cohab stub body for "does this actually call through?"**
- **An un-translated stub announces itself — wire that up.** fastfat is fully cohab-flipped, but
  one legacy C body was never translated, so arm64 bugchecked **0x23 after the marker** during
  first-logon profile creation `[§9.45]`. A `FASTFAT_STUB_HIT: <fn>` line on serial is an
  *un-translated stub being called*, not a kernel bug — make every stub say its own name.
- **A dual-implementation file ships the `#if !DBG` STUB branch** and the real body vanishes.
  **`#ifdef MACRO` mapped to `#[cfg(MACRO)]` is always the wrong branch.** **Arch-cfg branches
  flattened to the desktop path.** **A flip omitting a C `#define` redirect/HACK** compiles,
  greens the serial, and renders wrong.
- **Data tables lose ENTRIES**, **static-table complex-literal fields get nulled**, and
  **a self-referential static initializer becomes `{null, null}`**. `rustconv` drops **all
  non-fn header content at scale** (structs, const tables, `#define`s, FuncTables), so a
  header-only driver comes back unimplemented.
- **`table_audit.py`** compares every file-scope C data table against the Rust; **`vtbl_gen.py`**
  generates COM vtbls from the C initializer plus bindgen; **`export_audit.py`** requires every
  `.spec` export to exist as a correctly-decorated `#[no_mangle]`. Use them; don't ask an LLM.

### A `#include`-built table has a different ROW SET under the wrong preprocessor config
The C does `static const T tbl[] = { #include "Data.h" };` where the rows are `#if`-guarded on
build macros. Preprocess with the wrong config and you get a different row set — compiling,
linking, silently wrong. `_WIN32_WINNT=0x502` instead of the module's real `0x600` flips every
`NTDDI_VERSION < NTDDI_LONGHORN` guard; forgetting `-D_SHELL32_` drops its rows. shell32's
`s_PolicyTable` came out 169 rows; the real build is **159**. **Verify against ground truth,
not a hand re-preprocess:** `strings -el <the pre-flip .c.obj>` shows exactly which literals
the real build compiled in. A bare `clang -E` with guessed `-D`s is how the bug arose.

### Never-executed translation code accumulates bugs, and the first execution finds them ALL
Bringing up a path nothing had exercised is not "fix the bug" — it is **fix the stack of bugs**.
WHPX x86 was shelved because the AHCI boot CD was dead end-to-end; it took **six stacked
never-executed-translation bugs plus firmware gaps** `[§9.46]`: uniata's Intel-AHCI path dropped
unconditionally; freeldr UEFI emitted no `"PCI"` MultifunctionAdapter node; hal read 32 config
bytes instead of `PCI_COMMON_HDR_LENGTH`; OVMF doesn't write per-device InterruptLine on all
builds; OVMF never programs the ELCR (so PIRQ-routed IRQs stayed edge-triggered); and **the
killer — hal `pic_pic.rs` extern-declared `KiI8259MaskTable` (an array) as a pointer**, the
array-vs-pointer class again. **Budget for a stack when un-shelving a path, and do not read bug
N+1 as evidence that the fix for bug N was wrong.**

### Semantic rewrites that flip meaning
- **`if (!Fn())` → `if !Fn() != 0`** — bitwise-NOT on an integer BOOLEAN makes the condition
  **always true**. Recurs wherever a C macro yields an `int`: `if(!MSVCRT_CHECK_PMT(x))` is
  `if X!(x) == 0`, never Rust's `!`. Loud when the result feeds a condition (`expected bool`),
  **silent** when it feeds an integer.
- **C post-inc/dec rewritten with the wrong read order** (`x = y--` reading the post value).
- **A post-decrement INSIDE a loop condition — `while (a && b && len--)`.** The decrement is
  part of the test, so the loop exits having *already* wrapped `len` to `SIZE_MAX`, and the
  `if (len && ...)` epilogue that follows fires on that wrapped value. A Rust `while len > 0`
  with a separate `len -= 1` reads like the same loop and is not: `len` stays 0 and the
  epilogue never runs. `_mbsnset_l` silently dropped its truncation this way. Whenever a C loop
  decrements in its condition, check what the code *after* the loop reads.
- **A C parameter reassignment translated as a block-local `let` SHADOW** — the new value dies
  at the block end. The same bug wearing a loop: **the C walks the PARAMETERS and the code after
  the loop reads what the walk left behind.** `_mbslwr_s_l`/`_mbsupr_s_l` looped over `let mut
  s = s` copies scoped inside a branch, so afterwards they tested the *original* `len` (making
  the buffer-too-small `EINVAL` unreachable) and wrote the terminator at the *original* `s`,
  i.e. the start of the string. `let mut s = s;` at function scope is the fix; the tell is any
  statement after the loop that mentions a parameter the loop advanced.
- **An out-parameter store made CONDITIONAL on success.** C's
  `Dest->Buffer = alloc(...); if (!Dest->Buffer) return STATUS_NO_MEMORY;` stores the failure
  into the caller's struct; the natural Rust — `let buf = alloc(); if buf.is_null() { return
  ... } (*dest).Buffer = buf;` — returns *before* the store. The caller is then left holding
  its own stale pointer beside the `Length`/`MaximumLength` the function just overwrote: a
  string that looks live and aims at foreign memory. Eleven `Rtl*String` converters had it.
  Whenever C assigns through an out-pointer and *then* tests what it assigned, the store is
  part of the contract — keep the order.
- **A guard condition swapped for a different nearby variable** — logic inverted per boot mode.
- **A NULL guard De Morgan'd the wrong way.** `MSVCRT_CHECK_PMT(start && str)` came back as
  `MSVCRT_CHECK_PMT!(start.is_null() || str.is_null())` — the *negation* of the C, so the guard
  fired on VALID pointers and passed NULL through to the deref it exists to prevent. Compiles
  clean. Read every translated parameter-validation guard against its C.
- **A C `int` local holding a `size_t` return, kept `usize` in Rust — the error path goes DEAD.**
  `int ch_len = mbrtowc(...)` is not sloppy C: the narrowing to `int` is what makes `mbrtowc`'s
  `(size_t)-1` visible as `-1`, so `ch_len < 0` catches a decode failure. Translate the local as
  `usize` and that branch is unreachable; `mbsrtowcs_s` walked past failures instead of
  returning -1. rustc says **"comparison is useless due to type limits"** — treat that warning
  as a defect report, never as noise.
- **C integer promotion dropped at a mask.** `ch & 0xf8` promotes `char` to `int` first, so a
  high-bit byte sign-extends and still matches `0xf0`; masking at `i8` width does not (and does
  not even compile — `literal out of range`). Same class as the `BYTE`-returning `HIBYTE`/
  `LOBYTE` macros, whose results C promotes to `int` before any arithmetic.
- **A narrowing cast truncates a wider C operand**; **a u64 sentinel cast as i64 inverts a
  bounds check**.
- **`_Analysis_assume_` translated as live code** → a NULL deref the annotation existed to
  assume away.
- **A facility-status return convention confused** — `return 0` where SUCCESS is a nonzero
  facility code (hidparser).
- **A ported tree/list rebalance with an untested special case** (predecessor = direct child)
  self-corrupts its links; found only by a random model test.
- **A lock/RAII-pair epilogue dropped when a macro was inlined.**
- **`addr_of!(field)` where C passes the pointer field's VALUE** — the buffer becomes struct
  innards (win32k clip-region 0x50).
- **A plain Rust-ABI `unsafe fn` address handed to the kernel** (timer/dispatch callback).
- **A hand-rolled `va_arg` walking `&last_fixed_param`** is amd64-only garbage.
- **Kernel Rust emitting x87 (`fild`/`fistp`) for 64-bit atomics** → an imprecise 0xc0000092
  BSOD blamed on an innocent function.

### Element count vs byte count — the recurring arithmetic bug
- **`write_bytes(ptr, v, size_of::<T>())`**: the count is in **ELEMENTS**, so a typed pointer
  zeroes `sizeof(T)²` bytes (kernel32 `QueueUserAPC`). `memset`'s count is in bytes.
  Gate-checked by a `WRITE_BYTES_SIZEOF` oracle.
- **A VARIABLE byte count on a `*mut u16`** is the same bug and the oracle **cannot see it** —
  `UNICODE_STRING.Length` is in bytes, so `write_bytes(buf_u16, 0, len)` zeroes 2× too much.
  I tried to lint this variant and **reverted it**: a tree-sitter scanner has no type info and
  caught 0 true positives. The mitigation is manual review at every `write_bytes` on a `u16`
  pointer.
- **`copy_nonoverlapping`/`copy` have the same element-vs-byte trap** (the `RtlCopyMemory`
  sibling).
- **Cast-before-offset**: `(ptr as *T).offset(1)` overshoots by the size delta → heap overrun.
  **A ZST-array pointer `.add()`** steps 0 bytes.
- **A dangling pointer to a by-value `transmute` temporary** — the WCHAR[] → storageless-
  temporary trap.
- **A literal size where a `sizeof` belonged** — cdrom's media-change SRB used `32` for an
  18-byte sense buffer → deterministic i386 `BAD_POOL_HEADER 0x19 (0x5)` at driver init,
  TCG-only timing `[§9.100]`. Useful trick: **the 0x19 parameters encode `line!()` as param3**,
  which maps the overrun straight back to its source line.
- **C's legal one-past-the-end idiom** trips `unconditional_panic`, **which only fires on
  `cargo build`, not `cargo check`** `[§9.111]`. The C is not a bug; the Rust index is.
  **A translated crate is not validated until it has been BUILT.** Grep every `[CONST]` index
  against the array's declared length. `unconditional_panic` is a MIR-level deny-by-default lint,
  so it needs codegen — and **an acceptance oracle that runs `cargo check` is not an acceptance
  oracle** `[§9.129]`. End-to-end on one file: check-only reported *1/1 candidates compile on
  every arch*, a real build reported *0/1, unconditional_panic*. A correct candidate of the same
  shape (`entries.as_ptr().add(i)`) still passes all three arches, so this is not over-strict.
  Record which pass produced a green so the verdict says what it means.

### `n`-suffixed CRT functions must never contain an unbounded scan `[§9.61]`
The bug a value-comparing fuzz **cannot** see: right answer, wrong memory. Rust `_strnset`
scanned to the NUL then clamped to `n` — the clamp makes the *answer* right while the *access*
is already out of bounds. It regressed the whole net suite while every differential test said
MATCH across 20,000 cases and all three smokes were green. The trap: msvcrt is user-mode and
the failures were *kernel-side* netstack self-tests, so "not my code" was very tempting. Also,
the first grep for callers found none — **because the grep was truncated by `head`**; it is
called from inside msvcrt itself. **Bound the scan by `n`; never scan-then-`min`.**

### A slice-based core cannot express a single-pass C string primitive `[§9.63]`
Wrapping a safe Rust core requires a `&[T]`, which requires a length, and computing the length
**is** the scan you were trying to avoid — so a byte-perfect flip is silently 2× the work.
Transcribe the copy/compare/search families directly against their C bodies. Gate lessons:
**a perf gate only defends what it benches**, and **bench best-of-3, not the mean** (this host
runs other agents' builds and QEMUs).

### Reading the C wrong `[§9.77]`
- **Never read C through a comment-stripping grep.** It ate `*_errno() = ...` — the second time
  the same filter has eaten load-bearing C (the first was `*p++ = *src++;`).
- **`#if defined(_WIN64)`: a host-compiled reference silently takes the i386 arm.** Grep every
  new flip *and everything it calls* for `_WIN64` before writing Rust.
- **Model fidelity differs measurably:** DeepSeek output is markedly buggier on constants than
  Claude. For C++ COM, the model owns bodies and **never** interfaces, vtables, offsets,
  lifetime, or activation `[§9.78a]` — `rustconv cpp-com` derives the COM map from the defining
  header.

---

## SEH, PSEH & exception handling

### DUMMY_PSEH turns a protected capture into an unprotected `memcpy` `[§9.59]`
A whole class, not three bugs: in C, a kernel API captures a caller buffer inside `_SEH2_TRY`;
under DUMMY_PSEH the `__try` evaporates and the copy is naked → several unrelated syscalls
BUGCHECK 0x1E `0xC0000005` at the *identical* PC. The insight that makes the fix work:

> **A copy is a CALL.** Put the faulting access in a CALLEE and clang's "calls-only" native SEH
> scope table covers it.

Order matters — let `MmAccessFault` run **first**, so a valid-but-nonresident page simply pages
in and the copy proceeds; only a truly bad address reaches the handler. **Do NOT "fix" this
with `MmIsAddressValid`** — it is the tempting portable stand-in and it is wrong (it answers a
different question, racily).

- **i386 got a fault-tolerant kernel copy WITHOUT fixing SEH** `[§9.123]` — a Linux-style
  exception table is ~40 lines and needs no unwinder. `rep movsb` is **one restartable
  instruction**, so the table has exactly one entry; the check runs *after* `MmAccessFault`
  declines; and it cannot swallow a fault it doesn't own (one exact PC, kernel-mode only).
  This cleared three stuck kmtest quarantines that were one missing primitive, not three bugs.
- **`_SEH2_EXCEPT(...)` emitted as an UNCONDITIONAL bare block** runs the handler on success.
  **Eager filter evaluation** additionally feeds the provable-null-deref ud2 fold.
- **Do NOT delete `_SEH2_TRY` "dead-code" wrappers** — they are an LLVM optimisation barrier.
- **DUMMY_PSEH defeats cdfs's raise→catch→post async model** → 0x1E `STATUS_CANT_WAIT`.

### A `__try`/`__except` translated to a BARE BLOCK — the flag survives, the handler doesn't `[§9.133]`
**The shape to recognise**, and it is worse than an obviously-missing handler because a reviewer
skimming for "is the failure handled?" *sees* a handler:

```rust
let mut exception_occurred = false;   // never assigned, anywhere
{                                     // <- this was the C's _SEH2_TRY
    ProbeForWrite(ApcFrame, size_of::<UAPC_FRAME>(), 16);
}
if exception_occurred {               // <- dead code, provably
    KiDispatchException(...);
}
```

The translation preserved the *silhouette* — a flag plus the `if` that reads it — while the `__try`
degenerated to a bare block and the `__except` body evaporated. It compiles, reads as if it
recovers, and cannot: nothing writes the flag, so under DUMMY_PSEH the raise propagates out of the
kernel and kills the box. **Grep for the tell: a `let mut <flag> = false` whose only other
occurrence is an `if`.**

**Fix** (convert the raise contract at the callee): validate first with a `Result`-returning probe,
then run exactly what the C `__except` ran — do not invent new control flow, and leave the C's
post-block statements where they are. The `'try_: { … break 'try_; }` idiom for this was **already
in the sibling arch's file**.

> **When one arch of a per-arch file is already converted and another isn't, that asymmetry IS the
> bug report.** Check siblings first: here i386 was correct, arm64's raising probe was cfg'd out,
> and amd64 was the only one left.

Three method lessons from the hunt:
- **A raise with no diagnostic is a guessing game — instrument before fixing.** The probe's
  access-violation arm printed *nothing*, so the bugcheck gave a function name and nothing about
  *what* was probed. One trace line answered it in a single boot, and **size and alignment identify
  the struct while the arithmetic identifies the caller** (an address of exactly `-sizeof(frame)`
  means the caller's stack pointer was zero). Report the two raise conditions *apart* — a garbage
  length and a bad pointer point at opposite bugs.
- **The instrument refuted the leading hypothesis, which is why it went in first.** Both victims
  were 32-bit guests, so a wrong-width thunk looked obvious. It wasn't, and one line of output
  settled it before any code was written against the wrong theory.
- **Before blaming the process that was running when the box died, check whether it merely
  outlived the one that killed it.** The real culprit was a third process entirely; the two
  apparent victims were downstream of the box-kill, which is why both looked guilty.

### An SEH filter expression is a PREDICATE, not a value `[§9.70]`
`GlobalLock` materialised `EXCEPTION_EXECUTE_HANDLER` into a local and branched on it, so it
**returned NULL unconditionally** — and every edit box in the OS was dead (Notepad, every
dialog), because `EDIT_LockBuffer` left `es->text == NULL`. It stayed hidden because both
callers *look* like they check for failure and both quietly absorb it. **Grep any translation
that stores an `EXCEPTION_*` disposition in a variable.**

### `NT_ASSERT`/`ASSERT`/`PAGED_CODE` are DIAGNOSTICS, not control flow `[§9.120]`
In retail C they compile to **nothing**; under DBG they are `KdPrint` + `RtlAssert`. None is
`__assume(0)`. Three translations lowered them to worse-than-C:
`core::hint::unreachable_unchecked()` (**UB** — licenses LLVM to delete the guarding check and
poison what follows, strictly worse than retail C in every configuration), a hard `assert!`,
and a `BUGCHECK` with an invented code. The hard `assert!` in fastfat's `FatLocateDirent` lay
dormant until `IRP_MJ_CLEANUP` actually started being delivered — **a correct fix elsewhere
woke a latent panic** `[§9.57]`, arm64-only.
- **When auditing a translated assert, ask what the C does in RETAIL, not what it means.**
  `ASSERT(x)` *reads* like "x must hold", which invites a hard failure; it compiles to nothing.

**Check the direction PER SITE — this class errs both ways, and neither fix may be a blanket
sweep.**
- **Too weak** `[§9.135]`: **1962** `debug_assert!` sites across `sdk/lib/rust` are dead code.
  `debug_assert!` is gated on `cfg(debug_assertions)`, the workspace `[profile.release]` never sets
  `debug-assertions`, and every kernel crate builds `--release` — while CMake sets `-DDBG=1`, so
  the C `ASSERT` these came from calls `RtlAssert` and fires loudly. Proven by execution, not
  inference: an assertion forbidding `SegCs == 0` was walked straight past by a boot that printed
  `SegCs=0x0`, and had it been live it would have caught a blank trap frame **at its source**
  instead of surfacing later as an unhandled probe raise that bugchecked the box. **Before flipping
  it**, set `overflow-checks = false` *explicitly* (it defaults to following `debug-assertions`, and
  kernel-wide arithmetic panics are a separate, much riskier change C never had), then measure —
  1962 assertions going live at once needs a triage pass across all three arches before it lands.
- **Too strong, but do NOT silence it** `[§9.136]`: four C `ASSERT`s in `ARM3/pfnlist.rs` became a
  live `KeBugCheckEx` with an **invented** subcode, and the guard fired on arm64 only. That file is
  **arch-generic** — one file, all three targets — so a guard firing on exactly one arch from shared
  code is not measuring the guard, it is measuring a **real arch-specific PFN divergence**. Deleting
  it to "restore C fidelity" would convert a loud arm64 fault into upstream's silent continue.
  > **When an assert-derived check fires on ONE arch from ARCH-GENERIC code, the finding is the arch
  > divergence, not the assert.** `git grep` the file's `#[cfg(` count answers "is this shared?" in
  > one command.

  Two separable defects needing opposite treatment: the arm64 PFN state bug is real and stays open
  (same family as the other arm64 PFN-database leads — treat them as ONE lead, not three), while
  the invented subcodes `0xA0`–`0xB7` are a §9.113 fidelity bug that **must not be renumbered while
  the real bug is open**, because they are currently the only way to tell which assert fired. Note
  the same file *preserved* its two genuine bugchecks, so "the translator mangles all bugchecks"
  would have been the wrong generalisation too.
- Gate: `tools/scan_assert_to_ub.py`, empty baseline. **The gate that should have caught the
  third one reported a clean zero** — see the oracle section; a lint that under-matches is
  indistinguishable from a lint that passes.
- **A C debug macro that does NOT block, translated as `loop {}`**, hangs the boot. Read what
  the macro expands to.
- **`DPRINT` is NDEBUG-dead in C**; translated as a live `DbgPrint` it is a boot slowdown that
  masquerades as a hang.

### The standalone kmtest drivers' own `_SEH2_TRY` was dead code `[§9.68]`
Three amd64 "kernel crashers" were **the kernel behaving correctly** — the *test* had no
working SEH. Fixed with `kmt_enable_native_seh()` applied to **all 17** drivers, not just the
faulting ones, because the invariant is "kmtest drivers have real SEH". **The negative result
is the valuable half:** it exonerated `MmMapLockedPagesSpecifyCache`. **When a test harness and
the kernel disagree, suspect the harness first if the harness is younger.**

### PSEH hand-lowering recipe
For FS-driver translations, `_SEH2_TRY`/`_FINALLY`/`_EXCEPT` lower to a `'seh:` labelled block
returning `Result`, with the finally path on both exits. `try_return` must actually branch.
---

## Stale artifacts & false greens

**The most expensive class in this file**, because the failure mode is a green that means
nothing. It has false-greened arm64 six times in a row (fastfat → ramdisk → pl011 → gdi32 →
user32 → advapi32). Rule: **`vibeos-cli smoke` hard-fails on stale media — never bypass it, and
judge a build by artifact existence, not a reported exit code.**

### The cargo kernel does not relink on a `.rs` edit `[§9.6b]`
`ninja livecd` silently ships the previous kernel: the cargo edge's DEPENDS list doesn't see
your file, so the command never reruns. **Detect:** compare `ntoskrnl.exe` mtime against your
edit. **Force:** nuke the cargo *target dir* (`sdk/lib/rust/target/<arch>-kernel`), not the
exes. **Applies to all three arches.** Driver/usermode **cohab** crates are unaffected;
cargo-**driven** `reactos_link` drivers ARE affected via a separate target dir — `ninja
<driver>.sys` reuses the stale body rlib.
**Corollary: env-only toggles never re-run a cargo edge** (ninja watches files, not env). Touch
the `build.rs` and **verify the output md5 actually changed** — the first layout-canary attempt
no-op'd twice.

### A cargo compile error leaves the prior rlib, and the link succeeds against it `[§9.87]`
Added a `DbgPrint` to win32k, rebuilt, booted — the print never fired. Three times. One file had
an `error[E0412]: cannot find ULONG`, but the `ninja` invocation was piped
(`... | grep -E 'error|Finished' && ...`) and `Finished` matched from another unit.
- **Never trust a piped or `&&`-chained ninja "Finished".** Grep for `error[` AND `FAILED`, and
  read the per-crate `Compiling <crate>` lines.
- **After a probe that MUST appear on serial, verify the literal is in the binary before
  booting:** `strings <module> | grep -c '<my unique literal>'`. Zero = stop and fix the build.
- **"Other edits in the crate shipped" does not prove YOUR file did** — cargo is per-file
  incremental and one file's error doesn't fail the archive.

### The arm64 staged build rebuilds too little `[§9.6c, §9.6c.1, §9.6c.2]`
The staged smoke set (`ntoskrnl`/`hal`/`uefildr` + `mkramdisk`) does **not** rebuild drivers.
Two consequences, escalating:
- **A kernel ABI change needs ALL drivers rebuilt** or arm64 wedges — sometimes *early*, where
  it impersonates an unrelated bug (a deterministic wedge at `KiInitializeUserApc done`).
  **Never gate an arm64 link/layout or static-lib change on a targeted-subset build.**
- **A boot-critical driver MISSING from the staged set false-greens the gate on a STALE
  pre-cohab-flip binary**, so the committed Rust translation is *never boot-exercised* — for
  weeks. When `ramdisk` was finally rebuilt it bugchecked 0x64 at the first file open, then
  0x23 because `IOCTL_DISK_CHECK_VERIFY` collided with a wrong constant. **x86 is immune only
  because `ninja livecd` repackages the ISO.**
- **Now mechanized** (`provenance.cohab_staleness_reasons` + `build.py`'s
  `boot_set_cohab_rels()` self-heal) — the net guards 20 arm64 cohab boot binaries. mtime, not
  content-hash, because the gate flow is build→smoke back-to-back.
- **A green cargo build does not imply boot-tested.** Only a clean boot of the actually-rebuilt
  `.sys` does.

### The two staleness oracles, both were wrong `[§9.81]`
- **(a) `cohab_staleness_reasons` false-REDs on a codegen-neutral edit** `[§9.81a]` — the rule
  `artifact.mtime >= newest .rs mtime` is a proxy, and a comment-only edit trips it. **A false
  RED is not the safe direction**; it is the mirror of the false green. Fixed by splitting the
  chain where mtime semantics change: **(A)** cargo's own output ≥ newest `.rs` (proves the
  crate rebuilt), **(B)** artifact ≥ the *copied* staticlib (proves the link consumed it).
- **(b) `_ninja_has_work` returned False even when a rebuild WAS pending — on every arch**
  `[§9.81b]`. So
  **editing any C file and running smoke booted the stale ISO and called it a pass**, for as
  long as the oracle existed. Fixed with `build_src_hash()`, a git content fingerprint. Two
  design rules: it is an **EXCLUDE-list** (`docs/`, `*.md`, `.github`, `.claude`), never an
  include-list; and **an absent stamp field is LOUD, not clean.**
- **Blind alley: do not "fix" (a) with `ninja -n`** — that swaps a false RED for a false GREEN.

### A lint that reads a BUILD ARTIFACT has a non-portable baseline `[§9.115a]`
`marshalgen audit` read a build artifact, so a **stale build dir** made it re-report 33
already-fixed syscalls — and because its output feeds a **checked-in baseline**, they were
re-blessed as accepted debt. Worse than an ordinary flake: it *un-fixes* landed work on paper.
**Fix, both parts needed:** derive the table from checked-in sources (`.spec` + headers), and
still read the artifact — but only to prove it **agrees** (`crosscheck_artifact()`).
Generalises to the fleet-isolation rule: **never resolve generated artifacts from a sibling
worktree.** `verify_signatures` once picked a sibling's `build-amd64` and its mismatch count
moved **987 → 9 with no source change**. A verdict that depends on which siblings exist is not
a gate. Pinned *toolchains* may be shared (content-addressed); generated *headers* are
per-commit state and must come from THIS worktree or the tool hard-fails.

### A build-time flavor knob that keeps the SAME output name defeats every name-based check
**Symptom:** a run is green and the screenshot is a full desktop, but the binary under test is
not the flavor you think. **Cause:** a cmake cache var that changes what a binary *is* without
changing what it is *called*. `VIBEOS_SMP_HAL=1` rebuilt the SMP HAL — into a file still named
`hal.dll`. Nothing downstream can tell the flavors apart: freeldr registers whatever it loads
as `hal.dll` in the loader block, the on-disk name is identical, and on arm64 there is no
`MISMATCHED_HAL` handshake, so an MP kernel on the UP HAL boots to a desktop and silently runs
on **one cpu** (`HalStartNextProcessor` is the FALSE stub). The only available assertion was a
*negative* runtime marker ("did the UP stub get called?"), which only fires if the wrong code
happens to execute. A build-dir-global knob also forces a full relink to switch (9m25s i386 /
12m22s amd64 warm), so nobody runs both flavors routinely — the check that would catch it is
the one you can't afford. **Fix: make the flavor the file name and ship both.**
`halmp-cargo` → `halmp.dll` beside `hal.dll` (mirroring `ntoskrnl-cargo` → `ntkrnlmp-cargo`),
selected per boot entry (`/HAL=`). The assertion becomes positive and load-time — "the
`[FLDR] load` stamp names `halmp.dll`" — and the mis-pair stops being reachable. **Corollary
for the second copy:** put the shared recipe in ONE place (here a `hal-link` crate both cells'
`build.rs` call) — two hand-maintained link recipes for binaries that must be identical apart
from one feature is the same single-copy-divergence class as a constant defined per-arch.

### Other staleness shapes
- **Multi-copy staleness** `[§9.6]` — `rtl`/`heap` links into **both** `librtl.a` (ntoskrnl) and a
  separate ntdll copy. An edit that "doesn't take" is a stale archive. `ar` never prunes, so
  also `rm` the copied `.a` and any stale C archive.
- **A shared `librtl.a` in both ntoskrnl AND ntdll** means kernel-VA reads fault in user mode
  (silent idle hang), and its alloc stubs return NO_MEMORY in the live kernel — needs a 3-mode
  gate.
- **`#[no_mangle] pub static mut` in a shared lib gives each consumer a PRIVATE data copy.**
- **A dual-archive cohab runs STALE Rust** when the consumer's DEPENDS list doesn't mirror it.
- **Editing a kernel C shim silently ships the stale object** (no `rerun-if-changed`).
- **Smoke "green ×3" lies when conftest nukes only `rust-kernel-target`** (there are five).
- **New trace points produce nothing in serial** → stale `livecd.iso`.
- **Smoke red right after a merge/handoff is usually a stale worktree, not a regression.**
- **A reconstructed `build-arm64` dir → perpetual CMake reconfigure → permanent false
  `STALE_MEDIA`** `[§9.6e]`. Not a code signal. `mkramdisk.sh --refresh` also assumes a prior
  full build populated the media tree.
- **A bad `mkramdisk` run hangs arm64 late boot** `[§9.6d]` — serial reaches csrss → user32 →
  winsrv → mpr → `ufat.dll`. Regenerate (`--refresh`) and re-smoke **before** suspecting code.
- **arm64 "uefildr StartImage: Unsupported" is a CORRUPTED `qemu/efivars.fd`**, not build drift
  `[§9.6f]` — the `…start failed: Unsupported` line is EDK2 noise present in *every* boot.
  Restore a known-good `efivars.fd`. A full `rm -rf build-arm64` (vs `--clean`) also exposes
  pre-existing gitignored-asset gaps.
- **A stale C `.obj`/`.a` member survives a merge** → false cold-`LINK_FAIL` on a `.c` that is
  commented out. Same for the freeldr `freeldr_pe`/`uefildr` cold dup (`_PcCons*`/`_Mach*`).
- **A workspace-composition change false-FAILs a warm-incremental cohab smoke.**
- **Cold build from a fresh worktree: generate psdk/xdk headers BEFORE the cargo crates.**
- **A cold build fails `encountered diff marker` in a crate another lane owns** — that is a
  merge artifact, not your bug.
- **`rustconv translate` assembles into the LIVE crate `src/`, which fails a concurrent gate**
  `[§9.132]`. The staleness check globs every `.rs` under the crate against the cargo-linked
  binaries, so translating while a gate or land is in flight fails the build for a completely
  unrelated reason — and the offending file is **untracked and referenced by no `mod`**, so cargo
  would never compile it. `git status` stays clean (new files are untracked, and `land`/`gate`
  sample dirtiness with `--untracked-files=no`), so nothing warns you: the tree looks landable and
  the build fails ten minutes later. **Translate to staging while anything is gating**
  (`-o ~/vibeos-rust-staging/<lane>/<file>.rs`) and integrate when you are ready to build; paid
  output is preserved regardless, since `.responses/` is written before assembly.
  Cleanup lesson from the aborted land: killing `land` released the FIFO lock, but its `gate` child
  was **reparented, not killed**, and held build permits for a verdict already discarded. **Kill by
  explicit pid after confirming ownership** (`/proc/<pid>/cwd`, `vibeos-cli host-status`) — two
  `ninja` processes at the canonical build path belonged to a *sibling* lane that had just taken the
  lock, and a pattern kill would have destroyed their work.

### Mainline can carry cold-build breaks while every lane's gate is green `[§9.126]`
`origin/master` did not build from scratch — **three independent regressions at once**. The
mechanism is not subtle once stated: **a compiler cannot report an error in a translation unit
it does not compile, and a linker cannot fail on an image it does not relink.** A warm gate is
structurally blind to any break in a crate it doesn't touch, and **nothing in the failing crates
had changed** — a pre-existing use site simply became ambiguous. Two roots: an **E0659 pair
where a bindgen island grew a name a local shim already had** (two different items under one
name via two globs), and a `__debugbreak` declared `extern "C"` instead of emitted (it is an
MSVC *intrinsic*).
- **Cold-build breakage comes in LAYERS** — fixing one exposes the next. Expect it.
- **Scan the NAMES, not the import shapes.** The first blast-radius scan looked for files
  globbing the island and produced 16 false positives while missing the real second site; the
  correct scan was for **the names the commit added**.
- **This is why `gate` builds cold.** It costs ~16m vs ~2-3m warm — affordable amortized,
  because the alternative is every lane independently rediscovering the same break. Two lanes
  hit and fixed the same `__debugbreak` break within an hour.

---

## Your oracle is lying: harnesses, lints & generators

A whole family of bugs where the *verification* is wrong, so the code looks correct. These cost
more than ordinary bugs because their output is trusted.

### The governing rule: a harness only falsifies what it varies
> **A harness proves parity only across the axes it actually VARIES. Every axis it holds
> CONSTANT is a silent assumption baked into every MATCH it prints.** `[§9.60]`

Before believing a verdict, enumerate what the harness holds fixed. Two blind spots that shipped
bugs: **ambient state the reference holds fixed** (locale — grep a flip candidate for
`get_locinfo` / `_l`-suffixed helpers / any read of process-global state), and **the reference
TU is compiled for the HOST**, so target-conditional `#if`s are graded on the wrong arm.

A third: **an out-parameter the harness only PARTIALLY compares.** rtl-difftest asserted
`(Length, MaximumLength)` on the STRING converters but not `Buffer`, so an eleven-site
divergence — the alloc-failure path storing nothing where the C stores NULL — was visible in
exactly the one test that happened to compare all three fields. Ten of eleven sites were graded
green by a suite that never looked at the field carrying the bug. **Compare the whole
out-parameter, or write down which field you dropped and why.** Pointer values legitimately
differ between the two sides, but `is_null()` does not — grade the property, not the address.

And after fixing anything a harness caught, **re-break one site deliberately and confirm the
suite goes red.** Widening an assertion is itself untested code: it is easy to add a field to a
tuple that can never disagree, and the result reads exactly like a stronger test.

**An IN-OUT cursor needs its own cell per side, compared by offset — and NULL is not offset 0.**
`_mbstok_s(str, delim, ctx, …)` and `mbsrtowcs(wcstr, pmbstr, …)` report where they stopped by
writing a pointer back through a `char **`. Three ways to grade that wrong, in increasing
subtlety: hand both sides ONE cell (the reference then resumes from wherever the subject left
off and agrees by construction); compare the two pointers (they address separate copies, so
they differ on every iteration); or map NULL to offset 0 (`mbsrtowcs` writes NULL to mean
"consumed to the terminator" and a real position to mean "stopped here" — collapsing them grades
two different outcomes as one). Offsets into each side's own base, with NULL as its own value.
Also seed the buffer: a cursor over a mutable buffer means the call operates ON it, whatever
else the signature looks like — `_mbstok_s`'s `delim` reads as a source string to a
"has-a-source ⇒ the buffer is a destination" rule, which hands it a blank buffer, and both sides
then tokenize an empty string and agree about nothing.

**A zero-argument accessor returning a pointer has no input to measure against — grade the
CONTENTS.** msvcrt's `__p_*` family (`__p__mbctype`, `__p___mb_cur_max`, a dozen more) hands
back an interior pointer into CRT state, and each side reads its own fixture slot, so the
addresses differ permanently. Dereference and compare, checking NULL-ness first and separately.
For a table, get the length from `sizeof` on the struct mirror, never typed out: too small
silently stops comparing the tail, too large reads past the struct, and **both are invisible in
the verdict.**

**Padding an input string is not enough — the padding needs a terminator.** msvcrt's MBCS
walkers step past the NUL on a malformed string: `_mbscmp("ab\x82", "ab\x82")` under cp932 treats
the trailing lead byte's terminator as its trail byte and advances TWO, resuming past the end.
The corpus already padded both copies identically so the two sides would read the same bytes
there — but the padding itself had no terminator, so a walker that consumed it left the
allocation and read heap. It sat latent for as long as the allocations happened to agree, and
surfaced as the subject and the reference disagreeing about two IDENTICAL strings, provoked by
an unrelated pair of cases shifting the heap. Pad, then terminate — with two NULs, so
termination does not depend on whether the walker's stride is one byte or two.

### Anything you define so the reference will LINK is part of the oracle `[§9.73]`
A 20,000-case differential fuzzer said MATCH for four waves while **six shipped msvcrt functions
were wrong** — because the harness's own convenience stubs for `_tolower_l` &c. graded the Rust
against *our model* rather than msvcrt's real locale-table lookup with a domain test that sets
`errno = EILSEQ`. The values were fine; the errno contract was not. **Fix:** make the real
`ctype.c` a reference TU so the reference grades itself, and delete the stubs.
- **Corollary** `[§9.74]`: `offset_of!` into a struct the harness defines differently is an
  oracle that *agrees* with a subject reading the wrong bytes. Also: `compile_commands.json`'s
  `command` is shell-quoted — use `shlex.split()`, never `.split()`; and bindgen records the
  Windows spelling of a fixed-width type (`__msvcrt_ulong` is 32-bit).
- **A harness improvement is worth nothing if you only point it at NEW code** `[§9.66]`.
- **A branch reached by NO row is a case that cannot fail, and rows are cheap enough that you
  will believe you covered it.** msvcrt's `_s` printf shells take their truncation branch only
  when `ret < 0 || ret == len`, and their ERANGE arm inside it only when `count > sizeOfBuffer`.
  A 33-row corpus varying the *format* and the *count* — but leaving the buffer generously
  sized in every row — never entered the branch at all, so the arm that fires the
  invalid-parameter handler, sets `errno`, and **zeroes the caller's whole buffer** was graded
  by nothing. Halving its `memset` as a negative control moved zero rows; adding nine rows with
  a small buffer made the same control fail eight. **The variable that unlocks a branch is not
  always the one the function is named after** — here it was the buffer size, not the count or
  the format. Perturb each arm and count the rows that move; an arm no row can reach is the
  vacuous pass, and green stops anyone looking again.
- **A hand-typed CONSTANT in the refstub is an oracle that grades against a value Windows never
  returns.** rtl-difftest's `refstub/rtl.h` carried
  `#define STATUS_UNMAPPABLE_CHARACTER 0x80000016` — the real value is `0xC0000162`, the same
  digits transposed. Both are negative, so `NT_SUCCESS` agrees and control flow is identical;
  only the returned value differs, which is precisely what a differential test compares. It
  read as a *Rust* bug for as long as nobody checked the reference's own header. Every constant
  you re-declare so the reference will compile is un-sealed data — the same class as the 61
  invented kernel stop codes `[§9.113]`, except no shipped-code scanner looks in a test's
  private header. **Audit the whole block against the SDK, not just the one that got caught**
  (16 constants, exactly 1 wrong); prefer including the real header over re-typing values.

### The oracle can be MISCOMPILED by the host's own headers `[§9.66]`
glibc's `__nonnull` / `__attribute_pure__` attributes **deleted the reference C's NULL guards**,
so the *reference* segfaulted (or silently answered wrong) on exactly the inputs the guard
exists for. **The C source and the machine code disagree — trust the machine code.** The fix is
to disarm the attributes, **not** `-fno-delete-null-pointer-checks` (that patches the NULL
symptom and leaves `pure` in place). **The tripwire matters more than the fix:**
`ref_integrity.rs` is a test whose subject is *the oracle itself* — for each reference function
whose C contains a guard, call it in a forked child with the input the guard exists for and
assert it returns rather than faults.

**The mirror image: the harness's own ENVIRONMENT binds to the subject.** A static definition
beats the shared libc, and the code under test is full of `#[no_mangle]` CRT names. msvcrt-
difftest's NLS layer called `iswalpha_l` to build its codepage tables; that resolved to the
*subject's* Rust export, which called `GetStringTypeW`, which called back into the NLS layer —
stack overflow on the first case. Anything the harness calls to CONSTRUCT the fixture must
reach the host libc explicitly (`dlopen("libc.so.6")` + `dlsym`, resolved once and cached), not
by name. Do not "fix" it by listing today's collisions: that shadow set grows with every file
you translate, so an enumeration has an expiry date and the next collision presents as a
mysterious hang, not a link error.

**A name the reference TU never DEFINES is quietly supplied by the host libc.** Renaming covers
what a TU *defines*; it says nothing about what it *calls*. msvcrt's `wcs.c` calls plain
`vsnprintf` from five entry points (`vsprintf`, `vsprintf_s`, `sprintf`, `sprintf_s`,
`_snprintf`) and, at `_MSVCR_VER=0`, never defines it — the real build closes that with
`stdio.h`'s `#define vsnprintf _vsnprintf`, but the harness's prelude had no such line, so the
name was in no rename list, left the object undefined, and the linker satisfied it from
**glibc**. Five reference entry points were grading the host libc while claiming to be msvcrt:
no `%I64` conversions, `(nil)` for `%p`, two-digit exponents. **Check `nm --undefined-only` on
every reference object for names your subject also implements** — an undefined symbol in a
reference TU is a silent host-binding, not a link error. Then keep a canary row in the format
the two disagree about (`%I64d`, which glibc prints literally) so a regression fails a row
instead of moving the oracle.

**But fixing it TU-by-TU, not globally: the same redirect can be right in one instantiation and
fatal in another.** msvcrt-difftest compiles `wcs.c` twice — once under the host's SysV va_list
(`ref_*`) and once under a flat `char*` `<stdarg.h>` (`pfflat_*`, which is the Windows model).
`_vsnprintf` forwards `&valist` as a `va_list*`, coherent only where a va_list IS a plain
pointer: correct for `pfflat_`, and an immediate fault for `ref_`, where `*valist` reinterprets
the decayed pointer as the register-save-area tag struct. So the redirect belongs in the pfflat
prelude only — applied to both, it SIGSEGVs somewhere unrelated to printf, because
`ref_create_mbcinfo` formats its codepage with `sprintf(buf, ".%d", cp)` and every mbcs case
dies in the fixture. glibc's `vsnprintf` stays the `ref_` TU's formatter deliberately, and
`".%d"` is a format both agree on exactly.

**The host compiler also REWRITES the reference, which matters for a PERF oracle.** Same rule —
trust the machine code — with a different consequence. msvcrt's `memset` is a plain loop in
`string.c`, but cc idiom-recognizes it and emits its own size-dispatched expansion (four
overlapping 8-byte stores for `n` in 16..32); `ref_memcpy` next to it survives untouched as a
tailcall to the hand-rolled `msvcrt_ref_memmove`. So one row of a rust-vs-C perf table is racing
a compiler's best effort and the row beside it is racing a byte loop, from one reference TU
compiled one way. For a BEHAVIOUR oracle this is harmless (the rewrite is semantics-preserving,
which is the whole point). For a THROUGHPUT oracle it silently changes what the number means, and
it is invisible in the C. Disassemble each `ref_*` you bench before you believe its ratio, and
say in the harness which rows compare against the C-as-written and which against the
C-as-the-compiler-rewrote-it. `#pragma function(...)` in the ReactOS source does NOT protect you
— it is MSVC-only and cc ignores it.

### Exercising a branch is not testing it `[§9.75]`
A branch is only tested when its two sides can produce **different answers**. Planting a
non-zero locale handle made msvcrt take path 2 — and still proved nothing, because path 2 folded
through the same table. **Generalises to any harness with a "mode"** (endianness, alignment,
locale, SMP-vs-UP, a feature flag): the alternate-mode tables must be one copy shared by both
sides. Related: **a green from an unexercised branch is not a green**, and **a value-comparing
test cannot see a lost *tolerance*** `[§9.64]` — a pure refactor dropped NULL-behaves-as-`""`
and no generator that produces real NUL-terminated buffers could ever notice. Fix by making the
tolerance a first-class assertion (`assert_null_tolerant()`).

### No ratio can catch a cost the harness stubs away — count, don't measure `[§9.76]`
Every instrument was green and the OS booted **3× slower**: the Rust folded each character
through `_towlower_l(c, NULL)`, and a NULL locale means **three Win32 calls per character**, in
a function on the boot's hot path. `perf` was blind and always will be, because the harness
stubs `get_locinfo()` as a static address. The new instrument counts calls:

> **A function that walks `n` elements may consult the environment O(1) times, not O(n).**

It found two more instances on its first run.

**And a ratio is only load-robust if the two sides are INTERLEAVED.** "Same process, same
data, same load" is not enough when each side gets its own consecutive window: `perf` ran
three rounds of Rust and then three of C, so best-of-3 rejected only spikes shorter than one
window — while the load a gate generates (three cold cross-builds, four booting QEMUs) stays
hot for minutes. A spike covering all three Rust rounds and none of the C ones reports a 2×
regression that does not exist, and it did, on the difftest stage's first automated gate.
Round-robin the rounds (`rust, c, rust, c, …`) and take each side's best: a spike then lands
on both sides of the same round, and the best round is the quietest one for both. Warm both
sides before timing either. Generally: **a timing harness that was only ever run by hand on an
idle host has never been tested under the conditions a gate creates** — wiring one into CI is
itself a change to the instrument.

**Interleaving was necessary and NOT sufficient.** The same gate went red again on the next run,
on a different case, and the rest of the fix is three more things — each of which failed to be
enough on its own:

- **Round LENGTH beats round count.** Best-of-K only reports the peak if some round dodges every
  interruption, and at ~18 ms a round essentially never does (the scheduler tick alone is 4 ms),
  so best-of-3 was reporting the *mean* while claiming the peak. Splitting the SAME total work
  into 9 short rounds — no extra wall time — moved `strcat/16` from a stable 1.71× to 0.95× and
  `memset/16` from 1.81× to 1.37×. Those "regressions" were the estimator.
- **A fixed threshold must clear the instrument's own noise floor, and the only honest floor is a
  MEASURED one.** At n=16 a call to a CRT primitive costs ~13 cycles end to end, so the reading
  is mostly code layout: taking the shipping `strcat` and *deleting two null checks* — strictly
  less work — read **1.65× slower** at n=16 and 1.82× at n=64, but 1.03× at n=4096. A 2.0× bar
  sits inside that. Bench **identical twins** — the same source through the same compiler,
  compiled into two symbols at two addresses — at every size, and scale the bar by how far from
  1.00× they land. The twins must be **same-language**: a Rust-vs-C twin reads 1.28–1.43× at
  large n, where per-call effects are noise, because that is rustc-vs-cc codegen — the thing the
  harness *exists to measure*. Folding it into the error bar inflates the bar to ~2.8× everywhere
  and quietly retires the gate. Keep the twins in separate TUs (gcc's `-fipa-icf` folds identical
  functions within one TU at `-O2`) and **assert their addresses differ** — a merged pair reads a
  vacuous, permanent 1.00× while measuring nothing.
- **A calibrated bar still cannot see a BURST**, because the calibration is one snapshot and the
  burst is not. At load 21 the bar widened correctly to 2.81× at n=16 and `memset/16` still read
  3.41×: its Rust rounds had been hit 3.9× while its C rounds took only 1.6×. Nothing measured
  once distinguishes that from a real regression — so **require the failure to REPRODUCE**. A slow
  body is slow in every pass; a burst has to hit the same case on the same side N times running.
  Costs nothing on a green run, since extra passes happen only after something already failed.

Verified in both directions, which is the part worth copying: under synthetic load 31 a flagged
case cleared on pass 2 and the run passed, and a deliberately-slowed body (four redundant scans
in `strchr`) failed all three passes at all three sizes. **A gate that has only ever been proven
to pass is half-tested** — perturb something and watch it fail before you trust it.

### A TEXT-SHAPED match is not a PROOF `[§9.119]`
The same regex measured **146** lowerable CRT forwarders; the sound implementation lowered
**11**. Measuring a shape by its syntax measures an **upper bound, not a yield**, and the
blocker is almost never the shape — it is the missing authority. Read with its mirror
`[§9.120]`, where a regex **MISS** was mistaken for proof a class was absent (`0` reported for
six weeks). **A pattern match is evidence about TEXT, never about SEMANTICS, in either
direction. Demand a positive AND a negative control before believing a count.**

A third direction of the same class: a regex can match a span that **is not a token at all**,
because it cannot see what preceded it. Symptom — `translate --free-functions shlexec.cpp`
refused the whole TU with `unsupported C wide character literal L':` quoting ~1,800 lines, and
the literal it named does not exist in the file. Cause — `L'(?:\\.|[^'\\])+'` matched the
trailing `L'` of `case 'L':` (a NARROW literal whose closing quote follows the letter L) and ran
to the next apostrophe, which was in a comment reading `PIDL's`. Fix — scan, don't match:
walk the source tracking comments and quoting and fold an `L` prefix into the span it belongs
to, so `'L'` starts at the quote and `L':'` one character earlier. **Lexical boundaries are the
one thing a regex structurally cannot find, because a token's start is defined by everything
before it.** The same file had all three variants that break the naive pattern — a wide literal
inside a comment, `L'` inside a narrow string, and the `case 'L':` — so treat one occurrence as
proof the others are present too.

### An oracle needs an oracle `[§9.122]`
A scanner printing `0 findings` is asserting one of two very different things: "I looked and the
tree is clean" or "I did not look." **This is not a fail-loud problem** — the scanner ran fine.
`tools/test_scanner_controls.py` is a declarative table giving every gate scanner a **positive**
control (a synthetic defect it must find) and a **negative** control (clean code it must not
flag). **Validate the harness by BREAKING the scanner** — restoring a pre-fix scanner must make
the harness fail. Two instances of the very class turned up while building it: the harness
itself was a liar on its first run (the positive assertion accepted *any* non-empty output), and
two scanners take FILES, not directories, so handed a directory they silently found nothing.

**A shared regex is an API between lanes, and positional groups are how it breaks.** Mainline
correctly taught `scan_cc_mismatch` to pair decls and defs on the *linker symbol*
(`#[link_name]` / `#[export_name]`) instead of the Rust identifier, which added one capture
group to each of two patterns. Every read in the *scanner* was updated; the `--fix` writer,
added by a different lane, read the same patterns positionally and was not. Two silent failures
fell out at once: `cfg, cc, name = m.groups()` started raising `ValueError`, and `d.group(1)`
became the new `link_name` group — `None` for an ordinary decl — so the fixer classified every
declaration as unnamed, relocated nothing, and exited reporting success. Fix the class, not the
call sites: **name every group and read only by name**, so an added group cannot renumber a
consumer you did not know existed.

**A feature whose pattern does not match the tree's actual spelling is dead on arrival**, and
it is invisible precisely because it reads as implemented. That same `#[link_name]` support
allowed only whitespace between the attribute and `fn` — but every `#[link_name]` in this tree
is spelled `#[link_name = "_wcsicmp"] pub fn wcsicmp(…)` (`rtl-actctx/src/shim.rs`,
`ps-rust/src/kill.rs`). The `link` group was therefore always `None` and the pairing silently
degraded to the identifier behaviour it was written to replace, tree-wide, with no finding
count change to notice. When adding a pattern for a shape, grep the tree for that shape and
assert the pattern matches what you find — a control fixture you wrote yourself only proves the
regex matches *your idea* of the spelling.

### "Canonicalising" a mismatch can silence it by editing the EVIDENCE
A lint that reports `X disagrees with Y` has no opinion about which side is wrong, so a
mechanical pass that makes them agree will always turn it green — including by changing the
side that was right. This lane did exactly that: a "full mechanical CC canon" pass
(`e33a810c9ad`) aligned every `extern "X" { fn F; }` declaration to F's `#[no_mangle]`
definition, and among the files it rewrote were two **generated bindgen islands**. bindgen's
`extern "stdcall"` there was not a local opinion — it was the transcript of the C header's
`NTAPI`. Flipping it to `extern "C"` silenced the finding on buslogic's 6 SCSI miniport
callbacks and cdrom's 2 WDF cleanup callbacks, every one of which our Rust wrongly defined
cdecl while the port driver / WDF calls them through a stdcall function pointer. The live
count fell 305 → 16 and read as burndown progress.

Three things generalise:

- **Classify each side as OPINION or EVIDENCE before aligning them.** A hand-written decl
  describing a symbol the crate itself defines is opinion; a generated island, a `.spec`, a C
  header, a DDK function-pointer typedef are evidence. Never edit evidence to match code.
  `scan_cc_mismatch --fix` now refuses to write any `types*.rs`/`bindgen*.rs` and reports
  those as "the DEFINITION is the suspect side" instead.
- **A `transmute` at the binding site removes the compiler from the argument.** buslogic
  registers its callbacks with `cb!(F)` = `transmute(F as usize)`, so rustc never compares
  `extern "C" fn` against the `extern "stdcall"` field it lands in. Wherever a fn pointer is
  installed by transmute, a *source-level* convention check is the only thing that can see it.
- **A finding count that drops without a corresponding fix is a question, not a result.**
  Re-derive it from the C, not from the tree's internal agreement.

The same pass's counterpart trap: an oracle can also be *falsely* consulted. `cfg_excludes_i386`
only ever saw attributes directly preceding a definition, so `#[cfg(target_arch = "x86_64")]
mod ke_amd64_exports { … }` made 8 amd64-only definitions look like i386 authority — and --fix
used them to rewrite ntoskrnl's *correct* i386 decls. **A cfg'd-out definition is not evidence
either**; the scanner now brace-matches per-arch modules.

### A generator can only flag what its TYPE SOURCE can see `[§9.115]`
Three near-identical crashes: an unmodified stock i386 applet added to the suite kept dying,
because wow64gen typed its thunks from a source narrower than the truth. **A report that
iterates the generator's own flagged set inherits the same blind spot.** Fix by crossing the
generated artifact against an independent source of truth — including **clang's own record
layouts** (`-fdump-record-layouts`) for i686 and the host. Two design points that make such an
audit usable rather than noise: **separate "provably divergent" from "I could not see the
layout"**, and derive from checked-in sources so the baseline is portable.
- **A generated table whose indices come from one side of a boundary and whose contents come
  from the other is only correct where the two agree — and it will not tell you where they stop
  agreeing** `[§9.71]`. `#ifdef _WIN64` in `w32ksvc32.h`, parsed as flat text, **misrouted 195
  WoW64 syscalls onto their neighbours.** x86 Doom ran beautifully — window created, game loop
  pumping, 4000+ frames blitted — because every guest before it used only syscalls *below* the
  shift (index 400 fine, divergence starts at 541). **An off-by-one in a dispatch table is
  invisible to every failure detector we have**: no fault, no bad status, no ud2. The only
  oracle is comparing the table against the thing that actually calls into it —
  `verify_svc_numbering.py` disassembles the **real i386 `user32.dll`/`gdi32.dll` syscall
  stubs** out of the build. **Parsing a C header with a regex is parsing it wrong.**

### A parse/derive failure that silently falls back `[§9.72, §9.80]`
- **A catastrophic-backtracking regex hung `postprocess`**, and the hang looked like the paid
  translate step. **The damage is the workaround:** the operator kills the wedged run and
  hand-assembles the output, bypassing the entire post-processor. **A hang is a correctness
  bug** — it trains an operator into a habit that skips your checks.
- **`abi_enforce` derived the right convention and then discarded it**; `_derive_convention`
  fell back to cdecl **on a comment**, mis-deriving 17 genuine WINAPI methods; and
  `CALLBACK`/`PASCAL` were missing from the keyword list. A silent fallback to a plausible
  default is the worst possible failure mode.
- **An oracle that resolves a type wrongly reports its own IGNORANCE as a defect** `[§9.80]`,
  and the ambiguity check cannot save you when the parser is blind to the true declaration —
  four flavours of the same mistake: an `.idl` outvoting the `.h`; the true declaration
  invisible so the false one looked unanimous; a naming convention applied blind
  (`^(?:LP|P)[A-Z]` read **`POINT` as a pointer to `OINT`**); and multi-word base types
  (`typedef _W64 unsigned long ULONG_PTR` parsed as base `_W64`).

### A C↔Rust semantic-divergence oracle judging Rust by C's rules `[§9.130]`
Both findings it produced on landed, hand-audited code were false positives, and both had the same
root — **C semantics applied to the Rust side**:
- A **CRITICAL "argument type mismatch"**: C's `0` beside Rust's `null_mut()` read as an 8-byte
  pointer where a 4-byte `DWORD` belongs. The real prototype takes `LPDWORD`, so the C `0` *is* a
  null pointer constant and the Rust is right. The model saw two bodies and no signature — while
  the signature sat in the very file the Rust came from. **Fix: give it the `extern` declarations
  the body calls, labelled authoritative for argument types.**
- A **HIGH "inverted condition"**: `flags & FLAG != 0` reported as `flags & (FLAG != 0)`. That is
  C's precedence. **In Rust `&` binds tighter than the comparisons**, so it is `(flags & FLAG) !=
  0` — and the alleged parse could not even compile, since `u32 & bool` is a type error. Verified
  with rustc rather than asserted.

After both fixes the same file went from 1 critical to **0 critical, 0 high**.
**A semantic-divergence oracle is a candidate generator, not a verdict** — both findings were
entirely plausible and wrong. Prefer giving the oracle the missing *authority* (prototypes,
language rules) over loosening what it reports.

### When the bug is in the TEST, not the kernel
- **A broken self-test gives a false FAIL** — arm64's `KmSafeCopy` self-test probed a "bad"
  address that was actually valid `[§9.93]`.
- **kmtest `CHECK_DATA` union-aliasing** fabricated "byte-shifted pointer" corruption that did
  not exist `[§9.97]`.
- **A synchronous-IRP helper must check the WAIT return** — on `STATUS_TIMEOUT` the IRP never
  completed and the stack `IO_STATUS_BLOCK` is still zero (== STATUS_SUCCESS), so it reports a
  FALSE "OK". This hid a TDI round-trip that never connected.
- **A test oracle can test the wrong thing entirely** — the first Doom playable-gate probe used
  F1, which opens Doom's help screen rather than proving input reached the game.
- **A compiled-but-never-called oracle lies in two ways** `[§9.83]`: it can answer with the
  wrong **libc** (only coding style revealed it), and Wine/ReactOS printf code is structurally
  **uncallable** under the SysV varargs ABI, so it must be given the flat `char*` `<stdarg.h>`
  shape that Windows-x64 actually uses. That probe's real payoff was **finding a kernel hang
  before any new Rust was written** — 111 MATCH / 27 DIVERGE was decision-grade.
- **A triage tool that lies costs more than no tool** `[§9.118a]` — the `[RAISE]` helper
  reported a STACK address as the raise site, because its "kernel-text-looking" window also
  contains kernel stacks.

### Lints as permanent gates
`tools/` carries the deterministic detectors; all are wired into `vibeos-cli check` at **0 NEW
vs `tools/lint-baseline/`**, and several hold an EMPTY baseline (strict).
- **`scan_irql_balance.py`** `[§9.28]` — leaked IRQL raises / spinlock acquires. **RAII is the real fix**
  (`ntkrnl_sys::guards::{IrqlGuard, SpinLockGuard}`). Engine rules earned from real false
  positives: strip lvalue wrapper noise (`addr_of_mut!`); field stores are cross-call, not
  intraprocedural; **a raise to a VARIABLE is a restore, not an acquire.**
- **`scan_ob_refcount.py`** `[§9.32]` — leaked Ob references. First sweep: 46 raw → 12 after engine
  iteration → 3 real. That ratio is the norm, which is why the **verify-fanout recipe** (one
  agent per finding-file, each required to exonerate or confirm) is worth the cost.
- **`waitdep`** `[§9.31]` — an always-on dispatcher wait-block integrity tripwire in
  `ke/waitdep.rs`, hooked at `KiUnlinkThread` entry (every unwait) and `KiWaitTest` entry. On
  violation it prints `[WAITDEP] VIOLATION <kind>` plus a bounded dump naming which invariant
  broke. Always-on rather than opt-in because the flake it hunts fires ~1/4 arm64 boots and a
  reproduction attempt perturbs it.
- Others: `scan_ud2`, `scan_ir_mismatch`, `scan_cc_mismatch`, `scan_magic_offsets`
  (`TYPED_PTR_HEX_ARITH`), `scan_wide_literals`, `scan_assert_to_ub`, `scan_bugcheck_codes`,
  `scan_const_contradictions`, `scan_nightly_surface`, `scan_silent_failures`,
  `verify_lost_exports`, `verify_match_consts`, `verify_svc_numbering`, `scan_doc_sections`.
- **Audit the tool with the tool.** Every false positive a table-auditor produces is a bug in
  the auditor's model of the substrate, and worth fixing there.

---

## Cohab flip & cold-link mechanics

### A partial cohab is un-cold-linkable, and the dup↔undefined pair is the oracle
If the unit is partially flipped, CMakeLists alone cannot link it cold: you get duplicate
symbols one way and undefined the other. That pair *is* the diagnostic.
- **Verify a flip with `llvm-nm libX.a`** — a stub-only archive has 0–1 `T` symbols (just the
  cohab stub). Audit **all** `static_libs` C archives on arm64 with `llvm-nm | grep -c ' T '`
  before trusting a flip.
- **A middle-layer flip surfaces undefined symbols in WAVES** as Rust objects get pulled for the
  first time; a leaf flip cascades a wall of undefineds in a companion DLL. Expect several
  rounds; it is not a sign the flip is wrong.
- **Completing an `rtl_vista` flip needs a THIRD edit** or `reactos-link` panics.
- **Commenting out a family's `*_SOURCE` to swap libcntpr breaks msvcrt.dll.**
- **Don't `git rm` a tree named in a prune table** — verify the build graph first.
- **Don't retranslate a `*_rust_shims.c` function** — a stale private Rust impl may exist.
- **Provide DDK/driver MACROS and FORCEINLINEs from the real headers, not bugcheck stubs.**

### An incomplete flip breaks ONLY on a clean arm64 tree
i386/amd64 keep working because the stale pre-flip `libX.a` (or a weak inline copy clang emits
on x86 but not arm64) still carries the symbol — a truly clean x86 rebuild would break
identically. Two shapes: the cargo consumer never added the rust archive, or the crate
references something that is inline/intrinsic on x86 with no arm64 provider (strsafe
FORCEINLINEs, `InterlockedCompareExchangePointer` → use `core::sync::atomic`, C variadic
forwarders, a Rust `static` consumed by C without `#[no_mangle]`).

> **Historical note (resolved).** Adding those archives on i386/amd64 was once gated
> `aarch64`-only because the extra archives shifted the kernel link layout and tipped a latent
> miscompile — which *migrated* three times under investigation (cmlib hive write → storage-stack
> 0x50 → a late user-mode hang), proving it was a class, not a site. **That wall is gone**: the
> five deep-C cohabs went cargo-native on all arches, the captures(none) root is toolchain-fixed,
> and the layout canary boots 5/5. Kept only because it is the canonical example of
> *any link perturbation tipping one latent miscompile* — if that symptom returns, this is the
> shape, and `VIBEOS_LAYOUT_CANARY=1` is the instrument.

### Usermode-DLL flips: the MIDL / master-header prelude
> **A multi-header redefinition wall is a claim about YOUR WRAPPER, not about the headers.**
> `[§9.108]`

The "netapi32 MIDL dragon" — 7 RPC-stub `.c` said to carry six mutually-conflicting interfaces —
**was never an interface-vs-interface conflict.** It was `<lm.h>`, the LAN Manager **master**
header, fighting the WIDL stubs: it pulls `lmat.h`/`lmshare.h`/… so every `*_c.h`'s own wire
types collide with it. **The oracle was sitting in the C the whole time:** `netapi32.h`
deliberately includes only `lmaccess`/`lmapibuf`/`lmerr` and lets each stub supply its own types.
Only browser↔srvsvc genuinely collide (`_SERVER_INFO_100`). **Fix: mirror each TU's real include
set** — `gen_driver_types.sh` grew `TYPES_STEM` (several bindgen islands per crate) and
`EXTRA_BL_ITEMS`. This misdiagnosis deferred the work **twice**, so: **test each header
standalone (5 minutes) before theorising, and never let an untested inference become a recorded
blocker.** Expect fallout when splitting an existing prelude — items that were transitively
available stop being so.

Four integration bugs a green `cargo check` cannot see `[§9.109]`, plus four C-fidelity classes
that survive `cargo check` **and** manual review `[§9.110]` — all covered above, listed here so
the flip checklist is in one place: the `.responses` assembly glob dropping 3-digit files;
`verify_lost_exports`' regex missing a one-line `#[no_mangle]`; a model silently skipping a
function; **MIDL `*_c.c` stubs referencing UPPERCASE `MIDL_user_allocate`/`MIDL_user_free`**; a
vestigial `extern` in a header making the translation invent a global the C never had;
`write_bytes` element-vs-byte counts; a non-blocking C debug macro lowered to `loop {}`; and a
C file-scope global defined in Rust **without `#[no_mangle]`**. Each layer caught what the
previous could not.

### raw-dylib in a freeldr-loaded DLL hangs the boot
A DLL that FreeLDR itself loads (`kdcom.dll`, `bootvid.dll`) must **not** use
`#[link(kind="raw-dylib")]` for cross-DLL kernel-data imports — the second `ntoskrnl.exe`
import descriptor it emits hangs FreeLDR's PE loader. Conversely, cross-DLL DATA refs in a
*kernel*-loaded Rust DLL **need** raw-dylib on i386, and `cgu>1` can emit a duplicate
descriptor. win32k's variant: a fake-export phantom descriptor (freeldr OOM) plus a real-export
per-arch bugcheck.

### Cohab link-clean ≠ runtime-correct
A clean cargo + ninja can still bugcheck mid-init. The link is a necessary gate, never a
sufficient one — which is the whole reason smoke is the north star.
---

## Mm, PFN & paging

### The `OriginalPte` / `RmapListHead` union overlay `[§9.13]`
The amd64 service-startup wedge (~50% of boots for weeks; `DcomLaunch done: 1053` on serial ⟺
wedge) was a **Mm working-set pushlock self-deadlock**: a page fault *inside* the
WS-EXCLUSIVE section-unmap teardown re-acquires the WS lock SHARED, and pushlocks are not
recursive. Chain of three bugs, all in PFN-union handling:
1. **`OriginalPte.Long` dereferenced as a pointer** (7 sites) — reading the PTE *value* as a
   PTE *pointer*.
2. **ROS PFNs have no `OriginalPte` — `RmapListHead` overlays it** (the C `_MMPFN` union). A ROS
   section's pages come from `MmAllocPage`, which never writes `OriginalPte`.
3. **A one-slot offset shift** in hand-rolled `NextLRU`/`PreviousLRU`/`RmapListHead` accessors
   (they missed an embedded `Wsle`), so LRU links, WS-lock bits and rmap heads all landed on the
   wrong words.

The earlier "PteFrame share-count chain" and "MMPFN accessor layout" hypotheses were red
herrings. **`OriginalPte` in this port is triple-duty** — proto-template protection for active
pages *and* rmap head for ROS PFNs — so any accessor over it needs the union discriminated.

### The fixes that cascade behind it
- **0xBE (write to read-only) on a session/GDI view** `[§9.14]` — `MiUnlinkPageFromList` ends
  with a line the C literally labels `/* ReactOS Hack */` that zeroes `OriginalPte`, destroying
  a **prototype** page's protection across a standby round-trip → `MM_NOACCESS` → the first
  win32k/GDI write faults. Guard the hack to skip prototype pages. (Session PTEs can't be read
  from a foreign context — that's why this needed an in-context trace.)
- **0x4E `PFN_LIST_CORRUPT` at process-exit teardown (amd64)** `[§9.15]` — cfg-drift:
  `MmInitializeProcessAddressSpace` never initialised the hyperspace PDPT's PFN because the
  `_MI_PAGING_LEVELS == 4` PXE block was gated `aarch64`-only. One-line cfg extension. Exposed
  *by* the 0xBE fix, since boot then reached process exits it never used to. **The
  `COLORED_LIST_HEAD` fingerprint (`0x01ffffffffffffff`) was a red herring.**
- **0x4E subcode 0x99 at teardown (arm64)** `[§9.16]` — the whole `MiPteToX` navigation family
  was **one self-map recursion level too high** on aarch64, so `MiPteToPde` returned the **PPE**
  and `MiDeletePte`'s proto branch decremented the page *directory*'s share count instead of the
  page *table*'s. Provable arithmetically: `MiAddressToPde(PTE-of-X) ≠ PDE-of-X` for all X.
  **When a `++`/`−−` ledger shows a *systematic* per-page-kind imbalance, suspect the navigation
  arithmetic, not the accounting.**

### A translation-fault STORM that hangs with no faulting PC `[§9.96]`
An arm64 kmtest quarantined as "fastfat stub bugcheck" was really `MmAccessFault` returning
SUCCESS while leaving the section-view leaf PTE **invalid**, so a kernel `RtlCompareMemory`
re-faulted forever — a HANG, no bugcheck. The L3 descriptor was `0x45c32441`: PFN set, AF set,
AP set, **bit0 (Valid) set but bit1 CLEAR** — malformed, written by a legacy/proto→hardware PTE
builder using a NOACCESS mask index. **True root: a DROPPED `PAGE_NOACCESS` early-return in the
Rust translation**, found *statically* from the malformed-value signature — no writer-hunt boot
needed. **Triage recipe for a hang with no faulting PC:** boot the already-staged conformance
media under GDB, then use QMP `gva2gpa` plus a PTE walk to read the actual descriptor.

### A nested 0x50 one page above SP is STACK OVERFLOW `[§9.11]`
`arg2 == 2` (write) with `arg1` ~one page **above** the trap-frame/SP means unbounded recursion —
often a constant-placeholder feeding an infinite loop — not a scheduler or LPC race. arm64 is
where it bites because `KERNEL_STACK_SIZE` is **12 KB on i386 and arm64 but 48 KB on amd64**.
**Diagnose with no DWARF:** walk the frame chain off `Fp` (x29) — `lr = *(fp+8)`, `next = *(fp)`
— and a short cycle of the **same 3–5 return addresses repeating** *is* the bug. Symbolicate
with the right binutils (`aarch64-w64-mingw32-nm`).

### Wild control transfer INTO a mapped DATA view `[§9.17]`
A rare amd64 post-marker `0x50` whose PC is in a driver region but **symbolicates to garbage**
means the boot **executed data** — a corrupted function pointer / vtable, in the win32k/GDI
dispatch family. Explicitly **not** the captures(none)/layout class (ruled out the same night).
The attribution recipe runs in ~15 min; **arm special pool first** (below) rather than doing
forensics on the innocent victim.

### Physical-page reuse: a frame backing a still-mapped user page is freed and re-handed-out
`[§9.98]` — **CHARACTERIZED, root-cause hunt OWED.** ~40% of arm64 kmtest sweeps kill a *later*
`kmtest_.exe`, with **two faces from one root**: `0xC000001D` at an image PC (the page under the
code changed) and `0xC0000005` in `RemoveEntryList` inlined into `RtlpCoalesceFreeBlocks` (the
page under the heap changed). **Disproven, don't re-chase:** an `rtl_heap` translation bug.
Tripwire #1 (free-while-mapped) came back **negative**, which narrows it. This entry exists
because §9.96 first registered the flake with a plausible-but-unproven label — **a label is not
a diagnosis.**

### Other Mm one-offs
- **A registry/hive addition that "shouldn't matter" trips an early 0x50** `[§9.10]` — the
  config manager loads the hive early and growing it shifts everything. **Don't ship the hive
  change**; treat the deterministic same-line repro as a gift, and confirm causation by reverting
  only the hive line.
- **Timer-table base skew makes every relative wait over-wait 4–60×** `[§9.9]`. The NT timer
  table requires `InterruptTime / KeMaximumIncrement == KeTickCount`, both 0-based since boot.
  Zero KUSER `InterruptTime`, KUSER `TickCount` and the `KeTickCount` global on the first tick.
  Oracle: a one-shot probe in `KxSetTimerForThreadWait`.
- **arm64-only AV on every demand-zero fault** → SMSS 0x6A. **UEFI memory-map retry never
  converges on arm64** (UINTN hardcoded `u32`).
- **`NtCreateSection` 0x50 is a dropped `ProbeForRead`, not a deep-MM bug** `[§9.104]` — and the
  dropped probe **masked** the deeper arm64 exception-dispatch recursion beneath it. **A missing
  guard can hide the bug it would have exposed.**

---

## SMP: what a second CPU unmasks

`-smp 2` does not usually find SMP bugs — it finds **UP assumptions** that were always wrong.

### The governing rule: an IRQL raise is not a lock
> **"On QEMU's single vCPU, DISPATCH_LEVEL is full mutual exclusion" is false the moment a
> second CPU exists.** In any component a `-smp N` boot exercises, an IRQL raise is *not* a
> lock. `[§9.55]`

netstack's TDI transport relied on it for its queue → the `-smp 2` "TDI dgram upcall TIMED OUT"
NET_FLAKE. Fix: a real `KSPIN_LOCK` via `SpinLockGuard`. Second instance in the same commit.

### Dropped `#ifdef CONFIG_SMP` blocks are TIME BOMBS, not dead code `[§9.95]`
`KiRetireDpcList`'s SMP-only deferred-ready drain was dropped in translation, so **an idle CPU
re-HLTs with runnable threads parked on its own `DeferredReadyListHead`** → `KeGuardedMutex`
kmtest TIMEOUT. **The census cracked it before GDB did** — `ke/census.rs` (a 10s sync watchdog)
printed "CPU 1 asleep in `hlt` with a runnable thread on its own deferred list." **Sweep every
`CONFIG_SMP` block in the C for one that never made it across.**

### The SwapBusy context-swap protocol was inverted/unenforced `[§9.94]`
NT's MP-only swap-busy protocol has three parts and i386 had none of them; amd64 had one. The
invariant: **a dying thread's stack cannot be freed until the thread is off every CPU**, and the
release must happen **after ESP has left the old stack.** Four edits: acquire-spin in
`KiSwapContextEntry`, release-clear in `KiSwapContextExit`, `KiSetThreadSwapBusy` before the
swap in `KiDispatchInterrupt` (both x86 arches), and a spin in `PspReapRoutine`/
`PspDeleteThread` (all arches). This closed the kernel-stack `IN_PAGE_ERROR` teardown race that
a system-PTE leak had been masking `[§9.88]` — **the leak fix unmasked the race, so it looked
like a new bug.**

### i386 SMP bring-up: three offset/layout bugs, each masked in UP `[§9.91]`
Swapped `PRCB_BUILD_*` constants → 0x79 MISMATCHED_HAL; `KDESCRIPTOR` missing its `Pad` field →
0x1E in `memcpy`; the loader block written by raw pointer → 0x50 with a masking 0x1E cascade;
and a cdecl worker handed to an NTAPI slot.

### The IPI-packet protocol trio `[§9.101]`
**An EMPTY "wait for completion" block** (`KeFlushEntireTb`'s SMP tail read as a comment and
translated as nothing — the *empty-wait-comment* class), **`IPI_FREEZE` treated as a request bit
when it is a STATE** (so the freeze request corrupted its own state machine), and a
scheduler-oracle mark with one uncovered promote site. **An oracle is only as good as its unmark
coverage.**

### First `-smp 2` boot: two latent bugs `[§9.51]`
uniata's `CPU_num > 1` switch to an nIEN-gated ATAPI path breaks under QEMU (gate it on bare
metal), and five+ hand-rolled `KUSER_SHARED_DATA` shadows with fantasy offsets → 0x4C. **Trap
for the debugger: uniata AND scsiport are both Rust translations**, so "the driver is C, it must
be the kernel" is wrong.

### Un-parking the AP into the scheduler `[§9.52]`
Four rules: per-processor Phase-1 loops must run **after** `KeStartAllProcessors`; the trap.S
interrupt stub's **handler** owns the IRQL protocol; **TLB-shootdown senders must poll-service,
never just wait**; and skipping the round-trip for `hlt`ed CPUs (lazy-idle TLB) is a large win.

### The ready-list tear: five refuted theories and the one real hole `[§9.56]`
**FIXED** (all three arches; 0 occurrences across every post-fix boot). The long hunt is the
lesson, so keep the shape: `Prcb->DispatcherReadyListHead[pri]` (8/9 observed) was found torn,
with faces `0x1E` IN_PAGE_ERROR, `0x19` BAD_POOL_HEADER, `0xF4`, and silent wedges — ~0.9%
organically, ~35% under the `cswitchdep` amplifier.

**Root cause: `KiIdleLoop` read, nulled and promoted `Prcb->NextThread` LOCK-FREE**, inherited
verbatim from ReactOS's never-validated `CONFIG_SMP` C (i386 *and* amd64 both lacked the
acquire). `KiDeferredReadyThreadImpl`'s standby-displacement branch — correctly under the PRCB
lock — read `NextThread = T` while the idle CPU concurrently promoted T to Running, then marked
the *running* T DeferredReady and re-deferred it; T later requeued itself run-side while still on
a drain chain, tearing the double links. **Every other promote site already held the lock; idle
was the single hole.** Fix: acquire the PRCB lock, **re-check `NextThread` under it** (a
displacement may have consumed it), promote, release, *then* `sti`.

> **A field guarded by lock L at N−1 of N sites is not guarded.** Grep every reader and writer of
> a scheduler field before concluding the locking is sound — and **treat every inherited
> `CONFIG_SMP` primitive as unreviewed**, because upstream never ran it.

**Five theories refuted with evidence first — do not re-derive them** (~40 min each):
1. **SwapBusy wait-side unimplemented** — the amd64 run-side spin *already exists in assembly*
   (`ctxswitch.S`), and a `.rs`-only grep missed it. **Grep the asm before declaring anything
   unimplemented.** (i386 genuinely has no spin — a separate latent gap, not this bug.)
2. **Double-queue on an already-Ready thread** — fired once, then later tears happened with the
   guard **silent**, which proves **`State` is the WRONG PREDICATE**: a path can change `State`
   without unlinking. This was the headline theory for weeks and it was wrong.
3. **Union-alias clobber** — `KTHREAD.__bindgen_anon_3` really *is* a union, so
   `KiInsertDeferredReadyList`'s `SwapListEntry.Next` write really *does* alias
   `WaitListEntry.Flink`. Beautiful theory; a precise still-linked guard was silent. **It never
   happens.**
4. **Already-linked node re-inserted** — a state-independent linkage test ("do my neighbours
   point back at me?") was silent.
5. **Missing/wrong PRCB lock at a mutator** — audited; every mutator pairs with the correct
   PRCB's lock, and the dispatcher lock is real too.

**Two false oracles in one hunt, both of which cost real time:**
- `cswitchdep` is a repro **AMPLIFIER, not a detector** — its violation output is noise under
  `-smp N` (a single global held-count with no per-CPU separation mis-fires on most switches).
  **It amplifies; it does not pin.**
- The attribution table's own hash lied: `rq_inflight_mark()` CAS-inserted at the first empty
  probe slot *before* finishing its 8-slot scan, so a legal re-mark created a duplicate twin that
  later fired a false violation. **In an open-addressed probe table, insert-at-first-empty may
  only happen after a full-window scan for the key** — and **a diagnostic that bugchecks must be
  held to the same correctness bar as the kernel it polices.**

**The method that finally worked:** stop adding tripwires and catch the corrupting *store*.
Enriched forensics that named it — report *which* CPU's list a thread is wrongly linked into plus
its state and owning PRCB, and dump the corrupted link *value* — turned "the list is torn" into a
specific traceable invariant ("a Running `CurrentThread` is simultaneously linked in a ready
list"), from which the upstream writer followed.

---

## arm64: SEH, unwind & bring-up

### `RtlVirtualUnwind` decoding and the frame-walk bugs
- **`save_next` resolves FORWARD, not backward** `[§9.114]` — a **32%-wrong** decoder that
  silently mis-restored callee-saved registers. ARM64 `.xdata` unwind codes are stored in
  **reverse-prologue order**, and the superseded decoder chained them the other way. **The
  reusable method: do not reason about a bit-level spec from memory — decode real `.xdata`
  against a disassembled prologue.** Corollary worth internalising: **a fix's RATIONALE can be
  wrong even when the fix is right**, and the wrong rationale costs the next reader.
- **A return address ONE PAST the function end** `[§9.93]` — when a `bl` is the last instruction
  (callee is noreturn, so LLVM emits no trailing instruction), the return address lands outside
  the function and the lookup fails. **Fix (arch-standard): look up `retaddr - 1`** in the arm64
  `RtlDispatchException` and `RtlUnwindEx` frame walks.
- **A garbage frame size wedges the boot, and it is NOT a fault storm** `[§9.69]`. Enabling the
  fault-tolerant copy on arm64 hung with no bugcheck: `RtlVirtualUnwind` decoded a garbage frame
  size → garbage SP → the transfer-to-handler faulted on the bad SP → **infinite
  exception-vector re-entry.** Two hypotheses falsified first (a DUMMY_PSEH "silent-skip crutch",
  and a recursive fault storm overflowing the stack). A *software* raise passes — only a real
  hardware fault exercises the path, which is why the self-test mattered.
- **Cross-module table-SEH drops every `__except` reached through a callee whose call site sits
  at its epilog boundary** `[§9.48]` — one line in `unwind_decode.rs`. It hid because the
  existing PoC only unwound *ntdll* frames. The `__finally` half needs `EstablisherFrame == x29`.
- **The kernel-mode SEH *resume* path used `NtContinue`** `[§9.92]`, which clears the shallow
  raise tests. Remaining in that cluster: `KeMutex` + `SeInheritance` raise from **inside**
  `KeWaitForSingleObject`; `ExHardError` is a *distinct* pool illegal-instruction (OPEN); and
  `MmSection` is an in-kernel **wedge**, not a raise-escape (OPEN).
  **A "one root, N tests" cluster label is a HYPOTHESIS** — un-mask each test separately.
- **`RtlWalkFrameChain(NULL, …)` was a cross-arch NULL-store AV**; `RtlCaptureStackBackTrace
  (BackTrace=NULL)` BUGCHECKs 0x1E and **aborted the whole i386 kmtest sweep** (52 → 74 tests
  once guarded) `[§9.105a]`. The quarantine note's stated root cause was **wrong** and cost real
  time; the smoking gun was an **off-by-one in the trace, not an address**. The guard is
  deliberately **cfg-gated to i386 and self-retiring** — guarding all arches measurably costs.
- **arm64 user-mode SEH never delivers first-chance** — `RtlRaiseException` is unimplemented for
  arm64 (OPEN; needs a `#[naked]` capture stub).

### arm64 bring-up bugs worth recognising again
- **PSTATE.DAIF is NOT per-thread state** `[§9.19]` — `KiSwapContext` doesn't save/restore it, so
  a context switch **leaked exception-entry DAIF into a new thread** → boot wedges with the
  serial completely silent. A masked-swap tripwire in `KiSwapContextResume` remains in tree.
  **The mirror bug is a dropped `cli`** in a C→Rust translation causing a callback-ERET
  IRQ-clobber race `[§9.54]`.
- **Every arm64 BUGCHECK wore a mask: the crash reporter crashed** `[§9.54]`
  (`KeGetBugMessageText` validation). **Triage rule: in any arm64 crash log, count the
  bugchecks — a second one is usually the reporter, and the first is the real one.**
- **ISR-queued DPCs never ran at desktop** `[§9.20]` — device-DPC delivery was gated on a starved
  `TimerRequest` flag; `KeInsertQueueDpc` returned 1 and nothing drained. Fix:
  `KiRetireDeviceDpcs` drains `DpcData[DPC_NORMAL]` unconditionally.
- **Desktop timer freeze = three stacked bugs** `[§9.21]`: `KeDelayExecutionThread` was still a
  busy-yield bring-up stub, explorer's main thread spun on a stuck `QS_PAINT` wake bit, and
  DISPATCH software-interrupt delivery was unimplemented.
- **Desktop never repaints** `[§9.23]`: `gpdeskInputDesktop` stranded on the *Winlogon* desktop,
  and `SystemTime` was never seeded from the RTC (the "query the clock" block was dropped —
  arm64 needs the PL031 at `0x0901_0000`).
- **Desktops only ever paint via the hidden→visible show transition** `[§9.49]` — arm64's early
  input-desktop switch broke that assumption for the whole window tree → BLACK desktop with green
  serial markers. Fix: `co_IntShowDesktop_impl` adds `RDW_ERASE`. **Exonerated en route, do not
  re-chase:** the dib1bpp/dib32bpp GDI blitters.
- **arm64 never delivered user-mode APCs on return-to-EL0** `[§9.90]` — a `UserMode`
  `KeWaitForSingleObject` returned `STATUS_USER_APC` **forever**, so the kmtest sweep captured
  exactly one result. The delivery step was simply **absent** on the arm64 SVC return.
  **Rule: a `UserMode` wait returning `STATUS_USER_APC` persistently means the APC is never
  drained, not that an APC is pending.**
- **"Unaligned access = Device memory" is only HALF a diagnosis** `[§9.50]` — **dump the OPCODE
  before blaming Mm.** On arm64 alignment faults have two instruction classes; FEX emits
  LDAR/STLR/LDAPR for x86-TSO emulation, which fault on Normal-but-unaligned. A 3-in-1
  discriminator is permanent in `ke/arm64/trap.rs`.
- **On an arm64 `0xC000001D`, decode the ESR before theorizing** `[§9.85]`. EC **0x18** =
  trapped-but-*legal* system instruction, i.e. a missing SCTLR/CNTKCTL/CPACR enable bit, and the
  ISS hands you the exact op and register. The FEX startup-death flake was a trapped `dc cvau`:
  SCTLR_EL1's EL0-usability bits (UCI/UCT/DZE) were never set. **Cracked entirely from saved
  serial logs, zero repro boots**, because the `[UEXC-KILL]` banner captures the faulting
  instruction bytes.
  > **"Flaky" + byte-identical registers on every hit = a DETERMINISTIC bug on a rarely-taken
  > path.** Hunt for why the path is rare, not for a race.
- **A silent arm64 Rust panic looks like an opaque boot hang** — sample `$pc`; it parks in
  `rust_begin_unwind`. Also: `RtlCreateAtomTable` divided by zero on `NumberOfBuckets == 0`.
- **arm64-only deadlock from UB**: `&immutable_local as *const T as *mut T` on an in/out FFI
  param — a Heisenbug that moves under instrumentation.
- **`trap.c` is dead code** — editing the wrong arm64 trap handler wastes a cycle.
- **arm64 user-mode pegged in `KeStallExecutionProcessor`** starves explorer/win32k entirely.

---

## WoW64 / FEX: 32↔64 divergences

Every entry here is one shape: **a size, offset or index that differs between guest and host.**

### A stock binary flushes out what a hand-written guest never can `[§9.67]`
Every guest through M10 was written by us — a hidden bias, because a guest you author can only
exercise what you thought of, and it silently *avoids* what you got wrong. One stock binary found
five bugs at once: the 64-bit loader read a 32-bit image's `IMAGE_TLS_DIRECTORY`; the environment
block was never marshalled (guest saw NULL, kernel32 walked from address 0);
`NtCreateSemaphore` was a loud stub, which killed **every** GUI app (wininet's DllMain →
rpcrt4 → user32); `LARGE_STRING` is 12 bytes in the guest and 16 on the host; and window extra
bytes were broken two ways. **Meta-lesson: the widest-blast-radius bugs are the ones only a real
binary reaches** — `GlobalLock` returning NULL killed every edit box in the OS and was found the
same way.

### Size tables are an ABI, and per-arch `size_of` is per-*HOST* `[§9.116, §9.118]`
`g_MsgMemory` sized the lParam struct with the **host's** `size_of`, so a 32-bit sender's buffer
was over-read → BUGCHECK 0x1E at win32k's `raw_move`. The first fix went into win32k and **that
was wrong** — one grep overturned it: the fix belongs in the **thunk layer**, and
`NtUserMessageCall` carried **three** divergences, one smashing guest stack on *every*
`SendMessage`. Rules banked:
- **Scope the marshalling by what the callee DOES with the arg, not by the arg's type.**
- **Fail the call, don't pass the buffer** — `WM_NCCALCSIZE`/`WM_COPYDATA`/`WM_CREATE` embed a
  pointer that cannot be narrowed safely.
- **Don't hand-write converters when a generator already emits them**, and **verify widths with
  the layout engine, never with arithmetic.**
- **Fixing the box-killer moves the ladder to the NEXT bug** — expect it; don't read it as a
  regression.
- **Timing-dependent means one green boot proves nothing**, and **prove pre-existing vs.
  introduced with a CONTROL, not with reasoning.**

### `IO_STATUS_BLOCK` is 8 bytes in the guest and 16 on the host `[§9.62]`
`NtReadFile` was never on the deep-marshal list, so the generator silently mis-thunked the whole
file-I/O family — **latent; nothing was visibly broken.** **The lesson: a generator that is
safe-by-default only where it has been told to be is not safe by default.** Async is the trap and
it is the dangling-stack-IOSB class: the host IOSB we marshal into is a *stack* temporary, so a
late completion writes into a reused frame. **The proper fix is kernel-side and is owed.**

### An OUT handle/pointer array not narrowed → 2× guest-heap overrun far from the write `[§9.105]`
The crash is minutes of guest activity *after* the write, as a first-chance AV with a NULL or
garbage `Blink`. **The dispositive signature in a dump: an array of `{VALUE:4, 0:4}` pairs** —
64-bit values written where 32-bit ones were expected.

### A flat-thunked `NtCreateThread` returns SUCCESS and builds a thread with a ZEROED context `[§9.134]`
`NtCreateThread` has five divergent params (`OBJECT_ATTRIBUTES`, `CLIENT_ID`, `INITIAL_TEB`,
`CONTEXT` — whose field lists don't even correspond across widths — and a `PHANDLE`), so a 32-bit
guest calling `CreateThread` hands the 64-bit kernel i386 structs reinterpreted at host width.
**It returns `STATUS_SUCCESS`, and that is the whole danger of the class:** the guest believes it
has a thread, the kernel believes it built one, and the trap frame is zeroed (`Rsp=0 Rip=0
SegCs=0`). The APC machinery then computes `0 - sizeof(frame)` and, before the bare-block fix
above, that killed the machine.

**Disposition: refuse, don't half-marshal.** It now returns `STATUS_NOT_IMPLEMENTED` loudly.
Marshalling the five structs would be *worse than nothing* — it would produce a thread that looks
even more plausible while still lacking the CPU-context setup a 32-bit guest thread needs, which
only a per-thread WoW64 entry can provide. **Losing a thread beats losing the box; a clean refusal
beats a convincing corpse.** Blast radius was *measured, not assumed*: exactly one call in a full
boot, from one applet.

### Ask where the payload LANDS: a divergence into global kernel state is a different bug
Most WoW64 divergences corrupt the guest that caused them. `NtUserInitializeClientPfnArrays`
does not: win32k `RtlCopyMemory`s the caller's `PFNCLIENT`/`PFNCLIENTWORKER` — arrays of
function pointers, 4 bytes guest / 8 host — into **`gpsi->apfnClientA/W/Worker` and
`aStoCidPfn`**, the *global* server info every process shares, and sets the global
`hModClient`. A 32-bit guest reaching that body is read for the HOST struct size (~2× its own
buffer) and installs pairwise-merged garbage as the **system-wide** client-callback table,
breaking native 64-bit callbacks. **Same divergence class, categorically larger blast radius —
so triage a finding by where the payload lands, not only by how the widths differ.**

It has never fired, for one reason: win32k early-outs on `ClientPfnInit != 0`, and native
user32 (csrss/winlogon) always wins that race long before a guest starts. **That is
initialization order, not a guarantee, and "currently unreachable by luck" is worth a
one-line intercept.** Fixed in the thunk layer, not win32k (the E13 rule), by returning
`STATUS_SUCCESS` without calling the kernel — which is precisely what the early-out already
produced, so observable behaviour is unchanged and the ordering dependency is gone. The guest
loses nothing: 32-bit callbacks are dispatched by the WoW64 callback bridge through
`Wow64pKiUserCallbackDispatcher32`, never through `gpsi`.

### An OUT `PHANDLE` divergence splits by ALIGNMENT — half silent overrun, half spurious failure
A `HANDLE` out-param is 4 bytes in the guest and 8 on the host, so a flat thunk hands the 64-bit
kernel a four-byte cell to write eight bytes into. The interesting part is that this does not
fail one way, it fails **two ways, decided by the address**, because the host's own probe is
width-dependent: `probe_write_handle` is `probe_write_user::<PVOID>`, so it demands 8-byte
alignment, and a 32-bit guest's `HANDLE` is only 4-byte aligned.

- Cell at ≡4 (mod 8): the kernel rejects the call outright with `STATUS_DATATYPE_MISALIGNMENT`
  (`0x80000002`) — a status the guest's own ABI says cannot happen for this call.
- Cell at ≡0 (mod 8): the probe passes and the kernel writes 8 bytes. Little-endian is what
  makes this the quiet half: the low dword lands where the guest expects, so **the guest reads
  a correct handle** while the high dword — and the probe's own NULL touch-test — zeroes the
  adjacent four bytes.

So the caller sees an intermittent, address-dependent failure *and* silent neighbour
corruption from the same bug. `NtOpenProcessTokenEx` + `NtOpenThreadTokenEx` were doing this
**12 times per amd64 boot**; the tell was `NtOpenThreadTokenEx` appearing *twice* per logical
open, the first returning `0x80000002`. After marshalling into a host-width local it is 1:1.
**Generalize the probe half: any host probe typed on a pointer-width value imposes host
alignment on a guest buffer, so `STATUS_DATATYPE_MISALIGNMENT` from a WoW64 guest is a
divergence signature, not a guest bug.**

Narrowing on the way back is not a truncation risk and needs no guard: a WoW64 process's
handles must fit in 32 bits by construction, or the guest could never name them.

**The reusable part is the triage, not the fix.** The audit baseline is a list of *possible*
divergences, and burning it down in file order wastes most of the effort — intersect it with
the syscalls a real boot actually dispatches (`[WOW64] <Name>(0x…)` lines) and the priority
collapses: of 195 divergent syscalls, **4** were called at all in a full boot, and of the 41
out-scalar-widen findings only **2**. Cold entries still matter for arbitrary apps, but they
are a tail to work through, not a queue to fear.

**But "cold" is a property of today's capabilities, not of the syscall.** Landing a capability
re-sorts the list under you: `NtWaitForMultipleObjects` sat in the baseline as a cold
`SCALAR_WIDEN` entry — its wait ARRAY is 4-byte handles in the guest against the host's 8-byte
ones — and went live the instant 32-bit guests could create threads, because
`WaitForMultipleObjects` is how a threaded app joins its workers. It returned
`STATUS_INVALID_HANDLE` on the first try. Single-object waits pass a handle *by value* and were
never affected, which is exactly why nothing had noticed. So re-run the live-set intersection
after every capability lands, and expect the newly-live entries to be the ones that capability
depends on.

### When the payload IS the CPU state, the CPU layer is the marshaller — not the kernel
`NtGetContextThread`/`NtSetContextThread` carry the same un-widenable `_CONTEXT` as
`NtCreateThread` above, and flat-thunking them is worse than usual in both directions: Get had
the 64-bit kernel write a 1232-byte host `CONTEXT` into the guest's 716-byte (`0x2CC`) buffer —
a >500-byte overflow of guest memory — while Set loaded the *native* thread's register file out
of i386 fields read at host offsets. Both returned `STATUS_SUCCESS`.

**The reflex is to refuse (as `NtCreateThread` does), and here that would have been wrong.** Ask
first *who owns the data*: the 64-bit kernel has no notion of the guest's registers, so it was
never a candidate — but the CPU layer already tracks them, and `BTCpuGet/SetContext` speak
exactly the i386 `CONTEXT` the syscall carries. `NtContinue` and the callback bridge were already
using those primitives. So the marshaller is real, and it is short. **Before writing a refusal,
check whether some *other* layer already holds the state in the guest's own width.**

Two traps, both from the amd64 CPU layer's contract (arm64's layer is FEX, which needs neither):
- **A saved context can be internally inconsistent.** `wow64cpu.S` saves the live `Esp` — still
  pointing at the stub's return address — but saves `Eip` as where the guest will *resume*.
  `Wow64CpuDispatch` reconciles the pair with `Esp += 4` on the normal return, so a snapshot
  handed to a guest must apply that itself or `Eip` and `Esp` disagree by one stack slot.
- **Resuming a replaced context bypasses the return-value fixup.** `BTCpuSetContext` flags the
  context as the state to resume, which skips `Wow64CpuDispatch`'s `Eax = result`, so the
  syscall's own return value has to be planted by hand.

Partial `ContextFlags` is the third: `BTCpuSetContext` installs the whole `SegGs..SegSs` window
unconditionally, so a partial set must read the live context first and overlay only the requested
groups. And since the CPU layer maintains no `FloatSave`/`Dr*`/`ExtendedRegisters`, a request for
those gets those bits **cleared from the returned `ContextFlags`** — the API's own way to say
"not supplied" — rather than a buffer that looks filled in and isn't.

Cross-thread stays refused: `BTCpuGet/SetContext` ignore their `Thread` argument and act on the
caller's CPU area, so answering a cross-thread query would return *the wrong thread's registers*
— confidently. Identify "is this me" by comparing `TebBaseAddress`, not just by spotting the
`(HANDLE)-2` pseudo-handle.

### In a WoW64 process the IMAGE is a PE32 — native code reading its headers must dispatch on `Magic`
`PEB->ImageBaseAddress` in a WoW64 process points at the **32-bit** exe, but the ntdll reading it
is built 64-bit, so `RtlImageNtHeader()` hands back a `PIMAGE_NT_HEADERS` that is the wrong
layout. The failure is not a wild pointer — the headers are real — it is a *field read at the
wrong width and offset*, which is far quieter.

`IMAGE_OPTIONAL_HEADER32` packs `SizeOfStackReserve` and `SizeOfStackCommit` as adjacent 4-byte
fields exactly where the 64-bit layout has one 8-byte `SizeOfStackReserve`. Reading at native
width therefore returns `(SizeOfStackCommit << 32) | SizeOfStackReserve` — for a stock 1MB/4KB
i386 image, **16 TB**. `RtlpCreateUserStack` fed that straight to `ZwAllocateVirtualMemory`,
which rejected it with `STATUS_INVALID_PARAMETER_4`, so **every** `RtlCreateUserThread` in the
process failed. Only a WoW64 process can reach this, which is why it survived until 32-bit
guests could create threads at all.

Fix: dispatch on `OptionalHeader.Magic` (`IMAGE_NT_OPTIONAL_HDR32_MAGIC`), never on the build's
own pointer width. `Magic` is at offset 0 of the optional header in both layouts, so reading it
first is always safe. **Generalize: in a WoW64 process, treat every PE structure reachable from
the PEB as foreign-width until you have checked.** The 64-bit loader reading a 32-bit image's
`IMAGE_TLS_DIRECTORY` `[§9.67]` is the same bug in a different field.

### An init that never returns leaves its caller's completion flag unset — and only thread #2 notices
`LdrpInitializeProcess` does not return in a WoW64 process: it loads wow64.dll and calls
`Wow64LdrpInitialize`, which drives the guest and never comes back. So its caller's tail —
`LdrpInLdrInit = 0` and `_InterlockedIncrement(&LdrpProcessInitialized)` — never runs, and
`LdrpProcessInitialized` is stuck at **1** for the life of the process.

Nothing notices while the guest is single-threaded. The moment a *second* thread starts, it
enters `LdrpInitialize`'s
`while _InterlockedCompareExchange(&LdrpProcessInitialized, 1, 0) == 1` gate — which means "some
other thread is still initializing the process, wait for it" — and spins on `ZwDelayExecution`
forever. The symptom is maximally unhelpful: a thread that is created successfully, resumes
successfully, and then produces **no output at all**, because it never reaches its start routine.

Fix: publish completion *before* the handoff, not after it. That is accurate rather than a
workaround — process init genuinely is finished at that point (`LdrpRunInitializeRoutines` has
returned); only the never-returning handoff remains.

**Generalize: any "and then we never return" call must first perform the bookkeeping its caller
would have done.** Grep the caller's tail before writing one. A flag that is only ever read by a
code path you cannot yet exercise is the definition of a latent bug.

### A per-arch `#[cfg]` on an address-space reservation = a TEB allocated on top of KUSER_SHARED_DATA
Four concurrent 32-bit guest threads killed the whole WoW64 process with `0xC0000005`, faulting in
the 64-bit `LdrInitializeThunk` before any guest code ran. The tempting read was "the CPU layer is
not concurrency-safe". It was not. Exactly **one** thread's TEB was wrong, and the serial log named
it without a rebuild:

```
Teb=0x000000007FFE0000
Teb.StackBase =0x000000000FD4BC00   <- KUSER_SHARED_DATA.InterruptTime.LowPart
Teb.StackLimit=0x90A70AA000000000   <- High2Time(0) | SystemTime.LowPart<<32
Teb.Dealloc   =0x0000000001310000   <- correct (offset 0x1478 = page 2, a real page)
```

Those are not corrupt stack bounds, they are **the clock**. The TEB sat at
`MM_SHARED_USER_DATA_VA` (0x7FFE0000), so everything below offset 0x1000 of it *was*
KUSER_SHARED_DATA. WoW64 PEB32/TEB regions are capped at the 32-bit `MM_HIGHEST_USER_ADDRESS`
(0x7FFEFFFF) so guest pointers can reach them, and they descend 0x3000 apart — the 5th thread
landed on the shared page.

Cause: `MiInsertSharedUserPageVad`, which reserves that page with a real VAD so the allocator
knows it is taken, was `#[cfg(target_arch = "x86_64")]`. **amd64 had the reservation; arm64 did
not.** That is why the same arch-neutral WoW64 code passed all nine rungs on amd64 and died on
arm64. Fix: extend the `cfg` to `aarch64`. Plus a tripwire in `MiCreatePebOrTebCapped` rejecting
any PEB/TEB VAD that straddles `MM_SHARED_USER_DATA_VA`, because the failure is otherwise silent
until a loader crash N threads later.

Two things disguised it as a concurrency bug. It needs five threads to reach the page, so it looks
load-dependent. And it kills threads whose own TEB is **fine**: `MmCreateTeb` zeroes the new TEB,
wiping the process-wide KUSER_SHARED_DATA that the guest's i386 ntdll — and our own `wow64.c`,
which reads `NtSystemRoot` straight out of `0x7FFE0000+0x30` — depend on. The time fields refill
within one timer tick, which is why the overlaid TEB reads back a *live* clock rather than zeros.

Do **not** "fix" this by lowering the cap to `0x7FFEFFFF - 16 * PAGE_SIZE` (the native
`MM_HIGHEST_VAD_ADDRESS` shape). Tried first, and it moved PEB32/TEB below 0x7FFE0000 and broke
WoW64 bring-up outright — an untested address-layout change bought nothing the real reservation
does not.

**Generalize: an allocator only avoids what it has a record of, and a `cfg` is a place a record
can be missing on one arch only.** When a bug reproduces on one arch and not another in
arch-neutral code, diff the *cfg-gated setup* the two arches ran, not the code that crashed.

### `ThreadZeroTlsCell` from a 32-bit guest zeroes the WRONG TEB — and wipes WoW64's own TLS
A guest that had passed every threading rung still died at exit. The proximate crash was a NULL
deref *inside* `Wow64pVeh` at `CpuArea+0xC0` and `+0xBC` — which are `WOW64_CONTEXT.Eax`/`.Ecx`,
i.e. `TEB->TlsSlots[WOW64_TLS_CPURESERVED]` had become NULL on a thread that was still running
guest code.

Cause: `NtSetInformationThread(ThreadZeroTlsCell)` walks every thread in the process and does
`Teb->TlsSlots[TlsIndex] = NULL` on the **64-bit** TEB. A 32-bit guest's TLS cells live in
**TEB32**; TEB64's low slots belong to WoW64 itself (slot 1 is the per-thread guest register
file). So a guest calling `TlsFree(1)` — perfectly ordinary cleanup — wiped wow64.dll's CPU-area
pointer for *every* thread in the process. Fix: in a WoW64 process, zero `TEB32.TlsSlots`
(0xE10) / `TEB32.TlsExpansionSlots` (0xF94) instead.

Two lessons beyond the fix:

- **A syscall can be "correctly implemented" and still be a WoW64 bug.** No marshalling was
  missing here — the argument is a plain `ULONG` at the same offset in both worlds, so no
  layout-diff tool flags it. What differs is *which structure the side effect lands in*. Audit
  syscalls by the state they mutate, not just by their parameter layout.
- **Never dereference the CPU area without a NULL check in an exception path.** `Wow64pVeh` and
  `Wow64pPrepareForExceptionHook` both did `(WOW64_CONTEXT *)(CpuArea + 16)` unguarded. A VEH runs
  on *any* thread, including one that never entered the guest or is past
  `Wow64FreeThreadCpuArea` — and faulting inside the handler re-enters it and **buries the
  original exception**, which is what hid this bug behind a bogus `libwow64fex` address for a
  whole session. Declining is not enough on its own either: FEX's `BTCpuResetToConsistentState`
  NtContinues back to the faulting instruction, so a handler that merely returns
  `EXCEPTION_CONTINUE_SEARCH` turns the crash into a livelock (31,511 iterations). Guard, log,
  and fix the cause.

### The shared USER handle table has no 32-bit view `[§9.58b]`
Client-side `ValidateHwnd` fails, so `DispatchMessage` silently drops `WM_PAINT`. **This is a
subsystem, not a marshaller** — WoW64 GUI needs a per-process 32-bit view of USER shared memory
(SERVERINFO, SHAREDINFO handle entries, DESKTOPINFO). The first instinct — a win32k-maintained
32-bit *shadow* — was rejected in favour of `user32-rust/src/wow64.rs` building the view
client-side. The (a) half was a cursor/icon align-8 UNICODE_STRING probe → 0x1E `[§9.58a]`.

### Loader ordering and process bring-up `[§9.47]`
A CUI process hangs in kernel32 `ConDllInitialize` with exit code **0x103 = STILL_ACTIVE**
(alive and hung, not a crash code). A dependency that imports kernel32 must not be `LdrLoadDll`'d
from early init. csrss dying 0xF4 when a wow64 process connects was a separate blocker — and a
**stale ramdisk** can reproduce it as a false red on a non-FEX gate.

### Un-gating a `.spec` export shifts ALL later ORDINALS `[§9.43-spec]`
`spec2def` assigns ordinals **sequentially over the entries it emits**, so un-gating a
`-version=0x600+` line renumbers everything after it and breaks by-ordinal importers → BUGCHECK
0x4C `0xc0000139`. Confirm by stashing just the `.spec` and rebuilding that one arch. Fix by
arch-gating the new exports, or pinning **explicit ordinals** on the new lines.
**Adding a named export is not free in an auto-ordinal spec.**

### A DLL "loads fine" then crashes at PC=0xFFBADD11 `[§9.44-poison]`
Our ntdll loader intentionally **poison-snaps** missing by-name imports instead of failing the
load, so the failure surfaces at the first *call*. Diagnose by diffing the DLL's imports against
the exporter. Arch-gated spec **forwarders to msvcrt** cost zero code. Trap within the trap:
ntdll ships via the CARGO link, so the usual rebuild path doesn't apply.

---

## win32k, GDI, input & desktop

### The systemic win32k flip failures
win32k is the largest single flip surface and its failures are stereotyped:
- **Links clean ×3 but explorer BUGCHECKs 0x1E `0xc000001d` at first `CreateDC`** → a folded
  ud2 from a guessed calling convention. **The #1 i386 win32k crash cause, solved
  deterministically by an ABI-overwrite pass**, not per-site repair. (A cfg-blind dedup once
  reverted `enforce_win32k_abi`, producing "no symbol" + "expected item after attributes".)
- **Links clean ×3 but explorer HANGS on the CreateDC path** — no bugcheck; the tell is serial
  volume (175 KB vs 676 KB green).
- **cargo-check GREEN but the `.sys` link fails on ONE arch only** —
  `@RECTL_bUnionRect@12`/`@REGION_X@N` (fastcall decoration), a table-referenced function
  (`DibFunctionsForBitmapFormat`) with wrong cdecl decoration, or the `GDIOBJ_*` family macros
  translated as externs.
- **cargo-check GREEN but link fails "SIMD vector type f64 requires sse"** on the i386 kernel
  target — a float crate pulled into the kernel.
- **Cross-crate type-namespace collision** — `expected ntkrnl_sys::ddk::_X, found
  win32k_sys::win32k::_X`. A glob re-export in a `*-sys` crate produced 118 E0659+E0308 errors.
- **`#[no_mangle] fn XFORMOBJ_iSetXform` emits the wrong symbol** where callers reference
  `EXFORMOBJ_*`.
- **The USER critical section (618 sites) was over-engineered** as a combined-guard problem.
- **A fn PARAMETER named like a glob-imported `const` becomes a const-PATTERN, not a binding** —
  the body then sees the const's type (`expected *mut _DDHAL_UNLOCKDATA, found u32`, plus E0614
  at every deref). The win32k-sys glob re-exports thousands of consts, and DirectDraw HAL params
  (`Unlock`, `Lock`, `Flip`) are prime collisions. **The dangerous variant is SILENT:** if the
  const's type is compatible, it compiles and the function reads the const instead of its
  argument. Prefer Hungarian `p*`/`lp*` parameter names against a big glob.

### Input path bugs
- **x86 PS/2 input was COMPLETELY dead — four stacked root causes** `[§9.27-ps2]`, found only by
  the Doom playable-gate: acpi-rust's **invented `CmResourceType*`/`ACPI_*` constants** broke PnP resource
  translation `[§9.37]`; i8042prt's mouse reset state machine stored **invalid enum discriminants**;
  winlogon loaded keyboard layouts **only on arm64**; and the input desktop can be stranded on
  "Winlogon" on x86 too. **When a device is dead but boot is green, walk the PnP resource
  translation first.**
- **`DO_BUFFERED_IO` translated as 0x10 (= `DO_DIRECT_IO`)** → every arm64 input packet
  `memcpy`'d to NULL `[§9.34]`. **When a reader sees ALL-ZERO structs the writer provably
  filled, suspect the transfer-mode flag.**
- **The first keystroke BUGCHECKs 0x1E** `[§9.38]` if the i8042prt DPC calls kbdclass
  `ClassService` as `extern "C"` — it is NTAPI stdcall on i386.
- **winlogon 0xF4 on the first FAILING user-mode callback** `[§9.39]` — i386
  `KeUserModeCallback` conflated the SEH status with the callback's return status and skipped the
  user-ESP restore. The `'seh` block must yield `Result<NTSTATUS, NTSTATUS>` so the two are
  distinguishable.

### GDI rendering bugs
- **A dropped `!` inverted `IgnoreSrc`** in `DIB_1BPP_BitBltSrcCopy_From1BPP`, so every identity
  1bpp→1bpp SRCCOPY **zero-filled its destination** → black desktop shortcut icons `[§9.33]`.
  **A boolean built from TWO C idioms (`!!x`, `x ^ !y`) is a prime inversion candidate.**
- **The GDI TEB batch was never flushed** `[§9.41]` — a hand-rolled `TEB.GdiBatchCount` offset
  read **0x74 instead of 0x1740**. One bug, four symptoms (Ctrl+Esc killing explorer, blank
  start-menu labels, CLIENTINFO corruption, a user32 `ValidateHandle` AV). Detector:
  `scan_magic_offsets` shape `TYPED_PTR_HEX_ARITH`. **Use a typed read**
  (`(*(teb as *const _TEB)).GdiBatchCount`), never pointer arithmetic on a hex literal.
- **`DesktopHeapReAlloc` passed the PDESKTOP as the HEAP pointer** `[§9.40]` (and never freed the old
  block) → amd64 0x1E on the first start-menu open. **When a Rust file reimplements C header
  inlines, diff EVERY helper, not just the one that crashed.**
- **amd64-only float bugcheck 0x1E `0xC0000090`** — MXCSR unmasked in win32k.
- **amd64-only `0xc0000024`** in win32k `ObReferenceObjectByHandle` (`ExEventObjectType`).
- **amd64 0x7F P1=8 (DOUBLE FAULT) in GUI-thread deep unwind chains** — and separately, the
  kernel shipped **ZERO unwind info for Rust code** `[§9.42]` (`requires-uwtable: false` in the
  custom target JSONs), so any exception walk died at the first Rust frame. One-word fix.
- **A cross-thread `WaitForSingleObject(INFINITE)` on a cosmetic UI element is a boot-wedge**
  `[§9.12]` — the amd64 status-window hang (~1-in-3 cold boots) in winlogon init. The class:
  `INFINITE` cross-thread waits where the wakeup lives on a window that may never be created.
  Fix defensively: **bounded wait** (20s) and **deep-copy** the title/message into worker-owned
  heap before handing it over. Localized by **breadcrumb bracketing**, because it was a timing
  race and prints move it. *(Not reproducing as of 2026-07-17 across 20 boots; a tripwire is
  armed — re-open only on a `[MSGINA] 9.12-TRIPWIRE` serial line.)*
- **Intermittent csrss self-terminate → 0xF4** was a thread-hash bucket off-by-one.
- **arm64 0xF4 winlogon death right after `[WIN32K] shell ready`** — a distinct, historical
  shape worth recognising.

### Driver settle delays are pure loss in a VM `[§9.22]`
Hardware drivers inherited from real-metal code are full of fixed settle delays. A boot phase
that is "slow" with zero serial and no CPU is the shape. Gate them on the driver's **own** VM
detection — uniata's 12 × `AtapiSoftReset` cost **2.85 s/boot**. Recipe: permanent `DbgPrint`
phase markers at the suspect boundaries, then read the timestamps.

---

## Per-arch quick reference

**The method that matters:** *i386 hides 64-bit bugs (narrower pointers, different layout luck)
and 64-bit hides i386 bugs (`extern "C"` == `extern "system"`).* Build and boot all three, every
time.

| | i386 | amd64 | arm64 |
|---|---|---|---|
| `extern "C"` vs `"system"` | **cdecl vs stdcall — diverge** | identical | identical |
| Extra convention | fastcall `@name@N` | — | — |
| Decoration | `_n` / `_n@N` / `@n@N` | bare | bare |
| `KERNEL_STACK_SIZE` | 12 KB | **48 KB** | 12 KB |
| `HIGH_LEVEL` | 31 | 15 (CR8, 4 bits) | 15 |
| IRQL raise provider | ntoskrnl (+`@KfRaiseIrql@4`) | **neither — use CR8 asm** | hal (halstubs) |
| `KeGetCurrentIrql` | real export | FORCEINLINE | FORCEINLINE |
| `KeQuerySystemTime` | real export | SharedUserData inline | real export |
| Floats | **no SSE: no by-value f32/f64** | fine | fine |
| Boot media | LiveCD ISO (`ninja livecd` repacks) | LiveCD ISO | **ramdisk — staged set is partial** |
| Baseline boot | ~1m48s | ~3m45s | ~40s |

**Per-arch smoke marker is the same on all three:** `[EXPLORER] _tWinMain entered` + `shell
ready`. `shell ready` alone is an intermediate **milestone**, not the verdict.
---

## Crash-hunting toolkit

The validated hunt loop `[§9.8]` and the toolkit design live in `kernel-debugging-infra.md`. Front door: `./vibeos-cli debug`, `smoke --tripwire`,
`farm`.

### Arm the tripwire BEFORE doing forensics
For any corruption bug, **special pool first** `[§9.18]`. It faults **at the corrupting store**
with the culprit PC symbolicated, instead of an innocent-victim BUGCHECK minutes later. Armed
per-boot over the fw_cfg channel — **no media rebuild**:
`./vibeos-cli smoke --arch amd64 --tripwire "SPECIALPOOL=Gla5"` (or `SPECIALPOOL=*`). For a rare
flake use `farm` + `farm-report`, never a hand-rolled boot loop.

### Breakpoints and watchpoints require TCG
**KVM fires neither gdb breakpoints nor hardware watchpoints** `[§9.1]`. Prepend `VIBEOS_NO_KVM=1` for
i386 hunts. Do **not** switch the smoke backend to native KVM to "repro faster" — it hits the
per-arch accelerator cap, so a parallel repro just queues.

### Symbolication
- **i386 Rust-in-ntoskrnl carries a `+0x291000` shift** `[§9.2]`. C: `file_VA = runtime - 0x7fe00000`.
  Rust: additionally `- 0x291000`. One-shot for both:
  `qemu/addr2sym.sh i386 --reloc 0x80091000 <addr>`.
- **win32k.sys lives in SESSION space (~`0xf74xxxxx`)**, not `0x80xxxxxx` — **re-derive its base
  after any win32k rebuild.** A stack-scan filter that only accepts `0x80200000..0x80900000`
  misses every win32k frame.
- **The offline index is NOT symbol-poor — it resolves function + source line** `[§9.137]`. Nine
  i386 quarantines waited ~13 days for a GDB session because their notes said otherwise. All
  `addr2sym.sh` needed was `--reloc`, because the index holds **file** VAs while a bugcheck reports
  **runtime** VAs. **Derive the base without a debugger:** every recorded
  `module+<offset> (<runtime addr>)` pair *is* an equation for the load base, so several agreeing
  pairs give it, minus `ImageBase`.
  > **Do not accept a base because one lookup "looks plausible"** — that is how a wrong base
  > produces a confident wrong function and relocates the hunt. The control is *semantic*: each
  > address must resolve to a function whose **name matches the test that crashed**. And use
  > **FRESH** addresses for that control, never recorded ones: recorded addresses predate weeks of
  > code motion, and in one set a stale address resolved to a different function in a different
  > subsystem entirely, which would have sent the hunt to the wrong module. A stale match is
  > corroboration at best.

  Function-level answers survive code drift; exact offsets do not. **arm64 has no equivalent yet** —
  its kernel runtime base is never printed on serial and cannot be recovered from a recorded pair,
  so offline arm64 triage is blocked on a one-line boot-time print of the base.
- **The same pair-derivation works for DRIVER bases, which is the bigger blind spot** `[§9.137a]` —
  the symbol index covers `ntoskrnl` only, so a PC landing in a `.sys` resolves to whatever kernel
  symbol sorts below it (one produced `STATUS_WAKE_SYSTEM+923526268`). Subtract a known
  `driver+<offset>` from the faulting PC; a **page-aligned** result is a 1-in-4096 coincidence, so
  it is evidence — but **driver bases are weaker than the kernel's**, because drivers load
  dynamically and a base from one boot need not hold for another. Confirm semantically: file VA =
  `ImageBase + RVA`, then `llvm-nm --numeric-sort` the `.sys` and check the symbol is a real
  function in the module you expect. **That is what licenses the base — not the page alignment.**
  In one case the symbol also settled the *classification*: the crash was in real fast-I/O code
  correctly guarding a raw user buffer with `_SEH2_TRY`, not a parameter-validation stub — so under
  i386 DUMMY_PSEH the dead guard is a **production** defect, not a test-harness artifact.
- **When `objdump`/`nm` come back empty** (cargo-built kernel): `qemu/build-index.sh <arch>` →
  `qemu/symbols-<arch>.tsv`, calibrate the runtime load base (the kernel is relocated), then
  `rva = addr − base` and take the nearest symbol ≤ rva. **If `rva` is past the tsv's max symbol,
  the address is not in this image** — that inference is the useful half. To identify a raiser
  directly, hook the `ExRaiseStatus` shim.
- **`find-kbase` works** (fixed for PE32+); `list-threads` still needs Rust DWARF in the linked
  image, which the release kernel lacks. Getting DWARF in is a **split-debuginfo** problem
  (`-C split-debuginfo=packed`, or `objcopy --only-keep-debug`): a plain workspace
  `debug=true` bloats the kernel until freeldr dies at "Switching stack…", and a per-package
  `debug=2` silently does not apply under the `-Z build-std` json-target build.
- **Rust release omits frame pointers** `[§9.5]`, so an EBP-chain `bt` is worthless. **Scan the raw stack
  upward from the fault ESP** for values in the kernel/win32k `.text` ranges — those are the call
  chain. On arm64, walk `Fp`(x29): `lr = *(fp+8)`, `next = *(fp)`.

### Watchpoint on a not-yet-mapped page `[§9.4]`
A session/desktop-heap address isn't mapped at boot, so sequence it: `hbreak` a function that
runs *after* the page maps but before the corruption, then set the watchpoint **conditioned on
the corruption signature** — e.g. `watch *(u32*)ADDR if ((*(u32*)ADDR)>>24) != 0xbc` (the
flink's high byte no longer matching the heap page). Condition on the signature, not on any
write.

### In-kernel validators beat watchpoints for heap desync `[§9.3]`
A capped, scoped, heap-relative free-list walker per alloc/free. Three properties that make it
usable: **capped** (bail after ~20; don't flood serial), **kernel-heap-only** (gate
`heap >= 0x80000000`, else user-mode heaps at `0x130000` flood it — including ntdll's *separate*
Rust heap copy), and **heap-relative sanity** (`flink.wrapping_sub(heap) < 0x4000000`). Then
**name the culprit** by stack-scanning from ESP. **If only alloc-ENTRY trips, corruption is not
introduced at any heap exit — it is an external stray write.**

### Reading a hang correctly
- **Slow ≠ hung** `[§9.7]`. A no-gdb KVM boot to a post-marker BUGCHECK is ~80–150 s; **wait
  ≥150 s.** A trickle of early serial (87/581 bytes) is CD load, not a hang.
- **A crash that varies its victim per boot is a CASCADE from an earlier first cause** — chase
  the earliest detectable corruption, not the visible victim.
- **A foreign binary that is stuck is usually one of three shapes** `[§9.29]`: sample the guest
  PC a dozen times via QMP `info registers` — all samples at one PC means a spin; parked in a
  wait with no file I/O (break on `NtReadFile`/`NtDelayExecution`); or spinning at a fixed
  usermode PC that disassembles to `b .` (`-O0` is the workaround until the pass is identified).
- **The smoke `full.log` is a TRUNCATED TAIL** (~140 lines). For a full capture read
  `qemu/<arch>-uefi_smoke_live.log`. This cost ~6 build cycles once.
- **Read `summary.json`, don't scrape stdout**, and never pipe `vibeos-cli` through `| tail` (it
  swallows the exit code).

### Localization techniques that repeatedly worked
- **DbgPrint-bisect with REAL `DbgPrint`.** Translated `TRACE!`/`WARN!`/`DPRINT` are no-op
  macros, so **their absence in the serial is NOT evidence of where execution stopped** — that
  false signal has cost multiple detours. Bracket each init step *and each callee*; the hang sits
  at the last printed trace's **next** call.
- **Breadcrumb bracketing for timing races**, because any print may move the bug.
- **Paired breadcrumbs across a boundary** (driver push vs consumer pop) localise a transfer bug
  in one boot.
- **Elimination when prints move the bug** — the SamSs 30s SCM stall was a Heisenbug where *any*
  `DbgPrint` in the handshake made it connect `[§9.89]`. Root: a cross-process handshake gated on
  a shared **named event** both sides create. **Rendezvous on the PIPE, not the event.**
- **Probe the LAST consumer, not the producers**, when N derived stages could each be wrong.
- **Instrument reachability before assuming a verbatim upstream bug matters** `[§9.79]` — one was
  provably **inert**. In the same port, a signed-char → WEOF trap **was** load-bearing, and an
  i386 x87 float-ABI wall turned out to be readable **backwards** on first analysis.
- **A control experiment beats reasoning.** `git stash` the delta, rebuild, boot: clean
  attribution in one cycle.
- **Check whether the owed fix is already in the tree, and read the LIVE Rust** `[§9.82]`. A
  tracked "winlogon 0x4C init-ordering race" was carried for weeks; the fix had been implemented
  the whole time — the C that "showed" the bug is **inert**. **The evidence never fit the
  hypothesis either**, and the tripwire's own comment recorded that. Leads checked and closed
  while there (all plausible, none the bug): struct-layout drift for
  `_BASE_STATIC_SERVER_DATA`, `CSR_SERVER_DLL.SharedSection` offset, shared-section view delta.
  **An inherited hypothesis is not evidence. Re-derive it from the live code before spending a
  boot on it.**
- **A structured "garbage" value (a small constant like 0x10c) on the stack is not random** —
  re-read the victim frame instead of hunting the writer. That is what cracked the i386
  `0x0A @ Eip=0x10c` post-`shell ready` crash `[§9.26]`: **a stdcall→fastcall shim tail-`jmp`
  leaked 4 bytes of stack per call**, and `KiCheckForApcDelivery`'s esp-relative loop `ret`ed to
  a stale slot. Fix: shims `call @Kf*Irql@4; ret 4` — clean their own frame.
- **Ground-truth an inherited bug list before working it** `[§9.27-ground]`. Of "3 open corruption
  bugs": 2 did not reproduce, 1 was reframed. **And validate the tool first** — the
  `SPECIALPOOL_SELFTEST=guard` run exists for exactly that. In the same pass, a plausible
  IRQL-leak hypothesis was **disproven live**.

### x86 UEFI boot bring-up (OVMF)
Three gotchas latent in the C, which surface only once you actually boot UEFI:
1. **i386 PAE→non-PAE transition** — OVMF hands over with paging ON and `CR4.PAE=1`; writing a
   non-PAE PDE to CR3 while PAE is set makes the MMU read it as a PAE PDPT → **instant
   triple-fault.** Clear PAE while paging is OFF.
2. **Synthetic ACPI 2.0 RSDP is required**, else no PS/2 input. Tell: zero `[[I8K]]` lines and
   `Tables: [XSDT]` (vs BIOS `[RSDT]`) plus `Unable to AcpiInitializeTables`.
3. **freeldr-loaded DLLs must not use raw-dylib** (see the cohab section).

**Recipe:** i386 = `qemu/OVMF32_CODE.fd` + `OVMF32_VARS.fd`; amd64 = the system `OVMF_CODE_4M.fd`.
**`-device ide-cd,drive=cd0,bootindex=0`** — a bare `-cdrom` drops to the UEFI shell. The
writable VARS pflash must be **qcow2** for QMP `savevm` to work. **Re-run `ninja livecd` after
touching the loader** — the El Torito entry folds in `uefildr`.

---

## Harness flakes, contention & host starvation

**Before declaring any smoke red, check whether the host was busy** (`./vibeos-cli ps`,
`./vibeos-cli host-status`). A lone red while another arch is mid-gate is a prime contention
suspect; re-run host-quiet to decide.

### Contention wears three costumes, not one
1. **`HANG_TIMEOUT` at `[freeldr] Switching stack...`** with zero kernel serial (the CGU lottery,
   above).
2. **A transient BUGCHECK** — e.g. `0x1E (0xc0000006 IN_PAGE_ERROR)` under memory/IO/KVM
   pressure. The identical binary boots green ×5 on a quiet host.
3. **A late-boot hang that REACHES `shell ready`** — explorer never reaches
   `_tWinMain entered` within the timeout, because a normally-1m48s i386 boot drifts past 6 min.

### `NET_FLAKE` that is not a flake `[§9.84]`
A gate showing `✓ build ×3 ✓ lint ×3 ✓ smoke: all three` and a red net suite, where **all 15
tests error (not fail) with the same 600 s number, first test included**, means the fixture never
booted — another lane's `farm` owned every host CPU lease. **Tell it apart in ten seconds:** a
real net regression fails *some* tests with *varied* timings; starvation errors *all* of them
identically. **Do not `--allow-stale`, do not re-gate** (it will starve again), and above all
**never broad-kill QEMUs** — use `./vibeos-cli kill`. Starvation must be *visible*: it used to
read as "my build is slow".

### Known-benign signals — do not chase these
- **A screenshot that looks solid black (346-byte PNG) is NOT a boot failure.** More generally,
  the screenshot is captured **at** the marker, before the taskbar finishes rendering, so a green
  run routinely shows bare wallpaper. **An empty tray is not a failure signal** — the `verdict`
  and boot duration are.
- **`cmlib\hivewrt.c:360/522 "Failed to write the primary hive"` is benign RO-media noise**,
  present on **every** boot including green ones. It was a red herring for a whole session.
- **The `…start failed: Unsupported` arm64 line is EDK2 firmware noise.**
- **`post_shell_ready` input/mouse smoke failing is a KNOWN FLAKE**, not a kernel bug.
- **A serial logfile stuck at 0 bytes** through an entire boot is a harness/plumbing problem.
- **The pre-existing ~33% i386 flake** where `winlogon` unconditionally spawns `nettest.exe`,
  racing explorer's csrss/win32k handoff, HANGs **even host-quiet**; the tail shows
  `[WINLOGON] nettest.exe created` with no `_tWinMain`. Distinct from contention.

### CRT / cnt runtime one-offs
- **cnt `swprintf` writes nothing → 0x1E** — a signed/unsigned `_cnt` compare.
- **cnt `strncmp`/`strncpy`/`strncat` source over-read → 0x50.**
- **"SIMD f64 requires sse"** = a float crate pulled into the i386 kernel.
- **`KeBalanceSetManager` spinning ~40% CPU** (amd64 boots 15+ min) was a timer *type* bug.
- **Folding ke-inlines/ke-macros back into ntoskrnl-rust → triple-fault, 0 serial bytes.**
- **Script-rewriting `#if`/`#elif` chains** produces tree-wide "unterminated conditional
  directive".

---

## Translating a shared C runtime library (Rtl-class)

Byte-diff verification is the methodology that works: build the C and Rust bodies, then diff
their output byte-for-byte over a generated corpus rather than trusting a spot check. The
symptom→cause→fix cases are folded into the sections above (constants, element-vs-byte counts,
`n`-suffixed scans, slice-vs-single-pass).

## Translating a COM DLL with a private header (ole32/oleaut32-class)

**The model owns bodies — never interfaces, vtables, offsets, lifetime, or activation.** Asking
an LLM to translate an ATL class as one blob lets it invent exactly the parts that must be
derived. `rustconv cpp-com` derives the class COM map from the defining C++ header; `vtbl_gen.py`
generates the vtable from the C initializer plus bindgen. The three things the translator
silently omits are the vtable layout, the aggregation/delegation wiring, and the registration
data. **Avoid the shell32-com C++ substrate for new work** until that lane lands.

### A "not the COM path" flag turned off the LANGUAGE layer too `[§9.131]`
`--free-functions` exists because a TU's classless functions need no COM scaffold — no vtable, no
`this`, no state layout. It was implemented as one boolean, and that boolean also gated everything
about **C++ the language** — but a free function in a `.cpp` is still C++. Three deterministic
lowerings vanished at once (wide literals, an interface-pointer macro, and body-local RAII types),
all found by *running* the path on one 172-line TU rather than reading it.

> **Rust has no UTF-16 string literal.** So for `L"text"` there was no correct output available and
> the prompt was asking the model to invent one — and a confident invention is what you get. **Any
> construct with no spelling in the target language must be lowered deterministically before the
> prompt is built; it can never be a model responsibility.**

Two general lessons:
- **When you add a flag meaning "not the X path", enumerate what else lives behind X.** This one
  bundled framework concerns (vtables, `this`) with language concerns (literals, macros, RAII) under
  a single name. Splitting the *call sites* is not enough — you have to split the *concepts*.
- **An empty allowlist is a licence to invent, not a neutral default.** The supported-RAII set held
  exactly one type, so every other body-local C++ owner sailed past the "unsupported RAII" gate and
  went to the model. **Absence from a refusal list reads as "supported".** The fix costs several TUs
  a refusal until someone declares them, which is the correct trade: **a loud refusal beats a paid
  invention.**

Also: **normalization must run on cache HITS, not just fresh responses.** A body banked before a
repair exists otherwise replays the un-repaired text for $0 forever — the same self-healing shape as
the sealed-output guard. Because that lowering happens *before* sealing it is part of the cache key,
so only the bodies that actually changed re-paid.

---

## Working in this repo without breaking a sibling lane

### A worktree showing a HUGE staged REVERT isn't your work `[§9.121]`
`git status` in a checkout you haven't touched reports thousands of files staged, nothing
unstaged, and `git diff --cached --shortstat HEAD` shows six-figure **deletions** that remove
landed work. **Nobody reverted anything:** the worktree's *files* are at an older commit while
its *branch ref* was moved forward from a **different** worktree (`git branch -f`,
`update-ref`, `fetch origin master:master`, `push . <sha>:master`). Git diffs index+worktree
against the NEW `HEAD`, so every commit in between appears inverted.
- **Diagnose in three commands, proving there is nothing unique to lose:** `git reflog master -5`
  ("branch: Reset to HEAD" ⇒ moved from elsewhere — and note `HEAD`'s reflog is **per-worktree**,
  so it has no entry for the move and its `HEAD@{0}` disagrees with `git rev-parse HEAD`, which
  is itself the tell); `git diff --stat <that-commit>` (EMPTY ⇒ your files are exactly that
  tree); `git merge-base --is-ancestor <that-commit> origin/master` (YES ⇒ already upstream).
- **The trap:** `git add -A && git commit` there creates a **real** mass-revert commit with a
  plausible diff. And the obvious cleanup (`git checkout .` / `git restore`) is forbidden by
  AGENTS.md.
- **Fix:** `git stash push -m "<scope>: …"` as cheap insurance, then `git fetch && git merge
  origin/master`.
- **Prevention (now a rule):** never move a branch ref from a worktree that doesn't have it
  checked out. Push straight to origin (`git push origin HEAD:master`) and let each worktree pick
  it up itself. `vibeos-cli land` does exactly that, which is why it never touches local
  `master`.

---

## Still open

Everything else in this file is fixed or closed. These are the live ones — check the tree before
trusting any of it, and see `docs/product/vibeos-roadmap.md` → "Known bugs & flakes" for the
authoritative registry.

| Bug | State |
|---|---|
| **`-smp` `0xF4` boot-phase flake** (~2/20 soak), described as §9.56-class | Separate from the ready-list tear, which is **fixed** — see `active-lanes/smp-bringup-quest.md`. |
| **arm64 physical-page reuse** — a frame backing a still-mapped user page is freed and re-handed-out `[§9.98]` | Characterized, two faces proven to be one root; ~40% of arm64 kmtest sweeps. Free-while-mapped tripwire negative. |
| **arm64 user-mode SEH first-chance** — `RtlRaiseException` unimplemented | Needs a `#[naked]` capture stub. |
| **`ExHardError`** pool illegal-instruction; **`MmSection`** in-kernel wedge `[§9.92]` | Distinct from the arm64 kernel-SEH cluster; both open. |
| **arm64 exception-dispatch recursion** under a dropped probe `[§9.104]` | Layer 1 fixed; layer 2 is the deep root. |
| **WoW64 async IOSB** — the host IOSB is a stack temporary `[§9.62]` | Proper fix is kernel-side and owed. |
| **`cohab_staleness_reasons`** false-REDs on a codegen-neutral edit `[§9.81a]` | Real, narrow, still there. |
| **`write_bytes` variable-count on `*mut u16`** | No mechanical oracle possible (no type info in tree-sitter); manual review only. |
