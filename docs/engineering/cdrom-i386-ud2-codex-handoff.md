> **✅ RESOLVED 2026-06-14 (codex + review).** The i386 ud2 collapse is FIXED. Root: i386 LLVM
> InstCombine weaponized UB in the read+SRB chain. Fix = systematic UB removal, **not** an opt hack
> (Rust translation preserved; only 2 stubborn blocks moved to cohab C shims):
> - `ScratchBuffer_SetupReadWriteSrb` (scratch.rs): every aligned SRB/CDB field write →
>   `write_unaligned`/`write_volatile` at explicit byte offsets — removes the align-8 base-pointer
>   assumption that was the `poison` seed.
> - `dispatch.rs` (kmdf-sys): `.unwrap_unchecked()` → `match Some/None`; `.add()` → `.wrapping_add()`
>   (kills the inbounds-GEP + unreachable-on-None UB).
> - NT_ASSERT ABI unified to `extern "system"`/i32 (autorun/sense/glue); ScratchBuffer_* defs →
>   `extern "system"` to match callers.
> - Partial-MDL alloc + complete-or-continue decision → C shims
>   (`CdromRustScratchAllocatePartialMdl`, `CdromRustScratchCompleteOrContinueReadWrite`).
>
> **Verified (cold, traces stripped, `g_cdromTrace=0`):** i386 ✅ GREEN (4m02s), arm64 ✅ GREEN (35s).
> amd64 ❌ — but that's the SEPARATE pre-existing storage-layer mega-stall (task #75), reproduced
> identically at HEAD with this fix stashed → **not** caused by this work. See
> `docs/engineering/cdrom-scsi-send-handoff.md` (banked on uniata convergence).
> The investigation notes below are retained as the historical record of how it was cracked.

# Handoff: crack the i386 cdrom read-path LLVM `ud2` collapse

**Goal:** find the *root cause* (and a clean, non-opt-hack fix) for why the i386 cdrom-rust read
path is `ud2`-collapsed by LLVM at `-O1+` while the identical Rust source compiles fine on amd64.
This is a deferred deep-dive after a prior agent characterized it thoroughly but did not crack the
i386-vs-amd64 divergence. Full prior notes: memory `project_cdrom_i386_llvm_ud2_readpath.md`.

## Symptom
i386 LiveCD boot reaches `cdrom!RequestHandleReadWrite` (`[RHW] enter` prints on debugcon) then
BUGCHECK `0x1E (0xc000001d, …)` = STATUS_ILLEGAL_INSTRUCTION = a bare `ud2`. amd64/arm64 boot fine.

## What it is (confirmed)
At `-O1/-O2/-O3`, LLVM **InstCombine** proves the read path is UB and emits `store i1 true, ptr poison`
→ the whole continuation becomes `unreachable` (a `ud2`). The optimized i386 IR of
`RequestHandleReadWrite` is literally: prologue → `[RHW]` trace → `WdfRequestWdmGetIrp` → `unreachable`.
On amd64 the same source compiles fully and returns normally. **It is not a layout/offset bug** (ty_2
and the WDF_REQUEST_PARAMETERS union are both genuinely align-8 and consistent; `align_of::<i64>()==8`
on i686-pc-windows-gnu; verify_offsets earlier = 0 mismatches). It is an i386-specific
LLVM-UB-exploitation.

## The key UNSOLVED question
**Why i386 and not amd64?** Both have align-8 i64 struct fields accessed through raw `*mut` params
(LLVM assumes align-1 for raw-ptr params on both arches). amd64 doesn't collapse; i386 does. Cracking
this is the whole job — it likely points at one specific InstCombine contradiction (hypothesis: an
align-4 access pins the base pointer to `≡0 mod 4` while an align-8 access on the same base needs
`≡0 mod 8`, and on the i386 layout — usize/ptr=4B vs 8B on amd64 — the offsets line up so LLVM derives
a contradiction → `poison`). Confirm or refute by reading the InstCombine input/output around the first
`poison`.

## Fast repro loop (no QEMU — ~5s per iteration)
```bash
cd sdk/lib/rust
CARGO_TARGET_DIR=/tmp/x RUSTFLAGS="-C target-feature=-sse,-sse2 --emit=llvm-ir" RUSTC_BOOTSTRAP=1 \
  cargo build --release --target i686-pc-windows-gnu -p cdrom-rust --manifest-path Cargo.toml
F=$(grep -l 'define.*@RequestHandleReadWrite' /tmp/x/i686-pc-windows-gnu/release/deps/*.ll)
awk '/define.*@RequestHandleReadWrite\(/{f=1} f{print} f&&/^}/{exit}' $F   # collapsed = ends in `unreachable` right after WdfRequestWdmGetIrp
```
Collapse signal: function body is ~11 lines ending `unreachable`; healthy = reaches
`ScratchBuffer_PerformNextReadWrite` + a normal `ret i32`. amd64 baseline for comparison: drop
`-C target-feature=...`, use `--target x86_64-pc-windows-gnu`.

## Pass-trace technique (find the exact pass + instruction that poisons)
Toolchain clang is too old to parse rustc IR — use **rustc's own LLVM**:
```bash
RUSTFLAGS="-C target-feature=-sse,-sse2 -Cllvm-args=-print-after-all \
  -Cllvm-args=-filter-print-funcs=RequestHandleReadWrite" RUSTC_BOOTSTRAP=1 cargo build … 2>trace.txt
```
Then diff the `PromotePass` dump (clean) → first `InstCombinePass` dump (introduces
`store i1 true, ptr poison`). Prior run located the first poison precisely; re-derive WHY InstCombine
considers that store's pointer poison.

## Bisection result (prior agent)
Collapse is systemic across the whole read+SRB chain. Trigger order when inlined into RHW:
`MmGetMdlVirtualAddress((*irp).MdlAddress)` (first) → `ScratchBuffer_BeginUse` → `RequestCompletion`
→ direct context-struct writes after the 2nd `RequestGetContext`. Each callee is **clean standalone**;
it only collapses the caller when inlined. `ScratchBuffer_PerformNextReadWrite` also collapses standalone
(its own inlined callees).

## Ruled out
- `-C target-feature=-sse,-sse2` (collapses with it removed too)
- `opt-level=1` (still collapses; **opt-level=0 does NOT collapse** — full body, normal ret)
- div-by-zero panic seed (guarding `maxLength` divisor didn't help)
- `__BindgenUnionField::as_ref` `transmute`→`&*` rewrite (no change)
- static struct field-offset / alignment mismatch (types are align-8 and consistent)

## Band-aids that work but DON'T scale (do not ship these — for understanding only)
- `#[inline(never)]` on each inlined cohab shim → moves the frontier forward one fn at a time.
- `core::hint::assert_unchecked((ptr as usize) & 7 == 0)` on each raw-ptr param → same.
- crate-wide `-C opt-level=0` → avoids the collapse entirely BUT is an opt hack (and surfaced latent
  ABI bugs — now fixed, see below). Last resort stopgap only.

## Latent bugs already fixed (prereqs, in tree now)
- `NT_SUCCESS` FFI `-> i32` vs `-> bool` (al-only return, garbage upper eax on i386) — 103 sites in
  ioctl.rs. **(real i386 bug, fixed)**
- `SCSI_REQUEST_BLOCK_SIZE` 0x50 placeholder → `size_of::<SCSI_REQUEST_BLOCK>()`. **(real, fixed)**
- `ScratchBuffer_{BeginUse,EndUse,ExecuteCdb,ExecuteCdbEx}` defined `extern "C"` (cdecl) but declared
  `extern "system"` (stdcall, `@N`) by all callers → i386 ABI mismatch (double stack-cleanup if ever
  called non-inlined; masked at -O2 by inlining). **Fixed**: defs switched to `extern "system"`;
  `nm` confirms `_ScratchBuffer_BeginUse@4` etc. now emitted.

## Known non-blocking artifact
`DeviceEvtIoInCallerContext` is defined **twice** — the Rust translation `cdrom.rs:1485` (extern "system")
and a C copy `drivers/storage/class/cdrom/rust_dbgprint_shim.c:464` (NTAPI). At -O2 the cohab link's
`--allow-multiple-definition` (first-match-wins) resolves it; only `-C opt-level=0` errors on it.
Determine which one is actually live at runtime (link order) — relevant because if the C copy wins, the
Rust read entry point `cdrom.rs` `DeviceEvtIoInCallerContext` is dead code.

## Suggested attack order for codex
1. Re-run the pass-trace; pin the exact InstCombine rewrite that creates the first `ptr poison` and the
   precondition it used (read InstCombine's `-debug-only=instcombine` if a debug LLVM is available, or
   reason from the before/after IR).
2. Build the SAME function for amd64 with `-print-after-all` and diff the InstCombine behavior — the
   first divergence between arches is the answer.
3. From the root, prefer a fix that removes the actual UB in the translation (e.g. the LLM-emitted
   pointer pattern that LLVM is entitled to treat as UB), not an opt-level hack. If the root is a
   genuine miscompile (UB-free Rust mis-handled), minimize a repro and consider a rustc/LLVM issue.
