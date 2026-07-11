# Payload Harness — Declarative App Staging, Autostart & Test Driving

> **ARCHIVED 2026-07-10** — harness shipped (doom+httpd); owed workstreams (A winlogon-hack, B vibeagent, PuTTY) are in the roadmap Check-back ledger. Kept as the as-built design. Outstanding items (if any) live in `docs/product/vibeos-roadmap.md`.


**Status: DESIGN (2026-07-07); step 1 (harness+doom) LANDED 2026-07-07, step 2 (httpd+tcp_hammer, netstack backlog+pended-send fixes) landed 2026-07-08 — see field manual §9.43. Rewrite as as-built once workstreams A/B land.**

**Lane closure (2026-07-10):** the `arm64-interrupts` lane that built this shipped its
core and was closed — payload harness (YAML → media + test interpreter, doom converted),
httpd stress + tcp_hammer oracle (amd64 320/320), netstack TDI listen backlog + pended
sends + arm64 hardening, and the fastfat `FatOpenExistingDirectory` translation (the real
arm64 fix). All ×3-gate-green on master. **Still owed (separate follow-ons, unclaimed):**
Workstream A (delete the winlogon spawn hack in favor of fixed `ProcessStartupItems()` —
small, tidy), Workstream B (vibeagent TCP guest-exec channel — the highest-value reusable
infra), and the PuTTY payload (pure-YAML generalization exam).
Lane: grew out of the Doom foreign-binary quest (`doom-on-vibeos.md`); first
consumers are the httpd stress gate and the PuTTY compat gate.

## Problem

Getting a test/demo app onto vibeos media and running at boot today takes four
hand-rolled, per-app hacks:

1. **Staging is a special case, duplicated per media path.** `VIBEOS_DOOM=1`
   has twin implementations in `qemu/mkramdisk.sh` (arm64 ramdisk) and
   `tools/strip_cd_media.py` (x86 livecd). Every new app would clone both.
2. **Autostart is a winlogon hack.** `TryStartDoom()` / `DoomStaged()` /
   the notepad focus-ordering dance are hardcoded, `#[cfg(target_arch)]`-riddled
   Rust in `winlogon-rust` — a cargo rebuild + full gate cycle per app-list
   change, in a security-critical component that should know nothing about
   demos.
3. **The real Windows mechanism is broken and being routed around.** The
   registry Run-key *data* path works (`hivedef.inf:1931` installs an HKCU Run
   entry for kbswitch.exe), but the *consumer* — explorer's
   `ProcessStartupItems()` — doesn't reliably run on amd64
   (`winlogon-rust/src/winlogon.rs` ~line 915: "shell startup is slow/fragile").
   Every winlogon spawn hack is a workaround wearing that undiagnosed bug.
4. **Test driving is per-app Python.** `test_doom.py` hand-codes the focus
   click, key sequence, settle times and frame-delta oracle. httpd and PuTTY
   would each clone that shape.

The stress-app roadmap (httpd hammer → PuTTY → sshd → …) makes each of these
O(apps). This design makes adding an app **one YAML file**.

## Non-goals

- A second payload disk volume (axis rejected: arm64 boots from a UEFI ramdisk
  with no second-disk story; FastFAT bugs make USB-msstor staging premature;
  media rebuild via `mkramdisk --refresh` is already seconds).
- Replacing the smoke tests. Payload gates are opt-in suites *beside* smoke,
  exactly like `--doom` today.
- General package management inside the guest. Payloads are baked into media
  at build time; runtime delivery is the vibeagent's job later, if ever.

## Design

One declarative spec per app, two consumers (build-time compiler, test-time
interpreter), plus two supporting workstreams (the ProcessStartupItems fix and
the vibeagent exec channel).

```
tools/payloads/
  doom/payload.yaml          # converted first — proof of the design
  httpd/payload.yaml         # stress quest #1
  putty/payload.yaml         # compat quest #2
  compile.py                 # YAML -> staging manifest + registry .inf
tests/qemu/test_payload.py   # generic interpreter (runs any payload's `test:`)
```

### The spec — `tools/payloads/<name>/payload.yaml`

```yaml
name: doom
binaries:
  # `src` = prebuilt artifact (arch-templated), or `build:` = script that
  # produces it. Paths are repo-relative; `dest` is guest-image-relative
  # under \ReactOS\.
  - src: tools/doom/doom-{arch}.exe
    dest: system32/doom.exe
  - src: tools/doom/doom1.wad
    dest: system32/doom1.wad

autostart:
  - cmd: "doom.exe"
    mode: demo            # demo = Run-key autostart at boot
                          # test = launched on demand post-snapshot-restore

test:
  arches: [amd64, arm64]  # i386 xfail reasons live here too, honestly:
  xfail:
    i386: "kbd layout SEC_IMAGE deadlock — see test_doom.py"
  settle: 8               # seconds after launch before first probe
  drive:                  # host-side QMP input script, interpreted
    - click: center       #   focus the app window (rel-mouse sweep + click)
    - keys: [esc, ret, ret, ret]
    - wait: 2
  oracle:
    frame_delta_min: 0.02        # rendering-alive threshold
    frame_delta_static_max: 0.012 # post-drive idle threshold (omit if n/a)
    serial: []                    # marker strings that must appear
    survives: true                # QMP query-status running==true at end
```

Schema notes:

- **`{arch}` templating** in `src` keeps one spec for all three arches.
- **`mode: demo` vs `mode: test`** exists because of a snapshot-ordering
  interaction: Run-key autostart fires *before* the harness saves the `ready`
  snapshot, so demo apps are baked into every branched test case. Test apps
  (httpd, putty) want a pristine desktop snapshot and an on-demand launch
  after each `loadvm` restore.
- **`drive`/`oracle` are deliberately small vocabularies.** The interpreter
  grows verbs only when a second payload needs one (YAGNI). Anything genuinely
  app-unique stays as a hand-written pytest module beside the generic one —
  the YAML path is for the 90% shape, not a Turing tarpit. Initial verbs:
  `click` (center | {x,y}), `keys` (qcode list), `wait` (seconds),
  `tcp` ({port, send, expect} — for server payloads, via slirp hostfwd).
  Initial oracles: frame deltas (from `test_doom.py`), serial markers,
  `tcp_hammer` ({port, connections, requests, expect_status}) for httpd,
  `survives`.

### Consumer 1 — build time (`tools/payloads/compile.py`)

`VIBEOS_PAYLOAD=<name>` (comma-separable) on `./vibeos-cli build` invokes the
compiler, which emits into the build dir:

1. **A staging manifest** (flat `src -> dest` list, arch-resolved) consumed by
   **one shared graft helper** used by both `qemu/mkramdisk.sh` and
   `tools/strip_cd_media.py`. The twin `VIBEOS_DOOM` hooks are deleted and
   become the doom payload. Empty payload set ⇒ media byte-identical to today.
2. **A registry fragment `hivepayload.inf`** with the `mode: demo` Run keys
   (`HKLM\Software\Microsoft\Windows\CurrentVersion\Run`), fed to the existing
   `add_registry_inf` hive build (`boot/bootdata/CMakeLists.txt`) the same way
   `hivesys_arm64.inf` is conditionally appended.

Compat: `VIBEOS_DOOM=1` becomes an alias for `VIBEOS_PAYLOAD=doom` for one
transition period, then is removed.

**The build-dir stamp (learned the hard way, 2026-07-07).** Media content must
be a declared property of the build tree, not of whatever env a later rebuild
happens to run under. The first cut keyed staging purely off the env, and the
very first amd64 payload gate caught the flaw live: smoke's net stage re-ran
`ninja livecd` (via conftest's clean-build) *without* `VIBEOS_PAYLOAD`, and
strip_cd_media silently rebuilt the ISO doom-less between two payload-gated
boots — boot0 played Doom, the payload stage found a bare desktop. (arm64
dodged it only because `mkramdisk --refresh` is additive.) Fix:
`vibeos-cli build` ALWAYS writes `build-<arch>/payload-selection.json` from the
env (set or cleared), and both media hooks resolve **env if the operator
expressed any selection (even set-empty), else the stamp** (`compile.py
resolve_payloads`/`--payloads-auto`). Env-less rebuilds now reproduce the same
media; changing the payload set requires going through the front door (or
setting the env on the rebuild).

### Consumer 2 — test time (`tests/qemu/test_payload.py`)

A generic interpreter, **not** codegen: it reads the same YAML at collection
time and parametrizes per payload. `./vibeos-cli smoke --arch X --payload doom`
(with `--doom` as alias). Flow per payload:

- boot the payload media once; assert `mode: demo` apps launched (serial
  marker / process evidence);
- save the `ready` snapshot at steady state (machinery already exists:
  `vm.py:save_snapshot`/`restore_snapshot`, pattern validated in
  `test_snapshot_apps.py` at 80–140× faster than reboot, byte-deterministic
  restores over 50 cycles);
- per `mode: test` app / per drive-script case: `loadvm ready`, launch, run
  `drive`, assert `oracle`.

`test_doom.py` is the first convert (its click/keys/frame-delta logic moves
into the interpreter verbs); it keeps a thin module only if something refuses
to fit the vocabulary — and that refusal is signal to either grow a verb or
keep it bespoke deliberately.

### Workstream A — fix `ProcessStartupItems()` on amd64 (no more winlogon spawns)

Root-cause why explorer's startup processing is unreliable on amd64 instead of
continuing to route around it. Assets already in place:

- **Permanent in-image canary:** kbswitch.exe's HKCU Run entry
  (`hivedef.inf:1931`) — a green fix means it launches on every boot.
- **Cheap iteration:** the snapshot harness rewinds to pre-shell states in
  ~seconds.
- Suspected family: the same shell/win32k startup races we fixed all week
  (desktop switching §9.22, explorer csrss-handoff contention — see the
  `nettest_explorer_race` note in winlogon.rs).

Exit criteria: Run-key apps launch reliably on all three arches ⇒ delete
`TryStartDoom`/`DoomStaged`/the notepad focus dance/the nettest direct spawn
from winlogon entirely; winlogon returns to knowing nothing about apps.
Until then, the compiler *may* keep emitting a transitional winlogon-read
list, but the intent is deletion, not a prettier hack.

### Workstream B — vibeagent (host-driven exec channel, phase 2)

A small Rust win32 exe staged like any payload binary, autostarted via Run
key, that **listens on TCP** behind a slirp hostfwd (usermode winsock is
proven: `[NETTEST] usermode winsock TCP echo OK`). JSON-lines protocol:

```
-> {"run": "system32\\doom.exe", "args": "", "id": 1}
<- {"id": 1, "pid": 172, "ok": true}
-> {"poll": 1}                      # alive? exit code?
<- {"id": 1, "running": true}
```

What it buys: zero-rebuild per-boot choice of what to run, a **structured
results channel back** (today's only oracles are serial markers + screenshots),
and it dogfoods the netstack — the test transport is itself a network stress.
It slots behind the interpreter's launch seam (the one `test_snapshot_apps.py`
explicitly left open) without changing any YAML.

Deliberately phase 2: it shares the winsock-server shape with httpd, so it is
grown *from* the httpd bring-up rather than built cold.

## Sequencing

1. **Spec + compiler + shared graft helper + interpreter skeleton**, doom
   converted end-to-end (staging parity, `--doom` alias green on amd64+arm64).
2. **httpd payload** (single-file mingw-friendly server, e.g. mongoose-class)
   + `tcp_hammer` oracle — first stress consumer; exercises snapshot branching.
3. **ProcessStartupItems root-cause** (Workstream A) — parallel lane to 2.
4. **vibeagent** (Workstream B) — grown out of 2's server shape.
5. **PuTTY payload** — the design's real exam: a foreign GUI+net binary that
   should be *pure YAML* (drive: type into terminal; oracle: rendered output
   deltas + host-side sshd/echo evidence).

## Open questions

- **Hive rebuild cost when toggling payloads:** Run keys live in the built
  hives, so switching payload sets re-runs the hive step (cmake), not just
  `mkramdisk --refresh`. Acceptable; if it grates, a later option is a
  first-boot importer that reads `\ReactOS\vibetest\payload.reg` — but that's
  another consumer to write and the cmake path reuses what exists.
- **Multi-payload composition** (`VIBEOS_PAYLOAD=doom,httpd`): dest-collision
  policy is "hard error", first come first served is forbidden (fail-loud rule).
- **Where does the WAD-sized asset budget bite?** arm64 ramdisk.img has finite
  headroom; the compiler should report per-payload staged bytes and fail loud
  past a threshold instead of producing a mysteriously unbootable ramdisk.
- **Keystroke fidelity:** QMP qcodes are PS/2-scancode-shaped; if a payload
  ever needs text-typing at scale, grow a `type: "string"` verb that maps via
  the `test_snapshot_apps.py` `_QCODE` table rather than hand-listing keys.
