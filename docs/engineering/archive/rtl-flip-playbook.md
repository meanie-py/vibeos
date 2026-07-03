# RTL C→Rust Flip Playbook (parallel-agent edition)

This is the shared guide for the parallel RTL translation effort. Each
agent owns a **cluster** of `sdk/lib/rtl/*.c` files, translates them to a
**dedicated Rust crate**, gets it compiling on all 3 arches, then wires
it into the build and smokes. Read this whole doc before starting.

The canonical project rules live in `AGENTS.md` (root). The two that
matter most here:
- **Rust is the goal.** Never revert a `.rs` to C, never delete a `.rs`,
  never re-run a paid rustconv translation without the user's OK.
- **Smoke tests are the north star.** All 3 arches must boot to a
  graphical desktop. Never weaken a test to make it pass.

## 0. Why RTL is cohab-safe to parallelize

`librtl` is built from C *plus* a single `librtl_rust.a` staticlib (the
cargo `rtl-rust` umbrella, which `extern crate`s each sibling rlib).
Flipping a `.c` means: write the Rust, `REMOVE_ITEM` that `.c` from
`sdk/lib/rtl/CMakeLists.txt`, and let the Rust `#[no_mangle]` symbol take
over. C and Rust coexist: cross-language calls work because the ABI
matches (NTAPI = `extern "system"`), and shared structs work because the
Rust mirror is **byte-exact** with the C header. So every cluster can be
flipped against *today's* C independently — there is no ordering
dependency between clusters.

## 1. Your worktree

You are in your own git worktree on your own branch (e.g.
`rtl-actctx-port`). Stay in it. Do **not** edit files outside your
cluster's crate + the shared wiring files (section 4).

`.venv` and `.env` (the rustconv API key) live **only in the main
worktree** `/home/curi/Projects/os-playground/vibeos`. Run rustconv from
there (it only reads C + writes a `.rs`, so it's read-only on main and
won't contend):

```bash
cd /home/curi/Projects/os-playground/vibeos
set -a; . ./.env; set +a
.venv/bin/python -m rustconv translate <ABSOLUTE path to your .c> -m opus -o /tmp/<name>.rs
```

The C source is identical across worktrees, so translating main's copy is
fine. `translate` (not `batch`) for files >1000 LOC — batch silently
stubs lazy functions. Copy the draft into your crate; **never lose it**
(paid work).

## 2. The crate template

Mirror `rtl-strings` (committed, the proven cabi+difftest pattern) and
`rtl-heap` (reference: a dedicated crate with a byte-exact type
substrate; read it at
`/home/curi/Projects/os-playground/vibeos-wt-rtl-priv/sdk/lib/rust/rtl-heap/`).

Create `sdk/lib/rust/rtl-<cluster>/`:

`Cargo.toml`:
```toml
[package]
name = "rtl-<cluster>"
version = "0.1.0"
edition = "2021"
[lib]
path = "src/lib.rs"
[features]
cabi = []
kernel = ["cabi"]
[dependencies]
ntkrnl-sys = { path = "../ntkrnl-sys" }
# + cnt-rust = { path = "../cnt-rust", default-features = false } if you
#   need string/mem primitives (see rtl-strings).
```

`src/lib.rs`:
```rust
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
pub mod types;              // byte-exact substrate (if your cluster has private types)
#[cfg(feature = "cabi")]
pub mod <cluster>;          // the translated body
```

`src/types.rs`: byte-exact `#[repr(C)]` mirrors of the private structs
your `.c` uses (the ones bindgen/ntkrnl-sys doesn't have — rustconv flags
them as "gap" identifiers in `resolve`). **Every struct gets a
compile-time size assertion** mirroring the C `C_ASSERT`s (and add your
own for the top-level structs):
```rust
const _: () = assert!(core::mem::size_of::<MY_STRUCT>() == N);
```
If you don't know N, get it from the C: `verify_sizes.py` in
ntkrnl-sys does this, or compile a tiny `sizeof()` probe. Wrong layout =
silent corruption, so the asserts are load-bearing.

`src/<cluster>.rs`: the rustconv draft. Reconcile it to your substrate:
`use crate::types::*;`, fix bindgen-isms (`.__bindgen_anon_1.X` → flat
field), arch-fragile array literals, etc.

## 3. Iterate cargo check (no build dir needed)

From `sdk/lib/rust/` in your worktree:
```bash
# i386 (no-SSE kernel ABI)
RUSTFLAGS="-C target-feature=-sse,-sse2" cargo check -p rtl-<cluster> --features cabi,kernel --target i686-pc-windows-gnu
# amd64
cargo check -p rtl-<cluster> --features cabi,kernel --target x86_64-pc-windows-gnu
# arm64
cargo check -p rtl-<cluster> --features cabi,kernel --target aarch64-pc-windows-gnullvm
```
Add `rtl-<cluster>` to `members` in `sdk/lib/rust/Cargo.toml` so cargo
sees it. This is the only shared-file edit you make *before* it compiles
— it's additive and harmless (nothing deps your crate yet, so the kernel
build ignores it).

Grind the errors to **zero on all 3 arches** before touching the build.
Typical categories: missing constants (add with exact C values — grep the
headers, don't guess), `BOOLEAN` is `u8` not `bool`, NTAPI fns are
`extern "system"`, `ULONG`=u32/`SIZE_T`=usize/`ULONG_PTR`=usize casts.

## 4. Wire into the build (only after cargo-check clean ×3)

Additive edits to shared files — small merge-conflict surface, all
different lines:
1. `sdk/lib/rust/rtl-rust/Cargo.toml`: add
   `rtl-<cluster> = { path = "../rtl-<cluster>", features = ["cabi","kernel"] }`
2. `sdk/lib/rust/rtl-rust/src/lib.rs`: add `extern crate rtl_<cluster>;`
   (REQUIRED — without it the `#[no_mangle]` symbols don't land in
   `librtl_rust.a`).
3. `sdk/lib/rtl/CMakeLists.txt`: `REMOVE_ITEM` your `.c` (comment the
   line `# foo.c — replaced by Rust implementation (rtl-<cluster>)`) AND
   add your crate's source files to the `add_custom_command` DEPENDS list
   (~line 168) so the build re-runs cargo when you edit them.

## 5. Output-pointer safety (captures(none) / ud2)

Any `#[no_mangle]` fn writing through an output pointer MUST use
`core::ptr::write_volatile(out, v)`, never `*out = v` — a plain write
lets LLVM prove `captures(none)` and elide it (silent miscompile, shows
as `ud2`). After building, `python3 tools/scan_ud2.py --check build-i386`
and `python3 tools/lint_kernel.py` catch these.

## 6. Differential test (recommended for leaves / pure logic)

`cnt-difftest` / `rtl-difftest` compile the real C reference (renamed
`ref_<sym>`) and fuzz it against your Rust in one process. For
algorithmic functions add a `diff_<fn>`. For deep pointer-walking code
(heap, actctx) where there's no clean reference harness, the smoke boot
is the test.

## 7. Smoke + land (the gate)

You need build dirs for smoke. Give your worktree one by **copy +
repoint** (never symlink — it poisons main's CMakeCache):
```bash
cp -a /home/curi/Projects/os-playground/vibeos/build-i386 build-i386   # CoW, ~13s
# repoint the cached source path:
grep -rl "Projects/os-playground/vibeos\b" build-i386/CMakeCache.txt | \
  xargs sed -i "s#os-playground/vibeos#os-playground/$(basename $PWD)#g"
cd build-i386 && cmake . >/dev/null && cd ..
```
Then the front door:
```bash
./vibeos-cli build  --arch i386 --clean    # cold = nukes rust target, proves cold-link
# STALE-ARCHIVE TRAP: librtl_rust.a is a thin archive; ar appends without
# pruning and --clean doesn't rm it. Before trusting a flip, rm it:
#   rm build-*/sdk/lib/rtl/librtl_rust.a   (then rebuild)
./vibeos-cli smoke  --arch i386            # refuses stale media; reads summary.json
```
Repeat amd64 + arm64. **All 3 must be green.**

ARM64 caveats:
- `--clean` leaves `ntoskrnl.exe` UNRELINKED → freeldr boot-loop. Fix:
  `cd build-arm64 && rm ntoskrnl/ntoskrnl.exe && ninja ntoskrnl/ntoskrnl.exe && bash qemu/mkramdisk.sh --refresh`, then `smoke --arch arm64 --allow-stale`.
- arm64 "OK" is idle-detection — **always serial-verify** in
  `full.log`: `BUGCHECK` count == 0 AND `_tWinMain entered` >= 1.

Land: commit (stage by explicit path, never `git add -A`; never
`git checkout`/`restore` to discard — use stash). Then reconcile + FF
onto `continue-rust-port`:
```bash
git merge continue-rust-port --no-edit   # crp churns; pull it in
# re-smoke ×3 if crp brought boot-path changes, then FF crp:
git -C /home/curi/Projects/os-playground/vibeos merge --ff-only <your HEAD>
```
Commit message ends with:
`Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`

## 8. i386 no-SSE float wall (only if your cluster touches floats)

rustc HARD-ERRORS any fn that *returns* `f32`/`f64` by value under
`-C target-feature=-sse,-sse2` (the i386 kernel ABI). Float *args* are
fine. If you hit it: make the fn `extern "C"` (returns via x87 ST0) and
use `core::mem::transmute` for bit→float (not `f64::from_bits`). Most RTL
clusters have no float — ignore this unless cargo check complains.

## Cluster ownership (current)

| Cluster | Files | Owner |
|---|---|---|
| heap | heap/heappage/heapdbg/heapuser | (coordinator) |
| actctx | actctx.c | Agent A |
| threadpool | threadpool/workitem/timerqueue | Agent B (first) |
| sync | critical/srw/condvar/slist/wait/runonce | Agent B (then) |
| leaves | error/registry/locale/debug/image/rangelist/rxact/sysvol/memstream/dbgbuffer/… | TBD |

Note for **sync**: `rtl/slist.c` (RtlInterlockedPushEntrySList etc.) may
already be covered by the existing `slist` crate — check before
translating (it could be a no-op REMOVE_ITEM or already provided). The
sync public types (RTL_CRITICAL_SECTION / RTL_SRWLOCK /
RTL_CONDITION_VARIABLE) ARE in ntkrnl-sys/bindgen, so sync needs less
substrate work than heap/actctx.
