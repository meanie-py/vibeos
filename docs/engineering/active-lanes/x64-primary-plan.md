# Promoting amd64 to the primary x86 arch

**Goal: when someone downloads an ISO to try vibeos, it is the amd64 one, and x86
applications run well on it.** Deprecating i386 as a *shipped* arch is the
follow-on prize (a real pile of tech debt retires with it).

**DIRECTION SET 2026-09-13: the 32-bit KERNEL is approved for deprecation.** The
owner's framing, relayed through `control-plane`: *"regarding i386, yeah we'll
need the DLLs and stuff, but we can drop the 32-bit kernel."* That is precisely
the split this document is built around and which the next section derives, so
the reframe below is no longer a finding awaiting a decision — it is the
direction. What retires is the i386 kernel, HAL and boot path; what stays is the
i386 user-mode PE set, for as long as we run x86 apps at all. No date attaches to
it and nothing here proposes ripping it out in one go; see **Sequencing**.

Opened 2026-09-13 by the `research` lane, as a survey. The depth work below was
mostly owned by `wow64-amd64` when this was written; the split has since been
resolved and most of it is now `research`'s — see **Ownership**, which is the
one place that answers it.

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

**The arm64 half of that 9/9 is softer than it reads, as of 2026-09-13.** fex relays two guests
faulting `c0000005` inside FEXCore's code cache on one arm64 boot — `lua32.exe` in
`LookupCache::FindBlock` (ESR 0x92000006) and `file32.exe`, one of OUR nine ladder guests, in
`ContextImpl::InvalidateThreadCachedCodeRange` (ESR 0x92000004). They checked the two things
that would make it an artifact: the processes loaded `libwow64fex.dll` at different bases, each
read from that process's own load line, and `file32` is a parity guest that normally passes.
**One boot, two guests — a strong lead, NOT a repro**, and a second sample was running when this
was relayed; it is recorded here with that caveat attached and must not be quoted without it.
`external/fex` is vendored unmodified — but note that "unmodified" locates the
faulting CODE, not the CAUSE: how we configure, build and wire FEXCore is ours, so an upstream
source file faulting under our workload does not establish an upstream defect. That inference
is the same one being unwound over the x64 question below, and it should not be made twice. Consequence for this plan: nothing in it leans on arm64 WoW64
being solid — amd64's CPU layer is our own `wow64cpu`, no emulator in the picture — but an
arm64 `--ladder` green is a weaker signal than an amd64 one until this is settled, so do not
read the two as interchangeable evidence.

*(This section previously said the opposite, on a control-plane summary that lane has since
retracted and that I recorded under its attribution. The `wow64.c` quote above is checked in
this tree; the amd64-first provenance is fex's, relayed.)*

## The x64-on-arm64 question is NOT this lane's — and nothing here turns on it

Recorded so it is not re-derived, and because a premise briefly travelled through this
document's neighbourhood. It was relayed on 2026-09-13 that there is no x64-on-arm64 path in
the tree — true as an observation: `libwow64fex.dll` is built as the *x86-guest* BTCpu plugin
and there is zero ARM64EC in our own source. The owner challenged the step that was then taken
from it: *"we've messed up if arm64 can't run x64 ... FEX is capable of x64 — it's the only
reason we picked this instead of box64."* He is right that the observation describes **how we
build and wire FEX**, not what FEX can do; nobody had checked which of the two it was before it
was passed on as a limit. The fex lane is auditing `build-fex.sh`, the FEXCore flags and the
BTCpu wiring to separate ours from upstream's.

**This lane does not wait on that answer and is not contingent on it.** The work here is the
marshalling/WoW64 layer, which sits ABOVE the `BTCpu*` contract — the separation this document
and the fex lane drew independently (see the transfer section). Whether arm64 gains an x64
guest path changes what runs on top of the marshaller; it does not change the marshaller. In
the owner's words: *"the research lane is on deprecating the 32-bit kernel by improving WoW64.
It doesn't (shouldn't) rely on FEX, which is the instruction side."* Checked against the ranked
gaps below: none of them names a guest architecture, and none would be written differently
under either answer.

---

## What "primary" means — partly decided 2026-09-13

The headline question is answered: **the 32-bit kernel goes.** See the goal
section for the owner's wording. What that does *not* settle is the gate policy,
which is a separate set of questions and still open:

- Which arch's red blocks a land? (Today all three are peers.)
- What is the default `--arch`? There is deliberately none, and that rule has
  paid for itself. "Primary" must not quietly reintroduce one.
- Which ISO is *the* download, and does the other two staying green remain a
  gate requirement or become a best-effort?

These remain the user's call, and the engineering below is orthogonal to them:
nothing in the ranked gaps changes depending on how they are answered.

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

1. **Guest processes — GAP CLOSED 2026-09-19, both halves.** `CreateProcess`
   from a 32-bit guest, and cross-process `CreateRemoteThread`. Every installer
   and every app that shells out hits this. `wow64-plan.md` calls it "the largest
   remaining hole" and names the CSR-LPC-32 layer as its prerequisite — which
   landed. Its status was therefore genuinely **unknown**, not known-broken, so
   measuring it on today's tree was the first move rather than a design question.

   **The measurement now exists.** `tools/fex/hello-compute/Spawn32.c` is an
   eleventh guest that self-spawns (same .exe plus one argument, so no second
   binary can go unstaged) and reports five rungs: `CreateProcessW`
   (CREATE_SUSPENDED), `IsWow64Process` on the child, `ResumeThread` across the
   process boundary, the child reaching its exit, and the exit code surviving.

   **It is DIAGNOSTIC, not a parity guest**, and that distinction is new
   machinery (`ladder.DIAGNOSTIC`). A guest that measures a gap we already know
   is open must not red every lane's gate — the fleet's answer to a check that
   reds for a known reason is to disable the check. So `ladder.check` never fails
   on it and `ladder.summary` prints its code; when it comes back `0x600D` the
   summary says so **in words** and tells the reader to promote it into `LADDER`,
   where a regression can red it again. The grading split is not a staging split:
   `staged_guests()` still includes it, because CMake's ladder list is
   all-or-nothing and an unbuilt diagnostic guest would cost every parity guest
   its coverage.

   **MEASURED 2026-09-19 on amd64, and the prediction below was WRONG — the
   real answer is worse and more interesting.** A 32-bit guest calling
   `CreateProcessW` does not get a clean refusal. `NtCreateProcessEx`
   **SUCCEEDS**, `MmCreatePeb` builds the child's WoW64 i386 PEB32, two
   `NtWriteVirtualMemory` calls put the process parameters into it — and then
   the kernel takes **BUGCHECK 0x1E (`c0000005`)**. All ten parity guests PASS
   on the same boot; the machine dies only when spawn32 runs.

   Symbolicated against `ntkrnlmp.exe` — **not** `ntoskrnl.exe`, which is the
   trap the roadmap registry already documents, and which cost me one wrong
   answer before I caught it (`ntoskrnl.exe` resolved the faulting PC to
   `NtShutdownSystem`, a plausible-looking symbol from an image the machine is
   not running):

        faulting PC   ProbeForWrite         ex-rust/src/exintrin.rs:267
        caller        MiDoPoolCopy          ntoskrnl-rust/src/ARM3/virtual.rs:987

   **Root cause, confirmed against the C rather than inferred.**
   `ntoskrnl/mm/ARM3/virtual.c:876` wraps both probes in
   `_SEH2_TRY { … } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status =
   _SEH2_GetExceptionCode(); }`. The Rust translation kept the body and
   **dropped the handler**, at four probe sites across `MiDoMappedCopy` and
   `MiDoPoolCopy`. So `ProbeForWrite` raises via `ExRaiseAccessViolation()`,
   nothing catches it, and KMODE_EXCEPTION_NOT_HANDLED follows. The tell is
   visible in the Rust alone, without the C beside it: `_FailedInProbe` is set
   to 1, the probe is called, it is set back to 0, and a later `if Status < 0 {
   if _FailedInProbe != 0 { … } }` recovery branch tests a combination nothing
   between those lines can now produce. **A recovery path that has become
   unreachable is the fingerprint of a deleted handler.**

   **This is NOT a WoW64 bug and the entry is filed here only because spawn32
   is what found it.** Any caller of `NtWriteVirtualMemory` into another
   process with a bad target address and `PreviousMode != KernelMode` reaches
   the same sites; a 32-bit guest was merely the first thing in the tree
   to do it. The sanctioned fix is the validate-first `probe.rs`
   (`Result<T, NTSTATUS>`) idiom that `dummy-pseh-verdict.md` already prescribes
   and that SE and OB have migrated to — an unfinished migration, not a new
   mechanism. Registered in the roadmap's bug registry; owned there, not here.

   **FIXED 2026-09-19, and the measurement moved on.** All four probe sites and all three
   copies in `MiDoMappedCopy`/`MiDoPoolCopy` now use `ntkrnl_sys::probe`
   (`probe_user_buffer` + `safe_copy_raw`). Converting only the probes was not enough and
   the tree said so rather than my reasoning it out: the same 0x1E came back inside
   `memcpy`, because the C's TRY covered `RtlCopyMemory` too. With both fixed the boot is
   clean and spawn32 reports a verdict instead of killing the machine.

   **The real next blocker, now visible for the first time:**

        NtCreateProcessEx        => 00000000   (child created, PEB32 built)
        NtWriteVirtualMemory     => 00000000
        NtWriteVirtualMemory     => 00000000
        NtWriteVirtualMemory     => 8000000d   STATUS_PARTIAL_COPY
        CreateProcessW           => FALSE, GetLastError 0x12b (ERROR_PARTIAL_COPY)

   **AND THAT IS NOW FIXED TOO — the path reaches the refusal this entry predicted
   at the top, three bugs later.** The failing write was the one kernel32 calls
   "Write the PEB Pointer", into `RemotePeb->ProcessParameters`. `RemotePeb` comes
   from `TEB->NtTib.ArbitraryUserPointer`, which `ps/process.c:866` fills in as
   `CurrentThread->Tcb.Teb->NtTib.ArbitraryUserPointer = Process->Peb`. Two
   compounding mismatches for a 32-bit caller: `KTHREAD.Teb` is the SIXTY-FOUR-bit
   TEB, and the value is the child's 64-bit PEB when a 32-bit parent needs its
   PEB32. The guest was reading a TEB32 slot nothing had ever written.
   `Wow64pPublishRemotePeb32` in `wow64_manual.c` now bridges it — the same shape,
   and the same fix, as the win32k last-error channel already in `wow64.c`, which
   exists because the kernel writes TEB64 and the guest reads TEB32. Today's state:

        NtCreateProcessEx        => 00000000
        NtWriteVirtualMemory x4  => 00000000      (all four now succeed)
        NtCreateThread           => c0000002      REFUSED, cross-process (task #64)
        CreateProcessW           => FALSE, GetLastError 1 (ERROR_INVALID_FUNCTION)

   **So the original traced prediction was right about the DESTINATION and wrong
   about it being NEXT** — three defects stood in front of it: a dropped
   `_SEH2_EXCEPT` (bugcheck), a wrong `STATUS_PARTIAL_COPY` constant hiding in the
   dead error path, and this missing TEB32 channel. Each was invisible until the
   one before it was fixed. That is the general shape worth keeping: **a stack of
   bugs is discovered strictly in order, and "the next blocker" is only ever
   knowable one at a time.**

   **Task #64 turned out to be TWO tasks, and the cheap half is DONE (2026-09-19).**
   The thunk refused every `ProcessHandle != -1` on one reason — the target's
   wow64.dll must run a per-thread entry and `Wow64ProcessPendingCrossProcessItems`
   is empty — but that reason only covers `CreateRemoteThread` into a **running**
   process. The **initial** thread of a brand-new child needs no channel at all,
   and the tree says so in two places: the child's `Wow64LdrpInitialize` takes the
   guest entry from `PEB32->ImageBaseAddress` and the image's own NT headers,
   never from a CONTEXT the parent passed (`wow64.c`), and it is reached from
   `LdrpInitializeProcess` on the strength of `ProcessWow64Information` on ITSELF
   (`ntdll-rust/src/ldr/ldrinit.rs:2379`), so it does not care who created the
   thread. The standing existence proof is that winlogon spawns all ten ladder
   guests exactly this way; the only difference for a 32-bit parent is that its
   `NtCreateThread` passes through the thunk and a native parent's does not.

   `Wow64pCreateForeignInitialThread` (`wow64_manual.c`) now takes that half:
   `RtlCreateUserThread` against the foreign handle, which does the stack
   allocation and host CONTEXT build in the target. The guest's
   CONTEXT32/INITIAL_TEB32 are deliberately ignored — they describe a 32-bit start
   the child recomputes for itself. Two conditions gate it, both load-bearing
   rather than defensive: the target must be a WoW64 process (a 32-bit parent
   spawning a **64-bit** child is a different, still-refused case), and the
   target's `PEB->Ldr` must be NULL, which is the kernel's word for "has not run
   user code yet" and exactly the precondition the argument above rests on. A
   running target falls through to the old refusal, which is the
   `CreateRemoteThread` half and still owed.

   **Measured the same day, and it moved:**

        [WOW64] NtCreateThread: cross-process INITIAL thread pid=2ec tid=2f0 suspended=1
        [WOW64] NtCreateThread(0x37) [manual] => 00000000

   All ten parity guests PASS on the same boot, no bugcheck.

   **The FOURTH defect in the stack, and the harness named it itself:**

        [WOW64] LPC: api 00010000 has NO payload marshaller — body copied verbatim,
                     fields past CsrCaptureData may be wrong
        [WOW64] LPC request: Api=00010000 DataLen=156
        [WOW64] LPC reply: api Status=c0000008          STATUS_INVALID_HANDLE
        [spawn32] rung1 CreateProcessW FAILED, GetLastError 0x6   ERROR_INVALID_HANDLE

   `0x00010000` is `BasepCreateProcess`, and `BASE_CREATE_PROCESS`
   (`subsys/win/basemsg.h:84`) opens with `HANDLE ProcessHandle; HANDLE
   ThreadHandle; CLIENT_ID ClientId;` — four pointer-width fields before anything
   else, so the verbatim body copy hands basesrv handles assembled from the wrong
   words. It is the identical failure `WOW64_CSRAPI_BASEP_CREATETHREAD` was written
   for, one struct larger, and the "no payload marshaller" warning existing at all
   is why it cost one boot instead of a hunt. Note this is a marshaller, not thread
   creation: **task #64's remaining half is `CreateRemoteThread`, and this is a
   separate, better-understood piece of work in front of it.**

   **DONE 2026-09-19, and it cost far less than this paragraph first estimated.** The
   estimate assumed the embedded `BASE_SXS_CREATEPROCESS_MSG` had to be widened — two
   `SIZE_T`s, three pointers, four `UNICODE_STRING`s whose `Buffer`s are guest pointers.
   It does not: `BaseSrvCreateProcess` (`smss-csrss-rust/src/basesrv/proc.rs:97`) never
   reads `Sxs` at all, because the SxS work belongs to a different API
   (`BaseSrvSxsCreateActivationContext`). Reading the server before budgeting the client
   turned thirty fields into eleven. `Sxs` is zeroed rather than left as whatever the
   verbatim copy put there, and a non-zero guest `Sxs.Flags` now prints, so the day the
   server starts reading it that announces itself instead of going quiet.

   Offsets were **measured, not derived**, per the note this paragraph used to carry, and
   the measurement had a control worth reusing: the structs were transcribed into a
   standalone `offsetof` dumper built with the llvm-mingw i686 and x86_64 targets, and
   its `sizeof` line has to reproduce 140/96 and 232/160 — the numbers
   `ntdll-sys/src/size_check_{i386,amd64}.rs` already asserts. A bad transcription fails
   as a size mismatch instead of as quietly wrong offsets. (The hand-derivation done
   first agreed, but agreement found afterwards is not what made it trustworthy.)

   One field needs naming because it does not look like what it is: `ProcessHandle`
   carries up to two FLAG bits in its low bits (kernel32's `AddToHandle`/
   `RemoveFromHandle`), which the server reads back as `Handle & 3` before masking.
   Zero-extension preserves them, which is the only reason the naive widening is correct.

   **Measured after the fix — rungs 1 through 4 PASS, all ten parity guests still PASS:**

        [spawn32] rung1 CreateProcessW OK
        [spawn32] child pid 0x000002e4
        [spawn32] rung2 child is a 32-bit guest OK
        [spawn32] rung3 ResumeThread OK
        [spawn32] rung4 child exited OK
        [spawn32] rung5 wrong child exit code 0x00000080      verdict 0xBAE5

   So cross-process `ResumeThread` works (rung 3 was written to catch a partial fix, and
   it did not fire), and the child really is a 32-bit guest by the SYSTEM's reckoning.

**The FIFTH defect, and it is the structurally interesting one: a 32-bit parent can
only fill the child's PEB32, but the child's SIXTY-FOUR-bit loader runs first.** The
child dies before printing a single loader line:

        [KI-EXC] spawn32.exe code=0xc0000005 rip=000007ffb7004c4f
        [EXIT-THR] image=spawn32.exe exit_status=0x80

`ntdll.dll` is based at `0x7FFB7000000`, so that PC symbolicates to
`LdrpInitializeProcess`, `ldrinit.rs:1827` — the first line of the function that
dereferences anything:

        ImagePath = (*(*Peb).ProcessParameters).ImagePathName.Buffer;

`PEB64->ProcessParameters` is NULL. The asymmetry is the whole bug. When a NATIVE
parent spawns an i386 child, its 64-bit kernel32 writes 64-bit process parameters into
PEB64, the child's loader finds them, and `wow64.dll` then DERIVES the 32-bit copy —
that is the `[FEX-WOW64C] PEB32->ProcessParameters built at ...` line every working
guest prints. A 32-bit parent cannot do the first step: it builds a 32-bit
`RTL_USER_PROCESS_PARAMETERS` and, since `Wow64pPublishRemotePeb32`, correctly writes
it into the child's PEB32. Nothing fills PEB64, and PEB64 is what runs first.

Note what this says about the earlier fix: `Wow64pPublishRemotePeb32` was right about
WHERE the guest's kernel32 should write, and that is exactly why this defect was
invisible until now — it was hidden behind two failures that stopped the child before
its loader ever ran.

**FIXED, and `spawn32` reports `0x600D` — five rungs, five defects, gap closed.**
`Wow64pPublishPeb64Params` widens the block the guest wrote and points PEB64 at it,
before the thread exists, because once it is running the loader has already
dereferenced that pointer. A failure there fails the creation rather than starting a
child that cannot survive its own first instruction.

It copies no string bytes at all, which is the part worth keeping. Every `Buffer` the
guest built already lives in the CHILD's address space below 2GB, and a 64-bit pointer
addresses those characters perfectly well — so the 64-bit strings point at the very
same memory and only the fixed header is rebuilt. The environment block is shared the
same way. The output is marked NORMALIZED with absolute pointers, which is what makes
the denormalized input free: a denormalized `Buffer` is an offset from the block base,
so resolving it is one addition done here instead of by every reader. Which fields that
applies to was read out of `rtl-ppb`'s `RtlDeNormalizeProcessParams` rather than
guessed — the eight `UNICODE_STRING`s, and deliberately not `Environment`, which is a
separate allocation. The live boot takes the denormalized path.

        [WOW64] PEB64->ProcessParameters published at 0000000000160000
                (widened from PEB32 block 00050000, strings denormalized)
        [spawn32] rung1 CreateProcessW OK          [spawn32] rung4 child exited OK
        [spawn32] rung2 child is a 32-bit guest OK [spawn32] rung5 child exit code intact OK
        [spawn32] rung3 ResumeThread OK            [spawn32] verdict 0x0000600d

**The `offsetof`-with-a-control method paid for itself here rather than in theory.**
Transcribing `RTL_USER_PROCESS_PARAMETERS` from `wine/winternl.h` gave `sizeof`
676/1040 against the 656/1008 that `ntdll-sys/src/size_check_{i386,amd64}.rs` asserts:
that header carries five trailing fields (`EnvironmentSize`, `EnvironmentVersion`,
`PackageDependencyData`, `ProcessGroupId`, `LoaderThreads`) which are not in the struct
this tree builds against. **Nothing above `RuntimeData` moved**, so every field offset a
derivation would have produced was correct and the BLOCK SIZE was wrong — the one error
a field-by-field review does not catch, caught by a line of output that had to match a
number already in the tree.

**`spawn32` PROMOTED from `DIAGNOSTIC` to `LADDER` the same day**, which is what its own
contract says to do when a diagnostic guest starts passing: it measured an open gap, the
gap is closed, and leaving it ungraded would mean a regression in the thing it proved
could pass unnoticed. Three consequences, all landed: it is built unconditionally
(both reasons for the opt-in are spent — the bugcheck was the dropped `_SEH2_EXCEPT`,
and "red by design" ended when it went green), it joined the all-or-nothing
`_wow64_ladder` CMake set, and **its launch order INVERTED** — it used to run dead last
because it could kill the machine, and a graded guest must now run BEFORE `sqlite32`,
which carries the registered BUGCHECK, or an sqlite32 crash silently costs spawn32 its
verdict. That is the same rule that already cost lua32 a verdict once. Ladder now reports
**11/11 guests PASS** on amd64.

The `DIAGNOSTIC` table is empty again, and its tests were re-pointed at a synthetic
guest rather than deleted — the machinery is what the next guest aimed at a known-open
gap will need, and coverage deleted because a table happens to be empty rots
unexercised until the moment someone depends on it.

**`CreateRemoteThread` — the other half of task #64 — CLOSED the same night, and it
did not need the channel either.** The refusal covered both callers on the harder
one's reason, and splitting them showed the hard one was smaller than advertised. Two
things make the in-process path unusable through a foreign handle, and only one is
interesting:

- The SEED is a pointer into the creator's heap. Allocate it in the target instead;
  `WOW64_THREAD_SEED` gained a `Remote` flag so the consumer frees it with
  `NtFreeVirtualMemory` rather than corrupting the heap with `RtlFreeHeap`.
- The START ROUTINE is a code address in the creator's copy of `wow64.dll`.

**That second one is where assuming would have cost a night.** "It is a system DLL,
it is always at the same base" is the comfortable reading, and it is FALSE here —
measured, on the live boot: `wow64.dll delta NONZERO - relocated`. The two processes
load it at different addresses, so the naive version starts a thread at whatever
happens to live at that address in the target. `Wow64CreateForeignGuestThread` walks
the TARGET's loader list for its own `wow64.dll` and translates
`Wow64pGuestThreadStart` through the measured delta; equal bases would make the delta
zero and cost only the walk. Loader-walk offsets measured with the usual control
(`PEB_LDR_DATA` 72, `LDR_DATA_TABLE_ENTRY` 152 on amd64).

**And that is why `Wow64ProcessPendingCrossProcessItems` is still empty and does not
need to be filled.** Win10 keeps that work list so the TARGET can do per-thread setup
on its own schedule; here the new thread's start routine IS the target's
`Wow64pGuestThreadStart`, so the setup happens in the target, on the new thread, by
the target's own code. The property the list exists to provide is obtained by
construction. It would start to matter only if the target had to act at a moment it
chooses — guest-memory invalidation after a foreign `WriteProcessMemory` being the
obvious candidate, which a JIT CPU layer would need and the native one does not.

**`spawn32` grew rungs 6 and 7** — `CreateRemoteThread` into the running child, and
the injected thread running to completion. Two independent observers of the same
event: the thread's own exit code, and the child noticing a named event it set.
Rung 7 caught a race in the TEST rather than the feature on its first run — the child
called `ExitProcess` the moment it was signalled, overwriting the injected thread's
`0x0DEF` with the process code, so rung 7 read the child's verdict back to itself. A
thread's exit code only exists while its process does; a third event now holds the
child open until the parent has collected rung 7.

        [FEX-WOW64C] foreign guest thread tid=2ec eip=7c4e112c esp=0050fffc
                     start=00000000001A1C90 (wow64.dll delta NONZERO - relocated)
        [spawn32] rung6 CreateRemoteThread OK, tid 0x000002ec
        [spawn32] remote thread: running INSIDE the child
        [spawn32] child: saw the injected thread's signal
        [spawn32] rung7 remote thread ran to completion OK
        [spawn32] verdict 0x0000600d          ladder: 11/11 guests PASS

**Seven rungs, seven defects-or-designs, one gap.** Nothing is owed on guest processes.

   *(Superseded, kept as a dated waypoint: while the `STATUS_PARTIAL_COPY` write was
   still failing I wrote here that the next question was "why the child's parameter
   block is not mapped where the parent is writing it." It was not a mapping fault at
   all — it was the missing TEB32 `RemotePeb` channel described above, and the block was
   always mapped. Do not follow that lead; it is closed.)*

   **A SECOND translation bug, found only because the first fix made the status
   visible — and it corrects something I wrote here an hour ago.** The first clean run
   reported `c000009b` and `ERROR_PATH_NOT_FOUND`, and I wrote that kernel32 maps
   partial-copy misleadingly to a path error. **That was wrong.** `STATUS_PARTIAL_COPY`
   is `0x8000000D`; `0xC000009B` is `STATUS_DFS_EXIT_PATH_FOUND`, so the Win32 mapping
   was doing its job correctly on a status that was itself a lie. All three
   "Return partial copy" sites in `virtual.rs` carried the wrong constant. Note where
   the wrong value sits: `STATUS_INSUFFICIENT_RESOURCES` is `0xC000009A`, used a few
   lines earlier in the same function — the bad constant is its numeric neighbour, which
   is what a plausible-looking invention looks like. Fixed by adding a named
   `STATUS_PARTIAL_COPY` to `ntkrnl-sys::base` and using it; verified the guest now
   reports `ERROR_PARTIAL_COPY` (299). **The general lesson: a wrong status code hides
   behind a correct-looking error message, and the first thing a restored error path
   buys you is the ability to see the next lie.**

   **Consequence for this lane, and a correction to the design above:** a
   diagnostic guest that BUGCHECKS cannot be carried by "report, never red".
   That promise covers a bad exit code and is worth exactly nothing against a
   dead machine, which reds the boot for every lane in the fleet. So spawn32 is
   **opt-in** — built only under `VIBEOS_WOW64_SPAWN=1`, staged `if(EXISTS)`
   rather than in the unconditional `_wow64_ladder`, and run by winlogon **dead
   last**, after even the stress guests, so that when it does take the machine
   down every other verdict has already been reported. That last rule is
   sqlite32's, and spawn32 is a harder case than sqlite32, not a softer one.

   **The original traced prediction, kept because being wrong is the point.**
   `CreateProcess` builds a child in two steps — `NtCreateProcessEx`, then
   `NtCreateThread` for the initial thread, passing the CHILD's `ProcessHandle`
   (`kernel32-rust/src/proc_create.rs`, the "Create the Kernel Thread Object"
   call). `wow64_manual.c`'s `NtCreateThread` thunk refuses any `ProcessHandle`
   that is not `-1`, deliberately: a cross-process thread has no per-thread entry
   in the target and nobody would initialize it. So rung 1 is predicted to fail
   and `CreateProcess` to report what its own source calls "a process is not
   allowed to exist without a main thread". **It was traced from source, NOT
   observed** — written down so the first boot could confirm or correct it, and
   the boot corrected it. The error was assuming that a refusal I had read in
   the thunk would be the FIRST thing the path hit; the kernel died two
   syscalls earlier than the code I was reading. Reading one layer and
   predicting the whole stack is the mistake, and it is an easy one to make
   when the layer you read says something definite.

2. **Registry redirection — DONE.** The wow64-amd64
   lane landed it: `878e0ac0b67` (E24) puts Wow6432Node redirection in the **CM parse**, not in
   wow64.dll, because registry opens are overwhelmingly RELATIVE — advapi32 caches a handle to
   `\REGISTRY\MACHINE` and passes RootDirectory + a subpath, so a user-mode rewrite would need
   a handle-to-path syscall on every open. `911b1e1d82b` (E21) fixed `NtOpenKey` denying every
   `KEY_WOW64_32KEY` open outright. This lane closed the runtime hole with `reg32.exe`, the
   tenth ladder guest, which `878e0ac0b67` structurally could not test from outside.

   **The last residue is closed too, re-verified on this tree 2026-09-19.** This entry used to
   say "one function left": two live `Wow64 not yet supported` sites in `RegDeleteKeyExW`,
   dropping an explicit `KEY_WOW64_32KEY`/`64KEY` on a DELETE and so deleting from the native
   hive when the caller had asked for the other view. `reg.rs:1073-1081` now passes the view
   selectors straight into `NtOpenKey`, which is all that was needed once E21 stopped refusing
   them and CM started honouring them. The only `Wow64 not yet supported` text left in
   advapi32 is the comment recording that fix, plus two commented-out `RegLoadMUIString`
   lines. **This entry has now called the feature an open silent bug and then a nearly-closed
   one; both were true when written and neither is now.** Original scoping kept below, because
   it is why it mattered:

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
   attached, and it is not answered here. **It is a USER decision, not a lane's.**

   **ANSWERED 2026-09-13: default-on.** The owner, relayed through
   `control-plane`: *"Yes, obviously, this was not even a question that we should
   have paused on"*, and on the `VIBEOS_FEX=1` gate itself — *"if even the fex
   lane is forgetting to stage and smoke fex then the build env-var is just a way
   for us to embarrass ourselves."* So the env-var gate in `qemu/mkramdisk.sh`
   comes out and default arm64 media stages the CPU layer. **This lane
   recommended the opposite** (keep it opt-in, on the fleet-wide cost) and was
   overruled on judgment rather than on numbers — the 84.7s drain and the ~+55%
   arm64 gate wall-clock below are what made it a real decision instead of a
   guess, and they are still the cost being accepted. Implementation is not
   assumed to be this lane's; `mkramdisk.sh` and `build-fex.sh` are the fex
   lane's files and that lane is auditing them right now.

   **Both unmeasured numbers are now measured (2026-09-19), and they settle it:
   `tools/fex/build-fex.sh` takes 68 seconds and `build-fex-arm64` occupies 25M.**
   This paragraph asked for them rather than guessing, and the guess would have been
   high — the staging debate was conducted against a cost that turns out to be one
   minute and a rounding error of disk, per worktree, once.

   What was already known: FEX source is **vendored in-tree** (`external/fex`, 58M),
   so this is not a fetch, licensing or availability problem — it is a per-worktree
   *build* cost. **Correction:** this said "exactly one worktree carries a
   `build-fex-arm64` today", meaning the fex lane's. Measured across the fleet
   2026-09-19: `vibeos-research` is the only one, and `vibeos-fex` has none. The recurring cost once staged is the ~85s ladder drain per arm64 gate, already
   measured. **Unmeasured, and deliberately not guessed here:** how long `tools/fex/build-fex.sh`
   takes and how much disk `build-fex-arm64` occupies — the only tree that has one belongs to
   another lane, and reading a sibling's generated artifacts is exactly what the fleet-isolation
   rule forbids. Whoever decides this should get those two numbers from the fex lane rather than
   from an estimate.

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

4. **Stock 32-bit GUI apps cascade into an execute-at-NULL fault — 58 faults each, and NOT
   established as amd64-only.** notepad, mspaint and wordpad each take 58 faults where every
   other image in the same boot takes 0 or 1: one fault executing from a STACK address, then 57
   at `eip=00000000` with `rw=8` (instruction fetch, not a data deref) with every GPR zeroed,
   each dispatched to 32-bit SEH at a deeper `esp` — a re-faulting cascade that consumes stack
   rather than one bad call. Specific to the stock GUI apps, not general WoW64 breakage: the
   CSR/LPC ladder is green in the same boot and `[WOW64CB] WINDOWPROC` callbacks dispatch and
   return correctly.

   **Now verified on master** (`06ec737b452`, registry entry), which upgrades this from the
   relayed claim it was — and the registry carries a correction this plan had wrong. **I called
   it amd64-ONLY; that is not established.** `sqlite32.exe` took the same signature once on
   arm64 (`eip=00000000`, `rw=8`, crashing before its first print) and did NOT reproduce on an
   immediate re-run. The reason arm64 does not show the *stock-app* cascade is simply that it
   never launches those applets — absence of a trigger, not absence of the bug. So it is
   weaker evidence against amd64-as-primary than I filed it as: a WoW64 defect this plan should
   track, not an amd64 disqualifier.

   *Worth noting as a pattern in this document: three claims filed here as amd64-specific have
   since proved otherwise — the marshalling provenance, the nightly cell, and now this. An
   "amd64 is primary" frame makes amd64-only the comfortable reading of any new finding.*

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

0. **Retiring the i386 kernel is now the direction, not a proposal** (2026-09-13).
   It does not reorder the list below: the ladder is still what tells you whether
   WoW64 held, and WoW64 holding is the precondition for the i386 kernel being
   safe to drop. Sequence the retirement *behind* the gaps, not ahead of them —
   the i386 kernel is the fallback that makes a WoW64 regression survivable today.
1. **Ladder into the gate — DONE 2026-09-13, all three arches.** Cheap, and it is
   the measuring instrument for everything below it. `gate._smoke_once` already forwards an `extra` list and
   `gate.py:803` already builds a `smoke_extra`, so the wiring is small. The real
   work is the staging policy, not the flag: guests stage only when the sibling
   `build-i386` / `build-fex-hello` trees exist, so a gate that demands the ladder
   unconditionally will report `LADDER_NOT_STAGED` on any lane without them. It
   has to distinguish "the ladder regressed" from "this worktree never staged a
   guest" and stay silent on the latter — the same discipline `ladder.py` already
   applies inside a single run, lifted to gate policy. That is what shipped:
   `ladder.gate_decision` opts each arch in from its own media and records a loud
   `skip` with a reason when it declines, so an unstaged tree is silent rather
   than red. With FEX staging going default-on (above), arm64 stops being the
   conditional case.
2. **Measure guest `CreateProcess`** on today's tree. Gate item between "runs
   applets" and "runs an installer".
3. **Registry redirection.** A silent wrong answer outranks a missing feature.
4. The kernel/user cargo-cell split experiment — independent of the above, and
   the only item here that touches the toolchain.

## Ownership

**RESOLVED 2026-09-19 — this paragraph used to hand items 2 and 3 to
`wow64-amd64` and it is out of date.** The split was settled the other way:
`research` takes item 1 (done), item 3 registry redirection (done — `reg32.exe`,
the tenth guest, closes the runtime hole that `878e0ac0b67` structurally could
not test from outside) and item 2 guest `CreateProcess` (in progress —
`spawn32.exe`). Item 4 should still be offered to whoever owns
`stable-toolchain-plan.md` rather than grabbed.

Left standing rather than deleted, because the reason matters: the coordination
cost of "coordinate before starting" was real, and it was paid twice over the
same two items before anyone wrote the answer down.
