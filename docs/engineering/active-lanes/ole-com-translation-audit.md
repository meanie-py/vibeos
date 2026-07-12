# ole32 + oleaut32 → Rust — Audit & Translation Strategy

*Lane doc, opened 2026-07-11 on the `research` branch. Research artifacts referenced
below were produced against the 2026-07-11 tree + the main checkout's `build-amd64`
binaries.*

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
