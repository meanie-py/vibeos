# Handoff: msafd WSPSocket hangs on the first usermode `socket()` (winsock round-trip RED)

> **ARCHIVED 2026-07-10** — the winsock vertical went GREEN (i386 roundtrip `946935e042a`); the two reusable bug classes landed in the field manual (`adf78d014fc`). Historical handoff. Outstanding items (if any) live in `docs/product/vibeos-roadmap.md`.


**Status:** OPEN. The Rust msafd.dll cohab flip compiles + links on all 3 arches and the
i386 net stack boots (netstack `device::bringup()` fixed separately — see
`netstack-i386-virtio-bringup-hang.md`). But the usermode winsock round-trip
(`ws2_32 → msafd → afd → netstack`) is RED: **`WSPSocket` (the first `socket()` call) hangs
and never returns.** Two real translation bugs on that path were found + fixed this session
(details below); a third hang remains, precisely localized but not yet root-caused. Target:
Codex. Kernel networking is healthy (DHCP, ping, DNS, TDI TCP/UDP, outbound echo all pass) —
this is purely the usermode winsock provider.

## Symptom

`nettest.exe` (auto-spawned at boot on i386) prints:
```
[NETTEST] usermode winsock proof starting
[NETTEST] WSAStartup OK          <- ws2_32 loaded msafd, WSPStartup returned the proc table
                                 <- then socket() is called and NEVER returns
```
`nettest`'s `test_tcp()` loops socket→connect→send→recv 40× with `Sleep(1000)`, printing a
failure marker on each failed call. We see **no `socket() failed` marker and no retry** — the
process is stuck inside the first `WSPSocket`. Test: `test_usermode_winsock_roundtrip` fails
with a boot/marker timeout.

## Two bugs already fixed this session (keep these — they are real)

Both are in the `WSPSocket` → `SockGetTdiName` path and are **general translation-bug
classes** worth grepping for elsewhere:

1. **Wide-string literal mis-encoded as ASCII-bytes-as-u16** — `dllmain.rs` ~line 292 built
   the AFD device name `\Device\Afd\Endpoint` as `b"\\Device\\Afd\\Endpoint\0".as_ptr() as
   *const u16` and passed it to `RtlInitUnicodeString`. That reinterprets ASCII byte pairs as
   UTF-16 (garbage device name) and can over-scan for the u16 NUL. Fixed with a real
   `static AFD_ENDPOINT: [u16; 21]` (dllmain.rs:296). Same class as the afd.sys `\Device\Afd`
   fix. Grep: `b"[^"]*"\.as_ptr\(\) as \*const u16` with NO interleaved `\0` between chars.

2. **bindgen flexible-array member indexed with a bounds-checked `[]` → panic → `loop{}`
   hang** (THE important class). `SockIsTripleInMapping` (helpers.rs) did
   `(*Mapping).Mapping[Row as usize]`. The C `WINSOCK_MAPPING` ends in a flexible array
   (`WINSOCK_MAPPING_ROW Mapping[]`); bindgen types it as `[T; 1usize]`. A Rust `[Row]` index
   is **bounds-checked**, so `Row >= 1` (every real mapping has multiple rows) triggers
   `panic_bounds_check`. Under this crate's `#[panic=abort]` + the `loop {}` panic handler
   (lib.rs), **a panic is an infinite hang, not a crash** — indistinguishable from a deadlock
   on the serial log. Fixed (helpers.rs:502) with pointer arithmetic:
   `let row = &*(*Mapping).Mapping.as_ptr().add(Row as usize);`.
   **Action for Codex:** grep the whole msafd (and other cohab translations) for
   `\.\w+\[\w+ as usize\]` / `[Idx]` indexing on any bindgen `[T; 1]` flexible-array member —
   this pattern silently hangs instead of crashing and is easy to miss.

Neither fix alone makes the round-trip pass — a third hang is upstream.

## The remaining hang — precisely localized

Breadcrumb trail (fresh clean build; `[MSAFD]` markers emitted via `OutputDebugStringA`):
```
[MSAFD] WSPSocket enter
[MSAFD] WSPSocket pre-tdiname
[MSAFD] sgtn list-traverse-done          (SockHelpersListHead empty, skipped — list head OK)
[MSAFD] sltl reg-open-ok
[MSAFD] sltl reg-open-status=0x00000000  (RegOpenKeyExW succeeded)
[MSAFD] sltl KeyHandle=0x000005c8         (valid registry handle)
[MSAFD] sltl pre-regquery-size
                                          <- RegQueryValueExW HANGS here, no further output
```
Call path: `WSPSocket` (dllmain.rs) → `SockGetTdiName` (helpers.rs) → `SockLoadTransportList`
(helpers.rs:~226) → `RegQueryValueExW(KeyHandle, "Transports", NULL, NULL, NULL, &size)` — the
**size query** for the `HKLM\SYSTEM\CurrentControlSet\Services\Winsock\Parameters` value
`Transports`.

Facts established:
- `RegQueryValueExW` is the **real advapi32 import** — verified in `msafd.dll`'s import table
  (`i686-w64-mingw32-objdump -p …/msafd.dll` shows `advapi32.dll … RegQueryValueExW`). No
  rogue Rust stub shadows it.
- `RegOpenKeyExW` returned `Status=0` and a **valid** `KeyHandle=0x5c8` — the args to
  `RegQueryValueExW` look correct (name is properly wide-encoded `b"T\0r\0a\0n\0s\0…\0\0"`).
- The C msafd calls `RegQueryValueExW` **identically** and the usermode winsock path worked
  with C msafd (amd64 net suite was 11/11 including usermode-winsock-TCP) — so this is
  **Rust-specific**, not an advapi32/registry-hive problem.

## The key clue: the hang location is NON-DETERMINISTIC across builds

- Earlier (partially-stale-rlib) builds sailed through **four** successful `RegQueryValueExW`
  calls (both size+data queries in `SockLoadTransportList` *and* `SockLoadTransportMapping`)
  and hung much later — in the flexible-array loop of `SockIsTripleInMapping` (bug #2 above).
- Fresh clean builds hang on the **first** `RegQueryValueExW`.

`RegQueryValueExW` therefore is not intrinsically broken (it demonstrably worked 4× in another
build). A call that sometimes works and sometimes hangs, whose exact failure point moves with
the binary, is the signature of **heap or stack corruption / an uninitialized-memory
dependency** — not a bug in RegQueryValueExW. Note every buffer on this path is
`HeapAlloc(GlobalHeap, 0, …)` — **flag 0 = NOT zeroed**, so any under-sized allocation or
partial fill leaves live garbage.

**Leading hypothesis:** an earlier bug in the `WSPStartup` / `DllMain` / early-`WSPSocket` /
`SockGetTdiName` path does an under-sized `HeapAlloc` + oversized write, a flexible-array
**write** past `[T;1]` (the write analogue of bug #2 — a `write_volatile`/store wouldn't even
panic, it would silently corrupt), or an uninitialized-struct read, corrupting the heap/CRT
state so the next allocator/registry call wedges. Find that, and both the RegQueryValueExW
hang and the residual instability should clear.

## Debug infrastructure already in the tree (uncommitted)

`glue.rs` (remove before landing — marked "TEMP"):
- `pub unsafe fn dbg_mark(s: &[u8])` → `OutputDebugStringA` (same serial channel nettest uses).
- `pub unsafe fn dbg_hex(label: &[u8], val: u32)` → prints `label=0x????????`.

~24 `dbg_mark`/`dbg_hex` breadcrumbs are placed across `dllmain.rs` (WSP entry points) and
`helpers.rs` (SockGetTdiName / SockLoadTransportList / SockIsTripleInMapping). Extend these to
narrow further, then strip them all (`grep -rn 'dbg_mark(b"\[MSAFD\]\|dbg_hex(b"\[MSAFD\]'`)
plus the two glue fns + the `OutputDebugStringA` extern before committing.

## Reproduction

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-netstack
# FORCE-CLEAN the msafd rlib every time — cargo incremental leaves a stale rlib that gives
# misleading (different) hang points (this cost several cycles; see the nondeterminism note):
rm -rf build-i386/dll/win32/msafd/rust-msafd-target build-i386/dll/win32/msafd/libmsafd_rust.a
( cd build-i386 && ninja msafd && rm -f livecd.iso && ninja livecd )   # ISO won't regen without the rm
PY=/home/curi/Projects/os-playground/vibeos/.venv/bin/python
PYTHONPATH=. $PY -m pytest tests/qemu/test_net.py -v --arch i386-uefi-net \
    -k "usermode_winsock_roundtrip" --timeout 600 --skip-clean-build
# breadcrumbs land in the serial log:
grep -aoE '\[MSAFD\][^\r]*' qemu/i386-uefi-net_smoke_live.log
```
Note: `--arch i386-uefi-net` is a pytest arch, not a `vibeos-cli` arch. i386 auto-spawns
nettest.exe every boot (winlogon.rs:728; see triage.py), so any i386-uefi-net boot exercises
the winsock path. Each cycle is ~3.5 min (build + boot).

## Suggested debug path

1. **GDB usermode backtrace at the hang (decisive).** Attach via the kernel debug toolkit
   (`qemu/start.sh i386-uefi-net --gdb`, `qemu/README.md`), let it wedge in `RegQueryValueExW`,
   interrupt, and get the faulting thread's stack. Distinguishes "blocked on a lock inside
   advapi32/ntdll" from "caller stack corrupted / spinning in the panic handler `loop{}`". If
   the PC is in `loop {}` (the msafd `#[panic_handler]`), it is another silent panic — find
   which bounds-check/`unwrap`/slice fired (walk the stack to the msafd frame).
2. **Hunt the corruption source** in the pre-hang path (`WSPStartup`, `DllMain`,
   `SockGetTdiName`, `SockLoadTransportList`, `SockLoadTransportMapping`): look for
   `HeapAlloc(…, 0, N)` where N is computed (element-vs-byte size mistakes, off-by-one on the
   NUL, `sizeof` of the wrong type), and for any flexible-array **write** `x.Field[i] = …` on a
   bindgen `[T;1]` member (silent heap smash — the write analogue of the fixed bug #2).
3. **Bisect RegQueryValueExW** cheaply: temporarily replace the size-query+alloc in
   `SockLoadTransportList` with a fixed 4 KB buffer (skip the `lpData=NULL` size query). If it
   then proceeds, the specific `lpData=NULL` size-query form is implicated (a value/ABI issue
   in *that* call shape); if it still hangs, the corruption is upstream.

## Code pointers

| File | Role |
|------|------|
| `sdk/lib/rust/msafd-rust/src/misc/dllmain.rs` `WSPSocket` | the hanging entry; AFD_ENDPOINT fix at :296; NtCreateFile(\Device\Afd) at ~:612 |
| `sdk/lib/rust/msafd-rust/src/misc/helpers.rs` `SockGetTdiName` / `SockLoadTransportList` (~:226) / `SockLoadTransportMapping` (~:170) / `SockIsTripleInMapping` (~:478, flex-array fix at :502) | the registry-driven TDI-name resolution; the hang is in SockLoadTransportList's RegQueryValueExW |
| `sdk/lib/rust/msafd-rust/src/misc/dllmain.rs` `WSPStartup` (~:3516) / `DllMain` (~:4632, `InitializeListHead`) | winsock init that runs before WSPSocket — prime place for the corrupting bug |
| `sdk/lib/rust/msafd-rust/src/glue.rs` :190-205 | the temporary dbg_mark/dbg_hex infra |
| `sdk/lib/rust/msafd-rust/src/lib.rs` | `#[panic_handler] loop{}` — why panics look like hangs |

## Scope / non-goals

- The bug is a msafd usermode translation defect. Do **not** touch advapi32/the registry hive
  (RegQueryValueExW is correct; the C proves the path works).
- Keep the two confirmed fixes (AFD_ENDPOINT, flexible-array pointer-arith). Don't "fix" the
  hang by disabling SockGetTdiName / hardcoding a transport — that hides the corruption.
- Uncommitted alongside this: Codex's netstack `device.rs`/`hal.rs` i386-bringup fixes and the
  `helpers.rs` extern-convention split — leave them; they are correct and needed.
