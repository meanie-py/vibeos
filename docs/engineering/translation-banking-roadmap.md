# Translation Banking Roadmap — "All The Rest"

**Date:** 2026-06-06
**Paradigm:** bank-now-integrate-later (see `docs/engineering/rustconv-cost-efficient-translation.md`)
**Purpose:** Enumerate *every* remaining C→Rust translation in the tree, grouped into
cache-friendly banking batches, sequenced by value, with cost estimates. This is the
master plan; individual session work pulls batches off it.

> **UPDATE 2026-06-09 — several batches below have since landed:**
> - **Phase-2 RTL leftovers ≈DONE** — the sync cluster + all misc leaves are flipped (c1cf116f38c); only heappage/heapuser remain (translated, cohab-flip pending). The ~$17 / 20K-LOC estimate no longer applies.
> - **L1a `pci` (3K) DONE** — pci.sys C→Rust committed 4c5b06893fc.
> - **L3 acpi OSL glue DONE** — ACPICA FFI-to-blob COMPLETE (20b051f7ee6, 85b5c45ffbe).
> - **Phase-0 ntuser bank is being integrated wave-by-wave on crp** (through ~wave 38), not sitting untouched; one unresolved boot regression remains in a 9-file batch (e164dd6a90a).

---

## The banking paradigm in one paragraph

`rustconv translate` is per-**function** with Anthropic prompt-caching: the per-file
shared context (imports/globals/gap-fns) is cached, so fn 1 pays ~1.25× and fns 2..N
read it at ~0.1×. **Bank a whole subsystem cheaply while the API is warm** (output →
`~/vibeos-rust-staging/<subsys>/*_out.rs`, durable + resumable via
`tools/rustconv/.responses/`), then **integrate in a separate no-cost phase** (wire →
crate ×3 → ninja → lint → ud2 → smoke ×3). Banking and integration never share a
session under a spending clock. Cost rule of thumb: **~$0.30–0.70 / 1,000 LOC** (Opus
sync + cache), higher on table/output-dense files. Iron rules: `translate` always
(never `batch`/`submit` — they stub silently); never Ctrl-C mid-call; never
re-translate banked output; watch the live `[running $X]` + "caching saved ~$Y" line.

---

## What is ALREADY DONE (no translation — do not re-bank)

These are the North Star and its neighbors. The kernel-to-Rust goal is **functionally
complete**; "the rest" is the peripheral OS.

| Area | State | Residual C |
|---|---|---|
| **ntoskrnl.exe** | ✅ ~405 .rs / DONE | **8 shim files, 1,037 LOC** — all intentional glue (C-variadic trampolines, `__FILE__`/`__LINE__` macro outlines, arch asm/ABI bridges, i386 fat-LTO workaround). **Not translatable on stable Rust.** Leave them. |
| **HAL** | ✅ DONE ×3 | 6 x86 `.S`, `.rc`, link glue only |
| **libcntpr (kernel CRT)** | ✅ DONE | printf + small residue |
| **User-mode spine** (smss/csrss/basesrv/lsass/services/winlogon/userinit/winsrv) | ✅ 99%+ Rust | RPC stubs (generated), variadic strsafe shims, GINA layer (by design). ~0 business logic. |
| **RTL core** (unicode/nls/sid/acl/sd/priv/heap/actctx/threadpool/version/ppb/slist/network + 18 algo crates) | ✅ DONE | cohab-commented in `sdk/lib/rtl/CMakeLists.txt` |

**Implication:** every remaining *translation* dollar now goes to **drivers,
secondary libs, win32k, and RTL leftovers** — not the kernel proper.

---

## PHASE 0 — Integrate the existing win32k bank (FREE, do first)

The single highest-leverage move requires **zero API spend**: ~70K LOC of ntuser +
DirectDraw is already banked in `~/vibeos-rust-staging/win32k/`. Wiring it drops
win32k.sys from ~54K LOC C to ~13–38K depending on how far it goes.

**Caveat — the bank is messy.** It holds 81 `.rs` files / 616K lines including
iteration dupes (`gen_old.rs`, `generated.before.rs`, `msgqueue_fixed` vs
`msgqueue_out`, `gdibatch_final` vs `gdibatch_out`, `notify_rs_raw` vs
`notify_postprocessed`). Only **~37 are the clean canonical translations**. Step 0 is
**curation**: pick the one true file per source, archive the rest. Do NOT blind-wire.

Clean canonical files (highest value first): `menu_out` (8.3K), `window_out` (6.2K),
`winpos_out` (4.9K), `message_out` (4.6K), `ime_out` (3.6K), `msgqueue_out` (3.3K),
`painting_out` (3.1K), `class_out` (2.9K), + the ~10 DirectDraw `ntddraw_*_out.rs`.

**Sub-phases** (each = wire → win32k-rust crate ×3 → `ninja win32k` → lint → ud2 →
smoke ×3; per-wave ABI/offset discipline from `project_win32k_ntuser_directdraw_banked`):
- **0a** menu / window / winpos / message → C: 54K→~29K
- **0b** msgqueue / ime / painting / class → ~29K→~13K
- **0c** remaining ntuser files in bank
- **0d** DirectDraw batch (`ntddraw_*`) → leaves ~shims + asm

**Cost: $0.** **Risk:** activates never-booted code (smoke reaches desktop without
typing, may miss input bugs — exercise keyboard/mouse in the screenshot check).

---

## PHASE 1 — win32k C tail (small bank)

What's still pure-C in win32k.sys and **not** in the bank:
| Files | LOC | Notes |
|---|---|---|
| `win32ss/user/rtl/` (text.c, bitmap.c) | ~1,448 | USER's rtl helpers — **rtl/image+text is the user's lane per memory; coordinate** |
| `win32ss/user/ntuser/misc/` (2 files) | ~747 | file I/O + registry access |
| any ntuser `.c` not covered by the bank | small | reconcile after Phase 0 curation |

**Bank target:** `~/vibeos-rust-staging/win32k/` (extend). **Cost: ~$1–2.**

---

## PHASE 2 — Kernel/boot secondary libs (high ROI, self-contained)

These are kernel- and bootloader-linked, self-contained, and unlock cleaner cargo
cutovers downstream.

| Lib | LOC | Why | Bank dir |
|---|---|---|---|
| **RTL leftovers** | ~20K | the un-translated `sdk/lib/rtl/*.c` still compiled (verified active): misc leaves (`error`, `registry`, `image`, `locale`, `message`, `propvar`, `rangelist`, `res`, `rxact`, `sysvol`, `bootdata`, `dbgbuffer`, `memstream`, `heappage`, `heapuser`) + **sync cluster** (`critical`, `condvar`, `srw`, `runonce`, `wait`) + **arch unwind** (`amd64/except`+`unwind`+`dynfntbl`+`stubs`, `arm64/except`, `i386/except`+`thread`). Continues `project_rtl_misc_leaves`. | `staging/rtl/` |
| **cmlib** | 8.8K | registry hive parse/write, kernel + freeldr; self-contained, top ROI | `staging/cmlib/` |
| **inflib** | 3.3K | INF parser; modest, low-risk | `staging/inflib/` |
| **pseh** | 1.5K | tiny but ABI-tricky (DUMMY_PSEH already in play); translate carefully | `staging/pseh/` |

**Order:** RTL misc leaves (hand or bank) → cmlib → inflib → pseh. Sync cluster
(`critical`/`srw`/`condvar`/`wait`/`runonce`) is **not** a free leaf — needs the API.
**Cost: ~$17.** Arch unwind (`amd64/unwind.c` etc.) is asm-heavy — review before banking.

> **Skip for now:** `sdk/lib/crt` (53K, mostly user-mode C legacy), `ucrt`, `vcruntime`,
> `rossym` (debugger-only). Low boot value; revisit only if a cutover demands them.

---

## PHASE 3 — Drivers + filesystems, separated by STRATEGY (not by tier)

**71 C-buildable driver targets remain** (the clean small-DriverEntry leaves — null,
nmidebug, arm64vid, arm64input, i8042prt, kbdclass, mouclass, beep, condrv, pciide —
are done; see `project_device_driver_cargo_cutover`). The realization (2026-06-06,
user steer): **the right axis is *how each thing gets done*, not whether.** Target =
**a usable desktop in QEMU** (input incl. USB, storage incl. USB, display, networking),
not merely boot. Five lanes:

- **L1 · C→Rust translate** — the existing rustconv path.
- **L2 · C++→Rust translate** — needs a one-time **tree-sitter-cpp frontend** added to
  rustconv (`tree_sitter_cpp` + a C++ scanner/resolver variant; the pipeline is already
  tree-sitter, confirmed). Enables all C++ drivers *and* explorer.exe later.
- **L3 · FFI to C blob** — keep upstream C, translate only the thin Rust shell/OSL.
- **L4 · Rust ecosystem crate** — re-architect on a real Rust crate instead of
  translating ReactOS C (networking → `smoltcp`).
- **L5 · Skip / leave as C** — legacy/alt hardware QEMU never touches.

Shared prereq for L1/L2 drivers: the **driver_ddk substrate** (bindgen is `ntoskrnl.h`
not `wdm.h` — `feedback_driver_ddk_substrate`) + the **export-driver cutover machinery**
(spec2def + export plumbing) built once, reused by every `.spec`/`.def` driver.

### L1 — C → Rust translate

**L1a · small boot/desktop leaves (~$12, hand or cheap bank):**
`pci` (3K), `mountmgr` (9.4K), `fs_rec` (2K), `partmgr` (2.6K), `ramdisk` (2.5K,
arm64 boot), `blue` (1.7K), `ksecdd` (1.4K, export), `swenum` (431), `processr` (507),
`pl011` (254, arm64 UART), `wmi` (60, export), `hidparse` (98, export).

**L1b · storage stack (~$50):** `classpnp` (42K, export), `cdrom` (32K), `disk`
(12.5K), `atapi` (6.5K), `scsiport`+`storport`+`storahci` (~13K, for SATA/AHCI).
Sequence boot-critical ones, smoke ×3 between.

**L1c · USB stack — KEEP per user (~$30):** `usbport` (17K, framework), `usbhub`
(10K), `usbehci`/`usbohci`/`usbuhci`, `usbccgp`, `usbstor` (USB disk → usable
desktop), `usbd` (721, export). Bank `drivers/usb/*` as one warm batch.

**L1d · HID stack — needed for USB input (~$6):** `hidusb`, `hidclass` (export),
`kbdhid`, `mouhid`. (PS/2 path already Rust; these add USB keyboard/mouse.)

**L1e · filesystems we actually mount (~$60):** `cdfs` (32K, LiveCD ISO9660),
`vfatfs` (14K) *or* `fastfat` (68K) for FAT, `ntfs` (14K, optional read). Plus
`sdk/lib/fslib` (12.7K, format libs). **Skip** ext2/btrfs/nfs/msfs/mup (L5).

### L2 — C++ → Rust translate (after tree-sitter-cpp enabler)

**Enabler first** (dev time, ~0 API): add `tree_sitter_cpp` grammar + C++
scanner/resolver/prompt variant to rustconv (classes→structs+impl, methods, namespaces,
templates→generics/monomorphized). Then translate as desired:
- `uniata` (11 cpp, alt IDE/AHCI — optional, pciide+atapi already cover IDE)
- audio: `portcls` (41 cpp, framework), `ks`-deps, `hdaudbus`, `ac97`, `CMIDriver`,
  `drmk` — **only if sound is wanted** (not needed for desktop)
- `udfs` (51 cpp, optical FS — niche, skip unless DVD)
- **Biggest L2 payoff is elsewhere: `explorer.exe` (C++)** — the shell itself; the
  tree-sitter-cpp work pays for itself there.

### L3 — FFI to C blob (keep upstream, translate the shell)

- **`acpi`**: the `acpica/` subtree is **88.8K LOC of Intel ACPICA upstream → link as a
  C static lib, FFI to the `Acpi*`/`Acpi…` API**. Translate only the **9.98K LOC**
  ReactOS OSL/glue (`main`, `buspdo`, `power`, `eval`, `pnp`, `osl`, `acpienum`,
  `interface`, `busmgr/*`) to a Rust shell that calls into the blob. **~$5 + FFI
  substrate**, vs ~$50 to translate ACPICA wholesale. **Do not translate ACPICA.**

### L4 — Rust ecosystem crate (re-architect, not translate)

- **Networking → `smoltcp`** (pure-Rust TCP/IP — also does UDP/ICMP/DHCPv4/DNS).
  **Design doc: `docs/engineering/smoltcp-netstack-design.md`** (decided 2026-06-07).
  Splice point **A**: smoltcp engine behind the **existing TDI device interface**
  (AFD/msafd/ws2_32 untouched), replacing tcpip's `ip/`+`lwip/` (~123K C) + a Rust TDI
  dispatch. Device backend = **virtio-net via `virtio-drivers`** for QEMU (NDIS `Device`
  scoped but **deferred**). De-risk via a throwaway in-kernel smoltcp+virtio engine
  self-test first; QEMU net test harness is a prerequisite. **$0 translation; real
  design+impl work.** This is the "use a Rust lib for network" decision.

### L5 — Skip / leave as C (QEMU never loads these)
`buslogic` (old SCSI HBA), `bus/pcix`, `bus/acpi_new` (alt versions), `e1000`,
`rtl8139`, `ndisuio`, `nfs`, `msfs`, `mup`, `tdi` (if AFD talks to smoltcp directly),
`ext2`, `btrfs`, `sndblst`, `usbaudio`, `serial` (16550, arm64 uses pl011). Revisit
only if a concrete need appears.

**Phase 3 cost (committed L1+L3 for usable desktop): ~$165 translation** + the L2/L4
enablers (dev time) + integration labor. L2 audio + L4 net add capability without
translation $.

---

## Enabler workstreams (unlock the lanes; mostly dev-time not API $)

1. **tree-sitter-cpp frontend for rustconv** → unlocks L2 (all C++ drivers + explorer).
2. **ACPICA FFI substrate** (acpica static lib + Rust `Acpi*` bindings) → unlocks L3.
3. **smoltcp integration architecture** (NDIS-miniport shim + AFD/TDI surface +
   virtio miniport) → unlocks L4. Needs a design doc before code.
4. **driver_ddk + export-driver (spec2def) machinery** → prereq for most L1/L2 drivers.

---

## Grand totals (revised — usable-desktop target, strategy-lane cut)

| Phase | Scope | Translation $ |
|---|---|---|
| **0** | Integrate existing win32k bank | **$0** (curate + wire) |
| **1** | win32k C tail | ~$2 |
| **2** | RTL leftovers + cmlib/inflib/pseh | ~$17 |
| **3·L1** | drivers C→Rust (boot + storage + USB + HID + FS we mount) | ~$160 |
| **3·L3** | acpi OSL glue (ACPICA stays C, FFI) | ~$5 |
| | **TOTAL committed translation** | **~$185** |

Plus, **outside the translation $ line** (dev-time / design, capability not LOC):
- **L2** C++ drivers (audio, uniata, udfs) + **explorer.exe** — gated on the
  tree-sitter-cpp enabler; do per appetite.
- **L4** networking on smoltcp — design + integration, $0 translation.
- **L5** skip-list (~37 drivers / ~470K C LOC) — left as working C indefinitely.

Down from the earlier ~$460 because: net is a Rust-crate rewrite not a 170K-LOC
translation; ACPICA is FFI'd not translated (−~$45); ext2/btrfs/nfs/extra-NICs skipped.
USB+HID+FS-we-mount are *added* (usable-desktop bar). Integration labor (free $, real
time) still applies to every banked batch: wire → crate ×3 → ninja → lint → ud2 →
smoke ×3.

---

## Recommended sequence

1. **Phase 0 now** — free, biggest single win; validates bank→integrate on a large real
   backlog. Curate the messy win32k bank first.
2. **Phase 2 RTL leaves + cmlib** — cheap, self-contained, continues an active lane.
3. **Build the prereq machinery** (driver_ddk + spec2def export plumbing) → then L1a
   small leaves flow.
4. **L1b/L1c/L1d** storage + USB + HID — the usable-desktop hardware path; smoke ×3
   between boot-critical ones.
5. **L3 acpi FFI** (substrate + 10K glue) and **L1e** filesystems we mount.
6. **Enablers as appetite allows:** tree-sitter-cpp (→ L2 + explorer), smoltcp design
   (→ L4 net). These unlock capability, not just cargo-coverage.

Bank in directory-sized batches (warm cache), smallest-first, with `--cost-cap`. Post a
per-batch tally (files, fn counts, 0-stub grep, $ spent) for the integration session.

---

## Rust crates we can lean on (the L4 lever, generalized)

Researched + verified 2026-06-06. All are **`no_std`** (we're already nightly +
build-std, so `alloc` and nightly-only `core_io` are fine). All are permissive
(MIT/Apache/0BSD) → GPL-compatible (ReactOS is GPL; GPL absorbs permissive). **The
catch is uniform: a crate gives you the *engine*; you still write the ReactOS *glue*
(driver object / IRP dispatch / NDIS-miniport / FSD callback / `Hal` DMA trait). They
cut the core logic, not the integration shim.** This is the L4 lane generalized — it
advances "usable desktop in Rust," not the "C LOC translated" metric.

**★ Biggest lever — `virtio-drivers` (rcore-os):** one `no_std` crate with guest drivers
for **net, block, GPU, input**, console, socket, sound, RNG — exactly the device classes
a usable QEMU desktop needs. QEMU exposes all of these as virtio. Instead of translating
`disk`/`usbstor`/`atapi` (storage), `e1000`/`netkvm` (net), framebuffer paths (display),
and USB-HID (input), we can expose **virtio-blk / virtio-net / virtio-gpu / virtio-input**
behind thin ReactOS driver shims. Requires implementing the `Hal` trait (DMA alloc +
phys↔virt) once, and a per-device IRP shim. We control the QEMU command line, so the
virtio device model is ours to choose. **Potentially collapses large parts of L1b/L1c
and the display path for the QEMU target.** (Real HW would still want the translated
drivers — but QEMU is the CI/dev target.)

| Need | Crate | State | Lane it serves |
|---|---|---|---|
| TCP/IP stack | **smoltcp** | TCP/UDP/ICMP/IPv4+6/DHCP/DNS, `no_std`, **proven in Redox** | L4 net core |
| QEMU net/disk/gpu/input | **virtio-drivers** | `no_std`, all device classes, needs `Hal` impl | L4 + L1b/storage + display |
| Crypto primitives | **RustCrypto** (sha2/aes/hmac/rsa/ecdsa/…) | `no_std`, mature | `ksecdd` / CNG (vs translating C crypto) |
| Registry hive read | **nt-hive** (Colin Finck, ReactOS dev) | `no_std`, **read + basic in-memory mod** (no full write) | Phase 2 `cmlib` boot/read path (kernel CM *write* still needs translate) |
| NTFS | **ntfs** (Colin Finck) | `no_std`, **read-only**, explicitly "firmware→kernel-mode FSD" | `ntfs` read mount |
| FAT | **fatfs** (rafalh) | `no_std` (nightly `core_io`), **read + write** | `vfatfs`/`fastfat` backing |
| ACPI tables | **acpi** (rust-osdev) | `no_std`, solid: RSDP/RSDT/XSDT/MADT/MCFG/FADT | L3 table side |
| ACPI AML interpreter | **aml** (rust-osdev) | `no_std` but **incomplete** — OK for simple/QEMU tables, risky on real HW | L3 AML — **prefer ACPICA-FFI for AML**, aml only if it proves enough for QEMU |
| Font rasterization | **ttf-parser + fontdue** (or **titanf**) | `no_std`+alloc, pure Rust, faster than FreeType | win32k text (vs the banked FreeType FFI) |
| Safe FFI struct casts | **bytemuck** / **zerocopy** | `no_std` | the translations themselves — replace hand-rolled `transmute` + `repr(C)` casts (e.g. the `PCI_COMMON_CONFIG` pattern) |

**Two ReactOS-insider signals:** `nt-hive` and `ntfs` are written by **Colin Finck, a
ReactOS core dev**, *specifically* for "firmware→kernel-mode" reuse and an eventual
Rust ReactOS bootloader. They are the closest-fit, lowest-risk external crates here.

**Recommended hybrid for L3 (ACPI):** `acpi` crate for static-table parsing +
**ACPICA-FFI for AML/_DSDT eval** (the `aml` crate isn't complete enough to trust for
the interpreter). Translate only the ReactOS OSL glue around both.

**Caveats to weigh per adoption:**
- **virtio path changes the guest device model** — needs virtio devices on the QEMU
  cmdline + a `Hal` DMA trait impl; great for the QEMU CI target, not a real-HW driver.
- **Read-only crates** (`ntfs`, `nt-hive` write) cover mount/boot read but **not** the
  write paths — those stay translate-or-keep-C.
- **Format/behavior compat:** a Rust FS/registry engine must match on-disk/on-hive
  semantics ReactOS tests assume — usually fine for read, verify for write.
- These are **L4-style rewrites**, not translations — they don't move the C→Rust LOC
  number, and they need design (esp. the smoltcp↔NDIS/AFD and virtio↔IRP boundaries).

## Cross-references
- Paradigm + gotchas: `docs/engineering/rustconv-cost-efficient-translation.md`
- Driver FFI playbook: memory `project_device_driver_cargo_cutover`
- win32k bank status: memory `project_win32k_ntuser_directdraw_banked`
- RTL leaf lane: memory `project_rtl_misc_leaves`, `project_rtl_heap_track`
- Driver DDK substrate: memory `feedback_driver_ddk_substrate`
- Cutover endgame map: memory `project_cargo_cutover_endgame_map`
