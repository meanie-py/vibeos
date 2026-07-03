> ⚠️ **ARCHIVED 2026-07-03 — superseded by [`rust-port-status.md`](../rust-port-status.md).**
> This per-surface audit is a point-in-time historical record. Its per-file counts were
> measured ~2026-06-21 (before the last swarm wind-down) and are **stale** — several
> "0% / in-progress" claims here are now done. For current Rust-vs-C coverage use the
> status doc; this file is kept only for the detailed per-file mapping and history.

# Driver Translation Audit

> **Ground-truth re-verified 2026-06-21** against each driver's `CMakeLists.txt` + `sdk/lib/rust/*` crates (cohab `# replaced by Rust` markers + `add_cargo_module`/`add_cargo_kernel_dll` wiring) and `origin/master`. **A big driver wave landed since the 2026-06-17 rollup.** Corrections this pass: **classpnp / cdfs / cdrom are all LANDED on master** (cdrom shipped to mainline `c2839ebaf5b`) — no longer in-flight; **fastfat.sys is DONE** (fastfat-rust, full 34-`.c` cohab flip, boots arm64 to desktop ×3) — moved out of "candidate / not started"; and the **entire previously-"C-only" storage/HID set is now Rust-flipped on master**: storport-rust, storahci-rust, serial-rust, ramdisk-rust, hidclass-rust, hidusb-rust, kbdhid-rust, mouhid-rust all carry cohab markers and live crates. **btrfs/ext2/ntfs** stay DELETED (commit 14a0563b291, 2026-06-09; ~16 dead `baseaddress_{ubtrfs,uext2,untfs,shellbtrfs}` constants linger harmlessly in `sdk/cmake/baseaddress*.cmake`).
>
> **Translation: 50 of ~52 in-scope drivers done (~96%).** Methodology: a driver counts DONE when its module is Rust-flipped on `origin/master` — i.e. the C bodies are cohab-commented `# replaced by Rust` (lld first-match to the `#[no_mangle]` Rust symbol) **or** the link is cargo-driven (`add_cargo_module … BODY_CRATE <crate>-rust`). Measured: **50 driver module dirs flipped** (`grep -rl "replaced by Rust" drivers/` ∪ the `add_cargo_*` dirs). The full `drivers/` universe is **81 module dirs**, but ~29 are out-of-scope: the WDM **audio** stack (portcls/ac97/CMIDriver/drmk/kmixer/hdaudbus/wdmaud/sysaudio), the **USB host-controller / hub / function** stack (usbehci/usbohci/usbuhci/usbport/usbhub/usbccgp/usbstor/usbaudio), the **NDIS/legacy-net** drivers (ndis/ndisuio/netio/tdi/afd/e1000/netkvm/rtl8139 + the vendored **tcpip/lwIP** — being *rewritten* as smoltcp netstack, not C-translated), plus **ks**, **buslogic**, **setup/blue**, **kdrosdbg**, **acpi_new**, **usbstor_new**, and the vendored **ACPICA core**. Excluding those leaves ~52 realistic targets, of which 50 are flipped (only **netstack** is a from-scratch rewrite, not a C→Rust translation; the 2 not-yet-done are noted below). Raw, unfiltered: 50/81 ≈ 62%.

Status of the `drivers/` fleet C→Rust translation (peer to `rtl-` / `ntdll-` /
`win32k-translation-audit.md`). Covers the kernel-mode `.sys` drivers under
`drivers/` plus the ARM64 custom drivers. The `sdk/lib/drivers/` *helper static
libs* (csq, arbiter, ntoskrnl_vista, kdcom transports) live in
`static-libs-translation-audit.md` — they're libraries linked into the kernel,
not standalone drivers; cross-referenced below where a driver depends on them.

> Keep this status-light. Volatile per-driver detail (commit hashes, next steps,
> blockers) lives in the linked agent-memory topic files + git, which is the
> source of truth — verify against current code before trusting a row.

## Fleet status

| Driver | Path | Status | Crate | Notes |
|---|---|---|---|---|
| **pci.sys** | drivers/bus/pci | ✅ LANDED (master) | pci-rust | The live PNP0A03 root-bus enumerator. Boots ×3. Cohab. Found bindgen PCI_COMMON_CONFIG drop. |
| **pcix.sys** | drivers/bus/pcix | ✅ LANDED (master) | pcix-rust | Full MS-arch PCI bus driver (11K LOC, DeepSeek+c2rust). NOT boot-loaded → link-clean ×3 is the bar. |
| **acpi.sys** | drivers/bus/acpi | ✅ LANDED (master) | acpi-rust | ReactOS glue → Rust; Intel ACPICA stays C (FFI-to-blob via acpica-sys). NOT a full C-translate. |
| **i8042prt / kbdclass / mouclass** | drivers/input | ✅ LANDED (master) | *-rust/*-cargo | PS/2 + input class drivers. arm64input.sys = custom ARM64 virtio (already Rust-era). |
| **swenum.sys** | drivers/ksfilter/swenum | ✅ LANDED (master) | swenum-rust | Software bus enumerator (smallest; c2rust-pipeline validation). |
| **usbd.sys** | drivers/usb/usbd | ✅ LANDED (master) | usbd-rust | USB descriptor parsers + bandwidth (export driver). usbdex helper also Rust. |
| **pl011.sys** | drivers/serial/pl011 | ✅ LANDED (master) | pl011-rust | ARM64 PL011 UART. |
| **processr.sys** | drivers/processor/processr | ✅ LANDED (master) | processr-rust | Processor/CPU driver. |
| **hidparse.sys** | drivers/hid/hidparse | ✅ LANDED (master) | hidparser-rust | HID report-descriptor parser static lib + wrapper. |
| **kdcom / kdgdb / kdvm** | drivers/base/* | ✅ LANDED (master) | kdcom-serial-rust | KD transports (non-MSVC kdcom = kdrosdbg). |
| **wmilib** | drivers/wmi | ✅ LANDED (master) | (wmilib) | WMI leaf. |
| **beep / null / condrv / nmidebug / pciide** | drivers/* | ✅ LANDED (master) | *-rust/*-cargo | Small leaf .sys drivers. |
| **disk.sys** | drivers/storage/class/disk | ✅ LANDED (master) | disk-rust | SCSI disk class driver (12.6K LOC), WDM. Full C→Rust via DeepSeek+c2rust. Cohab. Cold smoke ×3. Merged f4900f16109. |
| **partmgr.sys** | drivers/storage/partmgr | ✅ LANDED (master) | partmgr-rust | WDM, boot-loaded. Full C→Rust via DeepSeek+c2rust (~$0.20, 39 fns). Cohab (guid.c stays C). cargo-clean ×3, links ×3, **smoke i386+amd64+arm64 all GREEN**. Review: 0 CRITICAL/HIGH. Added as a rustconv target + gen_driver_types.sh case. Cherry-pick a78e0bba00e. |
| **pciidex.sys** | drivers/storage/ide/pciidex | ✅ LANDED (master) | pciidex-rust | PCI IDE bus extension, WDM **export driver** (PciIdeXInitialize/GetBusData/SetBusData/DebugPrint). Full C→Rust (fdo/miniport/pciidex/pdo/power.c, ~1.9K) via DeepSeek+c2rust (~$0.22). Cohab (rust_cohab_stub.c, spec2def kept). cargo-clean ×3, links ×3 (4 exports verified), **cold smoke i386+amd64 ✓ boot (desktop, PnP-exercised); arm64 cold-link ✓** (not on arm64 boot path — no PCI IDE). Caught match-on-const binding hazard (7 IRP_MN_*) + OB_OPEN_REASON mistranslation. Link traps: HalGetInterruptTranslator=HALDISPATCH-> macro, vDbgPrintEx=@stdcall, ExAllocatePoolZero/Ke/Ex FORCEINLINEs. |
| **classpnp.sys** | drivers/storage/class/classpnp | ✅ LANDED (master) | classpnp-rust | WDM, ~166K lines (35 `.rs`). cmake drives the cargo link (`add_cargo_module(classpnp classpnp-cargo classpnp.sys KIND kerneldll BODY_CRATE classpnp-rust)`); `classpnp-rust` is the body. Storage-class DDK shim substrate (SRB/sense/io-ke-mm inlines + class GUIDs). On the LiveCD read-stack. |
| **cdfs.sys** | drivers/filesystems/cdfs | ✅ LANDED (master) — `b92f3364841`, `aeb14a9055c` "cdfs 100% bodied, zero stubs" | cdfs-rust | WDM CD filesystem (35 `.rs` / ~65K LOC, 27/29 orig .c → Rust). Boots i386+amd64 cold to shell ready & arm64 to desktop. LiveCD read-stack leg. |
| **cdrom.sys** | drivers/storage/class/cdrom | ✅ LANDED (master) — SHIPPED TO MAINLINE `c2839ebaf5b` (2026-06-16) | cdrom-rust | **KMDF** — the fleet's only KMDF driver. `kmdf-sys` WDF-dispatch substrate + full C→Rust cohab (18 `.rs` / ~211K LOC; 11 `.c` bodies flipped). Boot-loaded on the LiveCD read-stack. Boots i386+amd64 to shell ready & arm64 to desktop. cdrom lane closed. |
| **mountmgr.sys** | drivers/storage/mountmgr | ✅ LANDED (master) | mountmgr-rust | Mount manager (9.4K, DeepSeek+c2rust, $0.87). Boot-start WDM, cohab, **cold smoke ×3 ✓ boot**. 4 runtime bugs fixed (empty RTL_CONSTANT_STRINGs, PWSTR/UNICODE_STRING, AllocatePool width, element-vs-byte overflow). |
| **scsiport.sys** | drivers/storage/port/scsiport | ✅ LANDED (master) | scsiport-rust | SCSI port driver (8.4K, 8 .c, DeepSeek+c2rust ~$0.67), WDM **export driver** (45 ScsiPort* exports; Notification/DebugPrint cdecl varargs). **Boot-loaded + on the LiveCD IDE read-stack** (uniata miniport) → fully boot-exercised. Cohab (guid.c stays C, sptilib kept). cargo-clean ×3, links ×3 (45/45 exports), **cold smoke ×3 ✓ boot**. 7 runtime bugs fixed (element-vs-byte ×3, lost `PortConfig+1`, ZST-array ptr `.add()`, live-DPRINT slowdown, SCAN_IN_PROGRESS fabricated as 0x1=FROZEN_QUEUE wedge, Rust-ABI fn-addr escape, hand-rolled va_arg on amd64) — all catalogued in the field manual. |
| **fs_rec.sys** | drivers/filesystems/fs_rec | ✅ LANDED (master) | fs_rec-rust | FS recognizer (~2.8K LOC, 11 modular files), **boot-loaded** → smoke-exercised. WDM, imports ntoskrnl+hal only. DeepSeek+c2rust (~$0.12). cold smoke ×3 ✓ boot. Merged 1ac0c71839d. |
| **atapi.sys** | drivers/storage/ide/atapi | ✅ LANDED (master) | atapi-rust | NT4-DDK ATAPI IDE SCSIPORT miniport (6.5K LOC, single TU, ~30 fns). NOT in txtsetup (live IDE path is uniata.sys) → **link-clean ×3 is the bar**. HAND-translated (link-only-verified ⇒ hand > DeepSeek) via **5-way parallel Claude subagents** over a shared signature-manifest contract — see [[feedback_parallel_subagent_translation]]. cargo-clean ×3 + links ×3 (i386 `_DriverEntry@8`, amd64/arm64 `DriverEntry`). Cohab (atapi.c kept on disk). New gen_driver_types.sh `atapi` case (+`BUILD_ROOT` override). Merged ab471845959. |
| **mup.sys** | drivers/filesystems/mup | ✅ LANDED (master) | mup-rust | Multiple UNC Provider (~2.6K, mup.c + dfs.c). WDM, imports ntoskrnl+hal only. **HAND translation** ($0 API), not LLM — every const header-harvested, every union field path from the ntkrnl-sys bindgen substrate. Private node structs (MUP_VCB/FCB/CCB/MIC/UNC/PFX/MQC) in-crate (types.rs). PSEH→DUMMY_PSEH lowering: TRY inline / FINALLY-always-runs (labeled block) / EXCEPT dropped / `ExRaiseStatus -> !`. cargo-clean ×3, links ×3 (45 KB), **cold smoke ×3 ✓ boot** (loads as builtin driver on i386/amd64, boot continues clean to desktop). 3 cross-arch link traps catalogued in field manual (ExRaiseStatus→RtlRaiseStatus link-name, amd64 SharedUserData shim, arm64 KeGetCurrentPrcb forward). |
| **uniata.sys** | drivers/storage/ide/uniata | ✅ LANDED (master) | uniata-rust | The live PNP0600 IDE/ATAPI miniport — first **C++→Rust** translation (tree-sitter L2 track). Cmake builds uniata-rust; C/C++ bodies kept on disk (never-delete). `ros_glue.cpp` stays as the C++ TU glue. On the LiveCD IDE read-stack → boot-exercised. Post-merge cold ×3 green. |
| **fastfat.sys** | drivers/filesystems/fastfat | ✅ LANDED (master) | fastfat-rust | FAT FS — full **34-`.c` cohab flip** (40 `.rs` / ~113K LOC). The live normal-FAT driver (fs_rec routes FAT→fastfat); boot media (USB/ramdisk). Cohab via `rust_cohab_{stub,forceinline,fat}.c` (real-header DATA/FORCEINLINE providers). **Boots arm64 to the desktop ×3.** 11 fastfat bugs fixed across the port (ChicagoMode offset-0, ke/arm64 KiDispatchException X19–X28 bounds, DUMMY_PSEH except bodies, …). |
| **msfs.sys** | drivers/filesystems/msfs | ✅ LANDED (filesystems lane, `a4e1e1ff8e1`) | msfs-rust | Mailslot IPC FS (~1.2K, 6 .c). NOT boot-loaded → link-clean ×3 is the bar. Cohab, mirrors mup. |
| **npfs.sys** | drivers/filesystems/npfs | ✅ LANDED (filesystems lane, `84e2b9adbc6`) | npfs-rust | Named-pipe IPC FS (~6.7K, 20 .c). NOT boot-loaded → link-clean ×3. Cohab; 20 modules translated by parallel subagents over a shared types/glue substrate, integrated centrally. |
| nfs / udfs / vfatfs | drivers/filesystems/* | ⏭️ DROPPED from translation scope | — | nfs = imported UMich ms-nfs41-client (→ userland later); udfs = UDF C++ DVD/BD (niche, non-boot); vfatfs = FATX/Xbox-only here (fs_rec routes FATX→vfatfs; normal FAT→fastfat). None are targets for the 3 QEMU arches. See rust-port-status-rollup.md "Filesystems". |
| **storport.sys** | drivers/storage/port/storport | ✅ LANDED (master) | storport-rust | Storport port driver (~65K LOC, 12 `.rs`). Cohab — `fdo/miniport/misc/pdo/storport/stubs.c` flipped (`# replaced by Rust`); `guid.c` stays C. NOT boot-loaded in smoke → link-clean ×3 is the bar. |
| **storahci.sys** | drivers/storage/port/storahci | ✅ LANDED (master) | storahci-rust | AHCI storport miniport (~123K LOC, 7 `.rs`). Cohab (`storahci.c` flipped, `rust_cohab_stub.c`); links storport. NOT the live LiveCD path (IDE=uniata) → link-clean ×3. |
| **serial.sys** | drivers/serial/serial | ✅ LANDED (master) | serial-rust | 16550 serial port driver (~7.9K LOC, 18 `.rs`). Cohab (bodies flipped, `guid.c` stays C). NOT boot-loaded in smoke → link-clean ×3. |
| **ramdisk.sys** | drivers/storage/class/ramdisk | ✅ LANDED (master) | ramdisk-rust | Pure-WDM RAM-disk class driver (~7.1K LOC, 7 `.rs`). Cohab (`ramdisk.c` flipped, `rust_cohab_stub.c`). |
| **hidclass.sys** | drivers/hid/hidclass | ✅ LANDED (master) | hidclass-rust | HID class driver (~2.8K LOC). Cohab — `fdo.c/hidclass.c/pdo.c/guid.c` flipped. |
| **hidusb.sys** | drivers/hid/hidusb | ✅ LANDED (master) | hidusb-rust | HID-over-USB minidriver (~2.5K LOC). Cohab (`hidusb.c` flipped). |
| **kbdhid.sys / mouhid.sys** | drivers/hid/{kbdhid,mouhid} | ✅ LANDED (master) | kbdhid-rust / mouhid-rust | HID keyboard / mouse mapper drivers (~1.7K / ~2.0K LOC). Cohab (bodies flipped). |
| **fltmgr.sys** | drivers/filters/fltmgr | ✅ LANDED (master) | fltmgr-rust | Filesystem Filter Manager (~17.8K LOC, 16 `.rs`). Cohab — 10 `.c` bodies flipped. |
| **ksecdd.sys** | drivers/crypto/ksecdd | ✅ LANDED (master) | ksecdd-rust | Kernel Security Support Provider (~3.6K LOC, 11 `.rs`). Cohab — 5 `.c` bodies flipped. Boot-loaded. |
| **bootvid.dll / framebuf** | drivers/base/bootvid · drivers/video/framebuf | ✅ LANDED (master) | bootvid-rust / framebuf-rust | Boot video. bootvid is cargo-linked (`add_cargo_kernel_dll … BODY_CRATE bootvid-rust`); framebuf-rust crate present. |
| **arm64vid.sys** | drivers/video/arm64vid | ✅ LANDED (master) | arm64vid-rust | Custom ARM64 framebuffer video (cargo-linked `add_cargo_module … BODY_CRATE arm64vid-rust`). |
| **netstack.sys** | drivers/network/netstack | ✅ LANDED (master) | netstack-rust | In-kernel **smoltcp** network stack — a from-scratch Rust *rewrite* (M0–M6: virtio-net → ARP/ICMP/TCP → DHCP/DNS → TDI \Device\{Tcp,Udp,Ip}), NOT a C→Rust translation. Replaces the dropped C tcpip/lwIP. Landed `c4911ea7c68`; amd64 net suite 11/11 incl real public-internet TCP. |

**Helper libs (in `static-libs-translation-audit.md`):** csq-rust (Cancel-Safe
Queue), arbiter-rust (resource arbiter, linked by pcix), ntoskrnl_vista.

## Storage read-stack — the cdrom KMDF question (RESOLVED)

The LiveCD boot read-stack is **ISO9660 → cdrom.sys → classpnp.sys → cdfs.sys**, and
**all three legs are now Rust on master** (cdrom-rust, classpnp-rust, cdfs-rust).

**cdrom was the only KMDF driver in the fleet** — its translation makes 112 distinct
`Wdf*` calls that real KMDF dispatches through `WdfFunctions[idx](WdfDriverGlobals,…)`.
The greenfield **`kmdf-sys`** WDF-dispatch substrate (each Wdf fn = a thin wrapper
indexing the version-stamped function table) was built, cdrom-rust stitched on top, and
the whole driver SHIPPED TO MAINLINE (`c2839ebaf5b`, 2026-06-16). The classpnp
storage-DDK substrate (SRB/sense/class+mountdev GUIDs) it shares is also landed. The
KMDF category is open. Detail: [[project_cdrom_i386_llvm_ud2_readpath]].

## Candidate targets (remaining in-scope)

The realistic C→Rust driver fleet is essentially complete (50/~52). What's left:

- **buslogic.sys** (drivers/storage/port/buslogic) — legacy BusLogic SCSI HBA miniport,
  C-only, not on any QEMU boot path. Small, low priority.
- **ks.sys** (drivers/ksfilter/ks) — the kernel streaming class library. Large; only
  relevant once the audio stack is in scope.

The **ide/atapi** stack is fully Rust (pciide + pciidex + atapi + the live **uniata**
C++→Rust miniport). All filesystems on the boot/data path are Rust (cdfs, fastfat,
fs_rec, mup, msfs, npfs).

## NOT recommended / out of scope (imported or vendored C, keep as-is)

| Driver(s) | Reason |
|---|---|
| drivers/network/tcpip (lwIP) | ~115K lines of imported lwIP — **superseded** by the smoltcp `netstack-rust` rewrite (landed `c4911ea7c68`), not C-translated. The lwIP tree stays on disk but is no longer the live stack. |
| drivers/network/{ndis,ndisuio,netio,tdi,afd} + dd/{e1000,netkvm,rtl8139} | NDIS/legacy-net + miniports — superseded by / orthogonal to the smoltcp netstack; not translation targets. |
| drivers/bus/acpi (ACPICA core) | The Intel ACPICA C library (`acpica/**`, 164 files → libacpica.a) is **vendored by policy** — NOT a translation target; only the ReactOS glue (osl/bus/pnp/power…) is Rust (acpi-rust + acpica-sys bindgen, done). |
| WDM **audio** stack (portcls, ac97, CMIDriver, drmk, kmixer, hdaudbus, wdmaud, sysaudio) | Not on any QEMU smoke path; out of scope. |
| USB host-controller / hub / function (usbehci, usbohci, usbuhci, usbport, usbhub, usbccgp, usbstor, usbaudio) | Large USB stack, not exercised by smoke; out of scope. (usbd descriptor lib IS Rust.) |
| setup/blue, kdrosdbg, acpi_new, usbstor_new | Helper/legacy/duplicate dirs, not separate translation targets. |

> **btrfs / ext2 / ntfs filesystem drivers were DELETED** (commit `14a0563b291`,
> 2026-06-09) — they were imported 3rd-party trees (WinBtrfs / Ext2Fsd) and are no
> longer on disk, so they are neither remaining-C nor translation targets. ~16 dead
> `baseaddress_{ubtrfs,uext2,untfs,shellbtrfs}` constants still linger in
> `sdk/cmake/baseaddress*.cmake` (harmless dead refs, no module consumes them).

## Patterns & infra

- Cohab recipes (kerneldriver / KernelDll / standalone .sys / static-lib),
  gen_driver_types.sh per-driver bindgen substrate, driver_ddk shared FFI:
  `rust-in-kernel.md` + [[project_driver_private_bindgen]] + [[feedback_driver_ddk_substrate]].
- Per-driver memories: [[project_pci_driver_rust]] (pci+pcix), [[project_cdrom_i386_llvm_ud2_readpath]] (cdrom/KMDF),
  [[project_acpica_ffi]] (acpi), [[project_driver_repair_pass]] (LLM-crate repair),
  [[project_fastfat_rust_track]] (fastfat), [[project_smoltcp_netstack_design]] (netstack).
