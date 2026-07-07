# Doom on vibeos — the foreign-binary beachhead

**Status: PLAYABLE on amd64 AND arm64 (2026-07-07); i386 keys blocked on the kbd-layout SEC_IMAGE deadlock.**
id Software's 1993 DOOM (via `doomgeneric`'s Win32/GDI backend, shareware
`doom1.wad`) runs on vibeos — and on amd64 the smoke gate literally **starts a
new game with injected mouse + keyboard** (click to focus → ESC/ENTER menu
navigation → E1M1). This is the first time vibeos has run a genuinely
*foreign* binary — code with zero knowledge of our quirks, exercising the PE loader, the
Win32 API surface (user32/gdi32/kernel32), `msvcrt.dll`, GDI blitting (`StretchDIBits`),
the message loop, and the filesystem end-to-end.

Screenshots: `~/vibeos-doom/shots/` (`doom-e1m1-shotgun.png`, `doom-demo-medikit.png`).

## Why it matters

Everything vibeos had booted before was *our own tree*. A foreign binary is an
adversarial integration test we don't otherwise have: every crash it hits is a real
compat bug our own binaries mask. Getting Doom to render surfaced **three real bugs**
(two in our kernel, one a toolchain miscompile) plus one open win32k gap — see below.

## Quick start (the `VIBEOS_DOOM=1` opt-in — ALL 3 ARCHES)

Doom is wired as an **off-by-default** image flag on every arch. Two commands:

```bash
tools/doom/build.sh all                            # fetch + cross-compile doomgeneric (-O0), 3 arches
VIBEOS_DOOM=1 ./vibeos-cli build --arch <arch>     # stage doom.exe + doom1.wad into the boot media
./vibeos-cli smoke --arch <arch> --doom            # opt-in smoke: launch + render + input gates
```

How the flag stays off by default and safe to ship:
- **arm64:** `mkramdisk.sh` only stages `doom.exe`/`doom1.wad` when
  `VIBEOS_DOOM=1` (env inherited straight through `vibeos-cli build` →
  mkramdisk). A normal build stages nothing.
- **i386/amd64:** `tools/strip_cd_media.py` (already run by every `livecd`
  build) appends the two graft-point lines to the mkisofs list only when
  `VIBEOS_DOOM=1`; flag set + assets missing = hard build abort (fail-loud).
- **One launcher, file-gated:** winlogon's `TryStartDoom()` (all arches —
  called before the arm64 cmd.exe wait, and on the common x86 path) launches
  Doom **only if `GetFileAttributesW` finds `doom.exe` in system32**
  (drive-letter agnostic via `GetWindowsDirectoryW`). No file (every normal
  build) → silent no-op, zero serial noise. Landed on mainline.
- Assets (`tools/doom/doomgeneric/`, `doom1.wad`, `doom-*.exe`) are gitignored —
  fetched/built on demand, never committed.

## The opt-in smoke suite (`--doom`)

`tests/qemu/test_doom.py` (front door: `./vibeos-cli smoke --arch <a> --doom`,
which runs it as a separate boot after a green North-Star boot, net-suite
style). Three gates, all against VIBEOS_DOOM-staged media:

1. **`test_doom_launched`** — the `doom.exe created` serial marker (winlogon
   arm64 / userinit x86). Fails with the build recipe if media wasn't staged.
2. **`test_doom_renders`** — two screendumps 2s apart must differ in ≥2% of
   pixels (the attract demo animates most of its viewport; a bare desktop only
   ticks the tray clock at <<0.1%). Catches GDI-blit/game-loop wedges.
3. **`test_doom_input_reaches_game`** — the PLAYABLE gate: click into the
   window (mouse chain: activation), then drive the menu with injected
   ESC→ENTER→ENTER→ENTER and **start a new game**. An idle first-person view
   is near-static (<1.2% frame delta, just the status-bar face) while the
   attract demo animates ~50% — a binary signal with no pixel heuristics.
   (Don't probe with F1: doomgeneric's `convertToDoomKey` doesn't map it —
   only ESC/ENTER/arrows/ctrl/space/shift are mapped.) `xfail` on arm64 until
   the win32k focus-routing gap (bug #4) is fixed; hard assert on i386/amd64.

Suite is skipped entirely without `--doom` — zero cost to normal smoke/gate.

The rest of this doc is the *why* behind each build flag and the bugs the bring-up
exposed — read it before touching `tools/doom/build.sh`.

## The build (host side, no OS rebuild needed)

`doomgeneric` (https://github.com/ozkl/doomgeneric) with the `doomgeneric_win.c` backend,
cross-compiled with our llvm-mingw toolchain. The three non-obvious link requirements:

1. **Link against classic `msvcrt.dll`, not UCRT.** llvm-mingw defaults to UCRT
   (`api-ms-win-crt-*` api-sets), which vibeos doesn't ship. Force classic msvcrt by
   putting `-lmsvcrt-os` *ahead* of the default libs. (There is no `-mcrtdll=` in this
   clang.) Verify with `objdump -p doom.exe | grep 'DLL Name'` → should show only
   `msvcrt.dll USER32.dll GDI32.dll KERNEL32.dll`.
2. **GUI subsystem, not console.** `doomgeneric_win.c` has `int main()`, so a plain link
   produces a **console (CUI)** subsystem exe. A console app spawned by winlogon (a GUI
   process with no console) blocks in CRT startup forever waiting for csrss/consrv to
   allocate a console — the process parks in an Executive/LPC wait and never runs `main`.
   Fix: `-Wl,--subsystem,windows` **and** `-Wl,-e,mainCRTStartup` (the subsystem flag
   alone makes lld default the entry to `WinMainCRTStartup`, which calls a non-existent
   `WinMain`; forcing the entry keeps `main` as the real entry point).
3. **`-O0`, NOT `-Os`/`-O2`** — see the miscompile section. This is load-bearing.

Full one-liner (arm64; swap `aarch64`→`x86_64` for amd64):

```bash
TC=.build-tools/llvm-mingw/bin
SRC=$(printf '%s\n' dummy am_map doomdef doomstat dstrings d_event d_items d_iwad \
  d_loop d_main d_mode d_net f_finale f_wipe g_game hu_lib hu_stuff info i_cdmus \
  i_endoom i_joystick i_scale i_sound i_system i_timer memio m_argv m_bbox m_cheat \
  m_config m_controls m_fixed m_menu m_misc m_random p_ceilng p_doors p_enemy p_floor \
  p_inter p_lights p_map p_maputl p_mobj p_plats p_pspr p_saveg p_setup p_sight p_spec \
  p_switch p_telept p_tick p_user r_bsp r_data r_draw r_main r_plane r_segs r_sky \
  r_things sha1 sounds statdump st_lib st_stuff s_sound tables v_video wi_stuff \
  w_checksum w_file w_main w_wad z_zone w_file_stdc i_input i_video doomgeneric \
  doomgeneric_win | sed 's/$/.c/')
$TC/aarch64-w64-mingw32-gcc -O0 -w -o doomgeneric-arm64.exe $SRC \
  -lmsvcrt-os -luser32 -lgdi32 -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
# doomgeneric_win.c: fix the include case first — `<Windows.h>` -> `<windows.h>`.
```

All 83 imports resolve against vibeos's own built DLLs (verify with `llvm-readobj
--coff-exports` on `build-arm64/{dll/win32/msvcrt,dll/win32/kernel32,win32ss/user/user32,
win32ss/gdi/gdi32}/*.dll` vs `objdump -p doom.exe`).

## Injecting into the arm64 ramdisk + launching

```bash
mcopy -o -i qemu/ramdisk.img doomgeneric-arm64.exe ::/reactos/system32/doom.exe
mcopy -o -i qemu/ramdisk.img doom1.wad             ::/reactos/system32/doom1.wad
mcopy -o -i qemu/boot.img    qemu/ramdisk.img      ::/ramdisk.img
VIBEOS_QEMU_EXTRA="-device virtio-keyboard-device -device virtio-tablet-device" \
  bash qemu/start.sh arm64
```

**NOTE (mkramdisk clobbers injections):** any `vibeos-cli build` re-runs mkramdisk and
rebuilds `ramdisk.img` from the build tree — your `mcopy`'d doom files vanish. Re-inject
*after* every build.

The launcher is landed in winlogon (`TryStartDoom()`, file-gated) — no scaffold
hunk needed anymore; `VIBEOS_DOOM=1` at build time is the only switch.

Serial confirms launch: `[WINLOGON] doom.exe created (PID …, TID …)`. Screenshot via QMP
`screendump` on `qemu/runs/<run>/qmp.sock`.

## Bugs this exposed

### 1. arm64input avail-ring bounds panic (REAL kernel bug — FIXED, landed)

First keypress on the arm64 desktop wedged the whole machine. Root cause:
`arm64input.rs` declares the virtio avail-ring as a C flexible-array-member
(`Ring: [u16; 0]`) and then **safe-indexed** it (`(*Avail).Ring[idx] = …`) in the
DPC drain path — safe indexing a zero-length array bounds-panics for *any* index. The
driver's (former) silent `loop {}` panic handler then span at DISPATCH, killing every DPC
on the box → dead input, frozen desktop. Fixed by using raw-pointer arithmetic (matching
the queue-init fill site) + a loud panic handler. Live-forensics trail: stuck PC →
MZ-scan for the module base → PE export fingerprint → frame-walk → panic `Location`
struct in guest memory (`arm64input.rs:1283`).

### 2. Silent `loop {}` panic handlers (FIXED for arm64input; sweep owed)

The wedge above was invisible for a full forensics session because the driver's
`#[panic_handler]` was a bare `loop {}`. Ported the loud win32k-style handler
(`DbgPrint` "RUST PANIC" + `file:line`, the shape smoke triage classifies as
`RUST_PANIC`) to `arm64input-cargo`. **~40 other `*-cargo/src/main.rs` still have bare
`loop {}` handlers** (incl. `hidparse`, `i8042prt`, `kbdclass`, `mouclass`) — tracked as
a sweep.

### 3. clang aarch64 `-Os`/`-O2` miscompiles a pointer-chase loop (TOOLCHAIN)

**This is the interesting one.** doom hung in `R_InitTextures` → `R_CheckTextureNumForName`,
whose body is the textbook hash-chain walk:

```c
texture = textures_hashtable[key];
while (texture != NULL) {
    if (!strncasecmp(texture->name, name, 8)) return texture->index;
    texture = texture->next;                       // <-- pointer chase
}
return -1;
```

The data is correct (`GenerateTextureHashTable` NUL-terminates every chain). But
**clang 19 (`aarch64-w64-mingw32`) at both `-Os` and `-O2`** hoists `texture` out of the
loop as loop-invariant and emits:

```asm
ldr  x8, [hashtable + key]     ; load texture ONCE, before the loop
cbz  x8, .Lreturn_minus1       ; if NULL -> return -1
.Lspin: b .Lspin               ; else: texture never reloaded -> b .  (infinite loop)
```

i.e. the `texture = texture->next` reload is eliminated and the loop becomes `b .`
(opcode `0x14000000`, branch-to-self) whenever the first chain entry is non-NULL. This is
the **same LLVM aliasing/LICM miscompile class vibeos has fought in the kernel**
(`captures(none)` / volatile-blind loads), now reproduced *independently* on a foreign
binary compiled with stock flags. **`-O0` compiles the reload faithfully and the loop
terminates.** Detector: `objdump -d doom.exe | grep -E '^\s+[0-9a-f]+:\s+14000000'`
(any hit = a `b .` self-branch smuggled in by the optimizer).

Takeaway for anyone cross-compiling apps for vibeos: **build foreign apps `-O0` until we
understand which optimizer pass is at fault**, and scan the output for `b .`.

### 5. x86 PS/2 input was COMPLETELY dead (FIXED 2026-07-07 — four stacked bugs)

The playable gate's first real catch, and it was a big one: keys and mouse
never reached ANY x86 app. Four independent root causes, each masking the
next — full forensics + oracles in field-manual **§9.27**:

1. acpi-rust's invented `CmResourceType` table typed IRQ descriptors as Null →
   i8042prt never connected its ISRs (kbd + mouse dead at the interrupt level).
2. i8042prt's mouse reset state machine `transmute`d invalid enum
   discriminants (1001..) → UB → **LLVM deleted the match arms** → mouse
   stalled at wheel-detect forever (new translation-bug class).
3. winlogon loaded keyboard layouts only on arm64 → win32k dropped every key
   at "NO keyboard layout" on x86.
4. The active input desktop could be stranded on "Winlogon" on x86 (the §9.23
   arm64 fix was cfg'd arm64-only; de-cfg'd).

Plus a latent upstream race fixed en route: the shared i8042 packet machine
now checks which port the in-flight packet targets, so keyboard packets can't
eat mouse bytes.

### 4. arm64 keyboard/mouse don't reach foreground apps (OPEN)

Post-driver-fix, keystrokes reach `KeyboardClass0` (serial: `[ARM64INPUT] KBD-DELIVERED`)
and the game loop runs, but typed text never lands in the focused edit control and mouse
clicks don't raise/activate windows. This is a **win32k raw-input → foreground/focus
routing gap** on arm64, separate from the driver. It's why Doom currently runs as the
self-playing attract demo rather than interactively. Blocks "playable Doom." Tracked
separately (related to the desktop-input-desktop work in field-manual §9.23).

## Next

- **Playable Doom on arm64** = fix bug #4 (arm64 kbd/mouse → focus), then flip
  `test_doom_input_reaches_game`'s arm64 `xfail` to a hard assert.
- **PuTTY** (the truly-unmodified official binary, incl. an ARM64 build) + our smoltcp
  netstack = SSH from inside vibeos — the natural follow-on foreign-binary quest.
