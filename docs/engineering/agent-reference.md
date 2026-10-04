# Agent Reference — commands, layout, and platform specifics

**This is a PULL doc.** It was split out of `AGENTS.md` on 2026-07-24 because that file is
the PUSH channel (loaded into every agent session, for every agent) and its own Knowledge
Routing rule says to keep it tiny. Nothing here was rewritten — the sections below are the
verbatim reference material that used to live in `AGENTS.md`, so anything that linked to
them still reads true. The behavioural rules stayed behind in `AGENTS.md`.

Read this when you need a command, a path, or a platform detail. Related deeper docs:
`build-system.md`, `rust-in-kernel.md`, `rustconv-pipeline.md`, `kernel-debugging-infra.md`,
`tests/qemu/cli/README.md`, `qemu/README.md`.

---

## Directory Layout

```
vibeos/                          # Git repo root (renamed from reactos/)
├── build-arm64/                 # CMake/Ninja build output (ARM64)
├── build-amd64/                 # CMake/Ninja build output (amd64)
├── build-i386/                  # CMake/Ninja build output (i386)
├── .build-tools/
│   └── llvm-mingw/              # Cross-compiler toolchain
├── qemu/                        # QEMU development environment
│   ├── run.sh                   # Launch QEMU (--arch arm64|amd64|i386, --gui, --debug)
│   ├── sym.sh                   # Symbolicate crash addresses
│   ├── mkramdisk.sh             # Create ARM64 ramdisk + boot images
│   ├── boot.img                 # FAT32 EFI boot disk (ARM64, gitignored)
│   ├── ramdisk.img              # FAT32 ramdisk with ReactOS files (ARM64, gitignored)
│   ├── QEMU_EFI.fd              # ARM64 UEFI firmware (EDK2, gitignored)
│   └── mouse_click.py           # Host-side mouse injection script
├── docs/
│   ├── product/                 # Roadmaps, status updates
│   └── engineering/             # Debug briefs, technical plans
├── ntoskrnl/                    # Kernel source
├── sdk/                         # SDK + Rust crates
├── tools/rustconv/              # Translation pipeline
└── tests/qemu/                  # Smoke test framework
```

## Landing your work — the merge queue

`./vibeos-cli land` **submits** this lane's tip to the fleet's merge queue and returns in
~2s. You do not merge `master`, do not run the gate, and do not wait. Whichever agent finds
the land lock free becomes the **conductor**: it merges every queued lane onto current
`origin/master` in a dedicated integration worktree, runs the full 3-arch gate **once**, and
fast-forwards `master` to exactly the commit it gated. N ready lanes therefore cost ONE gate.

```bash
./vibeos-cli land                  # submit (and train the batch if nobody else is)
./vibeos-cli land-status --mine    # conductor + queue + MY verdict (landed / rejected + artifact)
./vibeos-cli land --withdraw       # take MY queued submission out of line (not ready / would gate cold)
./vibeos-cli land --direct         # opt out: gate and push this lane alone
./vibeos-cli land --drain          # train without submitting (orphan recovery)
```

A merge conflict is **bounced back to you** with the conflicting paths — the conductor never
resolves it. A red batch **splits recursively**, so innocent lanes still land and the culprit
is rejected with its gate classification plus its artifact dir. Full design:
`tests/qemu/cli/README.md` → `land`, and the header of `tests/qemu/cli/queue.py`.

### Which local gate to run BEFORE submitting — and never DURING

`land` gates for you, so these are about not wasting a 25-minute queue wait on something you
could have caught in seconds. Every line below is a batch that actually bounced.

- **`./vibeos-cli check --arch i386`** when the lane adds translated `.rs`. An UNBUILT crate is
  covered only by `verify_c_signatures`, which has caught real signature bugs in crates no
  compiler had ever read (wldap32's `ldap_value_freeA/W` lost a `*`; netcfgx's `MoveItem` had
  `INT_PTR` for `INT`). A signature fix usually needs the BODY's casts fixed with it.
- **`./vibeos-cli check-tooling`** when the lane touches `tools/*.py`. `ruff check` plus a `-k`
  pytest subset is NOT this gate: `ruff format --check` is a separate check inside it, and one
  unformatted dict literal bounced a batch as TOOLING_FAIL after the full queue wait.
- **`./vibeos-cli check --strict --scope source-deferred`** when the lane adds a file walk,
  or any Python the source scanners read. The gate runs that partition in STRICT mode and
  plain `check` does not, so `check` goes green on exactly what the conductor rejects as
  LINT_FAIL. It takes ten seconds and reproduces the verdict. (`scan_unpruned_walk` bounced a
  batch for a bare `rglob("*.rs")` over a crate — not a style point: a crate's `target/` holds
  generated copies of its own modules, so an unpruned walk answers questions from build
  artifacts. Use `tools/source_walk.py source_files()`.)
- **`./vibeos-cli difftest`** when the lane wires a new `.rs` into `msvcrt-crt-rust` /
  `cnt-rust` / `rtl-rust`. The host harness links the subject crate beside the reference C, so a
  new `#[no_mangle]` can collide with a test's own fake stub or the reference TU's data globals
  — file.c's flip bounced DIFFTEST_FAIL on ten duplicate symbols. `check` cannot see that,
  because nothing links there.

**BEFORE, never DURING.** Running any of these — or a build — while your own lane sits in the
queue starves its smoke: one net test timed out at ~186s against ~44s passing and bounced the
batch NET_FLAKE, and the gate's three retries all fire inside the same window, so they cannot
rescue it. `./vibeos-cli host-status`, submit, then leave the host alone. Backgrounding `land`
and editing while it runs fails `receipt-validation` with "Tooling inputs changed during
checks" — which reads exactly like an intermittent flake and is not one.

**`receipt-validation` failing with every substantive check GREEN is infrastructure, not a
defect in your change** — rerun it. What actually invalidates it is narrower than it looks,
and an earlier version of this entry got it wrong in a way worth keeping as the example.

That version claimed activity in a SIBLING worktree could fail the conductor's receipts,
on the strength of tip `51e72900c8c9`: REJECTED TOOLING_FAIL with all 56 checks green while
its author ran `check-tooling` and `git merge` in a sibling checkout, then the
byte-identical tip landing as `f5d93d6c2174` on resubmission. Same commit, both verdicts —
and the author wrote "Measured" beside it. **It was not measured, it was a correlation with
an uncontrolled confound:** the resubmission also re-gated against a different master with
different co-queued lanes, so "the author sat still" was never the isolated variable. The
direct measurement, done afterwards, refutes it — `source_identity` and `runtime_identity`
of a live integration worktree, sampled before and after mutating a sibling checkout, come
back byte-identical. `_file()` is a pure content sha256 with no mtime, and `source_identity`
walks only the gating repo's own `git ls-files`. Sibling activity cannot reach it. Note the
tell that should have stopped the sentence: the author's own text said the code "does not
look like it should" behave that way, which is a reason to go measure, not a flourish to
put in front of a claim.

**The one genuine cross-worktree surface is `runtime_identity`,** which hashes the SHARED
toolchain — `.venv/bin/ruff`, `pyright`, and the rustup-selected rustc/cargo plus sysroot
`.so` files. So a `rustup component add`, a venv upgrade or a `pip install` mid-gate really
does reach the conductor and will invalidate its receipts; a merge, an edit or a
`check-tooling` in another worktree will not. Save toolchain changes for a quiet queue.

**A COMPLETION STAMP PROVES A BUILD FINISHED, NOT THAT IT BUILT EVERYTHING.** A `build-<arch>`
that predates a new entry in a model the tooling checks is stamped and still lacks the object
directory, so `check-tooling` refuses a change that has nothing to do with it — including a
docs-only one. Measured 2026-09-20: `c474d932eb7` added `wine` to
`verify_ucrtbase_cohab`'s `OURS_ON_LINK_LINE`, and
`test_every_modelled_target_has_a_real_object_directory` then bounced this lane's land
**three times in a row** — i386, then amd64, then arm64 — because the assertion reported only
the FIRST missing target, so each failure read as one stale tree until you fixed it and hit the
next. The remedy is `./vibeos-cli build --arch <a>` for each, and the assertion now names them
all at once. Note what the stamp does and does not assert before concluding the model is wrong:
if the named target was recently added, the tree is stale, not the check.

**A host-only stage that reads build output is not host-only.** Same incident, the general
form: `check-tooling` is documented as arch-independent, and `_lint_preflight` moves it earlier
on the stated grounds that it needs no `build-<arch>`. That test reads three of them. The
aggregate message removes the three-cycle cost, not the category — a stage advertised as
needing no build can still fail for reasons unrelated to the change. Worth deciding properly
rather than patching per-incident.

**A green local gate is WARM unless you pass `--clean`, so it is not evidence the tree builds
cold.** A warm gate once reported OK on a tree whose i386 build failed cold, minted a push
stamp, and the queue rejected the lane. When a lane touches a shared `-sys` crate, gate with
`--clean` or skip the local gate entirely and let the conductor's fresh worktree be the oracle.

**Read the summary at the TOP of a run, not the tail**, and never `| tail` a `vibeos-cli`
command whose failure you need to understand — that hides the `FAIL — N/M checks failed` line
and swallows the exit code. Environmental noise in one check does not license skipping the rest.

**A build races anything that writes `.rs`.** A translation running concurrently makes sources
newer than the cargo-linked binaries, and the staleness check then reports BUILD_INCOMPLETE.

## Discarding work safely — the stash, and why index numbers lie

AGENTS.md forbids `git checkout`/`restore` for discarding. Stash instead. The trap is that the
stash stack is **shared by every worktree**, so `stash@{0}` is whatever was pushed most
recently anywhere — a bare `git stash pop` has applied a sibling lane's 1124 files into the
wrong tree. Two consequences:

**Pushing.** `git stash push <pathspec>` **fails outright if any listed path is untracked**, and
paired with `|| true` it creates no entry while looking like it worked. Pass `-u` whenever the
pathspec includes new files, then confirm the entry landed:

```bash
git stash push -u -m "mylane: <what and why>" -- <paths>
git stash list | head -3          # your message must be at the top
```

**Dropping.** `git stash drop <sha>` does not work: drop and apply need a `stash@{n}` ref, so
"drop by hash" is not executable advice. Resolve the index from the message, re-read the message
at that ref, and refuse if it no longer matches — in one command, so nobody can push between the
lookup and the drop:

```bash
IDX=$(git stash list --format='%gd|%gs' | grep -F "<my msg>" | head -1 | cut -d'|' -f1)
MSG=$(git stash list --format='%gd|%gs' | grep -F "$IDX|" | cut -d'|' -f2-)
case "$MSG" in *"<my msg>"*) git stash drop "$IDX" ;; *) echo REFUSED ;; esac
```

**Recovery, if you applied someone else's.** Their work is safe: a conflicted `pop` does not drop
the entry. Back up your own uncommitted files, confirm your commits are pushed
(`git rev-list --left-right --count origin/master...HEAD` shows `0` ahead), then
`git reset --hard HEAD` plus `git clean -fd <your dirs>`, and restore your backups.

## Build Commands

**Builds run at the CANONICAL PATH** (`/tmp/vibeos` — a per-build bind mount, see
`tools/canon-run.sh` and `docs/engineering/build-system.md`): build
dirs bake `/tmp/vibeos/...` paths so caches share across worktrees. `./vibeos-cli build`
handles this automatically (+ mandatory ccache/sccache, provisioning, lints). For raw
ninja iteration, prefix with the wrapper — **plain `ninja` in a build dir fails loudly**
(its baked paths only exist inside the mount):

```bash
# ARM64
cd build-arm64
bash ../tools/canon-run.sh ninja            # Build everything
bash ../tools/canon-run.sh ninja ntoskrnl   # Just the kernel

# amd64 / i386 (same pattern)
cd build-amd64
bash ../tools/canon-run.sh ninja livecd     # Live CD ISO (livecd.iso)
bash ../tools/canon-run.sh ninja bootcd     # Boot CD ISO (bootcd.iso)

# Common patterns
bash ../tools/canon-run.sh ninja -k0        # Continue past errors
bash ../tools/canon-run.sh ninja <target>   # Any target — list: ... ninja -t targets all | grep <name>
```

After modifying a `CMakeLists.txt` or `.cmake` file, re-run cmake before building:
```bash
bash ../tools/canon-run.sh cmake .          # Re-configure (from any build-*/ dir)
```

### Long work gets DETACHED; the watcher is expendable

Agent background tasks on this box are killed, often within minutes, with
`system is running low on memory` — observed repeatedly at **40+ GiB MemAvailable, over
100 GiB free disk, and no OOM kill in `dmesg`**. (This is unrelated to the QEMU lifetime
reaper, `VIBEOS_QEMU_MAX_LIFETIME`; don't conflate them.) **Don't theorise about the
trigger** — "it only kills tasks that produce output" looked decisive after one 36-minute
survivor and was falsified by the next identical watcher dying at 12 minutes. What holds up
across many samples is only which *shapes* survive:

| shape | outcome |
|---|---|
| `setsid nohup bash -c '…' < /dev/null > log 2>&1 &` | survives |
| a foreground Bash call the harness auto-backgrounds at its timeout | survives |
| a harness `run_in_background` watcher | killed routinely |

So **put the WORK in a detached process that writes to a log file, and let any watcher be
expendable.** A killed watcher then costs a re-poll; a killed build costs 25 minutes.

```bash
setsid nohup bash -c './vibeos-cli build --arch arm64 > /tmp/.../build.log 2>&1; \
    echo "rc=$?" >> /tmp/.../build.log' < /dev/null > /dev/null 2>&1 &
```

Write a terminal marker (`rc=$?`) into the log, because `vibeos-cli` buffers its summary
until the end: an empty log means "still running", not "hung". Judge the result by the
artifact and that marker, never by a watcher's exit code — and when a watcher dies, check
`ps` before assuming the work died with it.

## Toolchain

- **Compiler:** llvm-mingw (clang 19, lld 19.1.7)
- **Unified toolchain file:** `toolchain-llvm-mingw.cmake` — builds all 3 architectures
- **Targets:** `aarch64-w64-mingw32`, `x86_64-w64-mingw32`, `i686-w64-mingw32`
- **External dependency:** `windmc` from Homebrew mingw-w64 (message compiler, not in llvm-mingw)
- **16-bit assembly:** Homebrew `i686-w64-mingw32-gcc` for boot sectors/frldr16/MVDM (Clang can't handle `.code16`)
- **Output format:** PE-COFF (Windows executables/DLLs), not ELF

### Rust is on PINNED STABLE — write stable Rust (2026-07-12)

**The Rust tree builds on stable rustc** (pinned in `rust-toolchain.toml`). Your new code
must compile on stable. **Do NOT add `#![feature(...)]`, and do NOT add `RUSTC_BOOTSTRAP=1`
to a CMakeLists** — that env var is the stable-rustc escape hatch that silently unlocks
nightly, and it is how nightly crept back in the same day the lane removed it. Both are
now hard-gated by `scan_nightly_surface` (`vibeos-cli check`), so you'll get a loud failure
rather than a surprise.

The **one** sanctioned nightly user is the **i386 kernel** `*-cargo` cell: it needs
`-Z build-std` + `i686-kernel.json` for `rustc-abi: x86-softfloat` (no builtin triple
provides a no-SSE `core`), pinned in `.rust-nightly`. It *retires* with the wow64 lane.
amd64/arm64 build the **builtin triples** with rustup's precompiled std.

If you think you need a nightly feature, you almost certainly don't:
- **`c_variadic`** — needed only to *DEFINE* a C-variadic fn. *Declaring* and *calling* one
  is fine on stable. No-op debug shims (`TRACE(fmt, ...)`) should be a `macro_rules!` (see
  `ole32-rust/src/debug_shims.rs`); to genuinely define one, use the C `va_start` trampoline
  in `sdk/lib/rust/varargs-rust/`.
- **`linkage`** — perma-unstable upstream; it will never stabilize. Restructure.

Two related traps, both real bugs we ate (field manual §9.65):
- **Never translate a C debug MACRO into a no-op `fn`.** An out-of-line no-op is an
  optimization barrier that uncloaks latent ABI mismatches → `ud2` → BUGCHECK. Use a macro.
- **Never hand-write `memcpy`/`memset`/`memmove`/`memcmp` in Rust.** LLVM folds any byte-copy
  loop back into a call to the function itself → infinite self-recursion → silent hang.
  ntoskrnl exports them; just import them.

Full reference: `docs/engineering/build-system.md` §"Cargo target model".

## Searching the tree — `grep` is ugrep, and it SKIPS 175 sources in silence

`grep` in this environment is a shell function wrapping **ugrep**. On a file that is not
valid UTF-8 it finds nothing and says nothing — no "Binary file matches", no warning, exit 1,
which is indistinguishable from a genuine no-match.

**175 of the 16,466 tracked C/C++ sources (1.1%) are ISO-8859, not UTF-8** — usually one
accented byte in an upstream copyright line. `dll/win32/shell32/CExtractIcon.cpp` is one
(`0xe9` at offset 234), along with `shellrecyclebin/guid.c` and `recyclebin_v5.h`.

```bash
grep -c DuplicateString dll/win32/shell32/CExtractIcon.cpp      # nothing, rc=1  ← LIE
grep -ac DuplicateString dll/win32/shell32/CExtractIcon.cpp     # 5  ← the truth
```

**Pass `-a` whenever you grep source, and treat an empty result on a file you have not
`-a`-checked as unproven.** (`--encoding=LATIN1` works too; `-a` is shorter and is the same
flag the serial logs need, since those carry NULs.) To enumerate the blind spot itself, read
the bytes in Python — `Path(f).read_bytes().decode('utf-8')` and catch `UnicodeDecodeError`.

This is the field manual's "a claim ABOUT a measurement needs a route the suspect tool does
not mediate", in its cheapest form. It cost a near-miss: an empty grep was about to be
reported as "`DuplicateString` exists nowhere in shell32 — rustconv invented it", when the
function is at `CExtractIcon.cpp:72` and rustconv had parsed it correctly. Two sweeps in that
same session were re-derived in Python afterwards; both conclusions survived (0 extra hits
for `#ifdef _M_ARM64` and for bare self-`QueryInterface`/`AddRef`/`Release`), and one tree-wide
count moved: `_CreatorClass::CreateInstance` is 5, not 4, the fifth in
`dll/win32/browseui/shellbars/CBandSite.cpp`.

**The repo's own scanners are NOT affected** — they read with
`read_text(encoding="utf-8", errors="replace")`, so the gate sees these files. This is an
interactive-search hazard only, which is exactly why it is written down here rather than
fixed in a tool.

## Key Technical Patterns

### SEH / DUMMY_PSEH (status 2026-07-10 — the ruling doc is `docs/engineering/dummy-pseh-verdict.md`)
`USE_DUMMY_PSEH` is still the default compile mode for **C** on all arches (PSEH3 is incompatible with Clang — ARM64 lacks x86 SEH; Clang's ESP-relative locals break i386 PSEH filters), so `_SEH2_TRY`-protected C must still validate pointers before dereferencing. **But the old "SEH is dead code everywhere, swallow software exceptions" era is OVER** (2026-07):
- **Rust frames have real unwind tables** (`requires-uwtable`): the kernel ships `.pdata`/`.xdata` for every Rust function and the unwinder works (`rtl-except`; field manual §9.35).
- **The exception dispatcher is live infrastructure**: hardware faults and software raises dispatch to user-mode SEH on all three arches (arm64 unwinder fixed 2026-07-10; user-SEH self-test in the gate). Do NOT re-add "swallow it" hacks.
- **Kernel-mode catch is possible** (validated PoC `ntoskrnl/seh_poc.c`); native SEH is a sanctioned per-module tool for the ~22 remaining C files with `_SEH2_TRY`.
- **Rust replaces PSEH semantics idiomatically**, not mechanically: validate-first probes returning `Result` (rustconv Stage 2), `try_*` contract conversions at raise sites, RAII `Drop` guards for `__finally`. See the verdict doc before writing any new SEH-shaped code.

### ARM64 PE-COFF Relocations
Use `adrp + add :lo12: + ldr` (not `adrp + ldr :lo12:`) in assembly. The latter generates `LDST64_ABS_LO12_NC` relocations requiring 8-byte alignment which PE-COFF sections may not provide.

### LLD i386 Forwarder Decoration Bug
LLD prepends `_` to PE export forwarder targets on i386. Workaround in FreeLDR (peloader.c) and ntoskrnl (sysldr.c): strip leading `_` from forwarder DLL name.

### GAS Intel Syntax on i386
In GAS `.intel_syntax noprefix`, `mov eax, label` does a **memory load** (reads bytes at label), NOT an address load. Must use `lea eax, [label]` or `mov eax, offset label`. Also `push offset label` fails to assemble under clang's integrated assembler ("unsupported relocation type") — materialize via `lea eax, [label]` + `push eax` (field manual §9.101).

### Build System Architecture Conditionals
Architecture checks use: `if(ARCH STREQUAL "arm64")` / `"amd64"` / `"i386"`. ARM64 uses UEFI-only boot. The `USE_DUMMY_PSEH` flag is set for all architectures when using llvm-mingw.

### llvm-rc Limitations
llvm-rc is stricter than MSVC's rc.exe:
- ACCELERATORS: Cannot combine `ASCII` type with `ALT`/`SHIFT`/`CONTROL` modifiers — use `VIRTKEY` instead
- DIALOGEX coordinates must fit signed 16-bit (-32768 to 32767)
- FILEVERSION/PRODUCTVERSION fields must fit 16 bits (max 65535)

## QEMU Development Workflow

```bash
# ARM64 — UEFI ramdisk boot
bash qemu/run.sh --arch arm64                  # Serial console
bash qemu/run.sh --arch arm64 --gui            # GUI with virtio input

# amd64 — LiveCD ISO boot
bash qemu/run.sh --arch amd64                  # Serial console

# i386 — LiveCD ISO boot
bash qemu/run.sh --arch i386                   # Serial console

# Common options
bash qemu/run.sh --arch arm64 --debug          # Pause for GDB on :1234
bash qemu/run.sh --arch arm64 --log            # Enable QEMU tracing
```

### ARM64 Ramdisk Update
```bash
mcopy -o -i qemu/ramdisk.img build-arm64/path/to/file.exe ::/reactos/system32/file.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img
# WARNING: Do NOT mcopy uefildr.efi into boot.img AFTER ramdisk — corrupts FAT32
```

### Symbolication
```bash
bash qemu/sym.sh 0xC804 0x50FC    # Look up offsets from UEFI bootloader (uefildr) crash dumps
# Formula: runtime_addr - module_base + PE_ImageBase = PE VA
```

(For ntoskrnl symbol lookups use the kernel debug toolkit's `addr2sym.sh`/`sym2addr.sh` — see next section.)

## Kernel Debugging Infrastructure (qemu/start.sh, inspect.sh, run-until.sh)

**For triaging kernel bugs (BUGCHECKs, hangs, regressions), prefer this toolkit over `qemu/run.sh` + ad-hoc DbgPrint instrumentation.** It is agent-driven: every operation is a single batch command in, structured result out, no interactive GDB sessions to babysit. The scripts are `start.sh` / `stop.sh` / `wait-for.sh` / `tail.sh` / `status.sh` / `inspect.sh` (bt, symbolicate-pc, dump-thread, dump-trapframe, pool-tag, identify-object, triage-bugcheck) / `run-until.sh` / `ps` / `build-index.sh` / `sym2addr.sh` / `addr2sym.sh` / `preflight.sh`.

**Standard recipe (validated end-to-end):**
```
RUN=$(qemu/start.sh amd64 --gdb)
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
```
On the amd64 KiUnwaitThread BUGCHECK flake, this returns the stop code, exception code, faulting PC + symbol, full KTRAP_FRAME, and `identify-object` on the bogus pointer — all in one tool call.

**Since the MP-default promotion, `build-arm64/ntoskrnl/*.exe` is the WRONG
kernel.** The media boots `/KERNEL=ntkrnlmp.exe` (`livecd.ini`), and the MP image
lives one directory deeper at `build-<arch>/ntoskrnl/ntkrnlmp/ntkrnlmp.exe`. A
glob or tab-complete in the obvious directory hands you the UP kernel, and it
*answers* rather than erroring — so hand symbolication (`llvm-symbolizer`,
`addr2line`, `add-symbol-file`) silently returns a plausible wrong symbol. A real
case: `ntoskrnl.exe+0x3BBA4` resolved to `KeSetIdealProcessorThread+0x1c` against
the UP image and `RtlRaiseStatus+0x44` against the MP one. Only the second fits a
`BUGCHECK 0x1E` with `P1 = STATUS_FILE_INVALID`; had the fault been anywhere near
scheduling, the wrong answer would have been believed. Check which image you fed
it before trusting any symbol from this period.

**If a breakpoint never fires on a symbol you know is hot, check which image
`find-kbase` chose.** Every media now carries BOTH kernels and boots MP by
default, while `VIBEOS_KERNEL` still names the UP image — so the scan could match
`ntoskrnl` while `ntkrnlmp` is executing, relocate every symbol to an image the
guest never runs, and leave breakpoints silently dead. `find-kbase` now prefers
the image containing `$pc`, then the MP image, and says so on stderr when it
overrides. Read its `[find-kbase] matched ... at 0x...` line against the guest's
own `[KRNLBASE]` serial line if anything looks wrong.

Full toolkit reference + design: `docs/engineering/kernel-debugging-infra.md` and `qemu/README.md`. (`qemu/ps` shows every live QEMU across all worktrees + slot usage; QEMUs are capped via `VIBEOS_QEMU_MAX_CONCURRENT`/`VIBEOS_QEMU_MAX_LIFETIME`; **same-arch i386/amd64 boots run in parallel on the default `windows-host` TCG backend** — the per-arch sub-cap is bypassed there (`vm.py:1295-1299`) and only serializes the accelerator-sharing backends (native+KVM, windows-host+WHPX); see the North Star note + `qemu/README.md` → Resource management); run `qemu/build-index.sh <arch>` after each kernel build for offline sym lookup.)

### Record/replay — when the bug is too rare to catch live

`./vibeos-cli rr record --arch arm64` boots once under QEMU's instruction-counted
record mode and keeps a self-contained trace under `~/vibeos-rr/`;
`./vibeos-cli rr replay latest` then re-runs that *exact* boot,
instruction-for-instruction. A 1-in-60 crash you have recorded reproduces every
time. **Never stop a replay from the debugger** — stepping, attaching mid-run
and even a breakpoint hit all abort it. Stop it with QMP `replay-break` at an
icount and then attach gdb to the already-paused guest; that is verified
working. See the doc before planning a debugging session.

It is single-vCPU, TCG-only, no-NIC, native-backend-only, and it *refuses*
rather than producing a trace that silently diverges. `--smp 1` is accepted and
is the interesting case (the MP kernel on one CPU). Full rules, gdb recipe and
the divergence log: `docs/engineering/qemu-record-replay.md`.

## Test Suite

### Rust Host Tests (no QEMU needed)

```bash
cd sdk/lib/rust

# Unit tests — algorithm correctness, pack/unpack roundtrips
cargo test -p slist

# Kani model checking — exhaustive verification of state transitions
cargo kani -p slist

# Loom concurrency tests — explores all thread interleavings
cargo test -p slist-loom
```

### Differential grading against the genuine C — `rustconv difftest-contract`

For logic-heavy code (parsers, string/codec/crypto routines, anything smoke never calls),
grade the SHIPPING Rust against the GENUINE C on the host: a reviewed JSON contract
generates typed wrappers and a corpus, runs both sides twice in isolated processes (C
under UBSan), and requires every graded function to have a caught sensitivity mutant.
No API calls; ~5 s per contract. It has found shipping bugs on 4 of its first 10 new
targets (compress, cryptlib sha1, version.dll ×3; see `git log --grep difftest-contract`).

```bash
PYTHONPATH=tools/rustconv .venv/bin/python -m rustconv difftest-contract \
    tools/rustconv/data/difftest/version.json --out ~/vibeos-rust-staging/<fresh-dir>
```

`report.json` is the verdict (`pass` / `mismatch` + `counterexample.json` / `invalid`).
Whole Wine/ReactOS TUs compile against the tree's own headers with
`"c_host_profile": "reactos-amd64"` (needs this worktree's `build-amd64`); what the C calls
out to goes in a reviewed `c_env` shared by both sides, and every other external becomes a
named trap. Contract format, examples and limits: `tools/rustconv/data/difftest/README.md`.
It complements the hand-written `*-difftest` crates (`./vibeos-cli difftest`, which takes
`--only <crate> --case <name>` for one exact case in crates honouring `--exact`).

Each bug class it finds becomes a gated `check` scanner over every body. The first four, plus
`scan_bool_width` and `scan_cfg_vs_c` (all with EMPTY baselines, all run by `./vibeos-cli check`):

- `scan_ptr_byte_diff`: `a as isize - b as isize` on WCHAR/ULONG pointers where the C's
  `a - b` counted ELEMENTS. The fix is `a.offset_from(b)`.
- `scan_misaligned_cast`: a `[u8; N]` cast to a wider pointer and dereferenced. The fix is
  `read_unaligned` / `write_unaligned`. `--pointers` also reports byte-POINTER bases, as a
  246-site review queue.
- `scan_dropped_null_guard`: the C null-tests an out-param, and the Rust stores through
  it unconditionally.
- `scan_store_on_failure`: the C writes an out-param only after its last failure `return`
  (compress.c's `out:` label), and the Rust writes it and then returns a status that can
  fail. A store dominated by a success test, and a `*pp = NULL` (COM's rule), stay quiet.
  Its oracle holds on ~670 of the tree's ~8,100 pointer-param stores, so a clean run means
  something.
- `scan_bool_width` (older, gated the same day): a u8/BOOLEAN return where the other side
  reads a BOOL — only AL is defined, so `!= 0` reads garbage (broke fastfat and mountmgr).
  Two directions: a Rust extern DECL wider than the Rust definition, and an exported Rust
  DEFINITION narrower than its C twin (every C caller) or than its own per-arch cfg twin.
  Its 202 findings were 25 wrong definitions, not 202 wrong decls; 56 in all.
- `scan_cfg_vs_c`: a per-arch `#[cfg]` inside a function whose arch set the C twin's own
  `#if`s never draw (arch macros, `_WIN64`, `_MI_PAGING_LEVELS` all evaluate to arch sets).
  `_WIN64` spelled `target_arch = "x86_64"` hands arm64 the 32-bit path; the fix is
  `target_pointer_width = "64"`. C `CONFIG_SMP` is Rust `feature = "smp"`; that line too is
  judged only where the C draws it. Where the C never names arm64, only the x86/x86_64 line
  is judged (upstream's `#else` lumps arm64 with i386). `--stats` shows how the in-body
  population splits: ~440 agree with the C mechanically, ~650 have no C arch `#if` to
  compare (Rust-only cfgs — those still need reading). Per-function, not per-block, so a
  Rust-only block in a function with an unrelated C `#if` is flagged: read, then `nolint`.

`scan_duplicate_symbols` (also EMPTY-baselined) is the link-level one. Every PE links with
`--allow-multiple-definition`, so two definitions of one symbol resolve by archive order and
nothing warns. It models what each binary links (`tools/rust_link_model.py`: the crates a
`*-cargo/build.rs` names, their path-dep closure, Cargo feature resolution, `if arch == …`
blocks, every definition's full cfg, and `global_asm!` `.globl` labels) and reports a name
defined twice on one arch. i386 decoration counts: cdecl `_X`, stdcall `_X@N` and fastcall
`@X@N` are different symbols. C objects are out of its scope. To check a finding against the
bytes, `llvm-nm --defined-only` the archives under
`sdk/lib/rust/target/<binary>-<crate>-rustlib/<triple>/release/`.

The first, third and fourth read the C twin by function NAME through `tools/c_twin.py`. Its index
of which file defines which function is cached in `tools/.cache/c_twin_defs.json`: a 12 s
cold build, then only changed files are re-read. Each scanner honours
`// nolint: <tag>` on a reviewed line, with the reason written beside it.

### QEMU Kernel Tests (28 tests via serial dispatch loop)

```bash
# 1. Build test kernel (canonical wrapper — see Build Commands)
cd build-arm64 && bash ../tools/canon-run.sh sh -c 'cmake . -DREACTOS_TEST_KERNEL=ON && ninja ntoskrnl'
cd ..

# 2. Update ramdisk
mcopy -o -i qemu/ramdisk.img build-arm64/ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img

# 3. Run tests (boots QEMU, sends commands over serial, checks [TEST PASS])
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -v       # All 28 tests
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -k SList # Just SList
```

### QEMU Smoke Tests (boot verification — see "Smoke Tests — The North Star" above)

```bash
source .venv/bin/activate

# Full test (clean Rust rebuild + boot + static analysis) — ~5 min per arch
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 600
pytest tests/qemu/test_smoke.py -v --arch amd64 --timeout 600

# Fast iteration (skip nuclear rebuild) — ~30s per arch
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build

# Static analysis only (no QEMU boot)
pytest tests/qemu/test_smoke.py -v --arch i386 -k "sacred_primitives or no_unexpected_ud2" --skip-clean-build
```

**IMPORTANT:** Cargo's incremental cache can mask regressions. Always run WITHOUT `--skip-clean-build` for final verification before committing. The clean build nukes `rust-kernel-target/` to ensure deterministic results.

Smoke/CLI harness internals (`vm.py`/`conftest.py`/`serial_client.py`/`qmp_client.py`/`debugcon.py`, boot media, QEMU backends + acceleration (windows-host TCG default / native KVM / WHPX opt-in), resource caps, debugcon boot tracing, test lifecycle): `tests/qemu/cli/README.md`.

## Kernel/HAL bindgen staging

`bash sdk/lib/rust/ntkrnl-sys/generate_bindings.sh` stages all three kernel
islands; `generate_hal_bindings.sh` in that directory stages amd64/i386 HAL
islands. Both default to a private `.build-tools/bindgen-runs/` directory and
accept `--output-dir NEW_DIRECTORY` (parent must exist). Unlike their old
behavior, neither command overwrites integrated bindings. Use this worktree's
configured build headers; missing or symlinked generated inputs are refused.

Each run retains `raw/` (raw Rust and header depfiles), `src/` (postprocessed and
ABI-normalized Rust), and a `COMPLETE` marker written only on success. Review
staged output before any integration; generation alone is not a compile/boot
verdict. Failed output remains available for diagnosis without a success marker.

## Rust in the Kernel

The Cargo workspace at `sdk/lib/rust/` holds Rust components that compile to PE-COFF static libraries (rlibs/staticlibs) and link into the cmake build. **All crates build for all 3 architectures unless explicitly noted.** The workspace long ago outgrew "kernel algorithms": it spans the bootloader (`freeldr-rust`), the HAL (`hal-rust`, all three arches), win32k (`win32k-rust`), user-mode components (`winlogon-rust`, `explorer`-side crates, server DLLs), and a growing driver fleet (`uniata`, the `usbport`/`usbhub`/HID stack, netstack, `disk`/`classpnp`, …).

```
sdk/lib/rust/
├── Cargo.toml              # Workspace (resolver v2). lto=false + codegen-units=256 firewall.
├── ntoskrnl-rust/          # The (shrinking) kernel-monolith crate — ~130K LOC (was ~456K).
│                           #   Being split per-subsystem (crate-split, ~70% done): what REMAINS
│                           #   here is the dense center — ke, mm/ARM3. The EXTRACTED crates
│                           #   (iomgr, ex, se, config, pnpmgr, ps, ob, fsrtl, kdbg, fstub,
│                           #   kd/kd64, rtlk, cc, po, dbgk, lpc, wmi, inbv, vdm, vf, …) talk to
│                           #   it over the FFI symbol boundary, NOT crate:: paths.
│                           #   See docs/engineering/ntoskrnl-crate-split-plan.md.
├── {iomgr,ex,se,config,pnpmgr,ps,ob,fsrtl,kdbg,fstub,kd,kd64,rtlk,cc,po,dbgk,lpc,wmi,inbv,vdm,vf}-rust/
│                           # Subsystems peeled out of the ntoskrnl-rust monolith (crate-split).
│                           #   Each deps ONLY ntkrnl-sys + dbgprint-rust; forced into the link by
│                           #   kernel-rust's `extern crate` (37 rlibs today). Extract another via
│                           #   the plan's recipe.
├── ntoskrnl-cargo/         # PRODUCTION cargo-driven ntoskrnl.exe link (graduated from cmake
│                           #   add_executable — reactos-link engine; ntoskrnl/CMakeLists.txt:118)
├── ntkrnlmp-cargo/         # SMP-kernel sibling link (ntkrnlmp.exe): kernel asm assembled with
│                           #   CONFIG_SMP. ON MASTER since 2026-07-11 (M0–M3 complete: -smp 2
│                           #   boots to desktop, both CPUs schedule, TLB shootdown + generic-call
│                           #   IPI live; all opt-in via `build --smp-kernel` + `smoke --smp N`,
│                           #   defaults stay UP). Quest log + remaining M4/M5 work:
│                           #   docs/engineering/active-lanes/smp-bringup-quest.md
├── kernel-rust/            # staticlib umbrella that pulls in all sibling rlibs via extern crate
├── ntkrnl-sys/             # Two-layer FFI crate: hand-written + bindgen (see below)
├── hal-rust/               # HAL Rust translation — x86 flavors (i386 + amd64, minihal feature
│                           #   for FreeLDR) AND arm64 (GIC/timer/DMA-adapter/PL031 modules)
├── rtl-rust/               # Umbrella over the algorithm rlibs from sdk/lib/rtl/* (~39 crates)
├── slist/                   # Lock-free SList (LDXP/CMPXCHG16B/CMPXCHG8B inline asm)
├── slist-loom/              # Loom concurrency tests for SList (std, not linked into kernel)
├── bitmap/                  # RtlBitmap operations
├── avltable/, splaytree/, generictable/  # Generic tree algorithms (used by mm, cm, etc.)
├── mm-avl/                  # Mm-flavor AVL on MMADDRESS_NODE (VadRoot etc.)
├── largeint/, time/, random/, crc32/, encode/, compress/, mem/, byteswap/,
│   utf8/, prefix/, luid/, access/, handle/, interlck/  # smaller algorithm crates
├── ke-inlines/, ke-macros/  # extracted ntoskrnl-rust ke/ helpers as separate rlibs
└── ir-output/              # rustc IR dumping helper
```

The structure isn't an exhaustive list — the workspace has ~250 members today. The principle is: anything that's pure-algorithm with a clean FFI surface gets factored into its own rlib so the workspace `lto=false` + `cgu=256` firewall keeps callers and callees in separate codegen units. (That firewall's *original* job — blocking LLVM's captures(none) volatile-blind miscompile across CGU boundaries — is **toolchain-fixed on LLVM ≥ 21**: we ship rustc 1.92 / LLVM 21.1.3, which respects volatile stores in attribute deduction (see rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN"). The firewall *stays* as belt-and-suspenders and because it still firewalls the **toolchain-independent** ABI-mismatch `noreturn`/ud2 cascade; `tools/scan_captures.py` is the tripwire to rerun after each toolchain bump in case a future LLVM regresses it.) The **crate-split** effort (docs/engineering/ntoskrnl-crate-split-plan.md) applies that same principle to the kernel itself for a second reason — cutting the edit→rebuild loop, since rustc's frontend is whole-crate.

### Rust Targets (GNU/LLVM, not MSVC)
- **ARM64:** `aarch64-pc-windows-gnullvm`
- **amd64:** `x86_64-pc-windows-gnu`
- **i386:** `i686-pc-windows-gnu`

### Reusable Cross-Platform Pattern
```
component/src/
  lib.rs        — FFI (#[no_mangle] extern "C", cfg for arch-specific conventions)
  algorithm.rs  — Pure logic (no unsafe, no asm, no cfg — Kani-verifiable)
  types.rs      — Shared types (cfg for platform layout variants)
  atomics.rs    — Platform asm (cfg(target_arch), common interface)
```

Principles:
1. **Algorithm is pure** — testable, Kani-verifiable, identical on all platforms
2. **Types use cfg at the boundary** — pack/unpack hides layout differences
3. **Atomics provide a uniform interface** — loop structure is identical per arch
4. **FFI is thin glue** — combines atomics + algorithm, minimal logic
5. **One crate, three targets** — same Cargo.toml for all

### Key Rust/FFI Gotchas
- **LLVM reserves rbx (amd64) and ebx (i386)** — use `xchg` workaround for CMPXCHG16B/CMPXCHG8B inline asm. **On amd64, the xchg MUST be 64-bit** (`xchg rbx, rbx_save`), not 32-bit (`xchg ebx, ebx_save:e`) — a 32-bit register write zero-extends into rbx, silently destroying the upper 32 bits of any caller-held pointer there. Cost a multi-hour BUGCHECK 0x50 debug session in 2026-05.
- **`#[no_mangle] extern "stdcall"`** on i686-gnu correctly produces `_Name@N` decoration
- **`#[export_name]` gets double-decorated** on i686-gnu with stdcall — use `#[no_mangle]` instead
- **i386 SList Pop must be `global_asm!`** — `KiCheckForSListFault` checks exact instruction bytes (0x8B 0x18)
- **NTAPI vs cdecl ABI** matters per arch: i386 NTAPI = stdcall (`_Name@N`), amd64/arm64 NTAPI = cdecl (just `Name`). `extern "system"` resolves to the right thing per arch. Mismatched `extern "C"` vs `extern "system"` declarations across rlibs link silently on amd64/arm64 but break i386. Detect with `tools/rustconv/rustconv/verify_ffi.py` (the gate's workspace-wide ABI checker; `verify_abi.py` is the narrower calling-convention-only variant it grew out of).
- **Cross-DLL data imports** (HAL referencing kernel-data exports like `KdDebuggerNotPresent`, `KeServiceDescriptorTable`): need `#[link(name = "ntoskrnl", kind = "raw-dylib")]` on the extern static block. `postprocess_bindgen.py --hal` adds this automatically for the 41 affected externs.

### ntkrnl-sys — Kernel FFI Declarations Crate (overview)

Dual-layer FFI crate at `sdk/lib/rust/ntkrnl-sys/` providing kernel type and function declarations:
- **Layer 1 — hand-written modules** (`ntkrnl_sys::base`, `::ke`, `::mm`, `::ex`, `::ps`, `::ob`, `::rtl`, `::interlocked`): curated declarations, inline functions, macros, and convenience wrappers bindgen can't generate.
- **Layer 2 — bindgen kernel module** (`ntkrnl_sys::ddk::*`): auto-generated from `ntoskrnl.h` + `mm/ARM3/miarm.h`, covering all internal subsystems (~3,300 extern fns, ~1,850 structs, ~4,400 type aliases per arch, with correct i386 fastcall/stdcall conventions). ~1,500 compile-time `size_of` assertions per arch verify C layout.

Full reference (directory tree + line counts, size-verification `verify_sizes.py`, bindgen regeneration via `generate_bindings.sh`, limitations), plus the CMake build-integration mechanics and the "Adding New Crates" steps: `docs/engineering/rust-in-kernel.md`.

### Cohabitation pattern (HAL)

For HAL Rust translation, each .c → .rs swap is independently revertible: the C file gets commented out in the per-flavor cmake (`hal/halx86/{generic,legacy,pic,acpi,apic}.cmake`) with a `# replaced by Rust (path)` line, and the Rust file's `#[no_mangle]` symbol takes over. `--allow-multiple-definition` allows side-by-side C+Rust during the swap-in phase; lld first-match wins. (Killing that flag outright is a parked lane — branch `kill-allow-multiple-def`, resume state in-branch.) See `docs/engineering/rust-in-kernel.md` → "HAL ABI contract" for the durable ABI reference, and `docs/engineering/archive/hal-translation-audit.md` for the (closed) per-file mapping.

### CLI

```bash
source .venv/bin/activate

python -m rustconv scan ntoskrnl/ex/uuid.c            # Parse, print analysis
python -m rustconv resolve ntoskrnl/ex/uuid.c          # Resolve deps → import manifest
python -m rustconv prompt ntoskrnl/ex/uuid.c           # Generate prompt to stdout
python -m rustconv estimate ntoskrnl/ob/               # Token count + cost estimate
python -m rustconv submit ntoskrnl/ex/uuid.c -o out.rs # Single-file API call
python -m rustconv submit ntoskrnl/ex/uuid.c --assemble  # Translate + write to crate
python -m rustconv translate ntoskrnl/ob/file.c        # Single-file synchronous (better for >1000 LOC; batch silently stubs lazy fns)
python -m rustconv batch ntoskrnl/ob/ -m sonnet        # Batch entire subsystem (smaller files only)
python -m rustconv probe-migrate se/sqos.rs            # Stage 2: ProbeForRead/Write → Result<T>
python -m rustconv trace-catalog                        # Generate debugcon trace catalog from inline asm
python -m rustconv check                                # cargo check ×3 targets
python -m rustconv candidate-check candidates.json --repair  # Batched x3 candidate oracle
python -m rustconv substrate-plan path/to/report.json   # Cluster compiler roots by authority
python -m rustconv crate-repair foo-rust --arches i386 amd64 arm64  # $0 deterministic repair
python -m rustconv.profile_derive dll/win32/foo --emit report       # Derive bindgen island
python -m rustconv bank-audit [--verbose]               # Paid .responses output NOT in the tree
python -m rustconv.export_audit --module dll/ntdll dll/ntdll/def/ntdll.spec  # Does the DLL still export what it promised?
python -m rustconv difftest-contract C.json --out DIR   # Grade shipping Rust vs genuine C (see Test Suite)
python -m rustconv objdiff a.obj b.obj                  # C vs Rust object ABI differences
python -m rustconv seh-classify file.c                  # Classify a C file's SEH blocks by pattern
python -m rustconv seh-inventory                        # win32k SEH triage (sites a–e + sidecars)
```

`export_audit` asks whether every implementable `.spec` entry is a `#[no_mangle]` fn with the
spec's calling convention. **Use `--module <cmake dir>`** — it resolves the module's C dir, every
crate the build declares it links (`add_cargo_module`/`add_cargo_kernel_dll`, `BODY_CRATES`, the
cargo crate's `build.rs` `.rust_lib(..)`, and any `include()`d `.cmake`), and the assembly it
assembles. Passing one crate directory by hand reports the other crates' exports as dropped —
hidparse reads as 30 MISSING against `hidparse-rust` alone and CLEAN once `hidparser-rust` is in.

Five verdicts, and four of them are "not a defect, here is who covers it": `STILL_C` (a
still-compiled `.c` defines it — cohabitation working), `GENERATED` (the module's assembly does:
a syscall stub expanded from a `SVC_` table, or a literal asm label), `NOT_EXPORTED` (a Rust fn
exists but is name-mangled — the StgCreateDocfile shape), `ABI_MISMATCH`, and `MISSING`. The
point of each split is that `MISSING` stays readable: ntdll reported **649 MISSING** before
`GENERATED` existed, of which **593** were the Nt*/Zw* stubs `ntoskrnl/ntdll.S` generates and
another ~20 were hand-written asm. It is 35 now, and those 35 are the CRT variadics and integer
intrinsics that come from reused Tier-C `.a` archives — a fifth supplier nothing models yet.
`--arch` also drops the `.spec` rows that arch does not build (`-arch=arm`, and the `win32`/
`win64` width aliases), which both removes false MISSINGs and keeps a per-arch pair like
hal.spec's two `ExAcquireFastMutex` rows from collapsing onto whichever is written last.

`bank-audit` is the oracle behind the "never delete a `.responses/` dir holding unintegrated
work" rule — it runs `.responses` → tree, the opposite direction to `fn_coverage`/`export_audit`,
so it sees paid bodies that never reached a crate (nothing references them, so nothing else
complains). Exits nonzero on a finding. A banked NAME is a label, not a symbol: `--verbose` shows
how each alias resolved (macro-expansion body, varargs twin, TCHAR macro, allowlist). Reviewed
exceptions live in `tools/rustconv/bank_audit_allow.toml` and each needs a written reason.

### Deterministic substrate closure

After assembling a candidate, run `candidate-check --repair`, then use the embedded report or
`substrate-plan` to fix distinct compiler roots by authority—not hundreds of individual rustc
sites. Full-module manifest entries should include `"source"` so source-local definitions are
routed correctly. Reuse/promote canonical Rust first, then project bindings, SDK/header
generation, explicit local/bridge generation, and scope repair. Constants, SDK layouts, WIDL
interfaces, vtables, ABI, and imports never go through the LLM. Ambiguous layouts stay red;
same-arity local overloads and trailing C++ default arguments are likewise resolved and
expanded deterministically before payment. If preflight cannot prove one binding, extend the
harness or leave the class red; never ask the model to guess C++ compiler semantics.
Sonnet is considered only after substrate closure leaves a genuine semantic/structural
residual. The complete protocol and stop rules are in
`docs/engineering/rustconv-substrate-closure-protocol.md`.

### How much C is left — `tools/port_audit.py`

```bash
python3 tools/port_audit.py                # the generated core ledger (markdown)
python3 tools/port_audit.py --json         # same rows, machine-readable
python3 tools/port_audit.py --check        # ratchet: every compiled source is classified
python3 tools/port_audit.py --detail "Win32 DLLs"   # live directories in one surface
python3 tools/port_audit.py --rust         # lines of Rust: translation / generated / harness
```

Answers "where are we at on the core OS" from the **ninja build graph of this worktree's
three build dirs**, unioned — so a surface's "live C" is what the link actually compiles,
after every `if(ARCH)` and every commented-out CMake entry. Core is everything we own
that ships in the base OS, **Wine-derived code included** (provenance is a column, not a
scope); only `vendor.yaml` trees, applications, tests and host tools are reported
separately. Its lines are C lines — compare Rust progress with `--rust`, which is ~2x.

Two things it will not do. It **hard-fails** on a missing build dir or a graph older than
a CMakeLists edit, rather than reporting the smaller tree a partial answer produces (fix:
`./vibeos-cli build --arch <a>`); and `--check` **fails on a compiled source that matches
no rule**, so a new top-level area has to be classified before the ledger prints at all —
the same ratchet `vendored.py --check` applies to a new vendor drop. Coverage is computed
from the `Rust twin` column (the module has a crate in `sdk/lib/rust/`), never from
"out of build", because out-of-build also holds upstream C this port never compiled.

It is a LINE ledger, not a symbol one: for who backs a specific export ask
`export_census.py`, and for whether a flip took at the symbol level ask
`verify_cohab_coverage.py`, which reads object files. Output goes in
`rust-port-status.md`; do not hand-edit those tables.

### Which C is NOT ours — `vendor.yaml`

```bash
python3 tools/vendored.py --emit           # the table: 23 trees, ~1.77M lines
python3 tools/vendored.py                  # --check (a `vibeos-cli check` stage)
python3 tools/vendored.py --write-stamps   # regenerate the VENDORED markers
python3 tools/vendored.py --suspects       # uncatalogued foreign-copyright dirs
```

`vendor.yaml` (repo root) is the **only** list of vendored third-party trees — ACPICA,
FreeType, FEX, Microsoft's ucrt, and the `sdk/lib/3rdparty` + `dll/3rdparty` sets. It is
what makes "how much of *our* C is translated" a computable question, and it is the
answer to "should I translate this?" for anything under those paths.

Adding a vendored library: add an entry, then `--write-stamps`. `--check` refuses an
unregistered directory inside a vendor container, a version probe that no longer matches,
and a stale stamp. Versions are stored as a probe (file + regex), never as a copied
literal — that is how 78K lines of Microsoft ucrt ended up here with no version stamp at
all. Do not add a second list of vendored paths anywhere; import `tools/vendored.py`.

`--suspects` is the discovery half: the container ratchet only watches the three vendor
containers, so a foreign tree living anywhere else (ACPICA under `drivers/`, ucrt under
`sdk/lib/`) is invisible to it. It reports directories of foreign-copyright source the
registry does not cover — **~400K lines today, WDF's 290K the bulk of it**. Those are
QUESTIONS, deliberately not gate findings: whether a driver framework this OS ships is a
translation target is a judgement call, and auto-excluding on a copyright grep would
shrink the denominator silently, which is the failure the registry exists to prevent.

### Moving vendored code — `tools/vendor_move_plan.py`

```bash
python3 tools/vendor_move_plan.py                       # blast radius per tree, cheapest first
python3 tools/vendor_move_plan.py --tree <vendored path> # every reference to one tree
```

Lists every reference that pins a vendored tree to its path — relative CMake, `.cmake`
fragments, bindgen scripts, include-root consumers, and paths built from variables —
before anyone runs `git mv`. Plan and hazards: `docs/engineering/vendor-relocation-plan.md`.
### Static Analysis Tools

```bash
# Check for LLVM miscompilation (ud2 in named functions)
python3 tools/scan_ud2.py build-i386              # Report all ud2
python3 tools/scan_ud2.py --check build-i386      # Exit 1 if unexpected ud2

# Check for unprotected output pointer writes (captures(none) vulnerable)
python3 tools/lint_kernel.py                       # Report all violations
python3 tools/lint_kernel.py --fix                 # Show suggested fixes
python3 tools/lint_kernel.py path/to/file.rs       # Single file

# Check sacred primitives survive optimization (KfRaiseIrql, KeAcquireSpinLock)
bash tools/verify_sacred_primitives.sh build-i386

# Check constant values against C headers
.venv/bin/python tools/rustconv/rustconv/verify_constants.py                    # Check all
.venv/bin/python tools/rustconv/rustconv/verify_constants.py --fix              # Auto-fix

# Check ABI mismatches
.venv/bin/python tools/rustconv/rustconv/verify_ffi.py

# Silent-failure shapes: bare loop{} spins, unwrap/expect in kernel code,
# swallowed Err arms (AST/tree-sitter; wired into `vibeos-cli check`, 0-NEW gate)
.venv/bin/python tools/scan_silent_failures.py           # sweep no-std workspace
# Suppress a sanctioned loud halt: // nolint: silent-failure (reason)

# Hand-written NUMERIC struct offsets (the MMPFN offset-shift / KTHREAD.Teb-0x1e8
# drift class — verify_offsets.py proves bindgen DECLS, this catches USAGE sites;
# fix = core::mem::offset_of!(bindgen type, field), never a bare number)
python3 tools/scan_magic_offsets.py                      # sweep sdk/lib/rust
# Suppress a verified FP: // nolint: magic-offset (reason)

# Numeric bugcheck STOP CODES (§9.113: the LLM invented the number for 61 of ~115
# translated KeBugCheck(SYMBOL) calls — plausible, self-consistent, and naming the
# WRONG subsystem, e.g. INVALID_AFFINITY_SET -> an IRP bugcheck. Invisible to rustc
# and to every smoke test; it only bites once the kernel is already dying.)
python3 tools/scan_bugcheck_codes.py                     # empty baseline — keep it at 0
python3 tools/gen_bugcodes.py                            # regenerate ntkrnl_sys::bugcodes
# Fix = KeBugCheckEx(ntkrnl_sys::bugcodes::NAME, ...); suppress a deliberate
# non-code (debug marker, DEFAULT_UNREACHABLE) with // nolint: bugcheck-code (reason)

# INVENTED CONSTANTS, generally (§9.113a/b) — two checkers that compose. Use BOTH:
#   verify_constants = precise oracle, but only sees headers in its curated catalog
#   scan_const_contradictions = no oracle, total coverage: "our tree spells one name
#   two ways, so one is wrong". Loop: contradictions finds the family -> resolve vs
#   the real C header -> add that header to verify_constants' catalog -> gated forever.
.venv/bin/python tools/rustconv/rustconv/verify_constants.py   # (--fix to auto-apply)
python3 tools/scan_const_contradictions.py               # baseline = debt + work queue
# DON'T "fix" coverage by sweeping all 2216 headers — tried, 486 findings, nearly all
# false (cross-header name collisions). §9.113b has the evidence.

# Boot coverage map: which functions a real boot EXERCISES (TCG plugin boot)
tools/coverage/run.sh amd64        # -> per-function exercised set + the
                                   #    never-executed-rust.txt landmine registry

# captures(none)-class post-opt IR audit (POISON_ARG committed folds /
# READNONE_OUTPARAM latents / UNREACHABLE_BODY). Class is toolchain-FIXED on
# LLVM >= 21 — rerun this after every rustc/LLVM bump to prove it stayed fixed.
VIBEOS_EMIT_LLVM_IR=1 ./vibeos-cli build --arch amd64   # emit per-CGU .ll
.venv/bin/python tools/scan_captures.py sdk/lib/rust/target/ntoskrnl-cargo-*-rustlib
# Related: VIBEOS_LINK_MAP=1 (lld -Map per cargo module — layout ground truth);
# VIBEOS_LAYOUT_CANARY=1 (deliberately perturbed kernel link — boot tripwire)
```

**Triage scanners that are NOT wired into `check`** — run them by hand; each is a work
queue, not a gate, and a finding is "look here", not "this is wrong":

```bash
python3 tools/scan_sibling_drift.py    # paired twins (Acquire/Release, Nt/Zw, X/XEx, per-arch
                                       # cfg twins) whose bodies differ on one line. 65
                                       # candidates, all hand-read 2026-09-20, re-checked 09-26
python3 tools/scan_arch_cfg.py         # ~2.8K per-arch #[cfg] sites that gate LOGIC, out of
                                       # ~6.7K; the input list for a per-arch drift audit.
                                       # Diff against the last audited tree, not the whole list
python3 tools/lint_wide_strings.py     # hand-encoded UTF-16 b"..." literals: C-octal confusion
                                       # and odd lengths (currently clean)
python3 tools/find_dead_tus.py         # linked TUs whose symbols nothing references — dead
                                       # C still counted as untranslated work
.venv/bin/python -m rustconv.hybrid_closure   # which symbols a half-translated TU still
                                              # exports to the link (CFSFolder: 2,283 lines, 21 symbols)
```

`scan_ud2.py` finds `ud2` (0f 0b) in named functions — LLVM proved the body has UB (on LLVM ≥ 21 almost always an ABI mismatch; the captures(none) volatile-blind root is toolchain-fixed, see rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN"); it's wired into smoke as `test_no_unexpected_ud2`. `lint_kernel.py` finds `*OutputParam = value` writes in `#[no_mangle]` functions that skip `write_volatile` (a captures(none) target) before compilation. (The smoke harness also nukes `rust-kernel-target/` per session unless `--skip-clean-build`, runs the ud2 scan pre-boot, and auto-symbolicates BSODs — see `tests/qemu/cli/README.md`.)

### Cost (Sonnet **Batch** API — the cheapest path, NOT what you pay synchronously)

These are **batch Sonnet** figures (50% off, async). The synchronous `translate`/`submit`
path you actually use pays **full list price**, and `-m opus` is **~3.3× the Sonnet rate**
($5/$25 vs $3/$15 per 1M) on top of losing the batch discount — so a "Full ntoskrnl"
Opus-synchronous run with per-function fan-out is an order of magnitude past the bottom row.
Always sanity-check against `rustconv estimate`'s "sync @5x" column and the live cost counter.

| Scope | Est. Cost (batch Sonnet, floor) |
|-------|-----------|
| Single file | ~$0.03 |
| Subsystem (11 files) | ~$0.30 |
| Full ntoskrnl (393 files) | ~$8.70 |
| 5 iterations | ~$43.50 |

## ARM64 Specifics

### Memory Layout
```
User space:     0x0000000000000000 - 0x0000FFFFFFFFFFFF
Kernel (KSEG0): 0xFFFFF80000000000
HAL VA:         0xFFFFFFFFFFC00000 - 0xFFFFFFFFFFFFFFFF
```

### QEMU virt
RAM at physical 0x40000000 (1GB). PL011 UART at 0x09000000. Serial via Unix socket at `qemu/serial.sock`.

### UART Trace Pattern (kernel debugging)
```c
extern volatile PVOID Arm64UartKernelBase;
volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
if (u) { while (*msg) *u = (ULONG)*msg++; }
```

### ESR Exception Classes
0x25=data abort, 0x21=instr abort, 0x3C=brk, 0x15=SVC

## Target Hardware

All development and CI happens in **QEMU**: `virt` machine (ARM64), `q35` (amd64), or standard PC (i386).

**Acceleration / backend (this misled people — read it):** the default smoke backend is **`windows-host`** — the harness launches the **Windows** `qemu-system-*.exe` on the host via WSL interop (media staged to `/mnt/c/temp/vibeos-qemu`), running **TCG (pure emulation) on all three arches**. This deliberately escapes WSL2 nested-virt: host TCG beats nested KVM, and there's no single-virt-slot contention, so the beast host can run all arches in parallel. **We are *not* on `/dev/kvm`** despite it being present in WSL2. Two alternates exist: `VIBEOS_QEMU_BACKEND=native` (local Linux qemu + Unix sockets + KVM for x86 — the old path, still used by the kernel-debug toolkit's GDB flow), and **WHPX** (Windows Hypervisor Platform, real x86 hardware virt in `qemu-system-x86_64.exe`, `accel: "whpx"`) — **works since 2026-07-10: both x86 arches boot to desktop in ~52s** (`VIBEOS_QEMU_WINDOWS_ACCEL=whpx`; the boot CD rides an ich9-ahci controller under whpx so OVMF DMAs it instead of ~5min of PIO VM-exits — see `docs/engineering/windows-host-qemu-whpx-spike.md` + field manual §9.46 for the six guest-side bugs that bring-up flushed out). Still opt-in (default stays TCG) until whpx accumulates reliability history; same-arch whpx boots serialize (hardware-virt contention, like KVM). arm64 is always TCG (WHPX is x86-only). Config + per-arch resolution: `tests/qemu/cli/config.py` (`qemu.backend`/`qemu.windows.accel`) + `tests/qemu/backend.py` (`resolve_accel`). Real hardware: a **Raspberry Pi 4B** bench boots arm64 VibeOS to the desktop (UP kernel, 2026-09-25). Boot your tree on it with `./vibeos-cli pi4 boot` (lease + doctor + desktop verdict, ~2 min). Bench wiring and the hands-free netboot loop: `docs/engineering/pi4-bench.md`; findings: `docs/engineering/active-lanes/pi4-bringup.md`.

## rustconv — model shortcuts, HAL mode, and pipeline stages

Model shortcuts: `sonnet`, `haiku`, `opus`, `gemini-pro`, `gemini-flash`, `deepseek`, or full model ID.

**The default is `deepseek-v4-pro` ($0.28/$1.10 per 1M) — you do not pass `-m` to get the cheap
lane, you pass it to LEAVE.** `get_backend()` prefers `DEEPSEEK_API_KEY` over
`ANTHROPIC_API_KEY`, so an unflagged `translate`/`submit`/`batch` bills DeepSeek. Sonnet is
**12.2× the list price** and is an escalation, not a floor: route substrate-clean high-band
compile failures to `-m sonnet` when agent repair time matters, and keep low/medium and
deterministic shapes on the default. Retrying a rejected worker with a stronger model measured
0/3 accepted for $0.13 — a reject is substrate work, not a model-capacity problem.

Switching models never re-bills: the resume-cache key (`preparation_namespace`) is mode + seal
flags, with no model component, so a unit banked by any model replays for $0 under any other.

`rustconv translate` accepts HAL paths and is HAL-aware (separate `system_prompt_hal.md`, hal_ddk imports, auto-CMake edit). Output quality is ~70% useful for HAL — review before integrating.

The pipeline has multiple stages: Stage 1 (C → Rust, `submit`/`batch`), Stage 2 (probe migration, `probe-migrate` — `ProbeForRead`/`Write` → `Result<T>`; SE+OB done), Stage 3 (Result+Drop-RAII idiomation, planned). See `docs/engineering/rustconv-pipeline.md` for the per-stage design.

## Branch history

Work was previously staged on `continue-rust-port` with `master` as a periodically
fast-forwarded mirror; the two were unified onto `master` on 2026-07-03 and
`continue-rust-port` was retired. You may still see the old branch name in stale refs.
