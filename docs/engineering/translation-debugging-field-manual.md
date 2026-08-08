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

**Detection.** `tools/scan_cc_mismatch.py` (same-crate decl-vs-def),
`tools/scan_import_cc.py` (cross-MODULE decls vs the .spec of the DLL that exports them,
scoped by `add_importlibs` — the usermode half, and the only one that sees a wrong `extern
"system"`), `verify_abi.py` (extern blocks vs `ntoskrnl.spec` + bindgen, `extern "C"` only),
`verify_ffi.py` (workspace-wide),
`tools/scan_ir_mismatch.py` (catches the fold *before* it folds, from `.ll`). Refined
fold-truth model `[§9.102]`: **CC mismatch folds against both defs and declarations**;
**signature mismatch (arity / int width / void-vs-value) folds only against a definition**;
**variadic-call vs fixed-arity def folds.** Align decls by rewriting to the def's IR shape
using always-resolving tokens — do not "simplify" the def to match a decl `[§9.107]`.

**The blind spot all the spec-based checkers share:** an *internal*, non-exported helper is
invisible to spec/bindgen cross-checks, and if it is declared only once (wrongly) the
"appears under both ABIs" scan misses it too. The scan that catches these is **decl-ABI vs
def-ABI per symbol, over the crate's own symbols.**

### Same convention, different `@N`: stdcall decoration is a free ABI *width* checker

Not every `undefined _Foo@N` is a convention mismatch. When BOTH sides are `extern "system"`
and the numbers merely differ, `@N` is the **byte count of the argument list**, so the delta
is telling you an argument's *width* disagrees — and that is a bug the 64-bit arches cannot
see, because they have no decoration to disagree about. amd64/arm64 link it green and pass a
wrong layout.

Read the delta arithmetically before you touch anything. i386: every arg rounds to 4, and a
64-bit scalar or a by-value struct takes its real size.

    ld.lld: undefined symbol: _FatAddMcbEntry@20     # caller: 5 args x 4 = 20
    definition exports                @24            # 4+4+4+8+4 -> one arg is 8 bytes, not 4

That was exactly it: `fsctrl.rs` widened `Lbo` from `u32` to `i64` (LBO is LONGLONG — "for
Fat32, LBO is >32 bits", fat.h:34) and fixed one of the three `extern` decls. The other two
kept `i32`/`u32`. A sibling break in the same driver had the *other* shape —
`_FatStringTo8dot3@12` vs a definition at `@16`, because a caller declared an `OEM_STRING`
parameter as `*mut OEM_STRING` (4) where the definition takes it **by value** (8).

So: an i386-only `undefined _Foo@N` after someone widens a type is a **real cross-arch ABI
bug that i386 caught for you**, not an i386 quirk to work around. Fix every decl to match the
definition, and fix the call-site casts with them — a widened parameter usually needs the
argument expression widened too, or you have only moved the truncation.

**The NO-`@N` variant: an `undefined _Foo` with no suffix at all is a Win32 API you declared
`extern "C"`.** WINAPI is stdcall, so on i386 the convention IS the symbol name —
`_SetErrorMode@4` vs `_SetErrorMode` — while on amd64/arm64 the two conventions coincide and
the same declaration links green. errno.rs declared `SetErrorMode`/`lstrcpyW`/`lstrcatW`
`extern "C"` and got exactly that: three undefined symbols, i386 only. **`cargo check` cannot
see this class at all** — it type-checks, it does not link a DLL — so a lane whose gates are
`check` + `check-tooling` + `difftest` hands a green tree to the merge queue and is bounced
LINK_FAIL.

**`tools/scan_import_cc.py` now closes that hole in `check` itself**, so this is a lint
failure rather than a build you have to remember to run. The oracle is the `.spec` of the DLL
that exports the name — spec2def turns that file into the import library, so it *is* the
link-time truth — scoped per crate to the DLLs its module names in `add_importlibs()`.
Scoping is the design and not a refinement: run unscoped, four of its first 22 findings were
pure name collisions between unrelated modules (user32's stdcall `UserRealizePalette` vs
win32k's FASTCALL one; ntoskrnl's exported `RtlAcquireSRWLockExclusive` vs its private
same-named cdecl inline shim, both real symbols in one binary).

**The reason it was worth building: the tree already held 14 more of these, in two shapes.**

*Papered over at the link layer.* fastfat's `rust_cohab_fat.c` carried three asm-aliased
stdcall trampolines (`_ExReleaseResourceLite@4`, `_ExAcquireFastMutexUnsafe@4`,
`_ExReleaseFastMutexUnsafe@4`) tailing into the real FASTCALL exports, and its comment stated
the diagnosis exactly right — "the Rust translation references them stdcall (`_Name@4`)" —
then fixed everything except the two declarations. That is not a lapse: with no gate able to
see a wrong-ABI import, a shim was the only tool that reached the symptom. The declarations
are fixed and the trampolines are gone. **A wrong-ABI import wants the declaration fixed,
never a trampoline.**

*Latent — declared wrong, never called.* An unused extern emits no relocation, so it links
green and detonates on first use. mscms had `memcpy` inside an `extern "fastcall"` block
(with `RtlUlongByteSwap`, which really is fastcall) and every call site still commented out;
shell32 had kernel32's `lstrlenA` in a block of CRT cdecl imports. wldap32-rust, translated
but not yet flipped, had **nine** — `HeapAlloc`, `GetProcessHeap`, `MultiByteToWideChar` and
friends, the whole Win32 section spelled `extern "C"`. Those nine were the clearest argument
for the lint: they are unreachable by any build gate that exists, because nothing compiles
that crate yet, and they would all have landed at once on flip day.

Two things about the lint that will bite whoever extends it. Its findings are keyed on the
`.spec` **flag order** — spec2def reads the convention token first and options *after* it
(`spec2def.c:1026`), so a pattern written flags-first silently drops every flagged export
(`@ stdcall -norelay GetProcessHeap()`, every `-arch=`, every `-private`) and the oracle
quietly under-reports rather than failing. And a finding in a generated bindgen island is
tagged `IMPORT_CC_ISLAND` because there the island transcribes the C header, so a clash with
the spec is the *header* against the *spec* — an upstream inconsistency to judge, not a decl
to rewrite. Four are baselined: ntdll exports `NtCurrentTeb` `@ stdcall -arch=win32` while
`psfuncs.h` declares it with no `NTAPI`, and `KeInitializeSpinLock` is the same shape.

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
- **A GUID-SPELLED name can be a function-like macro.** shell32's `IID_PPV_ARG(T, &p)` and
  `IID_NULL_PPV_ARG(T, &p)` match `IID_*`, so the cpp-com extern harvest emitted
  `static IID_NULL_PPV_ARG: GUID;` — a symbol in no library. It was excluded by a
  hand-maintained denylist naming `IID_PPV_ARG` alone, so the first new member of the family
  shipped the phantom. Decide it POSITIONALLY instead: a GUID constant is referenced, never
  called. Measured over `dll/ base/ sdk/include/ modules/`, every `(IID|SID|CLSID)_*` name
  followed by `(` outside a literal is a function-like macro; the only real IIDs that look
  called sit inside wine-test format strings, which `mask_code` blanks first.
- **A GUID-spelled name can also be a plain ALIAS `#define` for another IID.**
  `SID_SFolderView` is `#define SID_SFolderView IID_IFolderView` (shobjidl.idl:904) and
  `SID_IShellBrowser` is `#define SID_IShellBrowser IID_IShellBrowser`
  (shlguid_undoc.h:215) — service ids that name an interface id rather than existing.
  CDefaultContextMenu hit both at once: the generator invented an `extern static` for the
  second because it is GUID-shaped, and a HAND-WRITTEN support module had already declared the
  first for the same reason. Neither is exported by any library. Note this is not the
  function-like-macro case above — nothing is "called", so the positional test cannot see it;
  the check is whether a *definition* exists at all.
- **`DSA_GetItemCount` is a macro that dereferences its own handle** —
  `#define DSA_GetItemCount(hdsa) (*(int *) (hdsa))` (commctrl.h:4831), the count being the
  first field of the DSA. comctl32 exports `DSA_GetItemPtr` and `DSA_InsertItem` as real
  functions, so having *some* of a family be symbols is not evidence about the rest.
- **Before writing `extern { fn Foo }`, confirm `Foo` is a real out-of-line symbol** (`nm`, or
  grep for a definition), not a header macro.

### `EXTERN_C` without `WINAPI` is cdecl, and only i386 says so
Giving a C++ function C linkage so Rust can call it is the standard fix when a hybrid TU flips
and the C-linkage forwarders it published go away with it (`precomp.h` documents the
`DataObject_GetHIDACount` group). The trap is the *convention*: `EXTERN_C HRESULT Foo(...)` with
no `WINAPI` is **cdecl** (`_Foo`), not stdcall (`_Foo@N`). Declaring it `extern "system"` in Rust
compiles everywhere and links on amd64/arm64, where the two conventions coincide — it fails only
on i386, as `undefined symbol: _Foo@N`.

Read the C declaration for `WINAPI`/`STDAPI` rather than assuming: within one file the two mix
freely. `filedefext.cpp`'s bridges are `extern "C" ... WINAPI` (stdcall, so the Rust bridges that
replace them stay `extern "system"`), while the helpers they forward to —
`PathIsDotOrDotDotW`, `SHELL_DisplayNameOf`, `SHELL32_GetDllFromRundll32CommandLine` — are
plain `EXTERN_C` and so need `extern "C"`. Same file, both answers.

It cuts **both** ways, so "assume cdecl" is not the fix either. `CRecycleBin.cpp`'s flip
declared `DeleteFileToRecycleBinW` as `extern "C"` on the strength of the header's
`extern "C" {` block — but `recyclebin.h:94` spells it `BOOL WINAPI DeleteFileToRecycleBinW(…)`,
so the symbol is `_DeleteFileToRecycleBinW@4` and i386 reported the mirror-image failure. The
`extern "C" {` block controls MANGLING; `WINAPI` controls the CONVENTION; they are independent
and you have to read both.

### On a 32-bit HOST harness, `extern "system"` and `extern "stdcall"` stop agreeing
`"system"` is stdcall only on 32-bit **Windows**; on `i686-unknown-linux-gnu` it is cdecl.
An explicit `extern "stdcall"` is honoured on x86 whatever the OS. So in a 32-bit host
harness the two spellings, which are the same thing in the product, become different
conventions — and they land on different halves of the same crate: the graded exports are
`extern "system"` (cdecl there) while `msvcrt-sys`'s Win32 imports are `extern "stdcall"`
(still stdcall). **Symptom:** a harness stand-in written cdecl for a stdcall import leaves
four bytes on the stack per call, and the `ret` that follows jumps to a stack-resident value
— SIGSEGV with the PC *inside* the stack, no symbol, nowhere near the cause. **Fix:** give
the reference prelude two spellings, `__stdcall` defined AWAY for the graded exports and an
explicit `__attribute__((stdcall))` for the imports, and take each import's convention from
the `-sys` crate rather than from the name. A stub that never returns (an abort) is exempt,
because nobody pops. Precedent, from the other direction: errno.rs's `_SetErrorMode@4`.

### A `#define` in precomp.h means the C never called the function it names
`precomp.h:63` is `#define ShellMessageBoxW ShellMessageBoxWrapW`, so every `ShellMessageBoxW(…)`
in shell32's C++ is really a call to the Wrap twin — shell32 does not import the exported name at
all. A translation takes the source spelling literally, the island happily declares
`ShellMessageBoxW`, and i386 reports `undefined symbol: _ShellMessageBoxW`. Three TUs have now
hit this one (shlfileop.cpp, drive.cpp, CRecycleBin.cpp). Before believing a "missing export,"
grep `precomp.h` for the name: a `#define` there beats every other explanation, and the fix is
to call the macro's *expansion*.

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

**Grinding a post-assembly cargo-check on ONE target bakes that target's pointer width into
every cast you pick.** Symptom: the lane is clean locally and the merge queue rejects it
`CARGO_CHECK_FAIL` on amd64 or arm64. This is why `crate-repair` takes `--arches i386 amd64
arm64` and applies a width-dependent cast only on cross-target AGREEMENT; a hand grind needs
the same discipline and it is easy to skip, because i386 is the fast build. Four classes, all
invisible on i386, from `dialogs/dialogs.rs`:

- A DLGPROC returns `INT_PTR` (64-bit off i386) while `TRUE`/`FALSE` are `BOOL` and always 32,
  so a bare `return TRUE` only type-checks where the widths coincide. Key the fix on the
  ENCLOSING signature: a blanket widening over-reaches into the neighbouring functions that
  genuinely answer `BOOL`.
- A binding declared `i32` receiving `DialogBoxParamW`, which answers `INT_PTR`. Widen the
  binding and narrow once at the return, as the C++ does implicitly. **Not** rustc's suggested
  `.try_into().unwrap()` — that converts C's silent truncation into a panic in a live dialog,
  which is why `crate-repair` refuses that suggestion by design.
- `SendMessageW`'s `LRESULT` compared against an `i32` constant; `WPARAM`/`LPARAM` arguments
  given `UINT`-typed constants; `HeapAlloc` sized `as u32` instead of `as SIZE_T`.
- A callback transmute that names `extern "stdcall"` outright. **bindgen spells a callback's ABI
  as the RESOLVED convention for the target**, so the same pointer is `stdcall` on i686 and `C`
  on amd64/arm64, while our definitions say `extern "system"` — which IS that convention on each
  target, but Rust keeps the spellings as distinct nominal types on fn pointers. Hardcoding one
  compiles on i386 and fails everywhere else; use a `cfg`'d alias.

The habit that avoids all of it: **cast at the call site using the NAMED type** (`as WPARAM`,
`as SIZE_T`, `as INT_PTR`), never a concrete width. That is arch-neutral by construction —
`folders/CNetFolder.rs` was ground the same night with that habit and needed zero amd64 fixes.

**And cross-target agreement cannot rescue you once the cast is already width-baked** — it is a
filter, not a repair. On syssetup, of the 17 integer sites erroring on all three targets exactly
**ONE** had three-way agreement, because the targets were *correctly* asking for different
widths: the parameters are `SIZE_T`, `WPARAM`, `UINT_PTR`. Measured three ways over a session —
i386-derived casts moved i386 275→144 while pushing amd64 177→193; a blanket `usize` rewrite
moved i386 144→175; amd64-driven same-width moved i386 176→207. Every route regresses another
arch, and that is not a tuning problem, it is the class having no width answer. What is portable
is the NAME, so `crate-repair`'s `width_alias_cast` rule reads the callee's DECLARED parameter
type out of the crate's own island and writes `as SIZE_T` — one edit, right on all three, needing
no vote. Its guards are the interesting part and each is a real mis-fire: the callee must be
declared in **this** crate (a sibling's `SIZE_T` need not be yours — that shape already shipped a
gate-red), and the span must **be** the whole argument, because rustc labels the narrowest
disagreeing operand, so in `SetAutoDaylight(SendDlgItemMessage(..) != BST_UNCHECKED as i64)` it
blames a sub-expression of argument 0 and casting *that* to `BOOL` is well-formed and wrong.
syssetup: 34 sites the width rule could only withhold, plus 55 more it stood down from.

### An `extern` declaration for a symbol YOUR OWN CRATE defines
The worst version of a width mismatch, because both halves are yours and neither compiler nor
gate can see it. `msvcrt-crt-rust/prelude.rs` declared the nine `_isw*_l` predicates as
`fn(c: wint_t, l: _locale_t)`, and `wint_t` is `unsigned short` — but the crate **defines** all
nine itself (the `isw!` macro), taking `u32`, because that is what `_iswctype_l` takes. Every
call routed through the prelude therefore wrote 16 bits into an argument the callee read as 32,
with the upper half undefined per the Windows x64 ABI. It survived only because rustc happens to
zero-extend; nothing guarantees it, and no tool was watching. `scan_extern_arity` is structurally
blind here — **the arity matches**; only the width is wrong, and that bucket was triaged as
"mostly benign return-register width" (below), which is true for *returns* and false for
arguments.

Two tells that this had already been noticed and worked around rather than fixed, both worth
recognising as a signature: `fpnum.rs` carried a private `_iswspace_l_shim(wc: u32, …)` whose
only job was to call `crate::_iswspace_l` directly, and one `mbcs.rs` call site had an explicit
`as u16` cast — a *truncation written to satisfy a declaration that was itself wrong*. When you
find a local shim or a narrowing cast whose only purpose is to make a call typecheck, suspect the
declaration, not the call.

**Rule: never `extern`-declare a symbol your own crate defines — call it through `crate::`.** The
declaration adds nothing (the definition is in scope) and silently opts out of the type checking
that would otherwise be free. Where a prelude must declare it anyway (a shared block others
import), the declaration has to be checked against the definition by hand, because rustc treats
them as unrelated items.

**Sibling case, and it is a SEQUENCING rule rather than a typing one: a global's definition
may only cross the cohab boundary in the same commit as its WRITER.** msvcrt's
`msvcrt_tls_index` is written once by `DllMain` at DLL_PROCESS_ATTACH and read by every
`msvcrt_get_thread_data`. While `DllMain` was C, prelude.rs's `extern` was the only correct
arrangement, and its comment said why: defining it on the Rust side would have given the
reader *a second, always-zero index* — the C writer fills in the C's copy, the Rust reader
sees Rust's, and nothing fails to compile or link. Flipping `DllMain` retires that hazard,
but only if the definition moves with it. Split across two commits, one of them ships a
tree where every TLS lookup reads slot 0. The general form: **when you flip the function that
INITIALISES a global, the global comes with it, and when you flip a reader first, it must keep
importing.** For an entry point specifically, `nm` the decorated name out of a one-file
`rustc --emit=obj` before building anything — `extern "system"` + `#[no_mangle]` on i686 gives
`_DllMain@12`, which must match the C object and the `set_module_type(... ENTRYPOINT DllMain
12)` byte count. A mismatch is a linker error at best and a silently-unused entry point at
worst, and the probe costs one command.

**Same hazard, arriving through the `.spec` instead of through a Rust reader — and this one
SHIPPED. If the DLL exports a global, its Rust definition needs `#[no_mangle]`, or the export
silently keeps the C copy.** `msvcrt.spec:715` says `@ extern _mbctype MSVCRT_mbctype`: the
export name and the backing symbol differ, and only the backing symbol is what a definition
must be spelled. mbcs.rs defined `pub static mut MSVCRT_mbctype` without `#[no_mangle]`, so the
Rust array got a mangled private name, the export resolved to mbcs.c's copy, and the shipping
amd64 DLL carried **two 257-byte arrays at different RVAs** — `_setmbcp` (Rust) writing one
and the exported `_mbctype` naming the other, permanently zero. (`__p__mbctype` is *not* part
of the bug: it returns the per-thread `mbcinfo->mbctype`, a different table.) Nothing failed
to build, link, or boot, because the table IS all-zero in every single-byte locale; it only
bites under a DBCS codepage, where every byte then classifies as a non-lead byte. Find it with
`llvm-nm --defined-only <dll> | grep <symbol>` — **two definitions, one mangled, is the
signature**; `llvm-objdump -p` gives the export's RVA so you can say which one it points at.
Note that a symbol being in the `.spec` is *not* the check: `_mbctype` was there all along.
`tools/verify_cohab_coverage.py` now reads the second field of an `extern` line, which is the
one directive whose export name and backing symbol differ — before that fix, the tool built to
catch wrong-name flips was blind to the only shape where the name is written down twice.

**Sibling failure in the gate itself: a coverage checker that DERIVES the C object from the
module name fails open when the derivation misses.** `verify_cohab_coverage` resolved
`<mod>.c.obj` and treated "not a file" as a bare `continue`, so any module whose Rust name is
not its C basename was compared against nothing and reported clean — indistinguishable from
passing. It surfaced when `reactos/misc.c` had to become `reactos_misc.rs` (`misc.rs` is
msvcrt's *own* Wine-derived misc.c) and `reactos/system.cpp` is not a `.c` at all. **The rule
for any binary-oracle gate: "I could not find the thing to compare against" is a finding, not
a skip.** The distinction it does have to keep is arch-conditional compilation — `except_i386.c`
genuinely has no object on arm64 — so the discriminator is whether the C SOURCE exists in the
tree, with a short declared list for modules that have no C original at all (`cpp_rtti`,
`seh_i386`), each carrying its reason. Note the shape is the same as the `extern` bug directly
above it: both are the gate silently answering a question it never actually asked.

**A C++ TU is not automatically a C++ substrate problem — check what crosses the boundary
before pricing the lane.** `reactos/system.cpp` sat unported for months labelled "the C++
lane", and the label was the only thing blocking it: no RTTI, no vtable, no exception object,
no mangled name on any exported symbol — `system`/`_wsystem` are `extern "C"`, and `throw()`
is an empty exception specification rather than a use of exceptions. All the C++-ness was a
`template <typename Character>` instantiated at exactly two types and resolved at compile
time, which is a Rust generic over a trait. **The cheap check is `llvm-nm` on the object**: if
the only `_Z`/`?`-mangled globals are ones nothing outside the TU references, the C++ never
reaches the linker and the port is a plain-C port. The real hazard turned out to be naming, not
language — process.rs already held Wine's rival `system`/`_wsystem` (correctly `#[cfg(any())]`,
since process.c wraps them in `#ifndef __REACTOS__`), and two live definitions in one archive
are settled by member order.

**Sibling failure: an `extern` naming a symbol THIS TREE does not export.** `cargo check` and
`cargo build` both pass — an extern declaration is a promise to the linker, and the crate link
never tests it — so the first thing that fails is the DLL link, an hour later. scanf's console
source declared `__acrt_iob_func`, correctly citing `corecrt_wstdio.h` where `stdin` is
`(__acrt_iob_func(0))`. But that is the UCRT header; this tree's `sdk/include/crt/stdio.h:92,98`
says `#define _iob __iob_func()` / `#define stdin (&_iob[0])`, and msvcrt.spec exports
`__iob_func`, not the UCRT spelling. **Cite the header this tree compiles, not the one the API is
documented in, and confirm the name is in the `.spec`** — a one-grep check that costs seconds
against a link failure that costs a full build.

**Sibling failure: an `extern "C"` block holding a WINAPI import — i386-only, and it looks
fine on two arches out of three.** `WINAPI` is `__stdcall` on i386 and cdecl everywhere else,
which is *exactly* what `extern "system"` means; `extern "C"` is therefore right on amd64/arm64
and wrong only on i386, where the reference decorates as `_GetLastError` instead of
`_GetLastError@0` and nothing defines it. msvcrt's file.c port hit this on 39 call sites through
one hand-written block in `prelude.rs` — three lines below a comment naming this the project's
#1 historical i386 crash cause, and in a crate whose `msvcrt-sys` allowlist comment already said
these signatures are "DERIVED from the real headers and never hand-written as `extern` blocks in
the consuming crate." **Symptom:** i386 `undefined symbol: _<Name>` with no `@N`, while the other
two arches link clean. **Cause:** a hand-written `extern "C"` for a `WINAPI`/`NTAPI` function.
**Fix:** put it in the bindgen island instead of a hand-written block — bindgen reads the header's
convention and emits `extern "stdcall"` on i386, `extern "C"` elsewhere, so the arch split stops
being a judgement call. If a hand-written block is unavoidable, it must be `extern "system"`,
never `extern "C"`. Audit is one pass: any `extern "C"` block naming a Win32/NT function.
Note this fails LOUDLY at link, unlike the two siblings above — an i386-only *build* break is the
good outcome here; the same mistake on a function pointer or callback would have run.

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

**A cohab flip can CREATE this bug across languages, and neither compiler can see it.** When a
C++ class moves to Rust, the generated `--internal` factory takes the aggregation outer as an
extra leading argument, so Rust publishes `#[no_mangle] extern "C" fn X_CreateInstance(outer,
riid, ppv)` while the C++ header still declares `HRESULT X_CreateInstance(REFIID, LPVOID*)`.
Same name, same `extern "C"`, different arity: it links silently, and the C++ caller's `riid`
lands in `outer` and its `ppv` in `riid`, so the out-pointer is read from a register that holds
a GUID address. cdecl again means no stack damage and no diagnostic. shell32's
CRecyclerDropTarget hit exactly this (8deb9f26010). The fix is a 2-argument `_RustBridge`
adapter that supplies the outer, and the rule is: **when flipping a class, diff the Rust
factory's parameter list against the C header's declaration before deciding the C++ side just
needs `EXTERN_C`.** CDefViewBckgrndMenu in the same lane needed only `EXTERN_C`, because its
`--internal` form takes `psf` where the other took an outer — so the arity happened to match.
Do not generalise from one flip to the next; look.

**Three things that are NOT this bug**, baselined with reasons rather than "fixed": an
**address-only forward decl** (`fn BaseThreadStartup();` exists so kernel32 can write
`BaseThreadStartup as ULONG_PTR` into a new thread's `Rip` — never called through, so its arity
claims nothing; same for freeldr's `PcFindPciBios`); a **uniform syscall-table shape** (the SSDT
spells every entry `(_: usize, ...)` on purpose); and a decl in a file no `mod` chain reaches —
dead code makes no calls, which is why freeldr's parked `directui.rs` drops out on its own.

**The header's prototype is not always the symbol you will bind to — check the OBJECT.** A
sibling of the count bug, and it does not present as a count mismatch at all because the
declaration you write is perfectly consistent with the header you read. msvcrt's `locale.c`
calls `swprintf(wbuf+len, ARRAY_SIZE(wbuf) - len, L"%d", cp)` and `corecrt_wstdio.h` shows a
3-fixed-argument prototype, so an `extern "C" { fn swprintf(buf, count, fmt, ...) }` looks
obviously right. It is wrong twice over: at this build `swprintf` is the `static inline` arm
that forwards to `_vsnwprintf` — `llvm-nm --undefined-only locale.c.obj` imports
`__vsnwprintf` and **no `swprintf` whatsoever** — and the name `swprintf` that the DLL does
export is implemented by `_swprintf`, the NON-conforming **2-argument** form
(`msvcrt.spec:1466`). So the extern would have linked cleanly to a function that reads the
format string out of the count slot, on every call, forever. The rule that catches it costs
one command: **before writing an `extern` for a CRT sibling, run `llvm-nm --undefined-only`
on the C object and confirm the symbol is actually imported under that name.** A header arm
selected by `#if`, a `static inline`, and a `.spec` alias each break the header→symbol
assumption independently, and all three are invisible from the source.

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

### Per-arch `#[cfg]` twins: the OTHER nominal-ABI workaround, and what it hides
Same root cause as the transmute above — `extern "system" fn` unifies with neither
`extern "stdcall"` (i386) nor `extern "C"` (64-bit) in a fn-POINTER type — but the workaround is
to emit the body TWICE under complementary `#[cfg(target_arch)]` arms. It is ABI-correct, and it
is still a bug factory, because **only half of it is ever compiled**:
- A fix applied to one twin is INVISIBLE until someone builds the other arch. `cargo check` on
  any single target is structurally incapable of catching the divergence.
- It defeats `crate-repair`'s cross-target agreement rule outright. A width-sensitive numeric
  cast is only applied when 2+ targets independently demand it, and two targets can never agree
  about a site that exists on exactly one of them. On aclui this withheld **119 of 119** repairs;
  collapsing the twins first let 105 of them through on the next run.
- The 64-bit twin had by then accumulated arch-BAKED casts the i386 twin never got
  (`KillTimer(hwnd, (wParam as UINT) as u64)`), so the two bodies had silently drifted apart.

**Fix: own the callback type locally instead.** `UNBLOCK="DLGPROC WNDPROC ..."` in the module's
`gen_driver_types.sh` profile emits it into THIS crate's island, where `normalize_island_abi.py`
spells it `extern "system"` on all three arches; one definition then serves every target. Watch
for aliases and containers — `LPTHREAD_START_ROUTINE` is only an alias of
`PTHREAD_START_ROUTINE`, and the WNDPROC slot that matters is a FIELD of `_WNDCLASSW`, so
unblocking the pointee alone changes nothing. `tools/rustconv/collapse_abi_twins.py --apply`
folds existing pairs, refusing any whose bodies differ by more than cast decoration.

**And a convention twin is NOT an arch gate — it just looks like one.** msvcrt's except.c came
back with `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not)] extern "C"` pairs on its
callbacks, which is correct and says nothing about the fact that the file's ENTIRE 64-bit frame
handler lives inside `#ifndef __i386__`. Seeing cfg attributes on the page reads as "arch
handled" and the reviewer moves on. Left ungated, the Rust `__CxxFrameHandler` wins cohab
first-match over except_i386.c's completely different 32-bit SEH implementation — and it
**links clean**, because the symbol is the same and only the frame layout it walks is not.
Two rules fall out: the translator gates on CONVENTION, you must gate on the `#if`s yourself;
and the object file is the oracle for which arch defines what (`nm` shows `__CxxFrameHandler`
in except.c.obj on amd64/arm64 and absent on i386). Same reasoning applies to a TU compiled
TWICE under different `-D` — except.c is built into both msvcrt_static and msvcrt_shared, and
its `#if _MSVCR_VER>=140 || defined(__UCRTSUPPORT__)` block is live in exactly one of them. No
source-level analysis can settle a disjunction with an unknown term, and none should try; the
per-target object already knows. The mirror-image case is worth naming too: when the whole TU is
inside `#ifdef __i386__`, the `not(x86)` twin of every emitted pair is unreachable by
construction. Delete it — left in place it reads like an arch gate that is doing work, and it is
the same page-shape as the bug above.

**Transcribing `__ASM_GLOBAL_FUNC` (or any .s) into `global_asm!`: keep AT&T and then PROVE it.**
Symptom: an asm thunk translated to Intel syntax by hand, with no way to tell whether the
conversion was faithful. Cause: converting syntax is a re-derivation, and a re-derivation of
machine code is exactly what you cannot eyeball. Fix: pass `options(att_syntax)` and paste the
C's operand strings unchanged, so "transcribed" means byte-for-byte; then `llvm-objdump -d` the
rlib's asm object against the C `.obj` and diff. msvcrt's six except_i386.c thunks came out
instruction-for-instruction identical at every offset including the relocation record, which is
a total check for one objdump. Note what the macro expands to on PE — `.text`, `.align 4`,
`.globl`, and `.def NAME; .scl 2; .type 32; .endef` — and that `__ASM_CFI(...)` expands to
NOTHING there (no `__GCC_HAVE_DWARF2_CFI_ASM`), so omitting the CFI lines is the same object and
not a dropped directive. These thunks exist because each runs on a frame belonging to someone
else (`call_handler` sets `%ebp` to the throwing function's before calling its `__finally`
block); no Rust `fn` with a normal prologue can replace one.
On the 64-bit arches the SAME thunks carry `.seh_*` unwind directives, and there `-d` agreement
is necessary but NOT sufficient: a dropped `.seh_save_regp` leaves every instruction identical
and the unwind info wrong, which only shows up as a hang while an exception is already in
flight. Diff `.xdata` **and** `.pdata` as well. Two things make that readable: the C `.obj`
holds records for the whole TU, so match your thunk's record by looking its `.text` offset up
in the symbol table rather than diffing the section whole; and an *absent* `.pdata` entry is
itself a signal to compare — arm64 packs simple frames into `.pdata` and gives complex ones an
`.xdata` record, so `_setjmp` and `get_exception_pc` having no entry on **either** side is the
agreement that says that decision came out the same way. `__ASM_SEH(...)` is live on
x86_64/aarch64 mingw (clang defines `__SEH__`) even though `__ASM_CFI(...)` is not; check the
target's `-dM -E` output rather than assuming, since the two macros differ per target.

### A file-local fn-pointer TYPEDEF given a convention the C never had
The transmute entry above is about erasing a convention; this is about *inventing* one, and it
is easier to miss because there is no bindgen slot to disagree with — the typedef and both its
users are all in the file being translated, so the crate is self-consistent and compiles.
`shlexec.cpp` declares

```c
typedef UINT_PTR (*SHELL_ExecuteW32)(const WCHAR *lpCmd, WCHAR *env, BOOL shWait, ...);
static UINT_PTR SHELL_ExecuteW(const WCHAR *lpCmd, ...);   /* no WINAPI on either */
```

and the translation spelled the typedef `unsafe extern "system" fn(..)`, i.e. **stdcall on
i386**, while the function it is assigned from is `extern "C"`. Both are plain `__cdecl` in the
C: no `WINAPI`, no `STDMETHODCALLTYPE`. On i386 that is caller-and-callee disagreeing about who
pops the arguments, once per `ShellExecuteEx`. The rule is mechanical — **a fn-pointer typedef
inherits its convention from the C declaration's macro, and the absence of one means cdecl.**
`extern "system"` is not a safe default; it is a decision. Check every `pub type X = Option<...>`
in translated output against the `typedef` in the C tail.

**And the mirror image: a callback DEFINITION that lost the convention it did have.** Same file
class, opposite direction, and it does not even need a typedef — `dialogs/drive.cpp`'s

```c
static DWORD CALLBACK FormatDriveThread(PVOID pThreadParameter);   /* CALLBACK == __stdcall */
```

came back as a plain `unsafe fn`, i.e. **Rust's own convention**, and is then handed to
`SHCreateThread` to be called from a fresh thread. Nothing in the crate disagrees, because the
value only becomes a `LPTHREAD_START_ROUTINE` at the FFI call. The tell is in the C: `CALLBACK`,
`WINAPI`, `NTAPI`, `APIENTRY` and `STDMETHODCALLTYPE` all decorate a *definition*, and every one
of them must survive as an `extern "..."` on the Rust `fn`. Grep the C tail for those five
before believing a bare `unsafe fn`.

**Where to spell it: `extern "system"`, not the literal the header used.** `system` IS stdcall
on i386-windows and IS the C convention on 64-bit, so one spelling is right everywhere; writing
the literal forces a `#[cfg]`-duplicated copy of the whole function per arch, and two bodies
that must be edited in lockstep drift. That is not hypothetical — shell32 carried four such
twins (`SIC_CompareEntries`, `sic_free`, `dde_cb`, `_DestroyCallback`) purely because the island
spelled the slot `stdcall`. The generator-side fix is `normalize_island_abi.py`, which projects
every ABI-identical fn-pointer slot to `extern "system"`; once an island has been regenerated
through it, the twins collapse to one body. A slot that comes from a `-sys` crate instead of the
island (`DLGPROC`, `LPTHREAD_START_ROUTINE` — both ntdll-sys) has NOT been through it, so
re-declare that one API locally against an `extern "system"` fn-pointer, the way `prelude.rs`
does for `DLGPROC`. The ABI is unchanged; only the spelling is.

**A local re-declaration is not enough once a STRUCT FIELD carries the slot.** Re-declaring
`DialogBoxParamW` against a local `extern "system"` DLGPROC fixes the free function, but
`PROPSHEETPAGEW::pfnDlgProc` is typed `DLGPROC` *inside the island*, and a `pub type DLGPROC`
in the prelude does not retype a field of a struct the island emitted. A dialog-heavy module
hits both — netcfgx has twelve dialog procs plus two property-sheet page structs. The fix is
one profile line, `UNBLOCK="DLGPROC"` in `gen_driver_types.sh`: the island then emits its own
copy, `normalize_island_abi.py` spells it `extern "system"` on all three arches, and field and
free function agree, so ONE definition of each dialog proc serves every target. Without it the
crate needs a `#[cfg(target_arch)]` twin per proc — which is what netid-rust still carries, and
the twins had already drifted apart in incidental casts by the time anyone compared them.
Note the trap that makes this look optional: **Rust compares fn-pointer ABIs NOMINALLY**, so
`extern "system"` unifies with neither `extern "stdcall"` (i386) nor `extern "C"` (amd64/arm64)
even though it *is* each of them on that target. The mismatch is per-arch, so fixing i386 by
hand flips the error to the other two.

### Aggregates, arrays and unions across the FFI boundary
- **`i64` substituted for a by-value `_LARGE_INTEGER`** — i386 cdecl lowers an 8-byte
  aggregate differently from a scalar `i64`; amd64 hides it. Mirror the union type exactly and
  pass `*ptr`, not `(*ptr).QuadPart`.
- **A Rust `[T; N]` passed to a C-variadic is passed BY VALUE** — arrays do not decay. `%s`
  then consumes the first word as the pointer: i386 uefildr wedged in `strnlen(0x6f636553 /*
  "Seco" */, SIZE_MAX)`. Latent under TCG (unmapped reads return 0, scan ends instantly) and a
  multi-hour "hang" under WHPX/KVM, where every probe is a VM exit. amd64 accidentally works
  (Win64 passes >8-byte aggregates by reference). **`.as_ptr()` every array into a variadic.**
  Second instance, and it shows the shape a *macro* wrapper hides: msgina rendered
  `StringCchPrintfW(szBuffer2, _countof(szBuffer2), szBuffer, pgContext->UserName)` as a
  `macro_rules!` that spliced `$arg` straight into `wsprintfW`, so `UserName` — a
  `[WCHAR; 256]` — went by value and `%ws` read two characters of the user's name as an
  address. rustc says nothing: an array IS `Sized`, so it is a legal variadic argument. A macro
  wrapper is the worst place for this class because there is no declared parameter type
  ANYWHERE for a reviewer to compare against.
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

**Not every unexpected `ud2` is this class — `scan_ud2` reports all of them under it. Read the
instruction BEFORE the trap first; it names the cause in one look.**

| Preceding instruction | Cause | Fix |
|---|---|---|
| `call …panic_bounds_check` | array index the optimiser can't prove in range (see the indexing-panic entry) | pointer arithmetic at the unprovable site only |
| `call` to an ordinary function, no panic symbol | LLVM folded a UB path to `unreachable` after inlining | below |
| none of the above | a genuine CC/sig-mismatch fold — the five shapes above | fix the ABI disagreement |

The middle row is the subtle one, and msvcrt's file.c hit **both** it and the bounds-check row in
the same build. Shape: a callee that can return NULL, inlined into a caller that dereferences the
result unchecked. LLVM sees NULL reach a deref, concludes UB, and emits `unreachable` — which
lands as `call <whatever ran last>` then `ud2`, *one per calling function*, so the trap appears
in callers that contain no such code. `msvcrt_get_file` returns NULL on `calloc` failure and
`_fcloseall`/`_rmtmp`/`msvcrt_flush_all_buffers` all do `file->_flag` straight away — **which is
precisely what the C does.** The tell that you are in this row rather than the CC row is that
the ud2 count tracks the number of *callers*, and the C source has the same missing check.

**Fix by restoring the C's codegen shape — `#[inline(never)]` on the callee — not by adding the
null check.** Adding it is a behaviour change on the path the C would fault on, i.e. a
divergence in a differential port; out-of-line, the caller gets an opaque pointer, emits the
ordinary load, and faults exactly where C faults. Confirm the latent path is unreachable anyway
before you accept it (in msvcrt's case every caller walks `3..MSVCRT_stream_idx`, and those slots
are allocated at stream-creation time, so the `calloc` arm is never reached with such an index) —
and if it *is* reachable, you have found a real bug in the C and should say so rather than
silence the trap.

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

**A second route to the same `jmp .`, and this one is invisible in the source: a `#[no_mangle]`
export whose body calls an `extern` declaration carrying `#[link_name = "<that same export>"]`.**
Two different Rust identifiers, no recursion anywhere on the page, and the loop only exists
after the linker resolves both to one symbol. Seen 2026-08-04 in a `drvdefext.cpp` flip: the
model needed `SH_FormatByteSize`, which is C++-linkage in a *different* TU
(`dialogs/filedefext.cpp`) and only reachable through `drvdefext.cpp`'s own
`SH_FormatByteSize_RustBridge` forwarder — so it imported the bridge under that `link_name` and
then also published a `#[no_mangle]` bridge of the same name that called it. `cargo check` is
perfectly happy. `scan_self_recursive` does catch it, but only post-link, and a duplicate-symbol
error would usually fire first. **Rule: when a flip needs a `*_RustBridge` forwarder, the
forwarder belongs in the TU that OWNS the target function, not in the TU being flipped away.**
Grep a new module for any name that appears both as a `link_name` and as a `#[no_mangle]` fn.

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
- **A GENERATED fallback mirror shadows the real port silently.** The cpp-com support block
  emits local `CStringW` / `CComHeapPtr` when it sees those names, and a local `pub struct`
  beats a prelude glob with no warning at either site. In shell32's CDefaultContextMenu that
  hid `crate::atl`'s ports, whose methods the translated bodies call — but the layout is the
  real damage: the fallback `CStringW` is a 3-word `{buffer,len,capacity}` where ATL's is ONE
  pointer, and `StaticShellEntry` is `#[repr(C)]` with a `CStringW Verb` field, so the shadow
  silently re-laid-out a struct that has to match C++. Delete the fallbacks after every
  re-assembly; the missing-method errors are the *symptom*, not the bug.
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

### A UNICODE module inherits the ANSI TCHAR family from the `-sys` glob
`ntdll-sys` is generated WITHOUT `-DUNICODE`, so its bindgen carries `pub type LPTSTR = LPSTR;`.
A module built `set_module_type(<m> win32dll UNICODE)` picks that up through the island's
`pub use ntdll_sys::nt::*`, and every `LPTSTR`/`LPCTSTR`/`TCHAR` in the translation silently
means the NARROW type. aclui surfaced it loudly — `*mut i8` into `SetDlgItemTextW` — but that was
luck: the same alias inside `size_of::<TCHAR>()` compiles clean and sizes every buffer at half.
The A/W *function* aliases are usually derived into the crate prelude and the *type* family
forgotten; spell both, as explicit `pub use` re-exports that shadow the glob:
`LPWSTR as LPTSTR, LPCWSTR as LPCTSTR, PWSTR as PTSTR, PCWSTR as PCTSTR, WCHAR as TCHAR`.

**And the prelude is not enough — the ISLAND has the same problem, one level lower.** The
re-exports above fix the module's own code; they do nothing for a struct the ISLAND declares.
desk.h says `TCHAR szPath[MAX_PATH]`, so with the glob's `TCHAR = c_char` bindgen emitted
`_ICON_PATH { szPath: [TCHAR; 260] }` — 260 bytes where the real struct is 520, on a module
compiled `-DUNICODE`. That compiles perfectly, and the translation had already papered over the
symptom by sprinkling `.cast()` on the call sites to make the narrow pointers fit. **Fix it in
the PROFILE, not the prelude**: `UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR
LPTCH PTCH"` makes the module's own island emit them from its own `-DUNICODE` compile. The
check is one grep: `grep '^pub type TCHAR = ' src/types_i386.rs` must say `WCHAR`.
`scan_type_alias_contradictions` then reports the disagreement with ntdll-sys, which is real and
deliberate — baseline it, exactly as BCHAR already is for aclui/shell32/syssetup/winsrv.

### The same rule for TYPEDEFS: a private `type` alias that disagrees on WIDTH
The entry above is about shadow *structs*; this is the same disease one level down, and it is
cheaper to hit because a one-line `type X = ...;` does not look like a mirror at all.

**Symptom.** A green build on every arch, and a caller's stack local adjacent to an out-param
gets clobbered. No compiler, and no signature verifier, says anything.

**Cause.** Two files spell one C typedef differently and only the *resolved width* differs.
fastfat's `VBO` (`fat.h:38` = `ULONG32`, four bytes): the generated `types_<arch>.rs` islands
had it right, `dirsup.rs` said `type VBO = LARGE_INTEGER` and `dirctrl.rs` said `type VBO =
u64`. So `FatLocateDirent`, whose C prototype is `OUT PVBO ByteOffset`, stored
`(*ByteOffset).QuadPart` — an **eight-byte write through a pointer the interface says points
at four**. `create.rs`, `namesup.rs` and `fileinfo.rs` each backed that out-param with a `u32`
local, so every call lost 4 bytes of the caller's frame; `create.rs` is the file-OPEN path, and
arm64 boots FAT as its live system volume.

**Why it survived.** The two WRONG aliases agreed with each other — both 8 — so the definition
and its in-file callers were mutually consistent and only the honest callers were exposed.
`dirsup.rs` even carried a hand-rolled narrowing shim (`*ByteOffset = ByteOffsetLi.QuadPart as
u32`) at one site: someone met the mismatch, worked around it locally, and left the interface
wrong. **A local workaround for a width mismatch is a bug report against the type, not a fix.**

**Why no existing gate saw it.** Every single-arch build is green because each file is
internally consistent, and rustc has no reason to care what another file calls the same name.
`verify_c_signatures` compares the Rust decl against the C prototype — but both sides said
`PVBO` / `*mut VBO`, so the *names* agreed and only the resolutions differed. Only a whole-tree
read can catch this.

**Fix.** `tools/scan_type_alias_contradictions.py` now does that read, and the same-shaped
`scan_const_contradictions` is its sibling for values. Resolve against the C typedef, then
delete the private copies in favour of the generated island. Beyond VBO its first run found
`BOOLEAN = i32` in two fastfat files (DDK `BOOLEAN` is `unsigned char`) and `BOOL = u8` in
hidparser against `i32` in 22 others, the live one declaring `HidParser_UsesReportId -> BOOL`
for a function defined `-> u32` — benign only because the values are 0/1, so the low byte
happened to be the whole answer.

**The false positives are most of the work, and skipping them is what makes it trustworthy:**
`#[cfg]`-gated pairs (freeldr's `UINTN` is `u64`/`u32` by pointer width, only one compiled),
ASSOCIATED types (`impl Width for Narrow { type Ch = i8 }` vs `... for Wide { type Ch = u16 }`
is msvcrt's narrow/wide scanf — differing per impl is the *point*), and an unresolvable RHS,
which is skipped rather than guessed at. Those three took the tree from 102 findings to 78.

### A header MACRO re-ported per module drifts, and only some drift is visible
bindgen emits types, functions and object-like constants, but **never a function-like macro**,
and it drops constants it cannot fold (a cast, a `sizeof`, a pointer literal). So every
translated module re-ports the same `minwindef.h` / `winuser.h` / `wingdi.h` handful into its own
prelude, and nothing ever compares the copies.

**Symptom.** Nothing. Every copy compiles; the module it lives in is self-consistent. The
divergence is only visible from a whole-tree read, which is the same shape as the typedef entry
above.

**Cause.** The measured state before `win32macros-rust` existed: `LOWORD` defined **32 times
across 12 crates in 8 distinct spellings**, `HIWORD` in 8, `ZeroMemory` in 6, `DWLP_USER` in 9.
Three kinds of divergence hide in that, and they are not equally serious:

- **Type-only.** shell32/menuband spell `LOWORD` `as UINT` where the header says `WORD`. The
  mask makes the *value* identical; only the expression type differs. Harmless, but it is why a
  shared definition cannot simply be dropped in — 21 call sites are typed against `u32`.
- **Re-spelling.** `GetGValue` shifts the full `COLORREF` where `wingdi.h:3382` truncates to
  `WORD` first. Both select bits 8..15, for every input. Proving it *was* a re-spelling is the
  work; record the proof so the next reader does not redo it.
- **Genuinely wrong.** `wingdi.h:3386` masks each `RGB` component to `BYTE` **before** shifting
  it. All four in-tree copies shifted the unmasked value, so a component above 255 bled into the
  next channel — `RGB(r, 0x1FF, b)` gave `0x0001FF00 | b << 16` where C gives `0xFF00`. Latent
  rather than live (the tree's five call sites all pass `GetRValue`-style bytes), but it is the
  same class as msgina's `StringCchPrintfW` dropping its bound.

**Fix.** Port it once into a shared crate, transcribed from the header rather than from the
copies it replaces, with known-answer tests. `strsafe-rust` (inlines) and `win32macros-rust`
(macros + folded constants) are the two precedents.

**The boundary is narrower than "core primitives", and the compiler is what finds it.** A shared
crate cannot depend on any consumer's bindgen island, so it must express everything in primitives
— but two kinds of Win32 type look primitive and are not:

- **`HWND` is `*mut HWND__`**, an *opaque struct* bindgen emits separately into every island. It
  is NOMINAL, so a shared `*mut c_void` will not unify with it. `HANDLE` really is
  `*mut c_void` everywhere, which is why `INVALID_HANDLE_VALUE` shares and `HWND_BROADCAST`
  cannot.
- **`LPARAM`/`WPARAM`/`LRESULT` are not `isize`/`usize`.** They alias `LONG_PTR`/`UINT_PTR`,
  which bindgen emits as `c_long`/`c_uint` on i386 and `c_longlong`/`c_ulonglong` on the 64-bit
  targets — a *fixed-width* type that changes with the arch. `isize` is a distinct primitive from
  `i32` even where both are 32 bits, so a shared macro returning `isize` fails against all three.

So: share it only if its result type is a fixed-width primitive that is the SAME on all three
targets (`u8`, `u16`, `u32`, `i32`, `*mut u16`). Anything pointer-width or nominal stays in the
consumer, where the island's own alias is in scope — `MAKELPARAM` keeps its one-line local
definition and only the `MAKELONG` inside it is shared. Two more that stay local for their own
reasons: the TCHAR-neutral `MAKEINTRESOURCE`, which is `-DUNICODE`-dependent and so is a
per-consumer property, and `ListView_*`/`PropSheet_*`, which expand against island *struct*
types.

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
- **`--allowlist-file` matches where an item is DEFINED, not where it is REACHED.** An umbrella
  header that is a pure `#include` manifold contributes nothing by being allowlisted, and the
  failure is asymmetric in the worst way: bindgen still emits a STRUCT the allowlist reaches as
  a dependency of some allowlisted signature, while every CONSTANT and function that header
  owns stays absent. desk's island carried `IDataObject` (dragged in by an allowlisted
  prototype) and none of the `DVASPECT_*`/`TYMED_*`/`DATADIR_*` its own call sites pass, because
  objbase.h only `#include`s the widl-GENERATED objidl.h/wtypes.h — and gdiplus.h's C branch is
  likewise nothing but `#include`s, so allowlisting it emitted **zero** of the 608 `Gdip*`
  declarations. Allowlist the FAMILY (`gdiplus[a-z]*\.h`) or the generated header by name, and
  verify with a grep for one member rather than assuming the umbrella carried it.
- **`size_of_val` on a `__BindgenUnionField` marker returns 0** — it is a zero-sized accessor.
- **A C function TYPE (`typedef R (NTAPI Name)(args)` — no star) is emitted as a NAMED alias
  on amd64/arm64 but INLINED into the struct field on i386.** The two spellings share no key,
  so anything that projects one island onto another by NAME misses the slot entirely.
  `normalize_island_abi.py` left 40 of msv1_0's `SECPKG_FUNCTION_TABLE` slots spelled
  `extern "C"` on both 64-bit twins for exactly this reason, while the translation spelled
  every one of them `extern "system"` — 31 E0308s that appear on the 64-bit arches ONLY, from
  an island the i386 build had already normalized clean. The fix is to key on the struct FIELD
  and resolve through the alias, which is what the pass now does. ntsecpkg.h writes its whole
  SSP/AP surface this way, so expect it in any module lsasrv dispatches into.

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
- **A CORRECTLY-sized array whose guard is a runtime variable — caught at build time by
  `scan_ud2`, not at runtime.** Same panic, different reason: nothing is over-allocated and no
  index is ever out of range, but the *guard* the C relies on is a `static mut` the optimiser
  cannot use, so the bounds check survives and its panic arm ends in `ud2`. msvcrt's
  `msvcrt_get_file` guards `i >= MSVCRT_max_streams` before `MSVCRT_fstream[i/BLOCK]`;
  `MSVCRT_max_streams` is settable at runtime (`_setmaxstdio`, which does cap it at
  `MSVCRT_MAX_FILES`), so the invariant holds but only globally. Four exports inherited the trap
  by inlining — `_fcloseall`, `_fflush_nolock`, `_rmtmp`, `msvcrt_free_io` — the last of which is
  on the process-exit path of every program linking the DLL. **The tell is the disassembly:
  `call panic_bounds_check` immediately before the `ud2`**, which distinguishes it from the
  §9.99 CC/sig-mismatch fold that `scan_ud2` reports it under. The sibling that does NOT need
  fixing is the diagnostic: `get_ioinfo_nolock` indexes an identically-shaped array and emits no
  check, because it guards against the *constant* `MSVCRT_MAX_FILES`. **Provability, not safety,
  is what differs — so fix only the sites whose guard is a variable**, with the same pointer
  arithmetic (`addr_of_mut!(FOO).cast::<Elem>().add(i)` — the `.cast()` is what stops the stride
  being the whole array).
- **A HAND-WRITTEN `[T; 1]` in a translated module, not a bindgen one — same class, and it
  reached the shipping i386 DLL.** msvcrt's `SCOPETABLE_V4` ends in `SCOPETABLE entries[1]`;
  `except_i386.rs` indexed it `entries[trylevel as usize]` at eight sites across
  `_except_handler4_common` and `msvcrt_local_unwind4`, so every function with more than one
  `__try` level panicked *inside its own SEH handler*. The tell that this is a translation
  slip rather than a design choice is that it is **intra-file**: `msvcrt_local_unwind2`, six
  lines away, reaches the same table through a `*mut SCOPETABLE` and strides it correctly —
  the difference is only that one arrives as a struct MEMBER and the other as a pointer. Grep
  a translated module for `as usize]` before believing it; the count is small and every hit
  wants checking against whether the C declared a flexible array. Found by
  `msvcrt-seh-difftest` on its first run, and it could not have been found any other way:
  the module is `#![cfg(target_arch = "x86")]`, so nothing that runs on the build host ever
  executes it.
- **SILENT thread hang, message on debugcon not serial:** `IoCheckEaBufferValidity` panicked;
  the kernel panic handler writes `PANIC:<file>:<line>` to **debugcon port 0xe9**, then
  `loop {}`. QEMU's timer still preempts the spinning thread, so the box stays alive and it
  reads as one operation wedging forever. **When a single op wedges but the system is live,
  check the debugcon capture for `PANIC:` before suspecting a lock.**

### Faithful C unsigned underflow, panicking under debug overflow checks
C's unsigned wraparound is **defined**, so `size_t len; len -= 2;` at `len == 1` is legal and
the loop that follows walks off the end. Translate it as a plain `-=` and release is byte-
identical (overflow-checks off) while **debug panic-aborts** — an assertion firing on
behaviour the C standard guarantees. The cost is not theoretical: four such sites made the
entire msvcrt difftest suite un-runnable under plain `cargo test`, so every single-row
iteration had to go through `--release`, and `vibeos-cli difftest` builds release, so the
gate never showed it.

**Use `wrapping_sub`/`wrapping_add`, never a clamp.** The clamp is the tempting fix and it is
wrong: it returns a *different answer* than msvcrt for exactly the inputs that reach it.
`_mbsnbcmp_l`/`_mbsnbicmp_l` (mbcs.c:1414, 1527) hit `len -= clen` with `len == 1, clen == 2`
whenever the last byte of the range is a lead byte, and the resulting over-read *is* the
upstream contract; `_mbstowcs_s_l` (mbcs.c:3408-3413) indexes `wcstr[conv-1]` with `conv == 0`
because its guard is `conv == size` and size may be zero. Reproducing an upstream
out-of-bounds read is the job — a translation that "fixes" it silently diverges.

**It is not only unsigned counters, and the wider rule is what to remember: anywhere the C
relies on behaviour that is UNDEFINED in the standard but DEFINED by the hardware, Rust's
checked operator is a live abort.** Four more shapes, all from time.c and math.c and all found
by the first `cargo test` (not `--release`) that reached them:

* **Negation at the type's minimum.** `_abs64` is `n >= 0 ? n : -n`; `-INT64_MIN` has no
  representable answer, every compiler emits `neg`, and the value that comes back is
  `INT64_MIN`. Use `wrapping_neg`.
* **Subtraction that overflows before a promotion.** `_difftime32` is
  `(double)(time1 - time2)` and the subtraction happens in the 32-bit `time_t`, so
  `(INT32_MAX, INT32_MIN)` overflows. `wrapping_sub`, then cast.
* **A shift by the full width.** The rotate family is `shift &= W-1; return (num << shift) |
  (num >> (W-shift));`, so `shift == 0` shifts by `W`. x86's `shr` and arm64's `lsr` both MASK
  the count to `log2(W)` bits, which makes it `>> 0` and the answer `num`. Write the mask out —
  `num >> ((W - shift) & (W-1))` — rather than reaching for `rotate_left`, which is the same
  instruction but hides which fact is being relied on.
* **A signed value converted to unsigned SO THAT the multiply wraps.** `_localtime64_s` does
  `time -= MSVCRT__dstbias * TICKSPERSEC` with `__dstbias` negative; `(ULONGLONG)(-3600)` is
  near 2^64, the multiply wraps mod 2^64, and subtracting the wrapped product is how the C
  ADDS the hour. `wrapping_mul` + `wrapping_sub`, with a comment, because this one reads like
  a bug until you follow it through.

Two tells that this is the class you are in, rather than a real translation bug: the panic is
*only* under debug, and the C's operand types make the operation reachable. Confirm by
reading the C before touching anything — the same panic over a **signed** C counter, or one
the C guards, is a genuine mistranslation and wants the guard restored instead. Check test
helpers too, not just the shipping crate: one of the four was a double→`long double` rebias in
wave16.rs whose neighbouring subnormal branch was already `wrapping_*`.

### Wide/narrow literal sizing
- **The whole class above had ONE cause: the plain-C prompt never mentioned wide strings.**
  Every entry in this section is a symptom of the same missing rule, which is why fixing them
  module by module never made the next module better. `translate` in idiomatic mode loads
  `templates/function_prompt.md` — 29 lines, and its single string rule was the NARROW
  `String literals: b"text\0".as_ptr() as PCHAR`. The detailed (and self-contradictory —
  one passage said "use a static byte array and transmute", another "prefer" explicit arrays
  for a form that has shipped three distinct bugs) wide-string guidance lived only in
  `system_prompt.md`, which that path never loads. So for every tier-3 user-mode module the
  model was told to spell string literals as byte strings, and told nothing about `L"..."`;
  what it did with them — byte string, hand-encoded UTF-16, or raw `L"..."` left verbatim —
  was sampling noise. Note the contrast that proves it: shell32's C++ path has had *zero*
  literals of this class, because `cpp_com` lowers `L"..."` deterministically before the model
  sees the body. Fix: `_format_wide_literal_contract()` in `function_prompt.py` states the
  rule against the substrate the OUTPUT CRATE actually has — it greps the crate for `w!`/
  `wide!` and names them if present, falls back to the explicit-array form if not, and never
  names a macro the crate lacks (an unresolvable macro call is worse than no advice, because
  the repair pass has nothing to resolve it to). It rides in the per-file cached block, so it
  is not re-billed per function, and prompt prose is out of the resume-cache key, so adding it
  re-bills nothing already banked. **The durable version of this fix is a seal, not prose**, and
  it now exists: `--seal-wide` (`rustconv/wide_seal.py`, 2026-08-07) tokenises every wide
  literal out of the body, reveals only its Rust TYPE, and substitutes the code units after the
  response, so a dropped or invented one fails loudly instead of shipping. The prose contract
  stays as the backstop for the 48 occurrences the seal declines. Two things that measuring it
  changed about the story above: the dominant C shape is **not** `L"..."` but Wine's brace list
  `{'C','o','m','b','o','L','B','o','x',0}` (4,272 declarations vs 10,871 quoted tokens — and
  every array in the "18 missing terminators" batch was a brace list), and the restore is a
  generated static rather than `w!`, because `w!`'s result type is not uniform across this tree
  (secur32's yields `&[u16; N]`, aclui's `*const u16`), so naming it would be a type error in
  half the crates. The multiset oracle (`verify_translation_literals.py`) remains the backstop
  and must still be run on fresh output.
- **The `L` prefix is a width MARKER, not a character** — and the translation puts it in the
  string. Three distinct manifestations in one batch (Tier 3), from the same C construct:
  `L"%s.%s"` came back once UTF-16-encoded *including the L* (`b"L\0%\0s\0…"` — every full
  computer name would have carried a stray leading "L"), once as raw ASCII bytes of the whole
  spelling `L"%s.%s"` handed to a wide-string callee, and once as `L'\\'`, a wide CHARACTER
  literal left verbatim (that one at least fails to parse: `prefix \`L\` is unknown`). Only the
  third is caught by the compiler. **`scan_wide_literals.py` is blind to all of this**: it
  diffs a `[u16; N]` array against the `// L"..."` intent comment above it, and assembled
  translation output carries neither the array form nor the comment, so it reports a confident
  `0 findings` over 0 comparisons. Grep `b"L` and `\bL'` in fresh output; the correct fix is
  the crate's `w!()` macro.
- **A `b"..."` behind a wide pointer is TWO different bugs, and only one of them is latent.**
  `b"A\0S\0U\0S\0\0\0" as PCWSTR` is the content-correct one: hand-encoded UTF-16 whose only
  defect is ALIGNMENT 1, so it misbehaves when the linker happens to place it on an odd
  address (that is task #39's mswsock incident). `b"LogEvent\0" as *const u16` is the severe
  one — a NARROW string read as wide, where `"Lo"` is one code unit U+6F4C, so
  `RegQueryValueExW` looks up a value nobody has ever created and the call silently does
  nothing. sysdm shipped ~30 of the second kind. `postprocess.fix_wide_byte_string_literals`
  rewrites both to `w!()` and declines anything that does not decode to clean NUL-terminated
  ASCII, because a wrong rewrite here is silent data corruption rather than a compile error.
  **Do not key a detector on the CAST** — it misses `.as_ptr() as _` (syssetup) and a typed
  local binding `let k: &[u8] = b"K\0e\0y…"` (intl, deepseek). The bytes alone decide it: even
  length, odd bytes NUL, ASCII in the even ones, trailing NUL unit. That test finds 434 sites
  across 79 crates where the cast-keyed one finds 26.
- **A doubled backslash in a path literal survives every gate.** desk-rust wrote the
  appearance scheme to `Control Panel\\Appearance` for four `SHSetValueW` calls; theme.c says
  one backslash. Nothing fails loudly — the value is simply never read back. Legitimate
  doubles exist (UNC and device prefixes `\\.\`, `\\?\`, a bare `\\` computer-name prefix, and
  smbios's `http:\\www.abit.com.tw`, which the C really does spell that way because it matches
  a BIOS vendor's malformed URL), so this needs a diff against the C, not a grep.
- **A `L"..."` inside a MACRO argument loses the width with nothing to catch it.** rasadhlp's
  `RTL_CONSTANT_STRING(L"\\Device\\RasAcd")` came back as
  `RTL_CONSTANT_STRING(b"\\Device\\RasAcd\0")` — a UNICODE_STRING whose Buffer is ANSI bytes.
  It compiles (the macro takes anything), the Length/MaximumLength arithmetic is *right for
  the wrong encoding*, and `NtCreateFile` then just fails to open the device at runtime. This
  is the same L-drop class as above, but a fourth manifestation: the other three are visible
  in the literal's own type, this one hides behind a macro that is happy either way. When a
  crate defines an initializer macro, make it take an ASCII literal and widen it itself
  (`w!()`), so passing bytes is not expressible.
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
- **A `b"..."` cast to a wide pointer is wrong three different ways, and none of them is a
  compile error.** Rust's byte string is 8-bit and is *not* NUL-terminated for you, so the
  same construct produces: **(a)** plain ASCII reinterpreted as UTF-16 — msvcrt `_wsystem`'s
  `L" /c "` written `b" /c \0"`, where `wcscat` reads `' '`+`'/'` as ONE code unit `0x2F20`
  and then walks off the end hunting for a zero `u16`; **(b)** correctly interleaved but ONE
  terminator byte — `b"P\0A\0T\0H\0\0"` in `msvcrt_search_executable`, so the final code
  unit's high half is read from the next object in rodata and PATH search silently matched
  nothing for every `use_path=1` spawn; **(c)** an odd total length, which cannot be whole
  `u16` units at all. `scan_wide_literals.py` now decides all three from the literal's own
  bytes — no intent comment needed, unlike the array rule above, because this is a
  self-contained claim. **Widen the regex before you trust it**: the first version matched
  only a bare `u16`/`WCHAR`/`wchar_t` cast, reported 54 findings, and was blind to `as
  LPCWSTR` and to `transmute(b"…".as_ptr())` — 111 further sites, i.e. two thirds of the
  class, sat behind those two spellings. Also strip `//` comments first: several crates carry
  a comment quoting the bad form verbatim while *fixing* it, and matching inside one turns a
  fix into a finding.
  **(d) an INTERIOR `u16` NUL** — even length, proper terminator, correctly interleaved, and
  still truncated at the first zero code unit. This is what a *repair* produces: re-encoding a
  literal that was already UTF-16 but odd-length as if it were 8-bit interleaves it a second
  time, so `b"I\0n\0f\0\\\0\0"` becomes `b"I\0\0\0n\0\0\0…"` and decodes to `"I"`. It is not
  hypothetical in either direction — a fixup script wrote exactly that (with RAW NUL bytes,
  which made git call the file binary), and three shipped instances were already sitting in
  mswsock's stub `OutputDebugStringW` messages. **Do not exempt the REG_MULTI_SZ shape** even
  though multi-sz genuinely holds interior NULs and ends with two `u16` NULs: the tree has
  zero multi-sz byte-cast literals and three double-interleaved ones, two of which end in four
  zero bytes, so that exemption hides every real instance to protect a case that never occurs.
  Measure the corpus before writing an exemption.
- **A byte literal can carry the C SOURCE SPELLING instead of the string.** netcfgx's
  `swprintf(szBuffer, L"%u", Gw.Metric)` came back as `b"L\"%u\"\0"`, so the format string at
  runtime is the six characters `L"%u"` — the `L` and both quotes are data, and the list column
  renders `L"12"`. Every encoding rule is blind to it (the bytes are consistently 8-bit, just
  8-bit of the wrong text) and so is a reviewer diffing against the C, because `L"%u"` is
  exactly what the C says. `scan_wide_literals` checks it lexically — bytes starting `L"` and
  ending `"` — with no type information, which is what lets it cover the `as *const _` sites
  the cast-based rules cannot match at all.
- **Reading each flagged site against its C is what finds the SECOND bug.** Burning the class
  down by hand across 15 crates turned up three defects that were not encoding at all:
  netcfgx built device names from `L"\\Device"` where the C has `L"\\Device\\"` (dropped
  separator, both use sites); userenv encoded `"L%d"` because the C's `L"%d"` width marker was
  read as content (the same L-drop class as the top of this section, in the other direction);
  and netcfgx passed `LPBYTE as LPBYTE` as RegSetValueExW's data argument — a TYPE ALIAS where
  a value belongs, so not an expression at all. That last one had never failed a build because
  **netcfgx is not a workspace member yet**, which is the standing gap: an unbuilt crate is
  covered by source lints and `verify_c_signatures` and by nothing else, so a lint that reads
  it is worth more there than anywhere. A `sed` over the encoding would have preserved all
  three.
- **A new lint that shadows an existing module global disables the lint it was added to.** The
  byte-cast rule above arrived with its own `_ESCAPES` table, which shadowed the file's
  existing one and silently switched off *both halves of the array rule* — comparisons went
  88 → 51 and the motivating regression test went green by finding nothing at all. Nothing in
  the scanner's own output said so; the coverage ratio moved and the test suite caught it.
  This is the argument for the ratio being printed and for the must-NOT-fire tests existing.
- **The ELEMENTWISE spelling hides in plain sight and no `b"` rule can see it.** syssetup's
  wizard.c came back with 54 wide literals written out as `['S' as u16, 'Y' as u16, …, 0]` —
  the sound-event and theme tables, 270 lines of them. Two defects at once: in a `static`
  initialiser Rust promotes the array so it merely reads as unreviewable noise, but the same
  construct in a FUNCTION BODY is a temporary, and `[…].as_ptr()` there dangles the moment the
  statement ends. Nothing diffs it against the C either — an eyeball comparison of 48 `as u16`
  elements against `L"SYSTEM\\CurrentControlSet\\Control\\ProductOptions"` is not a review.
  `widen_byte_literals.py` decodes this shape and rewrites it to `w!("…")` against the same
  C-source oracle as the byte-string shape; syssetup converted 56 with zero left alone. **Strip
  exactly ONE trailing zero, not all of them**: `L"Terminal Server\0"` is double-NUL-terminated
  and eating its embedded NUL changes what the callee reads. And check the empty case against
  the C too — `[0].as_ptr()` is `L""` in a string table and a one-element numeric array
  everywhere else, and only the C tells them apart.
- **The declared LENGTH of a `static const WCHAR x[]` is a number nobody checked.** Five of
  wizard.c's six registry-path statics arrived with the wrong `[WCHAR; N]` — 59 for a 48-unit
  string, 52 for a 54-unit one. rustc catches the mismatch only because the initialiser is
  spelled out; had the model written `[0; 59]` and filled it at runtime, nothing would have.
  A `wide!("…")` macro that computes `N` from the literal makes the declared length checkable
  instead of asserted, and shortens 48 lines to one.
- **A C table of `const WCHAR *` has no direct Rust spelling**, and the workaround the model
  reaches for is wrong twice. A raw pointer is not `Sync`, so `static T: [[LPCWSTR; 2]; 25]`
  does not compile; what it wrote instead was `[…].as_ptr()` per element, which is both the
  elementwise bug above and (outside a static) a dangling pointer. Wrap the table in a
  `#[repr(transparent)] SyncTable<T>` with `Deref` — the newtype promises exactly what the C
  already promises (read-only, every pointer `'static`), and `Deref` keeps `Themes[n].ThemeFile`
  and `table.len()` spelled the way the C spells them.
- **`b"..."` has alignment 1**, so `b"C\0S\0…".as_ptr() as *mut u16` can land at an odd address
  → amd64-only 0x1E `STATUS_DATATYPE_MISALIGNMENT` when `ProbeForRead` enforces WCHAR
  alignment. Declare a real `static [u16; N]` (alignment 2 ⇒ always even).
- **`PWCHAR HalName = L"..."` is a pointer VARIABLE, not the bytes.** As a `[u16; N]` the
  symbol sits at the buffer start, so consumers read `'A','C','P','I'` as the pointer →
  triple-fault before any trace point. Emit storage + a `#[no_mangle] pub static mut ptr`.
  Fixed-size `T[N]` arrays keep the array shape.
- **An ANSI byte string cast to a WIDE pointer** makes every lookup silently fall back to
  defaults. **Hand-encoded UTF-16 has three distinct mis-encodings** — swept workspace-wide.
- **Two more shapes that `grep 'b"'` does not reach, and the oracle that catches all of them.**
  netcfgx carried 157 of these in one module. Beyond the three above, the literal can be bound
  to a LOCAL first (`let t = b"S\0Y\0S\0…"; copy_nonoverlapping(t.as_ptr() as *const u16, …)`
  — the C was a `WCHAR buf[N] = L"…"` array initializer), and it can arrive with NO `b` prefix
  at all (`"N\0e\0t\0…".as_ptr() as LPCWSTR`, where `.as_ptr()` on a `str` is `*const u8`).
  Grep the ESCAPE, not the prefix: any string literal still containing `\0` is a hand-encoded
  wide literal in a language that has `u16`.
  **The verification that works on assembled output** — where `scan_wide_literals.py` has no
  intent comment to diff against — is the C file itself: decode each literal back to text and
  require it to appear verbatim as `L"<text>"` in the TU it was translated from. On netcfgx
  that confirmed 107 of 108 distinct literals and the single failure was real: `L"\Device\"`
  had come back as `\Device`, losing the trailing separator that `wcscat` appends the adapter
  UUID after. It is ~20 lines, it needs no annotation in the Rust, and unlike a count it cannot
  report a green over zero comparisons.
- **A FOURTH mis-encoding: UTF-16 *bytes* used as u16 ELEMENTS.** `let n: &[u16] =
  &[0x43,0x00,0x68,0x00,0x61,0x00,…]` looks like the three above and is worse: the type is
  `u16`, so element 1 IS a code unit and IS zero. Every such string terminates after its first
  character. wkssvc shipped 39 of them — `SaveWorkstationInfo` opened key `"S"` instead of
  `System\CurrentControlSet\Services\LanmanWorkstation\Parameters` and wrote every value under
  a one-character name. It never looked broken because the bad key made `RegOpenKeyExW` fail
  and the function returned early, so the whole routine was a silent no-op. Decidable with no
  oracle at all: **in a `[u16]`, every second element being 0 is always this bug**, because
  real UTF-16 puts the zero in the high byte of ONE unit, not in a unit of its own.
- **An array sized to the character count drops the NUL, and `scan_wide_literals` cannot see
  it if the element type is spelled `TCHAR`/`WCHAR`.** desk shipped
  `szMonitorSelWndClass: [TCHAR; 18] = ['M',…,'S']` for `TEXT("MONITORSELWNDCLASS")`, which
  the C compiler sizes at 19. RegisterClass/CreateWindow/UnregisterClass then read past the
  array into whatever `.rdata` follows, and it survives only for as long as all three read the
  same garbage. `scan_wide_literals`'s `_DECL` matches a literal `u16` element type only, so
  the alias spellings are invisible to it — that is the gap to mechanise, not the alignment
  question. Note the alignment bullet above is about the KERNEL `ProbeForRead` path, which
  genuinely enforces WCHAR alignment; for a user-mode DLL that just hands the pointer to a
  Win32 callee, alignment-1 is a readability problem, not a fault.
- **The multiset oracle only works if you detect by SHAPE, not by declaration syntax.**
  `verify_translation_literals.py` covers 29 modules now, not just msvcrt. Getting it there
  meant abandoning per-spelling regexes: this tree writes the same wide literal as
  `static N: [u16; K]`, `const N: &[u16]`, `let n: [u16; K]`, an untyped `&[0x6Eu16, …]`, and
  a bare `['S' as u16, …].as_ptr()` with no binding at all. Each spelling the matcher had not
  met produced a WALL of confident MISSING findings against present, correct code — which is
  the failure mode to fear, because the real two are in there and nobody digs to them. Match
  instead on "bracketed list of integer-ish elements, NUL-terminated, otherwise text": one
  rule, all five spellings, and ordinary numeric tables stay out.

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
- **C FLATTENS an anonymous union; bindgen names it.** A tagged struct whose payload is an
  anonymous `union { ... }` reads `Addr.Ip4Address` in the C and must read
  `Addr.__bindgen_anon_1.Ip4Address` in Rust — the translation writes the C spelling every
  time, and it is an E0609, so this one is loud. Two knobs make the result readable rather
  than merely compiling: `gen_driver_types.sh`'s **`REAL_UNIONS=1`** (runs
  `bindgen_unions_to_real.py`) turns bindgen's `__BindgenUnionField<T>` emulation into a real
  Rust `union`, so the member access is a plain field rather than `.as_mut()` noise. It stays
  OPT-IN: ~170 island files across the driver crates carry the emulation and their
  translations call its accessor API, so flipping it on by default would rewrite all of them.
  New usermode crates want it (dnsapi, snmpapi, rasadhlp). **Wire it into the profile, not
  your fingers** — it was a remembered post-step, which is exactly how rasadhlp's islands came
  out unconverted while samlib's (hand-run the day before) were fine.
- **A crate with NO island is invisible to `verify_ffi`.** Its oracle is `ntoskrnl.spec` *and*
  `bindgen_<arch>.rs`; with no island there is nothing to compare against, so every hand-written
  extern block passes unchecked and the gate stays green. shell32-rust carried ~2.7K lines of
  hand-written substrate for this reason and appears in NONE of the 40 crates the
  clashing-extern passes converged — four real ABI disagreements survived in it until a semantic
  declaration comparison went looking. "The lint is green" means nothing if the crate is not in
  the lint's input set; check that before trusting it.

### A struct field nobody READS is a field nobody INITIALISES

**Symptom.** You implement a dormant feature by storing state in a struct field that already
exists and is the semantically right home for it. The field is a plain pointer, the struct is
the real bindgen type, no layout changes — and the kernel takes a `0xc0000005` on the first
dereference, at a wild address, far from your code.

**Cause.** "Nothing uses this field" is the evidence you check, and it is the wrong question.
The right one is "what writes it?" — and the answer is nothing, *because* nothing read it.
`CM_KEY_CONTROL_BLOCK.CachedSecurity` was free real estate for the CM security work, but
`CmpAllocateKeyControlBlock` (`config-rust/src/cmalloc.rs`) hands KCBs back off a recycle
free list, or carves them from a fresh page it never zeroes, and `CmpCreateKeyControlBlock`
initialises only the six fields it actually cares about. So the field held either stale bytes
from a previous key or raw page garbage. The first key whose garbage was non-NULL bugchecked
the boot, during PnP driver matching — nowhere near the security code.

**Fix.** Initialise it explicitly at construction, next to the other initialisers, and say in
the comment *why* (the allocator recycles and does not zero) — not just that it is NULL.
Clearing it on the teardown path is not enough: object-cache allocators have abort paths that
return an object to the free list without running cleanup, and a fresh page was never clean to
begin with.

**Generalises to** any pooled/recycled object: CM KCBs, lookaside-backed structures, and the
same shape recorded for arm64's uninitialised PFN `CacheAttribute` further down. Before
reviving a dormant field, grep for what *writes* it, not what reads it. If the answer is
nothing, you own its initialisation.

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
  Where an interface slipped through and IS island-typed, the failure is not identity but
  **callability**: bindgen wraps every vtable slot in `Option<fn>` where a hand-written vtable
  module uses raw fn pointers, so `((*(*p).lpVtbl).Release)(p)` is `E0618: expected function,
  found Option<...>` — which reads like a missing `.unwrap()` and is really "you took this
  interface from the wrong module". shell32's CDefViewBckgrndMenu hit it on `IDropTarget`; the
  fix is one explicit `use crate::shellcom::{IDropTarget, IDropTargetVtbl};`, not an unwrap.
  And when you import an interface from the vtable module, **the GUID family has to come with
  it, all of it**: the slots are spelled with that module's `GUID`, the `IID_*` statics with its
  `IID`, and the signatures with its `REFIID`/`REFGUID`. Taking some and not the rest does not
  reduce the error count, it MOVES the mismatch from the vtable slots to the statics — measured
  on CFSDropTarget, where importing `GUID`/`IID` without `REFIID` sat at exactly the same 89
  errors as importing none of them.
- **A local DEFINITION shadows a glob import; a second GLOB collides with it.** The distinction
  is worth stating because the assembler's comment got it wrong and three TUs paid for it. A
  crate that has adopted an island opens its modules with `use crate::prelude::*;` (which globs
  the island), while the stitcher adds `use crate::{shellcom,types,win32}::*;` — and every name
  the two share (`REFIID`, `HRESULT`, `ULONG`, ...) is then `E0659: ambiguous`. It presents as a
  catastrophic count that is pure import noise: CRecyclerDropTarget, CDefViewBckgrndMenu and
  CIDLDataObj opened with 339, 339 and 399 errors, of which collapsing to the prelude alone
  removed 339, 339 and 351. **A three-digit error count on a freshly assembled module is an
  import question until proven otherwise** — read the `use` block before reading a single
  finding. Fixed in `__main__.py` now, on the composed text rather than in the stitcher, because
  the prelude line comes from the scaffold, which the stitcher never sees.
  A related asymmetry to know about when a TU holds more than one class: the prelude line is
  added by `crate_integrate`, which only runs on the crate-assembly path, so every slice written
  with `-o` (mandatory for the second and later classes of a TU — the assembler refuses to let
  two classes overwrite one file) used to land on a *different* substrate from its siblings.
- **A duplicate layout in your vtable module is a real cost, but check WHO ELSE COMPILES THAT
  FILE before collapsing it.** shell32's `shellcom.rs` hand-writes `SHITEMID`/`ITEMIDLIST`
  beside the island's `_ITEMIDLIST`; two structurally identical `#[repr(C, packed)]`
  declarations are still two different Rust types, so every PIDL crossing from an island extern
  into a shellcom vtable signature fails `expected *const shellcom::ITEMIDLIST, found *const
  _ITEMIDLIST`. The one-line `pub use crate::island::{ITEMIDLIST, SHITEMID};` looks free — it
  cost zero errors in `shell32-rust` — and it is **wrong**: `shellcom.rs` is `#[path]`-included
  into `menuband-rust` (`lib.rs:38`) *specifically* so that crate gets a layout-identical copy
  without depending on the shell32-rust staticlib, and it has no island. `E0432: could not find
  island in the crate root`, surfaced only by the arm64 leg of a full gate, ~15 minutes after a
  clean three-arch `cargo check -p shell32-rust`.
  **Before deleting or re-pointing anything in a shared substrate file, `grep -rn "<file>.rs"
  --include=*.rs` for `#[path]` includes.** A path-included module has a second dependency
  surface that its own crate's `cargo check` cannot see, and `-p <crate>` is not the whole story
  — the gate builds every crate. Keep the duplicate, note *why* it is duplicated at the
  declaration, and `.cast()` at the crossings (5 sites in `brfolder`).
  The same measure-first rule applies for its own sake: shellcom's `STRRET` stays hand-written
  because the island's is a bindgen anon-union (`__bindgen_anon_1` + `__BindgenUnionField`) and
  collapsing to it would rewrite `.u.pOleStr` across four landed files.
- **An island's allowlist is grown by the TU you are translating NEXT, and regenerating it is
  cheap and deterministic — do that before hand-declaring a missing surface.** shell32's
  allowlist had no `ioapiset.h` and no `winioctl.h`, so `dialogs/drive.cpp` — which is nothing
  but `DeviceIoControl` against `GET_LENGTH_INFORMATION` / `DISK_GEOMETRY` /
  `PARTITION_INFORMATION{,_EX}` — had *no* layout for the buffers it fills and *no* value for the
  codes it issues. That is the §9.112 shape one layer upstream, and the instinct to write the
  six structs into the module by hand is exactly wrong: a hand-written layout is unverifiable,
  and the next disk TU writes its own second copy.
  Two things make the regeneration safe to reach for. It is **byte-reproducible** — re-running
  `gen_driver_types.sh <module>` with no change to the script produced a diff of exactly the
  seven declarations that `EXTERN_C` edits to `precomp.h` had made visible since the last run,
  i.e. the island was simply stale, and a stale island is invisible until something needs the
  missing name. And most of what an allowlisted header *would* add is already blocklisted to the
  sibling `-sys` crate, so the diff is far smaller than the header: winioctl.h contributed only
  the `FILE_DEVICE_*` / `METHOD_*` / `FILE_*_ACCESS` constants, because ntdll-sys already owns
  every disk STRUCT and keeps their single identity.
  What is still left over afterwards is the function-like macros — `CTL_CODE(t,f,m,a)` and
  therefore all ~200 `IOCTL_*` codes assembled from it. Compute those in the prelude from the
  island's own `FILE_DEVICE_*`/`METHOD_*` parts rather than transcribing the hex; a wrong IOCTL
  is a runtime `ERROR_INVALID_FUNCTION`, not a compile error.
- **`<stem>_extra.rs` is residue, not a spare copy — delete an entry the day bindgen learns to
  emit it.** `island_extra.rs` kept a hand-written `lstrcpynW` after all three arch islands
  began binding it (identically, `stdcall` vs the residue's `system`). A duplicate in the
  extra file does NOT shadow the generated one — `island.rs` globs both, so *every* use goes
  `E0659: ambiguous`, including a fully-qualified `crate::island::lstrcpynW`. The tell is an
  ambiguity error naming `types_arch` and `types_extra` as the two candidates.
- **A `-x c++` cell emits broken aliases for class-template member typedefs** — bindgen
  flattens them to non-generic aliases over unbound parameters (`pub type
  ATL_CComEnum__CComEnumBase = u8<Base, T, Copy>;`). Blocklist by the **C++ qualified name**,
  not bindgen's flattened Rust spelling. Do not "fix" this by dropping every alias carrying a
  `<`: on shell32 that rule would have deleted 82 legitimate `Option<fn-ptr>` callbacks.

Also: `profile_derive` mirrors each TU's include prologue verbatim, so a **quoted** include
copied from a TU in a subdirectory (`#include "CFSFolder.h"` in `folders/CDesktopFolder.cpp`)
breaks — a quoted include resolves against the *including file's* directory and the generated
wrapper lives in `/tmp`. Use the path-qualified spelling the module's own `precomp.h` uses.

**Glob the island INSTEAD of the hand-written substrate, never alongside it.** Symptom: adding
`use crate::island::*;` to a TU that already globs `crate::types::*` / `crate::win32::*` turns
every name both of them define into `E0659: ... is ambiguous` — on shell32's first two Wave-5
leaves, 296 occurrences across 54 names, where either glob *alone* produced none. Cause: two
glob imports of one name are ambiguous at the use site, and the two substrates overlap heavily
by construction. Fix: a `prelude.rs` that globs the island and re-exports the residue one name
at a time (an explicit import beats a glob, so it cannot go ambiguous). Measured on those
leaves, island-first took unresolved names from **228 to 77**, and the survivors were a clean,
nameable set: A/W-mapping and accessor macros, and the module's own internal helpers.

**Same width is not the same type: the nominal-alias split.** Symptom: after adopting an island
the call sites fail `expected i32, found isize` (or `expected shellord::SHELLSTATE, found
types_arch::SHELLSTATE`), and `crate-repair` withholds the repair with `target_divergent`,
correctly refusing to bake a per-arch width. Cause: two aliases for the same C type reached the
TU from different authorities — `shellcom::LPARAM` is `isize` via a hand-written
`LONG_PTR = isize`, the island's is `i32` on i386 via `LONG_PTR = c_long`. Identical bytes,
identical meaning, different Rust types, so all 3,406 island declarations reject the value. Fix:
scalars come from ONE authority, the island. Corollary for anything imported from a hand-written
sibling module: **bring the types in its signature from the same place, or bring neither** — and
when the two conflict (you want the island's struct for its bitfield accessors, but a sibling's
`fn` is typed against its own copy), declare the `#[no_mangle]` export in the prelude against
island types instead of importing the Rust `fn`. That is what the C does: include the header,
call the symbol. Verify layout equivalence field-by-field first; don't assume it.

**A UNICODE module inherits ANSI `TCHAR` from `ntdll-sys`.** Symptom: `LPTSTR` resolves to
`*mut i8` and every wide-string assignment fails on mutability or pointee. Cause: `ntdll-sys` is
generated *without* `UNICODE`, so it defines `TCHAR = c_char` / `LPTSTR = LPSTR`, and an island
whose preamble globs `ntdll_sys::nt::*` carries those into every consumer — even a module built
`set_module_type(... UNICODE)`. This is latent, not loud: a `TCHAR` buffer sized from the leaked
alias is **half** the bytes it should be. Fix: define the whole family (`TCHAR`, `LPTSTR`,
`PTSTR`, `LPCTSTR`, `PCTSTR`) explicitly in the module's prelude, where it beats the glob. Same
class: bindgen types `TRUE`/`FALSE` as `u32` because the `#define`s are bare `1`/`0`; pin them
to `BOOL` the same way.

**One dialog proc, not one per arch.** Symptom: a translated TU carries `#[cfg]`-duplicated
copies of a whole window/dialog procedure — `extern "stdcall"` for i386, `extern "system"`
elsewhere — and the copies drift apart under separate repair passes (recycler_prop.cpp's two had
already diverged into `as WPARAM` versus a baked `as u64`). Cause: bindgen writes the literal
convention it saw, so a callback typedef is `stdcall` on i386 and `C` on 64-bit, and a procedure
can only be written to match one spelling. Fix: alias the callback type yourself as
`extern "system"` on every arch — it *is* stdcall on i386-windows and the C convention on
64-bit — then one copy of the procedure is correct everywhere. Same reasoning as
`normalize_island_abi.py`, which does this for a generated island's own callback slots; the
hand-written alias is for types that arrive from a sibling `-sys` crate instead.

**Flipping a C++ TU: mark its exports `EXTERN_C` in the header.** Symptom: the cohab flip links
with `undefined: SomeFunction` even though the Rust `#[no_mangle]` symbol exists. Cause: the
function was declared in a `.h`/`.cpp` without `extern "C"`, so every C++ caller references a
*mangled* name that nothing now defines. Fix: put `EXTERN_C` on the declaration (this is what
`dialogs/view.cpp`'s `PostCabinetMessage` got in `precomp.h` when it was flipped) — and at
namespace scope, since a linkage specification is illegal inside a function body.
**`llvm-nm --defined-only` on the TU's `.obj` tells you which exports need it BEFORE you flip**:
an unmangled `_Name@N` is already `extern "C"`, a `__Z23Name...` is not. shldataobject.cpp had
five of each, and the mangled four had live callers in three other TUs. Intersect that set
against every *other* object's UNDEFINED symbols before counting the work — shlfolder.cpp
defines 83 mangled symbols but only 19 have a live caller; the rest are COMDAT template and
inline emissions each TU already makes for itself.
**A declaration can live in a `.cpp`, not only a header** — so grepping headers under-counts the
work. `brfolder.cpp` declares `INT WINAPI Shell_GetCachedImageIndexW(...)` at its own file
scope rather than including one, and that lone bare declaration was the only thing left holding
a mangled reference after iconcache.cpp flipped. `folder_options.cpp` repeated it exactly:
`CDesktopBrowser.cpp` declares `extern VOID WINAPI ShowFolderOptionsDialog(UINT, BOOL);` at its
own file scope, so the flip looked caller-free and failed at link on
`__Z23ShowFolderOptionsDialogji@8`.
**A symbol reached through a SHARED static library has as many providers as it has consumers,
and changing its linkage changes all of them at once.** `ShowFolderOptionsDialog` again, one
build later: `shelldesktop` is an OBJECT library linked into BOTH `shell32.dll` and
`rshell.dll`, and each supplies the symbol from somewhere different — shell32 from the flipped
Rust, rshell from a `misc.cpp` stub that shells out to `rundll32`. The CALLER's declaration
decides the mangling, so `EXTERN_C` on the shared `CDesktopBrowser.cpp` declaration silently
retargets rshell's link too: shell32.dll then builds and **rshell.dll** fails on
`_ShowFolderOptionsDialog@8`, which looks like a shell32 problem and is not. Before flipping a
symbol that a shared static/object library references, list that library's consumers
(`grep -rn <lib> --include=CMakeLists.txt`) and check each for its own definition — and put the
`EXTERN_C` on every provider in the same commit.

**And beware how the search itself lies: `grep -rn <sym> | head` is not a completeness check.**
That second case WAS searched across `.cpp`, and the one hit that mattered sorted past the
tenth line. Truncating a search whose whole purpose is "prove there are no other references"
turns a negative result into no result — count (`grep -c`), or read all of it. Conversely,
check whether the header you *do* find already wraps itself: `wine/shell32_main.h` opens with
`#ifdef __cplusplus extern "C" {`, so all four of iconcache's shell32-internal exports needed
no edit at all. And when the header is reached through the PCH (`precomp.h` includes
`dialogs/folder_options.h`), one `EXTERN_C` there covers every TU at once — check that before
editing call sites one by one.
**Exported DATA needs `#[no_mangle]` as much as exported functions do.** A flipped TU's
`pub static mut` is Rust-private without it, and the link fails with
`_ShellLargeIconSize: i386 cdecl — missing symbol`, which reads like an ABI problem and is not
one. `extern INT ShellLargeIconSize;` in a header is the tell: anything the header publishes is
part of the flip surface.

**`EXTERN_C` cannot carry an OVERLOAD SET — one name, one symbol.** Symptom: a TU is otherwise
ready to flip but exports two functions of the same name. Fix: rename one on both sides. Make
the choice by call-site count, not by which "feels primary": shlfolder's
`SHELL32_CoCreateInitSF(pidlRoot, clsid, csidl, riid, ppvOut)` became
`SHELL32_CoCreateInitSFWithCsidl`, and being module-private (declared only in `shfldr.h`) that
cost six call sites and nothing outside the DLL. **C++ reference parameters are NOT part of this
problem** — `extern "C"` changes linkage and mangling, not the type system, so
`EXTERN_C HRESULT F(REFIID riid)` is legal and yields an unmangled symbol. Believing otherwise
deferred this flip once; a 5-line compile test settles it in a minute.

**A C++ static MEMBER has no `EXTERN_C` spelling either — forward it in the header instead.**
Symptom: the same flip-links-but-mangled failure as above, except the undefined names are
`__ZN9CFSFolder10FormatSizeEyPwj` and friends — class-qualified. Cause: `extern "C"` is not
applicable to a member function; the name lives in class scope, so there is no unmangled
spelling for a Rust `#[no_mangle]` to satisfy. Fix: turn the declaration into an **inline
forwarder to the C entry point the TU already published**, right there in the class body, and
change nothing at the call sites — `static HRESULT FormatSize(UINT64 s, LPWSTR b, UINT c)
{ return CFSFolder_FormatSize_RustBridge(s, b, c); }`. Five of `CFSFolder.cpp`'s seven mangled
symbols went that way; `CFSFolder.h` already had one static of exactly this shape
(`CompareUiStrings`), which is the hint that the pattern is idiomatic here rather than a hack.
It also adds no bridge — the forwarders reuse the ones the TU was already exporting, so the
ledger count still only goes down. The other two of the seven were free functions and took the
ordinary `EXTERN_C`-in-the-header route.

**Count the RUST callers of a C++ overload too, not just the C++ ones.** Symptom: an overload
rename passes `grep` over `*.cpp` and still breaks. Cause: already-translated siblings call the
overload through a generated `Shell32Rust*` trampoline (rustconv emits one precisely because
Rust cannot name a C++ overload), and they declare it `extern "system"` because the trampoline
is `WINAPI` while the renamed function is cdecl. Fix: leave the trampoline — repointing five
declarations at the now-nameable Rust symbol is an ABI change for no functional gain — but fix
the emitter, which otherwise generates C++ calling a name that no longer exists.

**`scan_extern_arity` resolves `#[link_name]`/`#[export_name]`, not the Rust identifier.** A
declaration that redirects is a claim about a *different* symbol, so comparing it against a
definition that merely shares its Rust name is wrong: the five folder classes above declare a
5-param `SHELL32_CoCreateInitSF` linked to `Shell32RustCoCreateInitSF`, and keying on the
identifier reported all five as arity bugs against the real 6-param definition the moment
shlfolder.rs went live. The scanner had been quiet only because the parked file carried an
`ORPHAN-BY-DESIGN:` banner — which is the general shape of the trap: **an exemption that hides
a class of finding stops hiding it exactly when you flip, so read what a banner was suppressing
before you delete it.**

**A C++ header inline is in NO translation unit, so no translated TU carries it.** Symptom: a
freshly translated file has a cluster of unresolved names that are demonstrably not in its
`.cpp` — `HIDA_GetPIDLFolder`, `DataObject_GetData`, `DestroyCIDA`. Cause: they are
`static inline`/`inline` in a *header* (`sdk/include/reactos/shellutils.h`,
`dll/win32/shell32/utils.h`); the C++ compiler inlines them into every TU, and rustconv
translates only the `.cpp`. Fix: port the header once into its own module
(`shell32-rust/src/shellutils.rs`) rather than per-TU — these are shared by at least four TUs.
Check for this before assuming a name is a translation defect: `grep -l` the name across
`sdk/include` and the module's own headers first.
**A header inline can be hiding behind a `.cpp` that INLINES IT AND EXPORTS A WRAPPER**, which
looks like an ordinary definition right up until you flip that `.cpp`. shlfolder.cpp did this
for `wine/shell32_main.h`'s `SHELL_CreateVariantBuffer` / `SHELL_CreateSHDESCRIPTIONID`: it
carried `SHELL_Create*_RustBridge` wrappers, so translated Rust reached the inlines through a
C++ hop and nothing looked unresolved — then the flip took the only definition with it and i386
failed on `_SHELL_CreateVariantBuffer@8`. Port the inline rather than re-homing the wrapper in
another `.cpp`, or the next flip repeats it. `static inline` at namespace scope has INTERNAL
linkage, so a Rust `#[no_mangle]` of the same name cannot collide with the C++ TUs' own copies.
And when you port one, do **not** `#[no_mangle]` it unless a caller genuinely needs the symbol:
`SHELL_OsIsUnicode` is internal-linkage in every C TU that uses it, so exporting the Rust copy
would invent a symbol the C side never had.

**A function-local `static` with a constant initialiser is a hidden GLOBAL, and the translation
drops it.** Symptom: a freshly translated TU references a name that exists nowhere in its
output — iconcache.rs used `s_imgListIdx` in three places and defined it in none. Cause: C++
lowers `static int s_imgListIdx = -1;` inside a function to a global with no guard variable
(the guard appears only for a *dynamic* initialiser), and the translator kept the uses but not
the storage. Fix: a module-level `static mut`, which is exactly equivalent — module-private, so
widening the NAME's scope cannot collide with anything.

**A bindgen anonymous UNION cannot be initialised through the island when C picks a POINTER
arm.** Symptom: the translated static names fields (`Flags`, `CreatorBackTraceIndexHigh`,
`SpareWORD`) that the island's struct does not have, because bindgen collapses the union into
one opaque `__bindgen_anon_1`. Reaching for the right arm does not rescue it either: C's
initialiser here is `{ (DWORD_PTR)(__FILE__ ": SHELL32_SicCS") }`, i.e. the `ULONG_PTR
WineDebugString` member holding a *pointer*, and const-eval refuses to cast a pointer to an
integer. Fix: mirror the struct locally with that one arm typed as the pointer it actually
holds — same size and alignment, so the layout ntdll walks is unchanged. `ole32-rust`'s
`ring::ifs` established this for `RTL_CRITICAL_SECTION_DEBUG`; iconcache.rs is the second user.
Zeroing the union instead would compile and silently drop the debug string.

**`extern "C"` is LINKAGE; `WINAPI` is the CALLING CONVENTION — a `_RustBridge` needs both read
separately.** Symptom: i386 link fails on `_Name` while `llvm-nm` shows `_Name@12` defined.
Cause: the C++ is `extern "C" INT WINAPI Name(...)`, and the Rust declaration went into an
`extern "C"` block because the C++ said `extern "C"`. It is stdcall, so it belongs in
`extern "system"`. amd64 and arm64 link fine either way, which is why this reaches i386 alone.

**Porting ATL's smart pointers: `Drop` IS the translation's missing destructor.** Symptom: a
translated C++ TU declares `CComPtr<IShellFolder> p;` and never releases it — the translation
left at most a `// Destructor for CComPtr<...> – call Release` comment and no code. Cause:
rustconv does not lower C++ RAII. Fix: implement `Drop`, and the release-at-scope-exit is
restored for every call site at once. Three constraints, all load-bearing, all measured from
the emitted code (`shell32-rust/src/atl.rs`): the types must be **zeroable** (the translation
emits `core::mem::zeroed()` as a default-constructor stand-in, so no `Vec`/`NonNull` field —
a zeroed `Vec` is instant UB); the pointer types must be **`#[repr(transparent)]`** (the
translation reproduces ATL's `operator&` as `&mut p as *mut _ as *mut *mut T`); and if a
translation ever ALSO emits an explicit `.Release()`, delete the explicit call, never the
`Drop`. `CComPtr::drop` may reach `Release` by casting `*mut T` to `*mut IUnknown` because
every COM interface starts with a vtable whose slot 2 is `Release` — that is what ATL itself
does, and it is only valid for interface `T`.

**Rewriting a vtable dispatch: check whether `this` is already there.** Symptom: after a
mechanical `(*(*p).lpVtbl).M(args)` → `((*(*p).lpVtbl).M)(p, args)` rewrite, rustc reports
"takes 1 argument but 2 were supplied". Cause: the translation emits BOTH shapes — sometimes
`(*(*p).lpVtbl).Release()` and sometimes `(*(*p).lpVtbl).Release(p)`, the second already
carrying the interface pointer, so splicing one in yields `(...Release)(p, p)`. That compiles
as far as arity checking and is caught only there; on a slot whose first two parameters happen
to share a type it would not be caught at all. Fix: compare the first top-level argument
against the receiver text and add nothing when they match (`island_adopt.py`'s `already_this`).

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
- **A STRING LITERAL is a fact class the seal does not cover, and the failure is total.**
  smbios.c's `IsGenericSystemName` is a 71-row table of `{ L"vendor name", TRUE, TRUE }`.
  Every single `L"…"` came back as `TRUE` or `FALSE`, two rows were dropped, and the control
  flow was inverted on top — `i` started at `TRUE` instead of 0, a match returned `FALSE`, the
  fall-through returned `TRUE`. The predicate answered the exact opposite of its question and
  compiled clean. **`--seal-facts` reported success**: its invariant is over the CONSTANT set,
  a string literal is not in that set, so the model had the sealed `TRUE`/`FALSE` and nothing
  to anchor 71 pointer slots to, and filled them from what it did have. Same recall-permutation
  shape as msvcrt's `_tzset`, on a class the envelope cannot see at all.
- **Two audits catch it, both cheap, both worth running on every module** (they found six
  distinct defects in sysdm that three green gates did not): diff the MULTISET of string
  literals in each `.c` against its `.rs`, and diff each function's `return TRUE` / `return
  FALSE` COUNTS between the two. The second is what names an inverted predicate. Expect
  benign deltas — a dropped `DPRINT` format, an `#ifdef` arm the tree does not carry, a
  literal spelled as a `['P' as u16, …]` array — so read the output rather than asserting on
  it. Regenerate a wrong table by PARSING the C; transcribing 71 rows by hand is the same
  failure mode with a slower clock.
- **Model choice does not move this.** deepseek-v4-pro and claude-sonnet-4-5 produce the
  byte-string wide-literal form identically on the same file. The audits are the gate, not the
  model.
- **After fixing a class of bad data, re-audit everything derived FROM it.**
- **The invented value is usually a NEIGHBOUR from the same family, which is why review misses
  it.** `CmpHiveRootSecurityDescriptor` lowered `SECURITY_RESTRICTED_CODE_RID` (12) to a literal
  `33` — that is `SECURITY_WRITE_RESTRICTED_CODE_RID`, so the registry hive-root DACL granted
  `KEY_READ` to `S-1-5-33` instead of `S-1-5-12` for years. Every *other* literal in the same
  function (`ACL_REVISION` 2, `KEY_ALL_ACCESS` 0xF003F, `KEY_READ` 0x20019,
  `CONTAINER_INHERIT_ACE` 2, `FIELD_OFFSET(ACCESS_ALLOWED_ACE, SidStart)` 8) audited **correct**
  — one wrong value in six, and no way to tell which by reading. Only a conformance test that
  knew the right answer caught it. Re-name the whole family when you fix one, so the compiler
  and `verify_constants` can see them: a bare literal is checkable only by someone who already
  suspects it.
- **A constant that lives in only SOME arches' islands cannot simply be imported.**
  `KEY_ALL_ACCESS` is in `hal_bindgen_i386/amd64` and **not** `hal_bindgen_arm64`, so `use`-ing
  it breaks the arm64 build. Declare it locally *and pin it* —
  `#[cfg(any(target_arch = "x86", target_arch = "x86_64"))] const _: () = assert!(LOCAL == island);`
  — which keeps the hand-typed value honest wherever an oracle exists instead of trading one
  unchecked literal for another.

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
- **A guess can also come from the WRONG ARCH's `#ifdef`.** ex-rust re-declared
  `INTERLOCKED_RESULT` as a sign in `{-1, 0, 1}`. `exfuncs.h` does define a tidy `0/1/2` — but
  under `#ifndef _M_IX86`; on x86, `xdk/x86/ke.h` overrides it with the encoding the assembly
  original returned verbatim, the EFLAGS SIGN/ZERO pair (`RESULT_NEGATIVE` 0x8000,
  `RESULT_ZERO` 0x4000, `RESULT_POSITIVE` 0). Reading the generic arm of a header you are
  compiling out of looks exactly like reading the header. Note the failure mode is worse than
  "wrong number": `ResultPositive`(1) and `ResultZero`(0) had swapped roles, so a caller
  testing `!= ResultZero` — which is every in-tree caller — read *positive* as *zero*. bindgen
  had the right values in `bindgen_i386.rs` the whole time; the fix pins the local constants to
  them with a `const _: () = { assert!(...) }`.

**Anything `verify_constants --enums` resolves to exactly ONE header value is a strong real-bug
signal** (the ambiguity rule already drops multi-enum names). Two deliberate blind spots to
review by hand: `TAG_*` pool tags (byte order, above) and untagged inline literals at call
sites (`KeInitializeEvent(e, 1, 0)` with no comment) — `scan_inline_flag_literals` only catches
the commented ones. **Prefer bindgen enums over local consts in new translations.**

### The same-named constant with a DIFFERENT value per driver
Cross-driver value bleed: cdfs and fastfat both define a WAIT flag, with different values. A
"known" constant is only known *within its module*.

### A local `const` silently SHADOWS a glob import — no warning at either site
`ARM3/special.rs` declared `const POOL_FLAG_SPECIAL_POOL: ULONG = 0x10` while
`use ntkrnl_sys::mm::*` was already supplying the real `0x20` (`miarm.h:328`). `0x10` is
`POOL_FLAG_CHECK_DEADLOCK`, so `MiInitializeSpecialPool` set the wrong bit forever and
`ExAllocatePoolWithTag`'s `ExpPoolFlags & 0x20` test never fired: **special pool armed,
initialised, printed `ACTIVE`, and routed not one allocation.** rustc warns about neither the
shadowing nor the unused import. The same class by *type*: `fastfat-rust/src/dirctrl.rs` picks
up a 64-bit `VBO` from an `ntkrnl_sys` glob where fastfat's own `VBO` is `ULONG32`
(`fat.h:38`), so the C `FatLocateDirent` writes 4 bytes through a pointer Rust reads as 8 —
surviving only because the `= 0` init leaves the high half clear on little-endian.

**Never redeclare a name the crate already imports.** And the process half of the lesson:
`scan_const_contradictions` had *already found* this one — it sat in the lint baseline, and
nobody traced what the contradiction meant. **Baselining a constant contradiction defers a real
bug; when the constant gates a diagnostic, baselining it disables the diagnostic.** Two
unresolved neighbours still sit three lines away (`MI_HYPERSPACE_PTES` 0xFF vs 0x100).

It also runs the other way: **a glob-imported unit const captures a LOCAL VARIABLE of the same
name, because `let Name: T;` is then a constant PATTERN, not a binding.** msv1_0's
`BuildTokenGroups` has the C's `PTOKEN_GROUPS TokenGroups;`, and ntdll-sys exports
`pub const TokenGroups: _TOKEN_INFORMATION_CLASS` (the enum member). The `let` bound nothing,
and the next line failed with `E0070: cannot assign to this expression` pointing at a name that
looks like an ordinary local — the error never mentions the const it resolved to. C's separate
tag/ordinary namespaces make a local named after an enum member completely normal, so expect
this in any module that touches a `*_INFORMATION_CLASS`. Rename the LOCAL (no ABI consequence)
rather than the import. Loud here; the silent cousin is the same name in a `match` arm — see
"An undefined constant in a `match` arm becomes an irrefutable CATCH-ALL".

**It captures FUNCTION PARAMETERS too, and there the diagnostic names nothing useful.**
shell32's `InsertMenuItemAt(HMENU hMenu, UINT Pos, UINT Flags)` translated faithfully to
`fn InsertMenuItemAt(hMenu: HMENU, Pos: UINT, Flags: UINT)`, and ntdll-sys exports
`pub const Pos: _BUS_DATA_TYPE` (a bindgen enum-alias member). `Pos` therefore resolves as a
constant PATTERN, and rustc reports `E0308: expected u32, found i32` **on the parameter list**,
underlining `Pos` with "expected due to this" under its own declared type — a mismatch between
a parameter and the type written right next to it, which reads as nonsense. Changing `UINT` to
`u32` does not move it. The tell is that the "found" type belongs to neither side of what you
are looking at: here `i32` is `_BUS_DATA_TYPE`, mentioned nowhere in the file. Grep the whole
crate graph for `const <ThatName>` before you doubt the type. Rename the parameter; `mut Pos`
would also force a binding but leaves an `unused_mut` for the lint baseline to eat.

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

**The same blindness scales past arches: one C macro can acquire N private Rust copies.**
`RMAP_SEGMENT_MASK` (`~((ULONG_PTR)0xff)`, arch-invariant) had **six** — `mm/macros.rs` and
`mm/section_table.rs` correct; `inline/mm.rs` pointer-width-dependent (`!0xFFF` on 64-bit);
`ntkrnl-sys/src/mm.rs` `!1usize` with a matching `& 1` predicate; `mm/rmap.rs` the 32-bit-
truncated `0xFFFFFF00`. Two of the wrong three were *harmless* — `!0xFFF` accepts exactly the
same inputs a constructed segment address can take, and rmap.rs's truncation was laundered by an
`as ULONG` cast — which is why they survived: **a wrong copy that happens to be unreachable-wrong
keeps the population alive until one copy is reached.** Constructing with one copy and testing
with another is the failure. When a macro is defined more than once in our tree, the count itself
is the finding; grep the name before trusting any single definition.

### A hand-typed bit-flag literal that names the wrong member `[§9.86]`
`DC_vPrepareDCsForBlit` tested `fs & 0x04` (**DC_CANCELED**) where the C tests
**DC_DIRTY_RAO (0x10)** — so every GDI blit drew through a stale clip, and an app caching
`GetDC` blitted into a rectangle belonging to a previously destroyed window. Symptom looked
like an intermittent render bug and the lane was parked as "clip disproven", because probes had
measured the vis region and `prgnRao` *correctly* at update time. **A hand-typed bit-mask
literal is the flag-cousin of the magic struct offset — treat it as wrong until proven right.**
The constants were always available via the bindgen glob; the literals were pure debris.
**Corollary: when a pipeline has N derived stages, probe the LAST consumer, not the producers.**

**The same class one bit narrower — a PARTIAL mask.** `NtOpenKey` masked `& !0x0100u32` where
the C masks `~KEY_WOW64_RES` (**0x0300**): right family, half the width, and the named constant
was already defined in that same file and used correctly by `NtCreateKey` sixty lines above.
`KEY_WOW64_32KEY` therefore survived into `ObCheckObjectAccess`, which no key's DACL grants, so
`RegOpenKeyEx(.., KEY_WOW64_32KEY)` returned `STATUS_ACCESS_DENIED` while the identical
`RegCreateKeyEx` succeeded — and it hit native 64-bit callers (rapps, `reg /reg:32`, apphelp),
not just WoW64 guests. **An asymmetry between two syscalls that share a contract is the tell:**
neither call looks wrong on its own, so test the pair, not the function.
- **A translated file carries its own oracle.** These files keep the original C as trailing
  comments, so a commented `X &= ~NAME` with no live `!NAME` in the same file detects this whole
  class — no header catalog, no C build. It re-found the bug above without being told, and also
  turned up `ProbeAndCaptureObjectAttributes` masking `0x12F2` where the C says
  `~OBJ_VALID_KERNEL_ATTRIBUTES` (0x107F2), and i386 `KeFlushCurrentTb` having dropped its
  `KF_GLOBAL_PAGE` branch outright — a plain CR3 reload does **not** flush global TLB entries,
  and `patpge.rs` does enable PGE. That last one had no symptom anyone was chasing.
- **Calibrate the pattern or drown:** the live spelling is usually `&= !(NAME as UINT)`, so
  accepting only `!NAME` reports ~4x noise (109 raw hits vs 27).

**Two more from CM, both found by reading the commented C beside the live Rust.** `cmapi.rs`
guarded `NtSetValueKey` on `Flags & 0x00000004` where the C says `KEY_SYM_LINK` (**0x0010**) —
0x0004 is `KEY_HIVE_ENTRY`, so "don't let anyone mess with symlinks" fired on hive ROOTS
(setting any value on `\Registry\Machine\System` and friends returned `ACCESS_DENIED`) and never
fired on a symlink. `cmparse.rs` `CmpDoOpen` tested `ExtFlags & 0x00000040` where the C says
`CM_KCB_SYM_LINK_FOUND` (**0x08**); 0x40 is `CM_KCB_INVALID_CACHED_INFO`, so a resolved symlink
took the reparse path instead of using `RealKcb`, and a KCB carrying `INVALID_CACHED_INFO` read
`ValueCache.RealKcb` out of a union that at that point holds value-cache data. Note that
`cmapi.rs` *already defined* `KEY_HIVE_ENTRY` correctly thirty lines up: having the constant
available is no protection when the literal is typed anyway.

**The `&` TEST form is the bigger half of this class, and the scanner now reads it.** Extending
`scan_mask_drift` from `&= ~NAME` clears to plain `X & NAME` tests found **twelve** more on its
first sweep, in six subsystems, none of which any other lint can see. Worst of them: `ARM3`'s
session-creation interlock tested *and `InterlockedCompareExchange`-SET* `0x200`
(`PSF_WOW64_VA_SPACE_4GB_BIT`) where the C says `PSF_SESSION_CREATION_UNDERWAY_BIT` (**0x4000**),
so it silently stamped a WOW64 address-space flag onto every process that creates a session —
**a wrong literal that is written, not just read, corrupts state rather than mis-branching, and
the test and the set agreeing with each other is exactly why nothing downstream complained.**
Also `cmhvlist.rs` reading `KEY_COMP_NAME` as `0x0001` at four sites (that bit picks byte-per-char
vs UTF-16, so every hive-list path was decoded wrongly in both directions), `ob`'s
traverse-privilege fast path consulting `TOKEN_IS_RESTRICTED`, and kernel32's
`UnhandledExceptionFilter` testing `0x80000001` — a value that is not any `SEM_*` flag — for
*both* error-mode checks.

Three things the extension needed, all worth stealing for the next scanner of this shape:

- **Pair on (BASE, MEMBER), not on the member alone.** `cmapi.rs` tests `Kcb->Flags` against
  `KEY_SYM_LINK` (0x10) and a plain local `Flags` against `REG_NO_LAZY_FLUSH` (0x04); both
  lowercase to `flags`, and the live code held `0x04` in both — correct for the local, the bug
  for the other. First-and-last identifier of the expression left of the `&` maps C `Kcb->Flags`
  and Rust `(*Kcb).__bindgen_anon_5.Flags()` together without modelling member access.
- **A test that VANISHED is not a finding.** Clears report `DROPPED` usefully; a missing test is
  usually restructured control flow and buries the value comparisons in noise.
- **Report only what the file cannot explain another way.** Quiet if the live code also *names*
  the constant for that target, if there is more than one live literal, or if the C open-codes
  that same literal itself — i8042prt masks `RawButtons & 0xfc00` in its own C, with only a
  `DPRINT` beside it naming `MOUSE_LEFT_BUTTON_DOWN`. Those three rules took the sweep from 19
  hits with 7 false to 12 with 0.

**A `try/except` around a catalog load will hide the whole point of the tool.** `scan_mask_drift`
called `build_c_catalog()` with no argument inside `except Exception: _CATALOG = {}`;
`reactos_root` is required, so *every run since it was written* raised `TypeError`, swallowed it,
and compared against an EMPTY catalog — silently downgrading every header-defined constant from a
provable comparison to an `OPENCODED` shrug. It was invisible because the tool still printed
findings and still exited non-zero. A verifier that cannot resolve its oracle must **fail**, not
proceed. And `verify_constants`' curated ~25 headers are too narrow for a general flag check:
`cmdata.h` and `ntoskrnl/include/internal/cm.h` own the registry's own flag families, so a
repo-wide `#define` index (dropping any name with two *different* values) backs it now.

### A registry path that "obviously exists" returns `STATUS_OBJECT_PATH_NOT_FOUND`

Check the CASE of the first component before anything else. The registry root object is inserted
into the root object directory as **`\REGISTRY`**, upper case (`CmpCreateRegistryRoot`), and
`ObpLookupObjectName` only forces case-insensitivity when the *opened type* asks for it:
`if (ObpCaseInsensitive) if (!ObjectType || ObjectType->TypeInfo.CaseInsensitive) Attributes |=
OBJ_CASE_INSENSITIVE;`. Directory, SymbolicLink, Device and File all set that flag. **The Key
type does not, and must not** — `ObTypes.c:217` asserts it against WS03 and expects FALSE. So a
key open whose caller omits `OBJ_CASE_INSENSITIVE` matches that first component *exactly*, and
`\Registry\...` misses `\REGISTRY`. Everything past the first component is CM's own subkey
matching, which is case-insensitive regardless — so only that one component can ever be the
problem.

This is why NT spells every registry path it builds in upper case, and ReactOS does not.
It surfaced through a symbolic link, because a link stores its target verbatim and
`CmpGetSymbolicLink` hands that exact string back as a `REPARSE`: `CmpLinkKeyToHive` wrote
`\Registry\User\.Default`, so `\REGISTRY\USER\S-1-5-18` resolved the link and then failed to find
`Registry` next to `REGISTRY`. The bug looks maddeningly selective because most kernel callers do
pass the attribute — win32k opened `…\S-1-5-18\Control Panel\Mouse` fine all along, and only a
caller passing bare `OBJ_KERNEL_HANDLE` saw it. Grep for `w!("\\Registry` in kernel Rust; each
hit is a latent instance, and the ones that matter are symlink TARGETS.
- **Check the sibling arch before calling it drift** — amd64's `KeFlushCurrentTb` C really is
  CR3-only, so only i386 had a branch to lose.

**Mechanised as `tools/scan_mask_drift.py`** (in `vibeos-cli check`, baseline-gated). Its first
full sweep found the registry's whole `HIVE_*` family open-coded and four of eight copies wrong:
`CmLoadKey` set `0x4` and `CmUnloadKey` `0x40` for `HIVE_IS_UNLOADING` (**0x20**), while
`CmpParseKey` tested `0x4` and `CmpCreateKeyControlBlock` tested `0x200`. Only the load-side set
and one of the two guards agreed *by accident*, so the C's "refuse to resolve a key in a hive
that is unloading" protection could not fire on the unload path it exists for. Separately
`CmpLoadHiveThread` cleared `0x80` instead of `HIVE_NOLAZYFLUSH` (**0x2**), leaving lazy flush
disabled for that hive forever.
- **A wrong flag is invisible in proportion to how consistently it is wrong.** Nothing here was
  a compile error, and the writers and readers each looked locally fine. `verify_constants`
  needs a *name* and these were bare literals; `scan_const_contradictions` needs *two* copies to
  disagree and the one named copy was unique. The relationship to the C is the only oracle left.
- **Fix the whole family, not the finding.** These are now one `pub mod hiveflags` that everyone
  imports; the pre-existing `HIVE_NOLAZYFLUSH = 0x8` sitting in the const-contradiction baseline
  was the same bug, already caught and parked. **A baseline is a work queue** — read it when the
  scanner points you at that neighbourhood.
- **The counting rule is a trap.** "C masks it N times, live masks it fewer" reads a fix's own
  explanatory comment (which quotes the C) as another C occurrence, and a C block repeats a
  statement per call site. The scanner compares *values* and reports a target the live code
  never masks at all; it does not count.

### A C OCTAL literal becomes a Rust DECIMAL one, silently
`file.c` computes `buf->st_mode = _S_IFREG | 0444;` and `|= 0222`. Both literals survived
translation unchanged — and Rust has no C-style octal, so a leading zero is just a leading
zero. `0444` means four hundred and forty-four. msvcrt.dll shipped `st_mode` `0o776` where
msvcrt reports `0o666` — the execute bits set on every regular file, on all three arches,
through six `_fstat*` exports.

**Nothing in the toolchain has an opinion about this.** It compiles, it links, the value is
plausible, and the literal is *three characters* from correct while looking more right in its
C form than its Rust one. It is not the LLM inventing a value (§9.113) — the value was
transcribed perfectly; the BASE changed underneath it. Sealing does not help either: the token
came across intact.

Only a differential test caught it, which is the problem — msvcrt, cnt and rtl are the only
translated modules with a harness, the other ~40 crates have no oracle at all, and every C
source we translate is full of octal mode bits, ioctl numbers and terminal flags.
**Fix: `tools/scan_octal_literals.py`** (empty baseline, gate-wired). It reports a bare
`0[0-7]+` in numeric position *only where the two readings disagree* — `07` is seven either
way, and a lint that flags it gets tuned out. Two false-positive classes it must handle, both
of which it shipped first and then failed on: float exponents (`6.93e-01` — 78 findings in
cnt-rust's math tables alone) and Rust's use of the char-literal quote for LIFETIMES, which
makes `&'static` open a string that a regex-pass stripper closes on the next tick in the file.
The scanner tokenizes left to right for that reason.

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

**The audit was itself blind to a `case` label a MACRO generates**, which is the failure mode
it exists to catch, arriving through its own front door. `find_switch_constants` walks the
*unexpanded* AST for `case_statement` nodes, so msvcrt's `errno.c` — whose 42 arms are all
`ERR_CASE(x)` / `ERR_MAPS(x, y)` — audited as **zero switch constants and a clean bill**. It now
resolves such macros by substituting each parameter with a sentinel and re-parsing the body, so
only a parameter that genuinely lands in a case-label *value* counts (`ERR_MAPS`'s second
argument is the assigned value, not an arm; and the `#define`'s own span must be skipped or the
parameter NAME is reported as a 43rd constant). 77 → 120 constants on msvcrt. **When an audit
reports zero for a file you know dispatches, suspect the audit before believing the file** —
`grep -c 'case '` against the finding count is a five-second control.

**A C ENUM MEMBER is the shape most likely to get past you**, because bindgen renames it and the
translation does not. bindgen prefixes each member with the enum's OWN name, so fmifs's
`CALLBACKCOMMAND { PROGRESS, …, DONE, … }` becomes `CALLBACKCOMMAND_PROGRESS` /
`CALLBACKCOMMAND_DONE`, and shell32 `dialogs/drive.cpp`'s format callback arrived written
against the bare C spellings. `PROGRESS =>` as the FIRST arm is an irrefutable binding, so every
command — including `DONE` — would have run the progress arm and read a `PBOOLEAN` through a
`PDWORD`: no completion message box, no volume label written, and `Result` never set, so every
format reports `SHFMT_ERROR`. Only the later or-pattern arm (`VOLUMEINUSE | INSUFFICIENTRIGHTS |
…`) fails to compile, and it fails with E0408 — "variable is not bound in all patterns" — which
reads as a pattern-syntax complaint, not as a report that the two arms above it are already
broken. **Do not repair such an arm by transcribing ordinals from the C's declaration order**:
here `DONE` is 11 and `DONEWITHSTRUCTURE` is 1. Take the names from the island.

**A constant whose macro body is an EXPRESSION may not reach the island at all**, and this is
the one `profile_derive` already warns about — so the failure is usually that somebody read the
warning as a note. commctrl.h:137 is `#define NM_CUSTOMDRAW (NM_FIRST-12)`; bindgen emitted
`NM_FIRST` and not `NM_CUSTOMDRAW`, so desk's `NM_CUSTOMDRAW =>` arm bound every notification
and the seven arms below it were dead. The header WAS allowlisted — allowlisting is necessary
and not sufficient, because the constant has to fold. `SWITCH_CONST_EXPRESSION: <name> is a
macro EXPRESSION; bindgen may not fold it — verify it lands in the island` is a TODO with a
verification step attached, not an INFO line: after generating the island, grep it for every
flagged name. When one is missing, define it in the prelude **from the header's own macro**
(`NM_FIRST.wrapping_sub(12)`, matching C's unsigned wrap) rather than pasting `0xFFFFFFF4` —
a pasted number stops tracking the header the moment it moves.

**A third way the name goes missing is an A/W macro ALIAS**, and it hides among the other two
because the island looks like it has the constant. shell32's COpenWithDialog matched on
`TVN_SELCHANGED`, `NM_DBLCLK` and `NM_RETURN` in ONE `match`: bindgen kept `TVN_SELCHANGEDW`
and `NM_FIRST`, so the island greps positive for both stems while answering to neither name the
C uses. All three arms bound. rustc objected only to `NM_DBLCLK | NM_RETURN` — an or-pattern
cannot bind — so the error pointed at the two arms that were still REACHABLE and said nothing
about the `TVN_SELCHANGED` arm above them, which was already swallowing every `WM_NOTIFY`.
**Fixing the reported arm is what makes this dangerous**: had the or-pattern been two separate
arms, the file would have compiled silently. When E0408 names a notification or message
constant, check every OTHER arm of the same `match` before touching the one rustc pointed at.
Derive the alias from the island (`TVN_SELCHANGEDW as UINT`, `NM_FIRST.wrapping_sub(3)`), never
from the numeric value.

**The same shape reaches PARAMETER position, where it does not even look like a pattern.** A
function parameter is a pattern too, so an identifier that resolves to a constant is a CONSTANT
PATTERN there as well. syssetup's `SetupChangeLocaleEx(.., char Unknown, ..)` translated to
`Unknown: c_char` — and ntdll-sys glob-exports `pub const Unknown: _MEDIA_TYPE`, so rustc read
the parameter as a match against that constant and reported `expected i8, found u32` pointing at
a declaration that is obviously fine. Read it as "this name is already something else," not as a
type error: the fix is to rename the binding (`_Unknown`), never to change the type. The same
collision would be silent in a `let`, so a C local named after a glob-visible constant is worth
the same suspicion.

**A partial allowlist is the same bug, one layer down.** msvcrt-sys carried 13 of errno.h's 39
codes because the list had grown per-wave on demand — and 9 of `msvcrt_set_errno`'s 18 targets
were missing, including `EDEADLK` (36 in msvcrt, **35 in glibc**) and `ENOTEMPTY` (41 vs 39).
Nothing would have failed to compile; the model would have supplied a plausible host value.
**Allowlist a header's whole family, not the names this wave happens to reference.**

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

**A whole `switch` can be catch-all'd when its labels live in a header the build never
includes.** wldap32's `map_error` matches 16 OpenLDAP codes that only `ldap.h` defines, and
`config.h` says `/* #undef HAVE_LDAP_H */` — so *every* label was unresolved, the first arm
bound everything, and the function returned `WLDAP32_LDAP_SERVER_DOWN` for every input. The
tell was 16 `unreachable pattern` warnings in a row; nothing else fired. **N consecutive
"unreachable pattern" warnings on one `match` means arm 1 is a binding — read it as an error.**
Prophylactic beyond the island audit: if the C guards the `case` labels with an `#ifdef` that is
off in this build, the arms must carry that guard in Rust too — see the build-configuration
guard entry under *Dropped, degraded & silently-rewritten translation output*.

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
macro). Five more causes surfaced once the macro noise cleared, and the split matters because most
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
  **A second mechanism held the opposite opinion about the same construct** (2026-08-05,
  `CDefaultContextMenu::Initialize`). The seal correctly skipped `PCUITEMID_CHILD_ARRAY` in
  `const_cast<PCUITEMID_CHILD_ARRAY>(_ILCopyaPidl(...))`, and then cpp_com's *dropped symbolic
  constant* audit — an independent ALL-CAPS scan — demanded the token back and rejected a body
  that was right. `const_cast` lowers to **nothing**, so no faithful Rust can contain it. When
  two mechanisms judge one construct, they must share the judgement, not each hold an opinion:
  the audit now calls `fact_seal._named_cast_spans` rather than re-deriving a second answer.
- **Sealing hid the call sites from the pass that REPAIRS them** (2026-08-05, `CDefaultContextMenu`
  — five of seven rejects on one TU, each re-billed on every retry). A C++ sibling call spells its
  receiver implicitly, so the prepared C reads `_DoCallback_body(a, b, c)` while the Rust worker's
  first parameter is `this`. `normalize_worker_semantics` has always inserted that `this` — but it
  is keyed to the callee NAME, and under `--seal-calls` the name is an opaque token, so it could
  never fire; meanwhile `seal_calls` adds 1 to `argc` for a receiver callee (deliberately — a body
  that DROPPED `this` used to be accepted, and `CDrivesFolderEnum_dtor` shipped `DeleteList_body()`
  that way). Net: the envelope demanded a receiver the responsible pass could not see. `SealedCall`
  now records `receiver` and the repair runs before the envelope is judged, narrowly — short by
  EXACTLY the receiver, and not already leading with `this`. **The general lesson is about ORDER:
  a validator that runs before a normalisation pass judges text the pipeline was never going to
  ship.** When a seal rejects at scale, check what runs after it. Fixing it re-validated the
  already-paid bodies from the cache at **$0** — a tool fix can recover spend, not just prevent it.
- **A DIRECTIVE is not code, and an array PARAMETER's bound is not a value** (found on shell32
  `changenotify.cpp`, 8 rejected paid attempts across two functions). Three new shapes of the same
  unsatisfiable-envelope class, all now handled by `lex_mask.directive_spans` + `fact_seal.
  _signature_array_bound_spans`. `DumpEvent` defines `DUMPEV(x)` **inside its own body**, pastes
  `SHCNE_##x`, and `#undef`s it at the end; that sealed as `#define __RUSTCONV_CALL_0000__(x)`,
  `__RUSTCONV_FACT_0001__##x` and `#undef __RUSTCONV_FACT_0003__` — a macro DEFINITION name
  demanded back "as the called function identifier in executable Rust", and one half of a token
  paste demanded back as a value. Nothing inside a `#`-directive can be sealed. Separately,
  `DoGetAliasPIDLs(LPITEMIDLIST apidls[2], ...)` sealed the `2`: a C array parameter **decays to a
  pointer**, so the Rust signature has nowhere to put it. Scope that exclusion to the parameter
  list, *not* "everything before the first brace" — `int a[2] = {0, 1};` has its first brace at the
  INITIALIZER, and the loose rule silently stopped sealing a real local's bound (caught only by an
  existing zero-init test).
  **The macro-authority blind spot behind it:** `-E -dM` dumps only macros **still defined when
  preprocessing ends**, so a `#define`/`#undef` pair inside a TU is invisible to it and its
  invocations get sealed as runtime calls. `function_like_macros` now also scans the TU's own text.
  Residual, deliberately left: an ALL_CAPS **argument** to a function-like macro that pastes it
  (`DUMPEV(INTERRUPT)` → `SHCNE_##x`) is still fact-sealed, and restores to `INTERRUPT` when the
  Rust needs `SHCNE_INTERRUPT`. Closing it means threading the macro set into `fact_seal`; until
  then such a function fails LOUDLY, which is the correct behaviour, not a silent wrong answer.
  Diagnosis rule that saves the money: **when a worker is rejected twice for the same token, read
  the sealed prompt (`--dry-run --prompt --fn NAME`, free) before paying for a third attempt.**
  Four retries is the tool telling you the envelope is impossible, not the model being weak.
  **Two more members of the family, both from `folders/CFSFolder.cpp`.** A C++ **default argument**
  sealed as a value: `BOOL AllowFolder = __RUSTCONV_FACT_0000__` (`ExtensionFromPidl`, 3 rejects).
  Rust has no default arguments, so the token has nowhere to go — same helper as the array bound,
  since both are values living only in declarator syntax; guard the comparison operators while you
  are there, or `==`/`!=` opens a default-argument span and swallows the rest of the list. And a
  macro the body ITSELF defines: `_AddFindResult` opens with
  `#define SUPER_HIDDEN (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)`, and cpp_com's body audit
  rejected the worker for `const`-defining it "instead of using substrate" — but no island can
  carry a function-local macro, so defining it *is* the translation. Exempt what the body
  `#define`s; the operands stay sealed and the dropped-constant check still demands the name.
  **A third member, and the one that generalised the helper: a PARAMETER TYPE.**
  `IEnumFORMATETCImpl::Clone(LPENUMFORMATETC* ppenum)` sealed `LPENUMFORMATETC`, and a COM
  method's Rust signature comes from the flattened vtable authority rather than from the C
  declaration — the model never writes that line at all, so the token had nowhere to land on any
  sample, including the sanctioned retry. Once you have seen three of these the rule is obvious
  and the special cases are the wrong shape: a parameter list holds types, parameter names, array
  bounds and default arguments, and **none of those is a value**, so `fact_seal` now excludes the
  whole list rather than two constructs inside it. Two things make that safe to widen. Prove it
  is a definition header first — the balanced `(...)` must be followed by `{` or a ctor `:`, or
  else `Call(TRUE); Call(FALSE);` unseals `TRUE` and `sizeof(UNICODE_NULL)` unseals a real value.
  And it costs nothing already paid for: the resume cache compares seals **directionally**, so a
  banked entry sealed more tightly than the run wants is still a hit — the five bodies banked
  under the old rule replayed at $0 on the very next run.
  Generalisation worth carrying: **every one of these five is the audit demanding something that
  cannot exist.** If a rejection names a token whose only home is C declarator syntax or a
  TU-local directive, suspect the envelope before the model.
- **Compiled is not executed** (found on plain C, Tier 3 — `rustconv/dead_regions.py`). The seal
  walks raw C, so it also seals tokens in code the compiler emits but the build never enters, and
  then DEMANDS them of a translation that is right to omit them. Two flavours, one class:
  `#if`-disabled arms (snmpapi's `DllMain` — `DLL_WINE_PREATTACH` lives inside
  `#ifndef __REACTOS__`), and **`RpcExcept`/`_SEH2_EXCEPT` bodies under `-D_USE_DUMMY_PSEH=1`**,
  where `_SEH2_EXCEPT(x)` expands to `if (_SEH2_Volatile0 || (0 && (x)))` and `sdk/lib/pseh/dummy.c`
  defines `int _SEH2_Volatile0 = 0;` and never assigns it. Nine wlanapi RPC clients referenced
  `WlanRpcStatusToWinError`/`RpcExceptionCode` nowhere else and were rejected as damaged; 10 of 56
  functions in the batch, all correct. Dead-region tokens are now **permitted, not demanded**, and
  every accepted omission is PRINTED (`omitted in non-executed code: TOKEN=SPELLING`) — a
  relaxation nobody can see is how a real deletion slips through later. The `invented` check is
  untouched, so a value that never appeared in the C still fails inside a dead arm. Diagnosis rule
  worth reusing: *before concluding the model deleted your error handling, check whether the C's
  own handler is reachable in THIS build's flags* — read `compile_commands.json`, not the source.

  **The mirror-image failure, and the envelope's blind spot.** The same dead handler can come
  back as LIVE code, and the audit will PASS it. samlib shipped four of
  `Status = I_RpcMapWin32Status(RpcExceptionCode());` emitted as straight-line code right after
  the RPC call instead of inside `RpcExcept` — so it unconditionally overwrote the real status
  with a mapped exception code, and two of the four then branch on `if !NT_SUCCESS(Status)`.
  Both tokens are present, each exactly once, so the envelope is satisfied: **it verifies
  presence and multiplicity, never control-flow POSITION.** (The identical shape in `SamConnect`
  was caught only because its token count happened to come out wrong.) So a dead-region omission
  is now checked, but a dead-region *relocation* is not — when reviewing a sealed RPC wrapper,
  read where the handler landed, not just whether it is gone.

  **And the sharper form of the same blind spot: a body that ignores the envelope ENTIRELY.**
  msvcrt's `_tzset` came back with all 35 sealed constants in the wrong slots — `_lock(0)` for
  `_lock(_TIME_LOCK)`, `memset(&tzi, 0, 3)` for `sizeof(tzi)`, `*tz == 3600` for `*tz == '-'`,
  `tzi.Bias * CP_ACP` for `tzi.Bias * 60`. Nothing invented, nothing missing, the multiset
  exactly right: a **permutation**, which the closed-set check cannot see. The mechanism is in
  `unseal_facts`'s recovery path, written for a cheap model that swaps ONE token for a correctly
  guessed spelling: it re-tokenizes matching spellings **in source order**, so when the model
  wrote real values from memory instead of tokens, recovery re-tokenized the guesses and
  restoration became an identity. The seal reported success over the model's own memory of the
  function. Symptom to watch for: the values in the output are all values the C really contains,
  and several are obviously in the wrong argument (`'-'` compared against `3600`).
  **Fix (landed): `unseal_facts` refuses a body that preserved NO tokens when ≥4 distinct
  spellings were sealed** — distinct spellings, not occurrences, because that is the size of the
  permutation space, and `return 0 + 0 + S_OK` (two spellings) is the benign case recovery exists
  for. The last retry accepts and reports it as an `UNVERIFIED CONSTANTS` audit finding rather
  than blocking the file. Rate is not a tail case: it fired on `_ecvt_s` (31 tokens), `_fcvt`
  (21), `_setfp` (129), `msvcrt_valisttos`, `_wsystem` and `MSVCRT_asin` within one math.c +
  process.c pair, all of which would have shipped unverified. **Corollary for reviewers: a green
  `--seal-facts` run is evidence about the value SET, never about placement — read the
  constant-dense functions.**

  **Why it concentrates in msvcrt, and what to do about it: SEALING IS DEFEATED BY RECALL.**
  The sealed body handed to the model has the tokens inline, so writing real values back means
  the model did not translate the text in front of it — it recognised the function and wrote it
  from memory. msvcrt is one of the most-reproduced C bodies in existence, which is exactly why
  this shows up here and not in the driver waves, and it is also why RETRYING DOES NOT HELP:
  recall is stable across samples. `_setfp_sse` and `sqrt_validate` each burned six identical
  retries before being accepted-and-flagged. **Operationally: use `--retry-rejects 1` on a
  famous module** — one honest re-roll, then take the `UNVERIFIED CONSTANTS` list from AUDIT.md
  and diff those bodies against the C by hand. Retrying a recalled function is the same paid
  call over and over.

  **Calibration, so the list is read correctly: a recalled body is usually RIGHT.** Hand-diffing
  math.c's flagged bodies against the C found `MSVCRT_asin` (`0x3ff00000`, `_MCW_EM|_MCW_RC`,
  `_DOMAIN`) and `sqrt_validate` (`FP_ZERO`/`FP_NAN`/`FP_INFINITE`, the
  `0x8000000000000ULL` signaling bit) faithful in every slot. `_tzset` is the exception, not
  the rule. So `UNVERIFIED CONSTANTS` is a REVIEW QUEUE, not a defect list — treat a long one
  as a reading assignment rather than evidence the wave is broken, and do not "fix" a flagged
  body you have not first diffed.

  **But the guard fires only on WHOLESALE omission, so the flagged list is not the whole
  risk.** It triggers when *no* token survived; a body that preserves some tokens and permutes
  the rest passes silently and never reaches the list. `MSVCRT_exp` is the worked example, and
  it was never flagged: the C is `math_error(_DOMAIN, "exp", x, 0, 1.0 + x)` and the
  translation wrote `math_error(0, "exp", x, 1, 0 + x)` — `_DOMAIN` demoted to `0`, which is
  precisely the "don't set errno" case, so a NaN into `exp()` would have returned quietly
  instead of setting `EDOM`. Two more constants moved with it. Nothing in the audit said a
  word. The cheap generic defence, and the one that found it: **enumerate every call to a
  shared error/dispatch helper on both sides and diff the argument lists as a table.** All 11
  `math_error` call sites took under a minute that way, and the eleventh was the bug. The real
  fix is to anchor each fact token to its `(callee, argument index)` at seal time so a
  permutation cannot satisfy the envelope at all — tracked, not yet built.

  **CALL tokens permute too, and there the envelope DOES catch it — because arity is a
  second, independent witness.** Same disease as the constant permutation above, seen on
  DeepSeek v4-pro translating `COpenWithList::LoadInfo` (2026-08-06). The C is
  `HeapFree(GetProcessHeap(), 0, pBuf); return FALSE;`; the model emitted the `HeapFree` token
  as a **bare zero-argument call** and put the real 3-argument call on a different token:

  ```rust
  __RUSTCONV_CALL_0005__();                                     // = HeapFree, 0 args
  __RUSTCONV_CALL_0011__(__RUSTCONV_CALL_0006__(), __RUSTCONV_FACT_0010__, pBuf);
  ```

  `--seal-calls` rejected it loudly and precisely: *"wrong call site (argument count):
  `__RUSTCONV_CALL_0005__`=HeapFree wants 3, got 0; not exactly once:
  `__RUSTCONV_CALL_0011__`"*. Note WHY this is caught while the constant permutation is not:
  a call token carries an **arity and an occurrence count** that must both hold, so a swap
  breaks an invariant the closed-set check cannot express for a bare constant. That is the
  shape the tracked `(callee, argument index)` fact anchoring above is trying to buy for
  constants — calls already have it. Unsealed, this body would have compiled as a plausible
  `HeapFree(GetProcessHeap(), 0, pBuf)` somewhere adjacent, in a function whose whole job is
  freeing a version-info buffer down three exit paths.

  Shared-cleanup CFGs are the recurring habitat: this TU's one reject in 29 is the same
  function shape (multi-exit, one buffer, repeated free) the pipeline already measured as the
  DeepSeek reject class. Retrying it with a stronger model measured **0/3 accepted for
  $0.13** — treat it as decomposition work or hand-write the body, not a model upgrade.

  **`--seal-calls` has the same permutation weakness, and its arity oracle was switched off by
  a turbofish.** msvcrt's `_beginthread_trampoline` shipped
  `msvcrt_get_thread_data(&local, arg, sizeof)` where the C has `memcpy(...)`, `memcpy(arg)`
  where it has `free(arg)`, and `free(local.arglist)` in place of the
  `local.start_address(local.arglist)` call that is the whole point of the function — a thread
  that leaks its argument block and never runs the user's entry point. Every sealed callee
  appeared exactly once and executable, so the closed-SET invariant was satisfied by a slide.
  `_arity_disagreements` exists precisely to catch a slid envelope and HAD the evidence —
  `msvcrt_get_thread_data` takes zero arguments and was called with three — but it skipped any
  site whose argument text mentioned `::<`, and the third argument was
  `core::mem::size_of::<T>()`. That is not a corner: it is how nearly every `memcpy` lowering
  spells its length, so one over-broad guard disabled arity checking across a whole call family.
  **Fix (landed): `_mask_turbofish_commas` blanks only the commas inside a `::<…>` span** —
  length-preserving, `->`-aware, nesting-aware — instead of discarding the site; a callee-side
  turbofish needs no handling because `_argument_count` already returns None when anything sits
  between the name and the `(`. Tell for a reviewer: the shipped body was byte-for-byte
  reproducible by replaying the seal, and a near-identical sibling 40 lines away
  (`_beginthreadex_trampoline`) came back correct — **nearby-body contrast is the cheapest
  signal that the fault is placement rather than comprehension.**

  **An ABSENT constant is not sealed — it is INVENTED, and no audit fires.** Sealing can only
  seal a name the `-sys` island CARRIES; a constant the island lacks never becomes a token, so
  the model writes it from memory and the envelope has nothing to check. This is the mirror of
  the permutation above: permutation needs the constant PRESENT, this needs it ABSENT.
  msvcrt's `_heapset` is the worked example (wave 26, `--seal-facts` on, audit clean): the C
  is `while ((retval = _heapwalk(&h)) == _HEAPOK)` and `return retval == _HEAPEND ? _HEAPOK :
  retval`, and the translation shipped `if retval != 1 { break }` and `if retval == 5 { 1 }`.
  `_HEAPOK` is **-2** and `_HEAPEND` **-5**, so the loop body never executed and the return
  came from the wrong namespace entirely — a `_heapset` that fills nothing. The whole `_HEAP*`
  family was missing from the island at call time.
  It also sits in `profile_derive`'s blind spot: its switch-constant audit reads `case` labels
  only, and every one of these is in a `while` or a `?:`. **Fix: run `profile_derive` first
  and treat a constant used in ANY control expression the way it treats a `case` label — if
  the family is not in the island, put it there BEFORE paying.** Detection after the fact is
  the same table diff as above: list the C's ALL-CAPS vocabulary and check each is SPELLED BY
  NAME in the Rust; a bare literal where the C names a constant is the finding.

  **Diff constant COUNTS against the compiled C, never against the file.** The same sweep on
  heap.c showed `EINVAL` 13 in the C vs 9 in the Rust, `ERANGE` 8 vs 4 — which reads as four
  deleted parameter checks. Both were correct: the missing occurrences live in
  `#if _MSVCR_VER>=80/140` arms that this DLL does not compile. Strip the untaken arms first
  (`tools/verify_translation_literals.py:strip_untaken_arms` already does it) or the check
  produces confident false positives and trains you to ignore it.

  **The cheapest defence that catches this class wholesale: diff every translated constant
  against its header, mechanically, and do not wait for the audit to nominate one.** msvcrt's
  wave 23 (console/misc/exit/locale) came back with the audit reporting ZERO findings and
  still carried two permuted constants, both of which compile, link and run:

  * `exit.c`'s `_EXIT_LOCK1` as **8**, which is `_CONIO_LOCK` two `#define`s earlier in the
    same mtdll.h table. Effect: `_cexit`/`_onexit`/`atexit` serialize on the CONSOLE lock —
    racing each other while deadlocking against `console.c`'s `LOCK_CONSOLE`.
  * `locale.c`'s `cloc_time_data.lcid` as `0x0409`, a real LCID
    (`MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US)`) where the C says
    `MAKELCID(LANG_ENGLISH, SORT_DEFAULT)` = `0x0009`.

  Both fit the pattern exactly: not invented, not out of range, just the WRONG member of the
  right family. A twenty-line script that extracts every `const NAME: T = V;` from the
  translated module and greps the module's own `.c` and its header closure for
  `#define NAME` found both in seconds — including the `#if`-arm case, where the C's two
  definitions must first be narrowed to the one this `_MSVCR_VER` compiles. Run it on every
  translated module; it costs nothing and needs no model.

  The same wave's structural sibling is worth naming with it, because the fix is the same
  reflex: `cloc_time_data`'s initializer also assigned a **`wrefcount` field that does not
  exist** at `_MSVCR_VER=0` (it is the `>= 110` spelling), which shifted the C's `1, -1` one
  slot each. `refcount = -1` is the "statically allocated, never free me" sentinel
  `free_locinfo` tests, so the C locale's time data would have looked refcounted and
  freeable. Rust caught that one as a missing-field error only because the layout came from
  bindgen; a hand-written struct would have accepted it.

  **The durable fix for all of it is to stop transcribing.** Every constant in this list is
  now DERIVED in the `msvcrt-sys` bindgen island — the mtdll.h lock numbers, the
  `LC_`/`LANG_`/`SORT_` families — so the class cannot recur for those names. When adding a
  family to an island, check the regex actually matches: `LOCALE_[A-Z_]+` had silently
  matched **none** of the 42 digit-suffixed `LOCALE_SMONTHNAME1`..`12`/`LOCALE_SDAYNAME1`..`7`
  LCTYPEs, and a constant absent from an island fails by becoming an invented literal, not by
  failing to build. Grep the emitted bindings for a name you just allowlisted — that is the
  only way this shows up. (`RtlGenRandom` is the other shape of the same trap: ntsecapi.h
  `#define`s it to `SystemFunction036`, so the prototype bindgen sees is already renamed and
  allowlisting the documented name matched nothing.)

  **STRING LITERALS are the same class and were the blind spot: compare them as a MULTISET
  against the C, per file.** The constant diff above only reaches things spelled as
  constants. A demangler, a formatter or a path builder carries its behaviour in string
  literals instead, and there a permutation is invisible to every other check — the SET of
  strings in the file is unchanged, so nothing is missing, invented, or out of range.
  `tools/verify_translation_literals.py` is that comparison, and it found two the first time
  it ran:

  * `undname.c`'s calling-convention switch shipped `case 'C'/'D'` as `"__stdcall"` where the
    C says `"pascal"` — and `"__stdcall"` is a genuine member of that very switch, eight
    lines down in its `else` twin. Every `?f@@YCXXZ` would demangle wrong, forever, silently.
  * `time.c`'s `GetTimeFormatW(..., L"HH':'mm':'ss", ...)` shipped as `HH:'mm:'ss` — in a
    picture string `'` QUOTES the character it surrounds, so the C forces a literal colon and
    the translation asked for the locale's separator, with an unbalanced quote left over.
    This one had already LANDED and was green on every gate it ever ran.

  For a LOOKUP TABLE the multiset is not enough, and undname.c is the worked example: 155
  `case 'X': ... "name"` pairs, and a translation that swapped two of them leaves the
  literal multiset *and* the case-label set both perfectly intact. The sharper check is to
  extract the ordered `(case label, first string literal in the arm)` sequence from both
  files and `difflib` them — 155 pairs, and the only diffs were an extractor artifact
  (grouped `case 'A': case 'B':` reports its LAST label while a Rust `b'A' | b'B' =>` arm
  reports its first). Same idea for a pure `code -> string` function like `get_simple_type`:
  build the map on both sides and compare it, not its values. Worth ten minutes on any file
  whose job IS a table.

  Multiset, not set, is the whole point for the file-wide check: a literal moved from one arm
  to another leaves the set identical and changes exactly two counts. Make it usable rather than noisy by teaching
  it what is representation instead of content — assembly text (`__ASM_GLOBAL_FUNC` vs
  `global_asm!`), no-op debug channels, `L"..."` spelled as interleaved bytes, `\0` vs
  `\x00`, a `static char[64]` built rather than written — and above all the C arms this build
  does NOT compile. That last one is not optional: `asctime`'s `"%s %s %2d ..."` is the
  `_MSVCR_VER>=140` arm and the shipping one is `%02d`, so a checker blind to `#if` sends you
  hunting a bug that is not there. Two smaller traps in writing the checker itself, both of
  which silently UN-skip content: Rust loop labels (`'fail:`) look like the start of a char
  literal and desynchronise a naive comment scanner for the rest of the file, and a `;` in a
  `#[cfg(any())]` attribute's own trailing comment ends the "find this item" scan early.

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

**The seal covers function BODIES; the file-scope PREAMBLE has no such protection.** Symptom: an
assembled module references `CPP_COM_WSTR_<long digit run>` statics that were never declared, so
it fails as an undefined name. Cause: static initializer data goes through the preamble worker,
outside the fact/call envelope, and the model invented two plausible-looking literal names for
`g_RegFolderInfo`'s `pszRelativePath` and `pszName` in `folders/CControlPanelFolder.cpp`. Fix:
transcribe the literal from the C. It failed loudly only because an undeclared name does not
compile — a preamble mistake that *is* spellable (a wrong length, a wrong flag in an initializer)
has nothing catching it, so read assembled preambles against the C.

**When a worker is rejected, read the rejection before re-buying it — twice tonight the audit
had caught a genuine semantic bug, not a formatting one.** `CNetFolderBackgroundMenuCB` reused
the `DestroyMenu` token for the tail call to `SHELL32_DefaultContextMenuCallBack`, which would
have dropped default handling for every unhandled message *and* passed a shell-folder pointer to
a menu-destroying API. `CreateCplAbsoluteParsingPath` permuted three separate calls: it tested
`StringCchPrintfW(hr, 0)` as the `SUCCEEDED` predicate and then called `SHFree` with
StringCchPrintfW's six arguments. Neither is a near miss, and nothing except the seal would have
caught either. The rejected body is kept under `.responses/<run>/__rejected__/` as a forensic
copy; assemble around it with `--fn <comma-separated accepted set>` for $0 and hand-write the
one function.

---

## Dropped, degraded & silently-rewritten translation output

The model does not only get things wrong — it omits them. Every entry here compiles.

### A "stuck sampler" is almost always a harness bug wearing the model's clothes

**Symptom.** A TU will not assemble. One or two functions in it fail the deterministic audit
with the *same finding every time*, across independent samplings, while every other function
in the file passes. The natural read — "sampling noise on the biggest bodies, needs a
different approach" — is what parked netcfgx at 6 of 11 TUs for a week. Four separate causes
hid behind that one symptom, and three were ours.

**Cause 1: the audit ran twice.** `_audit_paid_body` unsealed the plain-C reply and then
unsealed the *unsealed* text again. The second pass sees no tokens, so every token reads as
missing; it passed at all only because `unseal_calls`'s by-name recovery re-sealed them from
the names the first pass had just restored. Any body where that round trip is not exact — a
`switch` fallthrough tail spelled twice for one C occurrence, a `goto` tail written by name —
was ACCEPTED by the first pass and REJECTED by the second. Cache hits ran it too, over banked
text that is unsealed by construction, so a false reject there sent already-paid work back to
be bought again.

**Cause 2: a SHIFTED envelope passes every count-based check.** A function-like macro gets no
call token, by design — there is no 1:1 Rust call to carry one. A model that assumes every call
it writes needs a token spends CALL_0000 on the macro, and every later token slides one call to
the left. The token count is still right, each appears exactly once, none is invented; only the
last one runs out, and the by-name recovery rescues even that. Unsealing then restores the
WRONG NAME at every slid site — netcfgx's `ApplyOrCancelChanges` came back calling
`CoTaskMemFree` with `swprintf`'s three arguments. Fix: compare the ARGUMENT COUNT at each
token's C site and its Rust site. A permutation cannot preserve arity except by coincidence,
and the check found 0 disagreements across 164 already-accepted bodies. Skip the sites where
the counts are not comparable — a C argument mentioning a macro (`IID_PPV_ARG` is one C
argument and two Rust ones), a turbofish, and rustfmt's trailing comma, which reads a
six-argument call spread over seven lines as seven.

**Two more places the two counts are not directly comparable, both found the expensive way.** A
C++ **default argument** (`SHELL_GetAutoRunInfPath(PCWSTR, PWSTR, BOOL ForInvoke = FALSE)`) lets
the C site pass two where a faithful Rust lowering must pass three, so such a callee needs a
RANGE and the declaration has to be read from the whole TU, not the sealed body. A C++ **method**
call carries an implicit receiver that `--cpp-com` spells as an explicit leading `this`, so the
Rust site has exactly one MORE argument than the C one. The tell for both is the documented one:
two different functions failing identically on one callee means the checker is at fault.

**The receiver case is the one to internalise, because it failed in BOTH directions at once.**
On `CDrivesFolderEnum` it rejected `AppendItemsFromEnumerator`, which had written the correct
`CALL_0000(this, pidl)`, twice — burning two paid calls and voiding the whole TU's assembly. And
on the destructor it did worse: attempt 1 wrote the correct `DeleteList_body(this)` and was
rejected, attempt 2 wrote `DeleteList_body()` — no receiver, against a fn whose first parameter
is `this` — and was ACCEPTED and shipped. **The retry loop drove the model from right to wrong
in order to satisfy a broken check.** A too-strict envelope does not merely cost money; with a
retry behind it, it manufactures corruption. When you loosen one, check what the *previously
accepted* bodies did at that site — the `__rejected__/` forensic copies are how the correct
destructor was recovered, at $0.

**And a banked body is replayed WITHOUT re-running the audit**, so fixing an envelope is
retroactively inert: the broken destructor above still replayed as `CACHED ($0)` and still
shipped, until its cache entry was repaired by hand. Re-auditing on replay costs nothing (the
audit is deterministic); what it needs is a measured sweep of which banked modules it would
newly flag, because with `--retry-rejects 0` a newly-failing body refuses to assemble.

**Cause 3: the retry re-sent the identical prompt.** That is a re-roll, not a repair, and the
rejects that survive to a second attempt are systematic rather than noisy: the model is
IMPROVING on the C. `ScanForInfFile` refused three samplings to reproduce a `continue` that
skips its own `FindNextFileW`; `CreateInstanceKey` hoisted the `goto done` tail per path and
lost the `if (ret == FALSE)` guard with it. Feed the audit finding into the retry, and say that
the C is authoritative *even where it is wrong* — if it loops forever or leaks, reproduce it.
That converted one of the two immediately.

**Cause 4 (not a seal bug at all): the scanner refused the file.** See the `STDAPI` entry below.

**The rule.** When one function fails the same way twice, stop sampling and read the finding.
Replay the stored `__rejected__` reply through the audit by hand; if it passes now, the harness
moved and the body was never wrong. `translate` does this automatically — a reject is paid
artifact, and it is re-audited before that function is bought again.

### A COM-class TU is only HALF translated by `--cpp-com`, and nothing says so

**Symptom.** `rustconv translate --cpp-com --class X` reports every worker accepted and
assembles a file, but the TU's free functions are simply absent from it. On shell32's
`CDefaultContextMenu.cpp` that was 14 of 57 top-level definitions — the whole `DCMA_*` menu
array API, both `*_RustBridge` entry points, `SHCreateDefaultContextMenu`, the file-scope
`g_StaticInvokeCmdMap` verb table. Nothing in the run's output is red; the gap only surfaces
as a pile of `cannot find function` errors once the module is wired, and it is easy to
misread those as missing HEADER inlines and start hand-writing ports of functions the TU
already contains.

**Cause.** `--cpp-com` translates the class; the rest of the TU is a separate
`--free-functions` run. That is by design (it is the shape `CNetFolder` used), but the class
run does not enumerate what it left behind. The one hint is easy to miss, in the integrate
line: `[c-source] embedded only this slice's C; elided N line(s) belonging to functions
another module owns`.

**Fix.** The class run now NAMES what it left behind — `[cpp-com] X: skipping N file-scope
function(s) — these need a separate --free-functions run on the same file: …` — mirroring the
report `--free-functions` has always printed for the class methods IT skips. Before that
existed the omission was silent, and shellrecyclebin duly lost all nine of its `extern "C"`
entry points, the whole `*_Constructor` family that is how anything obtains one of those
objects. Read the line, then do the second run.

Note what the loud failure depended on: those nine surfaced only because `cargo check`
resolves Rust paths standalone and other Rust modules happened to call them. A dropped
file-scope function that no Rust module calls passes `cargo check`, and once the TU is
cohab-flipped it link-resolves to the still-compiled C — a half-Rust module reporting nothing.

Historically you diffed the C++'s top-level definitions against the assembled Rust by hand. **It needs `-o`:** both runs derive their output path from
the same TU, so the second would overwrite the first — rustconv refuses rather than discard,
which is the fail-loud behaviour working, and names the flag. Give the free half its own
`<TU>_tu.rs`, and expect an overlap of a few helpers the class run already pulled in for its
own callers (they are `_body`-suffixed there, so they do not clash).

**Corollary — hand-written code does not survive a re-run.** Each half is overwritten
wholesale. Anything hand-written that both halves need (header-inline ports, constants the
island lacks, externs) belongs in a THIRD, hand-owned file that no run writes.

### A convention macro that also carries the RETURN TYPE deletes the function

**Symptom.** `rustconv translate` refuses a TU: *"N SPEC-EXPORTED function(s) are in the
unextractable set — this is a scanner parse failure, not a macro artifact."* Or, worse, no
complaint at all and the function is simply absent from the plan.

**Cause.** The scanner normalizes every calling-convention alias tree-sitter cannot parse to
the raw `__stdcall` keyword. That is right for `WSAAPI`, and wrong for the family that expands
to more than a convention: `STDAPI` is `EXTERN_C HRESULT STDAPICALLTYPE`, so normalizing it
leaves `__stdcall DllRegisterServer(void)` — a definition with no type, which is not a
function_definition. netcfgx.c lost `DllGetClassObject`, `DllRegisterServer` and
`DllUnregisterServer`, all three `.spec`-exported. Same class as the `TYPE * CDECL fn()` drop
(`0b2d3521e0a`) that cost msvcrt 15 exports: a convention keyword taken for the whole
decoration.

**Fix.** `conventions.KW_EXPANSION` holds the eleven aliases that are really
`[linkage] <TYPE> <conv>`, valued with the header's own expansion verbatim and resolved by
fixpoint (`WINOLEAPI` → `STDAPI` → `EXTERN_C HRESULT STDAPICALLTYPE`), so the existing
`EXTERN_C` and alias passes finish the job. Adding one is cache-safe *only* while the function
it rescues was being dropped: a body that already extracts is already banked under its current
text, and rewriting its signature re-bills it. `STDAPI_(t)` / `STDMETHODIMP_(t)` are still
open for that reason — they extract today with an EMPTY return type, and all 51 uses are C++
COM methods whose signatures come from the interface anyway.

### A C `switch` that falls through makes a faithful translation REPEAT a call token

**Symptom.** A paid body is rejected with `model damaged deterministic call envelope (not
exactly once: …)` naming several — sometimes all — of its tokens, and the Rust is correct.
One rejected worker assembles no file for the whole TU.

**Cause.** Rust `match` has no fall-through. The only way to lower a C arm that runs on into
the next one is to write the next arm's body into it as well, which emits every call token in
that body twice. ntmarta's `AccpGetTrusteeSid` falls `TRUSTEE_IS_OBJECTS_AND_NAME` through
into `TRUSTEE_IS_NAME` and duplicated all five of its tokens.

This is the same phenomenon `goto` causes — Rust has no `goto` either, so every lowering
duplicates code — reached by a different route, and the `goto` concession did not cover it.

**Fix.** `_replicable_after_goto` is positional, not numeric, and now starts at the earlier of
the first `goto`/label and the first `case` label whose arm falls through. Detection is
deliberately conservative: an arm counts as falling through unless it visibly ends in
`break`/`return`/`goto`/`continue`, and one ending in `}` is AMBIGUOUS — the jump may sit
inside the block, which is exactly the ntmarta shape — so it counts as one. An empty region
between two labels is a shared arm (`case A: case B:`), duplicates nothing, and concedes
nothing. Only the DUPLICATE half of the envelope weakens; `missing` still catches a dropped
call and `invented` still catches a hallucinated token.

**And say so in the contract.** The `goto` branch already documents why: told "do not
duplicate" when the C forces duplication, models MINT NEW TOKEN NUMBERS past the end of the
list rather than repeat one. The fall-through branch now gets the same sentence.

**The rejection that was real, from the same run.** Do not read this entry as "the audit
over-rejects." `AccRewriteGetNamedRights` was rejected for duplicating `SetLastError`, and
alongside it had written `let Ret = AccRewriteGetHandleRights(…)` INSIDE the `if` — shadowing
the outer `Ret`, so the function would have returned the open result instead of the query's,
silently, on the success path. `--retry-rejects` with the audit finding fed back fixed it.

**The envelope pushes the other way too: a call it demands be EXECUTABLE can turn a
never-taken branch into straight-line code.** Its invariant is "every call in the C appears
exactly once and is reachable", and a model that cannot satisfy that inside a dead arm will
satisfy it by deleting the arm's condition. msvcrt's `cpp.c` guards `__RTtypeid`,
`__RTCastToVoid` and `__RTDynamicCast` with `__TRY … __EXCEPT_PAGE_FAULT { throw
__non_rtti_object } __ENDTRY`; under DUMMY_PSEH (which is what msvcrt compiles with) that
`__EXCEPT` is literally `if (_SEH2_Volatile0 || (0 && filter))` — compiled, never taken. All
three bodies were rejected once for a damaged envelope and all three came back on retry with
the handler **unconditional**, i.e. every RTTI query would throw. It compiles, links, and the
audit is satisfied.

So: **whenever a function the seal rejected comes back accepted, diff its control flow, not
just its calls.** The retry is where this class is born, because the first answer was the one
that respected the branch. And when the C's protected region is DUMMY_PSEH (check for an
undefined `_SEH2_Volatile0` in the built object — no `__C_specific_handler`, no scope table),
the faithful lowering is the body followed by `if _SEH2_Volatile0 != 0 { handler }`: one
instruction, same as the C, and it keeps the handler compiling so it cannot rot. Deleting the
handler as "unreachable" is the other wrong answer — it throws away the only record of what
the C intends to do when the walk faults.

### `profile_derive`'s BLOCKING constant findings were checked against the wrong `-sys` crate
**Symptom.** `profile_derive <module> --emit report` reports `✗ ERROR SWITCH_CONST_IN_BASE_HEADER`
for constants you can see with your own eyes in the module's island, and the finding cannot be
satisfied by adding them — because they are already there. Read the crate name in the message:
*"is NOT in the -sys crate (**ntdll-sys**)"*.

**Cause.** `sys_crate_consts` hard-coded `ntdll-sys` for usermode and `ntkrnl-sys` for kernel.
That is only the DEFAULT. A module with its own bindgen island — msvcrt, win32k, shell32-com —
glob-imports THAT crate, so every constant in it reads as absent. msvcrt reported six, all six
already in `msvcrt-sys`.

This is worse than noise because these findings are **BLOCKING** and are step 1 of the plain-C
protocol: a gate that cannot be satisfied is a gate that gets ignored, and the next real
`SWITCH_CONST_IN_BASE_HEADER` — the class that shipped a BUGCHECK 0x1E — scrolls past with it.

**Fix.** `resolve_sys_crate()` picks `<module>-sys` when one exists on disk and falls back to the
mode default, the same existence-proof `rustconv translate --target` uses for the output crate.
After it, msvcrt goes 6 blocking errors → 0, and `shell32`/`afd` still resolve to the defaults.
**Whenever a deterministic gate reports something you are confident is false, check what it is
comparing against before you argue with the finding.**

### A bindgen enum member's BARE name is missing even when its header IS allowlisted

**Symptom.** `verify_match_consts` reports `[HIGH] <Name> -- not defined in-file and not defined
anywhere in this crate's dependency closure -> silent catch-all binding [+followed-by-arms:
unreachable smell]`. The crate compiles. The `match` has one live arm and the rest are dead.

**Cause.** bindgen emits a C enumerator as `pub const <TAG>_<MEMBER>: <TAG> = n;`. The bare
`<MEMBER>` the C `switch`es on is not in the island at all, and an unresolved name in a Rust
`match` arm is not an error — it is a fresh binding that matches EVERYTHING. `profile_derive`
does not catch this: its `SWITCH_CONST_HEADER_ADDED` asks whether the constant's owning HEADER
is allowlisted, which for an enumerator is necessary and not sufficient. On ntmarta it reported
one finding out of seventeen; on authz, zero out of seven — and all seven of authz's were
`case` labels in `AuthzGetInformationFromContext`, so every information class would have taken
the first arm.

**Fix.** Alias each one in the crate prelude, never re-declare with a literal:
`pub const DENY_ACCESS: _ACCESS_MODE = _ACCESS_MODE_DENY_ACCESS;`

**Check it BEFORE paying, and make the check multi-line.** Walk the generated island for
`pub const <TAG>_<X>: <TAG> = n;`, intersect the bare `<X>` with the identifiers the `.c` uses,
and subtract anything the island already defines bare. The trap is in the scan itself: bindgen
WRAPS the declaration when the name is long —

```rust
pub const _AUTHZ_CONTEXT_INFORMATION_CLASS_AuthzContextInfoPrivileges:
    _AUTHZ_CONTEXT_INFORMATION_CLASS = 4;
```

— so a single-line regex reports a confident **zero findings** for exactly the module that
needs it most. That is how authz's seven got past a hand-rolled check and had to be caught by
`verify_match_consts` at build time. Use `re.DOTALL` and allow `\s*` after the colon.

### A deferral note outlives the premise that justified it
`ARM3/virtual.rs` skipped an ARM64-only block of `NtAllocateVirtualMemory` and said so:
*"Skipped in this Rust port — arm64 boot currently fails before user-mode allocation, so this
path isn't exercised. Restore from virtual.c:4819-4869 once arm64 boot is fixed past
CmCheckRegistry."* Honest, correct when written, and self-invalidating: arm64 has booted to a
desktop for months, so the "isn't exercised" clause stopped being true and the note quietly
became a live memory corruptor.

What it was guarding matters. FreeLDR's identity-mapped page-table **pages** get recycled as
user-space page tables while still holding FreeLDR's entries (VA==PA, global, EL1-only), and on
ARM64 TTBR0 *is* the user half. A fresh user reservation therefore lands on valid-looking
descriptors: the fault that should demand-page it never fires, and teardown decodes the stale
descriptor into a PFN naming no frame (VA `0x50000` → pte `0x50703` → PFN `0x50`, far below
QEMU virt's `0x40000000` DRAM base). `MmDeleteRmap` then reports mapping-not-in-list and
`MmReleasePageMemoryConsumer` hands the allocator a frame that was never allocated — one
deterministic corruption presenting as four different stop codes plus clean runs, depending on
what later reused the frame. amd64 has the equivalent guard in `ARM3/sysldr.rs`, which is
exactly why 148 x86 sweep rounds never reproduced it.

**A deferral whose condition is a moving fact needs an executable trigger, not a sentence.** A
comment cannot notice that its premise expired. Either wire the condition to something that
fails loudly when it flips, or do the work. The tell when hunting: a note that names both the
restoration source and a precondition — check the precondition *first*, because if it has
already been met the note is describing a bug, not a plan.

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
- **A prefast declaration written with the function TYPE becomes a null STATIC, and takes the
  body with it.** fastfat's devctrl.c:32 says `IO_COMPLETION_ROUTINE FatDeviceControlCompletionRoutine;`
  — `<function type> <name>;`, no `*`, which declares the FUNCTION. That reads exactly like a
  variable declaration, so it came back as
  `pub static mut FatDeviceControlCompletionRoutine: IO_COMPLETION_ROUTINE = None;` and the real
  body 350 lines further down was never translated at all. The call site then handed
  `IoSetCompletionRoutine` a `None`, nothing signalled the event, and the
  `KeWaitForSingleObject` two screens below — null timeout — waited forever. Nothing else could
  catch it: a PRIVATE function that vanishes takes its callers' references with it, so there is
  no unresolved symbol and no `.spec` entry to audit. Same trap as the named-alias asymmetry
  under "A file-local fn-pointer TYPEDEF": `R (NAME)(args)` and `R (*NAME)(args)` are different
  declarations and only the second is a variable.
- **The oracle for this whole class is the C TEXT, not the plan** —
  `tools/rustconv/fn_coverage.py <module>` pairs each `<name>.rs` with its `<name>.c` and reports
  C functions with no Rust definition. It is cohab-aware (a `.c` still in the CMake source list
  provides its own functions and is skipped), understands `#if 0`, SAL annotations and
  macro-expanded definitions (`CSR_API(SrvFoo)`), and reports the reverse direction — a Rust fn
  whose name the C never mentions — as a possible INVENTED callee. It does not evaluate
  `#if`/`#ifdef` generally, so a config-gated function is the known residual false positive
  (`#if DBG` bodies get their own bucket and do not count; a function alive only in the `#else`
  of `#if defined(_USE_DUMMY_PSEH)`, like winlogon's `GinaLoadFailedWindowProc`, still shows).
  **Run it after every flip.**
- **A coverage tool's false positives are the whole game — six defects hid the one real bug.**
  The first sweep over 11 modules produced ~110 rows, of which two were real; the rest were the
  tool failing to SEE a Rust definition, and any of them could have buried a genuine drop in
  noise. Each is a distinct way a definition does not look like `fn NAME`:
  **(1)** the `#[no_mangle]` attribute shares the line with the signature, which is how every
  export is emitted — an anchor at `pub` missed all of them (cabinet `FDIIsCabinet`, dnsrslvr
  `CRrReadCache`); **(2)** the crate keeps hand-written modules at `src/` top level while the
  scan only walked the module SUBDIRECTORIES (kernel32's `interlocked.rs` — all four
  `Interlocked*` the spec exports); **(3)** the ABI is a macro parameter, `extern $abi fn`, in
  the per-arch twin macros; **(4)** the body is emitted by INVOKING a macro that takes
  `fn NAME(..)` as its argument — ole32/oleaut32 write every COM vtbl thunk as
  `stdmethod! { fn IErrorInfoImpl_AddRef(..) }`, so all 20 of errorinfo.rs's methods read as
  dropped; **(5)** a header `#define` renames the function between the C source and the symbol
  (gdi32's whole DirectDraw surface is `DdXxx` in the .c and `GdiEntryN` in the object file;
  `GetEnvironmentStringsA` IS `GetEnvironmentStrings`; `_tmain` IS `wmain`) — honoured only via
  an explicit `C source name <X>` provenance comment, never a fuzzy text match; **(6)** the
  `CSR_API(SrvFoo)` heuristic fired on any SHOUTY function name and took its first PARAMETER'S
  TYPE as the definition — `GDI_ROUND(FLOAT val)` reported `FLOAT` as a dropped function.
  Requiring the first argument to be a bare identifier terminated by `,` or `)` separates the
  macro from the merely-uppercase function. Also: a name provided by a still-compiled C TU is
  covered whichever language provides it — that is the varargs trampoline every flip leaves
  behind (`acpi_printf_shim.c` defines `AcpiOsPrintf`, whose home osl.c was flipped out).
- **The BACKWARD audit — did the tree ship everything we PAID for?** `fn_coverage` and
  `export_audit` both run C/spec → Rust, so neither can see paid output that never reached a
  crate: nothing references it, nothing links against it, and the money rule protecting
  `.responses/` needs an oracle it did not have. `rustconv bank-audit` runs the other way,
  `.responses` → tree, over both on-disk layouts (`manifest.json`, and the legacy
  `NN_Name.rs`). First run over 444 dirs / 7,953 banked functions: **zero unintegrated** — but
  a naive name compare flagged 8 dirs and every one was a false positive, because *the manifest
  key is a LABEL, not a symbol*. Four ways it legitimately differs from what ships, all now
  resolution rules that name themselves in the report: the key is a **macro** whose expansion
  holds the real functions (oleaut32's `RETTYP` banks 72 `VarI1From*` conversions under one
  key); the function is a **C-variadic**, which stable Rust cannot define, so it ships as a
  fixed-arity twin behind a trampoline; a **TCHAR macro** (`_tmain` IS `wmain` — same rename
  class as (5) above); or an **`#ifdef __REACTOS__` twin** where the banked name is the `#else`
  arm and never existed in our build (oleaut32's `interface_variant_size` vs the
  `interface_user_size` we compile — a reviewed allowlist entry with a written reason, since no
  mechanical rule can derive it).
  Three ways the audit itself was optimistic, each fixed and pinned by a test — the direction
  that matters, because an audit that calls abandoned work "shipped" retires the question:
  **(a)** an `extern` declaration counted as a definition, so a prelude *promising* `Foo` would
  have proved `Foo` shipped; **(b)** a containment test for variadic twins matched dhcp's
  `debug`/`error` to `debug_block_stamp` and `WDML_Initialize_error` in unrelated crates — a
  twin must be the name plus a KNOWN decoration (`_v`, `_body`, `_impl`), optionally under a
  module prefix; **(c)** a multi-function body counted as integrated when only ONE of its
  functions was in the tree. Note **our own twin convention is not uniform** — most modules emit
  `FatalError_v`, netid emits `netid_FormatMessageBox_body`.
  **`.partial` files are paid work too** — a call billed and then cut off mid-stream. They have
  no complete body, so an audit that only looks for finished ones is blind to the entire shape;
  they are scanned separately and reported as RE-TRANSLATE rather than INTEGRATE, because there
  is nothing to wire up. Adding that rule immediately found a live one (sysdm's
  `HardwareProfileProperties`, truncated — which is exactly why it was `--skip-fn`'d and owes a
  hand-write). Every other `.partial` in the bank belongs to a call that was retried
  successfully, which is the case that makes the rule look unnecessary right up until it isn't.

### A function whose body straddles a preprocessor conditional vanishes from the worklist

**Symptom.** The translation calls a function it never defined; the only signal is rustc's
"cannot find function" at a surviving call site. `scan.unextracted` is EMPTY, so nothing in the
pipeline reports the loss. aclui's `checklist.c:789 PaintControl` (277 lines) was dropped this
way, and the assembler emitted it as a commented-out stub with no finding.

**Cause.** The `#if SUPPORT_UXTHEME` block at checklist.c:947 ENDS MID-STATEMENT — the
`if (FAILED(hDrawResult))` condition is inside the conditional and its braced body follows the
`#endif`. Brace matching therefore sees a balanced function that the statement grammar does not,
and the extractor gives up on that one body while reporting nothing.

**Fix.** Resolve the module's own conditional into a scratch copy of the TU and translate from
that: the value is fixed at build time (`dll/win32/aclui/CMakeLists.txt:3` defines
`SUPPORT_UXTHEME`), so the resolved source IS the configuration that ships. The scanner then sees
37 functions instead of 36. `translate` needs the file at its REAL path — the call seal looks the
TU up in `compile_commands.json` and hard-fails otherwise (correctly) — so swap the resolved copy
in, buy, and restore from a byte-exact backup verified with `git diff --quiet`.

**Detection.** A column-0 sweep for `^name(` that is absent from the extracted set finds it with
zero false positives on this file. Repo-wide it is noisy (`WINE_DEFAULT_DEBUG_CHANNEL` and other
macro invocations sit at column 0 too), so it needs a macro-name filter before it can be a gate.

### A `.c` in the source list can compile to an object with ZERO defined symbols

**Symptom.** A file sits in the CMake source list, is 775 or 3,839 lines long, has real
function bodies and real logic, and appears in every "still to translate" census — and the
compiler throws all of it away. msvcrt's `handler4.c` and `concurrency.c` are both this: each
produces a **396-byte object with no defined symbols** on all three arches. Together they are
4,614 lines of apparent remaining work that is not work.

**Cause.** One outermost `#if` over the whole TU, decided by a value the module pins on the
compile line: `#if defined(__x86_64__) && _MSVCR_VER>=140` and `#if _MSVCR_VER >= 100`, against
`target_compile_definitions(msvcrt_static PRIVATE _MSVCR_VER=0)`. `DEAD_SOURCE` cannot see it
(the file IS in the list) and a reader cannot see it either, because the gate is 3,800 lines
away from the body you are looking at. Same shape as wldap32's `HAVE_LDAP`, but sourced from
the module's own `-D` rather than wine's `config.h`.

**Fix.** `python -m rustconv.profile_derive <module> --emit report` already reports it, at
ERROR, as `DEAD_TU` (≥90% of the TU's lines inside a decidably-FALSE `#if`), and
`never_compiled_lines` drops such bodies off the worklist so `translate` does not bill for
them. Confirm with the linker rather than the parser — `llvm-nm --defined-only` on the `.obj`
is the whole check.

Two traps around it:

- **Grep the report for `✗`, not for `ERROR`.** A `DEAD_TU` finding is invisible in a
  `grep -E 'ERROR|WARN'` of a module whose constants are named `ERROR_ACCESS_DENIED`,
  `ERROR_PATH_NOT_FOUND`, and forty more. Both findings here were correctly emitted and read
  straight past.
- **A pinned macro decides code, so derive it conservatively.** A name counts only where every
  target compiling the TU, on every arch, pins it to the same value — an msvcrt `.c` belongs to
  three targets, and `-D` inside `if(ARCH STREQUAL "i386")` is not a fact about amd64. Wrong in
  the permissive direction costs one paid call; wrong the other way silently deletes a live
  body from the translation.

### A sealed constant comes back as the enclosing `case` label

**Symptom.** `GetKeyState(VK_BACK)` where checklist.c:1972 says `GetKeyState(VK_SHIFT)` — inside
a `case VK_BACK:` arm. It compiles, links, and polls the wrong key, so shift-extend selection
reads the Backspace state. All three of the C's `GetKeyState` calls are `VK_SHIFT`; the
translation got one of them wrong.

**Cause.** The recall failure mode §9.117 describes, in its most confusing form: the model
substituted the constant it had just seen in the arm's LABEL for the one the expression uses.
The seal's invariant is a closed SET over the body, and both constants are legitimately present
in it, so nothing fires.

**Fix.** Nothing caught this but luck — Rust rejected the unrelated `i16 & 0x8000` on the same
line (C promotes the `SHORT` to `int` first; Rust needs the `as INT`). The cheap general defence
is the one §9.117 already prescribes for shared helpers: enumerate every call to the function on
both sides and diff the argument lists as a table. Here that is three rows and the odd one out is
immediate — `grep -c GetKeyState` gives 3 in the C and 3 live in the Rust, but the ARGUMENTS
differ.

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

**Variant: a macro that generates the `case` LABEL, which also puts a macro on the invoice.**
Wine writes switch arms as `ERR_CASE(ERROR_ACCESS_DENIED)` / `X(x) case (x): return #x;` — 31 TUs
here do it. A bare `IDENT(ARG)` where a case label belongs is not a statement, and the blast radius
is not the arm: on msvcrt's `errno.c` tree-sitter emitted **one `ERROR` node spanning lines
132–451**, swallowing `msvcrt_set_errno` and `_errno` whole while the walk recovered the survivors
from the wreckage. `unextracted` stayed **empty**, so the catalog looked healthy — and `translate`
planned to bill for 16 functions, missing the two that matter most and *including `ERR_CASE`,
which is a preprocessor directive*. Blanking the `#define` is again not enough (the invocations
still do not parse); `_expand_case_macros` expands them to real `case` labels before the parse.
Two traps if you touch it: recovery is **erratic, not monotonic** — two consecutive `ERR_CASE`
lose the function while one, or four, or a semicolon-terminated `ERR_MAPS` among them, recover, so
a fixture that "looks worse" can test nothing — and a name can be **redefined** with a different
body in one TU (dinput's `FE(x)` is `type = #x` then `wave = #x`), so expansion must track the
definition in force by POSITION, not keep a name→body map.

The two fixes compose, and the order matters: expanding is what lets `--seal-facts` see the arms
at all. Sealed, `msvcrt_set_errno`'s 60 constants go to the model as `__RUSTCONV_FACT_00NN__` and
cannot be invented — which is the whole defence, given `EDEADLK` is 36 in msvcrt and 35 in glibc.

### The export audit that was supposed to catch the shrunken plan was itself blind

**Symptom.** A module translates, `cargo check`s clean on all three arches, passes `check`, and
is **missing 22 of its exports**. `export_audit.py` — the tool whose whole job is "does the Rust
actually define what the `.spec` exports?" — reported 15 of them and called the other 7
*forwarded, no implementation needed*. Nothing else can see it: the crate is not flipped, so no
linker ever reads it, and a function that was never translated looks exactly like a function that
was never there.

**Cause — two independent holes, both in reading the `.spec`.**

1. **A bare trailing target is a LOCAL RENAME, not a forward.** Only a dot makes it a forward, and
   that is literally how spec2def decides (`fIsExternal = !!ScanToken(strTarget, '.')`,
   spec2def.c:691). `kernel32.FileTimeNow` means another DLL supplies the symbol; a bare
   `WLDAP32_ber_alloc_t` means *this* module must define it under that name. Treating both as
   forwards skipped every Wine-style renamed export — and ntoskrnl's
   `@ fastcall ExAcquireRundownProtection(ptr) ExfAcquireRundownProtection` rows are the same shape,
   so the kernel's audit had the same blind spot.
2. **`-version=` rows are not in the `.def` at all.** We spec2def at `DLL_EXPORT_VERSION 0x502`
   (`sdk/cmake/config.cmake:87`) and spec2def emits an entry only if that falls inside the range
   (spec2def.c:1155). Auditing them the other way invented 23 phantom MISSING exports on ws2_32
   alone — the entire `Wah*` family — in a module that is fully flipped and links. **Phantom
   findings are not harmless**: a tool that cries wolf on a shipping module is a tool you stop
   believing, which is why 22 real wldap32 misses sat next to them unnoticed.

Fixing both: 1388→1326 on ntdll, 72→2 on kernel32, 23→0 on ws2_32 — and the wldap32 gap became
visible. `SpecEntry` now carries `impl_symbol` and `in_version`, and `symbol` is what the linker
will actually look for.

**Fix.** Run `python -m rustconv.export_audit <module>.spec <crate>/src` **before** the cohab
flip, not after — pre-flip is the only window where nothing else can catch it. Once a module is
flipped, the link is a perfect oracle and the audit is redundant; while it is unflipped, the audit
is the *only* oracle.

**Corollary, and it bounded the blast radius to one module.** A FLIPPED crate is covered by two
oracles working together: `cargo check` proves every *static* the surviving code calls, and the
link proves every *export*. A dropped function is therefore only invisible in a crate that is
translated and **not yet flipped**. Sweeping all 174 `*-rust` crates after this fix: 100 are
flipped and thus already proven, and of the unflipped ones the only module this lane had
translated was netcfgx — audited at **0 real misses** across its 6 translated TUs. wldap32 was
exposed for exactly one reason: it was the single module sitting translated-but-unflipped while a
scanner fix landed underneath it. **The gap between translating a module and flipping it is where
silent drops live — so close it quickly, and audit before you do.** (Beware a crude sweep here:
`modules/rostests/winetests/<mod>/` shares the module's directory name and will report hundreds of
test-harness functions as missing, and an unparsed macro position surfaces as a "function" named
`NTAPI`/`Contxt`/`if`. Both are noise, and both drown the real finding.) The general shape: **an audit's exclusion list is where its blindness
lives.** Every "this one needs no implementation" branch is a place a real miss can hide, so each
one has to be justified against what the build tool actually does, not against what the syntax
looks like.

**The upstream cause of the missing 22 was `TYPE * CDECL fn()`** — the scanner position
tree-sitter cannot parse in any spelling, fixed in `0b2d3521e0a` after wldap32 was translated
against the older scanner. Every one of the 22 returns a pointer; not one non-pointer function was
lost. **When a drop set shares a signature feature, the parser is the suspect, not the model.**

The same hole reopens for **every convention keyword missing from `conventions.KW_TO_CONV`**, which
is the table `scanner.RE_CC_AFTER_PTR` is built from. `__thiscall` was absent, so msvcrt's `cpp.c`
came back as 37 of 62 bodies — 25 dropped, silently. The tell was again the return type:
`void __thiscall bad_typeid_dtor(...)` parsed while `bad_typeid * __thiscall bad_typeid_ctor(...)`
directly beside it did not, so the plan looked *partly* translatable rather than broken, which is
much easier to accept than a zero. **Before paying for a file, diff the plan's function count
against `llvm-nm` on the built object** — the plan is the only place this failure is visible, and
it costs one command. Fixed by adding `__thiscall`/`_thiscall` to the table; the only files in the
build that reach the shape are `cpp.c`, dead `concurrency.c`, `vssapi/main.c` and
`sdk/lib/crt/wine/cpp.c`, none of which had been translated, so nothing shipped with it.

### `thiscall` is not a spelling of `stdcall`, and collapsing it corrupts `this` on i386 only
**Symptom.** A translated C++ method family links clean on all three arches, exports the right
mangled names, and reads garbage for every member of `*this` — on i386, and nowhere else.

**Cause.** rustconv mapped `thiscall` to Rust's `"system"` (i.e. stdcall) in three places:
`postprocess._SPEC_EXPORT_ABI`, `_import_abi_map`, and — the one that actually won, because it runs
last — `abi_enforce._canonical_block`, where `thiscall` simply fell through to the `extern "C"`
default. **stdcall passes `this` as the first stack argument; thiscall passes it in ECX.** Every
downstream check still agrees: same export name, same arity, same callee-cleanup. Only the register
differs.

**Fix.** Treat it exactly like `fastcall`: a per-arch cfg dual, `extern "thiscall"` on x86 and
`extern "C"` elsewhere. Two facts make that the only option — `extern "thiscall"` is stable as of
rustc 1.92 and lowers identically to the C (`this` in ECX, `retl $N`, undecorated symbol), and it
is **x86-only**, so naming it on amd64 is a hard `E0570`. The `.spec` already says all of this:
msvcrt carries `thiscall -arch=i386` beside `cdecl -arch=win64` and `cdecl -arch=arm` for one
export. Match spec entries by **`impl_symbol`, not by export name** — C++ methods are exported under
mangled names (`??0bad_cast@@QAE@ABQBD@Z`) while the Rust fn is `bad_cast_ctor`, so a key-based
lookup finds nothing and the whole pass no-ops.

### A C++ `struct` used as a NAMESPACE falls between both C++ modes, and one skips it silently
**Symptom.** `rustconv translate --free-functions <tu>.cpp` reports its plan cheerfully and buys
every free function, having printed one line you will scroll past:

    [free-functions] skipping 4 class method(s) — these need --cpp-com --class: X_A, X_B, X_C, X_D

The assembled module then calls `X().B(...)` — C++ syntax, no such Rust — so it does not compile,
which is the lucky outcome. The unlucky one is a TU where the skipped methods happen not to be
referenced from the free functions.

**Cause.** shell32's `item_prop.cpp` declares `struct ShellPropSheetDialog` whose members are all
`static`: it is a NAMESPACE, not a class. No vtable, no state, no `BEGIN_COM_MAP`, no
`OBJECT_ENTRY`. `--cpp-com` refuses it correctly on the deterministic preflight —

    no BEGIN_COM_MAP entries found / aggregation policy is unsupported / no OBJECT_ENTRY activation

— and `--free-functions` classifies the same members as "class methods" and drops them. Neither
mode owns the shape, and only one of the two says so loudly. The word `struct` in the source is
what misleads: it does **not** imply the COM path.

**Fix.** Buy the free functions, hand-write the statics as free functions, and name them the way
rustconv's skip line already named them (`Struct_Method`) so the two halves line up. Do the same
audit before scoping any C++ TU: `grep -c 'BEGIN_COM_MAP'` is the real discriminator, not `struct`.

**`--local-raii` DOES reach `--free-functions`**, despite its `--help` saying "shell32 COM mode".
Confirm with `--dry-run --prompt --fn <name>`: the lowering appears in the prompt followed by a
mandatory usage mapping. That is what keeps a file-local RAII guard's drop semantics out of the
model's hands — worth checking, because the guards are exactly what a C++ translation loses.

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
- **A translated `extern fn` that names a C MACRO becomes a bugcheck stub.** The declaration is
  an unchecked *claim* that a symbol exists: no symbol does, so the stub generator "resolves" it
  into `KeBugCheckEx`. Rust compiles, the link succeeds, and the crash waits for the first caller.
  `FatGenerateFileIdFromFcb` / `FatGenerateFileIdFromDirentAndOffset` (`fatprocs.h:3067`/`3078`)
  and `SizeOfPackedEa` (`fat.h:738`) all shipped this way; the first fires on any
  `FileInternalInformation` query. **Grep the headers for `#define <name>(` before trusting an
  `extern` declaration** — and note this is the *inverse* of `[§9.53]` above: there a macro was
  stubbed to a silent no-op, here a macro is stubbed to a loud crash. Same root, opposite failure.
- **The reachable-stub set is computable — stop finding it one boot at a time.** Intersect the
  names defined with a `STUB_BUGCHECK` body in `rust_cohab_stub.c` with the names any `*-rust`
  module declares `fn <name>(`. That intersection is *unimplemented AND reachable*. fastfat: 109
  stub-file functions, 19 bugcheck stubs, **all 19 reachable**, mostly FSCTL handlers — exactly
  what a filesystem conformance test drives, so they surface one per boot as the test gets
  further. Expect the queue to alternate between macros (implement inline) and genuinely
  untranslated functions (`FatOpenVolume`, `create.c:2292`, ~290 lines — translate it).
- **Only arm64 exercises fastfat.** arm64 boots a FAT32 ramdisk, so fastfat is its live system
  volume; i386/amd64 boot a CDFS ISO and never enter it. An "arm64-only" filesystem crash is
  usually *coverage*, not an arch bug — and conversely a fastfat stub is invisible on two of
  three arches.
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

### The failure path survived; the sentence explaining it did not

**Symptom.** Something does not happen and NOTHING says why. The key is not there, the handle is
not created, the API returns a clean error code — and the serial log is empty, so the hunt starts
from zero. `CmpLinkKeyToHive` has three failure returns and upstream prints on each; all three
were dropped, so when `\REGISTRY\USER\S-1-5-18` did not appear there was no signal anywhere.
Restoring the prints, then getting SILENCE back from them, is what proved the create half was
fine and moved the hunt to the reparse — two build+boot cycles for a fact one grep would have
given free.

**Cause.** Translation keeps the code and drops the diagnostics. Measured across `sdk/lib/rust`:
**2891 commented-out `DPRINT1` sites against 304 live prints** — roughly 90% of upstream's
always-on kernel diagnostics are gone. No existing lint saw it, because `scan_silent_failures`
matches SHAPES (`loop {}`, `.unwrap()`, `Err(_) => {}`) and a branch that returns a correct error
code without printing is not a shape, it is an ABSENCE. Absence needs an oracle, and the oracle
was already in the file: rustconv keeps the original C as trailing comments.

**Fix.** `tools/scan_dropped_prints.py` pairs each commented-C function against the live Rust
function of the same name and reports one that prints in the C and never in the Rust. Baselined
(`tools/lint-baseline/scan_dropped_prints.txt`, 1425 rows) and gated by `vibeos-cli check`; burn
down by subsystem. Restoring a print is cheap, mechanical, and strictly increases what the OS can
tell you.

Four things that cost precision, all of them found by being wrong first:

- **Only count what upstream ACTUALLY emits.** DPRINT1, DbgPrint*, and wine ERR/FIXME are on by
  default; DPRINT (3579 sites), TRACE (3442) and WARN (247) are compiled out or off, so dropping
  them loses nothing. Scoring those would have tripled a harvest with nothing behind it.
- **Pair through the wrapper/`_impl` split.** 910 `_body` + 580 `_impl` functions tree-wide, plus
  a CamelCase `Impl`. The prints live in the body, so exact-name pairing calls every wrapper
  silent and is wrong every time.
- **Formatting into a buffer is not printing.** `rtl_string_cb_printf_w!` matched a "contains
  print" heuristic and scored as live, hiding three genuinely dropped diagnostics in
  `CmpCreateControlSet`.
- **A print-shaped thing that prints nothing is worse than none**, because it reads as covered.
  `cmlib-rust/cmheal.rs` defined `unsafe fn DPRINT1(_format) {}` reasoning that it matched "the C
  NDEBUG build" — but NDEBUG silences DPRINT, never DPRINT1. `rtl-strings`' `path_wave` module
  defined a no-op `DbgPrint` that SHADOWED the real one for the whole module, so four
  "RtlGetFullPathName_Ustr() returned 0" reports on the path that answers
  STATUS_OBJECT_NAME_INVALID went nowhere. Same shape in win32k (`ERR`, `FIXME`) and user32
  (`ERR`). All are the thing AGENTS.md forbids outright — see `[§9.65]`: an out-of-line no-op is
  also an optimization barrier that uncloaks latent ABI mismatches into a `ud2`. The scanner
  reports them as `NOOP` and treats their callers as silent. Note the C-side equivalent is
  invisible from a Rust scanner and must be grepped separately —
  `dll/win32/{cabinet,mlang}/rust_cohab_stub.c` define no-op `ERR`/`FIXME`.
- **Both spellings of a channel must score, or the lint cannot be satisfied.** `ERR` was in
  `ALWAYS_ON` (so an `ERR(...)` in the C counted) but only the bare-call form was recognised on
  the Rust side — never the `ERR!(...)` macro that every wine-derived crate actually uses. Its
  sibling `FIXME` was covered in both forms. So a function whose only diagnostic is ERR stayed
  flagged no matter how faithfully it was restored, which is noise, not silence. Fixed by adding
  `err` to `LIVE_PRINT_NAMES` (+3/-75 on the baseline; the 3 are `[ALL]`→`[PARTIAL]`
  reclassifications, not new debt). Corollary when reading a finding on a **usermode** crate:
  shell32-rust's `TRACE!`/`ERR!`/`WARN!`/`FIXME!` are all no-op `macro_rules!` in `lib.rs`, so the
  call sites score as live while emitting nothing. That is not the `[§9.65]` trap (they are
  macros, not out-of-line fns) but the diagnostics are still dark, and the scanner cannot see a
  cross-file no-op macro the way it sees a file-local no-op `fn`. Giving that crate a real wine
  debug sink is owed.
- **A WHOLE-TU flip is invisible to this scanner, so audit those by hand.** It pairs a
  *commented-out C function* against the live Rust of the same name — which is the per-function
  cohab shape. Remove a `.cpp` from a CMake source list instead and the C source stays on disk
  uncommented, so nothing pairs and the finding count does not move. `folders/CFSFolder.cpp` had
  **18 always-on `ERR`/`FIXME` sites and its three Rust modules had 3**; the scanner reported
  nothing. The cheap manual check when you flip a TU is
  `grep -cE '\b(ERR|FIXME)\(' <tu>.cpp` against `grep -cE '\b(ERR|FIXME)!\(' <modules>.rs`,
  remembering to discount any site inside `#if 0`. The tell that a diagnostic was the *whole*
  body of a branch is an **empty `if`**: CFSFolder's `_AddFindResult` translated
  `if (SUCCEEDED(hr)) { ERR("Got CLSID override for '%S'\n", Tmp); }` into `if hr >= 0 { }` — a
  branch that computes a condition and does nothing, which is what a lost print looks like from
  the outside.
  **Know WHY the count is near zero before you call it a regression:** for shell32, rustconv
  runs `strip_debug_statements` over the model's view of every body
  (`__main__.py`, `model_bodies = {... strip_debug_statements(dep.body_text) ...}`), so the LLM
  never sees a `TRACE`/`ERR`/`WARN`/`FIXME` and a freshly translated module starts at ~zero by
  construction. Measured on three TUs flipped 2026-08-04: `folders/CNetFolder.cpp` had 10
  always-on sites and its Rust modules 0; `CDefViewDual.cpp` had 11 and 0. The
  already-landed-long-ago `net_folder.rs` is also 0, which is the tell that this is systemic
  rather than per-flip. So "restore the print" here means **re-deriving it from the C**, not
  recovering something a model dropped, and doing it for three TUs while ~40 sibling modules
  stay bare buys nothing. It is a subsystem burn-down, and it is gated behind giving
  shell32-rust a real wine debug sink first — until then `ERR!`/`FIXME!` are no-op
  `macro_rules!` and a restored call site emits nothing anyway.
- **The mirror image: a PARTIAL-slice assembly makes every C-vs-Rust pairing lint over-report.**
  Same oracle, opposite failure. rustconv embeds the **whole** TU's C source as trailing comments
  no matter how narrow the run was, so a `--free-functions` or `--cpp-com --class X` assembly
  carries C whose Rust lives in a *sibling* module. `scan_mask_drift` then reports those as
  DROPPED even though the flag-clear is present and correct next door — three such rows appeared
  the moment `folders/CControlPanelFolder.cpp` and `folders/CDesktopFolder.cpp` were assembled
  (`rgfInOut &= ~SFGAO_VALIDATE`, `grfMode &= ~STGM_CREATE`, `pcsFlags &= ~SHCOLSTATE_ONBYDEFAULT`),
  and all three are live in `control_panel_folder.rs:1067` and `desktop_folder.rs:1621`. Check the
  sibling module before believing a DROPPED row on a file whose first line is
  `// rustconv-assembly: free-functions` or `class:...`. A whole-TU assembly has no such gap,
  because its embedded C and its Rust cover the same set.
- **Two TUs' anonymous enums can collide in `scan_const_contradictions`.** `IDC_PROPERTIES` is
  `2` in `CRecycleBin.cpp`'s `enum { IDC_EMPTYRECYCLEBIN = 1, IDC_PROPERTIES }` and `0` in
  `CNetFolder.cpp`'s `enum { IDC_PROPERTIES }`. Both translations are faithful; the name is simply
  not crate-unique. Prefix the newer one and keep the C spelling in a comment — do not "reconcile"
  the values, which would corrupt one of the two menus.

**And the print is sometimes missing because the BRANCH is.** `CmpSetValueKeyNew` lost its whole
`_SEH2_EXCEPT` around `CmpCopyName`, whose C comment says exactly why it exists ("this can crash
since the name is coming from user-mode"). Do not close such a finding by adding a bare print —
that clears the lint row and leaves the defect. It is the same relationship as `[§9.53]`: what
looks like a missing diagnostic is a missing handler.

### A `#if` on a WINDOWS VERSION comes back as `#[cfg(target_arch)]`

**Symptom.** Either a hard parse error (`attributes on expressions are experimental`, because
the `#if` sat between two ARGUMENTS of a call) or, worse, a clean build in which one
architecture takes a branch the C never compiles anywhere.

**Cause.** `#if _WIN32_WINNT >= 0x502` and `#if (WINVER >= 0x0600)` are assertions about the
TARGET WINDOWS VERSION, which is a `-D` flag identical on all three of our arches. To a
translator they look like the per-arch `#if defined(_M_AMD64)` it sees constantly, and
`target_arch`/`target_pointer_width` is the nearest Rust spelling of "a compile-time branch".
desk.cpl had two, independently, in different files — so this is a class:

  * `classinst.c`'s `#if _WIN32_WINNT >= 0x502 KEY_READ|KEY_WRITE #else KEY_ALL_ACCESS` became
    `#[cfg(target_pointer_width = "64")]` / `#[cfg(...= "32")]` on the two argument
    EXPRESSIONS. Had it parsed, i386 would have demanded ALL_ACCESS on a key it only reads.
  * `theme.c`'s `#if (WINVER >= 0x0600)` block became
    `#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]`, and its body reads
    `NONCLIENTMETRICSW::iPaddedBorderWidth` — a field that does not exist at the version the
    module actually compiles at, which is precisely why the C does not compile that block.

**Fix.** Resolve the version test at the module's real `-D` setting and keep only the live arm,
with a comment naming the directive and the value, so the dead arm's absence is a decision on
the page rather than a hole. Check the module's `VER_DEFS` in `gen_driver_types.sh` (or its
CMake `set_module_type`) for what that value is — do not assume the newest.

**Detection.** Grep the translated output for `cfg(target_arch` / `cfg(target_pointer_width)`
and check each against the C: a per-arch cfg is *legitimate* only where the C branched on
`_M_IX86`/`_M_AMD64`/`_M_ARM64` or on a pointer-width-derived macro. Anything else is this bug.

### A build-configuration `#ifdef` must be carried into the Rust, not resolved away

**Symptom.** A Wine-derived module translates cleanly and then cannot compile at all: hundreds
of `cannot find function` for an optional backend's API. wldap32 hit 1087 rustc errors, 778 of
them `ldap_*`.

**Cause.** `config.h` writes an off feature as `/* #undef HAVE_LDAP */` — an assertion about
*this* build. The C therefore compiles `return WLDAP32_LDAP_NOT_SUPPORTED` for every entry
point and never links a libldap. The translator renders the **whole** C, guarded arm included
(correctly — that arm is what makes the Rust worth more than the C), so the output references
a library that is not there. Two follow-on shapes, both silent: a `switch` whose labels all
come from the absent header becomes a catch-all (see the CATCH-ALL entry), and a `static
inline` helper block that the header gates gets emitted ungated.

**Fix.** Do what the C does — keep both arms, let a cfg feature pick. `python -m
rustconv.cfg_gate --c-dir <dir> --rust-dir <dir>` derives the boundary from the C and applies
it; `--apply` writes. It emits a `#[cfg(not(feature=...))]` twin carrying a **copy of the
attributes** (`#[no_mangle]` must survive with the feature off) whose body is the initializer
*lifted from the translated Rust*, never re-derived from the C constant. **Never delete the
guarded arm** — it is paid artifact, and it is the half that a future `HAVE_X=1` build needs.

Three things that bite:
- **Guards are plural.** wldap32 branches on EIGHT (`HAVE_LDAP`, `HAVE_LDAP_H`, and six
  per-entry-point probes like `HAVE_LDAP_PARSE_SORT_CONTROL`), each `#undef`'d independently.
  Gate on the obvious one alone and the other seven stay silently wrong. `cfg_gate` derives the
  set by intersecting the macros the module tests with config.h's `#undef` set; `#elif
  defined(X)` arms are NOT in that scan and need declaring by hand.
- **`#else` means the other arm is the one that ships.** Gating the translated arm there
  installs the wrong half. `cfg_gate` refuses these rather than guess.
- **One spelling only.** wldap32 carried four in one crate — `feature = "HAVE_LDAP"`,
  `feature = "have_ldap"`, bare `cfg(HAVE_LDAP)`, bare `cfg(have_ldap)`. All four are false
  today, so nothing looked wrong; flipping the feature either way would have taken half the
  crate the wrong direction. A bare `cfg(X)` is a custom cfg flag, *not* a feature.

Declare every feature in `Cargo.toml`, and prefer the semantic alias over a bindgen width alias
in FFI signatures (`heap_alloc(len: usize)`, not `SIZE_T`) — `crate-repair` will otherwise have
to bake `as u32` on i386 and `as u64` on amd64/arm64 for the same expression, and rightly
refuses.

**Variant: the condition was INVENTED — a `#if` mapped onto an unrelated `cfg`, so the DEAD
arm ships on one arch.** Worse than resolving one away, because the output looks properly
conditional and one arch keeps booting. msvcrt's `msvcrt_init_heap` is guarded by
`#if _MSVCR_VER <= 100 && !defined(__REACTOS__)`; here `_MSVCR_VER` is 0 but `__REACTOS__` IS
defined, so the condition is FALSE and the `#else` — `heap = NtCurrentTeb()->Peb->ProcessHeap`
— is what compiles on all three arches. The translation emitted
`#[cfg(not(target_pointer_width = "64"))] heap = HeapCreate(0,0,0)` against a 64-bit `#else`,
a condition with no relationship to either term. i386 then ran the **dead** `HeapCreate` arm
and **booted green** — a private heap is still a valid heap, which is exactly what hid it —
while amd64 and arm64 got the PEB pointer as their heap handle, so every `RtlAllocateHeap`
failed, `malloc` returned NULL forever, and user-mode starved: 15 threads parked >20s on
`WrFreePage`/`WrQueue`/`WrLpcReceive` and no desktop marker. `msvcrt_destroy_heap` carried the
same invented condition, which on i386 would have called `HeapDestroy` on the process heap.

**Fix, and it is free: cross-check the two LISTS.** `grep -n '#\[cfg(' out.rs` against
`grep -nE '^\s*#\s*(if|ifdef|ifndef|else|elif|endif)' in.c` and require every `cfg` to
correspond to a real `#if`. On heap.c that left exactly two legitimate arch-conditionals (a
`NtCurrentTeb` asm shim and a genuine `#ifdef _WIN64`) and both invented ones stood out
immediately. **Do this before the gate, and never trust one green arch**: the tell here was
that i386 passed while both 64-bit arches wedged, and an all-64-bit lane would have read as a
uniform failure with no asymmetry to point at the arm selection.

The same sweep on the very next file (`cpp.c`) caught two more, so run it every time — this is
not a one-off. `get_obj_locator_base`'s `#ifdef RTTI_USE_RVA` came back as
`#[cfg(all(target_pointer_width = "32", feature = "rtti_rva"))]`: **inverted** (cxx.h defines
`RTTI_USE_RVA` for everything EXCEPT `__i386__`, so 32-bit is precisely where the arm is dead)
**and** gated on a cargo feature that does not exist. Both terms false on all three arches, so
no arch took the arm, and the RTTI walk on 64-bit would have resolved every type against image
base 0. Note what a nonexistent feature does here: `#[cfg(feature = "…")]` for an undeclared
feature is not an error, it is simply never satisfied — so an invented feature name is a
silent, permanent `#[cfg(false)]`.

The second was subtler and worth internalising: `msvcrt_init_exception` guarded the *same*
`#ifdef RTTI_USE_RVA` with `#[cfg(target_pointer_width = "64")]`, which selects exactly the
right arches — today. It is still wrong, because the C branches on `__i386__` and the two
coincide only for the three targets we happen to build; a 32-bit non-x86 target would compile
every RTTI blob in RVA form and then never patch it. **A `cfg` that is accidentally right is a
defect, not a pass.** Key the condition to the term the C actually tests
(`not(target_arch = "x86")`), and the cross-check above becomes a real equivalence rather than
a coincidence you re-derive per arch.

**Variant: the resolved-away arm was a BUILD FLAVOUR, and the flavour later changed.** Same
class, but it compiles and boots, so nothing points at it for months. `hal/halx86/generic/
spinlock.c` opens with `/* This file is compiled twice. Once for UP and once for MP */`; the
Rust took the UP arm of `KxAcquireSpinLock`/`KxReleaseSpinLock` (a bare compiler fence) as
unconditional, and wrote a module docstring saying so — *"We're a UP-only build"*. True when
written. Then the halmp.dll twin landed and one crate started building both flavours, one
cargo feature apart, with neither funnel looking at the feature. **A docstring recording an
assumption is not a safeguard against the assumption changing — it is the bug's alibi.** Two
tells worth grepping for: a C comment that the file is compiled more than once, and a
translated file that names a build flavour in prose instead of in a `cfg`.

The damage was doubled by resolving away a *second* axis in the same expression:
`KxReleaseSpinLock` clears the lock under `CONFIG_SMP || KI_TRACK_SPINLOCK_OWNER`, and
`KI_TRACK_SPINLOCK_OWNER` is plain `#if DBG` — so the C clears on every checked build, UP
included. Dropping both arms left the *kernel* half stamping owners that the *HAL* half never
cleared. On MP that paired a real interlocked `KeTryToAcquireSpinLockAtDpcLevel` with a
release that did nothing: the i386 PFN queued lock could be try-acquired exactly once per
boot and failed forever after. Beyond the test row, halmp.dll's whole i386 `Ke*SpinLock*`
export surface — the HAL owns it there, per `hal.spec` `-arch=i386,arm,arm64` — offered no
mutual exclusion at all beyond the IRQL raise.

**Fix.** Delete the copy rather than repair it: both HAL funnels now call
`ntkrnl_sys::inline::ke::Kx{Acquire,Release}SpinLock`, the same code ntoskrnl-rust uses, with
`hal-rust/smp` forwarding to `ntkrnl-sys/smp`. `spinlock.h` had already written the rule down
— *"the two MUST agree, or a lock stamped by one side is never cleared by the other"* — which
a second implementation cannot enforce and a shared one cannot violate. i386 kmtest
`KeSpinLock` went 710/75 → 753/32 on UP and 706/79 → 754/31 on MP.

Two diagnostics that generalise. **Compare the same row across arches before theorising**: i386
sat at 710/75 where amd64 and arm64 were 321/2, and amd64 is exactly the arch that gets these
from `ntoskrnl.spec` instead of the HAL — the outlier named the layer. And **a ratchet compares
numbers, not reasons**: the pre-fix UP and MP runs both banked 306 IoInterrupt failures, so the
row looked stable, but UP was failing `ok_eq_bool(*Lock != 0, TRUE)` and MP was failing
`ok_eq_ulongptr(*Lock, 1)` — identical counts, different causes. Diff the failure TEXT, not the
tallies.

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
- **`delete this` gets a deallocator the model INVENTED, because the allocator is invisible to
  it.** shellrecyclebin has five textually identical `delete this;` — one per hand-rolled COM
  class `Release()` — and the C++ COM path lowered them five different ways, none correct:
  `dtor_body(this); free(this)`, `drop(Box::from_raw(this))`, `let _ = Box::from_raw(this)`,
  `let _ = SENDTO_ITEM_delete(this)` (a hallucinated call to an unrelated shell32 function),
  and `__rustconv_delete(this)` (an invented placeholder). The right target was generated by
  the scaffold in the same file: `X_delete`, which runs `X_dtor_body` then
  `HeapFree(GetProcessHeap(), 0, this)` to pair with `X_new`'s `HeapAlloc(GetProcessHeap(),
  HEAP_ZERO_MEMORY, …)`. Same root as the vptr store: **the scaffold owns allocation, so the
  model is being asked to invent the matching free for a strategy that appears in no source it
  reads** — C++ puts `operator new` and the vtable store in the constructor prologue, which is
  not in the body it was given. Do not read "it failed to compile" as a guard: all five failed
  only because `Box` is undeclared in a `no_std` crate without `alloc` and the other three
  names resolve to nothing. Wherever those names ARE in scope the file compiles clean and
  frees CRT-heap-style memory that came from `GetProcessHeap()` — corruption charged to
  whoever allocates next, arbitrarily far from the Release that caused it. shell32-rust
  already carries such a shim (`new_menu.rs`, `#[link_name = "free"] fn cpp_com_free`). Lower
  `delete <expr>` in a class method deterministically to `<Class>_delete(<expr>)`, and audit
  worker bodies for `free`/`Box::from_raw`/`__rustconv_*` so the next miss is loud instead of
  luck.
- **A comparison whose two sides were DIFFERENT index expressions, collapsed to the same one.**
  msvcrt's picture formatter counts the run of the current field with `while(format[0] ==
  format[count]) count++;` — `MM` is one field of width 2, `dddd` is the long weekday. The
  translation emitted `while *format_ptr.offset(count) == *format_ptr.offset(count)`, i.e. an
  element compared to ITSELF: unconditionally true, so `count` ran away until the read left the
  mapping. It reads as a plausible line and `cargo check` has nothing to say about it. The tell
  is that the two operands of a C comparison are rarely the same expression — **grep a fresh
  translation for `X == X`, `X != X` and `X < X` over any subscript or field path** (an
  `x != x` NaN test is the one real exception, and it is on a float). Reached only through
  `%c`/`%x`/`%X`, whose data lives in `locinfo->lc_time_curr` — a slot the harness left NULL
  until time.c became a reference TU, so the first row that ever ran the line is the one that
  found it.
- **A SIDE EFFECT inside a call argument, hoisted out of the condition it lived in.** Rust has
  no `++`, so `f(*p++)` has to become two statements — and WHERE the second one lands is the
  whole meaning. msvcrt's demangler reads
  `if (!get_calling_convention(*sym->current++, …) || !demangle_datatype(sym, …))`: the `++`
  runs before `demangle_datatype` sees the same `sym`. The translation put the increment after
  the entire `if`, so every function signature parsed its RETURN TYPE from the
  calling-convention letter — `A` reads as a reference, and `void * (__cdecl*)(long)` demangled
  as ` & (__cdecl*)(void,long)`, with the real return type shifted into the argument list.
  **Both forms advance the cursor exactly once**, so nothing leaks, nothing faults, and no
  bounds or lifetime check can fire; only the ORDER differs. It survived a build, a boot and
  every scanner we own, and was found by the first differential row that reached it. When a
  translated function takes a cursor or an index, diff the SEQUENCE POINTS, not just the
  arithmetic: for every `x++` / `++x` the C evaluates inside an argument, a condition, or a
  comma expression, ask which later operand in that same expression observes the new value.
- **A BOUNDED API substituted by an UNBOUNDED one — the buffer's size argument is dropped, and
  the substitute has its own, larger, internal bound.** msgina rendered strsafe.h's
  `StringCchPrintfW`/`StringCbPrintfW` as `macro_rules!` wrappers that discarded the count and
  forwarded to user32's `wsprintfW` — with a comment saying so, on the reasoning that
  `wsprintfW` bounds to 1024 wchars internally "matching the small fixed buffers these are
  called with". The buffers are `WCHAR[256]`, so the substitution turned a truncating API into
  one that writes 1.5 KB past a stack array whenever the interpolated value (a user name, from
  the logon dialog) is long. Nothing detects this: it compiles, it links, the common case fits,
  and the C's `STRSAFE_E_INSUFFICIENT_BUFFER` return was already being ignored. **A substitute
  whose bound is not the CALLER'S bound is not a substitute** — if the real body cannot be
  written, hard-fail rather than approximate, because a bound is the entire contract of the
  function that carries one. Symptom to grep for: a wrapper that takes a size parameter and
  never mentions it again (`let _ = $cch;`). **Swept the tree after fixing it and found no
  second instance** — shell32-rust has eleven copies of a per-file `%s`/`%c` mini-printf, and
  every one of them honours its `capacity` and returns `STRSAFE_E_INSUFFICIENT_BUFFER`. Writing
  a narrow formatter is fine; dropping the bound is not, and they are independent decisions.
- **A hand-approximated port of a header the tree already ports elsewhere.** Same module, same
  root cause: `static __inline` helpers have no object file anywhere, so each flipped module
  ports them again, and the Nth copy is written from what the module happens to call rather than
  from the header. msgina's `StringCbCopyNExW` dropped the fill paths, the `IGNORE_NULLS`
  handling, and computed `pcbRemaining` differently from the worker; syssetup's copy was a
  strict subset; desk's was faithful. Four copies, no two alike. The fix for the CLASS is one
  crate (`sdk/lib/rust/strsafe-rust`) on core primitives so it needs no island — `WCHAR`/
  `HRESULT`/`DWORD` are transparent aliases for `u16`/`i32`/`u32` in every island in the tree
  and Rust unifies aliases structurally. **Two copies of a ported header is the detection
  event**, not four.
- **The model FIXES a real bug in the C, and that is still a defect.** `CDefaultContextMenu::
  DoCreateNewFolder` frees `pidl` exactly once, on the success path, and leaks it when
  `SelectItem` fails. Every sample added a second `SHFree` on the failure path — correct as
  engineering, wrong as translation, and caught only because the call envelope's "exactly once"
  rule saw a duplicated token. Worth stating because the rejection reads like model sloppiness
  and is the opposite: the envelope is doing precisely its job. Two practical consequences —
  **retrying does not help** (the improvement is stable across samples, so `--retry-rejects` is
  money for nothing; hand-write it with a comment saying the leak is deliberate), and a
  translation that is *silently* more correct than its C is a divergence no lint can find later,
  because nothing is broken. Same family as recall-permutation: the model's prior beats the text
  in front of it, and here the prior is "this code should not leak."
- **A dropped `!` on a predicate, with the second operand garbled into the wreckage.**
  `CFSDropTarget::_QueryDrop` reads
  `if (!PathIsSameDrive(wstrFirstFile, m_sPathTarget) && m_dwDefaultEffect != D_LINK)`;
  the reply came back as
  `if PathIsSameDrive_body(..) && (*this) == FALSE.m_dwDefaultEffect != D_LINK`. The negation is
  gone, so the cross-drive rule INVERTS — a drag to a different drive stops defaulting to copy —
  and the tell is only the second half, which is not a condition at all. The lesson is the
  ordering: the garbled operand is what rustc reports (`comparison operators cannot be chained`),
  so it reads like a syntax slip; the semantic flip sits on the SAME line and passes unremarked
  once you fix the syntax. When a condition fails to parse, re-read the whole condition against
  the C rather than repairing the fragment rustc pointed at.
- **`SUCCEEDED(hr)` → `(hr as u32) >= 0`** — unconditionally true, so the FAILURE branch is
  unreachable. `SUCCEEDED` is `hr >= 0` on the SIGNED HRESULT; cast to unsigned first and every
  error becomes a success. Sibling of the `!`-on-an-integer bug below and just as silent: it
  feeds a condition, so there is no type error to catch it. `shlexec`'s verb probe took the
  `runas` path whenever `AssocQueryStringW` *failed*. Grep any HRESULT compared `>= 0` or `< 0`
  for an intervening `as u32` / `as DWORD`.
  The macro also comes back **expanded twice** — `if ((hr) >= 0) >= 0` in
  CDefViewBckgrndMenu's QueryContextMenu, where the scaffold's expansion and the model's own
  stacked. Rust catches that one (`expected bool, found integer`) only because it has no
  bool-to-int coercion; in C++ it would have compared `true` against 0 and inverted the branch
  that decides whether the folder's own menu items are counted. So it is a *type* error here and
  a *silent* one anywhere else — worth grepping `>= 0) >= 0` and `< 0) < 0` after any run.
- **`goto L` lowered to a block that jumps to ITSELF** — `'L: { break 'L; }` placed at the goto
  SITE rather than spanning what the goto skips. It compiles, it is a no-op, and execution falls
  straight through into the code the `goto` existed to bypass. In `SHELL_ExecuteW` the
  runas-cancelled path set `retval = 33` and then fell into the `else` arm that overwrites
  `retval` with `GetLastError()`. The tell is an `unused label` warning on the REAL label at the
  function tail — the jump never reaches it. Related but distinct from the replication shape in
  "A C lexer was masking Rust output" above: replication is faithful (each copy returns), a
  self-referential block is not.
- **A reinterpreting read `*(T*)&field` becomes a read of `field` CAST TO A POINTER.** The C
  `LogonTime.QuadPart >= *(UINT64*)&UserInfo->All.AccountExpires` takes the ADDRESS of an
  OLD_LARGE_INTEGER (a split `LowPart`/`HighPart` pair with no `QuadPart` member at all) and
  reads those eight bytes as one integer. msv1_0's `sam.rs` came back with
  `*((*UserInfo).All.AccountExpires.QuadPart as *const UINT64)` — a field that does not exist,
  and if it had, a *timestamp value* dereferenced as an address. It failed loudly here only
  because bindgen models OLD_LARGE_INTEGER without a `QuadPart`; **the same rewrite over a real
  LARGE_INTEGER compiles and reads a wild pointer.** The Rust is
  `*(core::ptr::addr_of!(x.field) as *const T)`. Grep translated output for `.QuadPart as *`
  and, more generally, for any `as *const`/`as *mut` whose operand is a struct FIELD rather than
  a pointer.
- **An out-parameter passed by ADDRESS instead of BY VALUE.** A C++ `T*& out` or `T** out`
  parameter has already lowered to `*mut *mut T` in Rust, so `addr_of_mut!(out)` adds a level of
  indirection: the callee writes into THIS frame's parameter slot and the caller's pointer is
  never filled. `shellex_get_contextmenu` passed `addr_of_mut!(cm)` where the C wrote
  `IID_NULL_PPV_ARG(IContextMenu, &cm)` on a `CComPtr<IContextMenu>&`. Whenever the C takes `&x`
  on a REFERENCE parameter, the Rust wants the parameter itself.
- **An A/W pair whose twins disagree on TYPES.** Only one of the two gets read carefully, and
  the other keeps whatever the translation guessed. `RealShellExecuteA`/`ExA` (SHELL32.265/266)
  had `HWND`, `PHANDLE` and the `HINSTANCE` return flattened to `isize` while the W twins were
  spelled correctly. i386 hides it — `isize` and a pointer are both 4 bytes — so nothing fails
  until an arch or a caller disagrees. Diff the two signatures against each other, not just each
  against the C.
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
- **`if (CAS(...))` compared against the WRONG operand — an atomic publish that frees what it
  just published.** C tests the compare-exchange result for truthiness: the call returns the
  PREVIOUS value, so non-NULL means *another thread got there first and my block is the loser*.
  msvcrt's `alloc_pioinfo_block` is the canonical shape —
  `if (InterlockedCompareExchangePointer(&pioinfo[i], block, NULL)) { …; free(block); }`. The
  translation rendered the condition as `!= block`, which reads plausibly ("did my value land?")
  and is exactly backwards: **the winning path returns NULL**, `NULL != block` is true, and every
  *successful* allocation frees the block it had just published into the global table. Every fd
  block became a dangling pointer, critical sections included, and the boot wedged inside
  `LdrLoadDll` at the first msvcrt load. **Rule: the result of a CAS is the OLD value — compare
  it against the COMPARAND (or test it for NULL), never against the exchange operand.** The
  inversion is invisible single-threaded, and `cargo check` cannot see it: both spellings are
  well-typed pointer comparisons. Check every translated CAS against its C by asking which arm
  is the *loser's*.
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
- **A C macro's EARLY-OUT guard dropped when the macro became a helper fn.** `fsrtlfuncs.h`'s
  `FsRtlTestAnsiCharacter` opens with `(SCHAR)(C) < 0 ? DEFAULT_RET : ...` — a high byte is DBCS
  and never reaches the table. All three Rust wrappers (`LegalFat`, `LegalHpfs`, `Wild`) kept
  only the table lookup and indexed `LEGAL_ANSI_CHARACTER_ARRAY_DATA`, a `[u8; 128]`, with the
  raw byte: **up to 128 bytes of out-of-bounds read past a static**, answering from whatever
  the linker put next in `.rodata`. Note the shape — the guard is not a "check", it is the
  BOUND, so dropping it converts a total function into UB whose answer changes with the link
  layout. `FsRtlIsLeadDbcsCharacter` lost its `< 0x80 ? FALSE` the same way. When a
  `FORCEINLINE`/macro becomes a `fn`, its leading ternaries are part of the contract.
- **An EXTRA store the C does not have** is as wrong as a missing one, and much quieter.
  `_KeAcquireGuardedMutex` wrote both halves of the `ApcDisable` union; `ke_x.h` writes only
  `SpecialApcDisable` (`KernelApcDisable` belongs to `KFAST_MUTEX`). A field that is written
  and never read is invisible to the compiler, the linker, and every in-tree caller — the only
  witness was a conformance test that pre-fills the object with `0x55` and asserts the field is
  untouched. When translating, diff the SET of fields assigned, not just the values.
- **`addr_of!(field)` where C passes the pointer field's VALUE** — the buffer becomes struct
  innards (win32k clip-region 0x50).
- **A plain Rust-ABI `unsafe fn` address handed to the kernel** (timer/dispatch callback).
- **A hand-rolled `va_arg` walking `&last_fixed_param`** is amd64-only garbage.
- **Kernel Rust emitting x87 (`fild`/`fistp`) for 64-bit atomics** → an imprecise 0xc0000092
  BSOD blamed on an innocent function.
- **A predicate macro INLINED as a different predicate.** `balance.rs` rendered
  `RMAP_IS_SEGMENT(Entry->Address)` — C's `((x) & ~0xff) == ~0xff` — as `Address & 1`. A segment
  rmap address is `RMAP_SEGMENT_MASK | page_index`, so `& 1` asks "is the page index odd?": it
  skipped half the segment entries and let the other half through as if they were process
  mappings. `Entry->Process` on a segment entry is a `PCACHE_SECTION_PAGE_TABLE`, so the
  balancer's page-out walk handed a page table to `ObReferenceObject`,
  `ExAcquireRundownProtection`, and `KeStackAttachProcess`, then ran `MiAddressToPte` on the
  `0xFFFF…FF00` pseudo-address and cleared an accessed bit through it. Both operands are
  plausible bit tests on the same field, so nothing downstream can tell them apart. **When a
  macro is inlined rather than called, the inlining is a translation of its BODY — diff the
  body, not the name.** The general tell: an inlined predicate that is *simpler* than the C
  macro it replaces.

### C++ constructs Rust cannot spell — overloads, default args, and RAII locals
Translating a `.cpp` adds a failure class translating a `.c` does not have: C++ features with no
Rust equivalent. Two are loud, one is silent, and the silent one is the reason to check for all
three by hand. All three came out of one file (`brfolder.cpp`, 2026-07-31).

- **A STATIC member function given a `this` it never had.** The COM scaffold hands every method
  body a `this` parameter, including one the C++ declares `static`. `CFSDropTarget::_DoDropThreadProc`
  is a static used as a raw `LPTHREAD_START_ROUTINE`; the C recovers the object from its parameter
  block (`data->This->_DoDrop(...)`). The reply kept the scaffold's `this` AND passed `data->This`,
  so the call had one argument too many and the wrong receiver. Loud here only because the arity
  was also wrong — with a matching arity it would have dispatched a real method on a bogus
  receiver. Check every `static` member the scaffold gave a receiver to, and make the raw thread/
  callback entry point a separate `extern "system"` fn that recovers `this` the way the C does.
- **A local RAII type whose sealed FACT token is a template argument has nowhere to land.**
  `CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlFull;` — the constant `--seal-facts` protects is the type
  ARGUMENT, and the faithful Rust (a raw pointer plus an explicit free, or the file's own
  `CComHeapPtr`) does not spell it, so the fact envelope reports it missing however the body is
  written. `CFSDropTarget::_DoDrop` was refused twice on exactly this and had to be hand-written.
  Same root as the `--local-raii` gap; the sanctioned CComPtr lowering is the assembly's own
  `CppComPtrGuard`, and a reply reaching for `scopeguard::` is rejected by design.
- **Overloads collapse to whichever one the model translated.** C++ had two
  `BrFolder_GetName`s — `(IShellFolder*, PCIDL, ...)` and `(BrFolder*, HTREEITEM, ...)`, the
  second delegating to the first. Rust has no overloading, so only one survived, and the call
  sites for the *other* one were left pointing at it — with a `.cast()` politely inserted on the
  first argument, which is exactly what makes it look deliberate. Loud (`E0308`/`E0061`), but
  read the C before "fixing" the call: the answer is to write the missing overload under a new
  name, not to bend the arguments until they typecheck.
  Again in `filedefext.cpp` (2026-08-04): `GetFileStats(HANDLE, ...)` and
  `GetFileStats(PCWSTR, ...)`, the path form opening the handle and delegating. Only the HANDLE
  form was translated, so the surviving call site — the `else` branch of `ProcessDirStatsItem`,
  i.e. every non-directory item in a folder's Properties size calculation — had no callee at
  all. Written by hand as `GetFileStats_Path`. Worth grepping the C for a duplicated signature
  before assuming a "missing" callee was simply dropped.
- **A free function and a MEMBER may share a name; C++ resolves it by scope, not by arity.**
  `CDefaultContextMenu` declares a static `MapVerbToCmdId/5` and a method `MapVerbToCmdId/3`.
  Inside a method body the unqualified call takes the member, and the free one is reached by
  writing `::MapVerbToCmdId`. A binder that knows only the free candidate reads the member call
  as an unbindable arity and hard-fails the entire TU before payment — which is how a 2,024-line
  file looked untranslatable when nothing was actually wrong with it. The tell that this is an
  asymmetry rather than a missing feature: the *rewrite* pass already handled it (no candidate at
  that arity → `continue` → the method pass binds it), and only the *reachability* walk raised.
  The mirror image is worth fixing at the same time: a method pass that matches a bare identifier
  without looking at what precedes it will rewrite `::f(...)` into a call on `this` whenever the
  arities happen to line up, turning an explicit "I mean the global one" into its opposite.
- **A default argument is not carried to the call site.** `BrFolder_FindTreeItemOfAbsoluteItem(
  BrFolder&, PCIDLIST_ABSOLUTE, HTREEITEM hItem = NULL)` called with two arguments in C++ needs
  three in Rust. Loud (`E0061`), trivial to fix, easy to mis-fix by deleting the parameter.
- **An RAII local becomes an UNINITIALISED binding, and runs NEITHER half. This one is silent.**
  `COleInit OleInit;` — a `CCoInitBase<OleInitialize, OleUninitialize>` from `shellutils.h` —
  came out as `let _ole_init: COleInit;`. That compiles: an unused, uninitialised Rust binding
  is legal, is never constructed, and is **never dropped**. So `SHBrowseForFolderW` ran its
  whole dialog with no `OleInitialize` and no `OleUninitialize`, against a C comment that says
  in so many words "we do it anyway in case the caller forgot".

  Two things conspire here. The guard is a class *template* in a header, so like any header
  inline it lives in no TU and rustconv never sees a body — but unlike an inline it leaves no
  unresolved name to trip over either, because bindgen renders it as an opaque
  `pub type COleInit = u8;`. A declaration against that alias is well-formed. Nothing anywhere
  in the pipeline is red.

- **A container FIELD and a container OF C++ OBJECTS are two different gaps, and only the
  second one can leak silently.** Symptom: a class is preflight-UNSUPPORTED on
  `CSimpleArray<FORMATETC> m_Formats` and on `CAtlList<StaticShellEntry> m_StaticEntries`, and
  they look like the same missing feature. They are not. The container FIELD is mechanical: it
  cannot be flattened to a pointer the way `CComPtr` is (its destructor walks storage the
  scaffold does not own), so the field keeps its type, is initialised with `new()` rather than
  `zeroed()`, and gets a `drop_in_place` in the generated `Release` — after the explicit dtor
  body, which is C++'s own member-destruction order. The ELEMENT is the real question, because
  a container `Drop` that does one `drop_in_place` per element reproduces the C++ only when the
  element is **trivially destructible**. `StaticShellEntry` holds a `CStringW` and
  `DynamicShellEntry` a `CComPtr<IContextMenu>`, so a container over either leaks every element
  — and nothing is red: it compiles, links, boots, and loses an interface reference per
  right-click. The deterministic proof that an element is safe is the **bindgen island**:
  bindgen only ever renders C declarations, so a name it emits as a struct came from a C header
  and has no destructor. A C++ class is never in that set. Follow aliases when you check —
  `FORMATETC` is `pub type FORMATETC = tagFORMATETC;` over `pub struct tagFORMATETC`, so
  matching `pub struct` alone proves almost nothing — and read every `-sys` crate the island
  imports from, not just the first: an island takes its ordinary base types from one
  (`ntdll_sys`) and its COM types from another (`com_sys`), and stopping at the first refuses
  every OLE struct.
  **The island is one of TWO safe families, and the second is where the leak actually lived**
  (2026-08-05, `CDefaultContextMenu`). An island struct is safe because it is trivially
  destructible. A *source-discovered POD* is safe only if it is RENDERED owning — and it was
  not. The POD renderer spelled ATL members with the COM **owner's** type, which is right for an
  owner (its generated `Release` releases each member explicitly) and fatal in a POD, whose
  field types **are** its entire destructor: `pCM: *mut IContextMenu` inside the element makes
  the container's per-element `drop_in_place` drop a raw pointer. `PlainStateField.owned_rust_type`
  is the "Rust drop glue owns this" spelling (`CComPtr<T>` / `CComHeapPtr<T>` / `CStringW`), all
  `repr(transparent)` over the same pointer so layout is untouched. Two corollaries fall out: a
  by-value POD member of the owner needed the same `drop_in_place` in `Release` (transitively —
  a wrapper struct owns whatever it contains), and a **packed** struct with a destructor member
  cannot exist in Rust at all, so refuse it in preflight by name instead of eating `E0793` after
  the money is spent.
  **The free-function renderer had the identical bug, and the SHIPPED file is what proved it.**
  `brfolder.rs` spells `BrItemData`'s fields `CComPtr`/`CComHeapPtr`; the generator emitted raw
  pointers — directly above the `CComHeapPtr` support block it was not using. So a regeneration
  would have quietly downgraded a correct file into a leaking one, and nothing would have been
  red. When a generator and a shipped artifact disagree, the artifact is evidence: diff them
  before assuming the generator is the source of truth. Sweep for the shape with a grep for C++
  structs holding `CComPtr`/`CStringW`/`CComHeapPtr` members — repo-wide there are 12, and only
  one had a Rust port, which is why this stayed latent until a TU needed it.
  **Grep every flipped `.cpp` for a bare `let <name>: <CppType>;` with no initialiser.** If the
  C++ type has a constructor or destructor, that line is a dropped side effect. Port the guard
  for real (`struct` + `impl Drop`) and construct it — `COleInit::new()` in
  `shell32-rust/src/shellutils.rs` is the worked example, including the `prelude` re-export that
  shadows the island's `u8` alias so the name still resolves to the real thing.

  **A ported guard gets exactly ONE definition — unlike an `extern` declaration.** Re-declaring
  an `extern` fn per module is normal FFI practice and shell32-rust does it 530 times over; a
  re-derived RAII guard is not the same thing, because the failure mode is a leak no compiler,
  linter or smoke test can see. Measured: `CRegKeyHandleArray` (shfldr.h) was re-derived
  independently in `cfs_folder.rs` and `desktop_folder.rs` and **both copies had lost the
  destructor** — the same defect twice, leaking up to 16 HKEYs per right-click, and the tell was
  that two translations of two different TUs made the identical omission. A third TU wanting it
  is the moment to consolidate, not to copy again. It now lives in `shell32-rust/src/shfldr.rs`.
  Watch the TYPE WORLD when you consolidate: a crate with both a hand-written substrate and a
  bindgen island has two `HKEY`s of the same width and different identity, so the shared module
  must name which one it is in, and a consumer in the other world needs an explicit `use` (which
  shadows a glob) rather than a silent coercion.

  **Two different conversion operators can fire on ONE object in ONE call.** C++'s
  `InitFunc(InitString, pDO, keys, keys)` passes `HKEY*` then `UINT*` from the same
  `CRegKeyHandleArray`, and `SHOpenPropSheetW(.., keys, keys, ..)` passes `HKEY*` then `UINT`.
  Translated, the repeated argument reads as a copy-paste and invites writing one expression
  twice; it must become `keys.hKeys.as_mut_ptr()` and `&mut keys.cKeys`. Grep any flipped C++
  for a call passing the same identifier twice adjacently and check the C++ signature.

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
- **A pointer difference with an adjustment folded into `offset_from`, and then applied
  AGAIN.** `sym->current - 1 - ptr` parses as `(current - 1) - ptr`; the translation correctly
  turned `current - ptr` into `offset_from(ptr)` and then kept the `- 1` as well, so msvcrt's
  demangler subtracted it twice and every identifier came out one byte short —
  `CBaseUnknown` as `CBaseUnknow`, in the shipping DLL, on every C++ symbol. The shape to
  distrust is any `a - K - b` where `a` and `b` are pointers: the `K` belongs to ONE of the two
  operands, and once `a - b` becomes a method call the reader can no longer see which.
- **Cast-before-offset**: `(ptr as *T).offset(1)` overshoots by the size delta → heap overrun.
  **A ZST-array pointer `.add()`** steps 0 bytes.
- **A dangling pointer to a by-value `transmute` temporary** — the WCHAR[] → storageless-
  temporary trap.
- **The `sizeof` OPERATOR itself vanishing** — the sibling of the entry below, and it survives
  a green C build because the C compiled fine and only the Rust ever sees the loss. Three shapes,
  all from shell32: `HeapAlloc(h, 0, len * sizeof(WCHAR))` became `len * 0` (a ZERO-byte
  allocation that `MultiByteToWideChar` then wrote `len` wide chars into, on every ANSI entry
  point of `shlfileop`); `startup.cb = sizeof(STARTUPINFOW)` became
  `size_of_val(&STARTUPINFOW)`, passing the TYPE where a value belongs; and both
  `if (sei->cbSize != sizeof(SHELLEXECUTEINFO{A,W}))` gates in `shlexec` lost the `sizeof`
  outright, leaving a bare type name — that comparison is the version check every caller of a
  public `ShellExecuteEx` is validated by. The first two compile to something wrong; the third
  does not compile, which is the only reason it was found. **Grep the translation for
  `size_of_val(&` on a bare type name, and diff every `sizeof` in the C tail against the Rust.**
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

### A flipped TU's leftover `.obj` keeps voting on the NEXT flip's closure
ninja never deletes the object of a source you remove from a CMake list, so every TU we flip
leaves a stale `.obj` under the link root forever. `rustconv hybrid-closure` globs `*.obj` for
its undefined side, so those leftovers keep counting as consumers and invent demand that no
longer exists: `filedefext.cpp`'s closure reported `SH_FormatByteSize` as still needed by
`drvdefext.cpp.obj` months after drvdefext.cpp became Rust, and `CDrivesFolder.cpp`'s reported
two `cxx_free` symbols needed by that same dead object and by `CDesktopFolder.cpp.obj`.

**The direction of the error is what makes it costly**: phantom demand makes the next flip look
*harder* than it is. It argues for keeping C++ linkage on a symbol nothing in the build calls,
or for a forwarder nobody needs. filedefext.cpp's real closure is 10 symbols, all already
`_RustBridge` free functions; the phantom 11th was the only thing that looked like a blocker.

The tool now filters by CMake liveness, conservatively — an object is dropped only when its
source is provably flipped (present under the target dir, absent from every live list). An
object whose source is not under the target dir belongs to another target and is always kept,
because *understating* a closure is the dangerous direction: it reads as "this TU is free to
leave" right up until the link fails.

**Do not confuse the two analyses.** A retained closure measures OUTBOUND demand — what the Rust
must publish once the `.cpp` goes. It says nothing about whether the Rust bridges can REACH the
functions they forward to; those targets may be C++-mangled and reachable today only through the
very forwarders that are leaving. Answering "zero C++ edits needed" from the closure alone is a
mistake — see the `EXTERN_C`-without-`WINAPI` entry for the link failure it produces.

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
- **Verify it in the MEDIA YOU BOOT, not the module you built.** `strings build-<arch>/ntoskrnl/
  ntoskrnl.exe` answers a different question than `strings <the .iso qemu opens>`, and only the
  second one is the experiment. This cost a 12-round, ~9-hour arm64 soak on 2026-07-26: the media
  was deliberately frozen for run-to-run comparability at 22:42, the two probes it was built to
  exercise landed at 02:43, and every round then booted a kernel with **zero** `FREEMAPPED` and
  **zero** `KSTACK` literals. The binary check passed; it was checking the wrong artifact. The
  soak's face taxonomy survived (bugcheck operands and the census were in that media) but every
  probe-derived conclusion was vacuous — and "the tripwire never fired" reads as evidence of
  absence when it is really absence of the tripwire (§9.122).
- **Freezing media and adding instrumentation are in direct tension** — a fixed image is what
  makes rounds comparable, and it is also what silently locks out the probe you add at hour 3.
  If you freeze, re-verify the frozen artifact after every probe commit, or re-cut the image.
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

### A file MISSING from the arm64 ramdisk is fabricated by the kernel, not reported
**Symptom.** A cross-arch conformance sweep showed `\REGISTRY\MACHINE\SAM` reporting the
*expected* 4-ACE ACL on arm64 and a "wrong" 5-ACE one on both x86 arches. arm64 looked like
the correct side, so the investigation started by hunting an x86 kernel bug.

**Cause.** `qemu/mkramdisk.sh` hand-lists the files it stages and had drifted from the livecd:
it copied three of the five hives `mkhive` emits (`system`, `default`, `software`), omitting
`sam` and `security`. The livecd's `add_cd_file` list (`sdk/cmake/CMakeMacros.cmake`) carries
all five. Nothing anywhere reported the gap — `CmpLoadHiveThread` cannot open the file, sets
`Allocate`, and `CmpLinkHiveToMaster` **fabricates an empty volatile hive** whose root key gets
`CmpHiveRootSecurityDescriptor()`. The key then exists, opens with SUCCESS, and answers queries.
arm64 had been booting with an empty SAM the whole time, samsrv re-creating the domain
structure into volatile state every boot.

**Why it inverted the diagnosis.** The fabricated descriptor is *character-for-character* what
the test expects, because both come from the same function. So the arch with the missing data
scored BETTER than the arch reading the real hive. **A test result that looks more correct on
one arch is evidence about that arch's INPUTS before it is evidence about its code.**

**Fix.** Stage all five; list them in `REQUIRED_FILES` so a missing one hard-fails instead of
being fabricated. Confirmed by prediction: arm64 `CmSecurity` moved 663/74 → **659/87**, exactly
i386 and amd64, with only the known-flaky `IoDeviceObject` assertion moving besides.

**Generalize.** Two hand-maintained media lists that must agree, with no mechanism forcing it,
drift silently — and the *kernel's* graceful degradation is what hides it. When one arch's
result differs, diff the media before diffing the code.

**Mechanised.** `tools/scan_boot_media_drift.py` (in `vibeos-cli check`, baseline EMPTY) now
gates the invariant statically: every hive `add_cd_file` stages into `reactos/system32/config`
`FOR livecd` must also be copied by `mkramdisk.sh` **and** listed in its `REQUIRED_FILES`, and
`mkhive`'s `-h:` list must match the `OUTPUT` files cmake declares for it. Run against the tree
as it stood before the fix it names `sam` and `security` outright. Scoped to the config
directory on purpose — the BCD hive goes to `efi/boot` for the EFI boot manager and is not a
`\REGISTRY\MACHINE` hive, so requiring it on the ramdisk would be a false positive.

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

### A second ninja pass re-emits a shared cargo cell, stranding the first pass's binaries
**Symptom:** `BUILD_INCOMPLETE` from `artifact-check`, intermittently, naming binaries nobody
touched — `ws2_32.dll`, `fastfat.sys`, `ntoskrnl.exe` — as older than the cohab staticlib they
link. Halted the merge train for ~an hour across four lanes.
**Cause:** `build.py` runs each `ninja_targets_optional` entry as its OWN `ninja <target>`
invocation, *after* the main pass. But an optional target shares cargo cells with the main
graph: `ntkrnlmp` and `ntoskrnl` both build `-p kernel-rust` into one `CARGO_TARGET_DIR` and one
copied `libkernel_rust.a`. So the optional pass re-emits an archive the main pass's binaries
already consumed, and nothing relinks them, because the main pass is over. Measured on amd64:

    libkernel_rust.a   02:08:11   <- re-emitted by `ninja ntkrnlmp`
    ntkrnlmp.exe       02:08:23   <- linked against the new bytes
    ntoskrnl.exe       00:27:35   <- 1h40m older than the archive it links

`mkramdisk` then stages that stale kernel — the arm64 false-green shape this whole section
exists for. Shipping both flavors under distinct names (entry above) fixed the *identity*
problem and left this *ordering* one behind it.
**Why every sighting is arm64:** `ninja_targets_optional` is non-empty for exactly one arch
(`archcfg.py:205` — `arm64input`, `arm64vid`, `ntkrnlmp`, `halmp`, `kmtest_all`); i386 and amd64
inherit the `()` default and so never run a second pass at all. If you see this on x86, the
cause is something else. `kmtest_all` is the widest of the five and reaches cohab crates far
outside the kernel, which is how `ws2_32.dll` and `fastfat.sys` — targets nobody had touched —
came to be named in a BUILD_INCOMPLETE.
**Fix:** a `ninja-reaffirm` pass over the same target list after the optional passes and
**before** any staging step. No-op normally (~1s), a real relink when a cohab lib moved; it
reports `relinked N` so a cell that re-emits on *every* build stays visible rather than being
silently absorbed.
**The triage hint was actively misleading, which is why it sat.** `provenance._lag` printed
"SAME-RUN ORDERING, suspect a race not a stale edge" for any sub-2s gap. But the copy is
`cmake -E copy_if_different`, so a fresh mtime means the **bytes really changed** — which a
finished link cannot have seen. A sub-second gap here is the *opposite* of benign. **General
rule: before calling an oracle's complaint a false positive, check what the write actually
guarantees** — `copy` and `copy_if_different` differ precisely on whether mtime implies change.

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
- **Don't run `rustconv translate` while a build is running in the same worktree.** The
  staleness oracle compares every workspace `.rs` mtime against the cargo-linked binaries, so
  a translation that lands `wldap32-rust/src/.../bind.rs` mid-build fails `artifact-check`
  ("13min newer — a genuinely missed rebuild") on four unrelated exes. The oracle is right and
  should not be loosened — it cannot know the new crate is not in the build graph yet — but it
  costs you the whole build. Sequence the two, or translate in a different worktree.
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

### A tri-state preflight that only refuses on `False` waves through the case that actually bites

**Symptom.** A preflight written to prevent exactly one failure lets it happen anyway, on a
fresh worktree. `conformance` refused when `_rostests_enabled()` returned `False` (a build dir
configured *without* the kmtest corpus) and printed the one-time cmake line. But the reader
returned a third value — `None`, "no CMakeCache yet" — and the caller tested only `is False`.
A worktree with no `build-<arch>` at all therefore sailed straight through, the fresh configure
defaulted **both** gating options OFF, and a full media built and booted before reporting
`[KMTEST-SWEEP] launch FAILED` minutes later and a layer away from the cause.

**Cause.** `None` was read as "nothing to complain about" when it actually means "not
provisioned yet" — which is *more* certain to be wrong than the `False` the check did catch.
A tri-state whose unknown arm is the common arm is a two-state check with a hole in it.

**Fix.** Prefer *making it true* over *reporting that it is false*: the requirement moved into
the build as a sticky opt-in (`build --kmtests`, `sticky_on` in `_maybe_canonize`), which is
named on the fresh configure too — a dir that does not exist carries nothing forward, so a
caller's request is otherwise dropped on precisely the one run that cannot detect it. The
reader and its unit test were deleted with the branch they served. Generalize two ways:
- **Audit any preflight returning `bool | None` for a caller that tests one arm.** Either
  handle all three or collapse the type.
- **A "sticky" cache opt-in is the opposite discipline from an instrumentation flag.** Sticky
  is one-directional (ON, then left alone) because rebuilding the state is expensive and
  keeping it is harmless; `--lockdep`/`--cswitchdep` are reconciled *every* build precisely so
  a gate can never inherit an instrumented kernel. Putting a flag in the wrong bucket is a real
  bug in both directions.

### An arch-gated no-op trace fn discards the diagnostics on the arch that needs them
`iomgr.rs` defined `io_trace` twice: a real one for x86, and
`#[cfg(target_arch = "aarch64")] #[inline(always)] unsafe fn io_trace(_s: &[u8]) {}`. All 40 of
its call sites are one-shot boot milestones and their `FAILED` reports — including
`PsLocateSystemDll FAILED`, after which arm64 **continues booting on purpose**. So the one arch
that tolerates the failure was also the one arch that could not report it, and the failure ran
on every boot for months. LLVM additionally drops the now-unused byte strings, so the binary
does not even contain the text: `"[IO] IopInitializeBootDrivers"` was **0** occurrences before
the fix and **2** after. Two consequences worth generalizing:
- **Grep for `_s: &[u8]) {}` / empty-bodied trace shims before concluding "the probe never
  fired."** Absence of a trace is only evidence if the trace exists on that arch.
- **An arch-gated tolerate-and-continue path needs LOUDER reporting than the arch that
  bugchecks, not quieter.** Where x86 stops dead, the tolerant arch keeps running with the
  damage — and if its report is a no-op, that hatch can never signal anything, so it becomes
  permanently-on and worthless as a signal. (Compare: the arm64 MM suppressions, which kept
  their recovery and gained a print.)
- Do not treat a hand-rolled `for &ch in msg.iter() { *uart = ch }` loop as the fix — LLVM
  unrolls it into immediate stores and the literal vanishes from the binary, which also defeats
  `smoke --require-probe`. Pass the string by pointer (`ARM64_UART_TRACE`), the form proven to
  reach the serial.

**The worse sibling: the translation drops the trace ladder outright.** A no-op shim at least
leaves the call sites visible. `ntoskrnl/ex/init.c` carries **54** live `ARM64_UART_TRACE` calls —
a full `[EXP]`/`[PHASE1]` milestone ladder through `ExpInitializeExecutive` and
`Phase1InitializationDiscard` — and its cohab-live `ex-rust/src/init.rs` carried **zero**. Every
one was dropped, so arm64 had no Phase-1 visibility at all, in precisely the phase where it
flakes; the boot log's silence read as "nothing to see." Debug output is not decoration and it is
not covered by a signature check, a `cargo check`, or a green smoke — nothing downstream of the
translation notices its absence.

*Audit it mechanically, per cohab-flipped pair*, since eyeballing a 4,000-line file will not find
an absence:
```
c=$(grep -c '^[^/]*ARM64_UART_TRACE' <file>.c)
r=$(grep -c '^[^/]*\(ARM64_UART_TRACE\|uart_trace\|io_trace\)' <file>.rs)
```
Run tree-wide, the ratio is the finding: most flipped files preserve or exceed their ladder
(`ke/arm64/stubs.c` 3→46, `ps/thread.c` 17→39), so a **0** or a large shortfall stands out at a
glance. Judge each shortfall by *what kind* of trace was lost, though — restoring is not always
right. `mm/section.c` (32→0) and `ARM3/procsup.c` (33→10) lost per-page and per-process hot-path
forensics (`GetPageEntry`, `IoPageRead`, every step of `MmInitializeProcessAddressSpace`); bulk
-restoring those would drown the serial and slow every boot. Restore **milestone ladders and
failure reports**, plus any *rate-limited anomaly* probe (`MiGrabDataSection`'s
`INCREATE/INDELETE` spin fires 1:1000 and only when wedged — exactly the hang diagnostic you
want); leave unconditional hot-path tracing dropped.

**The same audit applies to upstream's own `DPRINT1`, and there the shortfall is repo-wide.**
Across `sdk/lib/rust`: **2906** `DPRINT1(` sites in the commented-C trailers vs **306** live
`dbg_print!`/`dbg_print_ex!` — so ~90% of ReactOS's kernel diagnostics did not survive
translation. Unlike the trace ladders these are almost entirely *failure reports*, which is the
subset this entry says to restore. Concretely: `CmpLinkKeyToHive` (`config-rust/src/cmsysini.rs`)
has three failure returns and upstream prints on every one; all three were dropped, so when
`\Registry\User\S-1-5-18` failed to appear there was no signal anywhere and no way to tell a
failed `ZwCreateKey` from a failed `ZwSetValueKey` from a link that was created correctly.
Restoring them and then getting **silence** back is what proved the create half was fine — an
absence is only evidence once the print exists. Same counting recipe as above, with
`grep -c 'DPRINT1(' <file>.rs` (commented C) against the live print count.

### A `cfg(debug_assertions)` block is not code that works — it is code with no evidence either way

Same family as the entry above: source that is present in the tree and absent from the artifact.
There the diagnostic was thrown away at run time; here it was never compiled at all.

**Symptom.** Turning on `debug-assertions` for the first time does not merely start firing
assertions. It *compiles*, for the first time in the life of the tree, every
`#[cfg(debug_assertions)]` block in the crate — **108 of them** here — and *deletes* the 14
`#[cfg(not(debug_assertions))]` blocks, which is how a `not()`-only stub definition becomes an
undefined symbol. Three of the first blocks compiled were broken:

| where | defect | presented as |
|---|---|---|
| `freeldr-rust/src/heap.rs` | heap-statistics block deref'd the `PVOID` handle with no `as PHEAP` | 14 × `E0609: no field … on type c_void` |
| `ex-rust/src/init.rs` | `KeServiceDescriptorTable[0] = core::mem::zeroed()`, **invented by the translation** | nothing — it compiles |
| `ke/{amd64,i386}/kiinit.rs` | called `KiReportCpuFeatures`, never translated, C cohab-dead | `ld.lld: undefined symbol` |

The middle one is the lesson. `ntoskrnl/ex/init.c:1415` assigns only `.Count`; the Rust zeroed
the whole descriptor, discarding `Base`/`Limit`/`Number` — **the syscall dispatch table** — which
`ke/krnlinit.c:104-108` fills in earlier. It compiles clean, and it would have killed the box at
the first system call after Phase 0. It sat in the kernel's most-read init file, invisible,
because the compiler had never been asked to look and no boot had ever run it.

**Cause.** `cfg` is not `if`. Code behind a `cfg` that is never set is not dead-but-correct code;
it is unverified text — no type check, no borrow check, no lint, no execution. Translated
`#if DBG` blocks land in exactly this state, because cargo's release profile leaves
`debug_assertions` off while the C they came from is built `-DDBG=1` and runs them for real. The
C half of a cohab kernel checks its invariants; the Rust half's debug code quietly rots.

**Fix.**
- Treat "enable `debug-assertions`" as a **migration, not a flag flip**, and scope it.
  `[profile.release.package.<crate>]` overrides arm the crates a hunt needs (kernel core: 181
  assertions, 29 blocks) without also inheriting the loader/FS/driver backlog.
- Set `overflow-checks = false` **explicitly** in every override — it defaults to *following*
  `debug-assertions`, and kernel-wide overflow panics are a far larger semantic change than
  assertions, one the C never had (C wraps).
- Before believing a `cfg`-gated block, check every symbol it calls actually **exists**. An
  `extern` declaration in the same file is not evidence, and a `// fn Foo(...)` comment header in
  the translated module means the function was skipped entirely.
- Audit `#[cfg(not(debug_assertions))]` in the same pass — those are the arms that vanish.

**What is NOT a hazard:** `debug_assert!(expr)` itself. It expands to
`if cfg!(debug_assertions) { assert!(expr) }` — the `cfg!` *macro*, not the attribute — so its
expression is type-checked on every build regardless. Only the attribute form hides code from the
compiler. That asymmetry is why the assertion count (2,120) and the never-compiled-block count
(108) are unrelated numbers, and why only the second is a migration risk.

**Wider rule:** any `cfg` never enabled in a shipping configuration is an unverified region, and
how many you have is a measurable debt. The same argument covers any feature flag CI never
builds. A `cfg` you never compile is a `cfg` you cannot claim works.

### The governing rule: a harness only falsifies what it varies
> **A harness proves parity only across the axes it actually VARIES. Every axis it holds
> CONSTANT is a silent assumption baked into every MATCH it prints.** `[§9.60]`

Before believing a verdict, enumerate what the harness holds fixed. Two blind spots that shipped
bugs: **ambient state the reference holds fixed** (locale — grep a flip candidate for
`get_locinfo` / `_l`-suffixed helpers / any read of process-global state), and **the reference
TU is compiled for the HOST**, so target-conditional `#if`s are graded on the wrong arm.

**A stub that is merely INACCURATE can make a whole arm unreachable, and every row still says
MATCH.** Wave 16 had to grade `_wctomb_s_l`'s insufficient-buffer path, whose entire trigger is
`WideCharToMultiByte(...) == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER`. The harness's
model of `WideCharToMultiByte` *truncated* where real kernel32 writes nothing, returns 0 and
sets the error — so a single-character conversion could never return 0, and the arm was graded
by nothing. A reachability probe measured it at **0 hits of 0** across 3 codepages × 9,363
characters × 4 sizes, while every row over that family read MATCH. Note the shape: this is not a
missing stub (which fails loudly at link) nor a wrong *value* (which a row would catch) — it is
a stub whose ERROR MODEL differs from the real API, which is invisible precisely because the
error is what never happens. **When a branch exists to handle a failure, instrument how often
your harness actually produces that failure, and keep the probe.** Ours is permanent, so a
future regression goes red and names its own cause.

**And the converse: a preserved C quirk may be UNFALSIFIABLE, in which case a green row is not
evidence you preserved it.** `wctob`'s `wchar < 0xff` looks like exactly the kind of
off-by-one worth transcribing carefully, and the only input that could distinguish `<` from
`<=` is U+00FF — where `<` returns `EOF` (-1) and `<=` returns `(signed char)0xFF`, also -1.
Relaxing the comparison moves no row and *cannot*. The row is still worth having (it is
exhaustive, and a control that returns the raw value instead of the signed char fails at once),
but the commit message that credited it with pinning the comparison was wrong. **Run the
perturbation before claiming a quirk is pinned**; "I transcribed it faithfully" and "a test would
catch me if I hadn't" are different claims, and only the second needs a control to back it.

A third: **an out-parameter the harness only PARTIALLY compares.** rtl-difftest asserted
`(Length, MaximumLength)` on the STRING converters but not `Buffer`, so an eleven-site
divergence — the alloc-failure path storing nothing where the C stores NULL — was visible in
exactly the one test that happened to compare all three fields. Ten of eleven sites were graded
green by a suite that never looked at the field carrying the bug. **Compare the whole
out-parameter, or write down which field you dropped and why.** Pointer values legitimately
differ between the two sides, but `is_null()` does not — grade the property, not the address.

**The same rule governs RUNTIME tripwires, and there the blind spot is reachability: an
instrument only observes the call paths that reach the function it lives in.** A per-arch
reimplementation of a shared helper silently exempts its entire arch, and the clean rounds that
follow read as evidence when they were never a test. The `SPIN_LOCK_ALREADY_OWNED` self-deadlock
tripwire lives in `ntkrnl-sys` `KxAcquireSpinLock`; arm64's HAL had its own plain-store spinlock
helpers and never called it, and `hal.spec` exports the whole `Ke*SpinLock*` surface
`-arch=i386,arm,arm64` — so on arm64 every driver call site and every caller using the export
rather than the inline was invisible to it. A 20-round soak was recorded as "a weak negative for
the spinlock hypothesis"; it constrained almost nothing, because the detector was not installed
on the surface under suspicion. **Before crediting a negative from an instrument, grep for every
implementation of the function it lives in — per arch AND per build flavour — and confirm the
suspect path actually reaches it.** A tripwire in a shared funnel is only as broad as the set of
callers that genuinely funnel.

**Reachability is not the only way an instrument can be structurally blind. Three more, all
found on one wedge:** *(1)* **An instrument driven by the mechanism under suspicion goes silent
exactly when it should fire.** The `[CENSUS]` sync watchdog loops on
`KeDelayExecutionThread(10s)` — a timed wait — so a stall in timed-wait expiry stops the
watchdog first, and its silence gets read as "nothing is stuck". Ask what your detector *depends
on*, not just where it lives. *(2)* **A capture written to a directory someone else prunes is the
same as no capture.** `wait_for_boot` wrote its per-vCPU PC dump to the VM's `work_dir`, which
under the pytest `vm` fixture is a `tmp_path_factory` dir — pytest keeps the last 3 runs, so the
one dump anybody ever needed had been deleted by unrelated later runs before it was read, and
the run dir held only serial. Diagnostics go to the caller's durable dir, not wherever the
harness happened to be working. *(3)* **One sample cannot tell FROZEN from BLOCKED.** A dead
clock and a thread waiting on something that never arrives both park every vCPU in the idle loop
and produce byte-identical dumps — and they are completely different bugs. Sample twice, seconds
apart, and write the verdict into the artifact while the guest is still alive; the fork cannot be
recovered afterwards.

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

**A HOST HEADER can redefine a reference constant into different BEHAVIOUR, and only a
harness that watches internal state will see it.** At `_MSVCR_VER < 140` — the arm this DLL
builds — `msvcrt.h` spells `MSVCRT__NOBUF` as `_IONBF`, so that macro is not a `setvbuf` mode,
it is the "stream is unbuffered" flag bit that `_fwrite_nolock`, `_flsbuf`, `_filbuf` and
`fflush` all branch on. msvcrt's `_IONBF` is `0x0004`; **glibc's is `2`, which is `_IOWRT`.** The
reference TU took glibc's, so it read every write-mode stream as already unbuffered: it never
allocated a buffer, wrote a byte at a time straight to the fd, and left `_base` NULL. Every
`fwrite` row still returned the same count and left identical bytes on disk — the two sides'
file POSITIONS came out nine apart, and nothing but the FILE's internals could tell you why. Two
lessons. First, a reference TU compiled for the host inherits the host's macros for every name
it does not explicitly displace, and the dangerous ones are the values that are *plausible on
both sides* (`_IOFBF` is 0 in both, which is what makes 1-vs-0x40 and 2-vs-4 look like noise).
Derive them with the pinned cross-compiler and `#undef` before `#define`, the way the errno
block already does. Second, **grade the buffering, not just the answer**: put the FILE's
`_flag`, `_cnt`, `_bufsiz` and `_ptr - _base` in the compared image. Buffering is a behaviour
with no other window — buffered and write-through return the same value and produce the same
file.

**Once the SUBJECT defines libc names, the harness's shared ENVIRONMENT starts calling into the
code under test.** msvcrt-difftest's `fileimpl.c` is the POSIX-backed Win32 layer *underneath*
the stream layer, compiled without the rename prelude so its calls land on plain names — and a
strong definition beats libc at link time. It carried a comment saying `strlen` was "the only
name in this file's import set that collides with the subject's exports — checked, not assumed",
which was true right up until file.rs entered the link. Then `MoveFileExW`'s `rename()` became
msvcrt's `rename` → `_wrename` → `MoveFileExW`: unbounded recursion, stack overflow before the
first row ran. The general shape is worse than the crash, because a collision that does NOT
recurse is silent — the reference's environment simply runs the subject's code and every row
agrees. Two fixes, and take both: call the same operation under a name msvcrt does not export
(`renameat`, `dprintf`), and make the rule a command — intersect the environment object's
undefined symbols with the ported file's export list at build time. A rule with no check behind
it is a rule that was true when it was written.

### A shared ledger that collapses two configurations makes the A/B invisible
The corollary of the governing rule above: a harness only falsifies what it varies, **and you
can only tell what it varied from what it RECORDED**.

**Symptom.** Two sets of rounds in one corpus, wildly different failure rates, every recorded
field identical. The difference reads as a fix.

**Cause.** `conformance`'s ledger wrote `"smp": smp or 1`. Passing no `--smp` boots the UP
kernel (`ntoskrnl`); passing `--smp 1` boots the MP kernel on one CPU (`ntkrnlmp`, plus PSCI AP
bring-up and its own `--smp-kernel` media on arm64). Two different binaries doing different
work — and `or 1` wrote **1** for both. 2026-07-29: 44 arm64 rounds at 100% clean were compared
against 85 banked rounds at 49% clean and the gap was attributed to a fastfat fix. It was the
kernel swap. Nothing in the corpus could contradict it; the confound was only recoverable by
grepping serial logs for an `ntkrnlmp.exe` load line, which is exactly the "remembering a
recipe" smell that means a field is missing.

**Fix.** Record the distinguishing variable, not a normalised stand-in — `smp` truthfully
(0 = unspecified) plus an explicit `mp_kernel`. **A default that maps "unset" onto a real
setting's value is not a convenience, it is a lost degree of freedom**, and in a corpus other
lanes compute statistics from it silently corrupts their conclusions too, not just yours.

**Generalises to:** any run-ledger field with an `or <default>`. Ask what two distinct
configurations could produce the same stored row, and whether either is one you would ever want
to compare against the other. Same family as the worktree ambiguity in the same corpus, where
three trees' rounds are separable only by string-matching `run_dir`.

### A verdict that blocks other people must carry its evidence
Sibling of "an oracle needs an oracle": that one asks *did the oracle look?*, this one asks
*can anyone check what it saw?* One night's merge-train stall produced five instances, so treat
it as a class rather than five bugs.

**Symptom:** a gate/land stage goes red with a one-line verdict and there is nothing to read —
an empty run dir, an empty `salient`, a bare `exit 1`, or a number that is simply wrong.

**Cause, in the five shapes seen:**
1. **Captured and discarded.** `_smoke_once` ran the child with `capture_output=True` and kept
   only `summary.json`. When the child died *before* writing one, the console — the only record
   of why — was dropped. A lane was rejected on `smoke:arm64 FAIL` recorded as *exit 0, 1.6s,
   empty run dir*, for a boot that takes ~40s. Nothing to read, nobody to blame.
2. **A summary scraped by a marker search that misses.** `_tooling_once` looked for `FAIL`/`✗`
   in stdout; ruff and pyright print neither, and tracebacks go to stderr. TOOLING_FAIL blocked
   four lanes reading only "python tooling findings (exit 1)".
3. **Someone else's artifact in your evidence directory.** A unit test's `Run()` landed in the
   LIVE artifact tree, because `artifacts.ARTIFACT_ROOT` is resolved at *import* time and the
   test's `monkeypatch.setenv` therefore did nothing. Under the gate's `check-tooling` stage
   that planted a synthetic `verdict FAIL / classification BUGCHECK / duration_ms 0` where a
   `find` turns it up first. It was read as the real verdict twice, once in a status report.
4. **A shared artifact root with no owner field.** `VIBEOS_ARTIFACT_ROOT` defaults to
   `/tmp/vibeos-cli` for *every* worktree, so `ls -dt /tmp/vibeos-cli/*-build-arm64 | head -1`
   returns whichever lane started last. Another lane's green build was reported as this one's.
5. **A counter that cannot say zero.** `_ninja_did_work` matched `^\[\d+/\d+\]`, which also
   matches ninja's `[0/2] Re-checking globbed directories...` status pseudo-edge — so a pass
   whose output was literally `ninja: no work to do.` reported "relinked 1 target(s)". Always
   ≥1 breaks the signal in the direction that HIDES the condition it was written to expose.

**Fix:** persist the child's console *unconditionally*, next to the verdict, before deciding
anything (`smoke-console.log`, `check-tooling-console.log`, `difftest-console.log`). Give
"the child produced no verdict at all" its own classification (`SMOKE_NO_SUMMARY`) instead of
laundering it into a plain FAIL. Carry the real exit code into the `Stage`. Record the
`worktree` in every `summary.json`. And when a marker scan finds nothing, fall back to the
child's last non-empty line rather than printing a bare exit code.

**Rules of thumb.** A stage powerful enough to redden the whole train has no business
reporting without evidence. Test artifacts must never be able to reach a live artifact root —
pin it in an autouse fixture so no individual test can get it wrong. And when a status line
disagrees with the raw log, believe the log: every one of these was caught by reading the
output the summary was derived from.

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
- **Un-stubbing a global the harness defined as NULL is only HALF the fix; the other half is
  one per side.** msvcrt-difftest defined `MSVCRT_locale = 0` so mbcs.c would link, and that
  NULL is why `_setmbcp` was refused for months — its non-thread arm dereferences it. But that
  arm ends in `memcpy(MSVCRT_mbctype, MSVCRT_locale->mbcinfo->mbctype, …)`, so simply giving
  the shared global some storage would have had both sides installing into the same slot and
  each reading back what the other wrote: a row comparing a table against itself, green
  whatever either side did — and *more* dangerous than the refusal, because it would have
  looked like coverage. The discriminator is the one this file already states for
  `MSVCRT_mbctype` and `MSVCRT__sys_errlist`: **shared is right for state both sides must
  AGREE on (locale environment, kernel32 models), and wrong for anything the code under test
  WRITES.** Ask who writes it, not what it is.
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
- **…and the sibling that "prefer including the real header" CREATES: a HOST constant leaking
  into the oracle by inclusion.** msvcrt-difftest's refstub pulls glibc's `<stdlib.h>` for the
  narrow string family, and glibc defines `RAND_MAX` as `0x7FFFFFFF`. msvcrt's is `0x7FFF`.
  `rand()` ends in `(seed >> 16) & RAND_MAX`, so the REFERENCE returned 16-bit numbers where
  msvcrt returns 15-bit ones — and the divergence reads as a Rust bug until you check which
  side is wrong. Harder to find than the hand-typed kind precisely because nobody typed it:
  grepping the refstub for the constant finds nothing. **Any macro the host header and the
  Windows header both define, and define differently, is an oracle bug waiting for its first
  row** — the refstub already `#undef`s `LONG_MAX`/`ULONG_MAX` for exactly this reason, and
  that list is a starting point, not a complete one. When a new reference TU touches a
  `<limits.h>`- or `<stdlib.h>`-shaped constant, diff the two headers before trusting a row.
- **A shim that DISCARDS an input argument makes that argument ungradeable, and every row
  over it is vacuous.** msvcrt-difftest's `CreateFileW` had `(void)share;` and ignored
  `dwFlagsAndAttributes`. Both are inputs Win32 acts on and POSIX has no equivalent for, so
  dropping them looked reasonable — except that mapping `_SH_DENY*` onto `dwShareMode` is the
  *entire* reason `_fsopen`/`_wfsopen`/`_sopen_s`/`_wsopen_s` exist as separate exports, and
  `FILE_ATTRIBUTE_READONLY` is the *only* observable consequence of `_umask`. Delete either
  mapping and every row still opens the file, returns the same descriptor, and leaves the same
  bytes on disk. Two fixes, and which one applies differs: where the host CAN model the
  effect, model it (`FILE_ATTRIBUTE_READONLY` on create → `fchmod`); where it genuinely
  cannot, **record the argument and export a getter**, so the row compares what each side
  ASKED FOR and the mapping stops being invisible. The recording is one definition serving
  both sides, so read it immediately after each side's call, never once at the end. Same
  family as the `_wctomb_s_l` stub above, reached from the opposite direction: not an error
  that never happens, an argument that never matters.
- **A PROCESS-GLOBAL resource the substrate hands to both sides is shared substrate too —
  and the CRT is not where you redirect it.** msvcrt's `putchar`/`puts`/`getchar`/`gets`
  and eight kin take no FILE: they name `stdout` and `stdin`. Half the separation was
  already there (`stdout` is `&MSVCRT__iob[1]`, and the `_iob` rename wall gives each side
  its own array), which is exactly why it looked done — but `msvcrt_init_io` binds CRT fd 1
  from `GetStdHandle`, and the substrate answered with the harness's OWN fd 1. Both sides
  wrote to the same place, interleaved with each other and with the test's `println!`, and
  nothing could read either back. Eleven exports were ungradeable and read as trivially
  gradeable. **Give each side its own, and bind it BELOW the CRT**: a knob that points the
  three standard handles at caller-supplied host fds, called once per side immediately
  before that side's init, which is the only reader of `GetStdHandle`. The obvious
  alternative is the trap — `freopen(path, "w", stdout)` reaches the same state by CLOSING
  host fd 1, and the next `open` in the process hands fd 1 to a temp file, so the harness's
  own diagnostics vanish into it silently. Generalises past stdio: any singleton the
  environment supplies (a lock table, a TLS slot, a process-wide mode) is one definition
  serving both sides until you make it two.
- **A POSIX-backed shim that rejects a WINDOWS path separator deletes a whole search arm,
  on both sides, green.** msvcrt-difftest's `GetFileAttributesW` backs onto `stat`, and its
  wide→narrow converter passed `\` through untouched. That was invisible for four waves
  because nothing in file.c BUILDS a path — it forwards whatever its caller handed it. The
  first module that builds one exposed it instantly: `msvcrt_search_executable` joins a
  `PATH` entry to a program name with a literal `'\\'`, so every candidate it produced
  failed to `stat` and the entire `use_path` half of the function found nothing — the same
  nothing on both sides, so every row over it reported MATCH. **A shim must accept every
  input form the real API accepts, not just the ones the current caller happens to use**;
  a real `GetFileAttributesW` takes both separators. Same family as the discarded-argument
  entry above, arriving from the input side rather than the output side.
- **Compare an environment-allocated identifier by its ROLE, never by its value.** The
  process.c rows compare the `STARTUPINFOW` msvcrt hands CreateProcessW, and the three std
  handles came back `(4, 14, 8)` on the subject and `(5, 14, 9)` on the reference for
  behaviour that was identical and correct. Each side has its own CRT fd table caching OS
  handles the host allocated at whatever moment that side first opened something, and the
  subject runs first in every row, so its descriptors are always one lower. The contract
  under test is not the number — it is WHICH END OF THE PIPE the child gets, which is the
  only thing `_popen(cmd,"r")` and `_popen(cmd,"w")` disagree about. Resolve each recorded
  handle against *its own side's* `_get_osfhandle(0/1/2)` and compare the resulting labels;
  the raw comparison is a permanently red row that says nothing, and "fixing" it by dropping
  the field silently deletes the mode-string branch from the suite.
- **A code path that CALLS `exit` is a silent-green hazard, and the fix is in the
  substrate.** msvcrt's `_P_OVERLAY` spawn ends in `_exit(0)`: on the success path the
  harness process terminates, `cargo test` sees exit status 0, and the run is reported as a
  pass having compared nothing after that line. Every `_exec*` row therefore presets the
  spawn to FAIL — but "safe because nobody makes a mistake" is not safe when the failure
  mode is a green tick. `_exit` belongs to exit.c, i.e. the layer BELOW the code under test,
  so redirect it like any other environment entry point: to a terminator that prints what
  happened and leaves with a non-zero code. Leaving it plain has a second cost — it binds to
  the SUBJECT's `#[no_mangle]` copy, so the reference would be calling the code under test.

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

**"Both agree on that format" is only half the contract — the BUFFER contract can differ on a
format they spell identically.** Deciding glibc is close enough for the `ref_` TU covers the
CONVERSIONS and says nothing about what happens when the result does not fit, and msvcrt and
C99 disagree there completely: for a buffer of `len`, msvcrt writes `len` bytes with **no
terminator** and returns **-1**, while glibc writes `len-1` and a terminator and returns the
length it wanted. That is invisible until a caller PRINTS INTO A BUFFER AND THEN READS IT BACK,
which is exactly what `_ecvt`/`_fcvt` do — one byte of terminator becomes one missing digit, so
`_fcvt(0.0, 80)` answered 77 zeros against the subject's 78 and the row blamed the port. Take
the host's formatter if you must, but put msvcrt's buffer arithmetic on top of it, and write
that shim ONCE for both sides: this harness had two copies, and they had drifted in *opposite*
directions — the subject's asked glibc for `count + 1` so the NUL would have somewhere to go,
which lands it at `str[count]`, one byte past the caller's buffer, on every full-width
conversion into `_ecvt`'s 80-byte stack scratch. Two hand-written copies of one contract is the
hazard; the §9.73 objection to sharing does not apply to something NEITHER side owns.

**Same class, found by the same sweep, in the other direction: a rename wall can claim a symbol
the TU never defines.** The scrape that builds the wall is textual, so it cannot see a
preprocessor guard, and ReactOS forks Wine files by wrapping the replaced functions in
`#ifndef __REACTOS__`. Renaming a name that is defined-but-guarded-out is inert until something
CALLS it: math.c's `_scalb` is `return ldexp(num, power)`, `ldexp` sits in such a block, so the
wall rewrote the call to `ref_ldexp` and nothing defined it. The TU had already landed green —
the error only appears once a row references `_scalb`. Derive the guarded set from the source
and subtract it from the wall (it generalises: `string.c` and `process.c` carry these blocks
too), and pin the resulting name list so that adding or removing a block is a build-time
failure here rather than a link error in whichever test happens to reach it first.

**One more shape: a redirect that must be per-TU because a sibling reference TU DEFINES the
name.** `_gcvt_s` asks `_scprintf` how long a conversion would be, and in the `ref_` namespace
that is wcs.c's body, which walks straight into the `va_arg(*valist, …)` incoherence above and
segfaults — reproducibly, on the first row that ever drove `_gcvt_s`. It cannot be fixed in the
shared prelude, because wcs.c is a reference TU too and *defines* `_scprintf`, so a blanket
`#define` renames its definition on top of the harness's. It cannot be fixed with `-D` either:
the command line is consumed before the `-include` prelude, so the prelude's own rename
redefines it. A **second `-include`, after the prelude and on that one TU**, is the only place
such a redirect both fires and stays local.

**The flat instantiation is a RECIPE, not a wcs.c special case — reach for it whenever the
`&valist` idiom appears.** `file.c` needed the identical treatment for its sixteen
`v*printf` shells, and the second one cost far less than the first because the shape was
already known: make the TU's prelude a closure over the PREFIX rather than a string written
once, point the engine's six symbols (`pf_printf_a/w`, `create_positional_ctx_a/w`,
`arg_clbk_valist/positional`) at the flat instantiation and re-declare them after the
redefine (`-Werror=implicit-function-declaration` catches you otherwise, which is the
tripwire doing its job), and **re-emit the stage-5 protos header with its `__asm__` labels
retargeted** — an asm label is a string baked into the generated header, so a name the
`#define` wall cannot rename would otherwise be emitted under the FIRST instantiation's
prefix and collide. Two properties of the resulting TU are load-bearing and worth stating
where the rows can see them: its variadic entry points are broken by construction (the flat
`va_start` yields NULL, so `_open`'s `_O_CREAT` arm reads a pmode through it), so drive it
through non-variadic equivalents — `_sopen_s` + `_fdopen` where you would have used `fopen`.

**Do not read the "cannot be graded" verdict above more broadly than it is.** It is about
the `...` entry points, which need a real `va_start`. The `v*`-prefixed ones TAKE a
`va_list` and are perfectly gradeable in the flat instantiation, off a hand-packed 8-byte
slot buffer both sides read identically — the subject's `va_list` is `*mut u8` on every
target. The distinction is exactly whether the harness has to CREATE the argument area or
merely pass one, and it is the difference between 30 exports graded and 30 exports argued
about.

**And the end of that road: some code CANNOT be graded on the host at all — say so instead of
building a harness that appears to.** Both instantiations above are unusable for msvcrt's
variadic printf shells (`sprintf`, `_snprintf`, `_swprintf`, … — the `...` entry points), and
for opposite reasons. `pfflat_`'s flat `<stdarg.h>` makes `va_start` expand to
`(char *)&last_named + sizeof(last_named)`, which is exactly right on i386 cdecl and on
Windows and simply false on x86-64, where the variadic arguments arrive in registers and never
touch that address. `ref_` has a correct `va_start`, but then hits the incoherence above: the
shell hands `&valist` to the engine, which does `va_arg(*valist, T)` — an idiom that only
type-checks where a `va_list` IS a pointer. **There is no host process in which a real
`va_start` and msvcrt's engine agree**, so a differential test of these would be grading
something other than what it names, which is worse than no test (a vacuous branch at least
fails a perturbation; a mis-aimed one passes it). The reachable property is the one that
actually carries the risk — 30 near-identical three-line bodies where a wrong delegate or a
swapped `size`/`count` compiles, links, boots, and truncates only for small counts — and that
is checkable exactly, at the source: `tools/scan_varargs_shim_parity.py` compares each shell to
its wcs.c original, **deriving every tolerated rewrite from wcs.c's own one-line forwarders
rather than asserting it** (the shim cannot call the file-static `vswprintf_p_l_opt`, so it
names the exported wrappers; the scanner reads those wrappers and fails if they ever stop being
the pass-`options=0` identities that make the substitution sound). It also caught the shim
defining `_sprintf_p`/`_scwprintf_p`, which sit behind `#if _MSVCR_VER>=80` and exist at no
vintage this DLL builds. General rule: **when the oracle is impossible, move the assertion to
the property you can actually reach, and make the tool prove its own assumptions from the
source.**

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

**And it rewrites the reference's VALUES, wherever C leaves a constant unspecified.** Not a
throughput caveat this time — a wrong row. math.c returns its domain-error NaN from
`ret_nan(TRUE)`, whose body is `double x = 1.0; return (x - x) / (x - x);` — a compile-time
`0.0/0.0`, whose NaN sign C does not pin down. The host gcc that builds the reference folds it
to the x86 default NaN, `0xfff8000000000000`; the pinned llvm-mingw clang that actually builds
msvcrt.dll folds it to `0x7ff8000000000000`, and so does rustc. So `sqrt(-1.0)` "diverged" with
the SUBJECT matching the shipping binary and the REFERENCE the odd one out — and the obvious
fix, changing the Rust to match the reference, would have corrupted correct code to match a
host artifact. **Before treating a divergence as a port bug, ask whether the reference's
compiler is the shipping compiler**; where it is not, compile the disputed body with the tree's
own toolchain at the module's own flags and read the emitted constant (`… -S` and look for the
`.quad`). Then pin the SUBJECT to that measured value absolutely and let the reference differ
in that one bit only — which is stricter than comparing the two sides, not weaker, because it
now fails if the Rust ever stops agreeing with the compiler that ships it.

**A faithful port reproduces the C's infinite loops too — bound the rows, do not "fix" the
Rust.** msvcrt's `_fcvt_s` copy loop is `if ((ptr1 - buf) < stop) { if (size > 1) { *ptr2++ =
*ptr1++; size--; } } else ptr1++;` — the inner `if` has no `else`, so once `size` reaches 1
with digits still to copy neither pointer moves and the `while` spins forever. Any destination
smaller than the conversion hangs, in the C and in the Rust identically. That is correct
behaviour for a port and useless as a row, so it joins the other ungradeable domain in the same
family (inputs that overflow the C's own 80-byte scratch, after which both sides read back
their own stack garbage). **Skip such a case by PREDICATE, computed from the same formatter the
function uses, and print the count** — a hard-coded input list silently stops matching the C,
and a silent cap reads as "the family is graded" when the wide cases were the ones dropped.

**The host's TYPE MODEL is part of the miscompile, and it hides behind functions too small to
suspect.** `clock()` is four lines — `return (systime.QuadPart - init_time) / (TICKSPERSEC /
CLOCKS_PER_SEC);` — and BOTH of the things it names differ between msvcrt's headers and glibc's,
neither difference being a compile error. `clock_t` is `__msvcrt_long` (32 bits) on Windows and
64 on an LP64 host, which decides whether the division truncates — msvcrt's `clock()` wraps
after 24.86 days and an unpinned reference does not. `CLOCKS_PER_SEC` is 1000 in msvcrt's
`time.h` and 1000000 in glibc's, so an unpinned reference divides by 10 instead of 10000 and
answers in MICROseconds: every row off by exactly 1000x. Derive the constant (the
`gen_msvcrt_difftest_consts.py` probe, which `#undef`s glibc's first) and pin the type in the
TU's prelude — but note the ORDERING trap: a `#define clock_t __msvcrt_long` that is in scope
when glibc's `typedef __clock_t clock_t;` is parsed rewrites the TYPEDEF, which is a conflicting
definition rather than a rename. Pull `<time.h>` in the prelude yourself, while the TU's own
names are still hidden from it, and put the pin after.

**The same width trap on the TEST side, where nothing checks you at all.** A row that reads an
exported global through `__p__timezone()` must declare it `i32`, not `core::ffi::c_long` — that
is 64 bits here, so the read pulls in the next global as its high half. It does not crash and
it does not warn; it reports garbage, and the garbage differs between the two sides because
their globals are laid out independently, so it looks exactly like a port bug. `__msvcrt_long`
is 32 bits on every arch we build: spell it, and never reach for `c_long` in a harness.

### Exercising a branch is not testing it `[§9.75]`
A branch is only tested when its two sides can produce **different answers**. Planting a
non-zero locale handle made msvcrt take path 2 — and still proved nothing, because path 2 folded
through the same table. **Generalises to any harness with a "mode"** (endianness, alignment,
locale, SMP-vs-UP, a feature flag): the alternate-mode tables must be one copy shared by both
sides. Related: **a green from an unexercised branch is not a green**, and **a value-comparing
test cannot see a lost *tolerance*** `[§9.64]` — a pure refactor dropped NULL-behaves-as-`""`
and no generator that produces real NUL-terminated buffers could ever notice. Fix by making the
tolerance a first-class assertion (`assert_null_tolerant()`).

### A conformance score is not a defect count, and neither direction is self-explaining
Two independent traps, both of which have produced wrong conclusions in this tree.

**1. The denominator moves.** A kmtest that fails an early assertion often *bails* — skips a
setup step, returns early, never reaches the rest of the file. Fix the early branch and the
test suddenly runs hundreds of assertions it never reached before, so **total assertions is a
variable, not a constant**, and a rising *fail* count can mean "we got further in", not "we
broke something". The arm64 2026-08-01 re-seed is the worked example: of nine rows that moved,
eight had an unchanged total (a pure fail→pass move) and **`CmSecurity` went 389 → 910
assertions**, +521 of new ground, because it had been bailing early. Its failures dropped 89 → 4
in the same step — but had those 521 new assertions gone the other way, the row would have read
as a catastrophic regression while being an unambiguous improvement.
**So: always split total-assertion change from fail↔pass movement before calling a row.**
`(pass+fail)` unchanged = redistribution; `(pass+fail)` up = new coverage, and its failures are
newly *visible*, not newly *created*. The mirror image is already known — a falling count can
also be an improvement (amd64's `ExInterlocked` 662/40 → 180/0 when clang stopped deleting
`__except` blocks), which is why the re-seed guard *announces* count changes instead of
refusing them.

**2. Zero is the wrong reference point.** Many kmtests only pass cleanly on *real Windows*.
Stock ReactOS fails plenty of them too, so an absolute failure count is not a defect count, and
"N failures remain" says nothing about whether we are behind or ahead of upstream — **we may
well be exceeding it.** The ratchet's actual claim is narrow and worth stating precisely: *this
row got worse than it was on this tree*. It is a change detector against our own history, not a
distance-from-correct. Do not write "still N failures" as if it were technical debt, and do not
budget work against driving a row to zero, without an upstream reference measurement to say
zero is reachable. Cross-*arch* comparison is still fair game (it is the same codebase), which
is why "arm64 fails 9 where i386 fails 4" is a real signal while "arm64 fails 9" is not.

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

**A NEARLY-right match is worse than none, because it fails as a SKIP.** Both of `cfg_gate`'s
own shipped bugs were this, and neither raised anything:
- **Byte offsets into a `str`.** tree-sitter reports *byte* spans; slicing a Python `str` with
  them drifts one position per non-ASCII character already seen. A single em-dash in one file's
  doc comment shifted every name after it by two — `bvarraydup` came out `arraydup(b` — which
  then matched no entry in the lookup table, so **43 functions were silently skipped** instead
  of failing. Work in `bytes` and decode spans; and assert the shape you expect (a name that is
  not `str.isidentifier()` is now a hard error), because the *only* thing standing between a
  corrupt extraction and a silent no-op is a check that the extraction looks right.
- **Substring guard names.** `"ifdef HAVE_LDAP"` occurs inside `"ifdef HAVE_LDAP_PARSE_REFERENCE"`,
  so a `str.count` saw two guarded regions where there was one and refused two ordinary
  functions as too-complex-to-transform — again leaving them silently untransformed. Anchor
  identifier matches with `\b`, and sort an alternation longest-first so it cannot prefix-match.

**The tell for both: a tool that reports "skipped" rather than "failed" is making a claim it
cannot support.** Count what you *transformed* and reconcile it against what you *found*.

### A symbolicator resolves against SOME binary — make sure it is the one that booted
**Symptom:** a bugcheck PC resolves to a function that makes no sense at the crash site, or to a
constant (`KTRAP_FRAME_LENGTH+13864` — an ABSOLUTE symbol 13KB away), or the tool declines
entirely. **Two independent causes, and both lie quietly** because *some* symbol always sorts
below any address:

1. **Wrong binary.** `ntkrnlmp.exe` is a SEPARATE image from `ntoskrnl.exe` with its own layout
   (9847 vs 9771 symbols on arm64), and every `--smp N` boot runs it — so an index built only
   from the UP kernel misnames MP crashes. Build a per-flavor index and pick by what freeldr
   announced (`[FLDR] load …\ntkrnlmp.exe`). A missing MP index must be a HARD ERROR, never a
   fallback to the UP one: a wrong symbol is worse than no symbol because it reads like a right
   one.
2. **Wrong relocation.** The index holds file VAs (`RVA + PE ImageBase`, `0x400000` here), so a
   runtime PC needs `runtime_base - ImageBase` subtracted, not `runtime_base`. Getting this
   wrong pushes every lookup below the first real function.

**Don't guess the base at all when the banner already resolved it.** The bugcheck prints
`***   P2 = ntoskrnl.exe+0x3788` — that offset IS the RVA, so the index address is just
`RVA + ImageBase`: no runtime base, no per-arch reloc table, immune to where the image landed.
vibeos had hardcoded relocs for i386 only and a comment claiming arm64/amd64's base "cannot be
recovered from a saved log" — it was on the next line. Until that was used, every arm64 bugcheck
declined, and the one that mattered needed hand-computing to reach
`KiSynchronousExceptionEntry+4`.

### An oracle needs an oracle `[§9.122]`
A scanner printing `0 findings` is asserting one of two very different things: "I looked and the
tree is clean" or "I did not look." **This is not a fail-loud problem** — the scanner ran fine.
`tools/test_scanner_controls.py` is a declarative table giving every gate scanner a **positive**
control (a synthetic defect it must find) and a **negative** control (clean code it must not
flag). **Validate the harness by BREAKING the scanner** — restoring a pre-fix scanner must make
the harness fail. Two instances of the very class turned up while building it: the harness
itself was a liar on its first run (the positive assertion accepted *any* non-empty output), and
two scanners take FILES, not directories, so handed a directory they silently found nothing.

**An oracle built on a parse must check that the parse SUCCEEDED, or it reports the parser's
debris as findings about your code.** `preamble_audit` mines top-level declarations, resting on
"a top-level declaration is by construction never inside a function body" — true only while the
parse works. tree-sitter recovers from a parameter list it cannot read by HOISTING the function
body's statements to the root, so `WSAttemptAutodialAddr(IN CONST SOCKADDR FAR *Name)` made the
locals `Ip`, `NetBios`, `AutodialAddress` (and the parameter `Name`) look like file-scope state
the translation had dropped. All of them were present, as locals: five confident, specific,
wrong findings — which is how an oracle stops being read, a worse outcome than not having it.
Two fixes, and you want both: **reuse the parser normalizer the rest of the pipeline already
has** (`FAR`/`NEAR` are `#define`d to nothing and are the usual trigger — the same macro once
dropped `inet_addr` from a translation plan entirely), and then **refuse to mine a root that
still holds a top-level ERROR**, reporting "not audited" instead of guessing. Note `has_error`
is useless as that gate: it is true for nearly every real Windows TU because of macros inside
bodies, which hoist nothing. A **top-level ERROR node** is the precise signal.

**Nothing in the pipeline compares an `extern` DECL against the `#[no_mangle]` DEF of the same
symbol.** rustc checks a call against the declaration in scope; the definition lives in another
module and the two meet only in the linker, which compares nothing but the name. Both files
compile, both are internally consistent, and the caller hands over a different object than the
callee reads. `verify_signatures` and `verify_c_signatures` cannot help — both need a C
prototype, so a fully-translated module has no oracle at all. Three live instances: fastfat's
`FatUnpinRepinnedBcbs` (volinfo declared `*mut PIRP_CONTEXT` against cachesup's
`*mut IRP_CONTEXT`, so the callee read a pointer-to-pointer as the struct), fastfat's
`FatGetNextMcbEntry` (`*mut u64` for `OUT PVBO`, an 8-byte store into deviosup's 4-byte local
on every non-cached FAT read/write), and freeldr's `TuiDisplayMenu` (an 11th parameter the
definition and `tui.h` do not have, shifting every argument from `MenuItemList` on by one slot).
`scan_extern_decl_drift.py` gates the class. **Building it took five rounds and the count went
2682 → 113 → 39 → 10 → 8, every reduction a defect in the ORACLE rather than a suppressed
finding** — and that ratio is the lesson: an alias spelled differently across bindgen islands is
not ambiguous unless its resolved DEPTH differs; a path-qualified alias must reduce to its last
segment; a `use X as Y` rename is an alias; `#[link_name = "X"] fn Y()` declares X, so pairing
by Rust identifier compares unrelated functions; and only a decl that is CALLED governs an ABI
(ntoskrnl's SSDT declares every syscall `fn(_: usize, ..)` to count stack slots, and those
decls are correct on purpose). When a new scanner's first run produces thousands of findings,
the prior should be that the scanner is wrong, not the tree.

**Depth was only half of it: the tool then had to learn WIDTH, and that took three more oracle
bugs.** Comparing indirection depth catches a missing `*`; it cannot see `*mut u64` against
`*mut VBO`, which is one pointer on both sides and is the `FatGetNextMcbEntry` bug the tool was
built for. Resolving the base type's width behind the pointer took the count 8 → 138 → 97, and
the three reductions were all oracle defects again: an identifier that is neither an alias nor
an aggregate we have SEEN is **unknown, not depth 0** (generated bindgen islands live in
`build-<arch>/`, outside the sweep, which made mscms' `PPROFILEHEADER` a false positive); a
**function pointer is one indirection** however spelled, and `Option<T>` is `T`, or every
callback declared `*mut c_void` reads as drift against the typedef naming that same pointer;
and a pointer spelled as a pointer-**width integer** (`isize`, `ULONG_PTR`) is the same
register holding the same address, though `u32` against `HWND` is not and must still report.
Report by-value **widening** only — a callee reading *fewer* bits than the caller passed is
well-defined on all three ABIs and is how `BOOL` against `BOOLEAN` is spelled tree-wide.

**A `type X = path::X` re-export normalises to the cycle `X -> X`, and a cycle resolves to
"undecidable".** One `pub type c_void = core::ffi::c_void;` in `ntoskrnl-rust` made `*mut
c_void` unresolvable, and with it `PVOID` and the ~60 types spelled through it — most of the
tree. **Failure mode: silence.** No error, no finding, a clean report for the wrong reason, and
no negative control can catch it because nothing is flagged. The control has to be a *positive*
one asserting a specific finding that is only reachable if the alias resolves. Drop an alias
whose RHS normalises to its own name. The same pass needs memoising: resolution fans out over
every RHS a name is given anywhere, and unmemoised it went from 0.2s to not terminating.

**A COMMENT is a child of the argument list, so positional argument indexing is off by one
wherever anyone writes one.** `scan_ob_refcount` locates `ObReferenceObjectByHandle`'s object
by fixed index 4 and `scan_irql_balance` locates `KeRaiseIrql`'s `OldIrql` as the last
argument; both built their argument list by dropping only `(`, `)` and `,` from the
tree-sitter `arguments` node. fastfat's `ObReferenceObjectByHandle(h, 0, ty, 1, /* UserMode */
&mut fo, …)` therefore tracked the *comment's own text* as the object and reported a leak of a
variable named `UserMode` on two paths that both dereference correctly. The same shift is
silent in the other direction: a comment in `KeRaiseIrql(/* NewIrql */ DISPATCH_LEVEL, …)`
makes the level read as a non-constant, the acquire is not recorded, and the scanner goes
**blind** with no finding to notice. Filter `line_comment`/`block_comment`/`comment` alongside
the punctuation, and note which direction each scanner fails in when you write the control —
an FP fixture cannot catch blindness, only the positive control can.

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

### An automated repair loop cannot converge on an error reported at a MACRO's definition

**Symptom.** A bounded repair pass that inserts casts runs to its pass limit and leaves a
`macro_rules!` body wearing a stack of them:

```rust
macro_rules! ListView_GetItemCount {
    ($h:expr) => { (((((((((((( SendMessageW($h, LVM_GETITEMCOUNT, 0, 0) as c_int)
        as u32) as i32) as u32) as i32) as u32) as i32) as u32) as i32) as u32) as i32) as u32) as i32 };
}
```

**Cause.** rustc attributes a macro expansion's type error to the macro's DEFINITION span, not
to the call site that caused it. A repair pass keyed on `file:line` therefore edits the body,
the next pass sees the error again from a *different* call site, and appends another cast. It
can never converge, because the macro has one body and the call sites want different types —
and every added cast is a no-op on the width, so nothing else notices. `crate-repair` reached
eleven nested casts on netcfgx before hitting its pass limit, and separately appended a
`.cast()` inside the crate's `w!()` wide-literal macro, which left the pointee type
unconstrained at every use.

**Fix.** A cast belongs at the CALL SITE. Treat a `macro_rules!` body as off-limits to any
automated span-keyed rewrite: when the reported span is inside one, the finding is about the
expansion, and the tool has no way from that span to reach the code that has to change. Until
`crate-repair` refuses those spans, diff the prelude after every run — the edits are silent
and each one compiles.

Three more rules for any hand-rolled span applier, all of them measured on shell32's
CDefaultContextMenu grind, and all of them silent (they produce a file that no longer parses,
so the NEXT `cargo check` reports two syntax errors and hides the other two hundred):

- **Byte offsets go stale the moment a pass writes.** Running two different appliers against
  one `cargo check --message-format=json` corrupts the second's edits. Re-run the check
  between every pass.
- **Reject OVERLAPPING spans.** Applying end-to-start is necessary but not sufficient: two
  diagnostics can cover nested ranges, and the inner edit then lands inside the outer's
  replacement text.
- **Only rustc's own `MachineApplicable` suggestions are safe to bulk-apply.** A
  home-grown "wrap the span in `(expr) as T`" rule looks equivalent and is not: rustc's
  primary span for a mismatched *call* covers the callee, so wrapping produced
  `(SHUnicodeToAnsi) as i32((*mut pEntry).Verb, …)`. Convert rustc's
  `.try_into().unwrap()` suggestions to `as` casts (C truncates silently; the suggestion
  inserts a panic), but take the SPAN from a suggestion rustc actually authored. Those
  suggestions are ZERO-WIDTH APPENDS at the point a `.method()` could legally attach, which is
  exactly what makes ` as T` precedence-safe at that byte — the position is the useful part,
  not the text.
- **An append after an `as`-cast is a PARSE error, not a type fix.** `x as REFIID` + `.cast()`
  gives `x as REFIID.cast()` → "cast cannot be followed by a method call". Any applier that
  appends must reject a span whose text ends in `as <Type>`.
- **The type pair is not always on the primary span.** For
  "arguments to this function are incorrect" rustc puts `expected X, found Y` on a `note`
  CHILD whose single span covers just the offending ARGUMENT; the primary span is the whole
  call. An applier that only reads primary spans silently skips most real cases — on this
  grind it saw 0 of 19 crate-split mismatches until it walked the children.
- **Casting is only ever right for a NOMINAL split.** `.cast()` is the whole fix when
  `shellcom`, the island, `com-sys` and `ntdll-sys` each emit their own `IDropTarget`/`_GUID`
  — match on the two types sharing a final path segment (modulo leading underscores) and
  nothing else. The two exclusions that mattered here were both real bugs a cast would have
  buried: `StrIsNullOrEmpty` is C++-OVERLOADED (LPCSTR and LPCWSTR, shellutils.h:474-475) and
  the translation picked the wide one for an ANSI field, so a `*const i8` → `*const u16` cast
  would have shipped an ANSI string read as UTF-16; and a helper typed `LPCSTR` that is really
  a width test on any pointer wanted to be generic, not cast at its callers.

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

### A derived profile is a MODEL of the build, and every divergence is silent
`profile_derive` reads a module's CMakeLists + compile-db and emits the bindgen profile. It is
a *model* of how those TUs compile, and four separate places where the model quietly differed
from the build all surfaced in one batch (wldap32, rasadhlp). The shape recurs, so check for
it whenever a derived profile misbehaves — and note that three of the four fail LOUDLY at
bindgen time only because a header happened to be missing; the same divergence with a header
present is a silently-wrong island.

- **Our search path is not the generator's `-I` set.** Any dir listed in `_SDK_SEARCH` was
  omitted from `EXTRA_INC` on the assumption `gen_driver_types.sh` already passed it. For
  `sdk/include/wine` and `sdk/include/winrt` that is false. wldap32's only `config.h` lives in
  the former, so the derived profile could not generate at all. samlib had hit the same gap
  from the *other* direction and was patched per-instance — a hand-added `LATE_INC` found by
  header bisection. **A per-instance include fix is a bug report against the deriver.** The two
  sets are now separate constants (`_GENERATOR_BASE_INC`), and must be kept in sync with the
  bindgen invocation.
- **`build-<arch>/sdk/...` from the compile db must never be emitted.** The generator passes
  those as per-arch `${BUILD_DIR}`; echoing back the observed `build-i386/...` bakes one arch's
  *generated* headers into a 3-arch generator — the cross-arch leak AGENTS.md forbids, and it
  reads as correct because i386 is what we derive from.
- **A guarded include was promoted to an unconditional one.** `scan_preamble` dropped the
  `#ifdef`/`#endif` and KEPT the `#include` between them. wldap32 wraps OpenLDAP under
  `#ifdef HAVE_LDAP_H`; ReactOS does not vendor it. **Wine's `config.h` is an authoritative
  build-configuration oracle** — autoconf writes the off cases as `/* #undef HAVE_LDAP_H */`,
  an assertion about *this* build, not a maybe — so guards on its 330 undefs are decidable and
  the dead arm is dropped. Undecidable guards still keep both arms and warn.
- **Check what the module COMPILES TO before paying to translate it.** wldap32 reads as an
  8,625-line LDAP client. In this tree every entry point compiles to
  `return WLDAP32_LDAP_NOT_SUPPORTED;` — 181 `#ifdef HAVE_LDAP` blocks across 19 of its 20 TUs
  are disabled, because ReactOS does not vendor OpenLDAP. It is a **stub DLL**. Translating it
  bought 162 functions of the arm the compiler throws away, emitted UNCONDITIONALLY (so
  `ldap_addA` would marshal instead of returning NOT_SUPPORTED) and calling OpenLDAP symbols
  that do not exist here (so it could not link). LOC is not a proxy for how much of a module is
  real. `profile_derive` now emits **`MOSTLY_DEAD_BODIES`** before payment for any module with
  ≥15% of its compiled lines inside a decidably-false `#ifdef`, naming the macro and block
  count. Note the dead-region seal already *tolerated* omissions inside such a block — a
  tolerance is not a warning, and the two are easy to confuse. The salvage is to gate the
  translated bodies behind the same `#[cfg(feature = ...)]` the C uses, which keeps the paid
  artifact and makes the Rust strictly better than the C.
- **There are TWO decidable gate classes, and knowing only wine's was worth 4,614 dead lines.**
  `config.h`'s `/* #undef HAVE_LDAP */` settles `#ifdef HAVE_LDAP`; a module's own
  `-DNAME=<int>` settles `#if NAME >= 100` just as completely, and nothing was reading the
  *value*. msvcrt compiles at `-D_MSVCR_VER=0`: `concurrency.c` puts all 3,839 of its lines
  inside `#if _MSVCR_VER >= 100` and `handler4.c` 775 inside
  `defined(__x86_64__) && _MSVCR_VER>=140`. **Both build to a 396-byte object with ZERO
  defined symbols**, and both read as prime targets — concurrency.c is the largest untranslated
  file in the module. `_eval_int_guard` now settles these; short-circuit is what makes it work
  on handler4.c, where one decidably-false conjunct kills the block while `defined(__x86_64__)`
  stays forever unknown. It refuses mixed `&&`/`||` precedence rather than risk the inverse
  error, which is the one that hurts: reporting LIVE code dead sends someone to delete a body
  the compiler really is building.
- **The finding has to be PER-TU. A module-wide percentage hides exactly the file you were
  about to pay for.** Those two msvcrt files are 10.6% of the module's compiled lines — *under*
  the 15% `MOSTLY_DEAD_BODIES` threshold — so the module-wide warning stayed silent on 4,614
  lines of guaranteed-dead code. You choose a FILE to translate, not a module, so `DEAD_TU`
  fires at **ERROR** for any single compiled TU that is ≥90% dead. `DEAD_SOURCE` cannot cover
  this: these files ARE in the CMake source list and do get compiled — they just preprocess to
  nothing. Confirm any hit the same way it was found, with `nm` on the `.obj`.
- **`add_pch()`'s SKIP list is load-bearing.** Its third argument names the TUs that must not
  get the PCH, and they are skipped *precisely because they do not compile with it*. wldap32's
  `ber.c` includes `<winldap.h>` while `precomp.h` pulls `"winldap_private.h"`; the two declare
  ~30 `ldap_*` prototypes and `LDAPControlA`/`LDAPModA`/`LDAPSortKeyA`/`LDAPAPIInfoA` (+W)
  incompatibly. Force-including the PCH into `ber.c`'s wrapper reproduced that conflict exactly
  — and the conflict is also *why* the module needs two islands, so the CMakeLists was already
  telling you where the island boundary goes.

### A parse/derive failure that silently falls back `[§9.72, §9.80]`
- **A catastrophic-backtracking regex hung `postprocess`**, and the hang looked like the paid
  translate step. **The damage is the workaround:** the operator kills the wedged run and
  hand-assembles the output, bypassing the entire post-processor. **A hang is a correctness
  bug** — it trains an operator into a habit that skips your checks.
- **`abi_enforce` derived the right convention and then discarded it**; `_derive_convention`
  fell back to cdecl **on a comment**, mis-deriving 17 genuine WINAPI methods; and
  `CALLBACK`/`PASCAL` were missing from the keyword list. A silent fallback to a plausible
  default is the worst possible failure mode.
- **The MODEL default fell back to the expensive backend, and only the docs said otherwise.**
  Symptom: paid runs billed Sonnet at 12.2× DeepSeek while every doc and estimate assumed the
  cheap lane. Cause: `-m` had no default, so `resolve_model(None)` went to
  `DEFAULT_MODELS[get_backend()]`, `get_backend()` returned `"claude"` whenever
  `ANTHROPIC_API_KEY` was set, and `DEFAULT_MODELS` **had no `"deepseek"` key at all** — the
  house model was unreachable except by typing `-m deepseek`, which the docs recommended and
  nothing enforced. 69 Sonnet runs to 1 DeepSeek over four days; 382 calls at $3.81 where
  DeepSeek was $0.31. Fix: `get_backend()` prefers `DEEPSEEK_API_KEY`, and `DEFAULT_MODELS`
  carries every backend it can return. **The pre-payment banner printed the model AND its
  per-1M price on all 382 calls** — so this is not a visibility bug, and no amount of "read the
  banner" discipline would have caught it. A documented flag that nothing enforces is a
  deferred mechanism, and the drift is silent because both models return working Rust.
  Generalize: when a default routes to a *provider*, assert the resolved VALUE in a test —
  `default == DEFAULT_MODELS[get_backend()]` is a tautology that passes for any wrong answer.
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
- **A harness stub that returns a CONSTANT reports on the harness, not the OS.**
  `KmtAreInterruptsEnabled()` (kmt_test_kernel.h) reads EFLAGS.IF under
  `#if _M_IX86 || _M_AMD64` and falls through to a bare `return FALSE` for every other arch.
  On arm64 that was ~95 assertion failures per sweep — ExSingleList 39, KeSpinLock 36,
  ExDoubleList 20 — none of which any kernel change could ever have cleared. Worse, the
  ratchet had *baselined* them, so they were recorded as expected OS debt. In the results
  file a stub is indistinguishable from a real measurement of a broken kernel. **Symptom:**
  one assertion failing identically on every arch that shares an `#else`, inside tests that
  otherwise pass. **Fix:** implement it (arm64: read DAIF bit 7), or make the stub `skip()`
  so it cannot masquerade as a measurement. Its `#pragma message` warning had been firing
  on every arm64 build the whole time.
- **Half-disabling a debug facility leaves its two halves disagreeing — and the disagreement
  outlives the reason.** A checked build does two separable things to a spinlock: it STAMPS
  `KeGetCurrentThread() | 1` into the lock while held (observable behaviour — `KeTestSpinLock`
  reads it, and IoInterrupt asserts on it), and it BUGCHECKS on a mismatched acquire/release
  pair. This tree narrowed *both* to `DBG && !_USE_DUMMY_PSEH && !_USE_PSEH_CLANG` to stop the
  bugchecks during i386 bring-up (5c8505479a3, 15434cd8dea). Two consequences, neither
  intended:
  1. the build reports `KmtIsCheckedBuild = TRUE` while a held lock reads free, so every
     assertion that inspects one fails on a premise the kernel no longer honours;
  2. `KeTryToAcquireSpinLockAtDpcLevel` (ke/spinlock.c) stamps under **plain `#if DBG`** — it
     was never part of the narrowing — so it stamps a lock that `KxReleaseSpinLock` then never
     clears. `KdbpAcquireLock` (kd/kdio.c) spins on `while (!KeTestSpinLock(SpinLock))` before
     try-acquiring, so its *second* acquire of the same lock hangs forever. Dormant only
     because KDBG is off by default.
  **Rule: when you gate off a debug check to unblock a boot, gate off the CHECK, not the state
  it checks.** Give the two halves separate macros (here `KI_VALIDATE_SPINLOCK_OWNER` vs
  `KI_TRACK_SPINLOCK_OWNER`) so the next person can tell which one they are turning off, and
  so a partial disable cannot silently desynchronise two funnels that must agree.
  **Sequel (2026-08-01): restoring the stamp exposed a MISSING COMBINATION in the test.**
  `IoInterrupt`'s `CheckSpinLock` branches `if (KmtIsMultiProcessorBuild) … else if
  (KmtIsCheckedBuild) …`, so the MP branch — which demands `*Lock == 1` **exactly** — is
  reached first and the checked branch is unreachable on an MP kernel. But
  `KI_TRACK_SPINLOCK_OWNER` is gated on plain `#if DBG`, independent of `CONFIG_SMP`, so a
  checked MP kernel stamps `thread|1` and cannot pass. Upstream ReactOS's own DBG+SMP build
  fails it identically. **Symptom shape worth recognising: a test whose expectation is selected
  by build flavour can be missing a COMBINATION of flavours, and then a kernel that behaves
  identically under both flavours "regresses" purely by crossing the branch boundary.** Before
  filing an MP regression, check whether the UP and MP kernels actually wrote different values
  — here they wrote the same one.
- **A baseline seeded before a behavioural fix invents a cross-arch pattern that does not
  exist.** Same incident: today's post-stamp arm64 numbers compared against x86 baselines
  seeded four days *before* the stamp landed (`cfdff72b41d`) read as "x86 MP passes, arm64 MP
  fails" — an arch difference, and a tempting one, since arm64 is where MP is least mature. It
  was purely the seed dates; the per-arch behaviour is identical. **A baseline is a measurement
  with a timestamp, not a constant.** When a delta looks arch-specific, diff the baselines'
  commit dates against the fix's date before theorising, and re-seed every arch that predates
  it — a stale ratchet floor will swing in BOTH directions with no source change behind either.

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
- **The OBJECT'S symbol table is the authority on what a TU exports — the source is not.**
  Diff the C object against the Rust before you flip: `llvm-nm --defined-only <tu>.c.obj`,
  keep only the GLOBAL (uppercase) entries, and require the Rust to define every one. Three
  real bugs in one file (msvcrt math.c), none visible in the C:
  **(1)** the translation named two exports `_rotl64`/`_rotr64` while the C spells them
  `MSVCRT__rotl64`/`MSVCRT__rotr64` — Rust then defines symbols nothing imports, the C bodies
  stay live, and the "flip" is a no-op that builds and boots clean;
  **(2)** `CMakeLists.txt` carries `-Dmath_error=_invoke_matherr`, so the symbol the file must
  define is one the source never spells — grep the module's CMakeLists for `-D<name>=` before
  trusting any identifier;
  **(3)** the same `-D` list renames `asm_sqrt`/`asm_sqrtf`/`x87_asin` to their libm
  counterparts, which rewrites every CALL SITE but *not* the `__ASM_GLOBAL_FUNC` label —
  `#name` stringifies before macro expansion — leaving those three defined-but-dead. Rust must
  follow the call sites, not the stub. Symbols nothing references cannot drag the C object in,
  so they need no Rust definition; check with `llvm-nm --undefined-only` across the module's
  objects plus the `.spec` rather than assuming.
- **`#[export_name]` STACKS with i386 stdcall decoration, it does not replace it.** Give the
  UNDECORATED C name and declare the fn `extern "stdcall"`; rustc prepends the `_` and appends
  the `@<arg bytes>` itself. Writing the decorated name is the intuitive move and produces
  `___adj_fdiv_m16i@4@4` — an underscore and an `@4` too many, so the symbol binds to nothing
  and the flip is a silent no-op, exactly the failure the attribute gets reached for to avoid.
  (`#[no_mangle]` is not the escape either: on a *function* it emits the name verbatim and
  drops both halves. On a *static* it is right — no stdcall to decorate, and the leading
  underscore is still applied, which is what `MSVCRT__adjust_fdiv` needs to satisfy its
  `.spec` export.) Note also that a `short` parameter still takes a full 4-byte slot.
  **None of this is visible to `cargo check`, which never codegens** — an `#[export_name]`
  never becomes a symbol and a `global_asm!` block is never assembled, so only a real
  `cargo build` plus `llvm-nm` over the archive can tell you. That is what the `cohab:<arch>`
  gate stage automates.
- **Full coverage is the flip's PRECONDITION, not a nicety.** Leave one *referenced* symbol
  undefined and lld pulls the C object to satisfy it — and that object also defines every other
  symbol in the TU, so the flip collapses into duplicate definitions (or, with
  `--allow-multiple-definition`, into member-order roulette). This is why whole-file is the unit
  for msvcrt, and why the hand-written assembly stubs (`_CI*`, `_ftol`) had to be transcribed
  into `global_asm!` rather than left to the C.
  **`tools/verify_cohab_coverage.py`** does all of the above mechanically: for every `pub mod`
  in a cohab family's `mod.rs` it diffs the C object's referenced global exports against
  everything that links ahead of the static library — the Rust archive PLUS the module's own
  non-static objects. Compare against *binaries*, not Rust source: a source-level version
  reported 81 findings that were all false, because two provider categories are invisible from
  the twin file (a sibling in the same archive, and a C shim TU in the module target). It
  refuses a stale archive rather than judging one — run against archives three days old it
  produced 669 findings claiming errno.c, file.c and time.c were barely flipped at all, and a
  wrong verdict is indistinguishable from a right one at a glance. It therefore needs a build,
  so it is a post-build stage and not a `vibeos-cli check` lint: `gate` launches `cohab:<arch>`
  the moment that arch's build lands, classifies a gap `COHAB_FAIL`, and blocks the push stamp.
  It runs only when both `--no-lint` and `--no-build` are unset — it can judge only artifacts
  that run produced. Its own controls are `tools/test_verify_cohab_coverage.py`: it cannot join
  `test_scanner_controls.py` (which drives source-only `check` lints from fixture dirs), and a
  gate stage whose healthy output is `0 findings` needs an oracle like any other.
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
- **A header that DEFINES a variable loses its definer when you flip the last TU that included
  it.** `nsp_dns.h` does not declare its six service GUIDs, it defines them —
  `const GUID DECLSPEC_SELECTANY HostnameGuid = SVCID_HOSTNAME;` — COMDAT, so every including
  TU emits a foldable copy. bindgen renders them as `extern static`, which is right while some
  C still includes the header and wrong the moment none does: rasadhlp's `winsock.c` was that
  TU, and flipping it left `undefined symbol: _HostnameGuid` at link. Strip them from the
  island (`BLOCKLIST_VARS=1`) and define them in the crate. Grep a flipped module's headers for
  `DECLSPEC_SELECTANY` / `__declspec(selectany)` / `static const` at file scope *before* the
  link tells you. Build the values from the header's own macros rather than pasting bytes, so
  they stay checkable.
- **A `static __inline` FUNCTION in a header has the same problem, and no `-sys` island shows
  it.** strsafe.h's whole bounded-string family is `STRSAFEAPI` = `static __inline HRESULT
  WINAPI`, so it has internal linkage: every C caller inlines its own copy and NO object file
  in the tree defines the symbol. bindgen emits nothing for it either (it emits declarations,
  and there is no external one), so the crate's prelude ends up with a plausible `extern "C"`
  block that nobody satisfies — syssetup's said "provided by the strsafe static lib the module
  already links", which the module does not link and never did. It links until the last C TU is
  flipped, then `undefined symbol: _StringCchCopyW`. **Port the header body into the crate**
  (`src/<module>/strsafe.rs`, the same treatment shell32 gave its `shellutils.h` inlines)
  rather than re-homing a wrapper in a surviving .c — a wrapper only moves the problem to the
  next flip. Grep a flipping module's headers for `static __inline` / `__CRT_INLINE` /
  `FORCEINLINE` before the link tells you. Faithfulness matters here: these APIs TRUNCATE and
  return `STRSAFE_E_INSUFFICIENT_BUFFER`, so the port must not reach for slice indexing.
  If one of them is VARIADIC (`StringCchPrintfW`), its entry point still has to be the
  `rust_varargs_shim.c` trampoline — stable Rust cannot define a C-variadic — with the body in
  a fixed-arity `*_v` twin.
- **An `extern` in the prelude for a name the crate itself defines is an undefined symbol
  waiting for the flip.** `BLOCKLIST_FUNCS_FROM_CRATE` removes the crate's own FUNCTIONS from
  the island; it does not touch VARIABLES, so a `static mut` the translation owns
  (`hDllInstance`, `AdminInfo`, `hSysSetupInf`) is still declared `extern` there and the Rust
  definition, unmangled by default, exports nothing. `cargo check` is happy — both halves
  resolve inside the crate — and only the cold link says so. `#[no_mangle]` on the static is
  the fix; the same reasoning as `#[export_name]` above, and the reason a data global is the
  easy half (no stdcall decoration to get wrong).

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

### A C++ `static_cast` back to a Rust-owned class is already wrong, and it links
**Symptom.** None. It compiles, links, and runs — until the class's C++ half leaves CMake, at
which point you get an undefined symbol and discover the bug on the way past.

**Cause.** Once a class is Rust-owned, the object's layout is Rust's. A C++ TU that still holds
a `CFoo *` and calls a **non-virtual** method on it is doing two wrong things at once: the
`static_cast<CFoo *>(pInterface)` applies the *C++* class's base-subobject offset to a
Rust-laid-out object, and the method it reaches is the DEAD C++ copy that the cohab left
compiled but unactivated. Virtual calls are fine — those go through the vtable the Rust object
actually carries — so the bug hides in exactly the methods that don't. `CRecycleBinFolderViewCB`
held a `CRecycleBin *` this way and called `ParseRecycleBinPath` on it for as long as the cohab
existed.

**Fix.** Keep the interface pointer you were *given* (`IShellFolder2 *`, not `CFoo *`) and reach
the method through a `_RustBridge` that does its own `offset_of!` against the Rust struct. Grep
a flip's remaining C++ for `static_cast<`/`reinterpret_cast<` naming a Rust-owned class before
you believe the flip is closure-complete — the retained closure will not show it, because a
`static_cast` demands no symbol.

### A cohab shim declared VOID SWALLOWS the real function's return value

**Symptom.** Callers branch on a value the callee never delivered. Nothing warns: the C
compiles, Rust compiles, the link resolves.

**Cause (fastfat `FatAcquireExclusiveVcb`, all three arches).** `fatprocs.h` defines it as a
macro over `FatAcquireExclusiveVcb_Real`, which returns `FINISHED` (`typedef BOOLEAN`) —
"did I get the resource?". `rust_cohab_stub.c` forward-declared `_Real` as **`VOID`** and so
the wrapper shim was `VOID` too; it called `_Real` and returned nothing. Every Rust caller
that declared a return type was reading whatever survived in the return register across the
shim's epilogue — right only by accident of codegen, and wrong the moment the register is
touched. Live readers were `flush.rs`, `fileinfo.rs` and `volinfo.rs`, i.e. flush,
set-file-info and volume-info.

**The tell, and it is cheap to look for:** the crate disagreed with ITSELF. One symbol,
eight `extern` declarations, four different return types (`void` ×4, `BOOLEAN` ×3,
`NTSTATUS`, `i32`). `grep -rn "fn <Name>" <crate>/src` takes seconds and any disagreement is
a bug in all but one of the sites. `volinfo`'s `i32` read four bytes of a one-byte return.

**Fix.** Correct the forward declaration FIRST — a wrong prototype for the callee makes
forwarding impossible in principle, so the shim's signature is a symptom, not the root. Then
make the shim `return` the value, then normalize every Rust decl to the one true type.

**Generalization.** A cohab shim's signature is part of the ABI contract and nothing checks
it. When a shim exists only to fill in a macro, its return type must match the macro's
expansion, not whatever the shim author happened to need that day. `verify_c_signatures`
covers Rust-vs-C for declared prototypes; a shim that is *itself* mis-prototyped in C is
outside that net.
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

Third instance, and the purest form of it: **arm64's `KeSynchronizeExecution` raised IRQL and
called the routine, and never touched `Interrupt->ActualLock`** — while i386
(`ke/i386/irqobj.rs`) and amd64 (`ke/amd64/interrupt.c`) both acquire it. That is the one
function in the DDK whose entire contract *is* mutual exclusion against a driver's ISR, so on
the SMP kernel it delivered exactly nothing; on UP the IRQL raise happened to be enough, which
is why it survived. When a per-arch port of a synchronisation primitive is shorter than its
siblings, the missing lines are the primitive.

### A UP-tuned boot timeout reads an `-smp N` boot as a wedge
**Symptom:** an `-smp N` run fails the boot fixture with a plain `TimeoutError` while the serial
log shows the boot still *talking* — arm64 `-smp 2` conformance reached
`[WINLOGON] explorer.exe created (PID 216)` and was mid-SCM-startup when the 180s budget expired.
**Cause:** the per-arch `boot_timeout` is sized against the *UP* boot (arm64: ~40s observed →
180s budget). Under MTTCG every vCPU is its own host thread doing its own translation, and the
guest genuinely does more work (AP bring-up, IPI round-trips, real spinlock contention), so
wall-clock to any milestone grows roughly linearly in N — on an already-contended fleet host,
more. **Fix:** `vm.smp_boot_timeout()` scales `boot_timeout` by N. Scale the CLOCK only —
`hang_silence_seconds` is deliberately NOT scaled, because the silence watchdog is what actually
catches a wedge, so a genuine MP hang still fails in ~120s instead of burning the stretched
budget. Do not "fix" this by lowering the boot marker; the marker is what proves the desktop
came up. Recompute the scale from a stashed UP value, not from the current one, or a config
reused across a retry loop multiplies each time.

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

### An uninitialised PFN `CacheAttribute` turns every `VirtualProtect`ed page into DEVICE memory
`MiInitializePfn` (and the `AndMakePteValid` / `ForOtherProcess` variants) never set
`u3.e1.CacheAttribute` — faithfully, because upstream ReactOS's C doesn't either. **`MiNonCached`
is 0**, so a page coming off the free list reads as *deliberately uncached* when nobody ever
decided anything. `MiFlushTbAndCapture` consults that field on **every protection change** and
force-ORs `MM_NOCACHE` onto the mask for a `MiNonCached` PFN, so a plain `VirtualProtect` on
private memory silently rewrites the page uncached.

On x86 that costs only performance, which is why it survived for years. On arm64 `MM_NOCACHE`
selects MAIR **AttrIndx 1 = Device-nGnRnE**, where *every unaligned access alignment-faults* —
and unaligned stores are completely ordinary in x86 code an emulator is translating. The fix is
to set `MiCached` in the three init paths, which is what Windows does; genuinely uncached pages
(MMIO) never come through them — `MmMapIoSpace` and the MDL paths set the attribute explicitly.

Two recognition rules fall out of it:

- **A zero-valued enum member is not a default, it is an assertion.** `MiNonCached == 0` means
  "uninitialised" and "the caller chose uncached" are the same bit pattern, so every consumer
  believes a decision that was never made. Suspect this whenever a field is only ever *read*.
- **Decode the PTE, don't trust the protection.** `[ALIGN] FAR=… PTE=… MAIR=…` gives you the
  answer directly: `AttrIndx = PTE[4:2]`, index it into `MAIR_EL1` (ours is `0x004400FF`:
  0=Normal-WB, 1=Device-nGnRnE, 2=Normal-NC). A page that is *architecturally* Normal but
  *mapped* Device faults on exactly the accesses Normal memory permits, so the symptom looks
  like a CPU bug rather than a mapping bug. The tell is a fault that **repeats at a pinned PC**:
  the handler un-protects, resumes, and the same unaligned store faults again forever.

**The opcode tells you which of the two you have, and this is the whole triage.** An unaligned
`ldar`/`stlr`/`ldxr` faults on *any* memory type — that is an ordinary unaligned ATOMIC (x86
permits them, AArch64 does not) and the emulator's own handler is supposed to absorb it. An
unaligned **plain** `ldr`/`str` faulting means the page is Device, full stop, because Normal
memory permits it. Measured across the arm64 WoW64 ladder: before the fix, 6–14 `[ALIGN]` per
boot, `PTE=…447` (AttrIndx 1) with `OPC=0xB83F6AB4` — a plain `str`; after, 0–1 per boot,
`PTE=…4C3` (AttrIndx 0) with `OPC=0x88DFFE85` — `ldar`, absorbed and the run passes. So the
whole class collapsed to the one case that is *supposed* to happen.

### arm64 bring-up bugs worth recognising again
- **Every arm64 syscall ran with interrupts MASKED, end to end.** Taking any exception sets
  PSTATE.DAIF to all-masked in hardware; nothing in `KiEL0SynchronousEntry` (trap.S) undid it,
  so kernel-mode interrupt latency was bounded by the longest system call and every blocking
  syscall parked a DAIF-masked context on the swap chain. This was landmine #2 of the
  (archived) arm64-interrupts quest and had been known-and-deferred since 2026-07-05. Fix:
  `UNMASK_IRQ_FROM_SPSR` — re-enable **iff the interrupted context had them enabled**, reading
  the saved SPSR bit 7, which is exactly the rule x86 uses; a fault taken inside an IRQ handler
  or inside `KiTrapReturn`'s masked ELR/SPSR banking window therefore stays masked. It does not
  weaken IRQL: on this port IRQL is GIC-priority-backed (`GICC_PMR`) and no GIC line lives in
  the PASSIVE..DISPATCH band, so unmasking can only admit lines whose DIRQL genuinely exceeds
  the current IRQL. EL1 synchronous entry is still masked — a separate change.
  **Recognition rule: on a port where IRQL is priority-masked rather than DAIF-masked, DAIF
  belongs to exception entry/exit windows and nothing else. If a trap handler never touches it,
  it is running masked.**
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

### `&mut (x as T)` borrows a TEMPORARY, so the OUT parameter is written and dropped
**Symptom.** An out-parameter is always its initial value at the caller, and every branch that
depends on it takes the wrong arm. No crash, no warning, and the callee is provably correct —
put a probe in it and you watch it write the right answer.
**Cause.** In Rust `&mut (expr)` where `expr` is not a *place* — a cast, an arithmetic
expression — materialises a temporary and lends out *its* address. The callee's store lands in
the temporary, which dies at the end of the statement. rustc says nothing: taking `&mut` of a
temporary is legal and ordinary. It shows up in translated output because C's `&x` is
spelled `&mut x` here and a type mismatch invites someone to "fix" it by casting *inside* the
borrow instead of casting the pointer.
**Fix.** Give the local the callee's type and borrow it directly — `let mut flag: BOOLEAN = 0;
f(&mut flag)` — or take the address and cast the *pointer*:
`core::ptr::addr_of_mut!(x).cast()`. Never cast inside the `&mut`.
Live instances, all found while chasing decl/def drift and none of them visible to any lint we
had: fastfat `FatCommonCreate` passed `&mut (OplockPostIrp as u8)` to two callees that set it
to say "I posted the IRP, stop here", so the create path always saw FALSE and ran on; the same
function did it with `FileNameOpenedDos`, which six downstream calls use to decide DOS-name
semantics; atapi's `AtapiHexToString(&mut (ptr as PCHAR))` threw away the buffer advance the
callee writes back (inert only because both pointers are re-read from the array afterwards);
and iomgr dropped `MiResolveImageReferences`' `LoadedImports`. `grep -rn "&mut ([A-Za-z_][A-Za-z0-9_.]* as "`
found all four across the workspace and is worth re-running — the shape is rare enough to
eyeball and dangerous enough to be worth a scanner.
**Same lost write, one layer out: a C++ `&` out-parameter DEREFERENCED at a C ABI shim.** The
bridge generated for `CFSFolder::GetFSColumnDetails(UINT, SHELLDETAILS &)` was
`f(iColumn, *sd)` — take the caller's pointer, dereference it, pass the value. That type-checks
(`SHELLDETAILS` is `Copy`), and it throws away everything the callee fills in, so
`CDesktopFolder` read an uninitialised column. `GetDefaultFSColumnState(UINT, SHCOLSTATEF &)`
had it too. Fix: reborrow, `f(iColumn, &mut *sd)`. **Any `&` parameter crossing a hand-written
or generated C shim is this hazard**, and it is invisible from the Rust side — the callee is
correct and the caller compiles, so only reading the C++ prototype tells you the parameter was
an OUT. `grep -n '(\*[a-z]' over a module's bridge block finds them.

### An OUT-cell for a pointer-sized value overruns the 32-bit caller's variable
**Symptom.** A 32-bit guest corrupts a local, or the field next to a handle, with no failing
call anywhere near. The syscall that did it **succeeded** and returned the right value in the
right place.
**Cause.** The syscall takes a caller pointer to a cell the kernel fills with something
pointer-sized — `PHANDLE`, `PVOID *BaseAddress`, `PSIZE_T`. A flat thunk passes the guest's
4-byte cell straight through (a 32-bit guest pointer *is* a valid 64-bit pointer, so nothing
complains) and the kernel's 8-byte store runs four bytes past it. Same family as the
`IO_STATUS_BLOCK` 8-vs-16 overflow, and just as quiet: only the collateral write is wrong, so
the damage surfaces later and somewhere else.
**Fix.** Take the result in a host-side cell and narrow it into the guest's:
`HANDLE h = NULL; St = Nt...(&h, ...); if (St >= 0) *(ULONG *)guestCell = (ULONG)(ULONG_PTR)h;`
**Find them with the tool, not by reading.** A flat thunk over a divergent payload is the
*absence* of code, so review cannot see it — `marshalgen audit` derives the list from clang's
own 32/64 record layouts. The proof that memory is not enough: `NtOpenProcessTokenEx` and
`NtOpenThreadTokenEx` had been marshalled for months while the plain `NtOpenProcessToken` and
`NtOpenThreadToken` sitting beside them in the same table had not — someone fixed the variant
they happened to be looking at.
**Testing one:** park a canary immediately after the cell, and require the call to SUCCEED — a
failing syscall writes nothing and passes the check while completely unmarshalled. Make the
canary `volatile`: you only hand the callee `&cell.Handle`, so a conforming compiler may assume
the rest of the struct is untouched and fold the comparison away, deleting the bug from the test
but not from the OS.

### A bare `PUNICODE_STRING` argument makes the kernel read the string pointer from PAST the struct
**Symptom.** A 32-bit guest calls something that takes a `PUNICODE_STRING` and the box
**BUGCHECKs 0x1E**, with a `[PROBE-AV-R]` line just before it naming an address that is not
anything the guest passed.
**Cause.** `UNICODE_STRING` is 8 bytes in the guest (`{USHORT Length; USHORT MaximumLength;
ULONG Buffer}`) and 16 on the host (`{USHORT Length; USHORT MaximumLength; ULONG pad;
ULONGLONG Buffer}`) — `Buffer` at offset 4 vs offset 8. A flat thunk therefore reads the
pointer from *past the end* of the guest's object. Not a truncation: the value is assembled
from whatever two words followed the struct. `Length`/`MaximumLength` share offsets, so they
arrive intact and the operands look plausible — read `Len=` in the probe line, it will be the
CORRECT byte count for the string the guest meant.
**Fix.** Marshal it: copy Length/MaximumLength, zero the pad, widen Buffer.
**Reachability is not exotic.** `NtSetValueKey` was flat while `NtQueryValueKey` beside it was
marshalled, so registry *reads* worked and every `RegSetValueEx` from a 32-bit guest bugchecked
the machine — a local DoS from any 32-bit app. Same "somebody fixed the variant they were
looking at" shape as the out-cell entry above; both lists have to come from `marshalgen audit`.
**Testing one:** round-trip it — set, read back BY THE SAME NAME, delete, confirm gone. A wrong
`Buffer` cannot survive that. Checking only that the call returned success passes on a broken
build right up until the box dies.

### A struct that carries its own `sizeof` is rejected before a single field is read
**Symptom.** A whole family of Win32 APIs returns `FALSE` with `ERROR_INVALID_PARAMETER` for
every 32-bit process and only for 32-bit processes. Nothing crashes, nothing is logged, and the
marshaller for the struct is provably correct field for field.
**Cause.** The struct's leading `cbSize`/`lStructSize` is the caller stating its own `sizeof`,
and the callee version-checks it against the **host** `sizeof`. The guest wrote a number that
is only true on the guest, so the call is refused before the payload is looked at. A
field-perfect marshaller copies that number faithfully and is therefore still broken — this is
the one field where a verbatim copy of a same-width, same-offset field is *wrong*.
**Fix.** Rewrite the size on the way in and restore it on the way out, and do it **guarded on
the value**: only when it is exactly the 32-bit `sizeof`. A struct with several legal sizes
(`MONITORINFO` vs `MONITORINFOEXW`) must still reach the kernel with the size the caller chose,
and garbage must still be rejected as garbage. The restore matters as much as the rewrite: the
callee restamps the field with the host size, and handing that back leaves the caller holding a
number that does not describe its own struct.
**You need the bounce buffer too, and the size fix hides that you don't.** Once the size check
passes, the callee writes the HOST-sized struct — `GUITHREADINFO` is 48 in the guest and 72 on
the host, so "fixing" the size alone converts a silent no-op into 24 bytes of caller-stack
corruption. Test with a canary immediately after the struct; a rewrite with no bounce passes a
success check and fails the canary.
**The size can also be an ARGUMENT** (`NtUserSendInput`'s third parameter, `sizeof(INPUT)`),
which no struct-layout diff can associate with the pointer it describes — those stay
hand-written. Live instances on vibeos: `TrackMouseEvent` (16→24, so no 32-bit app ever
received `WM_MOUSELEAVE`), `GetCursorInfo` (20→24), `GetGUIThreadInfo` (48→72),
`GetComboBoxInfo` (52→64), `GetMenuInfo` (28→40), `SendInput` (28→40).
**Before you believe a WIDENING verdict, check what the callee does with the pointer.**
`NtUserThunkedMenuInfo` declares `LPCMENUINFO` and diffs as a clean 28→40 widening, but
win32k casts it to `PROSMENUINFO` and reads *`cbSize` bytes* — anything in
`[sizeof(MENUINFO), sizeof(ROSMENUINFO)]`. The payload is a **caller-declared variable
size**, so a generated marshaller hands over a 40-byte bounce and lets the kernel read past
the end of it: a bug the marshaller INTRODUCES, where the flat thunk it replaced at least
passed a real, correctly-sized caller pointer. Marshal only the size class you can name and
pass the rest through. A prototype is a claim about the declared type, not about what the
callee reads.
**Third time for the same meta-bug.** The hand-written `NtUserThunkedMenuItemInfo` thunk set
`M64.cbSize = sizeof(M64)` from day one; the generator that produced the other five did not.
Same shape as `NtOpenProcessToken` vs its `Ex` sibling and `NtSetValueKey` vs `NtQueryValueKey`
— whichever variant a human read is right. Fix the tool, not the instance.

### The kernel sets last-error in the 64-bit TEB and the 32-bit guest reads TEB32
**Symptom.** A win32k API fails for a 32-bit process and `GetLastError()` returns 0, or the
code the guest itself set before the call, or an error from some unrelated earlier call. The
app gets the `FALSE` and never the reason. Nothing crashes, so it survives any number of green
boots — and it silently disarms every test that discriminates on an error code.
**Cause.** win32k reports failure with `EngSetLastError`, which writes
`NtCurrentTeb()->LastErrorValue`. In kernel mode that is the thread's **64-bit** TEB; a 32-bit
guest's `GetLastError()` reads `TEB32->LastErrorValue`, a different structure at
`WowTebOffset`. Every code the whole USER/GDI surface produced landed in a TEB no 32-bit
process ever reads. The mirror image is true too: `EngGetLastError` cannot see a value the
guest set. The ntdll surface is unaffected — those syscalls return an NTSTATUS and the guest's
own ntdll32 converts it into its own TEB32.
**Fix.** Bridge it once at the syscall dispatcher, not in every `Eng*` caller. On entry copy
the guest's value UP into the 64-bit TEB and remember it; on exit copy DOWN only if the 64-bit
value **changed**. Both halves earn their place: without the copy-up, win32k's own
save-and-restore idiom (`Error = EngGetLastError(); …; EngSetLastError(Error);`) faithfully
preserves a stale value instead of the caller's; without the change test, an unconditional
copy-down overwrites what a kernel→user callback's `SetLastError` just wrote, and turns a fix
for failing calls into a corruption of succeeding ones.
**Test it with two different codes out of one API.** One call driven two ways — a bad size
answering `ERROR_INVALID_PARAMETER` and a bad flag answering `ERROR_INVALID_FLAGS` — proves the
code *travelled*; a single expected code is also satisfied by a bridge that hardcodes, guesses,
or zeroes. Pick an API whose client-side stub is a bare forward (`user32.spec`:
`TrackMouseEvent` → `NtUserTrackMouseEvent`) so the code cannot have come from user mode. Then
assert separately that a **succeeding** call leaves the guest's own last-error alone.
**This is why an assertion that gates on a kernel error code is dead on arrival.** The bug was
found by a control run whose test excused a failure unless `GetLastError()` was
`ERROR_INVALID_PARAMETER` — a discriminator that could never fire. If a guest test branches on
a code the kernel sets, prove the code arrives before trusting the branch.

### Registry redirection: the interception point is the PARSE, and the exemptions cannot be symlinks
**Symptom.** A 32-bit process shares the 64-bit `HKLM\Software`, so per-app registration has no
isolation and a COM `InprocServer32` read by a 32-bit app names a DLL it cannot load. Nothing
fails loudly; it is a fidelity gap that only bites once two component sets exist.
**Cause / where it goes.** Registry opens are overwhelmingly RELATIVE — advapi32 caches a
handle to `\REGISTRY\MACHINE` and passes `RootDirectory` + `"Software\Foo"` — so a path rewrite
in `wow64.dll` (where FILESYSTEM redirection lives) would have to resolve the root handle to a
path on every single open, which is both a syscall per open and stateful. Do it in CM's parse
procedure instead: `NtOpenKey`/`NtCreateKey` both funnel into `ObOpenObjectByName` →
`CmpParseKey`, and Ob's only parse-procedure bypass is for *directory* objects, which a key
never is. One wrapper there is a complete interception point and needs nothing above CM to
cooperate.
**Ob hands the parse procedure a remaining name in TWO shapes, and both occur on every boot.**
An open by ABSOLUTE path arrives with a LEADING separator (parent `\REGISTRY`, remaining
`\Machine\Software\...`); an open RELATIVE to a key handle arrives without one (parent
`\REGISTRY\MACHINE`, remaining `SYSTEM\...`). Code that assumes exactly one separator sits
*between* the parent path and the remaining name works for the relative form and silently does
nothing for the absolute one — the first cut of E24 compared `\Machine\Softwar` against
`MACHINE\SOFTWARE` on every absolute open and declined to redirect. Skip leading separators
before comparing, and carry that offset into wherever you splice. This failure mode is
**inert, not broken**: nothing crashes, no status is wrong, the feature simply never happens.
**Beware a hand-built test suite that shares the code's blind spot.** The same E24 splice was
checked offline against 16 hand-written cases and passed 16/16 — because every case had been
invented in the relative shape, the same shape the author had reasoned about. A test written
from the implementer's mental model inherits that model's gaps by construction. What found the
bug was a kernel trace printing the REAL arguments; the harness only became trustworthy once
its cases were transcribed from that trace. If a pure-logic harness is worth writing at all,
seed it from observed inputs rather than imagined ones.
**And make the assertion name what it FOUND, not just what it wanted.** The guest rung failed
with "default view is not in Wow6432Node" and did not print the name it had just been handed —
so answering "then what IS it?" cost a full 30-minute build plus a boot, for a string the
assertion was already holding.

**Two traps worth knowing before you design it.**
- **Shared-key exemptions cannot be registry symlinks.** It is tempting — CM already resolves
  them, and Windows really does keep `Software\Wow6432Node\Classes` as a link. But a link
  target is an ABSOLUTE path that Ob re-parses, and the re-parse redirects it *again*: a link
  from `Software\Wow6432Node\Policies` to `Software\Policies` bounces straight back into
  `Software\Wow6432Node\Policies`. Exemptions have to be a table the redirect itself consults,
  which is what Windows' redirection table is.
- **The view has to be sticky per handle, and it comes free.** Redirect only when the SOFTWARE
  boundary is crossed INSIDE the remaining name. If the caller's root handle is already at or
  below SOFTWARE, that handle was resolved under some view already and relative opens must stay
  in it — so "do nothing" is the correct answer, and a 32-bit app that opened SOFTWARE with
  `KEY_WOW64_64KEY` keeps the 64-bit view beneath it without any per-handle state.
**Redirection without population is a regression, and a miss-fallback is the cheap way out.**
Nothing populates `Wow6432Node` on a system that ships one component set, so redirecting reads
into it hides every OS key a 32-bit app needs. Either mirror the 64-bit subtree (Windows'
answer, via the installer) or retry the unredirected name when the redirected open returns
`OBJECT_NAME_NOT_FOUND` (Wine's answer). The fallback must be READS ONLY — a create that falls
back has no isolation, which was the point. And the node itself still has to exist in the hive:
`NtCreateKey` creates only the LAST component, so a create redirected into a missing
`Wow6432Node` fails on the intermediate.
**`KEY_WOW64_32KEY`/`64KEY` stop being strippable the moment views differ.** They are view
selectors, not access bits — no key's ACL grants either, so one left set denies the open (that
was E21). Stripping them early is correct only while the two views are identical; after that,
`KEY_WOW64_64KEY` is a WoW64 process's only route back to the 64-bit view. Move the strip down
to the parse so the bits survive long enough to pick a view and still never reach the access
check.
**Test that the two views DISAGREE, not that a write succeeded.** With no redirection at all a
create still succeeds and a read still finds it. Assert the default-view handle's own
`NtQueryKey(KeyNameInformation)` says `Wow6432Node`, AND that the identical path under
`KEY_WOW64_64KEY` cannot see the value just written — two independent encodings, so faking one
still fails the other.

### A probe-ladder guest can be RED on one arch for weeks — the smoke stops before it runs
**Symptom.** A guest verified green on one arch fails on the other, and no gate ever reports it.
**Cause.** The WoW64 probe ladder is spawned by winlogon *after* the desktop marker, and the
ordinary smoke's success condition IS that marker — so it stops watching before the ladder runs.
A late-rung, single-arch failure is structurally invisible to `vibeos-cli gate`.
**Fix.** `./vibeos-cli smoke --arch <a> --ladder`, on **both** 64-bit arches, whenever a guest,
`wow64.dll` or a marshaller changes. It drains serial past the marker until all nine guests
report and fails the boot on a wrong or missing verdict; arm64 needs media built with
`VIBEOS_FEX=1`, and a boot with nothing staged says so instead of reading as a regression.
Table + rationale in `tests/qemu/cli/ladder.py`. Concrete instance: `selfmod32`'s rung-5 fixture
writes under `%WINDIR%` — a writable ramdisk on arm64, a read-only CD on the amd64 livecd — so
it exited `0xBAD9` there from the day it landed, through every green gate.
*(The flag exists because the hand-rolled `--until "<guest>.exe exit"` recipe is exactly the
"remembering a recipe" smell AGENTS.md names as a bug report against the tool — and the same
blind spot had already been written down once, in the roadmap's `_MmCopyFromCaller` entry, and
still cost a day.)*
**Also the general rule for fixtures:** when the environment cannot support a rung's *fixture*
(not its subject), skip it loudly with the errno and keep the remaining rungs running, rather
than failing the whole guest. The arch where the fixture works keeps full coverage of that rung.

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

### On a JIT host, the CPU layer's memory notifications are a CAPABILITY, not a cache hint
The `BTCpuNotify*` family reads like invalidation bookkeeping you could skip and merely run slow.
It is not. FEX's decoder **refuses to translate any guest address outside the executable set it
has been told about** (`Frontend.cpp` `CheckRangeExecutable` → `HitNonExecutableRange`), and that
set is populated *only* by these notifications plus a one-time `VirtualQuery` sweep of the whole
address space at `BTCpuProcessInit`. So wiring `BTCpuNotifyMapViewOfSection` alone — which is
where this lane sat for eight phases — covers images and **nothing the guest allocates
afterwards**: a 32-bit guest could not execute `VirtualAlloc`'d code at all. That is ATL/MFC
thunks, VB6/Delphi trampolines, packers, self-extractors and every JIT, i.e. a large fraction of
real 32-bit binaries, failing on a path no image-based test can reach.

The one-time sweep is why bring-up got away with it, and it is also a trap: `wow64.c` had grown a
comment telling itself to *"allocate every raw guest page first; pages created afterward are not
visible"*. **A workaround that constrains allocation ORDER is a missing notification wearing a
disguise** — it holds exactly until something allocates outside your control, i.e. until the
guest runs.

Three properties of the contract that bite if you skim it:

- **`After` is a lock/unlock flag, not a status report.** `BTCpuNotifyMemoryAlloc`/`Protect`/
  `Free`/`Unmap`/`ReadFile` take an internal mutex when `After=FALSE` and release it when
  `After=TRUE`. Call them immediately either side of the host syscall with **no early return,
  goto or longjmp in between**: an unmatched *Before* deadlocks the whole guest process, an
  unmatched *After* unlocks a mutex nobody holds. Anything that reads like a scope guard should
  be written as one.
- **Notify for the CURRENT process only.** The layer's model describes *this* address space, and
  the protect path really does reprotect pages — hand it a `VirtualAllocEx` range belonging to
  another process and it mutates your own pages at those addresses. Test the handle, don't
  assume `(HANDLE)-1`.
- **The failure mode is a plausible wrong answer, not a crash.** Free-then-reallocate at the same
  base, or rewrite-then-`FlushInstructionCache`, silently execute the *previous* tenant's
  translation. A probe guest that only checks "did it survive" scores those as passes; check
  returned VALUES.

Coverage note: on a native CPU layer (amd64's `wow64cpu`) none of these exist, execution is
direct, and there is nothing to invalidate — so amd64 is the **control** for such a test, not a
second copy of it. Every rung must pass there with 0/8 entry points wired.

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

**Require `special pool ACTIVE (routing verified)` on serial — `ACTIVE` alone used to be a lie.**
Symptom: special pool armed, initialized, announced ACTIVE, and caught nothing; the stop code
came back bit-identical to the un-armed run and the `[POOL]` page walk showed ordinary pool
blocks. Cause: `ARM3/special.rs` declared a **local** `const POOL_FLAG_SPECIAL_POOL = 0x10`, which
shadowed the correct `0x20` the file already glob-imports from `ntkrnl_sys::mm` — `0x10` is
`POOL_FLAG_CHECK_DEADLOCK`, so `MiInitializeSpecialPool` set a bit that
`ExAllocatePoolWithTag` never tests and **not one allocation was ever routed**. Fix: the local
const is gone, and arming now allocates through the real `ExAllocatePoolWithTag` and checks
`MmIsSpecialPoolAddress` before claiming anything. Two durable lessons: **never redeclare a
constant the crate already imports** (a local `const` shadows a glob import silently, with no
warning at either site), and **a selftest that bypasses the production path proves nothing about
it** — `SPECIALPOOL_SELFTEST` called `MmAllocateSpecialPool` directly, so it stayed green through
all of this while calling itself an "end-to-end detection proof". Any pre-fix conclusion of the
form "special pool didn't catch it, so it isn't an overrun" is void; re-run it.

### Breakpoints and watchpoints require TCG
**KVM fires neither gdb breakpoints nor hardware watchpoints** `[§9.1]`. Prepend `VIBEOS_NO_KVM=1` for
i386 hunts. Do **not** switch the smoke backend to native KVM to "repro faster" — it hits the
per-arch accelerator cap, so a parallel repro just queues.

### Symbolication
- **The kernel now PRINTS its own runtime base — stop deriving it.** Every boot emits
  `[KRNLBASE] ntoskrnl runtime base=0x...` from `MiInitializeLoadedModuleList`
  (`ARM3/sysldr.rs`, right after `PsNtosImageBase` is taken from the loader block). Feed it
  straight to `qemu/addr2sym.sh <arch> --kernel-base 0x<that value> <addr>`, which does the
  `- 0x400000` itself so nobody has to remember the direction. This makes the whole *retained*
  crash corpus symbolicable offline with no VM — previously a saved arm64 bugcheck was a bare
  hex address unless a live VM was still around for `inspect.sh` to query. Measured stable
  across boots on arm64 (`0xFFFFF80042190000` twice), so a base recorded once stays good for
  that build. The derive-from-`module+offset`-pairs method below is now the *fallback*, for
  logs predating this line.
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
- **Read `hang-cpustate.txt` FIRST — a hang is the one failure with no PC of its own.** On a boot
  timeout the harness now asks QEMU where every vCPU is parked (`info status` / `info cpus` /
  `info registers -a`) *before* killing the VM, symbolicates each PC, and drops the result in the
  run dir next to `summary.json` (also surfaced as `cpu_state` + `hang:` salient lines). A guest
  spinning in a kernel spinlock does not block QEMU's main loop, which is why the monitor still
  answers a box that has gone silent. Both backends used to run `-monitor none`, so for years
  every wedge was triaged from the serial tail alone and nobody ever asked the VM where it
  stopped.
- **A foreign binary that is stuck is usually one of three shapes** `[§9.29]`: sample the guest
  PC a dozen times via QMP `info registers` — all samples at one PC means a spin; parked in a
  wait with no file I/O (break on `NtReadFile`/`NtDelayExecution`); or spinning at a fixed
  usermode PC that disassembles to `b .` (`-O0` is the workaround until the pass is identified).
  (Sampling is now automatic on a timeout — see `hang-cpustate.txt` above; hand-sampling is for
  when you need the *repeated* samples that distinguish a spin from slow progress.)
- **A timed wait that overruns its own timeout indicts the CLOCK, not the thing being waited on.**
  Read the timeout out of the source and compare it against how long the round actually ran: the
  SCM's `PipeTimeout` is 30 s (`services-rust/src/database.rs`), so an arm64 `--smp 1` round that
  sat at `ScmStartUserModeService(SamSs): waiting for connect` for 185 s was not a slow service —
  a 30 s wait cannot take 185 s unless nothing is expiring timers. Discriminator against the
  §9.89 handshake bug, which has the same last-line: **that** one prints `connect result=1053`
  when the 30 s elapses and the box keeps running; a stall with **no** `result=` line at all, and
  every other subsystem's serial stopping within a few lines of it, is a whole-machine freeze.
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
  `SPECIALPOOL_SELFTEST=guard` run exists for exactly that, but note it exercises only the
  detection machinery (it allocates via `MmAllocateSpecialPool` directly); the *routing* proof is
  the `routing verified` line described in the tripwire section above. In the same pass, a
  plausible IRQL-leak hypothesis was **disproven live**.

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

**All-15-error also comes in a FAST costume, and the 600 s number is not the tell.** 2026-08-01:
`15 errors in 28.59s` on a host at `build(cpuheavy) 4/4`, with the plain boot stage green and the
desktop reached (explorer + taskbar in the serial log). The discriminator that matters is
**error-vs-fail and uniformity, not the duration** — 15 *errors* is a fixture that never came up,
whereas a real net regression produces *failures* in a *subset*. Confirmed in one step for free:
re-running `pytest tests/qemu/test_net.py --arch amd64-uefi-net` against **the same unmodified
media** gave `15 passed in 63.69s`. That re-run is the cheapest control available — it needs no
rebuild and no second lane — so reach for it before theorising, especially when your change
touches something with a plausible story for breaking the network (this one altered registry
security descriptors, which made "it's contention" the *less* comfortable hypothesis and therefore
the one worth actually testing).

**Partial contention wears a different costume: ONE test, and the giveaway is the wall clock.**
`test_tdi_selftest_roundtrip` timed out on `[NETSTACK] TDI self-test: RECEIVE OK` while the other
14 passed — which reads exactly like a real regression in the MDL data path, and it failed all
three gate attempts. Compare the **durations**, not the pass/fail: the failing runs took ~186 s
where a passing one takes ~44 s. A logic bug does not make the suite four times slower; a loaded
host does. The confirming control is free and already on disk — the conductor gates the base
alone after a lone rejection, and that base run showed the *same* test failing its first attempt
at ~187 s and passing the next two at ~44 s. **Same test, same assertion, on master.**

Two things follow. First, the gate's retries fire back to back, so all three land inside the same
contention window and a retry policy cannot rescue you from it — a `NET_FLAKE` verdict after N
attempts is not evidence the failure is real. Second, and this is the one that cost the batch:
**do not run `check`, `check-tooling`, `difftest`, or a build locally while your own lane is
gating.** `./vibeos-cli host-status` before submitting, then leave the host alone. The lane whose
gate you are starving is your own.

**The third costume is the HOST's own network, and ruling out contention does not rule it in.**
2026-07-31: `test_dns_resolve`, `test_tdi_datagram_roundtrip` and `test_usermode_winsock_roundtrip`
failed 3/3 attempts on an idle host, on unmodified `master`, at normal boot durations. That is
exactly the evidence the paragraph above says means *real regression* — contention was genuinely
excluded — and the conclusion was still wrong: the developer's **VPN** was up, and disabling it
turned the suite green with no code change. Do not stop at "not contention"; the host is a third
suspect, and one the guest cannot see.

**Read the PASS/FAIL SET, not the timings — it names the culprit directly.** Our slirp
mini-internet has two kinds of endpoint, and the failures partitioned perfectly along that line:

- **Fails:** every test that reaches **`10.0.2.3:53`**, slirp's DNS forwarder, which proxies to
  *the host's resolver* — `test_dns_resolve` (kernel A record), `test_tdi_datagram_roundtrip`
  (TDI_SEND_DATAGRAM of a real DNS query), `test_usermode_winsock_roundtrip` (nettest's UDP leg
  is also a DNS query — `nettest.c`, `DNS_IP`/`DNS_PORT`).
- **Passes:** everything that stays inside the machine — the guestfwd `-cmd:cat` echo at
  `10.0.2.100:9999` and the hostfwd loopback ports. `test_tdi_selftest_roundtrip`, the registered
  flake and the one you would expect to break first, passed.

A kernel-side TCP/UDP/MDL regression does not respect that boundary; a rewritten host resolver
does, precisely. So: **if every failure needs the host's DNS and every in-guest endpoint is
green, suspect the host's network configuration — VPN, split DNS, a corporate resolver — before
you suspect the OS.** The DNS dependency is documented in `test_dns_resolve`'s own docstring
("slirp forwards 10.0.2.3 to the host's DNS"); it is the only external dependency in the suite,
which is what makes the partition so clean. Cost of missing it: a 19-hour halted merge train and
a regression handed to the wrong lane.

**A red base is an alibi only for the failures it EXPLAINS — compare the STAGES, not the
verdicts.** 2026-07-31: the queue sat undrained for 19 hours behind
`TRAIN HALTED — origin/master itself fails the gate (NET_FLAKE) ... No submission was rejected`.
That reads like the base check working, and it was half working. The batch had failed
`LINT_FAIL` while the base-alone control failed `NET_FLAKE` — two independent classes, and a
starved net suite says nothing about whether a lane added lint findings. The conductor collapsed
each gate to one `classification`, so "base red" became a blanket alibi and three real
`verify_signatures` regressions (`KiThreadStartup`, `NtSetLdtEntries`, `ObpSetHandleAttributes`)
went unattributed while every innocent lane queued behind them. The evidence was already on disk:
both `summary.json`s carry per-stage results, and the base **passed all three `lint:*` stages**.
`queue.py` now keeps that map (`GateVerdict.stages`) and rejects a lone submission for the
DETERMINISTIC stages the base ran and passed, while still halting the train — nothing can land on
a red base, but a doomed lane no longer blocks the queue with its defect unreported. Two guards
matter as much as the rule: a stage the base did not RUN proves nothing (an early-halting base
gate must not convict by omission), and smoke stages are excluded because they are exactly the
timing-sensitive ones this section is about. Generalise it: **when a control experiment fails,
check that it failed the same WAY before you let it absolve anything.**

### A smoke that dies with a Python traceback naming ANOTHER arch's directory

**Symptom.** A gate's arm64 smoke produces no boot at all — just a `FileNotFoundError` out of
`provenance.check_staleness` → `_crate_staticlibs` → `Path.rglob`, naming
`sdk/lib/rust/target/i686-kernel`. An **arm64** smoke, killed by an **i386** path, before QEMU
ever starts. The other two arches pass, so it reads like an arm64-only defect and is not one.

**Cause.** A parallel `gate` builds all three arches in ONE worktree against ONE shared cargo
target dir. The i386 `--clean` build deletes `target/i686-kernel/` at the same moment the arm64
staleness check is walking `target/` looking for staticlibs. `Path.rglob` is a generator over
`scandir`, and a directory that vanishes mid-walk raises instead of being skipped — so the
exception escapes into the smoke and takes the run with it. Any unsynchronised walk of a tree a
sibling arch is building is exposed; it is not specific to this one caller.

**Fix.** `provenance.rglob_stable()` redoes the whole walk (bounded retries, then re-raises).
Note what it deliberately does NOT do: catch the error and return the partial list. pathlib's
generator is dead after it throws, so "what we had" is a silently short list — and for this
caller a short list means "this crate has no staticlib", which is a *staleness verdict*, not a
missing answer. **A degraded oracle is worse than one that refuses to answer**, the same rule
`ProvenanceError` exists for. Generalise: when a walk and a build share a tree, the walk must
be idempotent-under-mutation or scoped so it never sees the other arch at all.

### A rare unit-test failure in the HARNESS is a bug report, not noise — and the wall clock is not monotonic

**Symptom.** `test_queue_is_fifo_by_arrival` failed roughly 1 run in 80, only ever under
parallel load, and passed 80/80 sequentially. The obvious readings — "clock granularity",
"a racy test" — are both wrong, and either one would have been fixed by quieting the test.

**Cause.** The merge queue orders submissions by a `seq` of `"<time_ns>-<lane>"` and relies on
lexical order being arrival order. `time.time_ns()` is **CLOCK_REALTIME, which steps.** Under
WSL2 the guest clock resyncs to the Windows host, and the captured failure shows a lane
submitted 2.59 **seconds** in the past relative to one already on disk:

```
lane-a  01785085499633383341   submitted FIRST, larger
lane-b  01785085497039700594   submitted SECOND, smaller by 2,593,682,747 ns
```

So the later arrival sorted first. The test was reporting a **live fairness bug in the thing
that decides whose work lands** — a lane could silently jump the queue.

**Fix.** Lamport-style bump in `queue._next_seq()`: keep wall-clock ns, but never hand out a
value at or before the queue's current high-water mark. Scan **pending only, not results** — the
invariant only has to hold among submissions competing for a conductor, and pending-only
self-heals, whereas including results would pin every future seq to a bad value until the
history was pruned if the clock ever jumped far *forward*.

**Two generalizations worth more than the bug.** (1) Never derive ordering, elapsed time, or
timeouts from `time.time()`/`time.time_ns()` — use `time.monotonic()` for durations, and for
anything persisted across processes make the *stored state* the authority. (2) A 1-in-80
failure is not a control. Reproduce it under the load that triggers it (8 workers × 25 runs
here), then pin it with a test that **fakes the trigger deterministically** — the regression
test steps the clock backwards by the exact observed 2,593,682,747 ns and fails against the
old allocator with the live signature.

### A cached handle in a reference shim is per-PROCESS, but the tests run per-THREAD

**Symptom.** Difftest rows that pass alone and disagree in a full run, with no error return
anywhere — the conversion simply produced different bytes.

**Cause.** `msvcrt-difftest/nlsimpl.c` cached one `iconv_t` per codepage in a `static` array and
handed the same descriptor to every caller. An `iconv_t` is not a handle to a pure function: it
*owns* the conversion state, `iconv()` advances it, and `iconv(cd, NULL, NULL, NULL, NULL)`
resets it. cargo runs `#[test]`s on parallel threads, so two of them converting at once was live
UB. glibc's failure mode is the worst available shape — a **short or garbled result with a
success return** — so the caller converts a string to the wrong string and proceeds. Measured:
8 threads over one CP932 fixture disagreed with the single-threaded answer within a handful of
iterations, returning empty results and uninitialized scratch bytes.

**Fix.** `static __thread` on both caches. The descriptors are then per-thread and need no
locking; leaving them unclosed is deliberate, since a worker thread's caches die with it.
`tests/nls_thread_safety.rs` pins it, and only bites because it converts through a **multi-byte**
codepage — the ASCII fast path opens no descriptor at all, so a fixture built from ASCII
exercises none of this and passes forever.

**Generalization.** Any `static` cache in a reference shim is shared by every test thread. The
ones that hurt are those holding a handle with *internal state* (iconv descriptors, `strtok`
positions, locale objects, RNG seeds), because sharing them corrupts results rather than
crashing. Grep a new reference TU for `static` before trusting a green suite; process-global is
the wrong scope by default, and the real DLL's equivalent is usually a **TEB slot** — per-thread
by construction. The same file's last-error slot is a single `static DWORD` for exactly that
reason and is documented as modelling "a single TEB slot"; it is one writer / one reader by
design, so it is correct only as long as no two threads race a failure through it.

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
