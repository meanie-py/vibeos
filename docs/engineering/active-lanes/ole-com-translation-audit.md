# ole32 + oleaut32 → Rust — Audit & Translation Strategy

*Lane doc, opened 2026-07-11 on the `research` branch.*

## ✅ STATUS (2026-07-13) — **ole32 AND oleaut32 are both 100% Rust.**

`oleaut32`'s last six C TUs (`connpt`, `olefont`, `olepicture`, `olepropframe`, `recinfo`,
`varformat` — 9,172 LOC) are translated, so **both DLLs' C `SOURCE` lists are now empty**
except `guid.c` (the `initguid.h` data TU) and the generated RPC proxies. Translation
cost: **$3.13**.

### The point of this pass was the TOOL, not the translation

We fixed rustconv FIRST, using ole32's shipped defects as evidence, then used oleaut32's
six files as the proving run. Full causal chain: field manual **§9.72**. Summary:

| Defect | Consequence | Fix |
|---|---|---|
| `drop_pseh_macro_externs` catastrophic-backtracking regex (clean 2^n; 46 decls = never returns) | `translate` hung → operator killed it and hand-assembled the `.responses/` pieces → **bypassed ALL of `postprocess()`**, incl. ABI enforcement + the already-paid preamble. The root of nearly every "LLM bug" on the big files. | linear line-scan; 2.3 ms, and handles the mixed fn/`static` blocks the old regex could never match at all |
| `abi_enforce` conflated **linkage** with **calling convention** (`static` ⇒ drop the `extern`) | ~130 ole32 + 153 oleaut32 COM methods got the **Rust ABI** instead of `__stdcall`. Links clean on amd64/arm64; corrupts the i386 stack at the first vtbl call. | `is_static` now controls visibility ONLY; the convention always comes from the C |
| `_derive_convention` stopped at a `/* [out] */` comment between `)` and `{` | silently fell back to **cdecl** for 17 more genuine WINAPI methods (survives the fix above: still wrong on i386) | skip comments as well as whitespace |
| `CALLBACK`/`PASCAL` missing from the convention keywords | the fns C hands to the OS **as function pointers** (an EnumFontFamiliesEx callback, a DlgProc) got the Rust ABI — corrupting the stack on a call **we never make ourselves**, so there is no call site to inspect and nothing to grep for | added; `WINAPIV` pinned explicitly to cdecl |
| `make_user_dll_target` had `spec_file=None` for EVERY user-mode DLL | the pipeline never knew which fns were exports — it neither told the model nor enforced `#[no_mangle]`. That is how `_StgCreateDocfile@16` went missing from a shipped DLL. | locate `<dll>.spec`; `export_audit.py` gates it |
| `c2rust_anchor` matched the compile DB by worktree path | the canonical bind-mount build (`/tmp/vibeos`) made **every** lookup miss → `--c2rust` always hard-failed, repo-wide. Written off as "c2rust doesn't work here." | map canonical paths back onto the checkout |

### New deterministic oracles (wired into the pipeline; no LLM, $0)
- **`vtbl_gen.py`** — COM vtbl tables from the C initializer + bindgen's field order. The
  LLM had dropped **all 10** of storage32's (a COM object with no methods at all). Agrees
  **149/149 slots** with the hand-written tables. An arity mismatch raises rather than
  zipping a table into the wrong methods. *Do not let c2rust emit these either* — it
  hardcodes each slot as `f as unsafe extern "C" fn(..)`, the amd64 spelling, wrong on i386.
- **`table_audit.py`** — every file-scope C data table compared element-by-element.
  Caught `fmtLongTime` in `varformat.c` — the one file c2rust **cannot** anchor (upstream
  panic, `convert_type.rs:518`) and therefore the only one with no other oracle. Models
  C's implicit zero-fill (`BYTE t[256]` with 103 written elements is 256 bytes).
- **`export_audit.py`** — every `.spec` export must be a `#[no_mangle]` fn with the spec's
  convention. **ole32 and oleaut32 now both audit CLEAN** (255 / 398 exports needing a symbol).
- **`preamble_audit.py`** — file-scope C decls missing from the translated preamble.
- **`crate_integrate.py`** — the "integrate into a crate with a prelude" step that had only
  ever existed as a hand-written one-off script. Drops locally-redeclared Win32 imports
  (`extern "C" fn HeapAlloc` — that's stdcall on i386!) in favour of the prelude's correct
  decls; computes the provided-name set from the crate's real prelude instead of a
  hardcoded list.

### The meta-lesson
Every defect above produced code that **compiled, linked and booted**. Smoke cannot see a
COM DLL with a wrong calling convention on a path the boot never takes, or a date-format
table with one wrong byte. **When translating a library rather than a boot-path component,
the boot is not the oracle — the C is.** Build the differential checks and run them in the
pipeline, or ship silent wrongness.

---

*(The 2026-07-12 status and the original strategy/inventory follow.)*

## ✅ STATUS (2026-07-12, final) — **ole32 is 100% Rust. All 29 TUs flipped, boots ×3.**

Landed `98ac36e4d4d`. `ole32.dll`'s C `SOURCE` list in `dll/win32/ole32/CMakeLists.txt` is
now **empty** — every TU is Rust. Only `guid.c` and the generated RPC proxies remain C.
Full `vibeos-cli gate` ×3 green (build + lint + smoke on i386/amd64/arm64 + net), on a tree
that already carries the stable-toolchain lane's Axis B change.

**Tier-D, flipped this pass (6 TUs, 20K LOC):** `storage32` (10.3K — the compound-file
implementation: IStorage, the dir-entry tree, block chains, transacted snapshots),
`stg_stream` (IStream inside a compound file), `stg_prop` (IPropertyStorage /
IPropertySetStorage — the property-set on-disk format), `datacache` (the OLE presentation
cache), `defaulthandler` (the default object handler), `oleobj` (the advise holders).

**Earlier passes:** the COM core (compobj + marshal/stubmanager/rpc/ftmarshal/git) and the
23-TU mechanical ring (`58f4446bb03`).

**`guid.c` must stay C forever** — it is an `#include <initguid.h>` TU whose entire job is
emitting the moniker CLSIDs as *data*. (Same trap the oleaut32 flip hit: comment out an
initguid TU and you silently delete every GUID it was defining.) None of the six Tier-D
TUs included initguid.h, so the flip was safe — but **check this before every flip**.

### What Tier-D actually cost, and where the time went

Not the translation — that was already paid for. The cost was that **rustconv translates
FUNCTIONS, not headers, and not file-scope data**. Three whole categories of the C were
simply absent from the output and had to be authored:

1. **`storage32.h` itself.** The `StorageBaseImpl` class hierarchy, its virtual-method
   table, the 16 `static inline` dispatchers through it, `DirEntry`, and every on-disk
   `OFFSET_*`/`BLOCK_*`/`DIRENTRY_*`/`RANGELOCK_*` constant. `storage32.rs` alone referenced
   these ~130 times, as `extern` symbols that link to nothing. Now
   `sdk/lib/rust/ole32-rust/src/storage_defs.rs`.
2. **Every file-scope `static const ...Vtbl` table** — 23 of them across storage32/stg_prop/
   datacache. Without these the objects have no methods at all.
3. **A few functions outright** (`StorageImpl_GetFreeBlockChainCacheEntry`,
   `StgConvertVariantToProperty`, …) and, worse, **an export's attributes**:
   `StgCreateDocfile` came back as a module-private Rust-ABI fn, so `_StgCreateDocfile@16`
   was simply missing from the DLL.

**If you translate another header-heavy C module, budget for this.** Grep the C for
`^static const .*Vtbl`, `static inline`, and the module's private header BEFORE you start,
and treat them as a separate authoring task.

### The tooling this lane produced (reusable — this is the durable asset)
- **`tools/gen_commacros.py`** — generates the COBJMACRO vtbl-call helpers from com-sys
  bindgen. Must be run PER-ARCH: a method whose *parameter* is a function pointer
  (`IViewObject::Draw`'s `pfnContinue`) carries that callback's calling convention in its
  TYPE, and bindgen spells it per-arch — one shared amd64-generated file compiles on
  amd64/arm64 and fails i386 with `expected "stdcall" fn, found "C" fn`. It also emits
  unsuffixed aliases for the 13 vtbl slots the Win32 A/W macros rename
  (`GetObject`→`GetObjectA` etc.), which C call sites use but bindgen never emits.
  Validated against oleaut32-rust's hand-made file: 55/55 helpers byte-identical.
- **`tools/harvest_consts.py`** — resolves C constant names to their values from the SDK
  headers the C actually compiles against (both `#define`s and enum members, resolved
  TRANSITIVELY), and emits Rust `pub const`s each citing its `file:line`. **Constants are
  the silent-failure class**: a wrong one produces no compile error and no link error, it
  just changes behaviour. Tier-D shipped SEVEN baked-in wrong values before this
  (STG_E_INVALIDHEADER that was really STG_E_INVALIDHANDLE; DISCARDCACHE_SAVEIFDIRTY as 1
  when the header says 0, inverting save-on-discard; PROPSETFLAG_NONSIMPLE tested as `& 2`,
  which is ANSI …). Never hand-transcribe a constant, an HRESULT, or a GUID.
  Watch its own foot-gun too: a C value that is an *expression* (`(DWORD)~X`) must be
  translated, not passed through — a parse error in a constants module is not local, it
  deletes every constant in the module and manufactures ~800 phantom errors elsewhere.
- **`tools/integrate_ole32_ring.py`** — normalizes a raw rustconv translation onto a
  crate prelude: drops the COBJMACRO externs (macros in C — they link to nothing), drops
  the Win32 `extern "C"` decls (WINAPI is stdcall on i386 — this is THE i386 trap), drops
  self-declarations (rustconv emits an extern for a callee that lands in the same file →
  E0428), keeps the IID/CLSID statics (real link symbols — never redefine a GUID), and
  REPORTS anything it doesn't recognize so nothing slips through silently.
  Since Tier-D it also (a) computes what the prelude actually **provides** — every `pub`
  item DEFINED by a shared/ring module plus every name DECLARED by the prelude's extern
  surface — and drops those decls, because a local `extern "C"` decl *shadows* a glob
  import and would silently link against the wrong i386 decoration; and (b) **promotes
  DECLSPEC_HIDDEN cross-TU helpers to `pub`** (`StorageUtl_*`, `StorageBaseImpl_AddStream`),
  which rustconv leaves module-private because it translates one file in isolation.
  Unrecognized externs across Tier-D: 111 → 0.
  Two bugs found IN it, both of which had made the "already provided" check near-useless:
  a non-multiline `^`-anchored regex silently KEPT every comment-preceded decl (and an
  unclassified decl is kept — the failure direction that hurts), and the ABI class
  `extern "[a-z]+"` never matched the uppercase `"C"`, so it saw one definition in the
  whole crate.

### Cost
The whole ole32 translation (ring + Tier-D, 43.7K LOC, 29 files) cost **~$10** of Sonnet
via `rustconv translate` — well under the $25–60 the original estimate implied.

### Two rustconv issues worth fixing upstream
1. **The stitcher hangs** on large files (`moniker.c`, `storage32.c`): every per-function
   API call completes and lands in `.responses/`, then the process spins forever in
   "Stitching N functions...". The paid work is safe — assemble the run dir's `*.rs`
   pieces in sorted order and you have the file. But it burns a core until killed.
2. It emits `memoffset::core::mem::offset_of!` (no such crate), the OLD bindgen union
   accessors (`x.QuadPart()`, `x.u()` — current bindgen has plain fields), and names the
   C's `ref` field `ref_` in the struct while emitting `.ref` at use sites (a Rust keyword
   → hard parse error). It is not even self-consistent about that last one: Tier-D spelled
   the same field `.ref`, `.ref_` AND `.ref_0` in different functions of the same file.
   All mechanical; the integrator now fixes them.
3. **It does not emit file-scope data.** No `static const ...Vtbl` tables (23 missing across
   Tier-D — the objects had no methods), and no header content at all. See "What Tier-D
   actually cost" above. This is the single biggest gap and the one to fix first.
4. **It drops export attributes.** `StgCreateDocfile` — an ole32 export — came back as a
   private Rust-ABI `unsafe fn`, so `_StgCreateDocfile@16` was absent from the link. The
   `.spec` file is the ground truth for which functions need `#[no_mangle] extern "system"`.
5. **It emits COM methods with the Rust ABI.** ~130 vtbl slots across Tier-D were plain
   `unsafe fn`, not `STDMETHODCALLTYPE`. This links on amd64/arm64 and corrupts the stack on
   i386 at the first call. Wrap every vtbl method in the crate's `stdmethod!` macro.

---

## ✅ STATUS (2026-07-12) — oleaut32 is **fully Rust**; ole32 core is Rust

**Milestone: every C TU of `oleaut32.dll` that has a Rust twin is now flipped** —
the whole automation surface (BSTR, VARIANT, the coercion matrix, SAFEARRAY, the
typelib reader/writer, IDispatch, user-marshal) runs Rust and boots i386/amd64/arm64
to desktop. Landed `7dd3a9e0038` (flip commit `a6be654f7b8`), full cold
`vibeos-cli gate` ×3 green.

**Rust in `oleaut32.dll` (flipped C TUs — Rust wins first-match):**
- `hash.c` → `hash.rs` (LHashValOfNameSys A/W)
- `variant.c` → `variant.rs` (VariantInit/Clear/Copy/ChangeType + arithmetic; 36 exports)
- `vartype.c` → `vartype.rs` (the VarXxxFromYyy coercion matrix; 253 exports)
- `usrmarshal.c` → `usrmarshal.rs` (VARIANT/BSTR/LPSAFEARRAY user-marshal + proxy/stub; 93)
- `safearray.c` → `safearray.rs` (SAFEARRAY; the 4 public ctors added 07-12)
- `oleaut.c` → `oleaut.rs` (SysAllocString family + DllGetClassObject/proxy-stub + OleTranslateColor)
- `dispatch.c` → `dispatch.rs` (CreateStdDispatch/DispInvoke/DispGetIDsOfNames/DispGetParam)
- `typelib.c` → `typelib.rs` + `typelib/` (LoadTypeLib/RegisterTypeLib/CreateTypeLib*/
  DispCallFunc/ClearCustData + the ITypeInfo/ITypeLib/ITypeComp objects and the MSFT+SLTG
  binary `.tlb` readers and writer)

**Still C in oleaut32** (never translated; no twin — not on the boot/import surface):
`connpt.c`, `olefont.c`, `olepicture.c`, `olepropframe.c`, `recinfo.c`, `varformat.c`.

Link-order rule that shaped this: the C `.obj`s precede `liboleaut32_rust.a` and
`--allow-multiple-definition` takes first-match, so a Rust twin only wins when its `.c`
is commented out — and must then FULLY cover that TU's exports. Only comment out a TU
whose Rust twin is a complete export match.

**The `initguid.h` trap (worth remembering — this class recurs on any wine-style flip):**
`oleaut.c` carried `#include "initguid.h"`, which is what turned the `DEFINE_GUID`s in the
headers it subsequently included into *definitions*. Flipping it out silently removed the
only definition site for 7 proxy CLSIDs (`CLSID_PSFactoryBuffer`, `CLSID_PS{Dispatch,
EnumVariant,TypeInfo,TypeLib,OAInterface,TypeComp}`) → link-time undefined. The generated
widl proxy only *declares* them (`PROXY_CLSID=CLSID_PSFactoryBuffer` expands to
`extern CLSID CLSID_PSFactoryBuffer;`). Fix: `oleaut.rs` is now their `#[no_mangle]`
definition site. **Before flipping any `.c` that includes `initguid.h`, check what GUID
data it was silently defining.**

**Bugs the flip surfaced + fixed (all committed):**
- `VarCyCmp` returned VARCMP_GT/EQ transposed (GT=1/EQ=2; correct GT=2/EQ=1).
- `VARIANT_NumberFromBstr` `dwInFlags` was NUMPRS_STD=0x1 not 0x1FFF (all parse flags).
- i386: enabled SSE for this user-mode crate (config disables it triple-wide for the
  kernel); flipped 72 `_Stub`/`_Proxy` defs + the WINAPI/RPC extern decls to
  `extern "system"` for correct `_Name@N` stdcall decoration.

### Owed follow-ups (tracked; none block the current boot)
1. ~~Complete `safearray.rs`~~ — **DONE 07-12** (the 4 ctors were the only gap; the
   internal `SAFEARRAY_Create`/`_CreateVector`/`_GetVTSize` helpers were already translated).
2. ~~Complete `oleaut.rs`~~ — **DONE 07-12** (the "missing CLSID_PSFactoryBuffer ref" was
   really the `initguid.h` definition-site problem above).
3. ~~Complete `dispatch.rs` / `typelib.rs`~~ — **DONE 07-12**; both twins already had full
   export coverage (dispatch's 4; typelib's 13 spread across `typelib.rs` + `dispcall`/
   `writer`/`typecomp`). They were only still C because the flip had never been *attempted*.
   Lesson: re-derive coverage from the symbol tables before assuming a twin is partial.
4. **Unify the `olesys::ITypeLib`/`ACTCTX`/`GUID` mirror types with `com_sys::*`** — the
   fable lane self-mirrored a few COM types; they're layout-identical (bridged with safe
   casts, one benign redeclaration warning) but should collapse onto com-sys.
   *Assessed 07-12: the real overlap is ONE bridge cast (`olesys::ITypeLib` at oleaut.rs's
   `LoadTypeLib` call). Low value, non-trivial COM-vtable-layout risk on a booting system —
   deliberately deferred, not forgotten.*
5. `oleaut32-rust` carries ~600 raw-translation warnings (unused mut/parens/unsafe) —
   a `cargo fix` cosmetic pass.

### arm64 netstack-TDI flake — observed again during this flip's gating (NOT caused by it)
The 07-12 gating hit `HANG_TIMEOUT` on arm64 twice, both wedged at the identical point
(`[NETSTACK] TDI server: accept 0x00000000`). Isolation run (worktree at the merge-base,
i.e. same tree WITHOUT the OLE flip): **baseline 8/8 green, flipped tree 7/9** — and the
last 10 boots (5 on each tree, run concurrently) were all green. `netstack-rust` is a
**kernel driver**; `oleaut32.dll` is user-mode, so there is no call path — a user-mode flip
can only shift boot timing and *expose* the race. Read as the known §9.36 arm64
netstack-TDI/0x1E residual surfacing under host load, not a regression. Re-gate came back
×3 green. **If you see this in a gate, don't chase it into the OLE lane.**

**Do NOT** re-run the fable lanes or rustconv streams — all outputs exist in
`~/vibeos-rust-staging/{ole32-lane-a,ole32-lane-b,oleaut32-rustconv}/` + their MANIFESTs.
Re-translating re-spends money for work that's done.

---
*(Original strategy below — retained as the lane's rationale + inventory.)*

## Why this lane

Explorer.exe is ~99% Rust (`explorer-rust`), but its **dependency graph** is the next
frontier. A full import-table walk of the explorer process (exe + every DLL it maps)
found ~17 zero-Rust DLLs totaling ~370K LOC. `ole32` (55K) + `oleaut32` (42K) are the
chosen first target because:

1. **They're the COM substrate** — browseui/shell32/shdocvw/comdlg32 all stand on them,
   and explorer's own Rust COM objects call into them constantly.
2. **They're plain C, not C++** (Wine-derived). The vtables are already hand-rolled
   (`static const IStreamVtbl` + free functions) — the exact shape the Rust side uses.
   No ATL, no templates, no mangling. The "dickload of C++" problem lives in shell32's
   remaining ~64 ATL TUs and browseui, **not** here.
3. **Cohab precedent exists** for wine-style spec2def DLLs: advapi32 (misc/*.c flipped
   to `advapi32-rust`). Same recipe applies — per-TU CMake comment-out + `#[no_mangle]`
   + `--allow-multiple-definition`, exports still driven by `ole32.spec`.

## Finding 1 — the real export surface is small

Import-table audit of every built module in the explorer process
(explorer.exe, rshell, shell32, browseui, comctl32, uxtheme, shdocvw, shlwapi,
oleaut32, comdlg32; amd64 build):

| DLL | exports used / total | notes |
|---|---|---|
| ole32 | **56 / 295** | 23 of the 56 are `*_UserSize/Marshal/Unmarshal/Free` NDR helpers imported **only by oleaut32 itself** → true external surface ≈ **33 exports** |
| oleaut32 | **23 / 398** | BSTR (Sys\*String\*), VARIANT init/clear/copy/ChangeType, SafeArray, LoadTypeLib/LoadRegTypeLib/RegisterTypeLib, OleLoadPicture, DosDateTimeToVariantTime |

Consumer breakdown (imports per module): oleaut32→ole32 36, shell32→ole32 26,
shell32→oleaut32 16, browseui→ole32 14, shlwapi→oleaut32 12, shdocvw→oleaut32 9,
explorer.exe→ole32 5 (CoInitialize/CoCreateInstance-class only).

The **inter-thread marshaling path is live**: shell32 imports
`CoMarshalInterThreadInterfaceInStream` / `CoGetInterfaceAndReleaseStream` /
`CoWaitForMultipleHandles` (CDefView-style background threads). So the marshal/stub/rpc
core can't be stubbed — but note the import table is a *reachability* statement, not a
boot-path one; a debugcon counter can tell us later whether it fires before the desktop
renders.

**Interface objects matter beyond exports:** the export surface understates the work
only mildly — most of both DLLs' code *implements COM objects reached through vtables*
(IStream from `CreateStreamOnHGlobal`, IDataObject from the clipboard, monikers from
`CreateItemMoniker`, …). The per-file tiering below accounts for that.

## Finding 2 — blast radius is confined to the shell graph

None of winlogon.exe, services.exe, lsass.exe, userinit.exe, msgina.dll, smss.exe
import ole32 or oleaut32. A regression here breaks explorer/shell, not the boot spine
before it. (Smoke's boot marker is explorer's `shell ready`, so the gate still catches
everything — but hangs/BSODs earlier in boot are off the table.)

## Finding 3 — file inventory, tiered

### ole32 (35 TUs, 55.2K LOC)

**Tier H — hardcore runtime machinery (subagent lanes; ~11–14K LOC):**

| file | LOC | contents |
|---|---:|---|
| `compobj.c` | 5,416 | CoInitialize/Ex, apartments (STA/MTA, TLS state), CoCreateInstance + class-object activation (registry→LoadLibrary→DllGetClassObject), CLSID/IID string fns, CoFreeUnusedLibraries |
| `marshal.c` | 2,251 | proxy manager, CoMarshal/UnmarshalInterface, inter-thread stream marshaling |
| `rpc.c` | 2,057 | the ORPC channel (IRpcChannelBuffer), message windows / apartment pumps |
| `stubmanager.c` | 841 | stub lifetime/refcount bookkeeping on the server side |
| *(adjacent)* `ftmarshal.c` 337, `git.c` 570 | free-threaded marshaler; global interface table — same domain, small |

**Tier M — mechanical C, ideal rustconv fodder (~24K LOC, ordered by import-surface priority):**

| file(s) | LOC | why it's on the surface |
|---|---:|---|
| `ifs.c` | 563 | CoTaskMemAlloc/Free/Realloc, CoGetMalloc — used by literally everything |
| `hglobalstream.c` | 675 | CreateStreamOnHGlobal/GetHGlobalFromStream (band persistence) |
| `ole2.c` | 3,292 | OleInitialize, DoDragDrop, Register/RevokeDragDrop, ReleaseStgMedium |
| `clipboard.c` | 2,339 | OleSet/Get/FlushClipboard (IDataObject clipboard) |
| `bindctx.c`, `moniker.c` + 5 `*moniker.c` | ~7K | CreateBindCtx, CreateItem/ClassMoniker, GetRunningObjectTable |
| `errorinfo.c` | ~500 | GetErrorInfo |
| `usrmarshal.c` | 3,166 | the `*_User*` NDR helpers (consumed by oleaut32 only) — dumb serialization code |
| `enumx.c`, `dictionary.c`, `memlockbytes.c`, `filelockbytes.c`, `comcat.c`, `ole32_main.c`, `ole2impl.c`, `ole2stubs.c`, `oleproxy.c` | ~4K | small helpers/stubs |

**Tier D — defer (not on the import surface; ~17K LOC):**
`storage32.c` (10.7K!) + `stg_stream.c`/`stg_prop.c` — compound-document structured
storage; `datacache.c`/`defaulthandler.c`/`oleobj.c` — OLE embedding. Nothing in the
explorer graph imports StgCreateDocfile & co. Translate last (or leave as active C
until a consumer shows up — they're self-contained).

**Not translated (irreducible glue):** the widl-generated proxy/stub C from
`add_rpcproxy_files(dcom.idl, ole32_*.idl)` + `generate_idl_iids` — same category as
ntdll's generated stubs.

### oleaut32 (14+ TUs, 41.6K LOC)

**Tier M — mechanical data-structure code, prime rustconv fodder (~20K LOC):**

| file | LOC | contents |
|---|---:|---|
| `oleaut.c` | 1,255 | **BSTR** (SysAllocString family) — tiny, foundational, first flip |
| `variant.c` | 5,932 | VARIANT init/clear/copy/ChangeType dispatch |
| `vartype.c` | 8,048 | the N×N coercion matrix (VarR8FromDec, …) — tedious, pure, unit-testable |
| `safearray.c` | 1,756 | SAFEARRAY |
| `hash.c` | 641 | LHashValOfNameSys |
| `usrmarshal.c` | 2,693 | VARIANT/BSTR/SAFEARRAY NDR (needs the ole32 `*_User*` helpers) |

**Tier H-lite:** `typelib.c` (11,651 — `.tlb` binary-format reader + ITypeInfo) —
`LoadRegTypeLib`/`LoadTypeLib` ARE imported (shell32/shdocvw/browseui, e.g.
CShellDispatch). It's big but it's a *file-format parser*, not concurrency machinery —
rustconv `translate` per-function with review, or fold into a subagent lane if the
budget allows. `dispatch.c` (462) rides with it.

**Tier D — defer (not on the surface):** `varformat.c` (2,618), `olefont.c` (2,285),
`olepropframe.c` (344), `connpt.c` (594), `recinfo.c` (713). `olepicture.c` (2,618) is
mid-priority: `OleLoadPicture` is imported.

## Finding 4 — bindgen over the COM headers WORKS (spike validated)

One wrapper header over the real SDK tree produced clean `#[repr(C)]` vtbl structs for
25 interfaces (IUnknown, IStream, IStorage, IDataObject, IDropTarget/Source, IMoniker,
IBindCtx, IPersist\*, IEnum\*, IClassFactory, IServiceProvider, …) — 1,331 lines, zero
hand-typing. Working recipe (amd64; crib of `ntkrnl-sys/generate_bindings.sh`):

```sh
# wrapper: #define WIN32_NO_STATUS / COBJMACROS; #include windef.h, winbase.h,
#          objbase.h, oleidl.h, servprov.h   (objbase.h REQUIRED before unknwn.h —
#          it defines `interface`/BEGIN_INTERFACE; that was the only real trap)
bindgen com_wrapper.h --use-core --no-layout-tests \
  --allowlist-type 'I(Unknown|Stream|DataObject|...).*' \
  -- --target=x86_64-pc-windows-gnu \
  -I sdk/include{,/psdk,/xdk,/ddk,/ndk,/vcruntime,/reactos,/reactos/libs} \
  -I sdk/lib/pseh/include -I build-amd64/sdk/include{,/psdk,/reactos} \
  -isystem sdk/include/crt -isystem .build-tools/llvm-mingw/lib/clang/19/include \
  -D__REACTOS__ -D_AMD64_ -D_M_AMD64 -D_WIN64 -D__x86_64__ \
  -D_WIN32_WINNT=0x600 -DWINVER=0x502 -D_USE_DUMMY_PSEH=1
```

Caveats for productionizing: (a) run per-arch — i386 needs the stdcall-decorated
target + the usual `extern "system"` check (`verify_ffi.py` gates it); (b) amd64 emits
`extern "C"` which is correct there; postprocess like `postprocess_bindgen.py` if we
want uniform `extern "system"`; (c) add the `size_of` assertion harness like ntkrnl-sys.

## Finding 5 — cost (rustconv estimate, floor)

| target | sync Sonnet floor | sync @5x | batch Sonnet |
|---|---:|---:|---:|
| ole32 (all 55K) | $8.32 | $41.62 | $4.16 |
| oleaut32 (all 42K) | $5.09 | $25.46 | $2.55 |

Tiering cuts this: deferring Tier D drops ~35% of the LOC. Budget expectation for the
full lane ≈ **$30–60 realistic** (translate-per-function on big files runs 2–5× floor).

## Strategy

### Phase 0 — tooling (before any translation)
1. **`com-sys` crate** (name TBD): bindgen-generated interface vtbls + IIDs + FORMATETC/
   STGMEDIUM/STATSTG types, per-arch, with size assertions. Both the ole32/oleaut32
   translations and every future shell-DLL translation consume it. (~a day, recipe
   above already works.)
2. **Promote explorer-sys's COM scaffolding** (`from_iface!` multi-vtable pattern, QI
   dispatch, refcount helpers, `server.rs` CComObject-equivalents) into a shared
   workspace crate (`com-rt`) so translated COM *servers* stop re-rolling boilerplate.
   explorer-rust keeps working unmodified (explorer-sys can re-export).
3. *(Deferred to the shell32 lane, not this one:)* ATL `BEGIN_COM_MAP` scaffold
   generator — parse COM_MAP blocks, emit struct+vtbls+QI skeleton, LLM fills method
   bodies. Noted here because it's the tooling win for the C++ pile; ole32/oleaut32
   don't need it.

### Phase 1 — oleaut32 mechanical ring (rustconv)
`oleaut.c` (BSTR) → `safearray.c` → `hash.c` → `variant.c` → `vartype.c` →
`usrmarshal.c`. Per-file cohab flips into a new `oleaut32-rust` crate, advapi32-style
CMake wiring. vartype/variant coercions are pure logic — add host-side unit tests for
the coercion matrix (Wine's conformance tests are a ready oracle for expected values).
Gate ×3 per flip batch.

### Phase 2 — ole32 mechanical ring (rustconv)
`ifs.c` → `hglobalstream.c` → `errorinfo.c` → `bindctx.c` → monikers → `enumx.c`/
small helpers → `ole2.c` → `clipboard.c` → `usrmarshal.c`. New `ole32-rust` crate,
same recipe. This ring has no apartment/threading subtleties beyond what the C already
does (clipboard has a hidden window — flag for review).

### Phase 3 — hardcore core (up to 2 fable subagents, sequenced after Phase 2)
- **Lane A: `compobj.c`** (5.4K) — apartments, TLS, activation. The single most
  load-bearing file; everything already flipped keeps exercising it during its own
  translation, which is the point of doing it last.
- **Lane B: `marshal.c` + `stubmanager.c` + `rpc.c` (+ `ftmarshal.c`, `git.c`)** (~6K)
  — the ORPC/marshaling cluster. Lanes A and B touch disjoint files and communicate
  through existing C headers, so they can run in parallel per the standard
  staging-dir protocol (agents write `.rs` to `~/vibeos-rust-staging/`, integration/
  build/gate is central).
- `typelib.c`+`dispatch.c` ride as a third work item for whichever lane finishes
  first (or stay on the rustconv track — decide by then).

### Verification
- Per-flip: `vibeos-cli gate` (smoke ×3) — explorer's boot IS the integration test for
  most of this surface (CoInitialize, CoCreateInstance, streams, task-band persistence).
- Host-side: unit tests for vartype coercions / BSTR / SAFEARRAY semantics in the crate.
- In-VM: `modules/rostests/winetests/{ole32,oleaut32}` exist as a deeper oracle if a
  regression needs bisecting beyond "desktop renders".
- Runtime counters: a debugcon print in the marshal path early on, to learn whether the
  inter-thread path fires pre-desktop (informs how carefully Lane B must be staged).

### Explicit non-goals (this lane)
- shell32's ~64 ATL TUs, browseui, comctl32 — separate lanes; comctl32 doesn't even
  import ole32.
- Structured storage + OLE embedding (Tier D) unless a consumer materializes.
- The widl-generated proxy/stub C (irreducible glue, stays C).
- DCOM/cross-*process* marshaling correctness beyond what the C does today.

## Decisions (user review 2026-07-11)
1. **Crate layout:** `com-sys` (bindgen interfaces) + separate `ole32-rust` /
   `oleaut32-rust` body crates.
2. **Phase 3 execution:** staging-dir mode — subagents write to
   `~/vibeos-rust-staging/`, central integration/build/gate.
3. **Tier D:** original ruling was "if unused, delete as trash" — but a full-livecd
   import scan (all built PEs, not just the explorer process) found real `Stg*`
   consumers: **wordpad.exe, msi.dll, ieframe.dll, msisip.dll**. Revised ruling
   (2026-07-11): **translate Tier D too, in a later pass** — storage32 + stg_stream +
   stg_prop + the OLE-embedding files join the rustconv queue after Phases 1–2 land.
   Full-tree ole32 surface is 108/295 exports.
4. **`typelib.c` + `dispatch.c`:** folded into subagent Lane A now (not rustconv).
5. **Model policy:** DeepSeek for simple/mechanical files, Sonnet for the rest
   (rustconv Phases 1–2); Fable subagents for the Tier-H lanes.

## Phase 3 lanes (KICKED OFF 2026-07-11)
- **Lane A** → `~/vibeos-rust-staging/ole32-lane-a/`: `compobj.c` (priority 1),
  `oleaut32/dispatch.c`, `oleaut32/typelib.c` (in that order).
- **Lane B** → `~/vibeos-rust-staging/ole32-lane-b/`: `marshal.c`, `stubmanager.c`,
  `rpc.c`, `ftmarshal.c`, `git.c`.
- Shared-type hazard: both lanes consume `dll/win32/ole32/compobj.h` private types
  (`struct apartment`, `struct oletls`, …) — each lane declares `#[repr(C)]` mirrors
  locally + notes them in its MANIFEST; integration dedupes into one internal module.

---
*Research artifacts (scratchpad, session-local): `ole-imports-raw.txt`,
`ole32-surface.txt`, `oleaut32-surface.txt`, `com_wrapper.h`, `com_bindings.rs` (the
1,331-line bindgen spike output). Regenerate with the recipes above.*
