# wow64gen — WoW64 syscall thunk generator

Re-runnable, deterministic codegen that turns our `.spec` syscall table into the
C thunk table `wow64.dll` dispatches through. Part of WoW64 Phase D
(`docs/engineering/wow64-plan.md`).

## What it is (and what it is not)

| Tool | Reads | Emits |
|------|-------|-------|
| bindgen | C headers | Rust **type** declarations |
| rustconv | C function **bodies** | Rust (via LLM) |
| **wow64gen** | a **`.spec` table** | C **marshaller** thunks (mechanical) |

wow64gen is a `spec2def`-class tool: no LLM, no network, same input → byte-identical
output. Add a syscall to the `.spec`, re-run wow64gen, and the thunk table regrows.
That is the "make it easy for ourselves later" property — as we translate more of
the API surface, thunking scales for free.

## The core idea

Under WoW64, a 32-bit guest calls a syscall; the CPU layer traps it and hands
`wow64!Wow64SystemServiceEx(serviceNumber, PULONG guestArgs)` the service number
(guest EAX) and a pointer to the guest's 32-bit argument dwords. Each thunk:

1. reads the guest 32-bit argument dwords,
2. **widens** each to the host 64-bit ABI, and
3. calls the real 64-bit `Nt*` (from the native ntdll), returning its NTSTATUS.

**The simplification that makes this mostly mechanical:** a 32-bit guest pointer
is already a valid 64-bit pointer (guest VAs are below 4 GB), so a `ptr` argument
widens by a plain zero-extension. Scalars widen by their width. So the flat
majority of `Nt*` are pure codegen.

## The honest limitation (deep marshalling)

A `ptr` in the `.spec` describes the pointer **value**, not the pointed-to data.
When it points at a struct whose **32-bit and 64-bit layouts differ**
(`OBJECT_ATTRIBUTES`, an `IO_STATUS_BLOCK` with its `ULONG_PTR Information`, a
context block with embedded pointers/handles/`SIZE_T`), passing the guest struct
straight through is **wrong** — those need a hand-written deep marshaller that
rebuilds the 64-bit struct on the stack.

wow64gen cannot infer struct-ness from the `.spec`, so it takes a **deep-marshal
list** (`manual.txt`) of names to force to `WOW64_NEEDS_MANUAL`. Those emit a loud
stub (table entry `Thunk = 0`) instead of a silently-wrong auto thunk. Implement
each by hand in `wow64.c` and drop it from `manual.txt`. **When unsure, list it** —
a loud stub is always safer than a wrong thunk.

## Usage

```bash
python3 tools/wow64gen/wow64gen.py \
    --spec dll/ntdll/def/ntdll.spec \
    --arch arm64 \
    --sysfuncs ntoskrnl/include/sysfuncs.h \
    --manual-list tools/wow64gen/manual.txt \
    --out-c <dest>/wow64_thunks.gen.c --stats
```

`--arch` picks which `-arch=`-gated `.spec` entries apply (arm64 today for
x86-on-arm64; amd64 later for the classic wow64cpu path). Output is a single
self-contained C file: extern decls for the 64-bit `Nt*`, one `static` thunk per
auto syscall, and `Wow64ThunkTable[]` indexed by service ordinal (with
`Wow64ThunkCount`). `wow64.c` includes it and dispatches
`Wow64ThunkTable[serviceNumber].Thunk(guestArgs)`.

Current ntdll surface (arm64): **296 syscalls — 268 auto, 28 manual**.

## Caveats / TODO

- **Service-number alignment: SOLVED (2026-07-10).** The table is indexed by
  `ntoskrnl/include/sysfuncs.h` order — the SAME list the i386 ntdll stubs (and
  every arch's `KiServiceTable`) number themselves from, so guest EAX indexes
  this table by construction. The `.spec` contributes argument *types* only; a
  sysfuncs entry missing from the `.spec`, or one disagreeing on dword count,
  emits a loud manual stub (zero such today).
- **`out` scalars.** A `ptr` to a plain output scalar (e.g. a `PHANDLE`) is
  layout-identical and auto-thunks correctly. A `ptr` to an output *struct* still
  needs the manual list.
- **Not yet wired into the build.** The generator + its test are gated by
  `vibeos-cli check-tooling`; emitting into the wow64.dll build is the next Phase
  D step (blocked behind Phase C M2 — guest code must execute first).

## Tests

`tools/wow64gen/test_wow64gen.py` (run by `vibeos-cli check-tooling`): spec
parsing, Zw/arch filtering, arg-width classification, manual-list forcing, the
emitted-C shape, and determinism (identical input → identical output).
