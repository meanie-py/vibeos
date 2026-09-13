# Promoting amd64 to the primary x86 arch

**Goal: when someone downloads an ISO to try vibeos, it is the amd64 one, and x86
applications run well on it.** Deprecating i386 as a *shipped* arch is the
follow-on prize (a real pile of tech debt retires with it), but it is not the
near-term ask and nothing here proposes dropping it soon.

Opened 2026-09-13 by the `research` lane, as a survey. The depth work below is
mostly owned by `wow64-amd64`; see **Ownership** before starting anything.

---

## The finding that reframes the goal

**i386 does not go away when the i386 kernel does.** SysWOW64 on an amd64 livecd
is populated *from a sibling `build-i386` tree* — `dll/win32/wow64cpu/CMakeLists.txt`
stages the full transitive import closure of the stock guests (computed by
`wow64-closure.py`, all-or-nothing, `FATAL_ERROR` on a partial set) out of
`${REACTOS_SOURCE_DIR}/build-i386/...`, plus the nine probe guests from
`build-fex-hello/` and ~15 stock applets. A WoW64-capable amd64 ISO literally
contains i386-built binaries, because that is what a 32-bit process loads.

So the thing that can retire is the i386 **kernel, HAL and boot path**. The i386
**user-mode build** has to survive for as long as we support running x86 apps —
which is the whole point of the promotion. Worth stating plainly because the
tree currently implies otherwise: `sdk/cmake/gcc.cmake:638` says *"That cell is
retired by the wow64 lane (i386 kernel goes away), not fixed here."*

That is optimistic as written, and the reason is one line up. `add_cargo_module`
gates on `ARCH STREQUAL "i386"` alone (`gcc.cmake:638-645`, `:803-807`), so
**every** i386 `*-cargo` cell gets nightly + `-Z build-std` + the custom
`i686-kernel.json`, user-mode ones included — `dll/ntdll`, `base/system/smss`,
`winlogon`, `lsass`, `services`, `csrss`, `basesrv`, `mlang`, `cabinet`. But the
reason for that cell is `rustc-abi: x86-softfloat`, which is a *kernel-mode*
constraint (no SSE at kernel IRQL). User-mode i386 has no such problem and
`i686-pc-windows-gnu` is a builtin stable triple.

**Hypothesis worth one experiment, not yet tested:** split the i386 cargo cell by
`KIND` — kernel cells keep the softfloat json and nightly, user-mode cells move
to the builtin triple on pinned stable.

*Scoped 2026-09-13 by reading only, no experiment run, because "flip the branch"
understates it.* `i686-kernel.json` carries ~10 non-default fields. Most are **link
plumbing the amd64/arm64 migration already proved neutralisable** — their old jsons
were "the builtin spec minus the user-mode CRT link plumbing", which moved to link
time (`reactos-link` stub archives + `-C link-self-contained=no`), with
`frame-pointer: always` restored as `-C force-frame-pointers=yes` and
`panic-strategy: abort` living in the workspace `Cargo.toml` profile. So
`dynamic-linking`, `allows-weak-linkage`, `eh-frame-header` and `requires-uwtable`
are covered by precedent. **Two fields are not, and they are the whole task:**
`rustc-abi: x86-softfloat`, the stated reason the cell exists and a kernel-IRQL
constraint the split assumes user-mode does not share; and
`abi-return-struct-as-int: true`, which gcc.cmake's own note calls "read solely by
the 32-bit x86 callconv, so it is inert **here**" — *here* being the 64-bit
targets. On i386 it is live, so **the amd64/arm64 ABI-identical proof does not
transfer at all**: it held precisely because the only flipping field was inert
there. Get that one wrong and the symptom is a silently wrong struct return, not a
build error. `cpu: pentium` also needs carrying across as `-C target-cpu=pentium`.
A scoped ABI task with one real unknown, then — still worth doing, still unowned.

**MEASURED 2026-09-13 against the builtin spec** (`rustc --print target-spec-json --target
i686-pc-windows-gnu`), which shrinks the task and retires the risk flagged just above:
`abi-return-struct-as-int` is **`true` on BOTH**, so the field I called the unresolved ABI
danger agrees, and `data-layout` is byte-identical. `frame-pointer: always`,
`function-sections: false`, `allows-weak-linkage: false`, `eh-frame-header: false` already
match, so none needs a restoring `-C` flag, and `panic-strategy: abort` is carried by the
workspace `Cargo.toml` profile. What actually differs: `rustc-abi` (custom `x86-softfloat` vs
builtin **`x86-sse2`**), `cpu` (`pentium` vs `pentium4`), `dynamic-linking`, `requires-uwtable`.

**And the direction of travel is the REVERSE of this item's premise.** The question was whether
user-mode i386 can leave softfloat for the builtin triple. Measured answer: anything disabling
SSE2 on i386 *needs* a softfloat ABI, because rustc is phasing out exactly the combination 55
crates in this tree already use — see "i386 crates build on a deprecated SSE/ABI combination"
in the roadmap registry. On the custom json, the kernel cells are the ones already correct. If it holds, the nightly surface shrinks
to the i386 kernel today and reaches zero the day that kernel retires, *without*
waiting for SysWOW64 to stop needing an i386 ntdll. If it does not hold, we learn
why, and the "last nightly cell" item gets an honest blocker instead of an
assumption. Do not quote the payoff until someone has run it.

## Where the fex lane's work transfers — the marshalling is ALREADY amd64-verified

`wow64.c:144-149` is explicit: the CPU layer differs by host — a JIT on arm64 (FEX), a native
heaven's-gate mode switch on amd64 (`wow64cpu`) — but "both export the identical `BTCpu*`
contract this file resolves by name, so only the DLL name and the `WOW64INFO`
NativeMachineType change". The marshalling sits **above** that contract. A 32-bit guest talking
to a 64-bit CSR server is a POINTER-WIDTH problem, not a CPU-layer problem, so the arm64/amd64
split does not apply to it at all.

**And the provenance runs amd64-first, not arm64-first.** The CSR/LPC family
(`ReadOnlyStaticServerData`'s 8-byte read at a 4-byte stride, `BASE_STATIC_SERVER_DATA`
narrowing, the CSR reply `Data` union, `BasepCreateThread`) was found, instrumented and
verified **on amd64**, against the native heaven's-gate path; arm64 followed as confirmation.
Both now report 9/9 on `smoke --ladder`. So treat that work as **verified on the host this plan
cares about** — do not re-derive it as though it were arm64 results of uncertain transfer.

*(This section previously said the opposite, on a control-plane summary that lane has since
retracted and that I recorded under its attribution. The `wow64.c` quote above is checked in
this tree; the amd64-first provenance is fex's, relayed.)*

## What "primary" means — a decision, not an investigation

Nobody has defined this, and it should be settled before the work is sequenced,
because it determines what counts as done:

- Which arch's red blocks a land? (Today all three are peers.)
- What is the default `--arch`? There is deliberately none, and that rule has
  paid for itself. "Primary" must not quietly reintroduce one.
- Which ISO is *the* download, and does the other two staying green remain a
  gate requirement or become a best-effort?

This is the user's call. The engineering below is orthogonal to it.

---

## Cleared — checked, and not blockers

Recorded because each is a plausible-sounding blocker that costs an afternoon to
re-derive, and all three came back negative.

1. **SEH parity.** i386 does *not* hold real `fs:[0]` PSEH while the 64-bit
   arches make do. `sdk/cmake/gcc.cmake:42-47` sets `USE_DUMMY_PSEH 1` for i386
   under llvm-mingw too, with `USE_PSEH_CLANG` commented out as a TODO. All three
   arches are on DUMMY_PSEH; the direction of travel is
   `docs/engineering/dummy-pseh-verdict.md` (Rust semantics, not a C mechanism)
   and it is arch-neutral. Demoting i386 costs nothing here.

2. **Component coverage.** ~40 `if(ARCH STREQUAL "i386")` gates in `drivers/`
   read like i386-exclusive builds — the whole USB stack, fastfat, hidusb, the
   audio stack. Every one opened is Rust target-triple selection
   (`i686-pc-windows-gnu` + `-C target-feature=-sse,-sse2`), not an exclusion.
   Same component set on all three arches. (Separately: that identical 12-line
   block is copy-pasted into 40+ CMakeLists. Real debt, arch-neutral, and a
   candidate for the "a wrapper script is a bug report against the tool" rule.)

3. **HAL and SMP.** amd64 is the strong arch here, not the weak one — it was
   already APIC before the `i386-apic-hal` lane brought i386 up to parity. The
   2026-07-12 "amd64 AP-start caps at ONE AP" finding is fixed: the SMP quest
   records amd64 `-smp 2` 2/2 and `-smp 4` 2/2 post-change. And BIOS
   `freeldr.sys` was pruned, so uefildr is the only loader — amd64 boots UEFI
   like the others.

**Also stale: the roadmap's headline WoW64 blocker.** "Every 32-bit guest is
refused by CSR — the shared section can't map into a WoW64 process (OPEN)" was
closed 2026-09-12/13 by `c277b0000d4` (M19: CSR/LPC works, a 32-bit binary
reaches PASS) and `a154628337b` (`BasepCreateThread` marshalled; **all nine probe
guests PASS on amd64**). The registry entry is corrected in the same commit as
this doc.

## Found by running it, 2026-09-13

Both of these sat between "the gate can watch the ladder" and "an amd64 tree has
a ladder to watch", and neither was visible from reading.

- **An amd64-only tree could not build its own WoW64 test guests.**
  `tools/fex/build-hello-compute.sh` opened with a hard `exit 1` when the FEX
  vendor build was missing, then built eleven things: two arm64 probes that
  genuinely need FEXCore, and the nine i386 guests that do not. On amd64 the CPU
  layer is our own `wow64cpu.dll` — no emulation — so FEX is not in that picture
  at all. Fixed (`aaaddafa1f4`): the guard now scopes to the step that needs it.
  Relevant beyond the script, because "amd64 is primary" means an amd64 tree must
  be self-sufficient for its own x86 coverage.
- **The nine guests were the last PE family stamping wall-clock.** Those direct
  clang links bypass `toolchain-llvm-mingw.cmake`, and with it the
  `--no-insert-timestamp` that `_VIBEOS_LLD_COMMON` gives everything CMake builds
  (deterministic-builds rung 2). Two links of `Hello32.c` two seconds apart
  differed in exactly 2 bytes tracking the clock — and hello32 links nothing, so
  it could not have been the importlibs. It matters because the guests are staged
  onto the livecd, so a rebuild changing nothing still moved the media, and media
  identity is what `smoke` staleness and the gate stamp key on. Fixed
  (`5a322b1c9be`); `vibeos-cli repro report` now reads `TimeDateStamp 0`.
- **A warm build silently ships the last configure's staging, and CMAKE_CONFIGURE_DEPENDS
  cannot fix it.** Staging is snapshotted at configure time; a warm `vibeos-cli build` does not
  reconfigure (`_maybe_canonize` early-returns at `build.py:437` for an already-canonical dir —
  *not* `:464`, which governs the nuke-and-reconfigure branch a warm build never enters). The
  obvious fix is this file's own idiom, since `wow64cpu/CMakeLists.txt` already lists
  `wow64-closure.py` and `nettcpip.inf` in `CMAKE_CONFIGURE_DEPENDS`. **Measured with a
  throwaway CMake project rather than assumed:** a listed path that does not exist causes no
  reconfigure loop (safe), a listed file that CHANGES does re-run configure (positive control
  passed), but a path that first APPEARS does **not** — which is exactly the case that bites.
  So the idiom closes "guests rebuilt" and cannot close "guests staged for the first time".
  Handled instead where it can be: `ladder.gate_decision` now distinguishes *built but
  unstaged* from *never built* and names the re-configure, turning a silent skip into an
  instruction.
- **The staging probe I wrote to prevent wasted drains caused one.** `media_has_guests`
  byte-scanned the image for `hello32.exe`, which is in EVERY amd64 livecd regardless of
  staging — winlogon's own error path carries the name in plain ASCII. It returned
  staged=True on a boot with zero guests, which then sat through the full 420s ladder
  drain to conclude `LADDER_NOT_STAGED`. Fixed (`907988eeb5f`) to read the
  `boot/<media>.cmake.lst` manifest's CD-path side. Filed in the field manual under
  "A NAME inside an image does not prove the FILE is in it". **The measured consequence
  matters for sequencing: an unstaged boot given `--ladder` costs the whole ceiling**,
  which is why the media check has to be right before this reaches a shared gate.
- **One un-buildable DOOM binary was gating all nine graded guests.** The staging guard
  required every entry of one list — the nine machine-verdicted guests AND ~20 stock applets
  AND `doom-i386.exe` — though `ladder.py` says the applets are not graded. DOOM cannot be
  built here at all: its shareware WAD URL 404s. So the ladder was unreachable, not merely
  unstaged. Split into required vs best-effort (`d7c995a8e17`).
- **The drain burned its whole 420s ceiling on a PASSING run.** `watch()` began from an empty
  buffer, so a guest reporting during the boot's settle read was waited for until timeout.
  Correct verdict, seven minutes of waste, invisible to every correctness check because the
  two halves of the serial are rejoined before grading. Found only because a duration log line
  existed. **Fixed and MEASURED: 420.2s -> 94.3s on the same boot with the same 9/9 verdict**
  (total smoke 554s -> 154s), so the standing cost of the ladder in a gate is ~94s on top of a
  ~60s amd64 boot, not the 420s ceiling. Filed in the field manual as "A watcher that starts from an empty buffer waits for
  what already went past".

---

## Real gaps, ranked by what a user hits

1. **Guest processes.** `CreateProcess` from a 32-bit guest, and cross-process
   `CreateRemoteThread`. Every installer and every app that shells out hits this.
   `wow64-plan.md` calls it "the largest remaining hole" and names the CSR-LPC-32
   layer as its prerequisite — which just landed. Its status is therefore
   genuinely **unknown**, not known-broken, and measuring it on today's tree is
   the first move rather than a design question.

2. **Registry redirection — LARGELY DONE 2026-09-13, one function left.** The wow64-amd64
   lane landed it: `878e0ac0b67` (E24) puts Wow6432Node redirection in the **CM parse**, not in
   wow64.dll, because registry opens are overwhelmingly RELATIVE — advapi32 caches a handle to
   `\REGISTRY\MACHINE` and passes RootDirectory + a subpath, so a user-mode rewrite would need
   a handle-to-path syscall on every open. `911b1e1d82b` (E21) fixed `NtOpenKey` denying every
   `KEY_WOW64_32KEY` open outright. **Residue, verified on master:** exactly two live
   `Wow64 not yet supported` sites remain, both in `RegDeleteKeyExW`, so an explicit
   `KEY_WOW64_32KEY`/`64KEY` on a DELETE is still logged and ignored — a narrow wrong-hive
   hazard confined to deletion. (The two in `RegLoadMUIStringA` are commented out.) **This entry
   previously called the whole feature an open silent bug; that was true when written and is
   not now.** Original scoping kept below, because it is why it mattered:

   *Was:* **a silent wrong answer, not a missing feature.**
   `sdk/lib/rust/advapi32-rust/src/reg/reg.rs:1073-1079` prints
   `ERR!("Wow64 not yet supported!")` and then proceeds into the native hive. No
   `Wow6432Node`; `KEY_WOW64_32KEY` / `KEY_WOW64_64KEY` are logged and ignored. A
   32-bit app writes its configuration into the 64-bit view and the next one
   reads someone else's. This is exactly the class that makes "runs x86 apps
   well" false in a way a probe ladder cannot show, because the ladder does not
   look at the hive. Task #66.

3. **The instrument is not in CI — DONE 2026-09-13 on ALL THREE arches, and it works.**
   arm64 joined 2026-09-13 05:40: a prefix collision in the manifest glob (`{stem}*.lst` with
   stem `boot` matching `bootcd.*.lst`, a different medium) had the manifest probe answering
   "no guests" and shadowing the ramdisk probe that had the right answer, so the opt-in never
   fired there. With that fixed, every arm64 gate runs the ladder — `gate_argv=['--ladder']`
   where it was `[]` — at a **measured 84.7s drain**, consistent with the 94.3s seen on amd64
   and nowhere near the 420s ceiling.

   **Scope, corrected 2026-09-13 06:30 — coverage is CONDITIONAL on staged media, not
   universal.** The gate now *asks* for the ladder on every arch; the ladder then declines
   unless that arch's media carries both a WoW64 CPU layer and the guests. A default arm64
   build stages neither: `qemu/mkramdisk.sh`'s entire FEX block is gated on `VIBEOS_FEX=1`
   **and** on `build-fex-hello/fexprobe.exe` existing, which needs the FEX vendor build. So a
   first post-fix arm64 gate shows `ladder:arm64 skip 0.0s — no WoW64 CPU layer on this arch's
   media`, and amd64 likewise skips in a tree without `build-i386`/`build-fex-hello`.

   What that buys, stated exactly: **a FEX-staged tree gets arm64 gate coverage it never had**
   (that is the fex lane today), default trees are unchanged at 0.0s with no new red surface,
   and the standing ~85-94s cost applies only where the media is staged. So "the instrument is
   in CI" is true conditionally. **Making arm64 WoW64 coverage continuous is a separate
   question** — whether default arm64 media should stage the FEX CPU layer — with a real cost
   attached, and it is not answered here.

   `gate` now opts each arch in on its own media (`ladder.gate_decision`), and the amd64
   ladder is **GREEN end to end: 9/9 guests PASS**, correct codes throughout. Getting there
   took three fixes beyond the wiring, all listed under "Found by running it" above. What
   remains is one confirmation run of `gate --arches amd64` showing the `ladder:amd64` stage,
   deferred only because running a gate during one's own landing is what the pre-land
   checklist forbids. Original scoping, kept because it is why this mattered: The nine probe guests are spawned by winlogon
   *after* explorer, and the smoke's success condition IS the desktop marker — so
   `smoke`/`gate` stop watching before the ladder starts. `tests/qemu/cli/ladder.py`
   exists and is good (per-guest expected exit codes, `LADDER_NOT_STAGED` vs
   `LADDER_FAIL`, a generous drain window so a slow host is not a fake failure),
   but it is reachable only as `smoke --ladder`. `gate` never passes the flag.
   This has already cost twice: `selfmod32` exited `0xBAD9` on amd64 from the day
   E17 landed it, and `thread32` sat dead on rung 1 until `a154628337b`. If amd64
   is promoted *for* x86 compatibility, the gate has to watch the thing it was
   promoted for.

4. **Stock 32-bit GUI apps fault on amd64 — 58 faults each, amd64-ONLY.** notepad, mspaint
   and wordpad take execute-at-NULL faults (`rw=8`) with a re-faulting SEH cascade at
   descending `esp`. arm64 never launches those, which is why it does not show there. This is
   the most load-bearing open item on this plan after guest processes: "amd64 runs stock 32-bit
   apps" is the promotion's whole claim, and this is the evidence against it.
   **Status: relayed by fex via control-plane, NOT verified here** — I could not find it in
   this tree's `vibeos-roadmap.md` registry, so it is either on the fex branch or unlanded.
   Confirm against that lane's artifacts before planning around it.

5. **Two amd64 test traps that will masquerade as WoW64 bugs.** The amd64 livecd's system
   volume is a **read-only CD**, so any file-write test returns `ACCESS_DENIED` (0x5) — the
   ladder's own `file32` guest skips its write rung for exactly this reason
   (`tools/fex/hello-compute/File32.c:204-207`). And the system volume is `X:` on the amd64
   livecd where arm64's ramdisk is `C:`, so a hardcoded drive letter passes on one and fails on
   the other (`File32.c:33-34`). Both checked in this tree. Anyone writing amd64 32-bit app
   tests should read that file first.

6. Lower, and latent rather than urgent: the 209 audit-baseline entries
   (dominated by win32k's kernel-graphics-driver surface — `SURFOBJ`/`BRUSHOBJ`/
   `XLATEOBJ`, ~81 entries, unreachable from a user-mode guest — plus
   `PORT_MESSAGE` and `TOKEN_GROUPS`); IOCTL embedded-pointer buffers (task #42);
   guest `int3` dispatch.

---

## What deprecating the i386 kernel retires

Stated with the caveat above: the i386 *user-mode* build stays.

- **The last nightly cell** (subject to the kernel/user split experiment).
  `gcc.cmake:638` — i386 is the only arch on `-Z build-std` plus a custom
  `i686-kernel.json`, because `rustc-abi: x86-softfloat` has no builtin triple.
- The `-C target-feature=-sse,-sse2` softfloat dance repeated across 40+ driver
  CMakeLists.
- The x87 / FDIV-errata debt in the CRT: 18 i386 FDIV fixups in asm, 16 `_CI*`
  x87 shims, 17 SEH/setjmp asm plus `except_i386`, 6 `i386/inout.s` port I/O.
  `rust-port-status.md` counts 13 i386-only files / 772 lines existing solely to
  serve x86-32.
- `__stdcall` name decoration — the ABI class `#[no_mangle]` structurally cannot
  spell (`rust-port-status.md:314`) — and the thiscall-in-`ECX` / `retl $N` C++
  ABI arm, which is x86-only.

Note how much of that list is *kernel* debt and how little is user-mode. That is
the same split as the finding at the top, and it is the main reason this is worth
doing.

---

## Sequencing

1. **Ladder into the gate.** Cheap, and it is the measuring instrument for
   everything below it. `gate._smoke_once` already forwards an `extra` list and
   `gate.py:803` already builds a `smoke_extra`, so the wiring is small. The real
   work is the staging policy, not the flag: guests stage only when the sibling
   `build-i386` / `build-fex-hello` trees exist, so a gate that demands the ladder
   unconditionally will report `LADDER_NOT_STAGED` on any lane without them. It
   has to distinguish "the ladder regressed" from "this worktree never staged a
   guest" and stay silent on the latter — the same discipline `ladder.py` already
   applies inside a single run, lifted to gate policy.
2. **Measure guest `CreateProcess`** on today's tree. Gate item between "runs
   applets" and "runs an installer".
3. **Registry redirection.** A silent wrong answer outranks a missing feature.
4. The kernel/user cargo-cell split experiment — independent of the above, and
   the only item here that touches the toolchain.

## Ownership

Items 2 and 3 are `wow64-amd64` territory and that lane is moving fast (five
commits in two days). `research` takes item 1, which is gate/verdict
instrumentation and its natural shape. Item 4 should be offered to whoever owns
`stable-toolchain-plan.md` rather than grabbed. Coordinate before starting 2 or 3.
