# Fable lane: FEX-Emu on vibeos — run x86 Windows binaries on ARM64

You're in a dedicated worktree (`vibeos-fex`, branch `fex-arm`, from `origin/master`). This is
the big swing. Opus rewrote the entire IP stack from scratch as Rust (`netstack-rust` +
`docs/engineering/smoltcp-netstack-design.md`). **This is an order of magnitude larger.** We
want to see how far a frontier model can push it — go big, think holistically, solve the
*class* not the instance. There is no "just do the minimum" here; there's "how far can you
get, done right, with smoke green at every checkpoint."

**The goal:** an x86/x64 Windows binary executing on ARM64 vibeos via **FEXCore** (Valve's
production x86→ARM64 JIT, the one behind Steam on ARM). Full vision + sequencing:
`$W/docs/product/vibeos-roadmap.md` §8 ("Homegrown Rosetta — WoW64 + FEX/TCG"). Read it, then
improve on it — the plan below corrects two things §8 got wrong (see "Reality check").

**Path discipline:** `W=$(git rev-parse --show-toplevel)`, prefix every path with `$W`. You
share one `.git` with ~8 live worktrees — never bare-path main, only stage what you author,
**never broad-kill QEMUs** (`pkill -f qemu` SIGKILLs your own agent tree — stop boots by uid
via `./vibeos-cli ps` / the reaper). North Star: `./vibeos-cli smoke --arch arm64` is truth;
read `summary.json`. Gate ×3-green on a clean tree before landing anything.

---

## The two-layer mental model (internalize this first)

FEX and "WoW64 thunking" are **complementary, not the same thing**:

- **FEX = the CPU.** x86/x64 guest instruction stream → FEX IR → ARM64 codegen. It's an
  emulated processor. **Vendored C++ — do NOT translate it to Rust.** It's fast-moving
  upstream; translating it is the frozen-fork anti-pattern the roadmap warns against (§7b).
  We reuse Valve's JIT wholesale.
- **WoW64 layer = the ABI boundary.** The `BTCpu*` plugin contract WoW64 uses to drive the
  emulator (`BTCpuProcessInit` / `BTCpuThreadInit` / `BTCpuSimulate` / …) **plus** the
  per-syscall 32↔64-bit argument marshalers (the "thunks"). When guest x86 code hits a
  syscall, FEX calls *out* through this layer into the native ARM64 NT kernel. **This layer
  is vibeos-authored, and it can be Rust.**

So `x86-on-ARM64 = FEX (vendored) + WoW64-layer (yours)`. The eventual thunk *generator*
(auto-producing marshalers from `ntkrnl-sys` bindgen) is one component of the WoW64 layer —
worth building once the boundary works, but not rung 1.

## Reality check (verified against this tree — trust these over §8)

- ✅ **JIT memory works.** Mm honors `PAGE_EXECUTE_READWRITE` (`sdk/lib/rust/ntoskrnl-rust/
  src/mm/mminit.rs`, `section_pe.rs`). FEX's RWX code buffers via `NtAllocateVirtualMemory`
  are feasible — the #1 potential hard-blocker is green.
- ✅ **Both x86 arches + arm64 already boot to desktop**, and `ntkrnl-sys` bindgen already
  holds every struct layout **per arch** — the exact input a thunk generator needs, for free.
- ❌ **§8 oversells the start.** It claims "ReactOS already has a partial wow64 skeleton
  (wow64.dll/cpu/win)." **It is NOT in this tree** — no wow64 dirs, no `BTCpu*` contract,
  nothing. The WoW64 layer is **from scratch**, and it's roughly as much work as the FEX
  integration itself. Plan for that; don't be surprised by it.

## The one risk to KILL on day one — fault/exception delivery

FEX's whole model leans on delivering guest CPU faults precisely — JIT page faults, guest
SEH, `#GP`/`#PF` → back into the guest's handler. vibeos runs `USE_DUMMY_PSEH`, and user-mode
software-exception dispatch is explicitly caveated. **Before building anything tall, prove
vibeos can deliver a synthesized fault into a controlled user context and have a handler
run.** If it can't, you want to know on day one, not day five.

Strong tailwind: the `dummy-pseh-question` branch JUST landed the kernel-mode half of this —
`requires-uwtable=true` gave the kernel real Win64 unwind info and the first working
kernel-mode SEH catch (`docs/engineering/dummy-pseh-verdict.md`, field manual §9.27). Read it:
it maps vibeos's exception model and the `KiUserExceptionDispatcher` path you'll ride. The
user-mode delivery path FEX needs is the sibling of what that lane just fixed.

## The ladder (each rung is a green checkpoint; climb as high as you can)

1. **Vendor FEXCore + build it as `aarch64-w64-mingw32` PE.** Wine already ships
   `libwow64fex.dll` as a PE via a clean plugin path — follow *that* build, not the Linux
   one. Pick a vendored home (e.g. a new top-level `sdk/lib/fex/` or `dll/win32/wow64fex/`);
   keep it clearly vendored/unmodified so upstream re-vendoring stays trivial.
2. **Minimal platform shim + the fault-delivery proof** (the risk item above). RWX alloc,
   thread/TLS context, exception plumbing. This is the hard, open, design-heavy part — your
   call on structure; think about it as "what host services does FEXCore assume, and what's
   the NT primitive for each."
3. **JIT + execute a static x86 compute blob** on arm64 vibeos, no syscalls — "hello
   compute." Proves decode → IR → codegen → dispatch loop → RWX execution end-to-end.
4. **Wire `BTCpuSimulate` + ONE hand-written thunk** (e.g. `NtWriteFile` → serial, or
   `NtDisplayString`) — "hello world." Proves the WoW64 boundary round-trips. After this, the
   thunk *generator* is the obvious scaling follow-on (bindgen → marshalers).

(TSO — x86 strong ordering on ARM64's weak model — is a *known-solved* FEX concern: it runs
barrier-mode TSO, correct-but-slower, which is exactly right on our QEMU-TCG test env. Don't
spend the window optimizing it.)

## Why this is worth the swing (the strategic payoff)

FEX isn't just "more apps." It's the lever that unlocks **§9 — retiring native i386
entirely.** Once x86 runs via FEX on ARM64 *and* WoW64 on amd64, we stop maintaining the i386
kernel port: 2-level paging, i386 asm, and pointedly **the stdcall/fastcall ABI hazards that
keep biting us** (they vanished the i386 desktop icons and are the prime suspect in the
ex-crate hang). FEX pays back by deleting a whole *class* of the bugs we fight. Aim there.

## Guardrails

- **Vendor, don't translate FEXCore** (above). Author only the vibeos side; that side may be
  Rust (preferred where practical — it's the project goal) but pragmatism wins for the shim.
- If a rung genuinely can't land, **write up exactly where it wedged** (the NT primitive that
  was missing / the fault-delivery behavior observed) — a precise wedge report is a real
  deliverable, not a failure. Route durable findings to `docs/engineering/` (a
  `fex-bringup-*.md` doc), not local memory.
- Land only what gates ×3-green across the arches it touches.

Show us what you've got. This is the marquee capability — have fun with it.
