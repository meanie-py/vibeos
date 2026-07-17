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

**Activation ownership is all-or-nothing.** One language must own the class factory and
every published vtable. A source file may remain compiled for source-local companion classes
or explicit `extern "C"` bridges while Rust owns the activated class; document every bridge
and prove its decoration with the i386 link.

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
translation still classifies any model-owned audit/compile failure after the fixed DeepSeek
budget before deciding whether stronger-model escalation is warranted.

## Routing corpus checkpoint (2026-07-15)

The 20-method experiment is complete. Ten deterministic templates compiled 10/10 on all
three arches for $0. Ten sealed DeepSeek workers compiled 3/10 raw and 7/10 after bounded
repair; combined acceptance was 17/20 for $0.04757 of fresh DeepSeek spend. Low-difficulty
model workers were 4/4 repair-green, medium 2/3, and high 1/3.

`rustconv skeleton-check` added a cross-language tree-sitter control-skeleton oracle. All
20 candidates preserved the normalized branch/loop/return skeleton, but a compile-red worker
scored 0.9417, so the metric is a veto for obvious structural drops rather than an acceptance
proof.

The three residuals were escalated through a fixed, isolated Sonnet run. All passed the
closed envelopes, but none compiled: two were dominated by C API spellings absent from the
synthetic Rust substrate, and one duplicated an ABI struct locally. Stronger generation did
not overcome missing deterministic context. Production routing should therefore classify
and close substrate gaps before paying for escalation; Sonnet remains for genuine residual
semantic/structural failures after that gate.

## Deterministic substrate replay (2026-07-16)

The missing-context backlog from the routing checkpoint is closed for the measured corpus.
Bounded repair now owns the known wide-length/allocation/byte-copy idioms, COM union and out
pointer adaptations, exact sealed strsafe append formats, ABI struct field/null restoration,
and signed C pointer offsets. Replaying the durable DeepSeek candidates made no paid calls
and compiles **20/20 x3**, so the DeepSeek model tier is 10/10 after repair on this sample.
The successful report is
`.responses/candidate-oracles/20260716-004449-286078/report.json`.

## Monster-class integration checkpoint (2026-07-17)

The first production batch validated the class-level path rather than isolated workers:

- `CNewMenu` and `CSendToMenu` are fully Rust-owned, with their C++ translation units removed
  from shell32 and activation routed through generated Rust factories.
- `CPrinterFolder` is Rust-owned at activation/vtable level. Its C++ translation unit remains
  compiled only for the source-local `CPrintersEnum` companion and three explicit bridges
  (enum construction, extract-icon construction, and the source-local CompareDetails helper).
  The full candidate passed the synthetic compiler on i386, amd64, and arm64 in
  `candidate-oracles/20260717-005929-192066`, then passed a real i386 shell32 link in
  `/tmp/vibeos-cli/20260717-010231-build-i386/full.log`.
- Full paid artifacts for `COpenWithMenu`, `CDrivesFolder`, `CNetFolder`, and `CMenuBand` are
  preserved in `~/vibeos-rust-staging/shell32-com/monsters-v2/`. Their rejections are substrate
  boundaries, not discarded translations: each closes over additional source-local companion
  classes or tables, so the next unit of work is that closure rather than model escalation.
  `CMenuBand` assembled 76 methods (49 cached, 27 deterministic) after $0.02 of fresh DeepSeek
  work; oracle `candidate-oracles/20260717-012346-448391` cleanly reaches the companion
  `CMenuToolbarBase`/`CMenuSFToolbar` boundary on all three targets.

The next monster, `COpenWithMenu`, is now Rust-owned at activation/vtable level while its
translation unit retains the substantial source-local `COpenWithList`/dialog implementation
behind opaque bridges. The paid candidate replayed for $0, reached zero substrate roots, and
compiled synthetically on all three targets in
`candidate-oracles/20260717-094206-576609`; the real i386 shell32 link then passed. This also
closed a scaffold ABI gap: WIDL-declared derived/base interfaces now share one physical slot,
as required for `IContextMenu2`/`IContextMenu`, without guessing from vtable similarity.

`CNetFolder` followed through the same hybrid rule: Rust owns activation, vtables, and the
22 class methods, while the retained C++ unit owns recursive network enumeration and its
context-menu callbacks behind explicit bridges. The current replay is zero-root and x3
synthetic-green in `candidate-oracles/20260717-101620-834612`, and the real i386 shell32
link passes. During the adjacent `CMenuBand` replay, the scanner also learned the legal C++
array spelling `OLECMD name []`; both attached and whitespace-separated parameter brackets
now decay deterministically to pointers before stitching.

The reusable additions include legacy `DECLARE_INTERFACE` discovery, deterministic SDK POD
closure, source-local numeric/table generation, overload/default-argument resolution,
scaffold-owned ATL RAII guards, explicit bridge generation, and idempotent cached-worker
normalization. Function-parameter array decay and multiple-IID aliases sharing one physical
COM interface slot are also scaffold facts. Cheap-model output is now treated as durable
behavior input to a generated class module, never as authority for ABI or ownership.

Model-emitted local ABI types are explicitly rejected. The next routing experiment should
expand beyond the three classes and, importantly, test generated scaffold/substrate
authority where no accepted Rust implementation of the same function exists. Until that
out-of-sample result exists, keep the fixed DeepSeek budget and all mechanical acceptance
gates; do not reinterpret this in-sample 10/10 as permission to bulk-convert shell32.

## Out-of-sample `CCopyAsPathMenu` checkpoint (2026-07-16)

The next Wave-1 source now passes deterministic COM preflight. rustconv owns its
`CComPtr<IDataObject>` field layout, AddRef/Release assignment, final release, sibling worker
binding, and C++ reference signatures. Two bodies lowered for $0; eight sealed DeepSeek body
artifacts are preserved under
`~/vibeos-rust-staging/shell32-com/copy-as-path-pilot/` and the in-repo response cache. An
old parallel fail-fast bug was fixed without regenerating any paid function: all futures are
now drained and individually cached, and declaration-only token externs are removed before
the executable call seal is checked.

The first full-module oracle,
`.responses/candidate-oracles/20260716-004452-273547/report.json`, reported 62 diagnostics
per architecture because this class had no accepted Rust module supplying its environment.
The deterministic substrate now owns ATL `CStringW`, complete shell-item interfaces,
menu/command structs, Win32 FFI/constants, macro lowering, the file-scope verb table, and
the internal-C++ linkage bridges. Replaying the exact cached class cost $0 and reached zero
diagnostics x3 in `.responses/candidate-oracles/20260716-141659-316267/`.

`CCopyAsPathMenu.cpp` is now disabled, its ATL object-map entry activates the generated Rust
factory, and the integrated image passed the complete i386/amd64/arm64 gate at
`/tmp/vibeos-cli/20260716-143231-gate-i386+amd64+arm64/full.log`. A targeted i386 context-menu
probe was inconclusive beyond no-crash because the pre-existing WOW64 32-bit `MENU`/`ITEM`
desktop-heap layout limitation corrupts menu labels. Treat the full class-atomic link and
shell-ready x3 gate as the landed oracle; retain a direct Copy-as-path invocation test as an
owed behavioral oracle once that menu-view limitation no longer masks it.

## Three-class scaling checkpoint (2026-07-16)

`CShellItem`, `CFontsFolder`, and `CAdminToolsFolder` are now Rust-owned activated classes.
Their complete generated modules compile with zero diagnostics on all three target triples,
and the integrated images pass the full shell-ready and network gate at
`/tmp/vibeos-cli/20260716-154657-gate-i386+amd64+arm64/full.log`.

The exact-body translation memory reused 19/22 folder workers across the Fonts/Admin pair,
reducing the second class to three API calls. Semantic review also caught a missed failure-
path `CComPtr` release that compilation and skeleton comparison could not see; body-local
`CComPtr<T>` cleanup is now a source-derived deterministic scope guard. Keep that ownership
guard mandatory for future RAII-bearing classes, and treat exact-memory reuse as an audited
zero-call route rather than a model route.

## Heterogeneous-class checkpoint (2026-07-16)

The broadened corpus now includes helper-heavy, dialog, registry, and FinalConstruct folder
classes. Rejections are informative and durable: raw C control flow, model-owned SDK structs,
and model-owned `CComPtr` representations stay red without retries. A false call seal on a
C++ direct initializer is fixed mechanically.

`CMyDocsFolder` is the accepted heterogeneous result: 25 cached DeepSeek workers plus one
deterministic stub, six generated interfaces / 53 slots, deterministic state ownership and
FinalConstruct failure cleanup, zero compiler diagnostics on all three targets, and 8/8 exact
representative control skeletons. It is wired as the Rust object-map owner; warm builds are
green x3, and the clean integrated build/lint/shell-ready/network gate passed at
`/tmp/vibeos-cli/20260716-173526-gate-i386+amd64+arm64/full.log`.

The paired Sonnet follow-up accepted 0/3 heterogeneous rejects under the same mechanical
gates ($0.127848). Do not route on LOC/branch complexity alone: `SFV_CREATE`, custom `CDrop`
RAII, and shared `goto` cleanup need deterministic type/lifetime/CFG ownership first.
Sonnet remains a residual judgment tier only after that classifier is clean.

A second comparison used five substrate-backed high-band workers and stitched each candidate
into an accepted full module. After bounded repair Sonnet was green x3 on 3/5 versus DeepSeek
2/5, with 2 versus 24 remaining diagnostics per architecture. Sonnet was closer to the
agent-finished body on four cases at 2.55x model cost. The emerging router is therefore:
cheap-first for ordinary workers, but escalate a substrate-clean high-band compile reject
instead of spending agent time repairing broad ownership/type-shape residue.

## Three-class production batch checkpoint (2026-07-16)

A production-shaped batch exercised `CQueryAssociations`, `CFolderOptions`, and
`CRecycleBinCleaner` together (~1,167 C++ lines). Deterministic preflight was extended for
ATL classes declared with `struct`, `OBJECT_ENTRY_AUTO`, fixed scalar-array state, and
recursively referenced WIDL callback interfaces. Internal `*_CreateInstance` classes still
fail loud because their initializer/factory argument contract is not mapped yet.

The router separated the three outcomes without discarding artifacts:

- `CQueryAssociations::GetString` (231 nonblank lines, 94 calls, 72 sealed call occurrences)
  damaged the call envelope with DeepSeek; the already-preserved Sonnet trial had also
  failed the same envelope. It is owed deterministic CFG decomposition, not another retry.
- `CFolderOptions` assembled entirely from the durable cache for $0, but remains substrate-
  red on property-sheet and browser-service ABI. Its model work is preserved for replay.
- `CRecycleBinCleaner` was the fresh accepted class: seven DeepSeek calls cost about $0.02
  and all passed fact/call/body audit. Source-derived support for callback vtables, resource
  strings, fixed arrays, recycle-bin/ShellExecute structs and APIs, implicit sibling
  receivers, and signed SDK fields reduced 24 unique compiler diagnostics to zero on
  i386/amd64/arm64 without regenerating a worker. ATL registration now delegates activation
  to the Rust factory; all three real shell32 images link.

This is the first batch where the useful result is deliberately heterogeneous: one accepted
class, one reusable substrate backlog, and one deterministic-decomposition reject. Bulk
throughput should count all three outcomes rather than treating non-integration as lost work.

## Monster-class checkpoint (2026-07-16)

`CShellLink.cpp` is the first giant-class stress case: 3,323 source lines, 13 COM
interfaces, 219 generated vtable slots, and a 91-worker prepared closure. rustconv now
extracts inline `STDMETHOD` bodies, derives packed embedded POD state, binds overloaded
interface methods to deterministic `_vN` workers, and routes every vtable slot without
asking a model to reproduce ABI. The complete closure is accepted: four proven stubs at
$0, 84 DeepSeek workers, and three Sonnet residuals. The outcome ledger records $0.72259518
for all 98 attempted calls including 12 audit rejects; accepted output alone is $0.5662.

The routing result is more useful than a raw acceptance percentage. Sonnet repaired
`SetAdvertiseInfo`, ANSI `GetPath`, and `GetIconLocation`, but independently repeated the
source-local `sized_chunk` declaration in `Stream_ReadChunk` and damaged the 26-call cleanup
envelope in `CreateShortcutIcon`. The first was an auditor false reject; the second remained
a true whole-worker reject. rustconv now permits only exact source-declared local POD field
shapes, removes model shadows of source-named SDK types, and replays already-paid `.partial`
streams through newer deterministic audits before making another call. That replay recovered
the earlier complete DeepSeek `CreateShortcutIcon` output for $0 and closed 91/91.

The full cached module is durable at
`~/vibeos-rust-staging/shell32-com/monsters-v1/CShellLink-full.rs`. It is not integration-ready:
the baseline full-module oracle at
`.responses/candidate-oracles/20260716-212005-236836/report.json` reported 743 diagnostics per
architecture (674 unresolved substrate symbols, 48 type mismatches, 21 other). The first
language-neutral substrate-closure pass added cross-arch diagnostic-root clustering,
authority indexing, source/header selection, canonical `com-sys` reuse, unique literal SDK
facts, and simple recursive SDK POD generation. A $0 replay at
`.responses/candidate-oracles/20260716-220619-970061/report.json` now reports 492 diagnostics
per architecture (309 unresolved, 152 type mismatches, 31 other) and only 51 distinct
unresolved roots, down from 164. This exposes deeper type errors while shrinking the actual
authority backlog; it is still not permission to wire the class. Follow
`docs/engineering/rustconv-substrate-closure-protocol.md` for the closure loop.

`CDefaultContextMenu.cpp` is the
contrasting deterministic reject: aggregation/internal activation and two
`CAtlList<...>` owners fail preflight before any model call. Add an explicit C++ list/factory
bridge before spending on that class; never reinterpret those owners as Rust `Vec`.

## Folder-options hybrid checkpoint (2026-07-17)

`CFolderOptions` replayed entirely from the durable cache for $0 and now has Rust-owned COM
identity, three published interfaces, refcounting, site ownership, activation, and trivial
methods. Its earlier synthetic rejection correctly identified two non-Rust-shaped surfaces:
property-page callbacks retain the object as an opaque dialog parameter, while the private
`IBrowserService2` dependency pulls a very large inherited SDK vtable into an otherwise small
class. Both are now contained behind narrow C ABI bridges instead of being guessed or passed
through the model. The view dialog calls exported Rust helpers rather than casting the opaque
parameter to the old C++ object layout.

This establishes a reusable hybrid rule: when C++ callbacks retain a Rust-owned COM object,
store its first published interface pointer, balance lifetime through `IUnknown`, and expose
every later operation through an explicit ABI helper. Never invoke nonvirtual C++ methods or
read C++ fields through that pointer. WIDL tag spellings such as `struct tagNMHDR *` are also
normalized deterministically to valid Rust tag names before candidate assembly.

## Second monster checkpoint: complete substrate closure (2026-07-16)

`CNewMenu.cpp` is the first giant class whose newly assembled module is compile-clean in the
synthetic oracle: three interfaces, 141 generated vtable slots, 23 prepared workers, and
2,206 lines of stitched Rust. Twenty-one bodies came from the durable DeepSeek cache and one
proven stub was generated for $0; replay after each harness improvement made no new model
calls. The full-module oracle at
`.responses/candidate-oracles/20260716-233727-653342/report.json` reports zero diagnostics on
i386, amd64, and arm64.

The closure work was reusable rather than class-shaped patching: nested source POD/enums,
transitive WIDL interface inheritance checked across all generated architectures, header-
owned UTF-16 statics, pointer/size wrappers, ATL `CStringW` operations, qualified C++ pointer
returns, repeated pure `wcslen`, and a call-nesting audit that rejects argument/receiver
reversal while allowing valid hoisting. The candidate was subsequently integrated as recorded
in the 2026-07-17 checkpoint above; the final three-architecture image gate remains the
landing authority.
