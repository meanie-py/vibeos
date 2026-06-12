# Driver Translation Audit

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
| **classpnp.sys** | drivers/storage/class/classpnp | 🟡 IN-FLIGHT | classpnp-rust | WDM. Branch `classpnp-port`, ~landing. Building a storage-class DDK shim substrate (SRB/sense/io-ke-mm inlines + class GUIDs). See `classpnp-cohab-link-gap.md` (branch-local). |
| **cdfs.sys** | drivers/filesystems/cdfs | 🟡 IN-FLIGHT (idle) | cdfs-rust | WDM CD filesystem. Branch `cdfs-port` (33 files committed + ~10 uncommitted grind). LiveCD read-stack leg. |
| **cdrom.sys** | drivers/storage/class/cdrom | 🔴 BLOCKED (banked) | cdrom-rust | **KMDF** — see the storage read-stack section below. Banked on `cdrom-port`. |
| **partmgr.sys** | drivers/storage/partmgr | 🟢 IN-FLIGHT (smoke-verifying) | partmgr-rust | WDM, boot-loaded. Full C→Rust via DeepSeek+c2rust (~$0.20, 39 fns). Cohab (guid.c stays C). cargo-clean ×3, links ×3, smoke i386+arm64 GREEN, amd64 verifying. Review: 0 CRITICAL/HIGH. Added as a rustconv target + gen_driver_types.sh case. |
| mountmgr | drivers/storage/mountmgr | ⚪ not started | — | Drive-letter/mount mgr (~9.4K LOC), boot-critical storage infra. |
| fastfat.sys | drivers/filesystems/fastfat | ⚪ not started | — | FAT FS (68.8K LOC). Boot media (USB/ramdisk). |

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

- **fastfat.sys** (drivers/filesystems/fastfat, 34 files / 68.8K) — FAT FS, boot
  media. Classic kernel-bug surface; well-specified.
- **mountmgr.sys** (drivers/storage/mountmgr, 7 files / ~9.4K) — drive-letter / mount
  mgmt, boot-critical storage infra. (partmgr.sys is DONE — see fleet table.)
- **disk.sys** — the other classpnp class driver (sibling to cdrom; WDM).

## NOT recommended (imported C, keep as-is)

| Driver | Reason |
|---|---|
| drivers/network/tcpip (lwIP) | 115K lines of imported lwIP — upstream C; replace via smoltcp rewrite ([[project_smoltcp_netstack_design]]), not C-translate |
| drivers/bus/acpi (ACPICA core) | The Intel ACPICA library (177 files) stays C; only the ReactOS glue was translated (acpi-rust, done) |
| drivers/filesystems/btrfs | 54 files — imported from WinBtrfs |
| drivers/filesystems/ext2 | 81 files — imported from Ext2Fsd |

## Patterns & infra

- Cohab recipes (kerneldriver / KernelDll / standalone .sys / static-lib),
  gen_driver_types.sh per-driver bindgen substrate, driver_ddk shared FFI:
  `rust-in-kernel.md` + [[project_driver_private_bindgen]] + [[feedback_driver_ddk_substrate]].
- Per-driver memories: [[project_pci_driver_rust]] (pci+pcix), [[project_cdrom_kmdf_blocker]],
  [[project_acpica_ffi]] (acpi), [[project_driver_repair_pass]] (LLM-crate repair).
