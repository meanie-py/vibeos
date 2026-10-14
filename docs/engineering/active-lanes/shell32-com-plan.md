# shell32 COM/ATL C++ → Rust (lane: `shell32-com`)

**Status:** COMPLETE (opened 2026-07-14, closed 2026-07-18). All 33 activatable
`shell32` object-map entries are Rust-owned. Worktree `vibeos-shell32-com`, branch
`shell32-com`.
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

## User-notification callback checkpoint (2026-07-17)

`CUserNotification` is the first class in this lane with Rust-owned ATL `CStringW` state
and a static Win32 worker-window callback. rustconv now constructs, assigns, and explicitly
drops owned strings around the raw COM allocation; strips legacy `IN`/`OUT`/`OPTIONAL`
parameter annotations before signature generation; and derives the callback adapter,
`NOTIFYICONDATAW` layout, message types, constants, and imports deterministically. On i386,
the generated imports bind `GetWindowLongPtrW`/`SetWindowLongPtrW` to their real
`GetWindowLongW`/`SetWindowLongW` exports, matching the SDK macro alias.

The unchanged paid workers reached zero synthetic diagnostics on i386, amd64, and arm64
after compiler-directed substrate closure. The real i386 shell32 link then caught only the
32-bit export alias above, which is now a reusable generator rule rather than a candidate
patch. This is the preferred shape for callback-heavy classes: generate the ABI, callback,
state lifecycle, SDK records, and imports; ask the model only for method control flow.

The next `$0` replay target is `CDrivesFolder` (1,455 source lines, 40 prepared functions).
All 35 cached bodies plus five deterministic stubs reassemble without an API call. This
replay found and fixed two harness roots before class closure: cached `CComPtr` normalization
could duplicate guards/nulling, and `HRESULT WINAPI CDrivesFolder::FinalConstruct()` lost its
return prefix when tree-sitter began the method node mid-line. The refreshed paid module is
preserved at `~/vibeos-rust-staging/shell32-com/monsters-v2/CDrivesFolder-full.rs`. Its next
closure pass must handle shared PIDL/helper authority, the source-local physical-drive
enumerator bridge, and compiler-directed PIDL/IID/union type repairs; do not regenerate it.

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

## CDrivesFolder / CQueryAssociations / CShellLink landed; CMenuBand deferred (2026-07-17)

Three more classes are Rust-owned and validated (i386 build+boot green; CDrives/CQuery also
amd64+arm64):

- **CDrivesFolder** (`CLSID_MyComputer`) — hybrid cohab. Rust owns all five published
  interfaces + state + activation; retained C++ keeps `CDrivesFolderEnum`, the drives
  context-menu/eject/format cluster, `CDrivesExtractIcon`, `CRegFolder` composition, and the
  C++-mangled folder helpers behind `*_RustBridge` shims (added to `CDrivesFolder.cpp`,
  `shlfolder.cpp`, `utils.cpp`). Stitching fixed five real cached-body bugs vs the C++
  (dropped `pidlRoot` store, reg-enum double-release, inverted GetDriveLabel priority,
  dropped `SFVM_DEFVIEWMODE`, wrong `PIDLDATA` casts in `CompareIDs`).
- **CQueryAssociations** (`CLSID_QueryAssociations`) — full Rust ownership; `GetString`
  hand-ported statement-by-statement (both DeepSeek and Sonnet had mangled its 72-call
  envelope) preserving the exact `pcchOut` probe/truncate/NOTRUNCATE contract.
- **CShellLink** (`CLSID_ShellLink`) — hybrid cohab (approach a): `CShellLink.cpp` stays
  compiled with dead C++ class bodies + retained free helpers; the two C-linkage
  `IShellLink_ConstructFromPath/FromFile` are `#if 0`'d (Rust owns them); construction
  rerouted at `CFSDropTarget.cpp:697`; one `SHGetNameAndFlagsW` bridge in `utils.cpp`.
  Runtime-exercised: the desktop `.lnk` shortcuts (Internet Browser / Command Prompt /
  Read Me) render through the Rust class on i386.

### CMenuBand — RESOLVED (relanded 2026-07-18 via shared `menuband-rust` staticlib, commit `3aa11392971`)

The shared-staticlib fix below was implemented: `menu_band.rs` now lives in a standalone
`menuband-rust` crate (`sdk/lib/rust/menuband-rust/`, `crate-type=["staticlib"]`, its own
`MenuBandRustModuleLock/Unlock` + panic-handler + `rust_eh_personality` stub) built once at the
top-level `CMakeLists.txt` (before `add_subdirectory(base)`) into `${MENUBAND_RUST_LIB}`, and
linked by BOTH `shell32.dll` and `base/shell/rshell/rshell.dll`. Each DLL provides its own
`MenuBandRustModuleLock/Unlock` over its COM module (`shell32.cpp` / rshell `misc.cpp`).
Green ×3, full desktop + taskbar + Start menu on all arches. This is the reusable
**shared-shellmenu-staticlib pattern** for the remaining shared `shellmenu` classes
(CMenuSite / CMenuDeskBar / CMergedFolder). The historical deferral analysis is retained below.

### CMenuBand deferral analysis (historical — needs a SHARED Rust staticlib)

`CMenuBand` was integrated by an agent (branch `worktree-agent-ad4910884ddab9c81`, commit
`7c278907ca5`; module preserved at `~/vibeos-rust-staging/shell32-com/monsters-v2/
CMenuBand-integrated.rs` + `CMenuBand-bridges.md` + `CMenuBand-integration-notes.md`) and the
merge was **reverted** (`4e4755b8d33`) because it breaks the OS build. Root cause — CMenuBand
is NOT a shell32-private COM class: `CMenuBand.cpp` lived in the **shared `shellmenu` OBJECT
library**, so the C++ class compiled into BOTH `shell32.dll` AND `base/shell/rshell/
rshell.dll`. The cutover moved the class to `menu_band.rs` inside `libshell32_rust.a` (which
only shell32 links) and patched the shared `CMenuToolbars.cpp`/`CMenuFocusManager.cpp` to call
the Rust exports — so `rshell.dll` (which links those objects but not the rust lib) fails with
~19 undefined `CMenuBand_*_RustExport` symbols, and the whole livecd won't build. Naively
linking `libshell32_rust.a` into rshell doesn't work either: `menu_band.rs` calls
shell32-internal C symbols (`Shell32RustObjectLock`, wine/pidl helpers) absent in rshell.

**Correct fix (owed):** promote `menu_band.rs` into its own standalone Rust staticlib
(`menuband-rust`, self-contained: its own object-lock + no shell32-private deps) that BOTH
shell32 and rshell link. This is the first "shared shell-infrastructure" class in the lane;
the shell32-private RustEntry pattern doesn't cover it. The stitched module itself is green
×3 in its oracle — only the link topology is unsolved.

Diagnosis note (correcting an earlier mis-read): the `undefined symbol: CMenuBand_*_RustExport`
link failures were ALWAYS `base/shell/rshell/rshell.dll`, NEVER shell32.dll — verified across
three build logs (`FAILED: base/shell/rshell/rshell.dll` every time). shell32.dll links fine
(it has `libshell32_rust.a`). There is NO cargo-rewrite-in-place build race; the single root
cause is the shared-DLL topology above. Do not add a temp-write/atomic-rename "fix" to the
staticlib custom command — there is nothing to fix there.

## CControlPanelFolder landed (2026-07-18)

**CControlPanelFolder** (`CLSID_ControlPanel`) — hybrid cohab, standard shell32-private
RustEntry pattern (like CDrivesFolder). Rust (`control_panel_folder.rs`) owns the folder's
three published interfaces (IShellFolder2 / IPersistFolder2 / IShellFolderViewCB) + state +
activation; `CControlPanelFolder.cpp` stays compiled with dead C++ folder bodies and the
retained companions `CControlPanelEnum` (enumerator) + `CCPLItemMenu` (context menu) + the
`CCPLExtractIcon`/free helpers. `COpenControlPanel` in the same TU was already Rust
(`open_control_panel.rs`) — undisturbed. Integration: `shell32.cpp` gets
`CControlPanelFolderRustEntry` + object-map swap; four `*_RustBridge` shims appended to
`CControlPanelFolder.cpp` (`CreateRegFolder` = CRegFolder aggregation, `CControlPanelEnum`
create, `CCPLItemMenu` create, `CCPLExtractIcon` create); the shared
`SH32_GetDetailsOfPKeyAsVariant_RustBridge` (shlfolder.cpp) reused as-is.

Pipeline: bulk-translated with `translate --cpp-com` (DeepSeek, 25 fns, all cached), then a
Sonnet subagent-stitch drove the module to green ×3 in an isolated candidate-oracle sandbox
(44 arch-identical errors → 0), fixing ~14 real cached-body bugs vs the C++ (notably:
`MessageSFVCB` took the dead `#else` branch — corrected to the active
`ROSPOLICY_CONTROLSFOLDER_DEFLARGEICONS=1` → `FVM_ICON`; `GetAttributesOf` passed element-0
instead of the walked pidl; `GetUIObjectOf` never dereferenced `apidl[0]`; malformed
`GetDetailsEx` vtable call; packed-`PIDLCPanelStruct` field refs → `addr_of!`;
`SHELLDETAILS.str__`→`str_`). One integrator fix on top: `_ILGetType` is `static inline` in
`wine/pidl.h:144` (no global symbol — `wine/pidl.c` is cohabbed out), so it became a local
inline in the module rather than an extern decl to a nonexistent symbol. Validated: build
green ×3 (i386/amd64/arm64).

## rustconv enhancement backlog (from the 2026-07-17 4-class session)

Footguns the subagent-stitch flow hit this session. The stitch method itself is the engine
(it's how the port reached ~4M lines) — these are about filing down the sharp edges around it,
NOT replacing it. Owner: whoever next picks up `tools/rustconv/` (paused here 2026-07-17 while
it's aimed at the winsock C work; do not edit rustconv from this lane meanwhile — conflict
risk). Priority-ordered:

1. **Link-topology / shared-symbol preflight (highest leverage — this is what sank CMenuBand).**
   The candidate oracle proves a module compiles *in isolation* but is blind to *who links the
   object files that reference its exports*. `CMenuBand.cpp` lived in the shared `shellmenu`
   OBJECT lib, compiled into BOTH `shell32.dll` and `rshell.dll`; the cutover only gave the
   Rust class to shell32, so rshell failed to link. A preflight check — "for class C, which
   final link targets pull the TU / the objects that call C's methods?" — would flag a
   shared-infrastructure class BEFORE any spend or integration, and tell the agent it needs the
   shared-staticlib pattern instead of the shell32-private RustEntry pattern. Cheap to build
   from the CMake target graph; would have saved the whole CMenuBand round-trip.

2. **ATL-inheritance / non-trivial-base substrate.** Preflight fails loud (correctly) on
   `CWindowImpl<...>` (CMenuDeskBar), `IDispatchImpl<...>` (CShellDispatch), and shared base
   classes (CMoveToMenu : CCopyMoveToMenu). The remaining shell32 tail is mostly these. A
   deterministic inherited-layout mapper (compute the base vtable/field prefix from the base's
   own translation or header) is the unlock for that whole cohort.

3. **Callback/WNDPROC state substrate.** `CCopyToMenu` fails on a `WNDPROC m_fnOldWndProc`
   member — the same shape `CUserNotification` solved with a static worker-window callback.
   Generalize that solution into reusable substrate so window-subclassing classes stop failing
   preflight.

4. **Scanner false-positive on ctor-in-header-and-impl** (`CMenuSite`: "method body defined in
   both implementation and header"). Looked like a cheap scanner fix — a class whose ctor is
   declared in the header and defined in the .cpp trips a duplicate-body detector. Unblocks a
   small (366-line) shared shellmenu class.

(Superseded — see the diagnosis note in the CMenuBand section: the apparent "build race" was
a mis-read; the failures were all rshell.dll from the shared-DLL topology, not a cargo
in-place-rewrite. No CMakeLists staticlib fix is warranted. Item #1 (link-topology preflight)
is the real lesson.)

## Environment gotcha: `rustconv translate --cpp-com` needs `rg` on PATH (2026-07-18)

`translate`'s final assembly step (`cpp_com.render_cpp_support` → `_sdk_numeric_definitions`,
`cpp_com.py:2406`) shells out to `rg` to resolve referenced SDK numeric `#define`s in one
indexed header pass. If ripgrep isn't installed it dies with
`PermissionError: [Errno 13] Permission denied: 'rg'` (or FileNotFound) — *after* every
per-function paid call has already streamed to `.responses/`, so no spend is lost, but the
`-o` module never gets written. This box (curi's WSL) has no `rg`. Fix without touching
`tools/rustconv/`: drop a small ripgrep-compatible shim at `~/.local/bin/rg` (on PATH) that
maps the `rg -n --no-heading --glob '*.h' <pat> <roots>` shape onto `grep -rnP --include=`.
Re-run `translate` with the shim on PATH — all functions replay from cache for $0 and the
assembly completes. (A single `grep -P` shim is enough; the invocation only uses `-n`,
`--no-heading`, `--glob`.)

## Substrate-unblock session (2026-07-18, autonomous) — CRecycleBin landed + 4 substrate capabilities + folder-cohort preflight unblocked

The "every remaining class needs substrate" tarpit was attacked directly (rustconv back in
this lane). Four general, tested cpp-com substrate capabilities were added (each check-tooling
green, regression-checked against the landed folders):

1. **Overload-aware header/impl method merge** — a method is `(name, arity)`; a static inline
   `GetClassID()` no longer collides with the virtual `GetClassID(CLSID*)`. (commit a35f5f8d2ca)
2. **C++ `bool` state fields** + **raw-pointer-array state** (`IShellFolder *m_cache[N]` →
   `[*mut T; N]`, lifetime in the translated ctor/dtor) + **constant-expression array bounds**
   (`('Z'-'A'+1)` via a whitelisted AST evaluator, searching the class's own DLL-module tree). (a35f5f8d2ca)
3. **Static class methods never bind to a vtable slot** — emitted as sibling helpers. (4b9d693956d)
4. **Non-QI'd interface bases are materialized layout-only** — a base that is a real WIDL
   interface but absent from the COM map (e.g. `IContextMenuCB` on CFSFolder/CDesktopFolder)
   gets its own vtable subobject (struct field + VTBL + offset_of! thunks + ctor init) but is
   NOT in QueryInterface. Field order is irrelevant (Rust-allocated; every iface pointer
   recovers `this` via offset_of!). (commit 4daf21f41be)

**CRecycleBin** (`CLSID_RecycleBin`) landed — hybrid cohab, 15 `*_RustBridge` shims (5 companion
create/init in CRecycleBin.cpp; 6 CFSFolder-static reuse + relocated `SHELL32_CreateSimpleIDListFromPath`
in CFSFolder.cpp; 1 each in shlfolder.cpp/utils.cpp/item_prop.cpp/recycler_prop.cpp). No CRegFolder
composition (unlike the other folders — CRecycleBin::Initialize only ILClones). Translated with a
mixed model (deepseek bulk + sonnet cleanup of 5 rejected fns — deepseek damaged CComPtr/fact
envelopes on the hard functions), stitched 151→0 errors ×3. Note: `GetRecycleBinPathFromDriveNumber`
is `EXTERN_C` **cdecl** (no WINAPI) — declare it `extern "C"` in Rust, not `extern "system"`, or i386
gets a stdcall-decoration mismatch.

### Folder cohort (CFSFolder / CDesktopFolder): preflight UNBLOCKED, translate BLOCKED (owed)

The IContextMenuCB layout-only fix (#4) makes **CFSFolder preflight PASS** (the file-system folder —
the highest-value remaining class). But the translate is blocked by a shared root cause worth a
focused fix:
- **Header-inline `STDMETHODIMP … override { }` accessors** (IItemNameLimits `GetMaxLength`/
  `GetValidCharacters`; the `static inline CompareUiStrings(…, LPARAM lParam = 0)`): sonnet
  *systematically* mistranslates these short functions — drops `S_OK` / drops a sealed fact token —
  and CDesktopFolder's scanner doesn't even extract the `STDMETHODIMP` header inlines ("COM map
  requires method bodies the scanner did not extract: GetMaxLength, GetValidCharacters"). Likely a
  header-inline extraction/prompting issue affecting both folders.
- **The sonnet (sequential) translate path ABORTS on the first audit-reject**, unlike the DeepSeek
  parallel path which collects rejects + continues (preserving all accepted work). `--concurrency`
  is DeepSeek-only (`__main__.py:1438`); the sequential audit handler (`__main__.py:2215-2223`)
  `raise`s instead of collect+continue. Mirroring the parallel path's collect behavior there would
  let a sonnet class translate its good functions in one run and surface all rejects at once —
  worth doing (general robustness, not just CFSFolder). Partial CFSFolder paid work is preserved in
  `tools/rustconv/.responses/folders__CFSFolder__*`.

Next-session order for the folder cohort: (a) fix the header-inline STDMETHODIMP extraction +
sequential collect-not-abort, (b) re-translate CFSFolder (sonnet), hand-fix the 2-3 stubborn
accessors, (c) stitch + integrate (large: CFileSysEnum + CFSDropTarget companions, many CFSFolder-
static bridges already exist from CRecycleBin).

## Folder cohort LANDED — CFSFolder + CDesktopFolder both Rust-owned (2026-07-18)

Both folders above are **done and on master**. The two blockers in the prior section were fixed
as general rustconv capabilities first (commit `22fb815ec25`):

1. **Header-inline STDMETHODIMP extraction** (`scanner.py`) — a macro-heavy multi-class ATL header
   (CDesktopFolder.h holds CDesktopFolder + CDesktopFolderViewCB + more) defeats tree-sitter's
   class parse and collapses the whole region into ONE flat `ERROR` node; the `class NAME : … {`
   headers survive as flat child tokens but the `class_specifier` structure (and the matching `}`)
   are gone, so every inline method came back `cpp_class=None` and was dropped → preflight hard-fail.
   Fix: brace-balanced class-owner recovery inside ERROR nodes (`_matching_brace_byte` /
   `_error_node_class_owner`) — attribute each inline method to the class whose `{`..`}` body
   encloses it; a helper past a class's closing brace is left unowned (not mis-attributed). CFSFolder
   (clean parse) unaffected; CDesktopFolder inlines now attribute correctly.
2. **Sonnet collect-not-abort** (`__main__.py`) — the sequential body-audit path `raise`d on the
   first reject, abandoning the rest of an already-paid sonnet run. Now mirrors the DeepSeek parallel
   path: record each reject, continue, surface all at once after the loop.

- **CFSFolder** (`CLSID_ShellFSFolder`, commit `c2b2a5e1653`) — 48 workers, 9 interfaces, 194 slots.
- **CDesktopFolder** (`CLSID_ShellDesktop`, commit `5fc33fdf35e`) — 36 workers, 6 interfaces; closed
  in ONE oracle pass by copying CFSFolder's just-landed substrate/bridges.

Both: translate --cpp-com (DeepSeek bulk + Sonnet cleanup of the CComPtr rejects, ~$0.04 / ~$0.11),
then a **closure subagent** drove the full module 240→0 / 144→0 in the candidate oracle, then a
**reconciliation subagent** created the C++ bridges + `#[link_name]` redirects to a clean i386 link.
Gate ×3 green (boot to desktop on all arches).

### Reusable integration playbook proven by this cohort (use for the next folder-shaped class)
- **`--repair` is a NO-OP for `full_module` candidates** (`candidate_oracle.py:258-263` writes the
  candidate verbatim, never calls `repair_worker`). It still computes the rich `substrate_analysis`
  plan, but you resolve everything BY HAND — mine decls from the green sibling modules + the C++
  prototypes + SDK headers, append a substrate block. Do not expect auto-repair to close a full module.
- **Bridge reconciliation is the bulk of integration.** The generated module calls shell32-internal
  C/C++ helpers by raw name; they don't link. Resolutions, in priority order: (1) `#[link_name =
  "X_RustBridge"]` to an EXISTING bridge (grep the .cpp for `X_RustBridge`); (2) a local `unsafe fn`
  copied verbatim from a sibling (`SHILClone`, `StringCchCatW`, `__SHFreeAndNil`, and every
  `static inline` wine/pidl helper like `_ILGetType`/`_ILIsEmpty` — these have NO external symbol);
  (3) `#[link_name]` to the real export for macro-aliased names (`sprintfW`→`_swprintf`,
  `PathAppend`→`PathAppendW`, `CLSID_NULL`→`GUID_NULL`); (4) a NEW `extern "C" <ret> WINAPI
  X_RustBridge(...)` shim matched to the exact C++ prototype. i386 is the decoration canary
  (`@N` stdcall vs cdecl); amd64/arm64 can still surface an i386-only CRT symbol (`_alloca` — the C++
  VLA; replace with a fixed `[WCHAR; MAX_PATH]` stack buffer, portable + leak-free).
- **Delegate closure + reconciliation to subagents** (proven high-quality here): one drives the module
  to oracle-green (fixing real cached-body bugs + hand-closing substrate), one creates the bridges to
  a green i386 link. You own the judgment parts: module copy + `pub mod` + the RustEntry adapter +
  `OBJECT_ENTRY` swap, and the ×3 gate. VERIFY the ABI-critical fixes yourself (vtable-subobject
  arithmetic, hand-written SDK struct layouts vs the header, the container_of offset_of) — a wrong
  "fix" links but crashes; smoke ×3 is the runtime oracle.
- The closure subagents caught real **runtime-behavior divergences** the compiler can't (a
  double-release/UAF, a wrong-vtable context-menu dispatch, a dead URL-scheme match, a 2-byte-early
  PIDL read). Always have the closure agent diff each body against the C++ and report divergences —
  that's the highest-value output, not just compile-green.

### Remaining activatable shell32 classes — all capability-gated (no quick pipeline wins)
Survey 2026-07-18 (22→24/33 object-map entries now Rust). Every remaining one is BLOCKED before the
proven pipeline can run — ordered by tractability:
- **CDrvDefExt** (`CLSID_ShellDrvDefExt`) — preflight PASSES; translate hard-fails pre-payment on a
  **body-local custom-RAII struct** (`CDrop`: ctor `IDataObject::GetData`, dtor `ReleaseStgMedium`,
  `GetCount`). Needs deterministic custom-RAII-struct lowering (translate the local struct → Rust
  struct + `Drop` + methods, rewrite usage). Bespoke but bounded — the single most tractable next class.
- **CFileDefExt / CShellDispatch / CMenuDeskBar / CMoveToMenu / CCopyToMenu** — all fail preflight on
  `custom/inherited base class … needs deterministic inherited layout mapping` (CObjectWithSiteBase /
  IDispatchImpl / CWindowImpl / CCopyMoveToMenu). The inherited-layout mapper (enhancement-backlog #2)
  is necessary but **NOT sufficient** — each has a SECONDARY hard blocker: CFileDefExt embeds a C++
  object member (`CFileVersionInfo m_VerInfo`) + `= NULL` init-state; CShellDispatch needs a typelib;
  CMenuDeskBar needs window machinery + is shared-shellmenu (staticlib); CCopyToMenu has a
  `WNDPROC m_fnOldWndProc`. None land from the mapper alone.
- **CMenuSite / CMergedFolder** — shared `shellmenu` (need the `menuband-rust` shared-staticlib pattern)
  + a scanner ctor-in-header-and-impl false positive (CMenuSite) / a WIDL `IAugmentedShellFolder`
  flattened-vtable gap (CMergedFolder).

## CDrvDefExt + CMergedFolder LANDED (2026-07-18 PM) — 24→26/33; two new capabilities

Both "capability-gated" classes from the survey above landed the same day by building the capability
instead of waiting on it:

**CDrvDefExt** (`CLSID_ShellDrvDefExt`, drive-properties sheet) via **`--local-raii`** (new rustconv
capability): a hand-verified Rust lowering of the file-local RAII struct is supplied at
`tools/rustconv/provided_raii/CDrop.rs`; `translate --cpp-com --local-raii CDrop=<path>` (a) lifts the
pre-payment reject, (b) seals the exact Rust API into every affected worker prompt
(`provided_raii_contract`, cpp_com.py), and (c) splices the definition verbatim into the assembled
module. The model maps usage onto a fixed API instead of inventing drop semantics. GOTCHAS hit:
resume-cache replays PRE-capability worker outputs (cache key is (unit, body) — the sealed contract
isn't in the key), so re-run affected workers with `--fn <a,b> --no-resume`; the postprocessor slaps
`#[no_mangle] extern "C"` onto inherent methods of the spliced struct (strip it by hand). Closure
subagent caught 5 real divergences: `M_PI` generated as `DWORD = 3`, Rust bitwise-`!` inverting the
C++ logical-not on `EM_SETREADONLY`, a `%c` formatter dropped by the generated StringCbPrintfW
helper, a missed PropSheetPageLifetimeCallback leak-guard AddRef, and Release through a nonexistent
lpVtbl. New bridges: SH_FormatByteSize / ChkDskDlg (dialogs/drvdefext.cpp),
CDrivesFolder_SetDriveLabel (folders/CDrivesFolder.cpp); reused IsDriveFloppyW + SHGetNameAndFlagsW.
Module at `shell32-rust/src/dialogs/drvdefext.rs`, wired as `#[path]` mod `drv_def_ext`.

**CMergedFolder** (`CLSID_MergedFolder`, shellmenu shared-staticlib) via **IDL promotion**: the
C++-only `IAugmentedShellFolder{,2,3}` + `QUERYNAMESPACEINFO` decls moved from CMergedFolder.h to
`sdk/include/reactos/shobjidl_undoc.idl` (built by the existing `add_idl_headers(reactos_idls ...)`;
CMergedFolder.h now includes the generated `<shobjidl_undoc.h>`), giving WIDL ownership of the vtable
ABI and the preflight its flattened-vtable ground truth (`discover_widl_header` gained the
`build-<arch>/sdk/include/reactos` root). widl gotcha: `PULONG` isn't an IDL type — spell `ULONG *`.
Integration mirrors CMenuBand: module lives in **menuband-rust** (`merged_folder.rs`, linked into BOTH
shell32.dll and rshell.dll), uses the dual-host `MenuBandRustModuleLock/Unlock` pair (NOT
`Shell32RustObjectLock` — that exists only in shell32.dll), C++ `RSHELL_CMergedFolder_CreateInstance`
now forwards to the Rust 3-arg factory (covers the rshell/browseui GetProcAddress path + spec export),
object-map swap to `CMergedFolderRustEntry` (`#undef CMergedFolder_CreateInstance` first — shellmenu.h
macro-owns the 2-arg spelling). The retained C++ companion `CEnumMergedFolder` is reached via 6
bridges (CreateInstance/SetSources/Begin/FindByName/FindPidlInList/QueryInterface) — its methods are
plain non-virtual C++, NEVER vtable slots (the model wrongly invented `lpVtbl` calls for them; every
one was rewritten to a bridge). Its object pointer doubles as IUnknown* because IEnumIDList is the
Itanium primary base (CComObjectRootEx is non-polymorphic) — the CreateInstance bridge asserts that
layout at runtime. Upstream quirks mirrored faithfully (do NOT "fix"): ParseDisplayName assigns the
`GetAttributesOf` HRESULT into `*pdwAttributes`; GetCurFolder hands out the raw `m_shellPidl`.

**Prover widened for the next cohort** (no class landed by these alone): `CComHeapPtr<T>` state
fields (owned raw pointer + CoTaskMemFree in generated Release), `WNDPROC`-style fn-pointer typedefs
(`_KNOWN_POINTER_TYPEDEFS`), and zero-valued C++11 default member initializers (`= NULL/nullptr/0/
FALSE` — exactly the scaffold's zero-init) all prove now.

## CMoveToMenu + CCopyToMenu LANDED (2026-07-18 eve) — 26→28/33 via the inherited-layout mapper

The **concrete-local-base mapper** is built (same-day capability→classes again): when a base is a
concrete non-template class defined in the SAME header (`CCopyMoveToMenu`), `analyze_class` flattens
it — base state fields precede the derived's (shadowing = loud preflight failure), the base's own
bases run through the same skip/interface/reject loop, FinalConstruct/Release scanned in both — and
`_extend_cpp_class_scan` merges + RE-OWNS base method bodies as derived-class methods, so vtable
binding, worker naming, and `this` typing all see one flat class. Devirtualization of the tiny
`GetFileOp()`-style pure-virtual calls falls out of the ordinary same-file arity binder (the derived
inline overrides are in the closure via the header-inline extraction fix). Special members: C++ runs
base-then-derived ctors — a trivially-empty derived ctor/dtor yields to the base's; two non-trivial
bodies fail loud (no sequencing generated). ATL template bases (CWindowImpl/IDispatchImpl/
CObjectWithSiteBase) still fail loud — this mapper is same-header-concrete ONLY.

- CCopyToMenu's 3 shadowed base fields (`m_pidlFolder`/`m_fnOldWndProc`/`m_bIgnoreTextBoxChange`)
  were verified DEAD (its only method forwards to the base impl) and deleted upstream in
  CCopyMoveToMenu.h — the mapper's shadowing reject pointed straight at them.
- Twin translation economics: CCopyToMenu reused 9/12 of CMoveToMenu's accepted workers via the
  exact-body translation memory ($0); whole pair ≈ $0.15 including two Sonnet envelope cleanups.
  When a worker rejects for one twin but passed for the other, just RE-RUN the rejecting twin with
  resume on — the memory seeds the accepted sibling body at $0 (worked for DoRealFileOp).
- Closure agent caught + fixed 4 real divergences: Initialize had the SHILClone/ILRemoveLastID
  branches SWAPPED (would mutate the caller's const pidl in-place and never clone); the two
  file-static callbacks (`BrowseCallbackProc`, `WindowProc`) were missing entirely (scanner gap —
  hand-translated); CStringW resource loads used the process EXE instead of `shell32_hInstance`;
  `GetVerb` returned a `*const u8` for LPCSTR. First HIDA consumer in Rust: `CDataObjectHIDA`
  rebuilt as a module-local RAII struct over 2 new bridges (+`SHELL_CIDA_ILCloneFull`, all in
  shldataobject.cpp) — its Drop calls DestroyCIDA UNCONDITIONALLY like the C++ dtor (even with a
  null CIDA the medium is released).
- **Resume-cache contract stamping** (root-cause fix for the --local-raii stale-replay gotcha):
  `ResponseCache` entries now record a hash of the worker's deterministic contract; a changed
  contract is a MISS. Pre-stamping entries still hit (banked paid work keeps its value).

Remaining 5 (all still capability-gated): **CFileDefExt**, **CMenuSite / CMenuDeskBar** (CWindowImpl
window machinery), **CShellDispatch** (IDispatchImpl/typelib), and note CMergedFolder's old slot is
done.

### CFileDefExt — sharpened path (scoped 2026-07-18 eve; 4 bounded mapper extensions, next session)
Much closer than the survey suggested. Ground truth checked:
- `CObjectWithSiteBase` (shellutils.h:625) is a tiny CONCRETE base: one field (`IUnknown *m_pUnkSite`),
  zero-init-equivalent ctor (`: m_pUnkSite(NULL) {}`), inline SetSite/GetSite, dtor `{ SetSite(NULL); }`.
- `CFileVersionInfo m_VerInfo` is **POD-layout** (`PVOID + WORD + WORD + WCHAR[64]`) with its 4 methods
  (Load/GetString/GetFixedInfo/GetLangName) defined in the SAME TU, inline ctor = zero-init, dtor =
  conditional HeapFree.
Sub-capabilities needed (in order):
1. **Out-of-header concrete base**: `_local_concrete_base` falls back to `find_class_header(source,
   base)` when the base isn't in the derived's header; `_extend_cpp_class_scan` merges inlines from
   EACH base's own header (today it scans only the derived's).
2. **Zero-init-equivalent ctor triviality**: an init list whose every initializer is NULL/0/FALSE +
   empty body == trivial (lets the base ctor yield without sequencing).
3. **Base-dtor sequencing**: both dtors are non-trivial (derived frees pidls; base releases the site).
   Plan: scan-extend renames the base dtor dep to a plain worker (`{Base}_base_dtor`, is_dtor=False,
   re-owned) and `analyze_class` records it so the scaffold's Release emits `{cls}_dtor_body(this)`
   THEN the base-dtor worker (C++ destruction order). SetSite inside the base dtor binds to the
   base's own SetSite (C++ dynamic type during destruction = base) — the closure's unique SetSite.
4. **Member-class method workers** (the genuinely new piece): `m_VerInfo`'s POD fields can materialize
   via the existing `pod_state_types` machinery (needs the discoverer to accept a `class` with
   methods/private when its FIELDS are POD), but its 4 same-TU methods must become closure workers
   typed `this: *mut CFileVersionInfo` — a SECOND this-type in one translation, which the worker
   naming/signature machinery doesn't model yet. Member dtor (HeapFree if m_pInfo) must run in
   generated Release.
Everything else about the class is standard (1643 LOC, property-sheet dialog procs like CDrvDefExt).

## Final five LANDED — object map 28→33/33 Rust-owned (2026-07-18)

`CFileDefExt`, `CShellDispatch`, `CMenuSite`, `CMenuDeskBar`, and the inline
`CStartMenuDummy` are now Rust-owned. Every entry in `shell32.cpp`'s `ObjectMap` points at a
`*RustEntry` adapter; there are **zero activatable C++ entries left**.

The capability work in `998fc730bf5` closed the remaining deterministic preflight gaps:
out-of-header concrete bases (`CObjectWithSiteBase`), plain embedded class state
(`CFileVersionInfo`), exact `IDispatchImpl` flattening, the deliberately narrow
`CWindowImpl` HWND substrate, non-trivial base/member destructor sequencing, zero-only
initializers, and legacy `DECLARE_INTERFACE` parsing. It also made partial replay reasons
visible and fixed `--cost-cap 0` so it blocks before parallel calls. The rustconv test suite
and `vibeos-cli check-tooling` are green.

Integration details:

- `CFileDefExt` is a documented hybrid TU: Rust owns the activated COM object and its state;
  the retained C++ file owns source-local property-page/helper machinery exposed through
  narrow C ABI bridges. i386 caught the C++-mangling and calling-convention gaps that the
  isolated candidate oracle cannot see.
- `CMenuSite` and `CMenuDeskBar` live in `menuband-rust`, linked into both `shell32` and
  `rshell`, following the existing shared-shellmenu pattern.
- `CShellDispatch` is fully Rust-owned; the existing `CFolder` caller reaches its constructor
  through an explicit bridge. `CStartMenuDummy` is a no-state Rust registration shim that
  delegates to the existing start-menu implementation.

The required body-by-body comparison caught behavior bugs that all compiled cleanly:

- `CShellDispatch`: an inverted `BrowseForFolder` predicate, wrong `NameSpace` VARIANT/PIDL
  handling, missing AddRefs, incorrect service indirection/result VARIANT writes, and wrong
  typelib/IID/version constants.
- `CFileDefExt`: lost `CFileVersionInfo` ownership/lifecycle, HIDA ownership mistakes, wrong
  wide-pointer arithmetic and field widths, incorrect property-sheet userdata/apply handling,
  and incomplete destructor order.
- `CMenuSite`/`CMenuDeskBar`: destructive condition checks, double releases, incorrect COM
  forwarding/window host state, lost `OnSelect` switch fallthrough, and incomplete
  site/client/popup cleanup.

Final clean-tree gate:

```text
gate i386+amd64+arm64 — OK (10m37s)
build ×3, strict lint ×3, smoke ×3, net smoke, push stamp: all green
i386 33.793s; amd64 38.394s; arm64 43.035s
/tmp/vibeos-cli/20260718-201325-gate-i386+amd64+arm64/
```

All three screenshots were inspected. i386 and amd64 show the full desktop/taskbar; arm64
is the expected shell-ready wallpaper capture before taskbar paint.

Paid-output incident, retained for honesty: before the cost-cap fix, recovery accidentally
regenerated `CMenuSite::SetDeskBarSite` (about $0.01) and launched three rejected
`CMenuDeskBar` calls while invoked with `--cost-cap 0`. The rejected-call metadata does not
contain exact token/cost totals. All streamed artifacts were preserved; nothing was killed
or discarded. After the fix, the same replay stitched all 39 cached workers at 0 tokens/$0.

## Weekend handoff (2026-10-12) — Tier-3 batches C/D1, then the upstream shell32 ports

The lane has since been reused for the Tier-3 native-userland queue. Landed this weekend:
batch C `69b672ef9f97`, batch D1 `4873e2e21525`, modrun suites and fixes `b00e3231c176`.
Coverage and per-module owed work are in `rust-port-status.md` (batch C/D1 rows), not here.
Spend: about $4.63 of the $5 approved for the phase (`-m haiku`). Nothing is uncommitted.

**Next session, in order:**

1. **The upstream shell32 ports: DONE**, landed 2026-10-14 as `5857362c9eb3`. That commit also
   restored six cargo cells' build-order dependencies on generated files, which a fresh build
   raced on.
2. **ftp.** The scaffold is parked at `~/vibeos-rust-staging/port/ftp/parked/` (crate plus
   `gen_arm.sh`).
   - About 87 functions are still to translate, roughly $0.10–0.15 on Haiku. That fits the
     ~$0.37 left, but ask before spending it.
   - Hand-write the five `setjmp` functions. The one live `longjmp` (`cmdabort` on SIGINT
     during a proxy transfer) becomes a flag that `proxtrans` polls.
3. **fc.** It needs per-TU UNICODE islands (`texta.c`/`textw.c`). State is in
   `~/vibeos-rust-staging/port/fc/`, including the `chk-*.txt` per-arch checks.
4. **cscript** (built from wscript's sources with `-DCSCRIPT_BUILD`) is still C.
5. **Tool gaps (shared, $0):**
   - msxml2 vtables belong in the island.
   - COBJMACROS shims should be generated.
   - `ARRAY_SIZE!`/`_countof!` return u32 (win32macros-rust).
   - wscript's `debugstr_variant` is a placeholder.
6. **D2** needs a new budget: setup16, stortest, testgui, testchild, rapps_com, devmgmt.
   kbswitch, winmine, utilman and the screensavers went to control-plane's wave; the disk
   tools went to CRT.

**Stashes from this lane, kept for diagnosis** (find by message, never by index):
- `shell32-com: netsh shim after replay dropped 3 trampolines`.
- Two msvcrt-difftest/unclock entries, superseded by CRT `708ac41690b`. They can be dropped.
