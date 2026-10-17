"""The ReactOS/vibeos PROJECT plugin: its translation targets and how to pick one.

Everything here is vibeos-specific DATA and POLICY — the 60-entry target registry, the
path-sniffing that maps a C file to a target, and the ReactOS crate/substrate resolution
rules. None of it is translation machinery, and nothing in the engine should import this
module directly: reach it through `config` (which re-exports lazily) until the plugin
loader lands, then through the loaded project.

Split out of config.py, where it was ~1,600 of the file's 2,164 lines and made the engine
look far more ReactOS-coupled than it actually is. The engine's real dependency on this
tree is a handful of names, listed in the import below.
"""

from __future__ import annotations

import re
from pathlib import Path

from ...config import (
    ACPI_DIR,
    AFD_DIR,
    ARBITER_DIR,
    ARM64INPUT_DIR,
    ATAPI_DIR,
    BINDGEN_FILES,
    BOOTVID_DIR,
    BUSLOGIC_DIR,
    CDFS_DIR,
    CDROM_DIR,
    CLASSPNP_DIR,
    CSQ_DIR,
    DISK_DIR,
    FASTFAT_DIR,
    FLTMGR_DIR,
    FREELDR_DIR,
    FS_REC_DIR,
    HAL_BINDGEN_FILES,
    HAL_DIR,
    HIDPARSER_DIR,
    I8042PRT_DIR,
    KBDCLASS_DIR,
    KDCOM_DIR,
    KDCOM_SERIAL_DIR,
    KDGDB_DIR,
    KDVM_DIR,
    KERNEL32_DIR,
    KS_DIR,
    KSECDD_DIR,
    LEGIDE_DIR,
    LSASRV_DIR,
    MOUCLASS_DIR,
    MOUNTMGR_DIR,
    MSAFD_DIR,
    NDIS_DIR,
    NTDLL_BINDGEN_FILES,
    NTDLL_DIR,
    NTOSKRNL_DIR,
    PARTMGR_DIR,
    PCIIDEX_DIR,
    PCIX_DIR,
    PL011_DIR,
    PROCESSR_DIR,
    RAMDISK_DIR,
    REACTOS_ROOT,
    RUST_DIR,
    SCSIPORT_DIR,
    SERIAL_DIR,
    SHELL32_COM_FILES,
    SHELL32_DIR,
    SHELL32_ISLAND_FILES,
    SHELL_EXPLORER_DIR,
    SMSS_DIR,
    SPEC_FILE,
    STORAHCI_DIR,
    STORPORT_DIR,
    SWENUM_DIR,
    TEMPLATES_DIR,
    USBAUDIO_DIR,
    USBCCGP_DIR,
    USBD_DIR,
    USBEHCI_DIR,
    USBHUB_DIR,
    USBOHCI_DIR,
    USBPORT_DIR,
    USBSTOR_DIR,
    USBUHCI_DIR,
    USERINIT_DIR,
    WIN32K_BINDGEN_FILES,
    WIN32SS_DIR,
    WINLOGON_BINDGEN_FILES,
    WINLOGON_DIR,
    WINSRV_BINDGEN_FILES,
    WINSRV_DIR,
    WS2_32_DIR,
    Target,
)

TARGETS = {
    "ntoskrnl": Target(
        name="ntoskrnl",
        source_dir=NTOSKRNL_DIR,
        output_crate=RUST_DIR / "ntoskrnl-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=SPEC_FILE,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "hal": Target(
        name="hal",
        source_dir=HAL_DIR,
        output_crate=RUST_DIR / "hal-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files=HAL_BINDGEN_FILES,
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt_hal.md",
        src_subdir=Path("src/x86"),
        cmake_deps_file=HAL_DIR / "halx86" / "CMakeLists.txt",
        cmake_source_files=[
            HAL_DIR / "halx86" / "generic.cmake",
            HAL_DIR / "halx86" / "acpi.cmake",
            HAL_DIR / "halx86" / "apic.cmake",
            HAL_DIR / "halx86" / "pic.cmake",
            HAL_DIR / "halx86" / "legacy.cmake",
        ],
        arches=("i386", "amd64"),
    ),
    # Pre-kernel boot DLLs (FreeLDR-loaded siblings of ntoskrnl/hal).
    # Cargo-link end-state: kdcom-rust/bootvid-rust staticlib → DLL via
    # reactos-link (hal-cargo precedent). reactos-link is i386-only today,
    # so arches=("i386",); amd64/arm64 stay C until it goes multi-arch.
    # No cmake cohabitation fields: the cargo-driven link replaces the
    # cmake add_library() target wholesale, it doesn't co-link into it.
    "kdcom": Target(
        name="kdcom",
        source_dir=KDCOM_DIR,
        output_crate=RUST_DIR / "kdcom-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=KDCOM_DIR / "kdrosdbg.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt_freeldr.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386",),
    ),
    # kdgdb — the GDB-stub KD transport (sibling of kdcom). Kernel-mode
    # driver: walks EPROCESS/ETHREAD lists, reads CONTEXT/KTRAP_FRAME, speaks
    # the windbgkd packet protocol — all kernel types from BINDGEN_FILES, so
    # the kernel/freeldr prompt + ntkrnl_sys substrate (same as kdcom). Same
    # 8-export kddll ABI. i386+amd64 only (i386_sup.c/amd64_sup.c; no arm64).
    "kdgdb": Target(
        name="kdgdb",
        source_dir=KDGDB_DIR,
        output_crate=RUST_DIR / "kdgdb-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=KDGDB_DIR / "kdgdb.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt_freeldr.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64"),
    ),
    # serial kdcom — classic NS16550 serial KD transport (sibling of kdgdb).
    # Same 8-export kddll spec, cportlib serial substrate (= cport-rust).
    "kdcom-serial": Target(
        name="kdcom-serial",
        source_dir=KDCOM_SERIAL_DIR,
        output_crate=RUST_DIR / "kdcom-serial-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=KDCOM_SERIAL_DIR / "kdcom.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt_freeldr.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64"),
    ),
    # kdvm — VirtualBox KD transport (builds kdvbox.dll). KdVmExchange asm
    # handled in-crate as global_asm!.
    "kdvm": Target(
        name="kdvm",
        source_dir=KDVM_DIR,
        output_crate=RUST_DIR / "kdvm-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=KDVM_DIR / "kdvm.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt_freeldr.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64"),
    ),
    "bootvid": Target(
        name="bootvid",
        source_dir=BOOTVID_DIR,
        output_crate=RUST_DIR / "bootvid-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=BOOTVID_DIR / "bootvid.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt_freeldr.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386",),
    ),
    # FreeLDR itself — static components compiled into the cmake-built
    # freeldr lib (freeldr.sys/uefildr.efi/rosload.exe), NOT a DLL with
    # a .spec. Integration is hal-rust-style cohabitation: a
    # freeldr-rust staticlib linked into the freeldr cmake target with
    # the .c commented out. 3-arch (i386/amd64/arm64; BIOS+UEFI).
    "freeldr": Target(
        name="freeldr",
        source_dir=FREELDR_DIR,
        output_crate=RUST_DIR / "freeldr-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt_freeldr.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # arm64input.sys — bespoke ARM64 virtio-MMIO kbd+mouse driver
    # (single self-contained arm64-only .c). Kernel-mode driver using
    # ntddk APIs (Io/Mm/Ke/Ps/Rtl) → kernel system_prompt + BINDGEN_FILES,
    # no .spec (only DriverEntry is exported). Integration is hal-rust-
    # style cohabitation: arm64input-rust staticlib linked into the cmake
    # arm64input target with arm64input.c commented out. arm64-only.
    "arm64input": Target(
        name="arm64input",
        source_dir=ARM64INPUT_DIR,
        output_crate=RUST_DIR / "arm64input-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("arm64",),
    ),
    # PS/2 input stack — kernel-mode class/port drivers translated as
    # standalone-.sys cohabitation crates (arm64input-style: ntddk APIs →
    # kernel system_prompt + BINDGEN_FILES, no .spec; only DriverEntry is
    # exported). Unlike arm64input these are live on ALL THREE arches
    # (i386/amd64 PS/2 + arm64 carries them too), so arches is the full set.
    "kbdclass": Target(
        name="kbdclass",
        source_dir=KBDCLASS_DIR,
        output_crate=RUST_DIR / "kbdclass-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "mouclass": Target(
        name="mouclass",
        source_dir=MOUCLASS_DIR,
        output_crate=RUST_DIR / "mouclass-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "i8042prt": Target(
        name="i8042prt",
        source_dir=I8042PRT_DIR,
        output_crate=RUST_DIR / "i8042prt-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # LiveCD read-stack drivers (kernel-mode .sys on the ntkrnl-sys/driver_ddk
    # substrate, same shape as i8042prt): ISO9660 boot path cdrom→classpnp→cdfs.
    "classpnp": Target(
        name="classpnp",
        source_dir=CLASSPNP_DIR,
        output_crate=RUST_DIR / "classpnp-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "cdrom": Target(
        name="cdrom",
        source_dir=CDROM_DIR,
        output_crate=RUST_DIR / "cdrom-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "cdfs": Target(
        name="cdfs",
        source_dir=CDFS_DIR,
        output_crate=RUST_DIR / "cdfs-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # afd.sys — Ancillary Function Driver (drivers/network/afd/afd, ~5.8K LOC:
    # main/connect/read/write/select/listen/event/info/bind/lock/context.c). The
    # kernel keystone of the netstack→winsock vertical: usermode Winsock
    # (ws2_32 → msafd) IRPs land here and are translated into TDI requests down to
    # the transport netstack exposes (\Device\Tcp|Udp|Ip). Not boot-loaded
    # (Start=auto), so the integration bar is cohab link-clean ×3 + a winsock
    # round-trip, not a boot marker. Kernel/TDI types from ntkrnl-sys ddk.
    "afd": Target(
        name="afd",
        source_dir=AFD_DIR,
        output_crate=RUST_DIR / "afd-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # pcix.sys — the full MS-architecture PCI bus driver (drivers/bus/pcix, 11K LOC,
    # arb/intrface/pci subdirs). NOT loaded at boot (pci.sys is the live PNP0A03
    # driver), so link-clean ×3 is the integration bar, not smoke boot. Private
    # structs from gen_driver_types.sh substrate (crate::types); kernel/PnP types
    # from ntkrnl-sys ddk. Links arbiter (Rust) + pciclass.
    "pcix": Target(
        name="pcix",
        source_dir=PCIX_DIR,
        output_crate=RUST_DIR / "pcix-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # disk.sys — the SCSI disk class driver (drivers/storage/class/disk, 12.6K
    # LOC: data/disk/diskwmi/geometry/pnp.c). WDM class driver built on classpnp
    # (imports ClassInitialize/Class* exports + the storage-DDK/SRB/sense
    # substrate landed with classpnp-rust). Boot-loaded (GenDisk→disk). Private
    # structs from gen_driver_types.sh substrate (crate::types); kernel/PnP/SCSI
    # types from ntkrnl-sys ddk.
    "disk": Target(
        name="disk",
        source_dir=DISK_DIR,
        output_crate=RUST_DIR / "disk-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # partmgr.sys — the partition manager (drivers/storage/partmgr, ~2.9K LOC, WDM
    # kernelmodedriver). BOOT-loaded storage infra (drive-letter/partition enum),
    # so it IS smoke-exercised (unlike pcix). Private device-extension structs
    # (FDO_EXTENSION/PARTITION_EXTENSION/BASIC_VOLUME_UNIQUE_ID/DISK_GEOMETRY_EX_
    # INTERNAL) from gen_driver_types.sh substrate (crate::types); MOUNTDEV_*/
    # ioevent / disk-geometry / volume types come from the same allowlisted driver
    # headers; kernel/PnP types from ntkrnl-sys ddk.
    "partmgr": Target(
        name="partmgr",
        source_dir=PARTMGR_DIR,
        output_crate=RUST_DIR / "partmgr-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # fs_rec.sys — the filesystem recognizer (drivers/filesystems/fs_rec, ~2.8K LOC,
    # 11 modular files: one sniffer per FS type + blockdev + fs_rec main). WDM
    # kernelmodedriver, BOOT-loaded (Group "Boot File System", recognizer driver →
    # IoRegisterFileSystem), so it IS smoke-exercised (like partmgr). All private
    # structs (PACKED_BOOT_SECTOR, BIOS_PARAMETER_BLOCK, TAG/EXTENT/AVDP,
    # FILE_SYSTEM_TYPE, DEVICE_EXTENSION + per-FS on-disk superblocks) live in the
    # driver's own fs_rec.h + the 6 per-FS headers (btrfs/cdfs/ext/ffs/reiserfs/udfs.h)
    # → gen_driver_types.sh substrate (crate::types). Kernel/FsRtl/Io types from
    # ntkrnl-sys ddk. Imports ntoskrnl+hal only.
    "fs_rec": Target(
        name="fs_rec",
        source_dir=FS_REC_DIR,
        output_crate=RUST_DIR / "fs_rec-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # mountmgr.sys — the mount manager (drivers/storage/mountmgr, 9.4K LOC, 7
    # files: database/device/mountmgr/notify/point/symlink/uniqueid). Boot-start
    # WDM driver (no .spec, DriverEntry-only). Manages drive letters / mount
    # points / volume symlinks via MOUNTDEV/MOUNTMGR ioctls. Private structs from
    # mntmgr.h (gen_driver_types substrate, crate::types); MOUNTMGR_*/MOUNTDEV_*
    # from the mountmgr/mountdev headers. Sibling of partmgr/disk.
    "mountmgr": Target(
        name="mountmgr",
        source_dir=MOUNTMGR_DIR,
        output_crate=RUST_DIR / "mountmgr-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # fastfat.sys — the FAT filesystem (drivers/filesystems/fastfat, MS-architecture,
    # 35 .c / 68.8K LOC: create/fsctrl/allocsup/fileinfo the giants). WDM kernelmode
    # FS driver, DriverEntry-only (no .spec; fatinit.c). PSEH-dense (create.c 119
    # try-finally) — the SEH/try_return cleanup control flow is the main repair surface,
    # not consts. Private structs (VCB/FCB/CCB/IRP_CONTEXT/PACKED_DIRENT/PACKED_BOOT_
    # SECTOR/FAT_DATA/...) from gen_driver_types substrate (crate::types, 53 structs);
    # FsRtl/cache kernel types via ntkrnl-sys ddk. arm64 = the boot-exercised arch
    # (ramdisk.img is FAT32, the live system volume). Compiles at the 0x502 profile.
    "fastfat": Target(
        name="fastfat",
        source_dir=FASTFAT_DIR,
        output_crate=RUST_DIR / "fastfat-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # pciidex.sys — the PCI IDE bus extension (drivers/storage/ide/pciidex, ~1.9K LOC,
    # 5 .c: fdo/pdo/pciidex/miniport/power). WDM kernelmodedriver + EXPORT driver
    # (pciidex.spec: PciIdeXInitialize/GetBusData/SetBusData stdcall + PciIdeXDebugPrint
    # varargs). BOOT-loaded (pciidex.sys=1) and exercised at boot — the live uniata.sys
    # (C++, PNP0600) AND pciide.sys both link PciIdeX* imports → a Rust pciidex is
    # smoke-exercised. Private extension structs (PCIIDEX_DRIVER_EXTENSION,
    # COMMON/FDO/PDO_DEVICE_EXTENSION) live in pciidex.h (pulls ide.h) → gen_driver_
    # types.sh substrate (crate::types). The 4 exports must survive (spec2def stays in
    # the cohab CMake; Rust provides them #[no_mangle]).
    "pciidex": Target(
        name="pciidex",
        source_dir=PCIIDEX_DIR,
        output_crate=RUST_DIR / "pciidex-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=PCIIDEX_DIR / "pciidex.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # bmfd.dll — the GDI font driver for bitmap (.fon) fonts (win32ss/drivers/font/bmfd).
    # A kerneldll loaded into session space by win32k: it imports only win32k's Eng* surface
    # (winddi.h, which its own island carries) plus libcntpr, so ntkrnl-sys is the substrate.
    "bmfd": Target(
        name="bmfd",
        source_dir=WIN32SS_DIR / "drivers" / "font" / "bmfd",
        output_crate=RUST_DIR / "bmfd-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # ftfd.dll — the GDI font driver for TrueType/OpenType over the vendored FreeType
    # (win32ss/drivers/font/ftfd).
    # A kerneldll loaded into session space by win32k: it imports only win32k's Eng* surface
    # (winddi.h, which its own island carries) plus libcntpr, so ntkrnl-sys is the substrate.
    "ftfd": Target(
        name="ftfd",
        source_dir=WIN32SS_DIR / "drivers" / "font" / "ftfd",
        output_crate=RUST_DIR / "ftfd-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # watchdog.sys — the display watchdog (Wd*) export library
    # (win32ss/drivers/watchdog). A kernelmodedriver over ntoskrnl alone.
    "watchdog": Target(
        name="watchdog",
        source_dir=WIN32SS_DIR / "drivers" / "watchdog",
        output_crate=RUST_DIR / "watchdog-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # atapi.sys — upstream's 2026 ATA port driver (drivers/storage/ide/atapi, ~11K LOC, 15
    # .c: atapi/fdo/pdo/enum/satl/scsi/ioctl/portstate/dev_*/smart/wmi). NOT the NT4-DDK
    # SCSIPORT miniport the name used to mean (that translation is parked as
    # atapi-legacy-rust; its C was deleted under the DDK EULA). A WDM port driver that sits
    # on pciidex's channel PDOs and exposes disks through a SATL. No spec.
    "atapi": Target(
        name="atapi",
        source_dir=ATAPI_DIR,
        output_crate=RUST_DIR / "atapi-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # ndis.sys — the NDIS wrapper (drivers/network/ndis, ~11K LOC, 20 .c): miniport and
    # protocol registration, packet/buffer pools over MDLs, OID requests, interrupts/DPCs,
    # send/receive queues, CoNDIS stubs. EXPORT driver (ndis.spec, ~290 exports that must
    # survive the flip). Exercised end to end only by `--payload ndisprobe` (e1000 82544GC
    # + ndisuio); a plain smoke runs DriverEntry alone.
    "ndis": Target(
        name="ndis",
        source_dir=NDIS_DIR,
        output_crate=RUST_DIR / "ndis-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=NDIS_DIR / "ndis.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # e1000.sys — the Intel PRO/1000 NDIS 5 miniport (drivers/network/dd/e1000, ~1.9K LOC,
    # 6 .c). Imports ndis/ntoskrnl/hal only. Runtime: `--payload ndisprobe` (QEMU e1000
    # 82544GC); the OptiPlex bench's I217-LM is the reason it is being ported. No spec.
    "e1000": Target(
        name="e1000",
        source_dir=REACTOS_ROOT / "drivers" / "network" / "dd" / "e1000",
        output_crate=RUST_DIR / "e1000-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # legide.sys — upstream's legacy (ISA-port) PATA bus enumerator, i386 only
    # (drivers/storage/ide/legide, 1 .c). A pciidex client like pciide; part of the same
    # new ATA stack as pciidex/atapi. No spec.
    "legide": Target(
        name="legide",
        source_dir=LEGIDE_DIR,
        output_crate=RUST_DIR / "legide-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # scsiport.sys — the SCSI port driver (drivers/storage/port/scsiport, ~8.4K LOC,
    # 8 .c: fdo/ioctl/pdo/power/registry/scsi/scsiport/stubs; guid.c stays C = GUID data
    # TU). WDM kernelmodedriver + big EXPORT driver (scsiport.spec: ~45 ScsiPort* — the
    # port/register I/O wrappers → HAL intrinsics, ScsiPortInitialize/GetBusData/etc.
    # stdcall, ScsiPortNotification cdecl varargs + ScsiDebugPrint varargs). BOOT-loaded
    # (scsiport.sys=1) and on the LiveCD IDE read-stack (uniata/atapi are scsiport
    # miniports → import ScsiPort*) → smoke-exercised. Private structs (SCSI_PORT_DEVICE_
    # EXTENSION, _LOGICAL_UNIT_EXTENSION, SCSI_REQUEST_BLOCK_INFO) in scsiport.h →
    # gen_driver_types.sh substrate (crate::types). Links sptilib + PSEH_LIB. The ~45
    # exports must survive (spec2def stays in the cohab CMake; Rust provides #[no_mangle]).
    "scsiport": Target(
        name="scsiport",
        source_dir=SCSIPORT_DIR,
        output_crate=RUST_DIR / "scsiport-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=SCSIPORT_DIR / "scsiport.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # storport.sys — the modern storage port driver (drivers/storage/port/storport, ~3.6K
    # LOC, 7 .c: fdo/miniport/misc/pdo/storport/stubs; guid.c = GUID data TU stays C). EXPORT
    # driver (storport.spec: StorPort* miniport API → storahci links it). WDM kernelmodedriver.
    # NOT boot-loaded in the QEMU smoke path (IDE via uniata/atapi) → link-clean x3 + no
    # regression is the bar. Leaf-driver lane (storport/serial/storahci), opus + c2rust anchor.
    "storport": Target(
        name="storport",
        source_dir=STORPORT_DIR,
        output_crate=RUST_DIR / "storport-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=STORPORT_DIR / "storport.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # storahci.sys — AHCI storport miniport (drivers/storage/port/storahci, ~2.6K LOC, 1 .c).
    # Links storport. Disabled from shipping (re-enable later); translate-for-completeness.
    # No spec. NOT boot-loaded → link-clean x3 bar.
    "storahci": Target(
        name="storahci",
        source_dir=STORAHCI_DIR,
        output_crate=RUST_DIR / "storahci-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # buslogic.sys — BusLogic BT-958 SCSI HBA miniport (drivers/storage/port/buslogic,
    # single 2.8K-LOC TU BusLogic958.c; Log.c/wmi.c are on disk but NOT in the build). A
    # SCSIPORT miniport like atapi: imports ScsiPort* and registers via ScsiPortInitialize.
    # No spec (kernelmodedriver, no exports). NOT on any QEMU boot path (live SCSI/IDE =
    # uniata) → link-clean x3 bar. Driver-private structs (BusLogic_HostAdapter_T/CCB/
    # *Mailbox/_HW_DEVICE_EXTENSION + SRB/CDB/sense/SCSI_WMI) from gen_driver_types.sh
    # substrate (crate::types). Heavy port register I/O + mailbox DMA — c2rust-anchored.
    "buslogic": Target(
        name="buslogic",
        source_dir=BUSLOGIC_DIR,
        output_crate=RUST_DIR / "buslogic-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # serial.sys — 8250/16550 UART COM driver (drivers/serial/serial, ~2.7K LOC, 12 .c).
    # No spec (kernelmodedriver). NOT boot-loaded in smoke (kernel owns the KD COM port) →
    # link-clean x3 bar. Self-contained; the ISR/DPC/timeout files are the hard part.
    "serial": Target(
        name="serial",
        source_dir=SERIAL_DIR,
        output_crate=RUST_DIR / "serial-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # ramdisk.sys — RAM-disk class driver (drivers/storage/class/ramdisk, ~2.5K LOC, 1 .c).
    # EASY leaf: pure WDM class driver — no miniport / no ISR / no DPC / no HW register I/O.
    # No spec (kernelmodedriver). NOT boot-loaded in smoke (boot ramdisk = freeldr, not this)
    # → link-clean x3 bar. Self-contained single .c; driver-private types from ntddrdsk.h.
    "ramdisk": Target(
        name="ramdisk",
        source_dir=RAMDISK_DIR,
        output_crate=RUST_DIR / "ramdisk-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # fltmgr.sys — Filesystem Filter Manager (drivers/filters/fltmgr, ~5.4K LOC, 11 .c:
    # Context/Dispatch/Filter/Interface/Lib/Messaging/Misc/Object/Registry/Volume). The layer
    # IO-manager <-> filesystems where minifilters attach. EXPORT driver (fltmgr.spec: the Flt*
    # API minifilters import). Private structs (FLT_FILTER/FLT_INSTANCE/FLTP_FRAME/
    # STREAM_LIST_CTRL/FLT_PORT_OBJECT) in fltmgrint.h; public API in ddk/fltkernel.h. NTDDI
    # WS03SP1. NOT boot-loaded in smoke -> link-clean x3 bar. Multi-.c cohab (npfs-style traps).
    "fltmgr": Target(
        name="fltmgr",
        source_dir=FLTMGR_DIR,
        output_crate=RUST_DIR / "fltmgr-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=FLTMGR_DIR / "fltmgr.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # ksecdd.sys — Kernel Security Support Provider (drivers/crypto/ksecdd, ~1.6K LOC, 5 .c:
    # ksecdd/dispatch/crypt/random/stubs). The SSPI provider; boot-loaded. EXPORT driver
    # (ksecdd.spec: the stdcall Secur32-style SSPI API). Private KSEC_* structs in ksecdd.h;
    # IOCTL structs in reactos/drivers/ksecdd/ksecioctl.h. Links the already-Rust cryptlib.
    # NOT boot-loaded in smoke -> link-clean x3 bar. Multi-.c export cohab (fltmgr-style).
    "ksecdd": Target(
        name="ksecdd",
        source_dir=KSECDD_DIR,
        output_crate=RUST_DIR / "ksecdd-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=KSECDD_DIR / "ksecdd.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbstor.sys — USB mass-storage class driver (drivers/usb/usbstor, ~3.9K LOC, 9 .c:
    # descriptor/disk/error/fdo/misc/pdo/queue/scsi/usbstor + guid). Bridges the USB stack
    # (URB/USBDI) to the SCSI/disk class — adjacent to the already-Rust disk/classpnp/scsiport
    # storage lane. kernelmodedriver MODULE, no spec (imports ntoskrnl/hal/usbd, links sptilib).
    # Private structs (CBW/CSW/IRP_CONTEXT/FDO+PDO device extensions/
    # STORAGE_ADAPTER_DESCRIPTOR_WIN8) in usbstor.h. NOT boot-loaded in smoke
    # (boots from CD/ramdisk, not USB) -> link-clean x3 bar.
    "usbstor": Target(
        name="usbstor",
        source_dir=USBSTOR_DIR,
        output_crate=RUST_DIR / "usbstor-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbccgp.sys — USB composite-device (generic parent) class driver (drivers/usb/usbccgp,
    # ~3.6K LOC, 7 .c: descriptor/fdo/function/misc/pdo/usbccgp + guid). Splits a composite USB
    # device into per-interface child PDOs. kernelmodedriver MODULE, no spec (imports ntoskrnl/
    # hal/usbd). Private structs in usbccgp.h (pulls hubbusif/usbbusif/usb100/usbdlib). Vista
    # (_WIN32_WINNT=0x600). NOT boot-loaded in smoke -> link-clean x3 bar. USB-only (no SCSI/SRB).
    "usbccgp": Target(
        name="usbccgp",
        source_dir=USBCCGP_DIR,
        output_crate=RUST_DIR / "usbccgp-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbhub.sys — USB hub class driver (drivers/usb/usbhub, ~10.4K LOC, 6 .c: debug/guid/ioctl/
    # pnp/power/usbhub). Enumerates/powers/resets downstream ports, bridges to the USB miniport
    # (usbmport.h) + WMI (wmilib). kernelmodedriver MODULE, no spec (imports ntoskrnl/hal/wmilib/
    # usbd). Many private structs in usbhub.h (USBHUB_FDO_EXTENSION/PORT_PDO_EXTENSION + contexts;
    # pulls usb/usbioctl/hubbusif/usbbusif/usbdlib/ks/usbport/usbmport). NOT boot-loaded in smoke
    # -> link-clean x3 bar. The heaviest USB leaf; do after usbccgp.
    "usbhub": Target(
        name="usbhub",
        source_dir=USBHUB_DIR,
        output_crate=RUST_DIR / "usbhub-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbehci.sys — USB 2.0 EHCI host-controller miniport (drivers/usb/usbehci, ~4.5K LOC,
    # 3 .c: debug/roothub/usbehci + guid). Registers a USBPORT_REGISTRATION_PACKET with usbport
    # (usbmport.h); drives EHCI hardware registers/queue-heads/TDs (hardware.h). kernelmodedriver
    # MODULE, no spec (imports usbport/usbd/hal/ntoskrnl). NOT boot-loaded in smoke -> link-clean
    # x3 bar. First of the 3 host-controller miniports.
    "usbehci": Target(
        name="usbehci",
        source_dir=USBEHCI_DIR,
        output_crate=RUST_DIR / "usbehci-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbohci.sys — USB 1.1 OHCI host-controller miniport (drivers/usb/usbohci, ~3.2K LOC,
    # 2 .c: roothub/usbohci + guid). Same usbport-miniport shape as usbehci; OHCI EDs/TDs
    # (hardware.h). kernelmodedriver MODULE, no spec (imports usbport/usbd/hal/ntoskrnl).
    "usbohci": Target(
        name="usbohci",
        source_dir=USBOHCI_DIR,
        output_crate=RUST_DIR / "usbohci-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbuhci.sys — USB 1.1 UHCI host-controller miniport (drivers/usb/usbuhci, ~3.3K LOC,
    # 2 .c: roothub/usbuhci + guid). Same usbport-miniport shape; UHCI QHs/TDs (hardware.h).
    # kernelmodedriver MODULE, no spec (imports usbport/usbd/hal/ntoskrnl).
    "usbuhci": Target(
        name="usbuhci",
        source_dir=USBUHCI_DIR,
        output_crate=RUST_DIR / "usbuhci-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbport.sys — USB host-controller PORT framework (drivers/usb/usbport, ~17K LOC, 14 .c:
    # debug/device/endpoint/iface/ioctl/iso/pnp/power/queue/roothub/trfsplit/urb/usb2/usbport
    # + guid). The miniport substrate the 3 host-controller miniports (ehci/ohci/uhci) register
    # WITH via USBPORT_RegisterUSBPortDriver; owns the URB queue, endpoint/transfer state, PnP
    # FDO/PDO, roothub PDO, USB2 bandwidth scheduler. kernelmodedriver MODULE WITH a .spec
    # (exports USBPORT_* to the miniports) -> cohab flip must keep usbport.spec/def + the export
    # surface. NOT boot-loaded in smoke -> link-clean x3 bar.
    "usbport": Target(
        name="usbport",
        source_dir=USBPORT_DIR,
        output_crate=RUST_DIR / "usbport-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=USBPORT_DIR / "usbport.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbaudio.sys — USB Audio class driver (drivers/usb/usbaudio, 4 .c / ~3.5K LOC; guid.c
    # stays a C data TU). KS/PortCls audio miniport (imports usbd + ks + portcls), NTDDI Vista.
    # No spec (kernelmodedriver). NOT on any QEMU boot path -> link-clean x3 bar. Private structs
    # (pshpack1 USB-audio descriptors + __DEVICE_EXTENSION__) + KS/PortCls/USB DDK from the
    # gen_driver_types.sh substrate (crate::types). DeepSeek-chat + c2rust, constant-safeguarded.
    "usbaudio": Target(
        name="usbaudio",
        source_dir=USBAUDIO_DIR,
        output_crate=RUST_DIR / "usbaudio-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # ks.sys — Kernel Streaming class library (drivers/ksfilter/ks, 22 .c / ~19K LOC). EXPORT
    # driver (ks.spec: the Ks* API the audio stack imports). The KS keystone. imports ntoskrnl+hal.
    # NOT boot-loaded -> link-clean x3 bar. DeepSeek-chat + c2rust, constant-safeguarded.
    "ks": Target(
        name="ks",
        source_dir=KS_DIR,
        output_crate=RUST_DIR / "ks-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=KS_DIR / "ks.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "sysaudio": Target(
        name="sysaudio",
        source_dir=REACTOS_ROOT / "drivers/wdm/audio/sysaudio",
        output_crate=RUST_DIR / "sysaudio-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        crate_bindgen_files={
            a: RUST_DIR / f"sysaudio-rust/src/types_{a}.rs" for a in ("i386", "amd64", "arm64")
        },
    ),
    "portcls": Target(
        name="portcls",
        source_dir=REACTOS_ROOT / "drivers/wdm/audio/backpln/portcls",
        output_crate=RUST_DIR / "portcls-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=REACTOS_ROOT / "drivers/wdm/audio/backpln/portcls/portcls.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        crate_bindgen_files={
            a: RUST_DIR / "portcls-rust/src" / f"types_{a}.rs" for a in ("i386", "amd64", "arm64")
        },
    ),
    # ── driver residue (Batch 2): standalone kernel-mode drivers on the
    # ntkrnl-sys/ddk substrate, same shape as i8042prt. ──────────────────
    # acpi.sys — ACPI driver glue (ACPICA FFI-to-blob Phase 3). OSL already Rust
    # (acpi-rust/osl.rs); this covers the driver glue (buspdo/bus big files via
    # c2rust). ACPI_* types come from acpica-sys, fixed up at integration.
    "acpi": Target(
        name="acpi",
        source_dir=ACPI_DIR,
        output_crate=RUST_DIR / "acpi-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    "swenum": Target(
        name="swenum",
        source_dir=SWENUM_DIR,
        output_crate=RUST_DIR / "swenum-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # usbd.sys — export driver (usbd.spec); descriptor parsers + bandwidth.
    "usbd": Target(
        name="usbd",
        source_dir=USBD_DIR,
        output_crate=RUST_DIR / "usbd-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=USBD_DIR / "usbd.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # hidparser — the HID report-descriptor parser STATIC LIB (libhidparser_km),
    # 5 files. Flagship c2rust-anchor target (bit-field parsing). Links into
    # hidparse.sys (whose wrapper is already Rust, Batch 1).
    "hidparser": Target(
        name="hidparser",
        source_dir=HIDPARSER_DIR,
        output_crate=RUST_DIR / "hidparser-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # pl011.sys — ARM64 PL011 UART serial driver (single file, 254 LOC).
    # MODULE kerneldriver, imports ntoskrnl + hal.
    "pl011": Target(
        name="pl011",
        source_dir=PL011_DIR,
        output_crate=RUST_DIR / "pl011-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # processr.sys — processor/CPU driver (processr.c + misc.c + pnp.c, 507 LOC).
    # MODULE kerneldriver, imports ntoskrnl + hal; PnP add-device path.
    "processr": Target(
        name="processr",
        source_dir=PROCESSR_DIR,
        output_crate=RUST_DIR / "processr-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # csq — Cancel-Safe IRP Queue static lib (6 IoCsq* exports), kernel-linked.
    "csq": Target(
        name="csq",
        source_dir=CSQ_DIR,
        output_crate=RUST_DIR / "csq-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # wdfldr.sys — the KMDF loader (sdk/lib/drivers/wdfldr), a kernelmodedriver.
    "wdfldr": Target(
        name="wdfldr",
        source_dir=CSQ_DIR.parent / "wdfldr",
        output_crate=RUST_DIR / "wdfldr-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # ntoskrnl_vista — Vista+ kernel APIs as a static lib for disk/cdrom/WDF.
    "ntoskrnl_vista": Target(
        name="ntoskrnl_vista",
        source_dir=CSQ_DIR.parent / "ntoskrnl_vista",
        output_crate=RUST_DIR / "ntoskrnl_vista-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # sptilib — SCSI pass-through helper static lib (atapi, scsiport, usbstor).
    "sptilib": Target(
        name="sptilib",
        source_dir=CSQ_DIR.parent / "sptilib",
        output_crate=RUST_DIR / "sptilib-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # arbiter — PnP resource arbiter static lib (kernel + pcix.sys).
    "arbiter": Target(
        name="arbiter",
        source_dir=ARBITER_DIR,
        output_crate=RUST_DIR / "arbiter-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    ),
    # win32k.sys subsystem. Resolves against BOTH ntkrnl-sys (the
    # ntoskrnl APIs win32k calls) AND win32k-sys (its own internal
    # surface: _WND/_DESKTOP/Int*/Gre*/co_*). win32k-specific translation
    # rules + the Pass-0/2/2b/3 sidecars are injected by prompt_builder
    # when present; the base system prompt stays system_prompt.md.
    "win32k": Target(
        name="win32k",
        source_dir=WIN32SS_DIR,
        output_crate=RUST_DIR / "win32k-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=WIN32SS_DIR / "win32k.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        win32k_bindgen_files=WIN32K_BINDGEN_FILES,
    ),
    # SMSS→CSRSS user-mode spine (smss.exe, csrss.exe, csrsrv.dll,
    # basesrv.dll). User-mode native: bindgen_files = ntdll-sys (the
    # primary surface, scanned as "ddk"), NOT ntkrnl-sys. Per-binary
    # rlib cohabitation; spec_file is per-component so left None here
    # (the assembler/ABI step resolves the right .spec per file).
    "smss-csrss": Target(
        name="smss-csrss",
        source_dir=SMSS_DIR,
        output_crate=RUST_DIR / "smss-csrss-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        # None ⇒ assembler namespaces by C parent dir
        # (smss-csrss-rust/src/{csrlib,csrsrv,smss,basesrv}/{stem}.rs).
        # Required: api.c/init.c/proc.c collide across components under
        # a flat src/. csrss.rs (already at src/csrss.rs, committed,
        # csrss.exe building) is untouched — not re-translated.
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # ntdll.dll — the user-mode NT loader / exception+APC dispatch /
    # rtl-glue body. Same user-mode substrate as smss-csrss (ntdll-sys
    # scanned as "ddk", collapsed to ntdll_sys::nt in postprocess), its
    # own ntdll-rust crate. Per-file cohabitation; the assembler
    # namespaces by C parent dir (nt_0600/ldr/rtl/dispatch/dbg/etw).
    # spec_file=None: ntdll.spec is mostly forwarders + generated
    # syscall stubs and nt_0600 carries its own ntdll_vista.spec, so the
    # ABI step resolves the right .spec per file (mirrors smss-csrss).
    # See docs/engineering/ntdll-translation-audit.md.
    "ntdll": Target(
        name="ntdll",
        source_dir=NTDLL_DIR,
        output_crate=RUST_DIR / "ntdll-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # kernel32.dll — the Win32 base API DLL. Per-file cohabitation into the
    # cmake-built kernel32.dll (libkernel32_rust.a linked in, the .c commented
    # out). Same user-mode substrate as ntdll/smss-csrss: ntdll-sys scanned as
    # "ddk" via NTDLL_BINDGEN_FILES, collapsed to ntdll_sys::nt in postprocess.
    # spec_file=None — WINAPI=stdcall(i386) handled by the user prompt + c2rust
    # anchor (mirrors ntdll). Assembler namespaces by C parent dir
    # (client/, winnls/string/, wine/) → kernel32-rust/src/<dir>/<stem>.rs.
    # proc.c + NLS/string family (locale/lcformat/nls/casemap/collation) done.
    "kernel32": Target(
        name="kernel32",
        source_dir=KERNEL32_DIR,
        output_crate=RUST_DIR / "kernel32-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # msafd.dll — AFD Winsock service provider (next rung above afd.sys on the
    # netstack→winsock vertical). Usermode DLL, same ntdll-sys substrate as
    # kernel32 (Nt*/Rtl* → ntdll_sys::nt); winsock2/WSP + AFD shared.h IOCTL
    # types added per-crate in msafd-rust. Cohab-flips into the cmake-built
    # msafd.dll; validated by the winsock round-trip (test_net.py), not a boot.
    "msafd": Target(
        name="msafd",
        source_dir=MSAFD_DIR,
        output_crate=RUST_DIR / "msafd-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # ws2_32.dll — the WinSock 2.2 library (top of the netstack→winsock vertical,
    # one rung above msafd). Usermode DLL, same ntdll-sys substrate; the winsock/
    # ws2spi surface + the private object model (ws2_32p.h) come from ws2_32-rust's
    # own bindgen `types` module, imported per-file at integration.
    #
    # spec_file is SET (unlike msafd's None) and that is deliberate: ws2_32.spec is
    # the ground truth for all 187 exports, so the pipeline both TELLS the model which
    # fns must be #[no_mangle] and can AUDIT the result against it afterwards
    # (export_audit). A None spec is exactly how ole32 shipped StgCreateDocfile as a
    # private Rust-ABI fn with `_StgCreateDocfile@16` simply absent from the DLL.
    #
    # Validated by the winsock round-trip in test_net.py (a real usermode nettest.exe
    # driving TCP + UDP through ws2_32 → msafd → AFD → netstack), not by a boot — but
    # that oracle only covers the hot path (~12 of 187 exports), so the deterministic
    # auditors carry the breadth.
    "ws2_32": Target(
        name="ws2_32",
        source_dir=WS2_32_DIR,
        output_crate=RUST_DIR / "ws2_32-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        crate_bindgen_files={
            a: RUST_DIR / "ws2_32-rust" / "src" / f"types_{a}.rs"
            for a in ("i386", "amd64", "arm64")
        },
        crate_module="types",
        spec_file=WS2_32_DIR / "ws2_32.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # lsasrv.dll — Local Security Authority server. Own lsasrv-rust crate; same
    # user-mode substrate as kernel32/ntdll (ntdll_sys::nt base, auto-collapsed).
    # The LSAPR_*/SAMPR_*/DSROLER marshalled + policy info types live in
    # lsasrv-sys and are imported per-file at integration (named, no glob clash).
    "lsasrv": Target(
        name="lsasrv",
        source_dir=LSASRV_DIR,
        output_crate=RUST_DIR / "lsasrv-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=LSASRV_DIR / "lsasrv.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # winsrv.dll — USER + Console server (consrv + usersrv + concfg
    # aggregated into winsrv.dll, loaded by csrss). Phase 5 of the
    # SMSS→CSRSS spine. CSR server DLL, NOT a standalone exe — entry
    # is DllMain (not _main), and the *DllInitialization callbacks
    # come from winsrv.spec.
    #
    # FFI substrate is winsrv-sys (a superset of ntdll-sys via
    # `pub use ntdll_sys::nt;`). NT/CSR/SM/BASE surface scanned as
    # "ddk" via NTDLL_BINDGEN_FILES and collapsed to winsrv_sys::nt
    # in postprocess; consrv/usersrv/concfg-specific surface scanned
    # as "winsrv" via WINSRV_BINDGEN_FILES and rendered as
    # winsrv_sys::winsrv (mirrors win32k_sys::win32k).
    "winsrv": Target(
        name="winsrv",
        source_dir=WINSRV_DIR,
        output_crate=RUST_DIR / "winsrv-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=WINSRV_DIR / "winsrv.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        # None ⇒ assembler namespaces by C parent dir
        # (winsrv-rust/src/{init,concfg,consrv,usersrv}/…).
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        winsrv_bindgen_files=WINSRV_BINDGEN_FILES,
        sys_crate="winsrv_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_winsrv.md",
    ),
    # userinit.exe — post-login shell launcher. Plain user-mode GUI exe on the
    # boot path to `shell ready`. Reuses winsrv-sys (broad Win32 superset) but
    # uses the generic user-mode addendum (not the CSR-server winsrv one).
    "userinit": Target(
        name="userinit",
        source_dir=USERINIT_DIR,
        output_crate=RUST_DIR / "userinit-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        # None ⇒ assembler namespaces by C parent dir, but userinit's two
        # files (userinit.c, livecd.c) live flat under base/system/userinit,
        # so they land at userinit-rust/src/{userinit,livecd}.rs.
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        winsrv_bindgen_files=WINSRV_BINDGEN_FILES,
        sys_crate="winsrv_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    # explorer.exe — the desktop shell. Own explorer-rust crate on the winsrv-sys
    # substrate (broad Win32 superset), generic user-mode addendum. Per-file
    # cohabitation into the cmake-built explorer.exe. Leaf-procedural files first
    # (util/settings/startup); the ATL/WTL/COM files need the vtable design in
    # docs/engineering/explorer-rust-translation-plan.md before bulk translation.
    "explorer": Target(
        name="explorer",
        source_dir=SHELL_EXPLORER_DIR,
        output_crate=RUST_DIR / "explorer-rust",
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        # None ⇒ assembler namespaces by C parent dir; explorer's .cpp live flat
        # under base/shell/explorer, so they land at explorer-rust/src/<stem>.rs.
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        winsrv_bindgen_files=WINSRV_BINDGEN_FILES,
        sys_crate="winsrv_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
    "shell32": Target(
        name="shell32",
        source_dir=SHELL32_DIR,
        output_crate=RUST_DIR / "shell32-rust",
        bindgen_files={},
        hal_bindgen_files={},
        spec_file=SHELL32_DIR / "shell32.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt_shell32_com.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        crate_bindgen_files=SHELL32_COM_FILES,
        crate_module="shellcom",
        # Exactly what the two analyze_class call sites used to hard-code, so shell32's
        # behaviour is unchanged by the parameterization.
        com_object_map=SHELL32_DIR / "shell32.cpp",
        com_island_files=SHELL32_ISLAND_FILES,
        sys_crate="crate",
        sys_umbrella=None,
    ),
    # winlogon.exe — logon manager, GINA-independent leaf files. Self-contained
    # winlogon-sys substrate (WLSESSION/GINA layout size-asserted ×3). Generic
    # user-mode prompt addendum. Output to its own winlogon-rust crate.
    "winlogon": Target(
        name="winlogon",
        source_dir=WINLOGON_DIR,
        output_crate=RUST_DIR / "winlogon-rust",
        bindgen_files=WINLOGON_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        # winlogon's .c are flat under base/system/winlogon, so they land at
        # winlogon-rust/src/{setup,environment,...}.rs.
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="winlogon_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
    ),
}

# The plain-C WDM audio stack: kernel modules whose crate is just `<name>-rust` on the bindgen
# island, like wdfldr. mmixer is a static lib linked into both wdmaud.sys and the user-mode
# wdmaud.drv; it includes wdm.h, so it ports as kernel code with a user feature for the DLL.
AUDIO_KERNEL_DIRS = {
    "mmixer": CSQ_DIR.parent / "sound" / "mmixer",
    "wdmaud": REACTOS_ROOT / "drivers/wdm/audio/legacy/wdmaud",
    "kmixer": REACTOS_ROOT / "drivers/wdm/audio/filters/kmixer",
}
for _name, _dir in AUDIO_KERNEL_DIRS.items():
    TARGETS[_name] = Target(
        name=_name,
        source_dir=_dir,
        output_crate=RUST_DIR / f"{_name}-rust",
        bindgen_files=BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=Path("src"),
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
    )


def detect_target(
    c_path: Path, output_crate: str | None = None, sys_crate: str | None = None
) -> Target:
    """Pick a target by sniffing the C path. Falls back to ntoskrnl.

    ``output_crate``/``sys_crate`` are the caller's explicit overrides, forwarded to
    the auto-detected dll/<name> branch below. Without them, a module whose crate
    cannot be guessed (msvcrt -> msvcrt-crt-rust) raised TargetResolveError on the
    auto-detect path with no way to answer it — the flag existed but only the
    explicit --target branch read it."""
    p = c_path.resolve()
    posix = p.as_posix()
    for name in ("sysaudio", "portcls", *AUDIO_KERNEL_DIRS):
        if p == TARGETS[name].source_dir or TARGETS[name].source_dir in p.parents:
            return TARGETS[name]
    if "/drivers/base/kdrosdbg" in posix:
        return TARGETS["kdcom"]
    if "/drivers/base/kdgdb" in posix:
        return TARGETS["kdgdb"]
    if "/drivers/base/kdvm" in posix:
        return TARGETS["kdvm"]
    if "/drivers/base/kdcom" in posix:
        return TARGETS["kdcom-serial"]
    if "/drivers/base/bootvid" in posix:
        return TARGETS["bootvid"]
    if "/boot/freeldr/" in posix:
        return TARGETS["freeldr"]
    # SMSS→CSRSS user-mode spine. winsrv (under /win32ss/) is NOT here
    # — it stays win32k until the Phase-5 ownership gate is resolved.
    if (
        "/base/system/smss" in posix
        or "/subsystems/csr" in posix
        or "/subsystems/win/basesrv" in posix
        or "/sdk/lib/smlib" in posix
    ):
        return TARGETS["smss-csrss"]
    # userinit.exe (post-login shell launcher). Own userinit-rust crate on the
    # winsrv-sys substrate. Keep before any generic base/system fallthrough.
    if "/base/system/userinit" in posix:
        return TARGETS["userinit"]
    # winlogon.exe (logon manager). Own winlogon-rust crate on the self-contained
    # winlogon-sys substrate. Leaf files only — keep before base/system fallthrough.
    if "/base/system/winlogon" in posix:
        return TARGETS["winlogon"]
    # explorer.exe (desktop shell). Own explorer-rust crate on the winsrv-sys
    # substrate. Last boot-path component still fully C++.
    if "/base/shell/explorer" in posix:
        return TARGETS["explorer"]
    # shell32's C++/ATL classes use a dedicated body-only COM pipeline. Keep
    # this before the generic dll/win32 fallback, which is intentionally C-only.
    if "/dll/win32/shell32/" in posix and p.suffix.lower() in (".cpp", ".cc", ".cxx"):
        return TARGETS["shell32"]
    # ntdll.dll body (loader/dispatch/rtl-glue). Its own ntdll-rust crate
    # on the same user-mode substrate. Note: the shared rtl algorithms it
    # links (sdk/lib/rtl/*) are NOT routed here — those are the rtl-rust /
    # rtl-strings crates; only dll/ntdll/'s own .c belong to this target.
    if "/dll/ntdll/" in posix:
        return TARGETS["ntdll"]
    # kernel32.dll body (Win32 base API). Own kernel32-rust crate, same
    # user-mode substrate as ntdll. NLS/string family + client/* etc.
    if "/dll/win32/kernel32/" in posix:
        return TARGETS["kernel32"]
    # msafd.dll body (AFD Winsock service provider). Own msafd-rust crate, same
    # user-mode substrate as kernel32 (ntdll_sys::nt base). winsock2/WSP + AFD
    # shared.h IOCTL types come from msafd-rust's own types, imported per-file.
    if "/dll/win32/msafd/" in posix:
        return TARGETS["msafd"]
    # ws2_32.dll body (the WinSock 2.2 library — top of the netstack→winsock
    # vertical). Own ws2_32-rust crate, same ntdll_sys::nt base; the winsock/ws2spi
    # surface + the ws2_32p.h object model come from ws2_32-rust's bindgen `types`.
    if "/dll/win32/ws2_32/" in posix:
        return TARGETS["ws2_32"]
    # lsasrv.dll body (LSA server). Own lsasrv-rust crate, same user-mode
    # substrate as ntdll (ntdll_sys::nt base). LSAPR_*/DSROLER marshalled types
    # come from lsasrv-sys, imported per-file at integration.
    if "/dll/win32/lsasrv/" in posix:
        return TARGETS["lsasrv"]
    if "/drivers/input/arm64input" in posix:
        return TARGETS["arm64input"]
    if "/drivers/input/kbdclass" in posix:
        return TARGETS["kbdclass"]
    if "/drivers/input/mouclass" in posix:
        return TARGETS["mouclass"]
    if "/drivers/input/i8042prt" in posix:
        return TARGETS["i8042prt"]
    if "/drivers/storage/class/disk" in posix:
        return TARGETS["disk"]
    # afd.sys — keystone of the netstack→winsock vertical (usermode sockets → TDI).
    if "/drivers/network/afd" in posix:
        return TARGETS["afd"]
    if "/drivers/storage/partmgr" in posix:
        return TARGETS["partmgr"]
    if "/drivers/filesystems/fs_rec" in posix:
        return TARGETS["fs_rec"]
    if "/drivers/filesystems/fastfat" in posix:
        return TARGETS["fastfat"]
    if "/drivers/storage/mountmgr" in posix:
        return TARGETS["mountmgr"]
    if "/drivers/storage/ide/pciidex" in posix:
        return TARGETS["pciidex"]
    if "/drivers/storage/ide/atapi" in posix:
        return TARGETS["atapi"]
    if "/drivers/storage/ide/legide" in posix:
        return TARGETS["legide"]
    if "/drivers/storage/port/scsiport" in posix:
        return TARGETS["scsiport"]
    if "/drivers/storage/port/storport" in posix:
        return TARGETS["storport"]
    if "/drivers/storage/port/storahci" in posix:
        return TARGETS["storahci"]
    if "/drivers/storage/port/buslogic" in posix:
        return TARGETS["buslogic"]
    if "/drivers/serial/serial" in posix:
        return TARGETS["serial"]
    if "/drivers/storage/class/ramdisk" in posix:
        return TARGETS["ramdisk"]
    if "/drivers/filters/fltmgr" in posix:
        return TARGETS["fltmgr"]
    if "/drivers/crypto/ksecdd" in posix:
        return TARGETS["ksecdd"]
    if "/drivers/usb/usbstor" in posix:
        return TARGETS["usbstor"]
    if "/drivers/usb/usbccgp" in posix:
        return TARGETS["usbccgp"]
    if "/drivers/usb/usbhub" in posix:
        return TARGETS["usbhub"]
    if "/drivers/usb/usbehci" in posix:
        return TARGETS["usbehci"]
    if "/drivers/usb/usbohci" in posix:
        return TARGETS["usbohci"]
    if "/drivers/usb/usbuhci" in posix:
        return TARGETS["usbuhci"]
    if "/drivers/usb/usbport" in posix:
        return TARGETS["usbport"]
    if "/drivers/usb/usbaudio" in posix:
        return TARGETS["usbaudio"]
    if "/drivers/ksfilter/ks" in posix:
        return TARGETS["ks"]
    if "/drivers/bus/pcix" in posix:
        return TARGETS["pcix"]
    if "/drivers/ksfilter/swenum" in posix:
        return TARGETS["swenum"]
    if "/drivers/usb/usbd" in posix:
        return TARGETS["usbd"]
    if "/sdk/lib/drivers/hidparser" in posix:
        return TARGETS["hidparser"]
    if "/drivers/serial/pl011" in posix:
        return TARGETS["pl011"]
    if "/drivers/processor/processr" in posix:
        return TARGETS["processr"]
    if "/sdk/lib/drivers/csq" in posix:
        return TARGETS["csq"]
    if "/sdk/lib/drivers/wdfldr" in posix:
        return TARGETS["wdfldr"]
    if "/sdk/lib/drivers/ntoskrnl_vista" in posix:
        return TARGETS["ntoskrnl_vista"]
    if "/sdk/lib/drivers/sptilib" in posix:
        return TARGETS["sptilib"]
    if "/sdk/lib/drivers/arbiter" in posix:
        return TARGETS["arbiter"]
    # winsrv (Phase 5) lives under /win32ss/user/winsrv — keep this
    # branch BEFORE the generic /win32ss/ → win32k fallback so the
    # winsrv-specific substrate + addendum get picked up.
    if "/win32ss/user/winsrv" in posix:
        return TARGETS["winsrv"]
    if "/win32ss/" in posix:
        return TARGETS["win32k"]
    if "/hal/halx86" in posix or posix.endswith("/hal") or "/hal/" in posix:
        return TARGETS["hal"]
    # ntoskrnl was previously the silent catch-all; make it an EXPLICIT match so
    # the fallback below can be a hard error instead of misrouting to ntoskrnl.
    if "/ntoskrnl/" in posix or posix.endswith("/ntoskrnl"):
        return TARGETS["ntoskrnl"]
    # Standalone user-mode DLLs (dll/win32/<name>/, dll/<name>/) → <name>-rust,
    # auto-derived from the path. (ntdll is matched earlier.) This is what keeps
    # e.g. dll/win32/mscms from silently landing in ntoskrnl-rust.
    dll = _dll_crate_from_path(posix)
    if dll:
        return make_user_dll_target(*dll, output_crate=output_crate, sys_crate=sys_crate)
    # No more silent ntoskrnl fallback: an unrecognized path is an ERROR — pass
    # --target <known> or place the file under a known root. (Misrouting paid
    # translations into the wrong crate is the bug this prevents.)
    raise ValueError(
        f"rustconv: cannot auto-detect a target for {p}. Pass --target <name> "
        f"(known: {', '.join(sorted(TARGETS))}) or move the file under a known root."
    )


def _dll_crate_from_path(posix: str):
    """For a path under dll/, return (crate_name, source_dir) using the convention
    dll/win32/<name>/ → <name>-rust (or dll/<name>/ → <name>-rust). Else None."""
    parts = posix.split("/")
    # base/services/<name>/... → <name>-rust (system services follow the same crate
    # convention as user DLLs; the source dir is base/services/<name>, so files in a
    # deeper subdir like dhcpcsvc/dhcp/ still route to the <name> target).
    if "base" in parts:
        brest = parts[parts.index("base") + 1 :]
        if len(brest) >= 2 and brest[0] == "services" and re.fullmatch(r"[A-Za-z0-9_]+", brest[1]):
            return brest[1], REACTOS_ROOT.joinpath("base", "services", brest[1])
    if "dll" not in parts:
        return None
    rest = parts[parts.index("dll") + 1 :]
    if not rest:
        return None
    if rest[0] == "win32" and len(rest) >= 2:
        name, sub = rest[1], ("dll", "win32", rest[1])
    else:
        name, sub = rest[0], ("dll", rest[0])
    if not re.fullmatch(r"[A-Za-z0-9_]+", name):
        return None
    return name, REACTOS_ROOT.joinpath(*sub)


def _detect_com_method_macro(crate: Path) -> str | None:
    """ "stdmethod" if the target crate defines that macro, else None.

    Detected, not assumed: emitting `stdmethod! { … }` into a crate that lacks the
    macro is a build break, and hardcoding a list of "COM crates" rots. Absent the
    macro, abi_enforce falls back to the inline cfg-dual, which is always correct.
    """
    src = crate / "src"
    if not src.is_dir():
        return None
    for f in src.rglob("*.rs"):
        try:
            if "macro_rules! stdmethod" in f.read_text(errors="replace"):
                return "stdmethod"
        except OSError:
            continue
    return None


def _detect_concrete_stdcall_slots(crate: Path) -> bool:
    """Does this crate's SUBSTRATE spell fn-pointer slots `extern "stdcall"` on i386?

    This decides whether a translated `static` + system function has to be emitted as a
    per-arch `#[cfg]` twin pair. The dual exists because Rust compares fn-pointer ABIs
    NOMINALLY -- `extern "system" fn` unifies with neither `extern "stdcall"` nor
    `extern "C"` -- so a callback could only be stored in a concretely-spelled bindgen slot
    if it carried that arch's literal convention.

    `normalize_island_abi.py` has since respelled the islands `extern "system"` on all three
    arches, which retires the premise: shell32-rust's `island_i386.rs` and com-sys's
    `bindgen_i386.rs` both contain ZERO `extern "stdcall"`. Where that holds the dual is pure
    cost, and not only line count -- only half of it is ever compiled, so a fix to one twin is
    invisible until someone builds the other arch, and it defeats crate-repair's cross-target
    agreement outright (`collapse_abi_twins.py` exists to fold the leftovers).

    DETECTED from the island/bindgen files ONLY, never from translated output: a previous
    run's twins are themselves `extern "stdcall"`, so scanning the whole crate would make this
    self-perpetuating. Absent any island, the answer is the conservative True -- the dual is
    always ABI-correct, merely wasteful.
    """
    src = crate / "src"
    if not src.is_dir():
        return True
    # A NON-recursive glob on the two stems, not a tree walk: every island in the tree sits
    # directly in `<crate>/src`, and this runs at import for every target.
    # `types_<arch>.rs` is gen_driver_types.sh's island (the driver fleet, and every crate
    # `rustconv port` scaffolds). Unlisted, setuplib had "no island", got the conservative
    # answer, and seven NTAPI callbacks came back as stdcall / "C" twins that match neither
    # of its `extern "system"` slots.
    islands = sorted([*src.glob("island_*.rs"), *src.glob("bindgen_*.rs"), *src.glob("types_*.rs")])
    if not islands:
        return True
    from ...crate_integrate import _without_comments

    for f in islands:
        try:
            # Comments masked first. `island_extra.rs` mentions `unsafe extern "stdcall"` in a
            # PROSE note explaining that lstrcpynW is now bound from the real header -- one
            # sentence, no code, and it flipped this answer for the whole crate. A name in
            # prose is not a use (same reason `crate_integrate` masks before its import scan).
            if 'extern "stdcall"' in _without_comments(f.read_text(errors="replace")):
                return True
        except OSError:
            return True
    return False


# The TARGETS above are dict literals, evaluated at import before the detector exists, so they
# cannot call it at their own definition point. Fill it in once here instead of hardcoding a
# per-target answer that would rot the moment an island is respelled.
for _target in TARGETS.values():
    _target.concrete_stdcall_slots = _detect_concrete_stdcall_slots(_target.output_crate)


def _find_dll_spec(name: str, source_dir: Path) -> Path | None:
    """The DLL's own .spec — the ground truth for which fns must be exported.

    This used to be None for every user-mode DLL, so the pipeline had no idea
    which functions were exports: it neither told the model nor enforced
    #[no_mangle]. That is how ole32's `StgCreateDocfile` came back a private
    Rust-ABI fn with `_StgCreateDocfile@16` simply absent from the DLL.
    A few dirs ship more than one .spec (msvfw32, uxtheme, winmm) — <dirname>.spec
    is the one their CMakeLists feeds to spec2def.
    """
    cand = source_dir / f"{name}.spec"
    return cand if cand.is_file() else None


class TargetResolveError(RuntimeError):
    """`--target <name>` could not be PROVEN against the tree.

    Raised instead of falling back to the conventional guess. Every message names
    an override flag that actually exists (`--output-crate` / `--sys-crate`) —
    a fail-loud whose remedy is fictional is worse than no message at all.
    """


def _resolve_output_crate(name: str, override: str | None = None) -> tuple[Path, bool]:
    """Which crate does this DLL's Rust live in? Returns (crate_dir, exists).

    `<name>-rust` is the CONVENTION, not a law: msvcrt's bodies live in
    `msvcrt-crt-rust` (the `-crt-` marks that it owns CRT bodies cohabiting a
    DLL that also keeps its C). Guessing `msvcrt-rust` pointed the whole
    pipeline at a crate that does not exist — assembly would have created a
    phantom crate nothing links.

    So: convention if it exists; an explicit override always wins; a NEW module
    (no crate resembling the name at all) legitimately gets the conventional
    name; and anything ambiguous RAISES rather than picking for you.
    """
    if override:
        crate = Path(override)
        if not crate.is_absolute():
            crate = RUST_DIR / override
        if not crate.is_dir():
            raise TargetResolveError(f"--output-crate {override}: no such directory: {crate}")
        return crate, True

    conventional = RUST_DIR / f"{name}-rust"
    if conventional.is_dir():
        return conventional, True

    # No conventional crate. Is there an unconventionally-named one for this DLL?
    candidates = sorted(
        p for p in RUST_DIR.glob(f"*{name}*-rust") if p.is_dir() and (p / "Cargo.toml").is_file()
    )
    if candidates:
        listed = "\n    ".join(str(p.relative_to(RUST_DIR)) for p in candidates)
        raise TargetResolveError(
            f"--target {name}: the conventional crate `{name}-rust` does not exist, but these\n"
            f"  crates in sdk/lib/rust/ look like they belong to this module:\n\n    {listed}\n\n"
            f"  Refusing to guess — assembling into the wrong crate silently produces Rust that\n"
            f"  nothing links. Pass the right one explicitly:\n\n"
            f"    --output-crate {candidates[0].name}\n"
        )
    # Genuinely new module: the convention is correct, and the crate is created on assemble.
    return conventional, False


def _resolve_sys_name(crate: Path, crate_exists: bool, override: str | None) -> str:
    """The `*-sys` crate name: the override, the convention, or the crate's own Cargo.toml.

    Every path returns a name or raises — there is no None outcome. That was already true
    inline, but the type checker could not see it through the nested branches and read it
    as a reachable None (routed by the fex lane as a possible real bug). It was not one;
    extracting it is how the signature says so instead of a comment claiming it.
    """
    if override is not None:
        return override
    if not crate_exists:
        # New module, no Cargo.toml to read: ntdll-sys is the right default for a
        # standalone user-mode DLL (it is what the user-mode prompt addendum names).
        return "ntdll-sys"
    cargo = crate / "Cargo.toml"
    if not cargo.is_file():
        raise TargetResolveError(f"{crate}: no Cargo.toml — cannot derive the FFI substrate")
    deps = sorted(set(re.findall(r"^\s*([A-Za-z0-9_-]+-sys)\s*=", cargo.read_text(), re.M)))
    if len(deps) == 1:
        # str(): `re.findall` is typed `list[Any]`, so returning the element bare would
        # launder Any through this signature rather than prove it.
        return str(deps[0])
    if not deps:
        raise TargetResolveError(
            f"--target: {crate.name} declares no `*-sys` dependency, so there is no\n"
            f"  FFI substrate to translate against. Add the dependency, or name it:\n\n"
            f"    --sys-crate ntdll-sys\n"
        )
    raise TargetResolveError(
        f"--target: {crate.name} declares several `*-sys` dependencies "
        f"({', '.join(deps)}).\n  Refusing to guess which one is the translation "
        f"substrate. Pick one:\n\n    --sys-crate {deps[0]}\n"
    )


def _derive_sys_substrate(
    crate: Path, crate_exists: bool, override: str | None = None
) -> tuple[str, str | None, dict]:
    """(sys_crate_ident, sys_umbrella, bindgen_files) proven from the tree.

    The FFI substrate used to be hardcoded to ntdll-sys for every standalone DLL.
    That is right for the winsock/net fleet — and wrong for any module carrying its
    own `-sys` island. msvcrt's locale/thread state lives in `msvcrt-sys`
    (a bindgen snapshot double-guarded by two compilers); `msvcrt-crt-rust` does not
    even depend on ntdll-sys, so every `ntdll_sys::` path the model emitted was
    unresolvable by construction — and per the `crate_bindgen_files` contract an
    unresolved name is not a no-op: the model invents a forward declaration that
    compiles, links, and is silently wrong. For msvcrt that means a GUESSED
    `threadmbcinfo`, whose `mbctype[257]`/`mbcasemap[256]` every predicate indexes.

    So read the dependency the crate actually declares. Every pre-existing
    user-DLL crate (ws2_32, iphlpapi, netapi32, dnsapi, wkssvc, …) declares
    exactly `ntdll-sys`, so this returns their historical value unchanged.
    """
    sys_name = _resolve_sys_name(crate, crate_exists, override)
    sys_dir = RUST_DIR / sys_name
    if not sys_dir.is_dir():
        raise TargetResolveError(f"--sys-crate {sys_name}: no such crate: {sys_dir}")

    bindgen = {}
    missing = []
    for arch in ("i386", "amd64", "arm64"):
        f = sys_dir / "src" / f"bindgen_{arch}.rs"
        (bindgen.__setitem__(arch, f) if f.is_file() else missing.append(f))
    if missing:
        raise TargetResolveError(
            f"--target: substrate crate `{sys_name}` is missing per-arch bindgen files:\n    "
            + "\n    ".join(str(m) for m in missing)
        )

    return sys_name.replace("-", "_"), _detect_sys_umbrella(sys_dir, sys_name), bindgen


def _detect_sys_umbrella(sys_dir: Path, sys_name: str) -> str:
    """The module path the substrate re-exports its bindgen under.

    THREE shapes exist in-tree and all must work:
      * `ntdll-sys`  — `pub mod nt { … }`, so imports collapse to `ntdll_sys::nt::*`.
      * `msvcrt-sys` — flat `pub use bindgen_<arch>::*;` at the crate root, so they
        collapse to `msvcrt_sys::*`.
      * `com-sys`    — `include!("bindgen_<arch>.rs")` at the crate root, which splices
        the items in TEXTUALLY. That is flat too — the items really are crate-root
        items, there is no module to name — but it looks nothing like a `pub use` glob,
        so it used to fail as "cannot tell how com-sys re-exports its bindgen" and took
        the whole `menuband` target down with it.

    Returns "" for the flat shape (see `collapse_sys_crate`: "" means *no umbrella
    segment*, and is distinct from None, which means "not a user-mode target at all").
    """
    lib = sys_dir / "src" / "lib.rs"
    if not lib.is_file():
        raise TargetResolveError(f"--target: substrate crate `{sys_name}` has no src/lib.rs")
    src = lib.read_text(errors="replace")
    flat = bool(re.search(r"^pub use (?:self::)?bindgen_\w+::\*\s*;", src, re.M)) or bool(
        re.search(r'^\s*include!\("bindgen_\w+\.rs"\)\s*;', src, re.M)
    )
    umbrellas = re.findall(r"^pub mod (\w+)\s*\{", src, re.M)
    if flat and not umbrellas:
        return ""
    if len(umbrellas) == 1 and not flat:
        return umbrellas[0]
    raise TargetResolveError(
        f"--target: cannot tell how `{sys_name}` re-exports its bindgen "
        f"(root glob: {flat}; pub mod: {umbrellas or 'none'}).\n"
        f"  Expected either one `pub mod <umbrella> {{ … }}` or a flat "
        f"`pub use bindgen_<arch>::*;` at the crate root."
    )


def make_user_dll_target(
    name: str,
    source_dir: Path,
    *,
    output_crate: str | None = None,
    sys_crate: str | None = None,
) -> Target:
    """A self-contained user-mode-DLL target for a `dll/win32/*` module not in TARGETS.

    Everything that used to be ASSUMED here is now PROVEN from the tree, or raises
    `TargetResolveError` naming a real override flag: the output crate
    (`_resolve_output_crate`) and the FFI substrate + its bindgen files and umbrella
    (`_derive_sys_substrate`). Behaviour for every pre-existing target is unchanged —
    they all follow the `<name>-rust` + `ntdll-sys` convention this used to hardcode.
    """
    crate, crate_exists = _resolve_output_crate(name, output_crate)
    sys_ident, umbrella, bindgen_files = _derive_sys_substrate(crate, crate_exists, sys_crate)
    return Target(
        name=name,
        source_dir=source_dir,
        output_crate=crate,
        bindgen_files=bindgen_files,
        hal_bindgen_files={},
        spec_file=_find_dll_spec(name, source_dir),
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate=sys_ident,
        sys_umbrella=umbrella,
        system_prompt_addendum=_user_prompt_addendum(name),
        com_method_macro=_detect_com_method_macro(crate),
        concrete_stdcall_slots=_detect_concrete_stdcall_slots(crate),
        # com_object_map stays None: a module outside shell32 has no OBJECT_ENTRY class-object
        # map, its COM classes being created by plain `*_Constructor` free functions.
        com_island_files=_derive_com_islands(crate),
    )


def _derive_com_islands(crate: Path) -> dict:
    """arch → the crate's own bindgen island, for the C++ class analyzer.

    Two stems occur in the tree and both are legitimate: `island_<arch>.rs` when the profile
    sets TYPES_STEM=island (shell32, to leave a hand-written types.rs alone) and
    `types_<arch>.rs`, which is gen_driver_types.sh's default. Probed rather than configured
    because the profile already decided, and a second place to say it is a second place to
    get it wrong. Empty dict when the crate has no island yet — the caller reports that as a
    missing prerequisite rather than proceeding against no layouts.
    """
    src = crate / "src"
    islands = {}
    for arch in ("i386", "amd64", "arm64"):
        for stem in ("island", "types"):
            candidate = src / f"{stem}_{arch}.rs"
            if candidate.is_file():
                islands[arch] = candidate
                break
    return islands


def _user_prompt_addendum(name: str) -> Path:
    """Per-module prompt rules, when the module has idioms the generic user-mode
    addendum does not cover (msvcrt: CRT locale/thread reach, errno discipline)."""
    specific = TEMPLATES_DIR / f"system_prompt_{name}.md"
    return specific if specific.is_file() else TEMPLATES_DIR / "system_prompt_user.md"


def hal_cmake_for_c(c_path: Path) -> Path | None:
    """Map an HAL C source path to the *.cmake file that lists it.

    Most cases follow `hal/halx86/{subsystem}/file.c` → `{subsystem}.cmake`,
    but some files live in one dir and are listed in another cmake (e.g.
    `hal/halx86/generic/timer.c` is listed in `pic.cmake`, not `generic.cmake`).

    Resolution strategy: grep every TARGETS["hal"].cmake_source_files for the
    file path; return whichever lists it.
    """
    hal_target = TARGETS["hal"]
    rel_to_halx86 = None
    try:
        rel_to_halx86 = c_path.resolve().relative_to(HAL_DIR / "halx86")
    except ValueError:
        return None

    rel_str = rel_to_halx86.as_posix()
    for cmake_path in hal_target.cmake_source_files:
        if not cmake_path.exists():
            continue
        text = cmake_path.read_text()
        # Match exact file path on a line (not commented out)
        for line in text.splitlines():
            stripped = line.strip()
            if stripped.startswith("#"):
                continue
            if rel_str in line and not stripped.startswith("#"):
                return cmake_path
    # Fallback: walk up looking for sibling-named cmake
    p = c_path.resolve().parent
    while p != p.parent and "halx86" in p.as_posix():
        cmake = p.parent / f"{p.name}.cmake"
        if cmake.exists():
            return cmake
        p = p.parent
    return None


# Backend selection: "deepseek" (the house translation model), then "claude", then "gemini".
#
# DeepSeek is checked FIRST even though ANTHROPIC_API_KEY is also provisioned, because an
# unflagged `translate` is the common case and it must land on the cheap lane. This used to
# return "claude" whenever ANTHROPIC_API_KEY was set, and since DEFAULT_MODELS had no
# "deepseek" key at all, `-m deepseek` was the ONLY way to reach DeepSeek — a flag the docs
# recommended and nothing enforced. The shell32 tail lane duly drifted: 69 Sonnet runs to 1
# DeepSeek from 2026-08-02, 382 paid calls billed at $3.81 that DeepSeek would have served
# for $0.31 (12.2x). The banner printed the model and price on every one of those runs, so
# visibility was never the missing piece — the DEFAULT was.
#
# Escalation is unaffected: `-m sonnet` / `-m opus` still switch the backend by model prefix
# (see submit_single), which is the documented route for substrate-clean high-band repairs.
# Switching the default re-bills nothing — the resume-cache key (preparation_namespace) is
# built from mode + seal flags and does NOT include the model, so every banked unit still
# replays for $0 regardless of which model bought it.


#: The COM module-lock symbol pair each output crate calls, keyed by crate name.
#:
#: This is a property of the CRATE and not of the --target, because the host DLL is what
#: defines the pair. `menuband-rust` builds a staticlib that links into BOTH shell32.dll and
#: rshell.dll, so it needs a name the two of them agree on: shell32.cpp and
#: base/shell/rshell/misc.cpp each define `MenuBandRustModule{Lock,Unlock}` forwarding to their
#: own `gModule`. Emitting shell32's private `Shell32RustObject*` spelling into that crate links
#: shell32.dll perfectly and breaks rshell.dll -- and not at once, but whenever a later cohab
#: flip first makes an rshell-reachable object pull the offending module out of the archive.
_MODULE_LOCK_BY_CRATE = {
    "menuband-rust": ("MenuBandRustModuleLock", "MenuBandRustModuleUnlock"),
}

#: What a crate with no entry above gets: shell32.dll's own pair.
_DEFAULT_MODULE_LOCK = ("Shell32RustObjectLock", "Shell32RustObjectUnlock")


def module_lock_for_crate(crate: str | None) -> tuple[str, str]:
    """The (lock, unlock) symbol pair the COM scaffold should call in `crate`."""
    return _MODULE_LOCK_BY_CRATE.get(crate or "", _DEFAULT_MODULE_LOCK)
