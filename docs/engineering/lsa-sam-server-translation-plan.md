# lsasrv + samsrv → Rust — Translation Plan

**Status:** IN PROGRESS (2026-06-21, kernel32-port). Scope: translate the ReactOS **`lsasrv`**
and **`samsrv`** server DLLs C→Rust, multi-arch (i386/amd64/arm64), completing the boot/logon
spine in Rust. Sibling of the userland audit set — see `userland-translation-audit-part2.md`
(these are 2 of its 3 High-value native targets; **msgina**, the 3rd, landed `e69a7adaf82`).

> ### Execution progress (kernel32-port)
> - **M0a ✅ (`effde8d4469`)** — Rust `lsa-stub` replaces widl's `lsa_s.c`; i386+amd64 boot.
>   Fixed `tools/widl_to_rust.py` named-type ExprEval (lsa.idl's `LSAPR_ACL` conformant array).
> - **M0b ✅ (`238dca6d6ee`)** — `lsasrv-sys` bindgen substrate + `dssetup.c` body; i386+amd64 boot.
> - **M1 ✅ (`f9f54798af0`)** — substrate (`prelude.rs`/`consts.rs` + crate TRACE macros) + leaf
>   cluster `privileges.c` + `utils.c` flipped; lsasrv-rust compiles ×3, DLL links ×3, i386 boots.
> - **M2 (in progress)** — `security.c` ✅ (`00a7b6afc8f`, SD/ACL builders), `registry.c` ✅
>   (`af3e0ba7e99`, Nt*Key wrappers). **6 of 16 units Rust** (lsa_s stub + dssetup + privileges +
>   utils + security + registry); each ×3-link-verified + i386 boot-gated.
> - **Next:** `lookup.c` (2534, SID predicates — big), then M3 (database/policy/session — incl. the
>   lone DUMMY_PSEH fix + NtAuthority/WorldSidAuthority globals consumed by security.rs),
>   M4 (LPC/port/authpackage), M5 (lsarpc.c 75-method fan-out + ×3 gate). Then samsrv S0–S4.
>
> **⚠️ Link is the oracle for cross-file exports.** rustconv occasionally emits a cross-file C
> function as a private Rust fn (no `#[no_mangle]`) → undefined-symbol at DLL link (e.g.
> `LsapRegQueryValue`). A stale DLL can falsely boot green — always confirm `ninja lsasrv` rc=0 +
> `undefined symbol` count 0 ×3 before trusting a smoke. Restore `#[no_mangle] pub extern "C"`.
>
> ### The per-file integration recipe (validated; follow for every remaining file)
> 1. **Translate:** `python -m rustconv translate dll/win32/lsasrv/<f>.c -m deepseek --c2rust
>    --cost-cap N` (PYTHONPATH=worktree/tools/rustconv; `source .env`). Output assembles to
>    `sdk/lib/rust/lsasrv-rust/src/lsasrv/<f>.rs` (~$0.01–0.05/file on DeepSeek).
> 2. **Imports:** change the file's `use ntdll_sys::nt::*;` → `use crate::prelude::*;`. Add
>    `use crate::lsasrv::<other>::{fn,…};` for cross-file lsasrv-internal calls (the callees are
>    `pub` in their module — same-crate Rust calls, not externs).
> 3. **Wide-string statics (c2rust dangling-pointer):** rewrite `transmute::<[u8;N],[u16;K]>
>    (*b"..").as_ptr()` → `transmute::<&[u8;N],&[u16;K]>(b"..").as_ptr()` (script in commit
>    `f9f54798af0`'s history; **only** `.as_ptr()`-suffixed transmutes — NOT by-value array fields
>    like the `__wine_debug_channel.name`). Should be folded into rustconv postprocess (TODO).
> 4. **Boundary casts:** dual-crate `_LUID`/`SID` etc. (lsasrv_sys vs ntdll_sys, same layout) →
>    `transmute`; `SIZE_T`↔`usize` (ntdll ULONG_PTR=u32 on i386 ≠ usize) → normalize to `as usize`
>    for `MIDL_user_allocate`; resource-id `IDS_*` (INT) used as UINT arg → `as u32`.
> 5. **New substrate gaps:** missing kernel32/CRT externs → add to `prelude.rs`; missing consts →
>    `consts.rs`; missing LSAPR type → add to the prelude's curated `lsasrv_sys::{…}` import (and
>    if absent from lsasrv-sys, widen `lsasrv-sys/generate_bindings.sh` allowlist + regenerate ×3).
> 6. **Build green ×3** (`cargo build -p lsasrv-rust --target {i686,x86_64,aarch64}-…`), then
>    **cohab-flip** (comment the `.c` out of `dll/win32/lsasrv/CMakeLists.txt` SOURCE), `cmake .`,
>    `ninja lsasrv` ×3, smoke. Run a batch of files, flip together, gate once.
> - **Crate layout:** bodies live in `lsasrv-rust/src/lsasrv/<f>.rs` (assembler namespaces by C
>   parent dir); list each in `src/lsasrv/mod.rs`. lsasrv-rust deps `ntdll-sys` + `lsasrv-sys`.
> - **The lone DUMMY_PSEH fix** (`database.c:833`, M3) — validate the handle in safe Rust before
>   deref; the `_SEH2_EXCEPT` body that mutates `bValid` must be dead code.

> **Why this is the prize:** `lsass.exe` is already Rust (`lsass-rust`/`lsass-cargo`), but it
> links **lsasrv** + **samsrv** as C importlibs — those two DLLs hold the *actual* LSA/SAM
> logic. Translating them finishes the whole logon path in Rust:
> `smss→wininit→lsass→lsasrv/samsrv init` and `winlogon→msgina→LsaLogonUser`. Both are
> boot-critical.

---

## 0. Headline findings from the research spike (these shaped the plan)

1. **The MIDL/RPC-server-in-Rust cohab is already a solved, shipping pattern in this repo —
   NOT a novel unknown.** It exists for **svcctl** (services.exe) and **winreg** (winlogon):
   - `tools/widl_to_rust.py` (669 LOC) transliterates a widl-generated `<iface>_s.c` →
     `<iface>_s.rs` against the `sdk/lib/rust/midl-rt/` substrate crate (ABI-asserted
     `repr(C)` mirrors of `MIDL_STUB_DESC`/`MIDL_SERVER_INFO`/`RPC_SERVER_INTERFACE`/
     `RPC_DISPATCH_TABLE`, per-arch size/offset asserts).
   - `reactos-link::generate_rpc_stub_rust(idl, "<iface>_s", "<c_target>")` drives it from a
     stub crate's `build.rs` (mirror `sdk/lib/rust/svcctl-stub/`). It reads the **compiled
     `_s.c.obj`** via `llvm-nm` to get exact per-arch symbol decorations (solves the i386
     `@N` problem automatically).
   - The server methods are plain Rust: `sdk/lib/rust/services-rust/src/rpcserver.rs` has
     `#[no_mangle] pub unsafe extern "system" fn RCloseServiceHandle(...)`. The generated
     stub's `SERVER_ROUTINE` table references them by decorated name → the linker binds them.
   - **Build note:** this tree uses **widl**, not MS-MIDL (`sdk/cmake/widl-support.cmake:82`
     is the active `add_rpc_files`; `midl-support.cmake` is dead). `add_rpc_files(server x.idl)`
     emits only `x_s.c` + `x_s.h` into `build-*/dll/win32/<mod>/`.
   - **Consequence:** we **replace** `lsa_s.c`/`sam_s.c` with a generated Rust stub (the svcctl
     recipe), we do **not** keep them as compiled C. The earlier "keep the `_s.c` as C" idea is
     superseded — the Rust-stub path is cleaner and battle-tested.

2. **samsrv computes NO crypto itself — it's 100% delegated to advapi32.** The only crypto
   touchpoints are 3 FFI calls: `SystemFunction006` (LM OWF), `SystemFunction007` (NT/MD4 OWF),
   `SystemFunction013` (DES cross-decrypt) — declared `samsrv.h:447-461`, called in
   `samsrv.c:36-60` + `samrpc.c:8004-8035`. So there is **no hash/RC4/DES code to translate or
   fuzz inside samsrv** — just wire up 3 `extern "system"` shims. (The real LM/NT primitives
   live in `dll/win32/advapi32/wine/crypt_lmhash.c` — a self-contained leaf that is a *perfect*
   difftest target *if/when an advapi32-crypto lane opens*, but that's a different module.)

3. **The differential-fuzz payoff is the blob/SID/RID byte-math, not crypto** (see §5). These
   are pure `&[u8]→&[u8]` transforms with explicit offset/length arithmetic — exactly the
   captures(none)/pointer-arith bug class, and ideal `rtl-difftest`-style targets.

4. **Exactly one DUMMY_PSEH site per module** — `lsasrv/database.c:833-846` and
   `samsrv/database.c:309-322`, both validating a caller handle's signature and **mutating an
   output in the `_SEH2_EXCEPT` body** → must become dead code under `USE_DUMMY_PSEH` (validate
   the pointer in safe Rust *before* deref). Same class as the kernel32 IsBad*Ptr / cdfs bugs.

5. **The big RPC files are flat and per-method-independent** (`lsarpc.c` 75 `Lsar*`,
   `samrpc.c` 68 `Samr*`) → excellent for **parallel per-method translation** once the shared
   surface (mapping tables, helper signatures, marshalled types) is centralized.

---

## 1. Architecture: the cohab shape

Per module, three new crates + a cohab flip (mirror the svcctl/services trio):

| Crate | Kind | Role |
|---|---|---|
| **`lsasrv-sys` / `samsrv-sys`** | rlib (bindgen) | `LSAPR_*` / `SAMPR_*` marshalled types + RPC handle/string types. Bindgen the generated `build-*/dll/win32/<mod>/<iface>_s.h` per-arch (allowlist `LSAPR_*`/`SAMPR_*`), mirroring `sdk/lib/rust/lsass-sys/` layout (`bindgen_{i386,amd64,arm64}.rs` + `generate_bindings.sh`). **Not** covered by `lsass-sys`. |
| **`lsa-stub` / `sam-stub`** | staticlib | Generated Rust RPC server stub. `build.rs` = `reactos_link::generate_rpc_stub_rust("sdk/include/reactos/idl/lsa.idl","lsa_s","<lsasrv-c-target>")`. Dep `midl-rt`. Mirror `sdk/lib/rust/svcctl-stub/`. Replaces `lsa_s.c`/`sam_s.c`. |
| **`lsasrv-rust` / `samsrv-rust`** | staticlib | The translated bodies: `#[no_mangle] pub unsafe extern "system" fn LsarXxx/SamrXxx(...)` + the `*_HANDLE_rundown` context-rundowns + `MIDL_user_allocate`/`MIDL_user_free`. Mirror `services-rust/src/rpcserver.rs`. Add `cabi` feature for difftest. |

**Reuse directly (already on mainline):**
- **`lsass-sys`** — LPC message types (`LSA_API_MSG`, `LSA_API_NUMBER`, `LSA_TRUSTED_CALLER`,
  `LSA_CONNECTION_INFO`, per-request msg structs). lsasrv's **auth-port server** marshals the
  same structs. Add as a dep; do not re-bindgen.
- **`lsalib-rust/src/lsa.rs`** — the *client* marshalling recipe for every `LSA_API_NUMBER`;
  mirror it *in reverse* for the server dispatch loop (lsalib is client-only; the LPC **server**
  half — `NtCreatePort`/`NtListenPort`/`NtAcceptConnectPort`/`NtReplyWaitReceivePort` + dispatch
  table — is new code in `authport.c`/`authpackage.c`).
- **`lsass-cargo` / `reactos-link`** cohab recipe — copy for the eventual `lsasrv.dll`/
  `samsrv.dll` cargo link (`ModuleKind::Win32Dll`, spec-driven exports from
  `lsasrv.spec`/`samsrv.spec`, self-hosted import libs). `lsass-rust`'s `extern` decls of
  `LsapInitLsa`/`ServiceInit`/`SamIInitialize` are exactly the symbols our Rust must export.

**Cohab flip:** comment the translated `.c` out of `dll/win32/{lsasrv,samsrv}/CMakeLists.txt`
(kept on disk per never-delete), drop `${CMAKE_CURRENT_BINARY_DIR}/<iface>_s.c`, link the three
Rust staticlibs. lsass is already cargo-driven, so this slots into the existing machinery.

---

## 2. lsasrv — per-file map (~15K LOC, 16 files)

Source `dll/win32/lsasrv/`. Suggested order: leaves/pure first → DB → policy/session/lookup →
LPC/port → lsarpc.c last (it consumes everything).

| File | LOC | Role | Top hazards | Difficulty |
|---|---|---|---|---|
| **lsarpc.c** | 4391 | 75 `Lsar*` RPC server methods; handle dispatch over policy/account/secret/trusted-domain | MIDL alloc ownership; self-relative SD packing (`:271-290`); RID/`domain\account` ptr-math (`:827,1243`); 75 stdcall entry points; secret-crypto is FIXME-stubbed in C | **Very High** |
| **lookup.c** | 2534 | SID↔name engine; well-known SID registry | SID byte layout/ptr-math (`:119,1010-1034`); manual SID build (`:1050-1149`); `wcschr`-delta name split; global well-known list | **Very High** |
| **authpackage.c** | 1856 | Auth-package (NTLM/Kerberos) loader + dispatch; cross-proc token copy | cross-proc `Nt{Allocate,Free,Write,Read}VirtualMemory` (`:402-469`); SID-in-`TOKEN_GROUPS` layout; plugin fn-ptr ABI; global dispatch table | **High** |
| **database.c** | 1089 | Registry-backed LSA object store (Policy/Account/Domain/Secret) | ⚠️ **the lone `_SEH2` site** (`:833-846`, EXCEPT mutates `bValid` → dead code); unvalidated handle cast (`:830`); REG_NONE blob (de)serialize; global SID/domain state | **High** |
| **policy.c** | 1033 | Policy query/set (audit, domain info, server role) | query-NULL-then-alloc TOCTOU (`:80-86`); goto-cleanup double-free risk; MIDL allocs | **Med-High** |
| **security.c** | 778 | Builds self-relative SDs for LSA objects — **the SD/ACL packing core** | **CRITICAL** ACE-size arith `sizeof(ACE)-sizeof(ULONG)+RtlLengthSid` ×4 (`:127-133,…`); two-call size-then-write | **High** (isolated/pure → best fuzz target) |
| **session.c** | 581 | Logon-session list by LUID + self-relative serialize | global list **no lock** → Mutex; ULONG_PTR-subtraction offset encoding (differs i386/amd64) | **Med-High** |
| **authport.c** | 420 | LPC auth-port server (logon-process reg + dispatch loop) | LPC server syscalls; `PORT_MESSAGE` bitfield; token inspection | **Med-High** |
| **registry.c** | 418 | Nt/Zw registry key/value wrappers | blob sizing FIELD_OFFSET ptr-math; null-term injection | **Medium** |
| **privileges.c** | 397 | Privilege/right LUID↔name bijective tables | const-table LUID packing; resource-ID offset | **Low-Med** (cleanest oracle) |
| **lsasrv.c** | 351 | LSA init + MIDL allocator stubs + 6 `LsaIFree_*` deep-frees | nested frees over 15 policy info classes | **Medium** |
| **srm.c** | 322 | SRM port client to kernel SRM | LPC header bitfield packing; void-ptr casts | **Medium** |
| **notify.c** | 217 | Policy-change notification list | `RTL_RESOURCE` rw-lock; CONTAINING_RECORD; cross-proc handle dup | **Medium** |
| **service.c** | 187 | Service dispatcher (NETLOGON, SAMSS) | LoadLibrary/GetProcAddress glue; named-event sync | **Low-Med** |
| **utils.c** | 186 | PE resource-string loader + SID append | packed-string `p += *p+1`; `RtlAllocateAndInitializeSid` ≤8 RIDs | **Low-Med** |
| **dssetup.c** | 169 | DsRole setup RPC iface (1 method) | info-class switch; inline struct packing | **Low** |

**Interfaces:** `lsa.idl` (41.8 KB, **72 `Lsar*` methods**) + `dssetup.idl` (1 method,
`DsRolerGetPrimaryDomainInformation`), in `sdk/include/reactos/idl/`.

**Spec (`lsasrv.spec`, 147 entries):** 59 real (`Lsar*` server methods + `LsaI*`/`Lsap*`
internal helpers — note: the *public* `LsaOpenPolicy`/etc. live in **advapi32**, not here),
**88 `@ stub`** genuinely-unimplemented (`LsaI*` forest-trust/audit, `DsRoler*` DC ops) →
translate-as-stub. i386: `#[no_mangle] extern "system"` → correct `_Name@N` (never `#[export_name]`).

**Global state →** `OnceLock` (init-once SIDs/handles/domain-names: `database.c:15,23-26`;
`lookup.c` well-known SIDs; `lsasrv.c:13` product type) · `Mutex` (intrusive lists: well-known
SIDs, sessions `session.c:30` *no C lock*, logon contexts, notifications, auth packages) ·
real locks (`PolicyHandleTableLock` `lsarpc.c:14`, `NotificationListLock` rw `notify.c:22`).

---

## 3. samsrv — per-file map (~16.5K LOC, 12 files)

Source `dll/win32/samsrv/`. `samrpc.c` is 60% of the module and **flat/per-method-independent**.

| File | LOC | Role | Top hazards | Difficulty |
|---|---|---|---|---|
| **samrpc.c** | 9824 | 68 `Samr*` methods (~13 are FIXME thin stubs) + 5 `GENERIC_MAPPING` tables + `Samp*` info-class packers | per-method `RtlAcquireResource`/`Release` pairing + `goto done`; SD stored as `L"SecDesc"` attr; access-check is a plain mask test (`database.c:330`), not `SeAccessCheck` | **Very High** (volume), shallow per method |
| **security.c** | 1703 | 6 default-SD builders (server/builtin/account-dom/alias/group/user) | two-phase `RtlAbsoluteToSelfRelativeSD`; ⚠️ **latent C bug: Sacl alloc'd with `DaclSize` not `SaclSize`** (`:160,422,736,1023,1310,1583`) — translate faithfully + flag, do **not** silently fix | **High** (pure → fuzz target) |
| **setup.c** | 1203 | Initial SAM population at install (Builtin/Account domains, aliases, Admin/Guest) | hardcoded RID/SID/policy literals; `Empty{Nt,Lm}Hash` extern blobs | **Medium** (mechanical) |
| **samsrv.c** | 755 | DLL entry, `SamIConnect/Initialize`, `SampInitHashes`, 18 `SamIFree_SAMPR_*` deallocators | `SampInitHashes` calls `SystemFunction006/007`; 18 recursive frees mirror marshalled shapes | **Medium** |
| **user.c** | 730 | Group-membership arrays, password-history rotation, logon-hours bit-packing | **top fuzz file** — array pack/unpack + bitfield (§5) | **Med-High** |
| **database.c** | 617 | DB object lifecycle + per-object access check + attr get/set | ⚠️ **the lone `_SEH2` site** (`:309-322`, signature deref → dead code); `static HANDLE SamKeyHandle` | **High** |
| **registry.c** | 419 | Nt*Key wrappers + buffer math | `KEY_VALUE_FULL_INFORMATION` offset arith (`:289`) | **Medium** |
| **alias.c** | 371 | Alias (local-group) SID membership w/ bidirectional keys | dual-registry sync invariant; `ConvertSidToStringSidW` | **Med** |
| **domain.c** | 352 | Name↔RID map, account-SID creation, bulk member removal | `SampCreateAccountSid` appends RID to domain SID (`:303-350`) | **Med** |
| **display.c** | 202 | In-memory user display cache (largely stub) | global `UserListHead`/count (`:23-25`); RID hex parse | **Low-Med** |
| **group.c** | 173 | Group member arrays (ULONG RID arrays) | ULONG array pack/unpack (fuzz) | **Low-Med** |
| **utils.c** | 163 | String-resource loader + **SID/RID math** | `AppendRidToSid`/`SampGetRidFromSid` pure SID byte math (fuzz) | **Low-Med** |

**Interface:** `sam.idl` (46 KB) → ~52 wire `Samr*` methods; generated `sam_s.{c,h}`.

**Spec (`samsrv.spec`):** ~52 real `Samr*` + `SamIConnect`/`SamIInitialize`/`SamIFreeVoid` +
**18 `SamIFree_SAMPR_*`**; **~90 `@ stub`** (DS/GC/promotion/replication `SamI*` family) →
near-free no-op stubs.

**Crypto = 3 FFI shims only:** `SystemFunction006/007/013` (`extern "system"`). The
SetInfo password path has a C `/* FIXME: Decrypt NT/LM password */` (`samrpc.c:7507`) — preserve.

**Fixed-data structs** (`samsrv.h`): `SAM_{USER,DOMAIN,GROUP,ALIAS}_FIXED_DATA` serialized
verbatim as `REG_BINARY` under the `L"F"` attr → `#[repr(C)]` + `size_of` asserts (ntkrnl-sys
pattern). RID alloc = `NextRid` field read-increment-write.

**Global state →** `OnceLock` (`SamKeyHandle` `database.c:15`, `Empty{Nt,Lm}Hash`,
`SampProductType`) · `RwLock` (`SampResource` — acquired in nearly every method) · `Mutex`
(display cache `display.c:23-25`) · immutable (5 mapping tables).

---

## 4. Differential-fuzz strategy (the `rtl-difftest` pattern)

For the pure byte-math pieces we **translate and prove bit-for-bit equivalence** vs the genuine
compiled C — strictly better than C-anchoring (achieves the translation goal *with* an oracle).
Host crates (`cargo test`, **no QEMU**), `publish=false`, workspace members.

**The 4 load-bearing tricks (copy from `sdk/lib/rust/rtl-difftest/`):**
1. `build.rs` writes a wrapper TU: `#define Foo ref_Foo` for each public symbol, then
   `#include "<repo>/…/genuine.c"` verbatim → genuine & Rust coexist as `ref_Foo` vs `cabi::Foo`.
2. `cc::Build` with **`.cargo_metadata(false)` + emit each object as
   `cargo:rustc-link-arg=<obj>`** (a build-script `rustc-link-lib` does NOT reach integration
   tests; link-args do).
3. **Force widths:** `typedef unsigned short WCHAR;` and `typedef int NTSTATUS;` in `refstub/`
   (host `wchar_t` is 32-bit, host `long` 64-bit → would break u16 strings + `NT_SUCCESS`).
   Replace host `wcslen`/`swprintf` with u16-aware models.
4. **Minimal `refstub/`** replaces the genuine header chain; stub the unused link surface as
   unprototyped `int f(){}`. `scrape_protos()` (copy verbatim) auto-emits forward prototypes.

`tests/diff.rs`: `extern "C" { fn ref_*(...) }`, an xorshift RNG, per-iter call both + `assert_eq!`
on return + every out-param + the whole dest buffer (incl. untouched tail); exhaustive where the
domain is small (e.g. all `0..=0xFFFF`).

**Concrete fuzz targets (translate → fuzz):**

*lsasrv:* `security.c` `LsapCreate{Policy,Account,Secret,Token}Sd` (the ACE-size arith) ·
`lookup.c` `LsapIsPrefixSid`/`LsapGetRelativeIdFromSid`/`LsapCopySid` ·
`privileges.c` the 5 bijective lookup fns · `utils.c` `LsapAppendRidToSid` ·
`registry.c` `LsapRegQueryValue` blob unpack.

*samsrv:* `user.c` `SampGet/SetLogonHoursAttribute` (bitfield), `SampAdd/RemoveGroupMembership*`
(array shift), `SampSetUserPassword` (OWF-history rotation — the "password" fuzz, but it's byte
rotation not crypto) · `group.c` `SampAdd/RemoveMemberToGroup` · `utils.c` `AppendRidToSid`/
`SampGetRidFromSid` · `domain.c` `SampCreateAccountSid` · `security.c` the 6 SD builders
(`rtl-difftest/build.rs:652+` already has a SD-packing TU to copy) · the `SAM_*_FIXED_DATA`
`#[repr(C)]` round-trips.

*(Crypto itself: none in samsrv. If an advapi32-crypto lane opens, `crypt_lmhash.c`
`SystemFunction006/007/001` is the textbook difftest leaf — out of scope here.)*

---

## 5. Sequencing & milestones

**Sequence: lsasrv → samsrv** (lsasrv establishes the shared SD/SID + RPC-glue patterns; do the
9.8K `samrpc.c` with those in hand).

**lsasrv:**
- **M0 — Spike (de-risk the cohab end-to-end).** `lsasrv-sys` (bindgen `lsa_s.h`) + `lsa-stub`
  (generated Rust stub via `generate_rpc_stub_rust`) + translate **one** method (`LsarOpenPolicy`)
  + the `LSAPR_HANDLE_rundown` + MIDL alloc/free. Prove: links behind the Rust stub table,
  dispatches, boots i386+amd64. *(This validates the whole shape before any bulk spend.)*
- **M1 — Centralize substrate:** `lsasrv-sys` complete; globals (`OnceLock`/`Mutex`); the lone
  DUMMY_PSEH fix (`database.c:833` → validate-before-deref); shared mapping/helper signatures.
- **M2 — Leaf/pure files + difftests:** privileges, utils, security (SD builders), registry,
  lookup SID predicates → with `lsasrv-difftest` proving parity.
- **M3 — DB + stores:** database.c, policy.c, session.c.
- **M4 — LPC/port:** authport.c, srm.c, notify.c, authpackage.c (mirror `lsalib-rust` in reverse).
- **M5 — lsarpc.c:** fan out the 75 `Lsar*` methods in batches (flat/independent); integrate
  centrally; cohab flip; ×3 gate.

**samsrv (after lsasrv patterns exist):**
- **S0** — `samsrv-sys` (bindgen `sam_s.h`) + `sam-stub` + the 3 `SystemFunctionXXX` FFI +
  `SAM_*_FIXED_DATA` `#[repr(C)]` + size-asserts + the lone DUMMY_PSEH fix (`database.c:309`).
- **S1** — pure fuzz crates (`samsrv-difftest`): user.c/group.c/utils.c/domain.c packers +
  security.c SD builders.
- **S2** — registry.c + database.c (I/O boundary).
- **S3** — fan out `samrpc.c` 68 methods in batches; setup.c + ~90 spec stubs (mechanical) last.
- **S4** — cohab flip; ×3 gate.

**Model/loop:** the proven msgina pipeline — guarded rustconv (worktree-pinned `PYTHONPATH`),
**DeepSeek-first + c2rust anchoring**, Sonnet fallback, wine traces behind the `wine_debug`
feature. Parallel per-method subagents write `.rs` to `~/vibeos-rust-staging/`; integrate +
build + ×3-smoke centrally.

---

## 6. Risk register

| Risk | Mitigation |
|---|---|
| **SD/ACL/SID pointer-math** (captures(none) class, sibling to msgina find.rs cast-before-offset) | difftest vs compiled C (§4); `lint_kernel.py`/`write_volatile` discipline |
| **DUMMY_PSEH dead-except** (1 site each, both mutate output) | validate handle pointer/signature in safe Rust *before* deref; never rely on the catch |
| **i386 stdcall `@N`** on `Lsar*`/`Samr*`/`SamI*`/`LsaI*` | `#[no_mangle] extern "system"` (auto `_Name@N`); `widl_to_rust.py` reads `.obj` for stub decorations; `verify_abi.py` |
| **Constant-placeholder bugs** (the project's signature class) | run `scan_inline_flag_literals` on every new crate (caught real ×3 bugs) |
| **samsrv security.c latent `DaclSize`/`SaclSize` bug** | translate faithfully + flag in commit; do not silently "fix" (preserve C behavior) |
| **Boot-critical** (any regression = no logon) | ×3 cold-smoke gate; spike (M0/S0) before bulk; arm64 flake-tolerant per new policy |
| **Volume** (75 + 68 methods) | flat files → parallel per-method subagents over a centralized substrate |
| **`MIDL_user_allocate`/`free` + `*_HANDLE_rundown`** must exist or stub won't link | provide in `lsasrv-rust`/`samsrv-rust` (cdecl `__RPC_USER` / `extern "system"`) |

---

## 7. Open items / decisions

- **M0/S0 spike first** before committing to bulk translation (validates the cohab on real hardware).
- **arm64:** these are boot-critical → arm64 boot is exercised, but per the new gate policy
  (2026-06-21) arm64 is flake-tolerant (no-worse-than-baseline) while the open `KiTrapReturn`
  SPSR flake is chased in arm64-love's lane. i386+amd64 stay hard boot-gates.
- **Coordination:** the lsass spine (`lsass-sys`/`lsalib-rust`/`lsass-cargo`) is on mainline;
  no active owner conflict, but flag in `#swarm` before the cohab flip (boot-critical path).
- **Cost:** lsasrv ~15K + samsrv ~16.5K LOC. Per-function DeepSeek+c2rust; budget with
  `rustconv estimate` "sync @5x" + the live counter. samrpc.c (9.8K) is the single cost driver.

---

## Key file references
- Pattern: `tools/widl_to_rust.py` · `sdk/lib/rust/midl-rt/src/lib.rs` ·
  `sdk/lib/rust/svcctl-stub/build.rs` · `sdk/lib/rust/services-rust/src/rpcserver.rs` ·
  `sdk/lib/rust/reactos-link/src/lib.rs:1860` (`generate_rpc_stub_rust`) ·
  `sdk/cmake/widl-support.cmake:82`
- Reuse: `sdk/lib/rust/lsass-sys/` · `sdk/lib/rust/lsalib-rust/src/lsa.rs` ·
  `sdk/lib/rust/lsass-rust/src/lsass.rs` · `sdk/lib/rust/lsass-cargo/build.rs`
- Difftest: `sdk/lib/rust/rtl-difftest/{build.rs,tests/diff.rs,refstub/}` ·
  `sdk/lib/rust/cnt-difftest/`
- Source: `dll/win32/lsasrv/` · `dll/win32/samsrv/` ·
  `sdk/include/reactos/idl/{lsa,dssetup,sam}.idl`
- Crypto leaf (out-of-scope ref): `dll/win32/advapi32/wine/crypt_lmhash.c`
