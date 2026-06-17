# Driver Translation Audit

> **Ground-truth re-verified 2026-06-14** against each driver's `CMakeLists.txt` + `sdk/lib/rust/*` crates. Corrections this pass: **uniata.sys is DONE** (uniata-rust, first C++→Rust translation, landed on crp — moved out of "candidate targets"); **btrfs/ext2/ntfs drivers were DELETED entirely** (commit 14a0563b291, 2026-06-09 — removed from the "NOT recommended / imported C" table; ~16 dead `baseaddress_{ubtrfs,uext2,untfs,shellbtrfs}` constants linger harmlessly in `sdk/cmake/baseaddress*.cmake`); classpnp/cdfs/cdrom remain IN-FLIGHT on their own branches (not landed).

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
| **pci.sys** | drivers/bus/pci | ✅ LANDED (crp) | pci-rust | The live PNP0A03 root-bus enumerator. Boots ×3. Cohab. Found bindgen PCI_COMMON_CONFIG drop. |
| **pcix.sys** | drivers/bus/pcix | ✅ LANDED (crp) | pcix-rust | Full MS-arch PCI bus driver (11K LOC, DeepSeek+c2rust). NOT boot-loaded → link-clean ×3 is the bar. |
| **acpi.sys** | drivers/bus/acpi | ✅ LANDED (crp) | acpi-rust | ReactOS glue → Rust; Intel ACPICA stays C (FFI-to-blob via acpica-sys). NOT a full C-translate. |
| **i8042prt / kbdclass / mouclass** | drivers/input | ✅ LANDED (crp) | *-rust/*-cargo | PS/2 + input class drivers. arm64input.sys = custom ARM64 virtio (already Rust-era). |
| **swenum.sys** | drivers/ksfilter/swenum | ✅ LANDED (crp) | swenum-rust | Software bus enumerator (smallest; c2rust-pipeline validation). |
| **usbd.sys** | drivers/usb/usbd | ✅ LANDED (crp) | usbd-rust | USB descriptor parsers + bandwidth (export driver). usbdex helper also Rust. |
| **pl011.sys** | drivers/serial/pl011 | ✅ LANDED (crp) | pl011-rust | ARM64 PL011 UART. |
| **processr.sys** | drivers/processor/processr | ✅ LANDED (crp) | processr-rust | Processor/CPU driver. |
| **hidparse.sys** | drivers/hid/hidparse | ✅ LANDED (crp) | hidparser-rust | HID report-descriptor parser static lib + wrapper. |
| **kdcom / kdgdb / kdvm** | drivers/base/* | ✅ LANDED (crp) | kdcom-serial-rust | KD transports (non-MSVC kdcom = kdrosdbg). |
| **wmilib** | drivers/wmi | ✅ LANDED (crp) | (wmilib) | WMI leaf. |
| **beep / null / condrv / nmidebug / pciide** | drivers/* | ✅ LANDED (crp) | *-rust/*-cargo | Small leaf .sys drivers. |
| **disk.sys** | drivers/storage/class/disk | ✅ LANDED (crp) | disk-rust | SCSI disk class driver (12.6K LOC), WDM. Full C→Rust via DeepSeek+c2rust. Cohab. Cold smoke ×3. Merged f4900f16109. |
| **partmgr.sys** | drivers/storage/partmgr | ✅ LANDED (crp) | partmgr-rust | WDM, boot-loaded. Full C→Rust via DeepSeek+c2rust (~$0.20, 39 fns). Cohab (guid.c stays C). cargo-clean ×3, links ×3, **smoke i386+amd64+arm64 all GREEN**. Review: 0 CRITICAL/HIGH. Added as a rustconv target + gen_driver_types.sh case. Cherry-pick a78e0bba00e. |
| **pciidex.sys** | drivers/storage/ide/pciidex | ✅ LANDED (crp) | pciidex-rust | PCI IDE bus extension, WDM **export driver** (PciIdeXInitialize/GetBusData/SetBusData/DebugPrint). Full C→Rust (fdo/miniport/pciidex/pdo/power.c, ~1.9K) via DeepSeek+c2rust (~$0.22). Cohab (rust_cohab_stub.c, spec2def kept). cargo-clean ×3, links ×3 (4 exports verified), **cold smoke i386+amd64 ✓ boot (desktop, PnP-exercised); arm64 cold-link ✓** (not on arm64 boot path — no PCI IDE). Caught match-on-const binding hazard (7 IRP_MN_*) + OB_OPEN_REASON mistranslation. Link traps: HalGetInterruptTranslator=HALDISPATCH-> macro, vDbgPrintEx=@stdcall, ExAllocatePoolZero/Ke/Ex FORCEINLINEs. |
| **classpnp.sys** | drivers/storage/class/classpnp | ✅ LANDED (crp) — confirmed 2026-06-17 rollup (crate + cargo wiring on c-r-p, e.g. `78dd585132c` all-3-arch cohab substrate) | classpnp-rust | WDM, ~166K lines translated (35 `.rs`). cmake drives the cargo link (`add_cargo_module(classpnp classpnp-cargo classpnp.sys KIND kerneldll BODY_CRATE classpnp-rust)`); `classpnp-rust` is the body. Storage-class DDK shim substrate (SRB/sense/io-ke-mm inlines + class GUIDs). |
| **cdfs.sys** | drivers/filesystems/cdfs | ✅ LANDED (crp) — confirmed 2026-06-17 rollup (`b92f3364841`, `aeb14a9055c` "cdfs 100% bodied, zero stubs") | cdfs-rust | WDM CD filesystem (35 `.rs`, 27/29 orig .c → Rust). Boots i386+amd64 cold to shell ready & arm64 to desktop. LiveCD read-stack leg. |
| **cdrom.sys** | drivers/storage/class/cdrom | 🟡 IN-FLIGHT | cdrom-rust | **KMDF** — `kmdf-sys` WDF-dispatch substrate BUILT on `cdrom-port`; links clean amd64 + loads + enumerates. No longer blocked — active lane. |
| **mountmgr.sys** | drivers/storage/mountmgr | ✅ LANDED (crp) | mountmgr-rust | Mount manager (9.4K, DeepSeek+c2rust, $0.87). Boot-start WDM, cohab, **cold smoke ×3 ✓ boot**. 4 runtime bugs fixed (empty RTL_CONSTANT_STRINGs, PWSTR/UNICODE_STRING, AllocatePool width, element-vs-byte overflow). |
| **scsiport.sys** | drivers/storage/port/scsiport | ✅ LANDED (crp) | scsiport-rust | SCSI port driver (8.4K, 8 .c, DeepSeek+c2rust ~$0.67), WDM **export driver** (45 ScsiPort* exports; Notification/DebugPrint cdecl varargs). **Boot-loaded + on the LiveCD IDE read-stack** (uniata miniport) → fully boot-exercised. Cohab (guid.c stays C, sptilib kept). cargo-clean ×3, links ×3 (45/45 exports), **cold smoke ×3 ✓ boot**. 7 runtime bugs fixed (element-vs-byte ×3, lost `PortConfig+1`, ZST-array ptr `.add()`, live-DPRINT slowdown, SCAN_IN_PROGRESS fabricated as 0x1=FROZEN_QUEUE wedge, Rust-ABI fn-addr escape, hand-rolled va_arg on amd64) — all catalogued in the field manual. |
| **fs_rec.sys** | drivers/filesystems/fs_rec | ✅ LANDED (crp) | fs_rec-rust | FS recognizer (~2.8K LOC, 11 modular files), **boot-loaded** → smoke-exercised. WDM, imports ntoskrnl+hal only. DeepSeek+c2rust (~$0.12). cold smoke ×3 ✓ boot. Merged 1ac0c71839d. |
| **atapi.sys** | drivers/storage/ide/atapi | ✅ LANDED (crp) | atapi-rust | NT4-DDK ATAPI IDE SCSIPORT miniport (6.5K LOC, single TU, ~30 fns). NOT in txtsetup (live IDE path is uniata.sys) → **link-clean ×3 is the bar**. HAND-translated (link-only-verified ⇒ hand > DeepSeek) via **5-way parallel Claude subagents** over a shared signature-manifest contract — see [[feedback_parallel_subagent_translation]]. cargo-clean ×3 + links ×3 (i386 `_DriverEntry@8`, amd64/arm64 `DriverEntry`). Cohab (atapi.c kept on disk). New gen_driver_types.sh `atapi` case (+`BUILD_ROOT` override). Merged ab471845959. |
| **mup.sys** | drivers/filesystems/mup | ✅ LANDED (crp) | mup-rust | Multiple UNC Provider (~2.6K, mup.c + dfs.c). WDM, imports ntoskrnl+hal only. **HAND translation** ($0 API), not LLM — every const header-harvested, every union field path from the ntkrnl-sys bindgen substrate. Private node structs (MUP_VCB/FCB/CCB/MIC/UNC/PFX/MQC) in-crate (types.rs). PSEH→DUMMY_PSEH lowering: TRY inline / FINALLY-always-runs (labeled block) / EXCEPT dropped / `ExRaiseStatus -> !`. cargo-clean ×3, links ×3 (45 KB), **cold smoke ×3 ✓ boot** (loads as builtin driver on i386/amd64, boot continues clean to desktop). 3 cross-arch link traps catalogued in field manual (ExRaiseStatus→RtlRaiseStatus link-name, amd64 SharedUserData shim, arm64 KeGetCurrentPrcb forward). |
| **uniata.sys** | drivers/storage/ide/uniata | ✅ LANDED (crp) | uniata-rust | The live PNP0600 IDE/ATAPI miniport — first **C++→Rust** translation (tree-sitter L2 track). Cmake builds uniata-rust; C/C++ bodies kept on disk (never-delete). `ros_glue.cpp` stays as the C++ TU glue. On the LiveCD IDE read-stack → boot-exercised. Post-merge cold ×3 green. |
| fastfat.sys | drivers/filesystems/fastfat | ⚪ not started (active on fastfat-port) | — | FAT FS (68.8K LOC). Boot media (USB/ramdisk). |

**Helper libs (in `static-libs-translation-audit.md`):** csq-rust (Cancel-Safe
Queue), arbiter-rust (resource arbiter, linked by pcix), ntoskrnl_vista.

## Storage read-stack — the cdrom KMDF question (TRACKED DECISION)

The LiveCD boot read-stack is **ISO9660 → cdrom.sys → classpnp.sys → cdfs.sys**.
cdrom.sys IS boot-loaded (`cdrom.sys = 1` in txtsetup.sif; CriticalDeviceDatabase
`GenCdRom→cdrom`), so a translated cdrom WOULD be smoke-exercised — unlike pcix.

**cdrom is the only KMDF driver in the fleet.** Its translation makes **112 distinct
`Wdf*` calls as direct `extern "C"`** — but real KMDF has no such exports; they
dispatch through `WdfFunctions[WdfXxxTableIndex](WdfDriverGlobals, …)` (`wdfldr` =
loader only). So all 112 are undefined at link without a greenfield **`kmdf-sys`**
WDF-dispatch substrate (~bounded, mechanical: each Wdf fn = a thin wrapper indexing
the version-stamped function table). cdrom is ~93K LOC of verbose c2rust output to
stitch *on top of* that substrate. Detail: [[project_cdrom_kmdf_blocker]].

**classpnp overlap (analyzed 2026-06-10):** cdrom is KMDF-native, NOT classpnp-based
(it does not call classpnp's `Class*` exports). BUT it shares ~180 storage/SCSI/SRB/
sense/MOUNTDEV types with what classpnp is building (srb-accessors, sense inlines,
class/mountdev GUIDs). So classpnp's storage-DDK substrate **lowers cdrom's stitch
cost IF landed in a shared location** (storage-ddk / driver_ddk, not classpnp-local)
— but does **not** unblock cdrom: the WDF-dispatch layer is KMDF-only and classpnp
(WDM) builds none of it.

**Sequencing decision:** *let classpnp land first.* Then (a) factor its storage-DDK
substrate shared so cdrom + disk.sys reuse it, (b) decide whether to open the KMDF
category at all (build `kmdf-sys`), (c) only then stitch cdrom. Do NOT build kmdf-sys
speculatively, and flag the classpnp lane now about sharing the substrate to avoid
duplicate SRB/sense harvesting.

## Candidate targets (not yet started)

- **ide / atapi** (drivers/storage/ide) — ATA/ATAPI class stack **fully Rust now**:
  pciide + pciidex + atapi.c miniport + **uniata** (the live PNP0600 C++ driver,
  done — see the LANDED table). Nothing remaining in this stack.
- **fastfat.sys** (drivers/filesystems/fastfat, 34 files / 68.8K) — FAT FS, boot
  media. Classic kernel-bug surface; well-specified. Multi-session.

## NOT recommended (imported C, keep as-is)

| Driver | Reason |
|---|---|
| drivers/network/tcpip (lwIP) | 115K lines of imported lwIP — upstream C; replace via smoltcp rewrite ([[project_smoltcp_netstack_design]]), not C-translate |
| drivers/bus/acpi (ACPICA core) | The Intel ACPICA C library (`acpica/**`, 164 files → libacpica.a) is **vendored by policy** — NOT a translation target; only the ReactOS glue (osl/bus/pnp/power…) is Rust (acpi-rust + acpica-sys bindgen, done) |

> **btrfs / ext2 / ntfs filesystem drivers were DELETED** (commit `14a0563b291`,
> 2026-06-09) — they were imported 3rd-party trees (WinBtrfs / Ext2Fsd) and are no
> longer on disk, so they are neither remaining-C nor translation targets. ~16 dead
> `baseaddress_{ubtrfs,uext2,untfs,shellbtrfs}` constants still linger in
> `sdk/cmake/baseaddress*.cmake` (harmless dead refs, no module consumes them).

## Patterns & infra

- Cohab recipes (kerneldriver / KernelDll / standalone .sys / static-lib),
  gen_driver_types.sh per-driver bindgen substrate, driver_ddk shared FFI:
  `rust-in-kernel.md` + [[project_driver_private_bindgen]] + [[feedback_driver_ddk_substrate]].
- Per-driver memories: [[project_pci_driver_rust]] (pci+pcix), [[project_cdrom_kmdf_blocker]],
  [[project_acpica_ffi]] (acpi), [[project_driver_repair_pass]] (LLM-crate repair).
