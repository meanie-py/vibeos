#!/usr/bin/env bash
# gen_driver_types.sh — generate a driver crate's `types.rs` (driver-PRIVATE structs)
# via bindgen, with exact C layouts, for all 3 arches.
#
# WHY: standalone .sys drivers (classpnp, cdrom, cdfs, ...) define private structs in
# their own headers (classp.h, cdromp.h, cdstruc.h) + the public ddk header (classpnp.h).
# These are NOT in the shared ntkrnl-sys kernel bindgen. DeepSeek/LLM translation stubs
# them with placeholder `_dummy: [u8;N]` fields, which breaks every field access. This
# script generates the EXACT Rust layouts instead.
#
# HOW: `--allowlist-file` bounds emission to types DEFINED in the driver's headers; every
# type name the shared ntkrnl-sys bindgen already exports is blocklisted and resolved via
# `pub use ntkrnl_sys::ddk::*;` — so kernel types (DEVICE_OBJECT, IRP, KEVENT, SCSI_*) keep
# a SINGLE identity shared with the rest of the kernel, and only driver-unique structs
# (FUNCTIONAL_DEVICE_EXTENSION, TRANSFER_PACKET, CLASS_*, FCB/VCB/CCB, ...) become local.
#
# Usage: gen_driver_types.sh <driver-name>
#   driver-name ∈ { acpi | classpnp | cdrom | cdfs | pcix | partmgr }  (config table below)
set -euo pipefail

DRIVER="${1:?usage: gen_driver_types.sh <acpi|classpnp|cdrom|cdfs|afd|msafd|ws2_32|wshtcpip|disk|pcix|partmgr|fs_rec|pciidex|scsiport|atapi|buslogic|usbaudio|ks|usbstor|usbccgp|usbhub|usbehci|usbohci|usbuhci|usbport|uniata>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../.." && pwd)"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
NTKRNL_SYS="${SRC}/sdk/lib/rust/ntkrnl-sys/src"
NTDLL_SYS="${SRC}/sdk/lib/rust/ntdll-sys/src"

# --- per-driver config -------------------------------------------------------
# WRAPPER_INC : header(s) the wrapper.h #includes (pulls in the private structs)
# ALLOW_FILES : --allowlist-file regexes (types DEFINED here become local)
# DRIVER_INC  : extra -I for the driver's own dir
# VER_DEFS    : version macros matching the driver's CMakeLists.txt
EXTRA_INC=()
# LANG_ARGS: extra clang language flags. Default empty = C. C-in-.cpp drivers whose
# HEADERS use C++ (default args, bool, anon unions — e.g. uniata) set `-x c++` so the
# parse succeeds; bindgen still emits the plain structs.
LANG_ARGS=()
# SYS_CRATES: extra sibling -sys crates whose bindgen types must ALSO be blocklisted
# (kept a single identity) and re-exported. Format: "crate_name:src_subdir". Only ACPI
# bridges two -sys crates (ntkrnl-sys + acpica-sys), so it's the only user today.
SYS_CRATES=()
# USERMODE: usermode DLL crates (msafd) blocklist against ntdll-sys (the NDK/Native-API
# bindgen) instead of ntkrnl-sys (the kernel-internal bindgen), and re-export
# `pub use ntdll_sys::nt::*;` (PUBLIC — so the translated modules see NT base types via
# `use crate::types::*`) instead of the private kernel `use ntkrnl_sys::ddk::*;` glob.
# Only winsock/WSP/wsahelp/AFD/TDI/msafd-private structs become local. Default 0 = kernel.
USERMODE=0
# BLOCKLIST_FUNCS: drop ALL function declarations from the emitted types module. Needed for
# usermode wrappers whose header (msafd.h) declares the WSP* prototypes the translation
# itself #[no_mangle]-defines — emitting them here is redundant and risks name clashes under
# the flat `use crate::types::*` glob. Default 0 = keep functions (kernel driver behaviour).
BLOCKLIST_FUNCS=0
case "$DRIVER" in
  acpi)
    # ACPI bus driver glue (buspdo.c/bus.c). Private structs live in acpisys.h
    # (device extensions, DEVICE_PNP_STATE) + acpi_bus.h (Linux-style acpi_device*
    # with :1 bitfields). Both pull in ACPICA's acpi.h — those ACPI_* types are
    # blocklisted here and resolved via acpica-sys to keep one identity.
    DRIVER_DIR="${SRC}/drivers/bus/acpi/include"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <acpi.h>\n#include "acpisys.h"\n#include "acpi_bus.h"'
    ALLOW_FILES=( --allowlist-file '.*acpisys\.h' --allowlist-file '.*acpi_bus\.h'
                  --allowlist-file '.*acpi_drivers\.h' --allowlist-file '.*[/\\]list\.h' )
    VER_DEFS=( -DWINVER=0x603 -D_WIN32_WINNT=0x603 -DNTDDI_VERSION=0x06030000
               -DACPI_USE_LOCAL_CACHE -DACPI_USE_SYSTEM_INTTYPES )
    EXTRA_INC=( -I"${SRC}/drivers/bus/acpi/acpica/include" )
    SYS_CRATES=( "acpica-sys:acpica-sys/src" )
    ;;
  classpnp)
    DRIVER_DIR="${SRC}/drivers/storage/class/classpnp"
    # classp.h does NOT transitively reach wmistr.h / ntddstor.h / ntddscsi.h, so the WNODE_*,
    # STORAGE_HW_FIRMWARE_*, BLOCK_DEVICE_RANGE_DESCRIPTOR, POPULATE_TOKEN_HEADER, DEVICE_DATA_SET_*
    # types never get PARSED (allowlist can only filter parsed types, not emit unreached ones).
    # Pull them explicitly so bindgen emits the full local layouts the modules reference.
    WRAPPER_INC=$'#include <classp.h>\n#include <wmistr.h>\n#include <ntddstor.h>\n#include <ntddscsi.h>'
    # classp.h pulls scsi.h/srb.h/ntddstor.h — the storage structs (SRBEX_DATA, STOR_ADDRESS,
    # UNMAP_LIST_HEADER, WNODE_*, MODE_PARAMETER_HEADER, READ_CAPACITY_DATA, ...) the modules
    # use are defined there and are NOT in the ntkrnl-sys kernel bindgen, so allowlist them too.
    ALLOW_FILES=( --allowlist-file '.*classp\.h' --allowlist-file '.*classpnp\.h'
                  --allowlist-file '.*scsi\.h' --allowlist-file '.*srb\.h'
                  --allowlist-file '.*srbhelper\.h' --allowlist-file '.*ntddstor\.h'
                  --allowlist-file '.*ntddscsi\.h' --allowlist-file '.*wmistr\.h'
                  --allowlist-file '.*storduid\.h' --allowlist-file '.*mountdev\.h'
                  --allowlist-file '.*ioevent\.h' )
    VER_DEFS=( -DWINVER=0x602 -D_WIN32_WINNT=0x602 -DNTDDI_VERSION=0x06020000 )
    ;;
  disk)
    # disk.sys — the SCSI disk class driver (drivers/storage/class/disk). Built on
    # classpnp: its FUNCTIONAL_DEVICE_EXTENSION / COMMON_DEVICE_EXTENSION / CLASS_INIT_DATA
    # come from the PUBLIC classpnp.h, and its OWN private structs (DISK_DATA,
    # DISK_GEOMETRY_EX_INTERNAL, UPDATE_DISK_GEOMETRY_WORKITEM_CONTEXT, REGISTER_*) live
    # in disk.h. disk.h does NOT transitively reach wmistr.h/ntddstor.h/ntddscsi.h/
    # ntdddisk.h/ntddvol.h, so pull them explicitly (allowlist can only filter PARSED
    # types) for the WNODE_*/DISK_GEOMETRY_EX/VOLUME_* layouts the modules reference.
    DRIVER_DIR="${SRC}/drivers/storage/class/disk"
    WRAPPER_INC=$'#include "disk.h"\n#include <wmistr.h>\n#include <ntddstor.h>\n#include <ntddscsi.h>\n#include <ntdddisk.h>\n#include <ntddvol.h>\n#include <ioevent.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]disk[/\\]disk\.h' --allowlist-file '.*classpnp\.h'
                  --allowlist-file '.*classp\.h' --allowlist-file '.*scsi\.h'
                  --allowlist-file '.*srb\.h' --allowlist-file '.*srbhelper\.h'
                  --allowlist-file '.*ntddstor\.h' --allowlist-file '.*ntddscsi\.h'
                  --allowlist-file '.*ntdddisk\.h' --allowlist-file '.*ntddvol\.h'
                  --allowlist-file '.*wmidata\.h'
                  --allowlist-file '.*wmistr\.h' --allowlist-file '.*storduid\.h'
                  --allowlist-file '.*mountdev\.h' --allowlist-file '.*ioevent\.h' )
    VER_DEFS=( -DWINVER=0x602 -D_WIN32_WINNT=0x602 -DNTDDI_VERSION=0x06020000 )
    ;;
  mountmgr)
    # mountmgr.sys — the mount manager (drivers/storage/mountmgr). Driver-PRIVATE
    # structs (DEVICE_EXTENSION, DEVICE_INFORMATION, SYMLINK_INFORMATION,
    # UNIQUE_ID_REPLACE_WORK_ITEM, ASSOCIATED_DEVICE_ENTRY, …) live in the driver's
    # own mntmgr.h — allowlist it. mntmgr.h pulls mountdev.h / ntddvol.h /
    # ntdddisk.h; the MOUNTMGR_* ioctl structs (MOUNTMGR_MOUNT_POINT,
    # MOUNTMGR_CREATE_POINT_INPUT, MOUNTMGR_DRIVE_LETTER_*, …) live in the separate
    # ddk/mountmgr.h, which the .c files reach only transitively — pull it
    # explicitly so its layouts get emitted (allowlist can only filter PARSED types).
    # Version macros mirror partmgr (the sibling storage-infra driver): WS03 0x502,
    # NTDDI forced to 0x05020400 for the amd64 peb_teb.h ACTIVATION_CONTEXT_STACK reason.
    DRIVER_DIR="${SRC}/drivers/storage/mountmgr"
    WRAPPER_INC=$'#include "mntmgr.h"\n#include <mountmgr.h>'
    ALLOW_FILES=( --allowlist-file '.*mountmgr[/\\]mntmgr\.h' --allowlist-file '.*mountmgr\.h'
                  --allowlist-file '.*mountdev\.h' --allowlist-file '.*ntddvol\.h'
                  --allowlist-file '.*ntdddisk\.h' --allowlist-file '.*ioevent\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  cdrom)
    # KMDF driver (set_module_type cdrom kmdfdriver) — needs the WDF 1.17 headers.
    DRIVER_DIR="${SRC}/drivers/storage/class/cdrom"
    # Pull the storage/SCSI/CD headers explicitly so bindgen PARSES (and, with the
    # allowlist below, EMITS) the SCSI/DVD/CD/AACS structs the modules use — CDB,
    # INQUIRYDATA, MODE_PARAMETER_*, DVD_*/CDVD_*/AACS_* (ntddcdvd.h), RAW_READ_INFO/
    # SUB_Q_* (ntddcdrm.h), STORAGE_*/MOUNTDEV_* — none of which are in the ntkrnl-sys
    # kernel bindgen. (classpnp profile does the same for its storage surface.)
    WRAPPER_INC=$'#include <ntddk.h>\n#include "cdrom.h"\n#include <scsi.h>\n#include <ntddstor.h>\n#include <ntddscsi.h>\n#include <ntddcdrm.h>\n#include <ntddcdvd.h>\n#include <ntddmmc.h>\n#include <ntdddisk.h>\n#include <mountdev.h>'
    ALLOW_FILES=( --allowlist-file '.*cdrom\.h' --allowlist-file '.*cdromp\.h' --allowlist-file '.*ioctl\.h' --allowlist-file '.*mmc\.h' --allowlist-file '.*scratch\.h'
                  --allowlist-file '.*scsi\.h' --allowlist-file '.*srb\.h' --allowlist-file '.*srbhelper\.h'
                  --allowlist-file '.*ntddstor\.h' --allowlist-file '.*ntddscsi\.h' --allowlist-file '.*ntddcdrm\.h'
                  --allowlist-file '.*ntddcdvd\.h' --allowlist-file '.*ntddmmc\.h' --allowlist-file '.*ntdddisk\.h'
                  --allowlist-file '.*mountdev\.h' --allowlist-file '.*storduid\.h' --allowlist-file '.*ioevent\.h' )
    # SCSI_REQUEST_BLOCK is opaque ({_address:u8}) in ntkrnl-sys but cdrom uses its
    # fields ~197x (.SrbStatus/.CdbLength/.Cdb/...). Re-allow it so cdrom's own bindgen
    # (which includes scsi.h) emits the full local layout.
    UNBLOCK="_SCSI_REQUEST_BLOCK SCSI_REQUEST_BLOCK PSCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x603 -D_WIN32_WINNT=0x603 -DNTDDI_VERSION=0x06030000
               -DWDF_DEVICE_NO_WDMSEC_H -DDEBUG_USE_KDPRINT -DKMDF_MAJOR_VERSION_STRING=01 -DKMDF_MINOR_VERSION_STRING=017 )
    EXTRA_INC=( -I"${SRC}/sdk/include/wdf/kmdf/1.17" -I"${SRC}/sdk/include/wdf/shared" -I"${SRC}/sdk/include/wdf" )
    # WDF types (WDFDEVICE/WDFREQUEST/WDF_REQUEST_PARAMETERS/...) get a SINGLE identity on
    # the kmdf-sys substrate — blocklist them here + re-export from kmdf_sys, so cdrom's
    # private structs (CDROM_DEVICE_EXTENSION.Device: WDFDEVICE) and kmdf-sys's dispatch
    # fns agree on the type. kmdf-sys is itself a gen_driver_types output → stem "types".
    SYS_CRATES=( "kmdf-sys:kmdf-sys/src:types" )
    ;;
  cdfs)
    DRIVER_DIR="${SRC}/drivers/filesystems/cdfs"
    WRAPPER_INC='#include "cdprocs.h"'
    # cd.h carries the ISO9660 on-disk raw structs (RAW_DIRENT/RAW_PATH_ENTRY/RAW_PATH_ISO/
    # RAW_READ_INFO/SYSTEM_USE_XA/DIR_CONTEXT/FILE_NAME) referenced all over the driver —
    # cdprocs.h #includes it, but without it allowlisted bindgen parsed-but-dropped them.
    # ntddcdrm.h carries the public CD-ROM DDK types (TRACK_MODE_TYPE/RAW_READ_INFO/
    # CDROM_READ_TOC_EX/CDROM_DISK_DATA/CDROM_TOC_*) used by the raw-read path; cdprocs.h
    # #includes it too but the kernel bindgen doesn't, so allowlist it here for cdfs.
    ALLOW_FILES=( --allowlist-file '.*cdstruc\.h' --allowlist-file '.*cdprocs\.h' --allowlist-file '.*cdfs\.h' --allowlist-file '.*[/\\]cd\.h' --allowlist-file '.*ntddcdrm\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 )
    ;;
  afd)
    # Ancillary Function Driver (drivers/network/afd) — the usermode Winsock→TDI
    # bridge, keystone of the netstack→winsock vertical. Driver-PRIVATE structs
    # (AFD_FCB, AFD_DEVICE_EXTENSION, AFD_TDI_OBJECT[_QELT], AFD_ACTIVE_POLL,
    # AFD_STORED_DATAGRAM, AFD_DATA_WINDOW, AFD_MAPBUF, AFD_IN_FLIGHT_REQUEST) live in
    # the driver's own include/afd.h. The usermode<->afd IOCTL-shared structs
    # (AFD_RECV/SEND/POLL_INFO, AFD_WSABUF, AFD_HANDLE, AFD_EVENT_*, AFD_CONNECT_INFO,
    # AFD_TDI_CONNECTION_INFORMATION) live in sdk/include/reactos/drivers/afd/shared.h
    # (#included as <afd/shared.h>); the TDI connection helpers in
    # tdihelpers/include/{tdiconn,tdi_proto}.h. Allowlist all of those local; tdi.h /
    # ntifs.h kernel+TDI types stay on the single ntkrnl-sys ddk identity (blocklisted).
    # afd.h also pulls winsock2.h/windef.h for SOCKADDR (shared.h's transport addresses).
    DRIVER_DIR="${SRC}/drivers/network/afd/include"
    EXTRA_INC=( -I"${SRC}/drivers/network/tdihelpers/include"
                -I"${SRC}/sdk/include/reactos/drivers" )
    WRAPPER_INC=$'#include "afd.h"'
    ALLOW_FILES=( --allowlist-file '.*afd[/\\]include[/\\]afd\.h' --allowlist-file '.*afd[/\\]shared\.h'
                  --allowlist-file '.*tdiconn\.h' --allowlist-file '.*tdi_proto\.h'
                  --allowlist-file '.*tcpioctl\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 )
    ;;
  partmgr)
    # The partition manager (drivers/storage/partmgr). Driver-PRIVATE structs
    # (FDO_EXTENSION, PARTITION_EXTENSION, BASIC_VOLUME_UNIQUE_ID,
    # DISK_GEOMETRY_EX_INTERNAL) live in the driver's own partmgr.h — allowlist it.
    # partmgr.h pulls mountdev.h / ntddvol.h / ntdddisk.h / ioevent.h; the
    # MOUNTDEV_*, VOLUME_*, DISK_GEOMETRY/DRIVE_LAYOUT_INFORMATION_EX/
    # PARTITION_INFORMATION_EX and DEVICE_EVENT_* types those define are NOT all in
    # the shared ntkrnl-sys bindgen, so allowlist them too (blocklist keeps any that
    # ARE shared on the single ddk identity). Version macros match the real partmgr
    # compile command (WINVER/_WIN32_WINNT 0x502 + _WIN32_IE 0x603); NTDDI is forced
    # to 0x05020400 (WS03SP1) for the same amd64 peb_teb.h ACTIVATION_CONTEXT_STACK
    # reason as pcix.
    DRIVER_DIR="${SRC}/drivers/storage/partmgr"
    WRAPPER_INC=$'#include "partmgr.h"'
    ALLOW_FILES=( --allowlist-file '.*partmgr[/\\]partmgr\.h' --allowlist-file '.*mountdev\.h'
                  --allowlist-file '.*ntddvol\.h' --allowlist-file '.*ntdddisk\.h'
                  --allowlist-file '.*ioevent\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  fs_rec)
    # The filesystem recognizer (drivers/filesystems/fs_rec). Driver-PRIVATE structs
    # (PACKED_BOOT_SECTOR, PACKED_BIOS_PARAMETER_BLOCK, BIOS_PARAMETER_BLOCK, TAG,
    # EXTENT, AVDP, FILE_SYSTEM_TYPE, FS_REC_STATE, DEVICE_EXTENSION) live in the
    # driver's own fs_rec.h. Each per-FS sniffer references on-disk superblock structs
    # from its own header (btrfs.h/cdfs.h/ext.h/ffs.h/reiserfs.h/udfs.h) — all LOCAL to
    # the fs_rec dir, so allowlist them path-scoped (the local cdfs.h is the 24-line
    # boot-volume-descriptor one, NOT the cdfs FS driver's cdprocs/cdstruc). Only
    # <ntifs.h> is pulled for kernel types (blocklisted → single ntkrnl-sys identity).
    # fs_rec CMakeLists sets no version macros → NT 5.2 default; force NTDDI 0x05020400
    # for the same amd64 peb_teb.h ACTIVATION_CONTEXT_STACK reason as pcix/partmgr.
    DRIVER_DIR="${SRC}/drivers/filesystems/fs_rec"
    WRAPPER_INC=$'#include "fs_rec.h"\n#include "btrfs.h"\n#include "cdfs.h"\n#include "ext.h"\n#include "ffs.h"\n#include "reiserfs.h"\n#include "udfs.h"'
    ALLOW_FILES=( --allowlist-file '.*fs_rec[/\\]fs_rec\.h' --allowlist-file '.*fs_rec[/\\]btrfs\.h'
                  --allowlist-file '.*fs_rec[/\\]cdfs\.h' --allowlist-file '.*fs_rec[/\\]ext\.h'
                  --allowlist-file '.*fs_rec[/\\]ffs\.h' --allowlist-file '.*fs_rec[/\\]reiserfs\.h'
                  --allowlist-file '.*fs_rec[/\\]udfs\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  pciidex)
    # The PCI IDE bus extension (drivers/storage/ide/pciidex). Driver-PRIVATE structs
    # (PCIIDEX_DRIVER_EXTENSION, COMMON_DEVICE_EXTENSION, FDO_DEVICE_EXTENSION,
    # PDO_DEVICE_EXTENSION) live in the driver's own pciidex.h — allowlist it. pciidex.h
    # pulls <ide.h> (sdk/include/ddk/ide.h), the shared IDE-miniport header defining
    # IDE_CONTROLLER_PROPERTIES / IDE_CHANNEL_STATE / PCIIDE_* / the IdeBusData enums the
    # modules reference; those are NOT in the ntkrnl-sys kernel bindgen, so allowlist
    # ide.h too (blocklist keeps any shared ddk types on the single identity). No version
    # macros in the pciidex CMakeLists → NT 5.2 default; NTDDI 0x05020400 (peb_teb trap).
    DRIVER_DIR="${SRC}/drivers/storage/ide/pciidex"
    WRAPPER_INC=$'#include "pciidex.h"'
    ALLOW_FILES=( --allowlist-file '.*pciidex[/\\]pciidex\.h' --allowlist-file '.*ddk[/\\]ide\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  scsiport)
    # The SCSI port driver (drivers/storage/port/scsiport). Driver-PRIVATE structs
    # (SCSI_PORT_DEVICE_EXTENSION, SCSI_PORT_LUN_EXTENSION, SCSI_PORT_COMMON_EXTENSION,
    # SCSI_REQUEST_BLOCK_INFO, CONFIGURATION_INFO, SCSI_BUS_INFO, SCSI_PORT_INTERRUPT_DATA,
    # STORAGE_ADAPTER_DESCRIPTOR_WIN8, ...) live in the driver's own scsiport.h — allowlist
    # it. Those LUN/SRB-info structs embed SCSI DDK types (SCSI_REQUEST_BLOCK, CDB,
    # SENSE_DATA, SCSI_PORT_CONFIGURATION_INFORMATION, ACCESS_RANGE) from <scsi.h> +
    # <ntddscsi.h>, none of which are in the ntkrnl-sys kernel bindgen → allowlist those
    # two headers too so bindgen emits exact SRB/CDB layouts (the blocklist keeps any
    # shared ntkrnl-sys ddk types on the single identity). No version macros in the
    # scsiport CMakeLists → NT 5.2 default; NTDDI 0x05020400 (peb_teb trap).
    DRIVER_DIR="${SRC}/drivers/storage/port/scsiport"
    WRAPPER_INC=$'#include "scsiport.h"'
    ALLOW_FILES=( --allowlist-file '.*scsiport[/\\]scsiport\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]srb\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h' )
    # _SCSI_REQUEST_BLOCK is OPAQUE in the kernel bindgen ({ _address: u8 }) — unblock it
    # so this driver's bindgen (which includes scsi.h) emits the FULL SRB (SrbStatus/Cdb/
    # DataBuffer/SrbFlags/...). Otherwise every Srb.<field> access resolves to the stub.
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  pcix)
    # The full MS-architecture PCI bus driver (drivers/bus/pcix). All driver-PRIVATE
    # structs (PCI_FDO_EXTENSION, PCI_PDO_EXTENSION, PCI_FUNCTION_RESOURCES,
    # PCI_CONFIGURATOR*, PCI_INTERFACE, PCI_ARBITER_INSTANCE, ...) live in the driver's
    # own pci.h — allowlist that ONE header (the shared drivers/pci/pci.h is at a
    # different path so it is NOT matched, keeping PCI_COMMON_CONFIG/PCI_SLOT_NUMBER on
    # the single ntkrnl-sys ddk identity). pci.h also pulls arbiter.h (ARBITER_INSTANCE,
    # ARBITER_*) and cmreslist.h — those are NOT in the kernel bindgen, so allowlist them
    # too so the bus-resource arbiter structs the modules reference get full local layouts.
    DRIVER_DIR="${SRC}/drivers/bus/pcix"
    WRAPPER_INC=$'#include "pci.h"'
    ALLOW_FILES=( --allowlist-file '.*bus[/\\]pcix[/\\]pci\.h' --allowlist-file '.*arbiter\.h'
                  --allowlist-file '.*cmreslist\.h' )
    # pcix CMakeLists sets no version macros → driver builds at the toolchain default
    # (NT 5.2 / WinXP-era). NTDDI MUST be 0x05020400 (WS03SP1), the value the real build
    # derives — at exactly 0x05020000 peb_teb.h takes the pre-SP1 path that inlines an
    # UNDEFINED ACTIVATION_CONTEXT_STACK (→ negative-size array, fatal TU-abort on amd64).
    # The extra _WIN32_WINDOWS/_WIN32_IE/_SETUPAPI_VER/DLL_EXPORT_VERSION match the real
    # pcix compile command so the header gates resolve identically.
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    EXTRA_INC=( -I"${SRC}/sdk/lib/drivers/arbiter" )
    ;;
  kmdf)
    # The WDF *client-binding* API surface (NOT a driver) → the kmdf-sys substrate.
    # cdrom is the fleet's only KMDF driver; its 109 Wdf* calls dispatch through the
    # WdfFunctions[] table (see kmdf-sys/src/lib.rs). Allowlist EVERYTHING under a
    # .../wdf/... path so bindgen emits the full WDF 1.17 surface: the WDF handle
    # types (WDFDEVICE/WDFREQUEST/WDFKEY/...), the value structs (WDF_OBJECT_ATTRIBUTES,
    # WDF_IO_QUEUE_CONFIG, WDF_REQUEST_PARAMETERS, ...), the PFN_WDFXXX dispatch-pointer
    # typedefs, WDFFUNC, WDF_DRIVER_GLOBALS/WDF_BIND_INFO, and the 444-entry
    # _WDFFUNCENUM (the WdfXxxTableIndex constants). Kernel types referenced by WDF
    # structs (PDRIVER_OBJECT, NTSTATUS, UNICODE_STRING, ...) stay blocklisted → the
    # single ntkrnl-sys ddk identity. Output → kmdf-sys/src (OUT_CRATE_DIR override);
    # gen_kmdf_dispatch.py then turns the PFN typedefs + indices into src/dispatch.rs.
    # KMDF 1.17 / NTDDI Win8.1 — matches cdrom's CMakeLists.
    DRIVER_DIR="${SRC}/sdk/include/wdf/kmdf/1.17"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <wdf.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]wdf[/\\].*' )
    VER_DEFS=( -DWINVER=0x603 -D_WIN32_WINNT=0x603 -DNTDDI_VERSION=0x06030000
               -DWDF_DEVICE_NO_WDMSEC_H -DKMDF_MAJOR_VERSION_STRING=01 -DKMDF_MINOR_VERSION_STRING=017 )
    EXTRA_INC=( -I"${SRC}/sdk/include/wdf/kmdf/1.17" -I"${SRC}/sdk/include/wdf/shared" -I"${SRC}/sdk/include/wdf" )
    ;;
  fastfat)
    # fastfat.sys — the FAT filesystem (drivers/filesystems/fastfat, MS-architecture,
    # 35 .c / 68.8K LOC). Driver-PRIVATE structs (VCB, FCB, DCB, CCB, IRP_CONTEXT,
    # FAT_DATA, DIRENT/PACKED_BOOT_SECTOR on-disk layouts, BCB, FAT_ENUMERATION_CONTEXT,
    # node-type codes, ~30 pool tags) live in the driver's own fatstruc.h / fat.h /
    # nodetype.h / lfn.h / fatdata.h. fatprocs.h is the PCH that #includes ntifs.h + all
    # of them in order — pull it so every private struct gets PARSED, then allowlist ONLY
    # the fastfat-private headers so the embedded kernel/FsRtl types (FSRTL_ADVANCED_FCB_
    # HEADER, SECTION_OBJECT_POINTERS, ERESOURCE, FILE_LOCK, ...) stay blocklisted on the
    # single ntkrnl-sys ddk identity. Version macros from the real fastfat compile command
    # (all 0x502); NTDDI forced to 0x05020400 for the amd64 peb_teb.h ACTIVATION_CONTEXT_
    # STACK reason as the 0x502 siblings (partmgr/fs_rec/pcix/pciidex).
    DRIVER_DIR="${SRC}/drivers/filesystems/fastfat"
    WRAPPER_INC=$'#include "fatprocs.h"'
    ALLOW_FILES=( --allowlist-file '.*fastfat[/\\]fat\.h' --allowlist-file '.*fastfat[/\\]fatstruc\.h'
                  --allowlist-file '.*fastfat[/\\]nodetype\.h' --allowlist-file '.*fastfat[/\\]lfn\.h'
                  --allowlist-file '.*fastfat[/\\]fatdata\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  atapi)
    # The NT4-DDK ATAPI IDE miniport (drivers/storage/ide/atapi/atapi.c). It is a
    # SCSIPORT miniport: its private register/IDENTIFY structs (IDE_REGISTERS_1/2/3,
    # ATAPI_REGISTERS_1/2, IDENTIFY_DATA/IDENTIFY_DATA2, MODE_SENSE_10/MODE_SELECT_10/
    # MODE_PARAMETER_HEADER_10, BROKEN_/NATIVE_MODE_CONTROLLER_INFORMATION) live in the
    # driver's own atapi.h — allowlist it. The HW_DEVICE_EXTENSION / HW_LU_EXTENSION are
    # defined in atapi.c (not a header), so a bindgen-only atapi_devext.h in the crate dir
    # reproduces them (it embeds IDENTIFY_DATA + SCSI_REQUEST_BLOCK/SENSE_DATA/MECHANICAL_
    # STATUS_INFORMATION_HEADER by value → those scsi.h/srb.h layouts must be emitted too).
    # Like scsiport, _SCSI_REQUEST_BLOCK is OPAQUE in the kernel bindgen → UNBLOCK it so
    # THIS driver's bindgen (which includes scsi.h) emits the full SRB. No version macros
    # in the atapi CMakeLists → NT 5.2 default; NTDDI 0x05020400 (amd64 peb_teb trap).
    DRIVER_DIR="${SRC}/drivers/storage/ide/atapi"
    WRAPPER_INC=$'#include <ntddk.h>\n#include "atapi.h"\n#include <ntddscsi.h>\n#include <ntdddisk.h>\n#include <ntddstor.h>\n#include "atapi_devext.h"'
    ALLOW_FILES=( --allowlist-file '.*ide[/\\]atapi[/\\]atapi\.h'
                  --allowlist-file '.*atapi_devext\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]srb\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h' )
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    EXTRA_INC=( -I"${SRC}/sdk/lib/rust/atapi-rust" )
    ;;
  usbaudio)
    # USB Audio class driver (drivers/usb/usbaudio, 4 .c). A KS/PortCls audio miniport:
    # usbaudio.h pulls portcls.h (PORT/KSPIN/SUBDEVICE interfaces), ksmedia.h (KS audio
    # formats/property sets), usb.h/usbdi.h/usbdlib.h/usbioctl.h + hubbusif/usbbusif (the USB
    # descriptor + bus interfaces), and defines the driver-private structs inline (the
    # pshpack1 USB-audio descriptor structs + __DEVICE_EXTENSION__). None of the KS/PortCls/USB
    # surface is in the kernel ddk bindgen -> allowlist usbaudio.h + those headers so the crate
    # types.rs carries the full KS/PortCls/USB types the bodies use. _URB is opaque in the
    # kernel ddk bindgen -> unblock it so this driver's bindgen (includes usb.h) emits the full
    # URB. NTDDI Vista (0x600) per the usbaudio CMakeLists (-D_WIN32_WINNT=0x600).
    DRIVER_DIR="${SRC}/drivers/usb/usbaudio"
    WRAPPER_INC=$'#include "usbaudio.h"'
    ALLOW_FILES=( --allowlist-file '.*usbaudio[/\\]usbaudio\.h'
                  --allowlist-file '.*[/\\]portcls\.h'
                  --allowlist-file '.*[/\\]ksmedia\.h'
                  --allowlist-file '.*[/\\]ks\.h'
                  --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usbdi\.h'
                  --allowlist-file '.*[/\\]usbdlib\.h'
                  --allowlist-file '.*[/\\]usbioctl\.h' )
    UNBLOCK="_URB URB PURB"
    VER_DEFS=( -DWINVER=0x600 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 )
    EXTRA_INC=( -I"${SRC}/sdk/lib/rust/usbaudio-rust" )
    ;;
  ks)
    # Kernel Streaming class library (drivers/ksfilter/ks, 22 .c / ~19K LOC). The KS keystone:
    # EXPORT driver (216 Ks* exports). Private structs live in the driver's own kstypes.h; the
    # COM-ish interface vtables in ksiface.h. precomp.h pulls ntddk + portcls.h + kcom.h + ntstrsafe.
    # -DBUILDING_KS/_COMDDK_ expose the internal Ks* decls. Allowlist kstypes.h + ksiface.h +
    # portcls.h + kcom.h + ks.h so the crate types.rs carries the full private + KS/PortCls surface;
    # kernel types stay blocklisted (single ddk identity). NT 5.2 default; NTDDI 0x05020400.
    DRIVER_DIR="${SRC}/drivers/ksfilter/ks"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <portcls.h>\n#include <kcom.h>\n#include <ntstrsafe.h>\n#include "kstypes.h"\n#include "ksiface.h"'
    ALLOW_FILES=( --allowlist-file '.*ksfilter[/\\]ks[/\\]kstypes\.h'
                  --allowlist-file '.*ksfilter[/\\]ks[/\\]ksiface\.h'
                  --allowlist-file '.*[/\\]portcls\.h'
                  --allowlist-file '.*[/\\]kcom\.h'
                  --allowlist-file '.*[/\\]ks\.h' )
    VER_DEFS=( -DBUILDING_KS -D_COMDDK_
               -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 )
    EXTRA_INC=( -I"${SRC}/sdk/lib/rust/ks-rust" )
    ;;
  buslogic)
    # The BusLogic BT-958 SCSI HBA miniport (drivers/storage/port/buslogic/BusLogic958.c,
    # single 2.8K-LOC TU). Like atapi it is a SCSIPORT miniport: it imports ScsiPort* and
    # registers via ScsiPortInitialize. NOT in any QEMU boot path → link-clean ×3 is the
    # bar. ALL its private structs (BusLogic_HostAdapter_T, BusLogic_CCB_T, BusLogic_*Mailbox,
    # NONCACHED_EXTENSION, and the _HW_DEVICE_EXTENSION device-extension itself) live in the
    # driver's own BusLogic958.h — allowlist it. BusLogic958.h #includes srb.h/scsi.h/
    # ntddscsi.h/scsiwmi.h, so the SRB/CDB/sense + SCSI_WMI layouts must be emitted too →
    # allowlist those headers. Like scsiport/atapi, _SCSI_REQUEST_BLOCK is OPAQUE in the
    # kernel ddk bindgen → UNBLOCK it so THIS driver's bindgen (which includes scsi.h/srb.h)
    # emits the full SRB. No version macros in the buslogic CMakeLists → NT 5.2 default;
    # NTDDI 0x05020400 (amd64 peb_teb trap).
    DRIVER_DIR="${SRC}/drivers/storage/port/buslogic"
    WRAPPER_INC=$'#include <ntddk.h>\n#include "BusLogic958.h"'
    ALLOW_FILES=( --allowlist-file '.*buslogic[/\\]BusLogic958\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]srb\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h'
                  --allowlist-file '.*[/\\]scsiwmi\.h' )
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    EXTRA_INC=( -I"${SRC}/sdk/lib/rust/buslogic-rust" )
    ;;
  uniata)
    # The UniATA IDE/SATA/AHCI driver (drivers/storage/ide/uniata, 10 .cpp / 24K LOC,
    # C-in-.cpp). All .cpp #include "stdafx.h", which pulls the driver's OWN private
    # headers — the core structs (HW_DEVICE_EXTENSION/HW_CHANNEL/HW_LU_EXTENSION/ATA_REQ/
    # IORES) live in bsmaster.h, plus atapi.h/scsi.h/srb.h/ntddscsi.h/id_sata.h/bm_devs*.h/
    # uata_ctl.h/badblock.h/tools.h/config.h (all in the uniata dir, which uniata's OWN
    # scsi.h/srb.h/ntddscsi.h SHADOW the sdk ones — the -I DRIVER_DIR comes first). The
    # CrossNt.h dynamic-ntoskrnl compat layer is NOT pulled (bsmaster.h doesn't use it).
    # Allowlist the whole uniata header dir → emit every private struct; blocklist keeps
    # shared ntkrnl-sys kernel types on one identity. The live config.h feature flags
    # (USE_OWN_DMA/QUEUE_STATISTICS/IO_STATISTICS/USE_REACTOS_DDK) are passed so the
    # header conditionals resolve to the real build. NTDDI 0x05020400 (peb_teb trap).
    DRIVER_DIR="${SRC}/drivers/storage/ide/uniata"
    WRAPPER_INC=$'#include <ntddk.h>\n#include "atapi.h"\n#include "ntdddisk.h"\n#include "ntddscsi.h"\n#include "bsmaster.h"\n#include "id_sata.h"\n#include "badblock.h"'
    ALLOW_FILES=( --allowlist-file '.*ide[/\\]uniata[/\\].*\.h' )
    LANG_ARGS=( -x c++ -std=gnu++17 )  # headers use C++ (default args, bool, anon unions)
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DUSE_OWN_DMA -DQUEUE_STATISTICS
               -DIO_STATISTICS -DUSE_REACTOS_DDK -D_NEW_DELETE_OPERATORS_
               -DUSE_COMPILER_EXCEPTIONS )
    EXTRA_INC=( -I"${SRC}/drivers/storage/ide/uniata/inc" )
    ;;
  storahci)
    # AHCI storport miniport (drivers/storage/port/storahci). Private structs (AHCI_ADAPTER_
    # EXTENSION, AHCI_PORT, AHCI_CAP/GHC/PORT_CMD bitfield-union registers, AHCI_COMMAND_HEADER,
    # AHCI_RECEIVED_FIS, AHCI_SRB_EXTENSION, AHCI_QUEUE, ...) live in storahci.h. It also pulls
    # the storport.h miniport interface (HW_INITIALIZATION_DATA, PORT_CONFIGURATION_INFORMATION,
    # STOR_DPC, STOR_PHYSICAL_ADDRESS) + ata.h (IDENTIFY_DEVICE_DATA), none in the kernel bindgen
    # → allowlist all three so bindgen emits exact layouts. No version macros (only -DDEBUG) →
    # NT 5.2 default.
    DRIVER_DIR="${SRC}/drivers/storage/port/storahci"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <ata.h>\n#include <storport.h>\n#include "storahci.h"'
    ALLOW_FILES=( --allowlist-file '.*storahci[/\\]storahci\.h'
                  --allowlist-file '.*[/\\]storport\.h'
                  --allowlist-file '.*[/\\]ata\.h' )
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DDEBUG )
    ;;
  storport)
    # Storage port driver (drivers/storage/port/storport). Private structs (FDO_DEVICE_EXTENSION,
    # PDO_DEVICE_EXTENSION, MINIPORT_DEVICE_EXTENSION, DRIVER_OBJECT_EXTENSION, DRIVER_INIT_DATA,
    # MINIPORT, UNIT_DATA, EXTENSION_TYPE) live in precomp.h; the public storport.h miniport
    # interface (HW_INITIALIZATION_DATA, SCSI_REQUEST_BLOCK, STOR_*) is not in the kernel bindgen
    # → allowlist both. NT 5.2 default (no version macros).
    DRIVER_DIR="${SRC}/drivers/storage/port/storport"
    WRAPPER_INC=$'#include "precomp.h"'
    ALLOW_FILES=( --allowlist-file '.*storport[/\\]precomp\.h'
                  --allowlist-file '.*[/\\]storport\.h' )
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  serial)
    # 8250/16550 UART COM driver (drivers/serial/serial). Private SERIAL_DEVICE_EXTENSION +
    # SERIAL_* config structs live in serial.h; ntddser.h carries the SERIAL_* IOCTL structs
    # (SERIAL_BAUD_RATE, SERIAL_LINE_CONTROL, SERIAL_TIMEOUTS, SERIAL_STATUS, ...) not in the
    # kernel bindgen → allowlist both. NT 5.2 default.
    DRIVER_DIR="${SRC}/drivers/serial/serial"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <ntddser.h>\n#include "serial.h"'
    ALLOW_FILES=( --allowlist-file '.*serial[/\\]serial\.h'
                  --allowlist-file '.*[/\\]ntddser\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  ramdisk)
    # RAM-disk class driver (drivers/storage/class/ramdisk). Its private device-extension
    # structs (RAMDISK_EXTENSION, RAMDISK_BUS_EXTENSION, RAMDISK_DRIVE_EXTENSION) are defined
    # INLINE in ramdisk.c → translated directly from the .c, not bindgen'd. The only header
    # types not in the kernel bindgen are the public IOCTL structs RAMDISK_CREATE_INPUT /
    # RAMDISK_CREATE_OPTIONS in reactos/drivers/ntddrdsk.h → allowlist that one header. NT 5.2.
    DRIVER_DIR="${SRC}/drivers/storage/class/ramdisk"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <ntdddisk.h>\n#include <reactos/drivers/ntddrdsk.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ntddrdsk\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  fltmgr)
    # Filesystem Filter Manager (drivers/filters/fltmgr). Private structs (FLT_OBJECT,
    # FLT_FILTER, FLT_INSTANCE, FLTP_FRAME, STREAM_LIST_CTRL, FLT_PORT_OBJECT, FLT_SERVER_
    # PORT_OBJECT, FLT_MESSAGE_WAITER_QUEUE, ...) live in fltmgrint.h (a bare struct header,
    # no includes -> needs fltmgr.h context first). The public minifilter API (FLT_REGISTRATION,
    # FLT_OPERATION_REGISTRATION, FLT_RELATED_OBJECTS, FLT_CALLBACK_DATA, PFLT_FILTER/INSTANCE/
    # VOLUME) is in ddk/fltkernel.h; the shared fltmgr<->minifilter structs in
    # reactos/drivers/fltmgr/fltmgr_shared.h. Allowlist all three. NTDDI WS03SP1 (0x05020100).
    DRIVER_DIR="${SRC}/drivers/filters/fltmgr"
    WRAPPER_INC=$'#include "fltmgr.h"\n#include "fltmgrint.h"'
    ALLOW_FILES=( --allowlist-file '.*fltmgr[/\\]fltmgrint\.h'
                  --allowlist-file '.*[/\\]fltkernel\.h'
                  --allowlist-file '.*[/\\]fltmgr_shared\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020100
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  ksecdd)
    # Kernel Security Support Provider (drivers/crypto/ksecdd). Private structs
    # (KSEC_CONNECTION_INFO, KSEC_ENTROPY_DATA, KSEC_MACHINE_SPECIFIC_COUNTERS) in ksecdd.h;
    # IOCTL structs in reactos/drivers/ksecdd/ksecioctl.h. ksecdd.h also pulls tomcrypt/md4/md5
    # (cryptlib) -> allowlist ONLY ksecdd.h + ksecioctl.h so bindgen emits just the KSEC types,
    # not the whole crypto surface. NT 5.2.
    DRIVER_DIR="${SRC}/drivers/crypto/ksecdd"
    EXTRA_INC=( -I"${SRC}/sdk/include/reactos/drivers/ksecdd" -I"${SRC}/sdk/lib/cryptlib" )
    WRAPPER_INC=$'#include <ntifs.h>\n#include <ksecioctl.h>\n#include "ksecdd.h"'
    ALLOW_FILES=( --allowlist-file '.*ksecdd[/\\]ksecdd\.h'
                  --allowlist-file '.*[/\\]ksecioctl\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  msafd)
    # USERMODE Winsock service provider (dll/win32/msafd) — the rung ABOVE afd.sys in the
    # netstack→winsock vertical: ws2_32's WSP* calls land here and become IOCTL_AFD_*
    # DeviceIoControls down to afd.sys. NOT a kernel driver → USERMODE=1: blocklist against
    # ntdll-sys (the NDK/Native-API bindgen) and re-export `pub use ntdll_sys::nt::*;` so the
    # NT base types (HANDLE/ULONG/UNICODE_STRING/LARGE_INTEGER/NTSTATUS/IO_STATUS_BLOCK/...)
    # keep a SINGLE identity shared with the translation (the misc/*.rs `use ntdll_sys::nt::*`).
    # Only the winsock/WSP/wsahelp/AFD-shared/TDI/msafd-private structs become local.
    #
    # Wrapper = the real msafd.h (it sets NTOS_MODE_USER + pulls windef/winbase/ws2spi, the
    # NDK exfuncs/iofuncs/obfuncs/rtlfuncs, wsahelp/tdi/afd-shared/mswsock, wine/debug, and
    # the driver-private include/helpers.h — i.e. the whole msafd type surface). The local
    # structs (SOCKET_INFORMATION/SOCKET_CONTEXT/ASYNC_DATA/MSAFD_*_APC_CONTEXT) live in
    # msafd.h itself; HELPER_DATA/WSH_* in include/helpers.h + wsahelp.h; SOCK_SHARED_INFO +
    # AFD_*_INFO IOCTL structs in afd/shared.h; WSPUPCALLTABLE/WSAPROTOCOL_INFOW/WSABUF/SOCKADDR
    # in ws2spi.h/winsock2.h/ws2def.h; TRANSPORT_ADDRESS/TDI_* in tdi.h. Allowlist all of those;
    # windef/winbase usermode structs (OVERLAPPED/SYSTEMTIME/FILETIME/SECURITY_ATTRIBUTES) are
    # pulled automatically as transitive deps (anything ALSO in ntdll-sys stays blocklisted →
    # single nt identity). BLOCKLIST_FUNCS=1: msafd.h declares the WSP* prototypes the
    # translation #[no_mangle]-defines, so drop all fn decls from the type substrate.
    # Version macros mirror the global usermode build (CMakeLists.txt: WINVER/_WIN32_WINNT/
    # _WIN32_WINDOWS 0x502, _WIN32_IE 0x603, _SETUPAPI_VER 0x502); NTDDI forced to 0x05020400
    # (WS03SP1) for the amd64 peb_teb.h ACTIVATION_CONTEXT_STACK reason (same as the 0x502
    # driver siblings partmgr/fs_rec/pcix).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    BLOCKLIST_VARS=1   # msafd defines its own module globals (SockEvent/GlobalHeap/Upcalls/...)
    CALLBACK_SYSTEM=1  # LPWSP*/upcall fn-ptr types -> extern "system" (match the "system" WSP defs)
    # MACRO_SIGNED left OFF: tried signed winsock consts to kill the Status=WSAExxx (i32=u32)
    # E0308 flood, but the translation mixes int/DWORD/handle loosely, so signed just traded
    # those for the reverse (u32-param = i32-const) set and NET-increased E0308 325->458. The
    # real fix is per-site casts in the modules, not a substrate const-type default.
    DRIVER_DIR="${SRC}/dll/win32/msafd"
    WRAPPER_INC=$'#include "msafd.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]msafd\.h'
                  --allowlist-file '.*msafd[/\\]include[/\\]helpers\.h'
                  --allowlist-file '.*[/\\]ws2spi\.h' --allowlist-file '.*[/\\]winsock2\.h'
                  --allowlist-file '.*[/\\]winsock\.h' --allowlist-file '.*[/\\]ws2def\.h'
                  --allowlist-file '.*[/\\]wsahelp\.h' --allowlist-file '.*afd[/\\]shared\.h'
                  --allowlist-file '.*[/\\]mswsock\.h' --allowlist-file '.*[/\\]tdi\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  ws2_32)
    # USERMODE Winsock 2.2 library (dll/win32/ws2_32) — the rung ABOVE msafd in the
    # netstack→winsock vertical, and the TOP of it: the public Win32 sockets API that
    # applications call. ws2_32 dispatches through the transport-provider catalog into
    # msafd's WSP* entry points (which become IOCTL_AFD_* down to afd.sys). Same USERMODE=1
    # treatment as msafd: blocklist against ntdll-sys and re-export `pub use ntdll_sys::nt::*;`
    # so NT base types keep ONE identity across the whole vertical.
    #
    # Wrapper = inc/ws2_32.h, which IS the module's PCH (CMakeLists: `add_pch(ws2_32
    # inc/ws2_32.h SOURCE)`). That matters: the .c files include NOTHING themselves — they
    # rely entirely on the PCH having already pulled windef/winbase/winreg/winnls/winuser,
    # ws2spi, the NDK rtlfuncs, pseh2, ws2help, nsp_dns, iptypes, and finally the private
    # inc/ws2_32p.h. ws2_32p.h has no #includes of its own and does not compile standalone,
    # so the PCH is the ONLY correct wrapper.
    #
    # The whole point of allowlisting ws2_32p.h is the object model: TCATALOG/TCATALOG_ENTRY/
    # TPROVIDER (transport-provider catalog), NSCATALOG/NSPROVIDER/NSQUERY (namespace),
    # WSPROCESS/WSTHREAD/WSSOCKET (the refcounted per-process/thread/socket objects), and
    # WSASYNCBLOCK (the async window-message machinery). These are reference-counted C-with-
    # objects with hot field access on every socket call — hand-transcribing them is exactly
    # the offset-drift class `scan_magic_offsets` exists to catch, so GENERATE them.
    #
    # BLOCKLIST_FUNCS=1: ws2_32p.h declares the internal Ws*/Ns* prototypes (WsProcOpen,
    # WsSockOpen, WsTcOpen, ...) that the translated modules themselves define — emitting them
    # as externs here would collide under the flat `use crate::types::*` glob.
    # BLOCKLIST_VARS=1: ws2_32 keeps module globals (WsProcess/WsAsyncQueue/startup lock).
    # CALLBACK_SYSTEM=1: the LPWSP*/LPFN_*/upcall fn-ptr types must be extern "system" —
    # on i386 these are stdcall, and a cdecl fn-ptr in a provider dispatch slot is a silent
    # stack imbalance on every socket call (the ole32 CALLBACK class, one layer down).
    #
    # -DLE mirrors the module's own `add_definitions(-DLE)`; the extra -I matches its
    # `include_directories(inc ${REACTOS_SOURCE_DIR}/sdk/include/reactos/winsock)`.
    # Version macros track the global usermode build (DLL_EXPORT_VERSION default 0x502 →
    # ws2help_static is NOT linked, so no ws2help static surface to model).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    BLOCKLIST_VARS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/ws2_32/inc"
    # ws2_32.h is the PCH, but addrinfo.c/rnr.c ALSO include <ws2tcpip.h> on top of it (that
    # is where SOCKADDR_IN6/IN6_ADDR/addrinfo live — via ws2ipdef.h). Allowlisting a header
    # the wrapper never includes does nothing, so include it here as the .c files do.
    WRAPPER_INC=$'#include "ws2_32.h"\n#include <ws2tcpip.h>'
    # ws2ipdef.h/ws2tcpip.h carry the IPv6 surface (sockaddr_in6 with its scope-id union,
    # in6_addr, ipv6_mreq) that addrinfo.c/rnr.c use. Leaving them out does not produce an
    # error — it produces a MISSING type, which the model then hand-invents, and a
    # hand-invented sockaddr_in6 is a silent wire-format bug.
    ALLOW_FILES=( --allowlist-file '.*ws2_32[/\\]inc[/\\]ws2_32p?\.h'
                  --allowlist-file '.*[/\\]ws2spi\.h' --allowlist-file '.*[/\\]winsock2\.h'
                  --allowlist-file '.*[/\\]winsock\.h' --allowlist-file '.*[/\\]ws2def\.h'
                  --allowlist-file '.*[/\\]ws2ipdef\.h' --allowlist-file '.*[/\\]ws2tcpip\.h'
                  --allowlist-file '.*[/\\]ws2help\.h' --allowlist-file '.*[/\\]nsp_dns\.h'
                  --allowlist-file '.*[/\\]nspapi\.h' --allowlist-file '.*[/\\]svcguid\.h'
                  --allowlist-file '.*[/\\]iptypes\.h' --allowlist-file '.*[/\\]wsahelp\.h'
                  --allowlist-file '.*[/\\]mswsock\.h' )
    EXTRA_INC=( -I"${SRC}/sdk/include/reactos/winsock" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DLE )
    ;;
  wshtcpip)
    # USERMODE Winsock TCP/IP helper DLL (dll/win32/wshtcpip) — the transport-specific
    # WSH provider ws2_32/msafd load to resolve the TCP/IP address family: it maps winsock
    # protocol/socket params to TDI and answers WSHOpenSocket/WSHGetSocketInformation/
    # WSHIoctl. Same USERMODE=1 treatment as msafd/ws2_32: blocklist against ntdll-sys and
    # re-export `pub use ntdll_sys::nt::*;` so the NT base types keep ONE identity with the
    # translated modules (`use ntdll_sys::nt::*`). Only the WSH/TDI/winsock structs go local.
    #
    # Wrapper = wshtcpip.h (it pulls windef/winbase/wsahelp/tdiinfo/tcpioctl/tdilib/
    # ws2tcpip/rtlfuncs — the whole type surface). iflist.c additionally pulls iptypes.h
    # (IP_ADAPTER_INFO for the interface enumeration), so allowlist it too. The private
    # structs the translation touches: WSH_HANDLE/WSH_*_INFO + WINSOCK_MAPPING (wsahelp.h),
    # TDIEntityID/TDIObjectID/TDI_ADDRESS_* (tdiinfo.h), TCP_REQUEST_QUERY_INFORMATION_EX
    # (tcpioctl.h), SOCKADDR/SOCKADDR_IN/SOCKADDR_IN6/WSAPROTOCOL_INFOW (ws2tcpip →
    # winsock2/ws2def/ws2ipdef), IP_ADAPTER_INFO/IP_ADDR_STRING (iptypes.h). A hand-invented
    # sockaddr_in6 or TDIObjectID is a silent wire-format/IOCTL bug — the scan_magic_offsets
    # class — so GENERATE them.
    #
    # BLOCKLIST_FUNCS=1: wsahelp.h/tdilib.h declare WSH*/tdi-helper prototypes the translated
    # modules define/call as their own externs — emitting them here collides under the flat
    # `use crate::types::*` glob. CALLBACK_SYSTEM=1: WSH provider entry points are stdcall on
    # i386, and a cdecl fn-ptr in a provider slot is a silent stack imbalance (the ws2_32/
    # ole32 CALLBACK class). Version macros track the global usermode build (0x502).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/wshtcpip"
    WRAPPER_INC=$'#include "wshtcpip.h"\n#include <iptypes.h>'
    ALLOW_FILES=( --allowlist-file '.*wshtcpip[/\\]wshtcpip\.h'
                  --allowlist-file '.*[/\\]wsahelp\.h' --allowlist-file '.*[/\\]tdiinfo\.h'
                  --allowlist-file '.*[/\\]tcpioctl\.h' --allowlist-file '.*[/\\]tdilib\.h'
                  --allowlist-file '.*[/\\]tdi\.h' --allowlist-file '.*[/\\]ws2tcpip\.h'
                  --allowlist-file '.*[/\\]winsock2\.h' --allowlist-file '.*[/\\]winsock\.h'
                  --allowlist-file '.*[/\\]ws2def\.h' --allowlist-file '.*[/\\]ws2ipdef\.h'
                  --allowlist-file '.*[/\\]iptypes\.h' )
    EXTRA_INC=( -I"${SRC}/sdk/lib/tdilib" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
    ;;
  mswsock)
    # USERMODE Winsock service-provider DLL (dll/win32/mswsock) — the Microsoft WinSock
    # extensions (AcceptEx/TransmitFile/GetAcceptExSockaddrs), the RNR/namespace-provider
    # wrappers (Get/SetService*, NSP* via nspapi/svcguid), and the WSPStartup helper glue.
    # Same USERMODE=1 treatment as ws2_32/wshtcpip: blocklist against ntdll-sys and re-export
    # `pub use ntdll_sys::nt::*;` so NT base types keep ONE identity with `use ntdll_sys::nt::*`
    # in the translated modules. Only the winsock/NSP/DNS structs go local.
    #
    # Type surface (from what the .c files pull on top of the windef/winbase PCH): winsock2 +
    # ws2spi (SOCKET/WSAPROTOCOL_INFO/LPFN_*/SOCKADDR), mswsock (TRANSMIT_FILE_BUFFERS +
    # the LPFN_ACCEPTEX/TRANSMITFILE fn-ptr typedefs), nspapi/svcguid (SERVICE_INFO/NS_*/
    # the GUID service classes RNR uses), windns (DnsQuery types nsplookup consults), iptypes
    # (IP_ADAPTER_INFO). A hand-invented sockaddr/WSAPROTOCOL_INFO is a silent wire-format bug
    # (scan_magic_offsets) — GENERATE them.
    #
    # BLOCKLIST_FUNCS=1: ws2spi/mswsock/nspapi declare WSP*/NSP*/mswsock prototypes the
    # translated modules define/call as their own externs (collide under the flat glob).
    # CALLBACK_SYSTEM=1: LPFN_ACCEPTEX/LPWSPStartup/NSP dispatch fn-ptrs are stdcall on i386;
    # a cdecl fn-ptr in a provider slot is a silent stack imbalance (the ws2_32 CALLBACK class).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/mswsock"
    WRAPPER_INC=$'#include <winsock2.h>\n#include <ws2spi.h>\n#include <mswsock.h>\n#include <nspapi.h>\n#include <svcguid.h>\n#include <windns.h>\n#include <iptypes.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ws2spi\.h' --allowlist-file '.*[/\\]winsock2\.h'
                  --allowlist-file '.*[/\\]winsock\.h' --allowlist-file '.*[/\\]ws2def\.h'
                  --allowlist-file '.*[/\\]ws2ipdef\.h' --allowlist-file '.*[/\\]ws2tcpip\.h'
                  --allowlist-file '.*[/\\]mswsock\.h' --allowlist-file '.*[/\\]nspapi\.h'
                  --allowlist-file '.*[/\\]svcguid\.h' --allowlist-file '.*[/\\]windns\.h'
                  --allowlist-file '.*[/\\]iptypes\.h' --allowlist-file '.*[/\\]nsp_dns\.h' )
    EXTRA_INC=( -I"${SRC}/sdk/include/reactos/winsock" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DLE )
    ;;
  ws2help)
    # USERMODE Winsock 2 Helper DLL (dll/win32/ws2help) — the Wah* handle-context table,
    # APC-helper, and notification-handle helpers that ws2_32/msafd use to map SOCKET handles
    # to per-provider context (WahInsert/EnumerateHandleContexts, WahQueueUserApc, ...). Same
    # USERMODE=1 treatment: blocklist against ntdll-sys and re-export `pub use ntdll_sys::nt::*;`
    # so NT base types keep ONE identity with `use ntdll_sys::nt::*` in the translated modules.
    #
    # Local structs: WSH_EA_DATA/WAH_EA_DATA2 (wshdrv.h — the WS2IFSL driver EA blobs), the
    # WAH handle-context table + WAH_HANDLE (ws2help.h), on top of the ws2spi/winsvc surface
    # the precomp PCH pulls. BLOCKLIST_FUNCS=1: ws2help.h declares the Wah* prototypes the
    # translation itself #[no_mangle]-defines. BLOCKLIST_VARS=1: the module keeps globals
    # (GlobalHeap/pSDPipe/ghWriterEvent/Ws2helpInitialized). CALLBACK_SYSTEM=1: the APC/request
    # routine fn-ptrs are stdcall on i386 (a cdecl fn-ptr in a dispatch slot is a silent stack
    # imbalance — the ws2_32 CALLBACK class).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    BLOCKLIST_VARS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/ws2help"
    # Mirror precomp.h's include ORDER exactly: windef/winbase/winreg define WINAPI+DWORD
    # BEFORE ws2help.h's `DWORD WINAPI Wah*()` prototypes (without them WINAPI is empty and the
    # decls parse as returning a function type). INCL_WINSOCK_API_TYPEDEFS is a VER_DEF below.
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winreg.h>\n#include <winsvc.h>\n#include <ws2spi.h>\n#include <ws2help.h>\n#include "wshdrv.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ws2help\.h' --allowlist-file '.*ws2help[/\\]wshdrv\.h'
                  --allowlist-file '.*[/\\]ws2spi\.h' --allowlist-file '.*[/\\]winsock2\.h'
                  --allowlist-file '.*[/\\]winsock\.h' --allowlist-file '.*[/\\]ws2def\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/ws2help" -I"${SRC}/sdk/include/reactos/winsock" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DINCL_WINSOCK_API_TYPEDEFS=1 )
    ;;
  userenv)
    # USERMODE user-environment DLL (dll/win32/userenv) — user profiles (Load/UnloadUserProfile,
    # Create/DeleteProfile), environment blocks (CreateEnvironmentBlock), group-policy path
    # resolution, and per-user shell-folder / registry-hive setup. Loaded on the BOOT PATH by
    # winlogon→userinit during profile load. USERMODE=1: blocklist against ntdll-sys and
    # re-export `pub use ntdll_sys::nt::*;` so NT base types keep ONE identity with the modules.
    #
    # Local structs: DYN_FUNCS/DYN_MODULE (internal.h — the GetProcAddress-loaded fn tables for
    # secur32/sfc/etc.), POLICY_VALUES (internal.h — gpolicy), PROFILEINFOA/W (userenv.h). Mirror
    # precomp.h's include order so WINAPI/DWORD resolve before userenv.h's WINAPI prototypes.
    # BLOCKLIST_FUNCS=1: userenv.h/internal.h declare the profile/environment prototypes the
    # translation #[no_mangle]-defines. CALLBACK_SYSTEM=1: DYN_FUNCS holds stdcall Win32-API
    # fn-ptrs (a cdecl fn-ptr slot is a silent i386 stack imbalance — the ws2_32 CALLBACK class).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/userenv"
    # objbase.h (LPUNKNOWN) + ndk/rtlfuncs.h (PUNICODE_STRING) are pulled by precomp.h before
    # internal.h — include them here in the same order or internal.h's fields won't resolve.
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winreg.h>\n#include <winnls.h>\n#include <objbase.h>\n#include <ndk/rtlfuncs.h>\n#include <userenv.h>\n#include "internal.h"'
    ALLOW_FILES=( --allowlist-file '.*userenv[/\\]internal\.h' --allowlist-file '.*[/\\]userenv\.h'
                  --allowlist-file '.*userenv[/\\]resources\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/userenv" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DNTOS_MODE_USER -DWIN32_NO_STATUS
               -D_INC_WINDOWS -DCOM_NO_WINDOWS_H )
    ;;
  netapi32)
    # USERMODE LAN Manager API DLL (dll/win32/netapi32) — NetUser/Group/LocalGroup/Share/Server/
    # Wksta/Dfs/Alert/Message APIs + the NetBIOS (nb30) surface. USERMODE=1: blocklist ntdll-sys,
    # re-export `pub use ntdll_sys::nt::*;` so NT base types keep ONE identity with the modules.
    #
    # Type surface: <lm.h> is the LAN Manager master header — it pulls the whole lm* family
    # (lmaccess/lmshare/lmserver/lmwksta/lmsvc/lmat/... → USER_INFO_*/GROUP_INFO_*/
    # LOCALGROUP_INFO_*/SHARE_INFO_*/SERVER_INFO_*/WKSTA_INFO_*/AT_INFO/...) in one include.
    # nb30 (NCB/NetBIOS), the local nbnamecache/netbios private structs. dfs.rs's one RPC call
    # (NetrDfsAdd) + DFS_INFO come via the MIDL netdfs_c.h in the build dir (EXTRA_INC). A
    # hand-invented *_INFO_* is a silent wire-format bug (scan_magic_offsets) — GENERATE them.
    #
    # BLOCKLIST_FUNCS=1: lm*/nb30 declare the Net*/Netbios prototypes the translation
    # #[no_mangle]-defines. CALLBACK_SYSTEM=1: the enum/worker callback fn-ptrs are stdcall on
    # i386 (a cdecl fn-ptr slot is a silent stack imbalance — the ws2_32 CALLBACK class).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/netapi32"
    # Mirror netapi32.h's include surface: winsock2 (SOCKET/WSABUF/sockaddr_in used by user/nbt),
    # <lm.h> (the whole LAN Manager family), nb30 (NCB), iphlpapi (MIB_*), ntsecapi+ntsam
    # (LSA_UNICODE_STRING/SAM_HANDLE/USER_ALL_*/DOMAIN_* — user.rs/group_new drive the SAM/LSA
    # surface), wine/debug.h (the __wine_debug_channel/__wine_debug_class/__WINE_DBCL_* the
    # c2rust-expanded WINE_DEFAULT_DEBUG_CHANNEL/TRACE references), and the local private structs.
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winsock2.h>\n#include <lm.h>\n#include <nb30.h>\n#include <iphlpapi.h>\n#include <ntsecapi.h>\n#include <ndk/rtlfuncs.h>\n#include <ntsam.h>\n#include <wine/debug.h>\n#include "nbnamecache.h"\n#include "netbios.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]lm[a-z]*\.h' --allowlist-file '.*netapi32[/\\].*\.h'
                  --allowlist-file '.*[/\\]nb30\.h' --allowlist-file '.*[/\\]netdfs_c\.h'
                  --allowlist-file '.*[/\\]ntsecapi\.h' --allowlist-file '.*[/\\]ntsam\.h'
                  --allowlist-file '.*[/\\]winsock2\.h' --allowlist-file '.*[/\\]winsock\.h'
                  --allowlist-file '.*[/\\]ws2def\.h' --allowlist-file '.*[/\\]iphlpapi\.h' )
    # wine/debug.h stays in WRAPPER_INC (for the parse) but is NOT allowlisted: the
    # __wine_debug_channel/__wine_debug_class/__WINE_DBCL_*/ros_dbg_log shim is defined in the
    # crate prelude (the lsasrv-rust pattern) so the c2rust-expanded TRACE/ERR path resolves
    # to a release-inert channel + the real variadic ros_dbg_log at link.
    # netdfs_c.h (MIDL client stub for dfs.rs's NetrDfsAdd + DFS_INFO) is generated into the
    # build dir; the wire types are arch-independent so build-i386's copy serves all 3 arches.
    EXTRA_INC=( -I"${SRC}/dll/win32/netapi32" -I"${SRC}/build-i386/dll/win32/netapi32" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DUNICODE -D_UNICODE -DNTOS_MODE_USER )
    ;;
  dnsapi)
    # USERMODE DNS client API DLL (dll/win32/dnsapi) — DnsQuery/DnsRecordList*/DnsNameCompare/
    # DnsExtractRecords + the adns-backed resolver. USERMODE=1: blocklist ntdll-sys, re-export
    # the NT base types so they keep ONE identity with the modules.
    #
    # Type surface: windns.h (DNS_RECORD/DNS_STATUS/DNS_RRSET/IP4_ARRAY/DNS_HEADER/DNS_WIRE_*),
    # reactos/windns_undoc.h (the ROS-private extras), adns.h (the 3rdparty async resolver —
    # adns_state/adns_answer/adns_rr_*/adns_status/adns_rrtype; query.c/record.c drive it), and
    # dnsrslvr_c.h (the SINGLE MIDL client interface: DNSRSLVR_HANDLE + R_Resolver* stubs, used
    # only by rpc.c). BLOCKLIST_FUNCS=1: windns.h declares the DnsQuery*/DnsRecord* prototypes the
    # translation #[no_mangle]-defines; adns_* + R_Resolver* stubs come from the linked
    # adns staticlib / dnsrslvr_c.c (extern in prelude). wine/debug.h parsed-not-allowlisted (the
    # __wine_debug_channel/ros_dbg_log shim lives in the crate prelude — the lsasrv/netapi32 pattern).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/dnsapi"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winnls.h>\n#include <windns.h>\n#include <reactos/windns_undoc.h>\n#define NTOS_MODE_USER\n#include <ndk/rtlfuncs.h>\n#include <adns.h>\n#include <dnsrslvr_c.h>\n#include <wine/debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]windns\.h' --allowlist-file '.*[/\\]windns_undoc\.h'
                  --allowlist-file '.*[/\\]adns\.h' --allowlist-file '.*[/\\]dnsrslvr_c\.h'
                  --allowlist-file '.*dnsapi[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/dnsapi" -I"${SRC}/build-i386/dll/win32/dnsapi"
                -I"${SRC}/sdk/lib/3rdparty/adns/src" -I"${SRC}/sdk/lib/3rdparty/adns/adns_win32" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -DNTOS_MODE_USER -DADNS_JGAA_WIN32 -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  dhcpcsvc)
    # USERMODE DHCP client service DLL (base/services/dhcpcsvc — loaded by services.exe). The
    # DHCP client API (Dhcpc*/GetAdapterInfo) + the BSD-derived dhclient state machine (dhcp/).
    # USERMODE=1: blocklist ntdll-sys, re-export NT base types.
    #
    # Type surface: rosdhcp.h is the master private header — it pulls dhcpcsdk.h (DhcpApi structs),
    # rpc.h + dhcpcsvc_c.h/_s.h (the SINGLE MIDL interface, client+server), and dhcpd.h (the ISC
    # dhclient universe: struct tree/hash_table/lease/iaddr/interface_info/packet/dhcp_packet +
    # the option tables). dhcp.h/tree.h/hash.h are the dhcp/ local headers. BLOCKLIST_FUNCS=1:
    # dhcpcsdk.h/dhcpd.h declare the Dhcpc*/dhclient prototypes the translation #[no_mangle]-defines;
    # the R_Dhcp*/Client_/Server_ MIDL stubs come from dhcpcsvc_c.c (extern in prelude). wine/debug.h
    # is NOT used here (ROS DPRINT); the debug shim (if any) surfaces in the grind.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/dhcpcsvc"
    # rosdhcp.h is the master private header — it sets up the BSD typedefs (ssize_t/u_int8_t/
    # u_int16_t/u_int32_t/caddr_t) THEN includes dhcpcsdk/rpc/dhcpcsvc_c/_s/dhcpd.h. Including
    # dhcpd.h directly fails (its ssize_t/u_int*_t are undefined), so mirror the .c's PCH include.
    WRAPPER_INC=$'#include <rosdhcp.h>'
    ALLOW_FILES=( --allowlist-file '.*dhcpcsvc[/\\].*\.h' --allowlist-file '.*[/\\]dhcpd\.h'
                  --allowlist-file '.*[/\\]dhcp\.h' --allowlist-file '.*[/\\]tree\.h'
                  --allowlist-file '.*[/\\]hash\.h' --allowlist-file '.*[/\\]rosdhcp\.h'
                  --allowlist-file '.*[/\\]dhcpcsdk\.h' --allowlist-file '.*[/\\]dhcpcsvc_c\.h'
                  --allowlist-file '.*[/\\]dhcpcsvc_s\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/dhcpcsvc/include" -I"${SRC}/base/services/dhcpcsvc/dhcp"
                -I"${SRC}/build-i386/base/services/dhcpcsvc" )
    # dhcpd.h:398 declares `extern time_t cur_time;` but NO translation unit in this DLL defines
    # it — every user (state_selecting/dhcpack/reset_adapter/bind_lease/dhcpoffer/send_discover/
    # send_request/dispatch) declares its OWN `time_t cur_time;` local, which shadows it in C.
    # Emitting the decl made the faithful translation impossible: Rust rejects a `let` that
    # shadows an in-scope static (E0530), which pushed a translation pass into "it's a global"
    # and aliased 8 independent stack values into one. Blocklist it. Field manual §9.108.
    EXTRA_BL_ITEMS=( cur_time )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -DNTOS_MODE_USER -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  iphlpapi)
    # USERMODE IP Helper API DLL (dll/win32/iphlpapi) — PARTIAL flip: only iphlpapi_main.c → Rust
    # (74 fns: Add/DeleteIPAddress, Get{IpAddr,IpForward,IpNet,If,Tcp,Udp}Table, GetNetworkParams,
    # GetAdaptersInfo/AddressesV2, GetPerAdapterInfo, ICMP/route/net-entry CRUD, the qsort *Sorter
    # comparators). The ~10 sibling *_reactos.c/icmp.c/media.c/registry.c/address.c STAY C —
    # iphlpapi_main.c calls into them via the local ifenum.h/ipstats.h/route.h/resinfo.h/dhcp.h
    # prototypes, so those headers must be in the parse (extern imports the Rust re-declares).
    #
    # Type surface: iphlpapi.h + iphlpapi_undoc.h (IP_ADDR_STRING/IP_PER_ADAPTER_INFO/FIXED_INFO/
    # IP_ADAPTER_INFO/MIB_*TABLE/MIB_*ROW/MIB_ICMP/MIB_IPSTATS), tdiinfo.h/tcpioctl.h (TDIEntityID/
    # TCP_REQUEST_QUERY_INFORMATION), tdilib.h, the local iphlpapi private headers. USERMODE=1:
    # blocklist ntdll-sys, re-export the NT base types so they keep ONE identity with the module.
    # BLOCKLIST_FUNCS=1: iphlpapi.h declares the AddIPAddress/GetIpAddrTable/... prototypes the
    # translation #[no_mangle]-defines. wine/debug.h stays in the parse but is NOT allowlisted
    # (the __wine_debug_channel/__WINE_DBCL_*/ros_dbg_log shim is in the crate prelude — the
    # lsasrv/netapi32 pattern).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/iphlpapi"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winreg.h>\n#include <winnls.h>\n#include <ws2tcpip.h>\n#define NTOS_MODE_USER\n#include <ndk/rtlfuncs.h>\n#include <iphlpapi.h>\n#include <iphlpapi_undoc.h>\n#include <tdiinfo.h>\n#include <tcpioctl.h>\n#include <tdilib.h>\n#include <fltdefs.h>\n#include <wine/debug.h>\n#include "resinfo.h"\n#include "dhcp.h"\n#include "ifenum.h"\n#include "ipstats.h"\n#include "route.h"\n#include "iphlpapi_private.h"'
    # fltdefs.h: INTERFACE_HANDLE / PF_FILTER_DESCRIPTOR / PFADDRESSTYPE / PFFORWARD_ACTION /
    # PF_INTERFACE_STATS / PF_LATEBIND_INFO / GLOBAL_FILTER — the packet-filter surface the 16
    # Pf* exports in iphlpapi_fakestubs.c take. Needed once the flip went FULL (the island was
    # originally built for iphlpapi_main.c alone, which touches none of them).
    ALLOW_FILES=( --allowlist-file '.*[/\\]fltdefs\.h'
                  --allowlist-file '.*[/\\]iphlpapi\.h' --allowlist-file '.*[/\\]iphlpapi_undoc\.h'
                  --allowlist-file '.*[/\\]iprtrmib\.h' --allowlist-file '.*[/\\]ipexport\.h'
                  --allowlist-file '.*[/\\]iptypes\.h' --allowlist-file '.*[/\\]ipmib\.h'
                  --allowlist-file '.*[/\\]tcpmib\.h' --allowlist-file '.*[/\\]udpmib\.h'
                  --allowlist-file '.*[/\\]tdiinfo\.h' --allowlist-file '.*[/\\]tcpioctl\.h'
                  --allowlist-file '.*[/\\]tdilib\.h' --allowlist-file '.*iphlpapi[/\\].*\.h'
                  --allowlist-file '.*[/\\]ws2def\.h' --allowlist-file '.*[/\\]ws2ipdef\.h'
                  --allowlist-file '.*[/\\]ipifcons\.h' --allowlist-file '.*[/\\]ifmib\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/iphlpapi" -I"${SRC}/dll/win32/dhcpcsvc/include"
                -I"${SRC}/sdk/lib/tdilib" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_UNICODE -DNTOS_MODE_USER
               -DGetAdaptersAddressesV2 -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  usbstor)
    # USB mass-storage class driver (drivers/usb/usbstor). All private structs are defined
    # INLINE in usbstor.h: CBW/CSW (Bulk-Only command/status wrappers, pshpack1), IRP_CONTEXT,
    # __COMMON_DEVICE_EXTENSION__ + the FDO/PDO device extensions, ERRORHANDLER_WORKITEM_DATA,
    # STORAGE_ADAPTER_DESCRIPTOR_WIN8. usbstor.h pulls wdm/ntstrsafe/usbdi/usbbusif/usbdlib/
    # classpnp -> allowlist ONLY usbstor.h so bindgen emits just the driver-private types, not
    # the whole USBDI/classpnp surface (those come from the kernel ddk bindgen). Includes
    # classpnp.h -> match classpnp's Win8 (0x602) version profile.
    DRIVER_DIR="${SRC}/drivers/usb/usbstor"
    WRAPPER_INC=$'#include "usbstor.h"'
    # usbstor function bodies use raw SCSI/USB types directly (INQUIRYDATA, SENSE_DATA,
    # SCSI_ADDRESS, USB_*_DESCRIPTOR, USBD_INTERFACE_LIST_ENTRY) that aren't transitively
    # reachable from the device-extension structs and aren't in the kernel ddk bindgen ->
    # allowlist their definition headers (scsi.h, usb.h, usbdi.h, usbdlib.h) alongside
    # usbstor.h so the crate types.rs carries the full SCSI/USB surface.
    ALLOW_FILES=( --allowlist-file '.*usbstor[/\\]usbstor\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h'
                  --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usbdi\.h'
                  --allowlist-file '.*[/\\]usbdlib\.h' )
    # _SCSI_REQUEST_BLOCK is OPAQUE in the kernel ddk bindgen ({ _address: u8 }) — unblock
    # it (and its aliases) so usbstor's own bindgen (which includes scsi.h) emits the FULL
    # SRB (SrbStatus/Cdb/DataBuffer/SrbFlags/DataTransferLength/...). Same as scsiport/atapi.
    UNBLOCK="_SCSI_REQUEST_BLOCK SCSI_REQUEST_BLOCK PSCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x602 -D_WIN32_WINNT=0x602 -DNTDDI_VERSION=0x06020000 )
    ;;
  usbccgp)
    # USB composite-device (generic parent) driver (drivers/usb/usbccgp). Private structs are
    # in usbccgp.h (FDO/PDO/function-descriptor extensions); it pulls hubbusif/usbbusif/usb100/
    # usbdlib. USB-only (no SCSI). Allowlist usbccgp.h + the USB definition headers so the crate
    # types.rs carries the USB descriptor/interface surface. Vista (_WIN32_WINNT=0x600).
    DRIVER_DIR="${SRC}/drivers/usb/usbccgp"
    WRAPPER_INC=$'#include "usbccgp.h"'
    ALLOW_FILES=( --allowlist-file '.*usbccgp[/\\]usbccgp\.h'
                  --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usb200\.h'
                  --allowlist-file '.*[/\\]usbdi\.h'
                  --allowlist-file '.*[/\\]usbdlib\.h'
                  --allowlist-file '.*[/\\]usbioctl\.h'
                  --allowlist-file '.*[/\\]hubbusif\.h'
                  --allowlist-file '.*[/\\]usbbusif\.h' )
    VER_DEFS=( -DWINVER=0x600 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 )
    ;;
  usbhub)
    # USB hub class driver (drivers/usb/usbhub). Many private structs in usbhub.h (USBHUB_FDO_
    # EXTENSION/PORT_PDO_EXTENSION + URB/status/idle/reset contexts); it pulls usb/usbioctl/
    # hubbusif/usbbusif/usbdlib + the miniport interface (drivers/usbport/usbmport.h) + ks/wmilib.
    # Allowlist usbhub.h + the USB headers + usbmport.h so the crate types.rs carries the hub +
    # miniport surface. _WIN32_WINNT default for the USB stack = Vista (0x600).
    DRIVER_DIR="${SRC}/drivers/usb/usbhub"
    WRAPPER_INC=$'#include "usbhub.h"'
    ALLOW_FILES=( --allowlist-file '.*usbhub[/\\]usbhub\.h'
                  --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usbioctl\.h'
                  --allowlist-file '.*[/\\]usbdlib\.h'
                  --allowlist-file '.*[/\\]hubbusif\.h'
                  --allowlist-file '.*[/\\]usbbusif\.h'
                  --allowlist-file '.*[/\\]usbmport\.h' )
    VER_DEFS=( -DWINVER=0x600 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 )
    ;;
  usbehci|usbohci|usbuhci)
    # USB host-controller miniports (drivers/usb/usb{ehci,ohci,uhci}). Each registers a
    # USBPORT_REGISTRATION_PACKET with usbport (drivers/usbport/usbmport.h) and drives its
    # controller hardware. Driver-private structs live in BOTH the main header (usbXhci.h:
    # endpoint/transfer/extension structs) and hardware.h (HW register/descriptor layouts:
    # EHCI QH/TD, OHCI ED/TD, UHCI QH/TD). Allowlist the whole driver dir + the USB headers +
    # usbmport.h so types.rs carries the full miniport + hardware surface. USB stack = Vista.
    DRIVER_DIR="${SRC}/drivers/usb/${DRIVER}"
    WRAPPER_INC=$'#include "'"${DRIVER}"'.h"'
    ALLOW_FILES=( --allowlist-file ".*${DRIVER}[/\\\\].*\\.h"
                  --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usbdi\.h'
                  --allowlist-file '.*[/\\]usbdlib\.h'
                  --allowlist-file '.*[/\\]hubbusif\.h'
                  --allowlist-file '.*[/\\]usbbusif\.h'
                  --allowlist-file '.*[/\\]usbmport\.h' )
    VER_DEFS=( -DWINVER=0x600 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 )
    ;;
  usbport)
    # USB host-controller PORT framework (drivers/usb/usbport). The miniport substrate the
    # ehci/ohci/uhci miniports register WITH. All driver-private structs live in usbport.h
    # (USBPORT_DEVICE_EXTENSION/ENDPOINT/TRANSFER/PIPE_HANDLE, roothub, USB2 scheduler, ...)
    # + usbdebug.h. usbport DEFINES usbmport.h (the registration packet) so allowlist it local.
    # Allowlist the whole driver dir + the USB headers + usbmport.h. USB stack = Vista.
    DRIVER_DIR="${SRC}/drivers/usb/usbport"
    WRAPPER_INC=$'#include "usbport.h"'
    ALLOW_FILES=( --allowlist-file '.*usbport[/\\].*\.h'
                  --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usbdi\.h'
                  --allowlist-file '.*[/\\]usbdlib\.h'
                  --allowlist-file '.*[/\\]hubbusif\.h'
                  --allowlist-file '.*[/\\]usbbusif\.h'
                  --allowlist-file '.*[/\\]usbmport\.h' )
    VER_DEFS=( -DWINVER=0x600 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 )
    ;;
  netapi32-rpc|netapi32-browser)
    # The netapi32 RPC-STUB TAIL (browser/dssetup/misc/netlogon/schedule/srvsvc/wksta_new) —
    # the 7 files the c2rust-anchored pass could not take. Their types come from the WIDL
    # *_c.h client stubs, which do NOT coexist with the `netapi32` profile's <lm.h>:
    #
    #   <lm.h> is the LAN Manager MASTER header — it pulls lmat/lmshare/lmserver/lmwksta,
    #   which declare AT_INFO/CONNECTION_INFO_*/SERVER_INFO_*/WKSTA_INFO_* — the SAME structs
    #   the IDL re-declares for the wire. netapi32.h itself never includes <lm.h>: it pulls
    #   only lmaccess/lmapibuf/lmerr and lets each *_c.h supply its own wire types. Mirroring
    #   that narrow include set is what makes these parse (verified: all 6 interfaces).
    #
    # Two islands, because two pairs are genuinely irreconcilable in one TU (the C never asks
    # for either combination):
    #   netapi32-rpc     → srvsvc + wkssvc + atsvc + dssetup + netlogon  (serves 6 of the 7
    #                      modules; dsrole.h is NOT included — dssetup_c.h re-declares DSROLE_*
    #                      and netlogon.c never references them despite including it)
    #   netapi32-browser → browser_c.h + lmbrowsr/lmserver (browser_c.h re-declares
    #                      _SERVER_INFO_100, which collides with srvsvc_c.h)
    # BLOCKLIST_FUNCS=1: the *_c.h declare the Netr*/Netp* client prototypes the MIDL .c stubs
    # define (those stay C) — bindgen must not re-emit them. USERMODE=1: one NT base identity.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    CALLBACK_SYSTEM=1
    DRIVER_DIR="${SRC}/dll/win32/netapi32"
    _NA32_BASE=$'#include <windef.h>\n#include <winbase.h>\n#include <winsock2.h>\n#include <lmaccess.h>\n#include <lmapibuf.h>\n#include <lmerr.h>\n#include <lmcons.h>\n#include <lmjoin.h>\n#include <ntsecapi.h>\n#include <nb30.h>\n#include <iphlpapi.h>\n#include <ndk/rtlfuncs.h>\n#include <ntsam.h>\n#include <wine/debug.h>\n#include <rpc.h>\n'
    if [ "$DRIVER" = netapi32-browser ]; then
      WRAPPER_INC="${_NA32_BASE}"$'#include <lmbrowsr.h>\n#include <lmserver.h>\n#include "browser_c.h"'
    else
      WRAPPER_INC="${_NA32_BASE}"$'#include <dsgetdc.h>\n#include "srvsvc_c.h"\n#include "wkssvc_c.h"\n#include "atsvc_c.h"\n#include "dssetup_c.h"\n#include "netlogon_c.h"'
    fi
    # The MIDL wire types (*_c.h) PLUS the narrow set of SUPPORTING headers the 7 tail modules
    # actually name. The `.*_c\.h`-only allowlist was too tight: the tail C files reference
    # RPC-runtime types (RPC_STATUS/RPC_WSTR), the LM error/const surface (NERR_*, LMSTR), the
    # DsGetDcName types (DOMAIN_CONTROLLER_INFO*/DS_DOMAIN_TRUSTS*), the LSA policy-notify enum,
    # and NETSETUP_JOIN_STATUS — none of which any *_c.h declares, so they resolved nowhere and
    # cost ~40 E0412/E0425. They cannot be borrowed from the sibling `types` island either:
    # these are SEPARATE Rust modules, so a name may legitimately exist in both (the islands
    # exist because the two C include-sets are irreconcilable in ONE TU, not because the Rust
    # names collide). Emitting them here is what gives the tail a resolvable, EXACT layout —
    # hand-transcribing a struct layout is never allowed.
    ALLOW_FILES=( --allowlist-file '.*_c\.h'
                  --allowlist-file '.*[/\\]rpcdce\.h'   --allowlist-file '.*[/\\]rpcdcep\.h'
                  --allowlist-file '.*[/\\]rpcndr\.h'   --allowlist-file '.*[/\\]lmcons\.h'
                  --allowlist-file '.*[/\\]lmerr\.h'    --allowlist-file '.*[/\\]lmjoin\.h'
                  --allowlist-file '.*[/\\]dsgetdc\.h'  --allowlist-file '.*[/\\]ntsecapi\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/netapi32" -I"${SRC}/build-i386/dll/win32/netapi32" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 -DUNICODE -D_UNICODE -DNTOS_MODE_USER )
    # both islands live in the netapi32-rust crate
    OUT_CRATE_DIR="${OUT_CRATE_DIR:-${SRC}/sdk/lib/rust/netapi32-rust/src}"
    ;;
  *) echo "unknown driver: $DRIVER" >&2; exit 2 ;;
esac

# Output crate dir defaults to main repo; override with OUT_CRATE_DIR for a worktree.
OUT_CRATE="${OUT_CRATE_DIR:-${SRC}/sdk/lib/rust/${DRIVER}-rust/src}"
mkdir -p "$OUT_CRATE"
# TYPES_STEM: emit the island as <stem>_<arch>.rs + <stem>.rs instead of types_<arch>.rs.
# A crate needs MORE THAN ONE island when its C translation units have mutually
# INCOMPATIBLE include sets — the netapi32 RPC tail is the worked example: srvsvc_c.h and
# browser_c.h each declare _SERVER_INFO_100, and dssetup_c.h re-declares dsrole.h's DSROLE_*,
# so no single bindgen pass can cover them. One island per compatible TU group mirrors what
# the C build already proves legal (each .c compiles), instead of fighting the redefinitions.
STEM="${TYPES_STEM:-types}"
WRAPPER="/tmp/${DRIVER}_${STEM}_wrapper.h"
printf '%s\n' "$WRAPPER_INC" > "$WRAPPER"

run_arch() {
  # BUILD_DIR supplies the configure-time-GENERATED headers (xdk/ddk/mc). A worktree
  # without its own build dirs can borrow another tree's via BUILD_ROOT (or legacy
  # BUILD_DIR_BASE); source headers under $SRC are identical git content, only
  # build-*/sdk/include is generated.
  local ARCH="$1" TARGET="$2" ARCHDEF="$3" BUILD_DIR="${BUILD_ROOT:-${BUILD_DIR_BASE:-${SRC}}}/build-${1}"
  local NTK="${NTKRNL_SYS}/bindgen_${ARCH}.rs"
  local RAW="/tmp/${DRIVER}_${STEM}_${ARCH}.rs"
  echo "=== ${DRIVER} types: ${ARCH} (${TARGET}) ==="

  # Blocklist SOURCE + re-export glob: kernel drivers blocklist against the ntkrnl-sys
  # kernel bindgen and resolve field types via the private `use ntkrnl_sys::ddk::*` glob.
  # USERMODE crates (msafd) blocklist against the ntdll-sys NDK bindgen (bindgen_<arch> +
  # the shared bindgen_common) and re-export `pub use ntdll_sys::nt::*;` PUBLICLY so the
  # translated modules get NT base types via `use crate::types::*`.
  local BLSRCS=( "$NTK" )
  local REEXPORT_LINE='use ntkrnl_sys::ddk::*;'
  if [ "${USERMODE:-0}" = 1 ]; then
    BLSRCS=( "${NTDLL_SYS}/bindgen_${ARCH}.rs" "${NTDLL_SYS}/bindgen_common.rs" )
    REEXPORT_LINE='pub use ntdll_sys::nt::*;'
  fi

  # Blocklist = every type name the shared ntkrnl-sys bindgen already exports.
  # Same list is re-exported (types only) from this crate so the LLM-translated
  # modules find kernel types (DEVICE_OBJECT, IRP, CDB, ...) via `use crate::types::*`
  # — matching how they were written — WITHOUT pulling ddk functions/consts (those
  # stay on the modules own explicit `use ntkrnl_sys::ddk::{fn}` lists; re-exporting
  # them here would create glob-vs-explicit ambiguity).
  # Blocklist = every type name the shared ntkrnl-sys bindgen exports, so kernel types
  # keep a single identity and only driver-PRIVATE structs become local here. The
  # translated modules then get private structs from `crate::types` and kernel types
  # from `ntkrnl_sys::{ddk,driver_ddk,...}`.
  local TYPELIST="/tmp/${DRIVER}_${STEM}_ntktypes_${ARCH}.txt"
  grep -hoE "pub (struct|enum|type|union) [A-Za-z0-9_]+" "${BLSRCS[@]}" | awk '{print $3}' | sort -u > "$TYPELIST"
  # UNBLOCK: types the shared ntkrnl-sys bindgen leaves OPAQUE (e.g. `_SCSI_REQUEST_BLOCK
  # { _address: u8 }`) — keeping them blocklisted resolves the driver's field accesses to the
  # useless opaque version. Re-allow them so THIS driver's bindgen (which includes scsi.h/srb.h)
  # emits the FULL struct locally; the driver then uses crate::types::<T> with real fields.
  if [ -n "${UNBLOCK:-}" ]; then
    local KEEP="/tmp/${DRIVER}_unblock_${ARCH}.txt"; printf '%s\n' $UNBLOCK > "$KEEP"
    grep -vxF -f "$KEEP" "$TYPELIST" > "${TYPELIST}.f" && mv "${TYPELIST}.f" "$TYPELIST"
  fi
  # Extra sibling -sys crates (e.g. acpica-sys): blocklist their types too so the
  # ACPI_* names acpi_bus.h references resolve to that crate, not get re-emitted here.
  # Re-export must be EXPLICIT (only names UNIQUE to the sys crate) — a `pub use
  # acpica_sys::*` glob collides with the `use ntkrnl_sys::ddk::*` glob on every shared
  # C primitive (BOOLEAN, UINT8, __BindgenBitfieldUnit, ...) → E0659 ambiguity. Shared
  # names resolve via the ddk glob; only acpica-unique names (ACPI_*, acpi_*, AE_*) are
  # re-exported by name.
  local SYS_USE=()
  for spec in "${SYS_CRATES[@]:-}"; do
    [ -z "$spec" ] && continue
    # spec = crate:subdir[:stem]  (stem defaults to "bindgen"; kmdf-sys uses "types"
    # because it's itself a gen_driver_types output, not an acpica-style bindgen crate).
    local crate subdir stem
    IFS=: read -r crate subdir stem <<< "$spec"
    stem="${stem:-bindgen}"
    local SYSBG="${SRC}/sdk/lib/rust/${subdir}/${stem}_${ARCH}.rs"
    local SYSNAMES="/tmp/${DRIVER}_${crate}_names_${ARCH}.txt"
    grep -oE "pub (struct|enum|type|union) [A-Za-z0-9_]+" "$SYSBG" | awk '{print $3}' | sort -u > "$SYSNAMES"
    # Unique = in sys crate but NOT already an ntkrnl-sys type (TYPELIST still ntkrnl-only here).
    local UNIQUE; UNIQUE="$(comm -23 "$SYSNAMES" "$TYPELIST" | paste -sd, -)"
    SYS_USE+=( --raw-line "pub use ${crate//-/_}::{${UNIQUE}};" )
    cat "$SYSNAMES" >> "$TYPELIST"   # blocklist ALL sys types (none re-emitted locally)
  done
  sort -u "$TYPELIST" -o "$TYPELIST"
  local BL=()
  while IFS= read -r t; do BL+=( --blocklist-type "$t" ); done < "$TYPELIST"
  # BLOCKLIST_FUNCS: drop every function declaration (usermode wrappers re-declare the
  # WSP* prototypes the translation defines — see the msafd case). Kernel drivers keep
  # fns (e.g. afd's c_assert) so the default leaves FN_BL empty.
  # EXTRA_BL_ITEMS: per-profile `--blocklist-item` entries. Use for a header symbol that is
  # DECLARED but never DEFINED anywhere in the DLL (a vestigial `extern` the upstream project
  # left behind). Emitting such a decl is actively harmful: cross-module code silently binds to
  # a global that does not exist (undefined symbol at DLL link), and — worse — it makes the
  # correct translation of a same-named C LOCAL illegal, because Rust forbids a `let` binding
  # shadowing an in-scope static (E0530). See the dhcpcsvc `cur_time` case, field manual §9.108.
  local ITEM_BL=()
  for _it in "${EXTRA_BL_ITEMS[@]-}"; do [ -n "$_it" ] && ITEM_BL+=( --blocklist-item "$_it" ); done
  local FN_BL=()
  if [ "${BLOCKLIST_FUNCS:-0}" = 1 ]; then FN_BL=( --blocklist-function '.*' ); fi
  # BLOCKLIST_VARS: drop every extern `static` (global variable) declaration via a post-pass
  # (see below). Usermode DLLs (msafd) define their own module globals (SockEvent/GlobalHeap/
  # Upcalls/...) in the translation; bindgen re-emitting them as `pub static mut` collides
  # (E0530 "let bindings cannot shadow statics") with the translation's local defs. NOTE: we
  # do NOT use bindgen `--blocklist-var` — bindgen models #define constants as "vars" too, so
  # `--blocklist-var '.*'` also nukes every winsock const (SOCKET_ERROR/FD_*/AF_*/...). The
  # post-pass strips only `pub static` lines, leaving the consts intact.
  # MACRO_SIGNED: emit #define integer macros as i32 instead of the u32 default. The Win32
  # usermode API surface (WSA* error codes, AF_*/SOCK_*/SO_*/IPPROTO_*/FD_*/MSG_*) is `int`,
  # so the translation stores them in INT (i32) locals and compares against i32 params — a
  # u32 default floods E0308/E0277 across every winsock const use. The few genuinely-DWORD
  # high-bit consts (INADDR_NONE, WSA_INFINITE, IN_CLASS*_NET, IOC_IN/INOUT) flip to negative
  # i32 with the same bit pattern; those are re-forced to u32 by the post-pass below.
  local MACRO_ARGS=()
  if [ "${MACRO_SIGNED:-0}" = 1 ]; then MACRO_ARGS=( --default-macro-constant-type signed ); fi

  bindgen "$WRAPPER" \
    --use-core --no-layout-tests --no-doc-comments \
    "${MACRO_ARGS[@]}" \
    "${ALLOW_FILES[@]}" \
    "${BL[@]}" "${FN_BL[@]}" "${ITEM_BL[@]}" \
    --raw-line '// @generated by tools/rustconv/gen_driver_types.sh — driver-private structs.' \
    --raw-line '#![allow(non_camel_case_types, non_snake_case, dead_code)]' \
    --raw-line '// base field types resolved via the sibling -sys glob below (kernel: private' \
    --raw-line '// ntkrnl_sys::ddk; usermode: public ntdll_sys::nt — see USERMODE in the script).' \
    --raw-line "$REEXPORT_LINE" \
    "${SYS_USE[@]}" \
    -o "$RAW" \
    -- -ferror-limit=0 -Wno-everything \
    -fms-extensions "${LANG_ARGS[@]}" \
    $ARCHDEF -D__REACTOS__ -D_USE_DUMMY_PSEH=1 -DDBG=1 "${VER_DEFS[@]}" \
    -I"${DRIVER_DIR}" "${EXTRA_INC[@]}" \
    -I"${SRC}/sdk/include" -I"${SRC}/sdk/include/ddk" -I"${SRC}/sdk/include/ndk" \
    -I"${SRC}/sdk/include/psdk" -I"${SRC}/sdk/include/xdk" -I"${SRC}/sdk/include/reactos" \
    -I"${SRC}/sdk/include/reactos/libs" -I"${SRC}/sdk/include/reactos/drivers" \
    -I"${SRC}/sdk/include/vcruntime" -I"${BUILD_DIR}/sdk/include" -I"${BUILD_DIR}/sdk/include/psdk" \
    -I"${BUILD_DIR}/sdk/include/ddk" -I"${BUILD_DIR}/sdk/include/reactos" \
    -I"${BUILD_DIR}/sdk/include/reactos/mc" -I"${SRC}/sdk/include/dxsdk" \
    -I"${SRC}/sdk/lib/pseh/include" \
    -isystem "${SRC}/sdk/include/crt" -isystem "${TOOLCHAIN}/lib/clang/19/include" \
    --target="${TARGET}"

  # Strip bindgen's `pub use self::_X as Y;` re-exports of BLOCKLISTED enum typedefs:
  # _X comes from the private `use ntkrnl_sys::ddk::*` glob, so re-exporting it is E0365
  # ("re-export of private item") which poisons the whole module → every private struct
  # becomes unreachable. Y (DEVICE_POWER_STATE, POOL_TYPE, ...) is a kernel type the
  # modules import from ddk/driver_ddk anyway, so dropping the alias here is safe.
  local FINAL="${OUT_CRATE}/${STEM}_${ARCH}.rs"
  python3 -c '
import sys, re
raw, final = sys.argv[1], sys.argv[2]
s = open(raw).read()
local = set(re.findall(r"^pub (?:struct|enum|union|type) (_[A-Za-z0-9_]+)", s, re.M))
out = [ln for ln in s.splitlines(keepends=True)
       if not (re.match(r"^pub use self::(_[A-Za-z0-9_]+) as [A-Za-z0-9_]+;", ln)
               and re.match(r"^pub use self::(_[A-Za-z0-9_]+) as", ln).group(1) not in local)]
open(final, "w").write("".join(out))
' "$RAW" "$FINAL"
  # C++ drivers (LANG_ARGS has `-x c++`): the driver's OWN functions are declared without
  # `extern "C"`, so bindgen emits them as `_Z`-mangled extern decls. In an all-Rust cohab
  # those are dead AND collide name-for-name with the translated modules under a flat
  # re-export. Strip every `_Z`-mangled extern block (a rare genuinely-mangled kernel import
  # is re-bound with C linkage in the crate foundation). See strip_cpp_mangled_externs.py.
  if printf '%s\n' "${LANG_ARGS[@]}" | grep -q 'c++'; then
    python3 "$(dirname "$0")/strip_cpp_mangled_externs.py" "$FINAL" >/dev/null
  fi
  # MACRO_SIGNED post-pass: re-force the genuinely-DWORD/IN_ADDR high-bit consts back to u32.
  # Under --default-macro-constant-type signed these flip to negative i32 (same bits) but are
  # used in u32 contexts (IN_ADDR.s_addr masks, DeviceIoControl ioctl codes, DWORD timeouts) —
  # keeping them u32 avoids re-introducing E0308 on the address/ioctl/timeout paths. Bit
  # pattern is identical; only the declared type changes. Families: INADDR_*, IN_CLASS*,
  # IOC_* (winsock ioctl direction bits), SIO_* (built from IOC_*), WSA_INFINITE.
  if [ "${MACRO_SIGNED:-0}" = 1 ]; then
    python3 - "$FINAL" <<'PYEOF'
import sys, re
p = sys.argv[1]; s = open(p).read()
# Re-declare the DWORD/IN_ADDR families as u32, wrapping any negative i32 literal back to
# its unsigned bit pattern (WSA_INFINITE/INADDR_NONE = -1 -> 4294967295, IOC_IN -> 0x80000000).
def fix(m):
    return f"pub const {m.group(1)}: u32 = {int(m.group(2)) & 0xFFFFFFFF};"
s = re.sub(r'pub const ((?:INADDR_|IN_CLASS|IOC_|SIO_)[A-Za-z0-9_]+|WSA_INFINITE): i32 = (-?\d+);',
           fix, s)
open(p, 'w').write(s)
PYEOF
  fi
  if [ "${BLOCKLIST_VARS:-0}" = 1 ]; then
    # Strip `pub static [mut] Name: T;` extern-global lines (msafd owns these in the
    # translation). Leaves the surrounding `extern "C" { ... }` block valid even if emptied.
    sed -i -E '/^[[:space:]]*pub static (mut )?[A-Za-z_][A-Za-z0-9_]*:/d' "$FINAL"
  fi
  if [ "${CALLBACK_SYSTEM:-0}" = 1 ]; then
    # Normalize fn-POINTER-TYPE calling conventions to the portable `extern "system"`.
    # bindgen spells WINAPI/stdcall callbacks (LPWSP*, upcall-table members, ...) as
    # `extern "stdcall"` on i386 and `extern "C"` on amd64/arm64. The translation defines the
    # WSP entrypoints as `extern "system"` (the portable per-arch form, same ABI), and Rust
    # does NOT unify "system" with a literal "stdcall"/"C" fn-pointer type -> E0308 on every
    # WSPPROC_TABLE assignment. Rewriting the types to "system" (ABI-identical, same as what
    # postprocess_bindgen.py does for the ntddk callback types) makes them unify on all arches.
    # BLOCKLIST_FUNCS drops all fn *declarations*, so every remaining `extern "X" fn` is a
    # fn-pointer TYPE -> a blanket rewrite is safe.
    sed -i -E 's/extern "stdcall" fn/extern "system" fn/g; s/extern "C" fn/extern "system" fn/g' "$FINAL"
  fi
  echo "  -> ${FINAL}  ($(grep -cE 'pub struct' "$FINAL") structs, $(wc -l <"$FINAL") lines)"
}

run_arch i386  i686-pc-windows-gnu    '-D_M_IX86 -D__i386__ -D_X86_'
run_arch amd64 x86_64-pc-windows-gnu  '-D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_'
run_arch arm64 aarch64-pc-windows-gnu '-D_WIN64 -D_M_ARM64 -D__aarch64__ -D__arm64__ -D_ARM64_'

# <stem>.rs dispatches to the per-arch module.
cat > "${OUT_CRATE}/${STEM}.rs" <<RS
//! Driver-private struct layouts (exact, bindgen-generated per arch).
//! Generated by tools/rustconv/gen_driver_types.sh. Do not edit by hand.
#[cfg(target_arch = "x86")]      #[path = "${STEM}_i386.rs"]  mod types_arch;
#[cfg(target_arch = "x86_64")]   #[path = "${STEM}_amd64.rs"] mod types_arch;
#[cfg(target_arch = "aarch64")]  #[path = "${STEM}_arm64.rs"] mod types_arch;
pub use types_arch::*;
RS
echo "=== wrote ${OUT_CRATE}/${STEM}.rs (per-arch dispatch) ==="
