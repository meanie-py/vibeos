# Delay-Import for llvm-mingw — Roadmap & Status

> **ARCHIVED 2026-07-10** — parked 2026-06-21, closed 2026-07-10 — the re-test unlock (blocker family fixed) is in the ledger. Outstanding items live in
> `docs/product/vibeos-roadmap.md` § "Check-back ledger" / § "Known bugs & flakes".


**Last updated:** 2026-07-10 (doc audit — see the re-test note below).
**State:** ✅ generator built, ×3 build-green, structurally proven · ⛔ real delay-load triggers an
i386 boot regression · 🅿️ **parked opt-in behind `VIBEOS_ENABLE_DELAYLOAD` (default OFF)**.
**Mechanism/design:** `delay-import-llvm-mingw-design.md`. **This doc:** where it stands + how to finish.

> **🔓 2026-07-10 — the blocker is probably already fixed; re-testing is cheap.** The 0x1E's
> crash face (`KeSignalGateBoostPriority: BAD gate …`) is the **§9.24 pushlock-class face**: the
> "gate" is the `WakeGate` field of a *stack-resident* `_EX_PUSH_LOCK_WAIT_BLOCK`, not session
> pool, and that whole bug family was root-caused and fixed in the §9.13 WS-pushlock work
> (2026-07-05; field manual §9.13/§9.24 — §9.24 explicitly reframed this exact face and stopped
> reproducing it). Next step for whoever picks this up: build with `VIBEOS_ENABLE_DELAYLOAD=ON`
> on today's master and gate i386 — decent odds it just passes now. If it still reproduces,
> §9.24's tool guidance applies (GDB watchpoint on the wait-block `WakeGate.Type` byte, NOT
> special pool).

---

## TL;DR

llvm-dlltool lacks GNU dlltool's `--output-delaylib`, so vibeos' `add_delay_importlibs()` produced
**empty `_delayed.a` stubs on all 3 arches** → everything marked "delay-load" fell back to a
**regular import, loaded eagerly at startup. Delay-load was never functional in this OS.** We built
a drop-in replacement (`sdk/tools/gen-delaylib.py`) that emits real delay-import libraries, wired it
into cmake, and proved it links and takes effect. But **turning it on regresses i386 boot** (a
win32k session-pool corruption, BUGCHECK 0x1E). So it's shipped **off by default** — which is
**byte-for-byte the prior behaviour** (no loss) — and gated behind `VIBEOS_ENABLE_DELAYLOAD` for
anyone continuing the runtime debug.

**"Are we losing something by keeping it off?" — No.** We never had working delay-load; default-off
is the status quo. What we don't yet *gain*: lazy loading, and clean handling of optional/missing
DLLs (the arm64 `libpng` case) without the `arm64_*_stubs.c` hacks. Those stubs stay necessary —
future payoff, not a current regression.

---

## What is done (and green)

- **`sdk/tools/gen-delaylib.py`** — parses an implib `.def` → emits the `.S` (ImgDelayDescr
  descriptor + Import Name Table + delay IAT + per-function thunk + public import stub) → `clang -c`
  → `llvm-ar`. Per-arch ABI (i386 stdcall-decorated + trivial stack thunk; amd64/arm64 bare names +
  volatile-arg-register save/restore around the helper call). Handles forwarders, ordinal-only
  exports, and dup symbols. **130/130 implib `.def`s generate clean.**
- **`sdk/cmake/gcc.cmake`** — `generate_import_lib` calls the generator when
  `VIBEOS_ENABLE_DELAYLOAD` is set (else emits an empty archive = old behaviour);
  `add_delay_importlibs` links `lib<x>_delayed` + the `delayimp` runtime accordingly.
- **Runtime helper** already in-tree: `sdk/lib/delayimp/delayimp.c::__delayLoadHelper2`
  (LoadLibrary + GetProcAddress + patch IAT). No changes needed there (Codex's probes removed).
- **Proven to take effect** (delay-load ON, i386): 50 `_delayed` libs link into the livecd; usp10/
  libpng/secur32 drop off the *regular* import tables; user32 gains ~116 `__delayLoadHelper2` refs.
- **Structure validated host-side** (design §5b spike): the descriptor/INT/IAT walk + the thunk →
  helper → IAT-patch path all resolve correctly. Helper + delay-lib are co-linked into the same DLL
  so `&__ImageBase` is consistent; multiple delay-libs per DLL use file-local symbols (no collision).

---

## The blocker — the i386 0x1E regression

Delay-load ON, i386, **post-`shell ready`** (desktop reached), right as **explorer.exe spawns**:

```
[WIN32K] shell ready
[KERNEL] SwitchDesktop OK
KeSignalGateBoostPriority: BAD gate b10c6cc0 Type=0   Size=227 SignalState=-152872192  caller=00000000
KeSignalGateBoostPriority: BAD gate b10c7ad8 Type=216 Size=12  SignalState=-1324582184 caller=00000000
*** BUGCHECK: 0x0000001E (0xc0000006, 0x805373a6, 0xf6ff8bc0, 0x00000000)
```

BUGCHECK 0x1E = KMODE_EXCEPTION_NOT_HANDLED; args = (exc **0xc0000006 STATUS_IN_PAGE_ERROR**,
faulting **PC 0x805373a6**, faulting **addr 0xf6ff8bc0**, 0). **Non-deterministic** — sometimes one
BAD gate is printed and the boot survives; sometimes 1–2 then the 0x1E + reboot loop.

### Decoded
- The "gates" live at **`0xb10xxxxx` = win32k session pool** (i386 session space ≈ 0xB0000000–
  0xC0000000), with **garbage, run-varying `Type`/`Size`** → corrupted dispatcher objects (a valid
  gate is `Type=7`). `KeSignalGateBoostPriority` is on the guarded-mutex release path; guarded
  mutexes are embedded in win32k session structures.
- The garbage `SignalState` values are **pointer-like, in `0xF6xxxxxx`** — the same high region as
  the fault address `0xf6ff8bc0` (paged pool / system-PTE range).
- ⇒ **Something writes `0xF6xx`-region pointers over session-pool gate objects; code later
  dereferences a clobbered gate → IN_PAGE fault on the bad address.** Classic **win32k session-pool
  overrun / wild-pointer write / use-after-free**, surfacing as a corrupted gate.

### Evidence delay-load is causal (A/B, host-quiet, serialized)
| Build | Result |
|---|---|
| delay-load **ON** | reproducible BAD gate + BUGCHECK 0x1E post-`shell ready` (runs `…-233131`, `…-233917`) |
| delay-load **OFF** (`VIBEOS_ENABLE_DELAYLOAD` unset) | **3/3 clean** host-quiet boots, ~108–110s, `shell ready`, 0 BAD gate, 0 bugcheck (`…-235844`, `…-000028`, `…-000213`) |

The crash tracks the flag, not the tree.

### Ruled out (don't re-spend)
- **Branch base lag** — base is current mainline (0 behind).
- **A plain thunk/relocation bug** — RVA math audited against `delayimp.c` + spike-validated.
- **Wave-1/2 user32 `.rs` alone** — they run in *both* A/B arms; the OFF arm is clean. (They could
  still *contribute* if the mechanism is "a control's window-proc fires a delay-load inside a win32k
  callback" — see H1.)
- **spoolsv→spoolss eager** (Codex tried making it eager) — still reproduced. Not that dep.

### Open hypotheses (ranked)
1. **Reentrant win32k entry via a delay-load fired inside a kernel→user callback.** A window proc
   (incl. the new Rust controls) running under `KeUserModeCallback` calls a now-lazy function →
   `LoadLibraryA` at a nested point → re-enters win32k while the outer callback holds session
   locks/a gate → session-pool corruption. Best fit for "overrun as explorer builds its controls."
2. **DUMMY_PSEH + `RaiseException` on a delay-load *failure*** (delayimp.c L118/L145). If any lazy
   `LoadLibrary`/`GetProcAddress` fails, the unhandled SEH fires at a bad moment. Check whether any
   delay-load actually fails at boot (search-path/order difference vs eager).
3. **i386 thunk ESP edge** — single-step one real thunk through to the target; confirm ESP exactly
   restored / args intact (an off-by-4 silently corrupts the usermode caller → malformed syscall).
4. **DllMain load-order** — a DLL whose `DllMain` does global/GUI init now runs later. Low prob.

### Debug recipe
```bash
qemu/build-index.sh i386
RUN=$(qemu/start.sh i386 --gdb)
qemu/wait-for.sh $RUN --pattern "shell ready" --timeout 180
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 180 \
    --then triage-bugcheck --then "bt 60" --then "symbolicate-pc 0x805373a6"
```
`PC 0x805373a6` is **below** the ntoskrnl symbol base → likely HAL/another module; resolve with
`symbolicate-pc` (module-aware), not `addr2sym.sh` (ntoskrnl-only). Instrument
`__delayLoadHelper2` to log `szDll`+`szProcName` on entry/failure and correlate the last delay-load
before the first BAD gate. To catch the *writer*, watchpoint the clobbered gate once its address is
known.

---

## Roadmap to GA (enable by default)

1. **Root-cause the 0x1E** (above). Definition of done: i386 boots clean to `shell ready` with
   `VIBEOS_ENABLE_DELAYLOAD=ON` across ~3–5 host-quiet boots (it's non-det — prove it), then
   amd64 + arm64 confirm green ON.
2. **Flip the default** to delay-load ON (drop the opt-in), ×3-gate.
3. **Delete the workaround stubs** that only exist because delay-load didn't work:
   `arm64_*_stubs.c` (user32→libpng, advapi32→secur32, rpcrt4→ole32, consrv→shell32) + the consrv
   stack-reserve band-aid — the real payoff (esp. arm64 optional-DLL tolerance).
4. **Second front — the cargo link path.** `reactos-link` (`sdk/lib/rust/reactos-link/src/lib.rs`)
   has its *own* import-lib generator + its *own* copy of the delay→regular collapse (the
   `stack_reserve` field at ~L415 is a direct symptom). No current cargo module is a delay-import
   consumer, so the cmake fix is complete for today; when one migrates to cargo, port the fix by
   shelling out to the **same** `gen-delaylib.py` (don't reimplement the `.S` emit in Rust). Design
   §7b. Bonus: real delay-load there obsoletes `stack_reserve` (csrss/consrv could lazily load
   shell32/ole32 on arm64 instead of eager-loading the tree that overflows the 1 MB stack).

---

## Files

- `sdk/tools/gen-delaylib.py` — the generator (new).
- `sdk/cmake/gcc.cmake` — `generate_import_lib` + `add_delay_importlibs`, gated on
  `VIBEOS_ENABLE_DELAYLOAD` (default off) / honours `VIBEOS_NO_DELAYLOAD` (force off).
- `sdk/lib/delayimp/delayimp.c`, `sdk/include/psdk/delayimp.h` — runtime helper + descriptor layout.
- `docs/engineering/delay-import-llvm-mingw-design.md` — mechanism + full design.

## Reproduce / toggle
```bash
# ON (reproduce the bug):
(cd build-i386 && cmake -DVIBEOS_ENABLE_DELAYLOAD=ON .) && ./vibeos-cli build --arch i386 && ./vibeos-cli smoke --arch i386
# OFF (default / status quo):
(cd build-i386 && cmake -UVIBEOS_ENABLE_DELAYLOAD .)   && ./vibeos-cli build --arch i386 && ./vibeos-cli smoke --arch i386
```
**Caveat:** serialize i386 smokes — two concurrent i386 boots on this host → false `HANG_TIMEOUT`
at ~361s (host-quiet ~110s). Confirm `./vibeos-cli ps` is clear before trusting a red.
