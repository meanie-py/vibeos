# shell32 COM/ATL C++ → Rust (lane: `shell32-com`)

**Status:** OPEN (opened 2026-07-14). Worktree `vibeos-shell32-com`, branch `shell32-com`.
**Experiment:** **Codex drives `rustconv`.** The OpenAI `codex` CLI is the *agent* operating
the pipeline here — it runs `python -m rustconv translate/submit/...` — in the seat Claude
normally occupies. rustconv is still the translator; the models it calls are unchanged. The
user drives codex directly. See §"Why Codex drives" for what we're actually testing.

⚠️ **This lane SPENDS REAL MONEY from the repo's API keys.** rustconv's paid-call rules apply
in full and codex must be told them: never kill a `translate` mid-run (one paid call **per
function** — killing at 35/40 re-pays all 35); output streams to `tools/rustconv/.responses/`
(the canonical copy — never `/tmp`); never re-translate a file that already has output without
asking; `estimate` is a **floor**, real cost is routinely 2–5×. See AGENTS.md §rustconv.

## Scope — the last un-started chunk of shell32

`shell32`'s 14 wine-derived `.c` files are **DONE** (~17.8K LOC of Rust in `shell32-rust`,
cohab-linked; the `.c` are commented out in `dll/win32/shell32/CMakeLists.txt:89-102`). What
remains is what `rust-port-status.md` calls "not started":

| | |
|---|---|
| **81 `.cpp`, ~64,175 LOC** | ATL/COM class implementations |
| **~34 headers** with `CComCoClass` | where the ATL machinery actually lives |
| **60 files** with `BEGIN_COM_MAP` | the interface maps |

This is the single largest remaining userland C++ frontier, and it is a **C++→Rust** problem,
not the C→Rust problem the rustconv pipeline was built for.

### The key structural finding (survey, 2026-07-14)

**The ATL burden is concentrated in the headers; the `.cpp` are ordinary method bodies.**
Almost none of the `.cpp` files contain `BEGIN_COM_MAP`/`CComCoClass` — those live in the
`.h`. Example, `CDropTargetHelper.h`:

```cpp
class CDropTargetHelper :
    public CComCoClass<CDropTargetHelper, &CLSID_DragDropHelper>,
    public CComObjectRootEx<CComMultiThreadModelNoCS>,
    public IDragSourceHelper,
    public IDropTargetHelper
{ ... STDMETHOD(DragEnter)(...) override; ... };
```

So each class splits into two very different jobs:

1. **The COM shell (per class, from the header) — mechanical.** Vtables, refcount,
   QueryInterface, class factory. A fixed recipe (below). This is where ATL's template
   magic has to be *replaced*, not translated. Do this by hand or with a tight prompt; it
   is small and highly patterned.
2. **The method bodies (from the `.cpp`) — the bulk, and the codex-friendly part.** Mostly
   straight-line Win32/shell logic once the COM shell exists.

Corollary: **do not ask codex to "translate the ATL"** — it will hallucinate a template
system. Give it the Rust COM shell as *context* and have it fill in bodies against it.

## The Rust COM shell recipe (the one rule that must not be gotten wrong)

A C++ class inheriting *N* COM interfaces has **N vtable pointers** at the front of the
object (MSVC multiple-inheritance layout). An interface pointer handed to a client points at
one of those slots — **not** at the object base. So:

```rust
#[repr(C)]
pub struct CDropTargetHelper {
    // one vtable ptr PER inherited interface, in DECLARATION ORDER — this is the ABI
    pub lpVtblDragSourceHelper: *const IDragSourceHelperVtbl,  // offset 0
    pub lpVtblDropTargetHelper: *const IDropTargetHelperVtbl,  // offset ptr-size
    pub refcount: LONG,
    // ... fields
}
```

- Each interface method receives `this` = **the address of its own vtable slot**. Recover the
  object with `container_of`-style arithmetic (subtract the field offset), never by casting
  `this` straight to the struct — that is only valid for the *first* interface.
- **Use `core::mem::offset_of!` for that arithmetic — never a hand-written number.**
  `tools/scan_magic_offsets.py` is a 0-NEW lint gate and will fail the build on a bare
  numeric offset. (This is the exact drift class that caused the §9.41 TEB bug.)
- `QueryInterface` returns the address of the matching vtable slot, and `AddRef`s.
- Refcount via `InterlockedIncrement`/`Decrement`; free on 0.
- Methods are `unsafe extern "system" fn` (stdcall on i386 — `extern "system"` resolves it).
- Vtables are `static` tables of function pointers (see prior art).

### Prior art — read these before writing the first class

| What | Where |
|---|---|
| **Vtable + `transmute` table construction** | `sdk/lib/rust/ks-rust/src/unknown.rs` (471 LOC) |
| **Shell COM types already defined** (GUID/IID/PIDL/STRRET + vtables) — *reuse, don't redefine* | `sdk/lib/rust/shell32-rust/src/shellcom.rs` (490 LOC) |
| **Largest COM body of work in-tree** (QueryInterface impls, class factories, marshalling) | `sdk/lib/rust/ole32-rust/src/` (62 `.rs`) |

`shellcom.rs` exists precisely to kill cross-file duplicate definitions. **Extend it** for new
shared interfaces rather than making per-file copies (its header notes Waves 0-3 kept
file-local copies and that retrofit was deferred — do not add to that debt).

## Cohab — already wired, nothing to build

`dll/win32/shell32/CMakeLists.txt` already links `libshell32_rust.a` alongside the C/C++, with
the translated sources commented out. Per-file swap-in is therefore:

1. Write `sdk/lib/rust/shell32-rust/src/<path>.rs`, `pub mod` it.
2. Comment the `.cpp` out of `CMakeLists.txt` with the standard
   `# X.cpp — replaced by Rust (shell32-rust/src/X.rs); cohab via libshell32_rust.a` line.
3. Build ×3, smoke ×3.

Note (from the CMakeLists comment): shell32's cohab links **without**
`--allow-multiple-definition` because the Rust symbols replace commented-out `.c` functions.
Keep that property — if you need the flag, the swap is wrong.

**A C++ class is all-or-nothing.** Unlike the `.c` files, you cannot half-translate a class:
the vtable must come from exactly one language. Swap a class only when every method is
translated.

## Wave plan — leaf-first, smallest-complete-class first

Ordered so the *pattern* is proven on a cheap class before the monsters.

- **Wave 1 — pilot (prove the vtable recipe end-to-end, ~700 LOC).**
  `CDropTargetHelper.cpp` (98), `droptargets/CexeDropHandler.cpp` (145), `CActiveDesktop.cpp`
  (156), `CCopyAsPathMenu.cpp` (162), `CFolder.cpp` (173).
  **Gate: one class must boot ×3 before Wave 2 is authorized.**
- **Wave 2 — self-contained subsystems.** `shellrecyclebin/` (4 files, 1,706),
  `shelldesktop/` (5 files, 2,081), `droptargets/` remainder.
- **Wave 3 — `folders/`** (12 files, 10,210) — `CFSFolder` 2,178 is the anchor.
- **Wave 4 — `shellmenu/`** (7, 6,674) + **`dialogs/`** (10, 8,609).
- **Wave 5 — the monoliths.** `CDefView.cpp` (4,864), `shlexec.cpp` (3,437),
  `CShellLink.cpp` (3,323), `shlfileop.cpp` (2,569), `utils.cpp` (2,191),
  `CDefaultContextMenu.cpp` (1,995).

## Translation rules — shell32 flavor (feed these to rustconv/codex)

⚠️ The rules in `docs/engineering/rustconv-pipeline.md` §"Codex CLI" are **kernel-mode**
(pool tags, IRQL, NTSTATUS, DUMMY_PSEH, `write_volatile` for captures(none)) — as is
rustconv's default system prompt, which assumes ntoskrnl/HAL C. shell32 is **usermode COM**.
Most of that list does not apply, and some of it is actively wrong here (see HRESULT below).
A first task for this lane is deciding whether these become a third rustconv system prompt
(alongside `system_prompt_hal.md`) or stay prompt-level context. Use these rules:

- `#[no_mangle] pub unsafe extern "system" fn` for exports; `extern "system"` for **every**
  COM method (stdcall on i386 — this is the ABI, not a style choice).
- **HRESULT, not NTSTATUS.** `SUCCEEDED(hr)` = `hr >= 0`; `FAILED(hr)` = `hr < 0`. (HRESULT is
  signed — this is the *opposite* of the kernel `NT_ERROR` rule; do not carry that habit over.)
- Return `S_OK`/`E_*` exactly as the C++ did. Do not "improve" an error code.
- **Reuse `shellcom.rs` types** (GUID/IID/PIDL/STRRET). Do not redefine them per-file.
- **`core::mem::offset_of!` for all `this`-recovery arithmetic.** Never a literal offset.
- **No `#![feature(...)]`, no `RUSTC_BOOTSTRAP=1`** — the tree is on pinned stable and both
  are hard-gated by `vibeos-cli check`. (`c_variadic`: use a `macro_rules!` for no-op
  TRACE-style debug macros; the sanctioned trampoline is `rust_varargs_shim.c`, already
  present in shell32.)
- **Never translate a C/C++ debug macro into a no-op `fn`** (field manual §9.65 — out-of-line
  no-op is an optimization barrier that uncloaks ABI mismatches → `ud2`). Use a macro.
- **`Drop` for ATL RAII** (`CComPtr` release, `CoTaskMemFree`) — but release at the *exact*
  original point (`drop(guard)`), not at function end.
- ATL `CComPtr<T>` → an explicit `AddRef`/`Release` pair or a Rust RAII wrapper; be
  deliberate, refcount bugs here are UAF.

## Verification checklist (per swap)

1. `./vibeos-cli check` — includes `scan_magic_offsets` (offset drift), `verify_ffi` (ABI),
   `scan_dangling_transmute`, `scan_silent_failures`. 0-NEW baseline gate.
2. `./vibeos-cli build --arch {i386,amd64,arm64}` — i386 is the ABI canary (stdcall
   decoration); an `extern "C"` vs `extern "system"` slip links silently on amd64/arm64 and
   **breaks only on i386**.
3. `python3 tools/scan_ud2.py --check build-<arch>` — `ud2` in a named function = ABI mismatch.
4. `./vibeos-cli smoke --arch <a>` ×3 arches — shell32 is on the boot path to the desktop;
   the start menu / taskbar / desktop render is the real oracle (see field manual §9.53 for
   how a shell32-adjacent translation bug actually presents).
5. Screenshot check — but note the North Star caveat: the shot is taken at `shell ready`,
   **before** the taskbar paints. An empty tray is not a failure.

## Why Codex drives (what this lane is actually testing)

Two independent things are under test, and the lane should report on both separately:

1. **Can a different agent harness drive rustconv?** Codex takes the seat Claude usually
   occupies: scoping the file, running `translate`, integrating output, building, gating.
   This is the real experiment — everything rustconv needs to be driven well (the paid-call
   discipline, the cohab recipe, the verify checklist) currently lives in AGENTS.md and in
   Claude's habits. If codex can drive it from the docs alone, the pipeline is genuinely
   agent-portable; where codex trips, the docs have a gap worth fixing.
2. **Can rustconv handle C++ at all?** Its prompts, c2rust anchoring, and post-processors
   assume C. ATL templates, multiple inheritance, and vtable layout are outside what it was
   built for. Expect to need a shell32/C++ system prompt; the §"Rust COM shell recipe" is
   deliberately written as prompt-ready context to hand it.

Report back into this doc: what codex got right, what it hallucinated, where the docs failed
to convey a rule, and whether the per-class recipe held. If rustconv can't be made to do C++
cleanly, that's a real result too — it tells us the C++ corpora need a different path.

**Do not let a negative result get papered over.** If rustconv can't anchor a C++ file it
must **hard fail** (AGENTS.md §"Fail loud"), not silently degrade to unanchored translation.

## Pipeline checkpoint (2026-07-15)

The lane now has a deterministic cheap-model acceptance path before any class integration:

- COM interfaces, vtables, object layout, factories, worker signatures, helper closure, and
  wide literals are generated without an LLM.
- `--seal-facts --seal-calls` closes constants/HRESULT predicates and every known runtime
  call occurrence around the paid body worker.
- `candidate-check` batches isolated candidates into one synthetic crate and checks i386,
  amd64, and arm64, with diagnostics attributed per worker.
- `candidate-check --repair` uses only generated/accepted-base/lexical authority. Two fresh
  DeepSeek `GetPath` candidates went raw-red on FFI typing and repair-green on all three
  arches for $0.00494 and $0.00425 respectively.
- The append-only outcome ledger records routing/cost evidence, and `dup-census` measured
  5.9% exact duplicate copies plus 1.2% alpha-only copies across `dll/win32`.

This is a pipeline spike, not authorization to swap `COpenControlPanel` or skip the Wave 1
pilot gate. The next experiment is a stratified 20-method routing corpus; production class
translation still escalates any model-owned audit/compile failure to Sonnet after the fixed
DeepSeek budget.
