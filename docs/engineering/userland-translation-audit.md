# Userland (usermode DLL/EXE) Translation Audit

**Scope:** the usermode side of the OS — `dll/win32/*`, `dll/ntdll`, the `win32ss`
client DLLs (`user32`/`gdi32`), and the EXE spine. Companion to the existing
component docs (`user32-translation-audit.md`, `ntdll-translation-audit.md`,
`smss-csrss-translation-audit.md`); this one is the **umbrella map** that was
missing.

**Last updated:** 2026-06-21. Built from a fan-out research pass over the
ReactOS-native client DLLs (LOC/native-vs-Wine/problem-points per target).

**Why this doc exists:** agents kept re-flagging finished or low-value work as
"remaining userland C" (dnslib's empty stubs, basesrv/csrsrv already-done). This
records the real frontier so we stop re-deriving it, and ranks the
**ReactOS-native** targets (the Rust-is-the-goal ones) over the Wine-imported
ports (low value — translating CodeWeavers' code isn't translating *our* OS).

---

## What's already Rust (the runtime spine — DONE)

- **`ntdll`** (ntdll-rust, cargo-driven) and **`kernel32`** (kernel32-rust +
  kernel32-shared-rust + kernel32-vista-rust) — the two core client DLLs, 100%
  Rust body at link (only irreducible DllMain/asm/stubs floor remains).
- **EXE spine:** `smss / csrss / services / lsass / winlogon / userinit` (all Rust).
- **CSR servers:** `winsrv`, `csrsrv`, `basesrv` (all Rust, cargo-driven).
- **Libs:** version, mlang, mscms, cabinet, dnslib (effectively — see its note),
  conutils, concfg, guids, lsalib, evtlib, …
- **🔄 In flight:** **`user32`** (`win32ss/user/user32`, ~60K LOC, Wave-1/2
  landing via `user32-rust`).

So the boot-to-desktop spine is Rust. The remaining userland is the **win32
client-DLL tail** — mostly *not* boot-critical, and a large fraction Wine-imported.

---

## Native-target priority ranking (the actionable list)

Ranked by ROI = (native fraction × strategic value) ÷ difficulty. "Native%" is
ReactOS-authored vs Wine-imported (the Rust-is-the-goal axis).

| # | Target | C LOC | Native % | Difficulty | Boot? | Coord | Verdict |
|---|--------|------:|---------:|------------|-------|-------|---------|
| 1 | **userenv** | 6.5K | ~100% | Low–Med | ✅ logon path | none | **Best first pick** — clean, all-native, self-contained |
| 2 | **gdi32** | 33K | ~54% | High | ✅ (explorer draws) | none | **Highest strategic value** — completes the USER/GDI client with user32 |
| 3 | **advapi32 (reg/ + token/ + sec/)** | ~20K native | ~60% | Med–High | ✅ heavily | RPC contract w/ services/lsass (already Rust) | High value; slice reg/ first |
| 4 | **ws2_32** | 14K | ~100% | High | ❌ | netstack (AFD/WSP contract) | All-native; global catalog is the hard part |
| 5 | **iphlpapi** | 8K | ~70% | High | ❌ | **netstack (TDI query contract)** | Gated on netstack TDI surface settling |
| 6 | **secur32** | 7.5K | ~7% native | Med | ❌ | lsalib/lsass (Rust) | Mostly Wine, but small + tractable |
| 7 | **msafd** | 6.4K | ~100% | Very High | ❌ | **netstack (lockstep)** | Hardest net DLL; do with netstack only |
| 8 | **netapi32** | 17.5K | ~55% | Very High | ❌ | RPC + SAM | Deprioritize — RPC/NBT/SAM heavy |
| 9 | **setupapi** | 32K | ~55% | Very High | ❌ | umpnpmgr (still C) | Deprioritize — INF engine + CM/RPC, no Rust peer |
| 10 | **shell32** | ~80K (64K C++) | ~80% native | Very High | ✅ (explorer) | none | The native *prize* but ATL/C++ COM — needs a vtable strategy first |
| — | **shlwapi** | ~27K | ~10% native | Med | ✅ (explorer) | none | **Skip** — ~90% Wine; only path/string/url algos worth anything |

**Bottom line:** start with **userenv** (easiest native win) and/or **gdi32**
(completes the graphics client). advapi32's `reg/` is the highest-value
boot-exercised slice. Everything net-related (ws2_32/msafd/iphlpapi) needs
netstack coordination. shell32 is the biggest native body but is C++/ATL/COM —
gated on proving a COM-vtable→Rust pattern. Skip the Wine ports (shlwapi, and the
crypto/INF/NBT pieces inside the mixed DLLs).

---

## Per-target detail

### 1. userenv — `dll/win32/userenv/` — **Low–Med, ~100% native, boot-relevant**
User profiles, environment blocks, Group Policy client. 6,473 LOC / 10 files,
**0% Wine** (Kohl/Poussineau/Belusca). Biggest: `profile.c` (2,375), `desktop.c`
(913), `environment.c` (857), `gpolicy.c` (602). 117 exports. Imports advapi32
(registry/SID), user32, kernel32, ntdll — **no RPC, no network, clean FFI**.
Consumers: msgina, syssetup, seclogon, umpnpmgr, shell32 → on the logon path.
**Problems:** profile registry/hive walking, env-block byte-exact layout, ACL
inheritance on dir creation, 1 PSEH site in `environment.c` (→ dead code under
DUMMY_PSEH). **Approach:** slice `sid.c`/`registry.c`/`directory.c` leaves first
to validate the advapi32 shims, then `environment.c`, then `profile.c` bulk.
rustconv-friendly. ~1–1.5 waves. **No external coordination.**

### 2. gdi32 — `win32ss/gdi/gdi32/` — **High, ~54% native, completes USER/GDI**
The GDI client (usermode half of win32k). 33,199 LOC / 47 files. **~54% native**
(`objects/` 13,970, `misc/` 3,747 — font/dc/bitmap/region/text/coord) + **~46%
Wine** (`wine/` metafile/EMF/WMF engine, 15,342 — its own sub-grind, lower value).
610 exports (~168 are thin `NtGdi*` syscall forwarders), 0 stubs. **Direct
precedent: `user32-rust`** (sibling client DLL, same cohab pattern); kernel side
already Rust (`win32k-rust`, incl. `gdibatch.rs`/`gdipool.rs`). **Problems:** the
GDI handle/userdata cache (`GdiGetHandleUserData`/`GdiGetDcAttr` — layout-exact
raw-pointer reads from the shared handle table; a wrong offset is a silent fault),
the per-thread TEB batch buffer (`GdiAllocBatchCommand` → `NtCurrentTeb()->
GdiTebBatch`), the `HANDLE_METADC` dispatch macro on every draw entrypoint, the
Wine metafile record marshalling, all-stdcall ABI (run verify_abi.py), light SEH
(4 sites in bitmap/font — dead code under DUMMY_PSEH). No asm/COM. **Approach:**
stand up `gdi32-rust` + an `ntgdi.rs` thunk layer (user32-rust template); flip the
thin forwarders first (`arc.c`/`pen.c`/`brush.c`/`colorspace.c`/`linedda.c`) to
prove decoration+cohab, then coord/region/text, defer `gdiobj.c`+batch/cache
helpers and the whole `wine/` engine to last. rustconv-batch the forwarders;
hand/sync-translate the cache+batch. **Large grind**, but proven precedent.

### 3. advapi32 — `dll/win32/advapi32/` — **Med–High, mixed (~60% native core)**
Registry, SCM client, security/token/LSA, crypto, eventlog. 32,117 LOC / 30 files.
**Native core ~20K** worth translating: `reg/` (6,456 — registry, the highest
value, boot-exercised constantly), `sec/` (6,127 — SID/ACL/SD/LSA, minus the Wine
hybrid `misc.c`), `service/scm.c`+`sctrl.c` (4,497 — SCM RPC client), `token/`
(345), most of `misc/` (logon/sysfunc/shutdown). `wine/security.c` (4,192) is a
**near-native hybrid** (143 Nt/Rtl sites, `#ifdef __REACTOS__`) — effectively
translatable. **Defer/skip the true Wine ~6K:** `wine/crypt.c`, `cred.c`,
`crypt_des/lmhash`, and `service/eventlog.c`. 724 exports (125 stubs). Imports
ntdll (heavy Nt*/Rtl* registry+security), rpcrt4 → **services.exe (services-rust)
and lsass (lsass-rust), both already Rust** (stable RPC contract, shared IDLs).
**Problems:** registry HKEY default-handle table + A/W + 1 SEH (DUMMY_PSEH); SCM
RPC stubs (`svcctl_c.c` stays C/FFI); variable-length SID/ACL pointer math
(cast-before-offset / element-vs-byte class — review by hand). **Approach:**
`token.c` (345, quick win) → `reg/` (highest value, smoke-exercised) → `sec/` +
`wine/security.c`. rustconv-batch reg/sec; hand-integrate the RPC `service/`.
**Leave Wine crypto as permanent C cohab.**

### 4. ws2_32 — `dll/win32/ws2_32/` — **High, ~100% native, netstack-coupled**
Winsock 2 API DLL. 14,220 LOC / 36 files, **all ReactOS-native**. 48 forwarder
ordinals + the full WSA2 surface. Dispatches to a transport provider (msafd) via a
`WSPPROC_TABLE`; does **not** hit `\Device\Afd` directly. The netstack "winsock
un-gate" was a build/stage change, **not** code here (netstack-rust is all
in-kernel). **Problems:** process-wide provider/namespace **catalog** with
refcounts+lock (`dcatalog.c`/`nscatalo.c` — the global-mutable-state hazard), the
SPI fn-pointer upcall tables (`dprovide.c`), `IsBad{Read,Write}Ptr` parameter
guards (`sockctrl.c:42` — the exact surface the kernel32 DUMMY_PSEH IsBad*Ptr bug
broke → preserve semantics), `WSAAsyncSelect` message-pump async, all-stdcall.
**Approach:** leaf modules first (`addrconv.c`, byte-order, `sputil.c`,
`enumprot.c`) via rustconv; defer the catalog + SPI dispatch (they own the global
state). Coordinate the WSP↔AFD IOCTL/EA contract with netstack.

### 5. iphlpapi — `dll/win32/iphlpapi/` — **High, ~70% native, gated on netstack**
IP Helper (adapter/route/ARP/TCP-UDP tables, ICMP). 7,946 LOC / 16 files. ~70%
native: the `*_reactos.c` TDI backends (ifenum/ipstats/route/resinfo) + `address.c`
(GetAdaptersAddresses) + `icmp.c`; the dispatch shell `iphlpapi_main.c` (3,344) is
Wine-patched-by-ReactOS. **Live counterparty: `netstack-rust/src/tdidev.rs`**
registers `\Device\Tcp|Udp|Ip` and handles `IOCTL_TCP_QUERY_INFORMATION_EX` —
iphlpapi's backends are the *client* of exactly that. **Coordination required:**
pin the entity-list/interface/addr/route/MIB query-class + EA contract netstack
answers (incl. the no-EA enumerate path flagged in the merge notes) before
translating. **Problems:** TDI IOCTL marshalling (nested EA/`TCP_REQUEST_QUERY_
INFORMATION_EX`), variable-length `MIB_*` row tables, the two-call
`AllocateAndGet*FromStack` size/fill dance (ERROR_INSUFFICIENT_BUFFER semantics
byte-exact). **Approach:** `address.c`/`icmp.c` as an early self-contained slice;
hand-translate the IOCTL/MIB marshalling (rustconv fumbles var-length buffers);
**sequence after netstack stabilizes the TDI query surface.**

### 6. secur32 — `dll/win32/secur32/` — **Med, ~90% Wine, small/tractable**
SSPI client + LSA LPC client + built-in NTLM/Negotiate/Kerberos. 7,551 LOC / 13
files. Only ~7% truly native (`lsalpc.c` 419, Ionescu — LSA LPC to lsass, links
`lsalib`); the SSPI core (sspi/wrapper/thunks, 3,164) is Juan Lang's Wine adopted
into ReactOS, and `wine/` (3,861) is pure Wine providers. **Adjacent Rust:**
`lsalib-rust` + `lsass-rust`/`lsass-sys` (the LSA peer is being translated).
**Problems:** SSP DLL loading + `SecurityFunctionTableA/W` fn-ptr dispatch + A/W
thunks, global linked-list package registry, LPC port to lsass (covered partly by
lsalib-rust). The `wine/` crypto leaves (base64/hmac-md5/ntlm) are self-contained
algos. **Approach:** rustconv the `wine/` algo leaves; hand-port the SSPI core
fn-ptr tables; wire `lsalpc.c` against `lsalib-rust`. ~1–2 weeks. Caveat: ~90%
Wine, so value is "convert a dependency," not "preserve ReactOS IP" — but small
size + existing lsa Rust neighbors make it more tractable than setupapi.

### 7. msafd — `dll/win32/msafd/` — **Very High, ~100% native, netstack lockstep**
The AFD service provider (Winsock SPI → `\Device\Afd` IOCTLs). 6,355 LOC / 5 files
(`dllmain.c` 4,415 is one giant SPI dispatch). All ReactOS-native (Wine only for
`debug.h` macros). **Two halves of one wire protocol with
`netstack-rust/tdidev.rs`** — the AFD packet/IOCTL/EA structs must be a single
shared source of truth. **Problems:** async/overlapped I/O + hand-built AFD request
structs + `NtDeviceIoControlFile` + IO_STATUS_BLOCK/event/APC (binary contract
must match the kernel AFD exactly), `WSPSelect`/`AFD_POLL_INFO`, event-select
bookkeeping, `*lpErrno` out-params (captures(none) → write_volatile). Zero SEH.
**Approach:** `helpers.c`/`event.c` first to validate `msafd.h` layouts, then
`sndrcv.c`, then `dllmain.c` per-SPI — **only in lockstep with netstack.** The
single hardest usermode net DLL.

### 8. netapi32 — `dll/win32/netapi32/` — **Very High, ~55% native, deprioritize**
LanMan client (NetUser/Group/Wksta/Server, NetBIOS/NBT, netlogon). 17,594 LOC / 22
files. ~55% native (group_new/netlogon/srvsvc/wksta_new — RPC clients to local
services), ~45% Wine (user.c 4,093, local_group, nbt.c 1,546, netbios). **Problems:**
8 MIDL `*_c.c` RPC stubs (keep as C cohab), per-file `RpcTryExcept` (dead code under
DUMMY_PSEH → rework error paths), stateful NBT name cache/command queue, `Netbios()`
NCB union ABI, many `NET_*_INFO` levels. Imports rpcrt4 + iphlpapi + ws2_32 +
delay-load samlib/secur32. **Not boot-critical**, callers (net.exe/usrmgr) aren't on
the smoke north-star. **Approach:** leaf utils first; keep MIDL stubs C; one RPC
file per agent; defer nbt/netbios/user.c. **Lower priority.**

### 9. setupapi — `dll/win32/setupapi/` — **Very High, ~55% native, deprioritize**
Device/driver install + INF parsing + Config Manager. 32,159 LOC / 17 files. ~55%
native (`cfgmgr.c` 8,876 — CM_*/PNP_* RPC to umpnpmgr; `devinst.c` 6,312 — SetupDi,
Poussineau), ~45% Wine (the INF engine: `parser.c`/`install.c`/`queue.c`, Julliard).
**No Rust peer:** the PnP-manager *server* (umpnpmgr) is still C; the `pnp_c.c` MIDL
client must be called via FFI. **Problems:** the Wine INF tokenizer/state-machine +
file-copy queue + `PSP_FILE_CALLBACK`, 64 `RpcTryExcept` blocks in cfgmgr (dead code
under DUMMY_PSEH), registry-heavy devinst, opaque cross-call handles (HDEVINFO/HINF/
HSPFILEQ). Not boot-gating. **Approach:** if pursued, the self-contained Wine algos
(`stringtable.c`/`dirid.c`/`diskspace.c`/`parser.c`) are the only clean rustconv
slices; defer cfgmgr (RPC) + devinst/install/queue. **Lowest priority of the
medium-size set** — large, ~45% Wine, no Rust peer.

### 10. shell32 — `dll/win32/shell32/` — **Very High, ~80% native, but C++/ATL/COM**
The shell library. **~80K LOC total: 64K C++ (ATL/COM, ReactOS-native) + 15K Wine
C + 0.3K native C.** This is the largest ReactOS-native userland body and a real
prize — but it's **C++/ATL**, not C: pervasive COM vtables (IShellFolder/IShellView/
IContextMenu/IShellLink) via `CComObject`/`CWindowImpl` multiple-inheritance layouts
Rust can't natively express, PIDL pointer-walking, full windowing/message callbacks.
**rustconv targets C, not C++/ATL.** Boot-relevant (explorer links it). **Approach:**
worth translating eventually (it's our IP), but **gated on first proving a
COM-vtable→Rust `#[repr(C)]` pattern** on a small IUnknown-bearing module. The 15K
`wine/` C is the easy part but low-value. Months of grind. **Defer until a COM
strategy exists.**

### shlwapi — `dll/win32/shlwapi/` — **SKIP (~90% Wine)**
Path/string/URL/registry utility grab-bag. ~27K LOC, **~90% Wine** (`-D__WINESRC__`,
Meissner/Griffiths/CodeWeavers). Only ~2.8K native (rosordinal.c + 3 ATL C++ files).
857 ordinal-heavy exports. The path/string/url algos are clean Kani-able C (good
rustconv fit *if* ever needed), but translating a Wine port fails the
native-code-value test. **Skip in favor of native targets.**

---

## Cross-cutting notes

- **Native-vs-Wine is the priority axis.** "Rust is the goal" means *our* OS code.
  The biggest C DLLs (comctl32, mshtml, ole32, msi, oleaut32, crypt32, gdiplus,
  windowscodecs, msxml3, jscript, vbscript, riched20, usp10, urlmon, wininet) are
  Wine imports — **out of scope** for translation value. This audit deliberately
  covers only the ReactOS-native client DLLs.
- **The reusable-substrate fraction is low.** Unlike the kernel (rtl/algorithm
  crates) or netstack (smoltcp), userland DLLs are mostly thin NT/RPC glue — the
  NT-IFS/RPC/COM boundary dominates. Expect file-by-file cohab grinds, not big
  algorithmic wins. (Same conclusion the filesystems lane reached for FS drivers.)
- **ABI:** every `.spec`-exported WINAPI fn is **stdcall** → i386 `_Name@N`; use
  `extern "system"`, run `verify_abi.py`. The kernel32 RPC-wait regression
  (`TranslateStdHandle` cdecl-vs-stdcall) is the cautionary tale.
- **DUMMY_PSEH:** any translated `_SEH2_EXCEPT`/`RpcTryExcept` body that mutates
  state must be **dead code** (the IsBad*Ptr / cdfs trap). RPC fault handling must
  become explicit status checks, not SEH filters.
- **Pointer-math classes to watch:** variable-length SID/ACL (advapi32 sec/),
  PIDL chains (shell32), MIB table rows (iphlpapi), metafile records (gdi32 wine/) —
  all cast-before-offset / element-vs-byte hazards from the field manual.
- **RPC/MIDL stubs stay C.** The generated `*_c.c` NDR clients (advapi32 svcctl/lsa,
  setupapi pnp, netapi32 ×8) are called via FFI; don't translate generated NDR.
- **Coordination map:** net DLLs (ws2_32/msafd/iphlpapi) ↔ **netstack** lane;
  advapi32 SCM/LSA ↔ **services-rust/lsass-rust** (RPC contract, already Rust);
  secur32 ↔ **lsalib-rust/lsass-rust**; setupapi ↔ umpnpmgr (still C).
- **No gate needed for this doc** (doc-only). Translation lanes off it follow the
  standard cohab + ×3-cold-gate flow.

---

## Recommended next pickups (if a userland lane opens)

1. **userenv** — easiest all-native, boot-path, no coordination. Best warm-up.
2. **gdi32** — highest strategic value: with user32 (in flight) it makes the
   entire USER/GDI *client* surface Rust. Proven user32-rust precedent.
3. **advapi32 `reg/` (+ `token/`)** — high-value, boot-exercised, clean Nt*/Rtl*
   FFI; services/lsass RPC peers already Rust.

Net work (ws2_32/iphlpapi/msafd) only with netstack coordination; shell32 only
after a COM-vtable→Rust pattern is proven; skip the Wine ports.
