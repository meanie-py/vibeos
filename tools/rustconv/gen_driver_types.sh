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

DRIVER="${1:?usage: gen_driver_types.sh <shell32|acpi|classpnp|cdrom|cdfs|afd|msafd|ws2_32|wshtcpip|disk|pcix|partmgr|fs_rec|pciidex|scsiport|atapi|buslogic|vioblk|usbaudio|ks|usbstor|usbccgp|usbhub|usbehci|usbohci|usbuhci|usbxhci|usbport|uniata>}"
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
# LATE_INC: -I dirs that must rank BELOW the standard SDK dirs, because they carry an
# alternate surface for headers the SDK also owns. sdk/include/wine is the case that forced
# this: its winioctl.h defines _FILE_FS_*_INFORMATION, which ndk/iotypes.h then redefines, so
# resolving <winioctl.h> to the wine copy is 23 redefinition errors — and, worse, would build
# the island against a DIFFERENT struct layout than the .c if it ever did compile. The real
# CMake command lists these last; EXTRA_INC is emitted FIRST, so they cannot go there.
LATE_INC=()
# LANG_ARGS: extra clang language flags. Default empty = C. C-in-.cpp drivers whose
# HEADERS use C++ (default args, bool, anon unions — e.g. uniata) set `-x c++` so the
# parse succeeds; bindgen still emits the plain structs.
LANG_ARGS=()
# REAL_UNIONS: rewrite bindgen's `__BindgenUnionField<T>` emulation into real Rust
# `union`s (bindgen_unions_to_real.py). Without it a C anonymous union member reads
# `p.__bindgen_anon_1.Field.as_mut()`; with it, `p.__bindgen_anon_1.Field` — which is
# what a translation of the C (where the union is FLATTENED and the member is just
# `p.Field`) actually needs.
#
# OPT-IN, and it stays opt-in: 170 island files across the driver crates carry the
# emulation today and their translations call the `__BindgenUnionField` accessor API,
# so flipping this on by default would rewrite all of them at once. New usermode
# crates want it (dnsapi, snmpapi, rasadhlp); converting an existing crate is a
# separate, per-crate change. It runs as a post-pass here rather than by hand — it was
# a remembered step, which is how rasadhlp's islands silently came out unconverted.
REAL_UNIONS=0
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
# EXTRA_RAW_LINES: extra `--raw-line` entries injected at the TOP of each generated
# <stem>_<arch>.rs, i.e. INSIDE the per-arch module. Needed whenever EXTRA_BL_ITEMS blocklists
# a type that other emitted types still REFERENCE: blocklisting removes the definition, not the
# uses, so without a re-export the island fails with E0412 on its own output. SYS_CRATES does
# this automatically for a whole sibling -sys crate; this is the hook for borrowing a handful of
# names from a hand-written module in the SAME crate (shell32 takes its COM interfaces from
# crate::shellcom so they keep the `lpVtbl` shape the translation calls through).
EXTRA_RAW_LINES=()
case "$DRIVER" in
  portcls)
    DRIVER_DIR="${SRC}/drivers/wdm/audio/backpln/portcls"
    PORTCLS_TAG=$(sed -n '/^#define TAG_PORTCLASS /p' "${DRIVER_DIR}/private.hpp")
    [[ $(printf '%s\n' "$PORTCLS_TAG" | wc -l) == 1 && -n "$PORTCLS_TAG" ]] || { echo 'ambiguous TAG_PORTCLASS' >&2; exit 1; }
    WRAPPER_INC=$'#include "resource_rust_bridge.h"\n'"${PORTCLS_TAG}"$'\nstatic const unsigned int RUSTOWNED_TAG_PORTCLASS = TAG_PORTCLASS;'
    ALLOW_FILES=( --allowlist-var RUSTOWNED_TAG_PORTCLASS
                  --allowlist-file '.*resource_rust_bridge\.h'
                  --allowlist-file '.*[/\\]portcls\.h' )
    EXTRA_INC=( -I"${DRIVER_DIR}" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000
               -DPC_IMPLEMENTATION -DCOM_STDMETHOD_CAN_THROW -DPC_NO_IMPORTS )
    ;;
  sysaudio)
    # Derived from the real sysaudio include closure; KS declarations are direct
    # behavior dependencies of pin.c as well as nested types of portcls.h.
    DRIVER_DIR="${SRC}/drivers/wdm/audio/sysaudio"
    WRAPPER_INC=$'#include "sysaudio.h"\nstatic const unsigned int SYSAUDIO_IOCTL_KS_PROPERTY = IOCTL_KS_PROPERTY;'
    ALLOW_FILES=( --allowlist-file '.*sysaudio[/\\].*\.h'
                  --allowlist-file '.*[/\\]portcls\.h'
                  --allowlist-file '.*[/\\]ks\.h'
                  --allowlist-file '.*[/\\]ksmedia\.h'
                  --allowlist-var SYSAUDIO_IOCTL_KS_PROPERTY )
    EXTRA_INC=( -I"${DRIVER_DIR}" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_COMDDK_ )
    ;;
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
    EXTRA_INC=( -I"${SRC}/vendor/acpica/include" )
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
    EXTRA_INC=( -I"${SRC}/vendor/wdf/include/kmdf/1.17" -I"${SRC}/vendor/wdf/include/shared" -I"${SRC}/vendor/wdf/include" )
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
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from drivers/storage/ide/pciidex/CMakeLists.txt
    # (25 compiled TU(s), 84 headers in the include closure, 74 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=0
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/drivers/storage/ide/pciidex"
    WRAPPER_INC=$'#include "pciidex.h"\n#include <acpiioct.h>\n#include <stdarg.h>\n#include <stddef.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]acpiioct\.h' --allowlist-file '.*[/\\]ata\.h'
                  --allowlist-file '.*[/\\]debug[/\\]driverdbg\.h'
                  --allowlist-file '.*[/\\]ide\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]ntintsafe\.h'
                  --allowlist-file '.*[/\\]ntstrsafe\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]drivers[/\\]ata[/\\]ata_shared\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]drivers[/\\]ata[/\\]ata_user\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]wdmguid\.h'
                  --allowlist-file '.*pciidex[/\\].*\.h'
                  --allowlist-file '.*[/\\]ahci\.h'
                  --allowlist-file '.*[/\\]guiddef\.h'
                  --allowlist-file '.*[/\\]ntddk\.h'
                  --allowlist-file '.*[/\\]ntdef\.h'
                  --allowlist-file '.*[/\\]pata\.h'
                  --allowlist-file '.*[/\\]pciidex\.h'
                  --allowlist-file '.*[/\\]wdm\.h' )
    EXTRA_INC=( -I"${SRC}/drivers/storage/ide/pciidex" -I"${SRC}/build-i386/drivers/storage/ide/pciidex" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_CRT_NO_POSIX_ERROR_CODES )
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
    DRIVER_DIR="${SRC}/vendor/wdf/include/kmdf/1.17"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <wdf.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]wdf[/\\].*' )
    VER_DEFS=( -DWINVER=0x603 -D_WIN32_WINNT=0x603 -DNTDDI_VERSION=0x06030000
               -DWDF_DEVICE_NO_WDMSEC_H -DKMDF_MAJOR_VERSION_STRING=01 -DKMDF_MINOR_VERSION_STRING=017 )
    EXTRA_INC=( -I"${SRC}/vendor/wdf/include/kmdf/1.17" -I"${SRC}/vendor/wdf/include/shared" -I"${SRC}/vendor/wdf/include" )
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
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-07 from drivers/storage/ide/atapi/CMakeLists.txt
    # (16 compiled TU(s), 91 headers in the include closure, 160 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=0
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/drivers/storage/ide/atapi"
    WRAPPER_INC=$'#include "atapi.h"\n#include <acpiioct.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]acpiioct\.h' --allowlist-file '.*[/\\]ata\.h'
                  --allowlist-file '.*[/\\]debug[/\\]driverdbg\.h'
                  --allowlist-file '.*[/\\]ide\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]ntdddisk\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h'
                  --allowlist-file '.*[/\\]ntddstor\.h'
                  --allowlist-file '.*[/\\]ntintsafe\.h'
                  --allowlist-file '.*[/\\]ntstrsafe\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]drivers[/\\]ata[/\\]ata_shared\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]drivers[/\\]ata[/\\]ata_user\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]drivers[/\\]ata[/\\]identify_funcs\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]section_attribs\.h'
                  --allowlist-file '.*[/\\]sptilib\.h'
                  --allowlist-file '.*[/\\]srb\.h'
                  --allowlist-file '.*[/\\]wdmguid\.h'
                  --allowlist-file '.*[/\\]wmidata\.h'
                  --allowlist-file '.*[/\\]wmilib\.h'
                  --allowlist-file '.*[/\\]wmistr\.h'
                  --allowlist-file '.*atapi[/\\].*\.h'
                  --allowlist-file '.*[/\\]atapi\.h'
                  --allowlist-file '.*[/\\]guiddef\.h'
                  --allowlist-file '.*[/\\]ntdef\.h'
                  --allowlist-file '.*[/\\]scsiex\.h'
                  --allowlist-file '.*[/\\]wdm\.h' )
    # OPAQUE in ntkrnl-sys ({ _address: u8 }) but defined by an allowlisted header and named by
    # this module's C, so the island emits the real layout (fields reachable).
    UNBLOCK="_SCSI_REQUEST_BLOCK SCSI_REQUEST_BLOCK PSCSI_REQUEST_BLOCK"
    EXTRA_INC=( -I"${SRC}/drivers/storage/ide/atapi" -I"${SRC}/build-i386/drivers/storage/ide/atapi" -I"${SRC}/sdk/lib/drivers/sptilib" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_CRT_NO_POSIX_ERROR_CODES )
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
  vioblk)
    # virtio-blk SCSIPORT miniport, written in Rust from scratch (no C body). It needs
    # only the SCSIPORT miniport surface: HW_INITIALIZATION_DATA / PORT_CONFIGURATION_
    # INFORMATION / ACCESS_RANGE (srb.h) and the SCSI command layouts (scsi.h). Same
    # SRB treatment as buslogic: _SCSI_REQUEST_BLOCK is opaque in the kernel ddk
    # bindgen, so UNBLOCK it here. NT 5.2 defaults, as buslogic.
    DRIVER_DIR="${SRC}/drivers/storage/port/vioblk"
    WRAPPER_INC=$'#include "vioblk.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*[/\\]srb\.h' )
    UNBLOCK="_SCSI_REQUEST_BLOCK"
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SETUPAPI_VER=0x502
               -DDLL_EXPORT_VERSION=0x502 )
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
                -I"${SRC}/vendor/adns/src" -I"${SRC}/vendor/adns/adns_win32" )
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
  # ---- Tier-2 network SERVICES (base/services/*) ----------------------------------------
  # All six below share the easy shape netapi32 did NOT have: ONE precomp.h per module that
  # every TU includes, so ONE bindgen island each (no TYPES_STEM split). The island is built
  # by mirroring that precomp.h's include set verbatim — the §9.108 lesson: a TU's own include
  # set is a guaranteed-legal wrapper, a master header (<lm.h>) is not.
  dnsrslvr)
    # USERMODE DNS-resolver-cache service DLL (base/services/dnsrslvr, svchost-hosted). The
    # server SIDE of the dnsrslvr.idl interface whose CLIENT half dnsapi already speaks — so
    # this flip closes that MIDL loop (dnsapi's rpc.c calls R_Resolver*; those land here).
    #
    # Type surface: windns.h (DNS_RECORDW/DNS_STATUS/DNS_CACHE_ENTRY), reactos/windns_undoc.h,
    # dnsrslvr_s.h (the generated MIDL SERVER stubs — DNSRSLVR_HANDLE + the R_Resolver*
    # prototypes this module DEFINES), plus the precomp-local RESOLVER_CACHE/_ENTRY structs.
    # BLOCKLIST_FUNCS=1: windns_undoc.h/dnsrslvr_s.h declare what the translation
    # #[no_mangle]-defines. The MIDL-generated dnsrslvr_s.c stays C and links alongside.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/dnsrslvr"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winnls.h>\n#include <winreg.h>\n#include <winsvc.h>\n#include <windns.h>\n#include <windns_undoc.h>\n#define NTOS_MODE_USER\n#include <ndk/rtlfuncs.h>\n#include <ndk/obfuncs.h>\n#include <dnsrslvr_s.h>\n#include <strsafe.h>\n#include "precomp.h"'
    ALLOW_FILES=(
                  --allowlist-file '.*[/\\]winsvc\.h' --allowlist-file '.*[/\\]winreg\.h'
--allowlist-file '.*[/\\]windns\.h' --allowlist-file '.*[/\\]windns_undoc\.h'
                  --allowlist-file '.*[/\\]dnsrslvr_s\.h' --allowlist-file '.*dnsrslvr[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/dnsrslvr" -I"${SRC}/build-i386/base/services/dnsrslvr"
                -I"${SRC}/sdk/include/reactos/idl" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -DNTOS_MODE_USER -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  wkssvc)
    # USERMODE Workstation service DLL (base/services/wkssvc). Server side of wkssvc.idl —
    # the half netapi32's NetWksta*/NetJoinDomain client stubs call, so this closes THAT loop.
    #
    # Type surface: lmcons/lmerr/lmjoin/lmserver (WKSTA_INFO_*/NETSETUP_JOIN_STATUS/
    # NET_API_STATUS), ntsecapi.h + ntmsv1_0.h (the LSA logon-package surface domain.c uses),
    # wkssvc_s.h (generated MIDL server stubs). NOTE this deliberately includes the four
    # lm*.h leaves and NOT the <lm.h> master header — mirroring precomp.h exactly is what
    # makes the island legal (§9.108). wine/debug.h is parsed but NOT allowlisted; the
    # __wine_debug_channel shim lives in the crate prelude (lsasrv/netapi32 pattern).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/wkssvc"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winerror.h>\n#include <winreg.h>\n#include <winsvc.h>\n#include <lmcons.h>\n#include <lmerr.h>\n#include <lmjoin.h>\n#include <lmserver.h>\n#include <ntsecapi.h>\n#include <ntmsv1_0.h>\n#define NTOS_MODE_USER\n#include <ndk/exfuncs.h>\n#include <ndk/obfuncs.h>\n#include <ndk/psfuncs.h>\n#include <ndk/rtlfuncs.h>\n#include <ndk/sefuncs.h>\n#include <wkssvc_s.h>\n#include <wine/debug.h>\n#include "precomp.h"'
    ALLOW_FILES=(
                  --allowlist-file '.*[/\\]winsvc\.h' --allowlist-file '.*[/\\]winreg\.h'
--allowlist-file '.*[/\\]lmcons\.h' --allowlist-file '.*[/\\]lmjoin\.h'
                  --allowlist-file '.*[/\\]lmserver\.h' --allowlist-file '.*[/\\]lmerr\.h'
                  --allowlist-file '.*[/\\]ntmsv1_0\.h' --allowlist-file '.*[/\\]ntsecapi\.h'
                  --allowlist-file '.*[/\\]wkssvc_s\.h' --allowlist-file '.*wkssvc[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/wkssvc" -I"${SRC}/build-i386/base/services/wkssvc"
                -I"${SRC}/sdk/include/reactos/idl" )
    # precomp.h is in WRAPPER_INC for its four WKSTA_*_PARMNUM #defines, but it ALSO carries
    # `extern` declarations of the four service globals. Those four are DEFINED by the Rust
    # translation itself (wkssvc.rs: VersionInfo/LsaHandle/LsaAuthenticationPackage; info.rs:
    # WkstaInfo502) — the crate is now the only translation unit, so the C-era cross-TU extern
    # is vestigial. Emitting it would put an `extern "C" { static mut X }` next to the crate's
    # own `static mut X` and collide. Blocklist the decls, keep the definitions; the prelude
    # re-exports them so every module still sees one identity. (Same shape as dhcpcsvc's
    # cur_time, field manual §9.108 — a vestigial extern that survived its C linkage model.)
    EXTRA_BL_ITEMS=( VersionInfo LsaHandle LsaAuthenticationPackage WkstaInfo502 )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -DNTOS_MODE_USER -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  browser)
    # USERMODE Computer Browser service DLL (base/services/browser, svchost-hosted). Server
    # side of browser.idl. Tiny (410 LOC): a service-control skeleton + the RPC listener.
    #
    # Type surface: reactos/svc.h (SVCHOST_GLOBAL_DATA — the svchost contract this DLL is
    # loaded under) + browser_s.h. wine/debug.h parsed-not-allowlisted (prelude shim).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/browser"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winreg.h>\n#include <winsvc.h>\n#include <svc.h>\n#include <browser_s.h>\n#include <wine/debug.h>'
    ALLOW_FILES=(
                  --allowlist-file '.*[/\\]winsvc\.h' --allowlist-file '.*[/\\]winreg\.h'
--allowlist-file '.*[/\\]svc\.h' --allowlist-file '.*[/\\]browser_s\.h'
                  --allowlist-file '.*services[/\\]browser[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/browser" -I"${SRC}/build-i386/base/services/browser"
                -I"${SRC}/sdk/include/reactos/idl" )
    # NOTE (2026-07-19): winsvc.h/winreg.h are allowlisted above so the island carries the
    # SERVICE_*/HKEY surface — island completeness is a CORRECTNESS property here, not a
    # convenience: an undefined SERVICE_CONTROL_* silently becomes a catch-all `match` binding
    # (field manual §9.112). This crate was FIRST built before that allowlist existed, so its
    # prelude.rs ALSO hand-declares that surface (verified byte-identical to the generated
    # values). If you regenerate this island, DELETE the duplicated hand-declarations from
    # prelude.rs or you will get conflicting definitions.
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  wlansvc)
    # USERMODE WLAN AutoConfig service EXE (base/services/wlansvc). Server side of wlansvc.idl
    # — the half wlanapi.dll's client stubs call. An add_executable, not a DLL: the staticlib
    # cohabs into the EXE link exactly the same way (the winlogon-rust pattern).
    #
    # Type surface: wlansvc_s.h (generated MIDL server stubs + the WLAN_* wire types) + the
    # precomp-local WLANSVCHANDLE. Uses ROS DPRINT, not wine/debug.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/wlansvc"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winsvc.h>\n#include <wlansvc_s.h>\n#define NTOS_MODE_USER\n#include <ndk/rtlfuncs.h>\n#include <ndk/obfuncs.h>\n#include "precomp.h"'
    ALLOW_FILES=(
                  --allowlist-file '.*[/\\]winsvc\.h' --allowlist-file '.*[/\\]winreg\.h'
--allowlist-file '.*[/\\]wlansvc_s\.h' --allowlist-file '.*[/\\]wlantypes\.h'
                  --allowlist-file '.*wlansvc[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/wlansvc" -I"${SRC}/build-i386/base/services/wlansvc"
                -I"${SRC}/sdk/include/reactos/idl" )
    # NOTE (2026-07-19): winsvc.h/winreg.h are allowlisted above so the island carries the
    # SERVICE_*/HKEY surface — island completeness is a CORRECTNESS property here, not a
    # convenience: an undefined SERVICE_CONTROL_* silently becomes a catch-all `match` binding
    # (field manual §9.112). This crate was FIRST built before that allowlist existed, so its
    # prelude.rs ALSO hand-declares that surface (verified byte-identical to the generated
    # values). If you regenerate this island, DELETE the duplicated hand-declarations from
    # prelude.rs or you will get conflicting definitions.
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -DNTOS_MODE_USER -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  tcpsvcs)
    # USERMODE simple-TCP-services EXE (base/services/tcpsvcs) — the RFC 862/863/867/865/864
    # quintet (echo/discard/daytime/qotd/chargen) behind one thread-per-service skeleton.
    # A pure ws2_32 CLIENT of the winsock stack we just finished flipping, which makes it a
    # useful end-to-end exercise of that stack rather than just more LOC.
    #
    # Type surface: winsock2.h (SOCKET/sockaddr_in/fd_set) + the local tcpsvcs.h (SERVICES).
    # NO NDK — this module is pure Win32. BLOCKLIST_FUNCS=1: tcpsvcs.h declares the handlers
    # the translation defines.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/tcpsvcs"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <winsock2.h>\n#include <tchar.h>\n#include <strsafe.h>\n#include "tcpsvcs.h"'
    ALLOW_FILES=(
                  --allowlist-file '.*[/\\]winsvc\.h' --allowlist-file '.*[/\\]winreg\.h'
--allowlist-file '.*[/\\]winsock2\.h' --allowlist-file '.*[/\\]ws2def\.h'
                  --allowlist-file '.*[/\\]inaddr\.h' --allowlist-file '.*tcpsvcs[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/tcpsvcs" )
    # NOTE (2026-07-19): winsvc.h/winreg.h are allowlisted above so the island carries the
    # SERVICE_*/HKEY surface — island completeness is a CORRECTNESS property here, not a
    # convenience: an undefined SERVICE_CONTROL_* silently becomes a catch-all `match` binding
    # (field manual §9.112). This crate was FIRST built before that allowlist existed, so its
    # prelude.rs ALSO hand-declares that surface (verified byte-identical to the generated
    # values). If you regenerate this island, DELETE the duplicated hand-declarations from
    # prelude.rs or you will get conflicting definitions.
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  telnetd)
    # USERMODE telnet daemon EXE (base/services/telnetd) — RFC 854 option negotiation + a
    # child-process console pumped over pipes. Another ws2_32 client.
    #
    # Type surface: winsock2.h + wincon.h (the console/pipe plumbing) + the local telnetd.h
    # (client_s / the IAC option macros) and syslog.h. ANSI module (no UNICODE define — the
    # C build doesn't set it here), so the char-vs-wchar surface must match: do NOT add
    # -DUNICODE or every TCHAR type in the island silently flips width.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/services/telnetd"
    WRAPPER_INC=$'#include <windef.h>\n#include <winbase.h>\n#include <wincon.h>\n#include <winsock2.h>\n#include <strsafe.h>\n#include "telnetd.h"\n#include "syslog.h"'
    ALLOW_FILES=(
                  --allowlist-file '.*[/\\]winsvc\.h' --allowlist-file '.*[/\\]winreg\.h'
--allowlist-file '.*[/\\]winsock2\.h' --allowlist-file '.*[/\\]ws2def\.h'
                  --allowlist-file '.*[/\\]inaddr\.h' --allowlist-file '.*telnetd[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/telnetd" )
    # NOTE (2026-07-19): winsvc.h/winreg.h are allowlisted above so the island carries the
    # SERVICE_*/HKEY surface — island completeness is a CORRECTNESS property here, not a
    # convenience: an undefined SERVICE_CONTROL_* silently becomes a catch-all `match` binding
    # (field manual §9.112). This crate was FIRST built before that allowlist existed, so its
    # prelude.rs ALSO hand-declares that surface (verified byte-identical to the generated
    # values). If you regenerate this island, DELETE the duplicated hand-declarations from
    # prelude.rs or you will get conflicting definitions.
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_CRT_SECURE_NO_WARNINGS -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  wsock32)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/wsock32/CMakeLists.txt
    # (1 compiled TU, 54 headers in the include closure, 0 switch constants).
    #
    # USERMODE WinSock 1.1 compatibility DLL (dll/win32/wsock32). Almost the whole module is
    # `.spec` forwarders to mswsock/ws2_32 that spec2def turns into thunks; the only real C is
    # a DllMain that returns TRUE. The island is correspondingly tiny — windef.h only.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/wsock32"
    WRAPPER_INC=$'#include <windef.h>'
    ALLOW_FILES=( --allowlist-file '.*wsock32[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/wsock32" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  wlanapi)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/wlanapi/CMakeLists.txt
    # (2 compiled TUs, 138 headers in the include closure, 5 switch constants).
    #
    # USERMODE WLAN client API DLL (dll/win32/wlanapi) — the client half of wlansvc.idl, whose
    # server half is the `wlansvc)` profile above. RPC client stubs come from the generated
    # wlansvc_c.h, which is why rpc.h/rpcndr.h/ole2.h are allowlisted: main.c names them
    # directly through that generated header, so their types must be island-local.
    #
    # The 5 switch constants are all RPC_* from winerror.h and resolve through the -sys glob.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/wlanapi"
    WRAPPER_INC=$'#define WIN32_NO_STATUS\n#define _INC_WINDOWS\n#define COM_NO_WINDOWS_H\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <wlansvc_c.h>\n#include <wine/debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ole2\.h' --allowlist-file '.*[/\\]rpc\.h'
                  --allowlist-file '.*[/\\]rpcndr\.h'
                  --allowlist-file '.*[/\\]wlansvc_c\.h'
                  --allowlist-file '.*wlanapi[/\\].*\.h' )
    # build-i386 for the generated wlansvc_c.h: ARCH is a local of the per-arch generate
    # function and is unset here in the case table, and the MIDL wire types are
    # arch-independent, so i386's copy serves all 3 (same convention as netapi32/dnsapi).
    EXTRA_INC=( -I"${SRC}/dll/win32/wlanapi" -I"${SRC}/build-i386/dll/win32/wlanapi"
                -I"${SRC}/sdk/include/reactos/idl" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  snmpapi)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/snmpapi/CMakeLists.txt
    # (1 compiled TU, 138 headers in the include closure, 14 switch constants).
    #
    # USERMODE SNMP utility API DLL (dll/win32/snmpapi) — SnmpUtil* octet-string/OID/varbind
    # helpers. Note the CMakeLists RAISES _WIN32_WINNT to 0x600 (it removes the 0x502 default),
    # so VER_DEFS must follow or the island is built against a different SDK surface than the C.
    #
    # 13 of the 14 switch constants are ASN_* from snmp.h, and every one is a macro EXPRESSION
    # (`(ASN_UNIVERSAL | ASN_PRIMITIVE | 0x03)`), not a literal — bindgen does not always fold
    # those. snmp.h is allowlisted so they land in the island, and the generated types_<arch>.rs
    # must be CHECKED for them: an ASN_* that fails to appear becomes a catch-all `match`
    # binding that swallows every other tag (field manual §9.112).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/snmpapi"
    WRAPPER_INC=$'#include <stdio.h>\n#include <stdarg.h>\n#include "windef.h"\n#include "winbase.h"\n#include "snmp.h"\n#include "wine/debug.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]snmp\.h' --allowlist-file '.*snmpapi[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/snmpapi" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000
               -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  mprapi)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/mprapi/CMakeLists.txt
    # (1 compiled TU, 72 headers in the include closure, 1 switch constant).
    #
    # USERMODE RAS/router administration API (dll/win32/mprapi). Almost entirely stubs; the one
    # switch constant is DLL_PROCESS_ATTACH, a winnt.h literal already reachable via the -sys glob.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/mprapi"
    WRAPPER_INC=$'#include <stdarg.h>\n#include "windef.h"\n#include "winbase.h"\n#include "mprapi.h"\n#include "wine/debug.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]mprapi\.h' --allowlist-file '.*mprapi[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/mprapi" -I"${SRC}/build-i386/dll/win32/mprapi" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE
               -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  netid)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/netid/CMakeLists.txt
    # (1 compiled TU, 94 headers in the include closure, 16 switch constants).
    #
    # USERMODE "Computer Name" system-properties page (dll/win32/netid) — a PROPSHEETPAGE dialog,
    # so most switch constants are prsht.h notification codes. PSN_APPLY and its siblings are
    # macro EXPRESSIONS (`(PSN_FIRST - 2)`), not literals, and bindgen does not always fold those:
    # CHECK types_<arch>.rs for every PSN_*, or a missing one becomes a catch-all `match` binding
    # that swallows the other notifications (field manual §9.112). lmjoin.h carries the
    # NetJoinDomain/NetGetJoinInformation surface the page drives.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/netid"
    WRAPPER_INC=$'#define WIN32_NO_STATUS\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <winuser.h>\n#include <winreg.h>\n#include <windns.h>\n#include <lm.h>\n#include <prsht.h>\n#include "resource.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]lm\.h' --allowlist-file '.*[/\\]lmjoin\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]windns\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*netid[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/netid" -I"${SRC}/build-i386/dll/win32/netid" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -DUNICODE -D_UNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1
               -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP
               -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  samlib)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/samlib/CMakeLists.txt
    # (2 compiled TUs, 172 headers in the include closure, 6 switch constants).
    #
    # USERMODE SAM client library (dll/win32/samlib) — the CLIENT half of sam.idl, whose server
    # half lives in samsrv. sam_c.h is the generated MIDL client stub header and must be in the
    # island (EXTRA_INC reaches sdk/include/reactos/idl for it); the generated sam_c.c stays C
    # and supplies the _Sam* stubs the Rust imports, exactly as wlansvc_c.c does for wlanapi.
    # NTOS_MODE_USER is required BEFORE <ndk/rtlfuncs.h> or the NDK exposes the kernel surface.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/samlib"
    WRAPPER_INC=$'#include "precomp.h"\n#define NTOS_MODE_USER\n#include <ndk/rtlfuncs.h>\n#include <ntsam.h>\n#include <sam_c.h>\n#include <wine/debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ntsam\.h' --allowlist-file '.*[/\\]ole2\.h'
                  --allowlist-file '.*[/\\]rpc\.h'
                  --allowlist-file '.*[/\\]rpcndr\.h'
                  --allowlist-file '.*[/\\]sam_c\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*samlib[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/samlib" -I"${SRC}/build-i386/dll/win32/samlib" )
    # BELOW the standard SDK dirs, exactly where the real CMake command puts them. Hoisting
    # sdk/include/wine ahead of them resolves <winioctl.h> to the wine copy, whose
    # _FILE_FS_*_INFORMATION then collide with ndk/iotypes.h — 23 errors, and a silently
    # different struct layout than the .c if it had compiled.
    LATE_INC=( -I"${SRC}/sdk/include/reactos/idl" -I"${SRC}/sdk/include/wine" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -D__WINESRC__ -DUNICODE -D_UNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1
               -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP
               -D_CRT_NO_POSIX_ERROR_CODES -DNTOS_MODE_USER )
    ;;
  rasapi32)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-26 from dll/win32/rasapi32/CMakeLists.txt
    # (1 compiled TU, 69 headers in the include closure, 0 switch constants).
    #
    # USERMODE RAS (dial-up) client API (dll/win32/rasapi32). Wine-sourced; almost entirely
    # ERROR_NOT_SUPPORTED stubs over the ras.h surface.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/rasapi32"
    WRAPPER_INC=$'#include <stdarg.h>\n#include "windef.h"\n#include "winbase.h"\n#include "ras.h"\n#include "raserror.h"\n#include "wine/debug.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ras\.h' --allowlist-file '.*[/\\]raserror\.h'
                  --allowlist-file '.*rasapi32[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/rasapi32" -I"${SRC}/build-i386/dll/win32/rasapi32" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE
               -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  rasman)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-26 from dll/win32/rasman/CMakeLists.txt
    # (1 compiled TU, 73 headers in the include closure, 1 switch constant).
    #
    # USERMODE RAS connection manager (dll/win32/rasman). The one switch constant is
    # DLL_PROCESS_ATTACH, a winnt.h literal already reachable via the -sys glob.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/rasman"
    WRAPPER_INC=$'#define WIN32_NO_STATUS\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <wine/debug.h>\n#include <rasshost.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]rasshost\.h' --allowlist-file '.*rasman[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/rasman" -I"${SRC}/build-i386/dll/win32/rasman" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE
               -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  rasdlg)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-26 from dll/win32/rasdlg/CMakeLists.txt
    # (1 compiled TU, 69 headers in the include closure, 1 switch constant).
    #
    # USERMODE RAS dialog UI (dll/win32/rasdlg). Same DLL_PROCESS_ATTACH note as rasman.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/rasdlg"
    WRAPPER_INC=$'#define WIN32_NO_STATUS\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <wine/debug.h>\n#include <rasdlg.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]rasdlg\.h' --allowlist-file '.*rasdlg[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/rasdlg" -I"${SRC}/build-i386/dll/win32/rasdlg" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE
               -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  rasadhlp)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-26 from dll/win32/rasadhlp/CMakeLists.txt
    # (3 compiled TUs, 168 headers in the include closure, 2 switch constants).
    #
    # USERMODE RAS AutoDial helper (dll/win32/rasadhlp) — the winsock NSP hook that triggers a
    # dial-up attempt on a failed name resolution, so it rides on the ws2_32 surface we already
    # translated. LATE_INC was DERIVED, not hand-added: the CMakeLists' include_directories()
    # puts sdk/include/reactos/winsock BELOW the standard SDK dirs, and hoisting it changes
    # which winsock2.h an include resolves to (the samlib collision class).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/rasadhlp"
    WRAPPER_INC=$'#include "precomp.h"\n#include <ndk/iofuncs.h>\n#include <ndk/rtltypes.h>\n#include <winnls.h>\n#include <nsp_dns.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]nsp_dns\.h' --allowlist-file '.*[/\\]wininet\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winsock2\.h'
                  --allowlist-file '.*[/\\]ws2def\.h'
                  --allowlist-file '.*[/\\]wsnetbs\.h'
                  --allowlist-file '.*rasadhlp[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/rasadhlp" -I"${SRC}/build-i386/dll/win32/rasadhlp" )
    LATE_INC=( -I"${SRC}/sdk/include/reactos/winsock" )
    # AUTODIAL_ADDR / AUTODIAL_CONN are tagged unions whose payload is an ANONYMOUS
    # union, so every access in the C is a flattened `Addr.Ip4Address`.
    REAL_UNIONS=1
    # nsp_dns.h DEFINES its six service GUIDs in the header, `const GUID
    # DECLSPEC_SELECTANY X = SVCID_...;` — COMDAT, so every TU that includes it emits a
    # foldable copy. bindgen renders them as `extern static`, but winsock.c WAS the only
    # definer and it is now Rust, so the externs resolve to nothing (undefined
    # _HostnameGuid/_AddressGuid at link). Strip them; the crate defines them instead.
    BLOCKLIST_VARS=1
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502
               -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE
               -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP -D_CRT_NO_POSIX_ERROR_CODES
               -DNTOS_MODE_USER )
    ;;
  wldap32|wldap32-types_1)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-26 from dll/win32/wldap32/CMakeLists.txt
    # (20 compiled TU(s), 97 headers in the include closure, 64 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    # TWO ISLANDS, derived not guessed: ber.c includes <winldap.h> while the other 19 TUs
    # include "winldap_private.h", and the two define LDAPControlA/LDAPModA/LDAPSortKeyA/
    # LDAPAPIInfoA (+W) INCOMPATIBLY. No single TU includes both, so there is no legal merged
    # island — the netapi32 lesson (§9.108): a multi-header redefinition wall is a claim about
    # YOUR WRAPPER, not the headers. Generate both, never glob both into one scope:
    #   gen_driver_types.sh wldap32
    #   TYPES_STEM=types_1 gen_driver_types.sh wldap32-types_1
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/wldap32"
    case "$DRIVER" in
      # island types — 19 TU(s):
      # add.c bind.c compare.c control.c delete.c dn.c error.c extended.c init.c main.c
      # misc.c modify.c modrdn.c option.c page.c parse.c rename.c search.c value.c
      wldap32)
        # NB no <ldap.h>: it is guarded by `#ifdef HAVE_LDAP_H`, which wine's config.h
        # `#undef`s for this tree — ReactOS does not vendor OpenLDAP. profile_derive now
        # reads config.h and drops decidably-dead prologue arms.
        WRAPPER_INC=$'#include "precomp.h"\n#include "config.h"\n#include "wine/port.h"\n#include <stdarg.h>\n#include "windef.h"\n#include "winbase.h"\n#include "winnls.h"\n#include "winldap_private.h"\n#include "wldap32.h"\n#include "wine/debug.h"\n#include "winuser.h"\n#include <stdio.h>\n#include "wine/winternl.h"\n#define LDAP_MAXINT  2147483647'
        ALLOW_FILES=( --allowlist-file '.*[/\\]config\.h' --allowlist-file '.*[/\\]winnls\.h'
                      --allowlist-file '.*wldap32[/\\].*\.h' )
        ;;
      # island types_1 — 1 TU(s):
      # ber.c
      wldap32-types_1)
        # NO precomp.h: ber.c is wldap32's PCH_SKIP_SOURCE. precomp.h pulls
        # "winldap_private.h" and ber.c includes <winldap.h>; the two declare ~30 ldap_*
        # prototypes incompatibly, which is WHY the CMakeLists skips it — and why this is
        # a separate island.
        WRAPPER_INC=$'#include "config.h"\n#include <stdarg.h>\n#include "windef.h"\n#include "winbase.h"\n#include "winldap.h"\n#include "wine/debug.h"'
        ALLOW_FILES=( --allowlist-file '.*[/\\]config\.h' --allowlist-file '.*[/\\]winldap\.h'
                      --allowlist-file '.*wldap32[/\\].*\.h' )
        ;;
    esac
    # sdk/include/wine EARLY — this is a wine-sourced module and the real command ranks it
    # ABOVE the SDK dirs (the opposite of samlib). It also owns the only config.h, so
    # without it bindgen cannot even open the wrapper.
    EXTRA_INC=( -I"${SRC}/dll/win32/wldap32" -I"${SRC}/build-i386/dll/win32/wldap32"
                -I"${SRC}/sdk/include/wine" )
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D__USE_CRTIMP -D__WINESRC__ -D_CRT_NO_POSIX_ERROR_CODES )
    OUT_CRATE_DIR="${OUT_CRATE_DIR:-${SRC}/sdk/lib/rust/wldap32-rust/src}"
    ;;

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh wldap32
  #   TYPES_STEM=types_1 gen_driver_types.sh wldap32-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).
  netcfgx)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-26 from dll/win32/netcfgx/CMakeLists.txt
    # (11 compiled TU(s), 169 headers in the include closure, 32 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    # Network configuration DLL (dll/win32/netcfgx) — the TCP/IP property sheets and the
    # netcfg component model. Derived clean: single legal include set, and all 32 switch
    # constants resolve through the island (no §9.112 exposure at derive time).
    USERMODE=1
    BLOCKLIST_FUNCS=1
    # MACRO_SIGNED: netcfgx is a dialog module, so nearly every constant it touches is an `int`
    # in the C — control ids (IDC_*/IDD_*), window messages (WM_*/CB_*/LB_*/LVM_*), and the
    # LSTATUS (= LONG) values every Reg*/Setup* call returns. Leaving bindgen's u32 default in
    # place cost 715 E0308s on `rc != ERROR_SUCCESS` and `GetDlgItem(hDlg, IDC_*)` alone.
    MACRO_SIGNED=1
    # UNBLOCK DLGPROC so the island emits its OWN, which normalize_island_abi.py then spells
    # `extern "system"` on all three arches. ntdll-sys's copy is `extern "stdcall"` on i386 and
    # `extern "C"` on the 64-bit targets, and Rust compares fn-pointer ABIs NOMINALLY — neither
    # spelling unifies with "system", so a module full of dialog procs would otherwise have to
    # carry every DLGPROC body twice under complementary `#[cfg(target_arch)]` arms.
    UNBLOCK="DLGPROC"
    DRIVER_DIR="${SRC}/dll/win32/netcfgx"
    WRAPPER_INC=$'#include "precomp.h"\n#include <olectl.h>\n#include <strsafe.h>\n#include <rpc.h>\n#include <winnls.h>\n#include <winsock.h>\n#include <iphlpapi.h>\n#include <dhcpcsdk.h>\n#include <dhcpcapi.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cfgmgr32\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]devguid\.h'
                  --allowlist-file '.*[/\\]dhcpcapi\.h'
                  --allowlist-file '.*[/\\]dhcpcsdk\.h'
                  --allowlist-file '.*[/\\]iphlpapi\.h'
                  --allowlist-file '.*[/\\]netcfgn\.h'
                  --allowlist-file '.*[/\\]netcfgx\.h'
                  --allowlist-file '.*[/\\]netcfgx_undoc\.h'
                  --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*[/\\]olectl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]rpc\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*[/\\]winsock\.h'
                  --allowlist-file '.*[/\\]winsvc\.h'
                  --allowlist-file '.*netcfgx[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/netcfgx" -I"${SRC}/build-i386/dll/win32/netcfgx" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  # ---- Tier-3 batch D: the security/auth cluster (ntmarta, authz, msv1_0, aclui). ----
  # All four are plain C with no COM substrate, so they take the ordinary
  # profile_derive -> seal -> crate-repair loop unchanged.
  ntmarta)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-04 from dll/win32/ntmarta/CMakeLists.txt
    # (1 compiled TU(s), 174 headers in the include closure, 30 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/ntmarta"
    WRAPPER_INC=$'#include "ntmarta.h"\n#define NDEBUG\n#include <debug.h>'
    # winsvc.h added by hand: ntmarta.c reaches the service database through
    # OpenSCManager/QueryServiceObjectSecurity, so SC_HANDLE and SC_MANAGER_CONNECT are part
    # of its real surface. The derive missed it because neither name is a `switch` label —
    # SWITCH_CONST_HEADER_ADDED only pulls in a header that owns a case constant.
    ALLOW_FILES=( --allowlist-file '.*[/\\]accctrl\.h' --allowlist-file '.*[/\\]aclapi\.h'
                  --allowlist-file '.*[/\\]ntsecapi\.h'
                  --allowlist-file '.*[/\\]winsvc\.h'
                  --allowlist-file '.*ntmarta[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/ntmarta" -I"${SRC}/build-i386/dll/win32/ntmarta" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  authz)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-04 from dll/win32/authz/CMakeLists.txt
    # (3 compiled TU(s), 67 headers in the include closure, 11 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/authz"
    WRAPPER_INC=$'#include "precomp.h"\n#define NDEBUG\n#include <debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]authz\.h' --allowlist-file '.*authz[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/authz" -I"${SRC}/build-i386/dll/win32/authz" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  msv1_0)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-04 from dll/win32/msv1_0/CMakeLists.txt
    # (7 compiled TU(s), 193 headers in the include closure, 14 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    # Emit the header function surface, minus the functions the crate defines itself. msv1_0 is
    # a THIN module over other people's APIs: it calls the SAM/LSA server entrypoints
    # (SamIConnect, SamrOpenUser, LsarClose, LsaIOpenPolicyTrusted, ...) through the samsrv and
    # lsasrv DELAY imports, plus advapi32's SystemFunction006/007/012 and the wide CRT. With
    # BLOCKLIST_FUNCS=1 those are ~60 names the prelude would have to declare BY HAND, and a
    # hand-written declaration of an RPC-shaped SAM entrypoint that is wrong by one pointer
    # level links fine and corrupts at run time. Let bindgen spell them from ntsam.h/ntlsa.h.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    DRIVER_DIR="${SRC}/dll/win32/msv1_0"
    # The derive unions the prologues of all 7 TUs, and the two under ntlm/ spell the SAME
    # header relatively: `../precomp.h` from dll/win32/msv1_0/ntlm IS dll/win32/msv1_0/precomp.h.
    # Kept once — a wrapper in /tmp cannot resolve the relative form, and both TU groups compile
    # against one header, so one island is correct (§9.108 is about GENUINELY different surfaces).
    WRAPPER_INC=$'#include "precomp.h"\n#include "wine/debug.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]intsafe\.h' --allowlist-file '.*[/\\]ntlsa\.h'
                  --allowlist-file '.*[/\\]ntmsv1_0\.h'
                  --allowlist-file '.*[/\\]ntsam\.h'
                  --allowlist-file '.*[/\\]ntsecapi\.h'
                  --allowlist-file '.*[/\\]ntsecpkg\.h'
                  --allowlist-file '.*[/\\]samsrv[/\\]samsrv\.h'
                  --allowlist-file '.*[/\\]sspi\.h'
                  # winbase.h for the four kernel32 calls the module makes directly
                  # (GetProcessHeap/HeapAlloc/HeapFree/GetComputerNameW). Allowlisting the
                  # header rather than transcribing four prototypes: GetComputerNameW's
                  # second parameter is an in/out LPDWORD, and a hand-written LPDWORD-for-DWORD
                  # is the exact bug authz shipped four times in one batch.
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*msv1_0[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/msv1_0" -I"${SRC}/build-i386/dll/win32/msv1_0" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_NON_CONFORMING_SWPRINTFS -D_UNICODE -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  aclui)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-04 from dll/win32/aclui/CMakeLists.txt
    # (5 compiled TU(s), 179 headers in the include closure, 76 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    # Emit the header function surface, minus what the crate defines. aclui is a DIALOG module:
    # nearly every line calls user32/comctl32/advapi32 (GetDlgItem, DeferWindowPos, SendMessage,
    # EqualSid, ...), which with BLOCKLIST_FUNCS=1 is ~200 prototypes to hand-write.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # MACRO_SIGNED: same reason as netcfgx. Every control id (IDC_*/IDD_*), window message
    # (WM_*/LVM_*/SB_*) and LSTATUS the module touches is an `int` in the C, so bindgen's u32
    # default turns each use into an E0308.
    MACRO_SIGNED=1
    # UNBLOCK: same reason as netcfgx above — ntdll-sys spells these with the arch's literal
    # convention and normalize_island_abi.py only rewrites a crate's OWN islands, so a slot that
    # resolves through the -sys glob stays `stdcall`/`C` while the callback is `extern "system"`.
    # aclui needs all three: SecurityPageProc (DLGPROC), CheckListWndProc (WNDPROC) and
    # LookupThreadProc (LPTHREAD_START_ROUTINE, via CreateThread).
    # PTHREAD_START_ROUTINE because LPTHREAD_START_ROUTINE is only an ALIAS of it — unblocking
    # the alias alone leaves it pointing back at the -sys definition. _WNDCLASSW because the
    # WNDPROC slot that matters is a FIELD of it (bindgen names the struct _WNDCLASSW), and a
    # blocklisted struct keeps the -sys field type no matter how the pointee is spelled here.
    UNBLOCK="DLGPROC WNDPROC LPTHREAD_START_ROUTINE PTHREAD_START_ROUTINE _WNDCLASSW WNDCLASSW"
    DRIVER_DIR="${SRC}/dll/win32/aclui"
    WRAPPER_INC=$'#include "precomp.h"\n#include <sddl.h>\n#define NDEBUG\n#include <debug.h>\n#include "resource.h"\n#include <uxtheme.h>\n#include <tmschema.h>\n#include <ntsecapi.h>\n#include <stdarg.h>\n#define WIN32_NO_STATUS\n#define _INC_WINDOWS\n#define COM_NO_WINDOWS_H\n#include <windef.h>\n#include <winbase.h>\n#include <objbase.h>\n#include <initguid.h>\n#include <aclui.h>\n#include <objsel.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]aclui\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  # accctrl.h owns the SI_* page/access flags aclui.h's own structs are built
                  # from; wtypes.h and objidl.h (WIDL-GENERATED, so they live under build-<arch>)
                  # own the CLSCTX_/DVASPECT_/TYMED_/VT_ enumerators the object-picker data
                  # transfer uses. Every one of these is an ordinary use rather than a `case`
                  # label, which is why the derive left them out.
                  --allowlist-file '.*[/\\]accctrl\.h'
                  --allowlist-file '.*[/\\]wtypes\.h'
                  --allowlist-file '.*[/\\]objidl\.h'
                  --allowlist-file '.*[/\\]oaidl\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]ntsecapi\.h'
                  --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*[/\\]objsel\.h'
                  --allowlist-file '.*[/\\]ole2\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]sddl\.h'
                  --allowlist-file '.*[/\\]tmschema\.h'
                  --allowlist-file '.*[/\\]uxtheme\.h'
                  # The three headers that own the module's actual API surface. The derive does
                  # not add them because SWITCH_CONST_HEADER_ADDED only pulls in a header that
                  # owns a `case` label, and aclui reaches user32/kernel32/advapi32 through
                  # ordinary calls.
                  --allowlist-file '.*[/\\]winuser\.h'
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*[/\\]winsvc\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*aclui[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/aclui" -I"${SRC}/build-i386/dll/win32/aclui" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DSUPPORT_UXTHEME -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # ---- Tier-3 batch E: umpnpmgr, the user-mode PnP manager service. Plain C, no COM;
  # the SERVER half of pnp.idl, which cfgmgr32/setupapi speak as clients. ----
  umpnpmgr)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-04 from base/services/umpnpmgr/CMakeLists.txt
    # (4 compiled TU(s), 180 headers in the include closure, 64 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    # Emit the header function surface, minus what the crate defines. umpnpmgr is a thin
    # service over other people's APIs — cfgmgr32/setupapi/advapi32/userenv/rpcrt4 — so
    # BLOCKLIST_FUNCS=1 would mean hand-writing ~200 prototypes, and a hand-written
    # prototype wrong by one pointer level links and then corrupts silently.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # MACRO_SIGNED stays OFF here, and that was MEASURED, not reasoned. netcfgx and aclui both
    # want it on because they are dialog modules whose constants are `int` control ids and window
    # messages. umpnpmgr looks similar but is not: it spends its time passing DBT_/WM_/CM_/SERVICE_
    # constants into UNSIGNED Win32 parameters (SendMessageW's WPARAM, RegSetValueExW's dwType),
    # so signing them cost 90 extra E0308s. Whole-crate check: 165 errors signed, 76 unsigned.
    MACRO_SIGNED=0
    # UNBLOCK so THIS island owns the thread-start callback type: ntdll-sys spells it with
    # the arch's literal convention and normalize_island_abi only rewrites a crate's OWN
    # islands, so the CreateThread slot would stay `stdcall` while the three thread bodies
    # are `extern "system"`. LPTHREAD_START_ROUTINE is only an ALIAS, so its target has to
    # be unblocked too or the alias just points back at the -sys definition.
    UNBLOCK="LPTHREAD_START_ROUTINE PTHREAD_START_ROUTINE"
    DRIVER_DIR="${SRC}/base/services/umpnpmgr"
    WRAPPER_INC=$'#include "precomp.h"\n#define NDEBUG\n#include <debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cfgmgr32\.h' --allowlist-file '.*[/\\]cmfuncs\.h'
                  --allowlist-file '.*[/\\]dbt\.h'
                  --allowlist-file '.*[/\\]ole2\.h'
                  --allowlist-file '.*[/\\]pnp_s\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]rpc\.h'
                  --allowlist-file '.*[/\\]rpcndr\.h'
                  --allowlist-file '.*[/\\]rtlfuncs\.h'
                  --allowlist-file '.*[/\\]setypes\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]umpnpmgr[/\\]sysguid\.h'
                  --allowlist-file '.*[/\\]userenv\.h'
                  --allowlist-file '.*[/\\]wdmguid\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*[/\\]winsvc\.h'
                  --allowlist-file '.*[/\\]winsvc_undoc\.h'
                  # The two headers that own the module's actual Win32 surface. The derive does
                  # not add them because SWITCH_CONST_HEADER_ADDED only pulls in a header that
                  # owns a `case` label, and umpnpmgr reaches kernel32/user32 through ordinary
                  # calls — lstrlenW/wsprintf/HWND_BROADCAST/PROCESS_ALL_ACCESS and ~30 more.
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*[/\\]winuser\.h'
                  # rpcdce.h owns the RPC SERVER surface this module IS — RpcServerListen,
                  # RpcServerRegisterIf, RpcServerUseProtseqEpW, RpcImpersonateClient, UuidEqual.
                  # rpc.h alone (which the derive picked) is only the umbrella header.
                  --allowlist-file '.*[/\\]rpcdce\.h'
                  --allowlist-file '.*umpnpmgr[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/base/services/umpnpmgr" -I"${SRC}/build-i386/base/services/umpnpmgr" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/reactos/idl" -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # ---- Tier-3 batch G: desk.cpl, the Display Properties control panel. Plain C: the
  # background/appearance/screensaver/settings/theme property pages, the owner-drawn monitor
  # selection control, and the display-device settings plumbing over setupapi. ----
  desk)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-05 from dll/cpl/desk/CMakeLists.txt
    # (20 compiled TU(s), 203 headers in the include closure, 166 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). No ERROR findings and no MOSTLY_DEAD_BODIES — desk
    # is real code, unlike wldap32's stub surface.
    USERMODE=1
    # Emit the header function surface, minus what the crate defines. desk is a thin driver of
    # other people's APIs — user32/gdi32/comctl32/shell32/setupapi/uxtheme/gdiplus — so
    # BLOCKLIST_FUNCS=1 would mean hand-writing several hundred prototypes, and a hand-written
    # prototype wrong by one pointer level links and then corrupts silently. Same call as
    # syssetup and umpnpmgr; the derive defaults to 1 and does not know the module's shape.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # MACRO_SIGNED is set by MEASURING — generate both islands and count one cargo check. Two
    # modules running (umpnpmgr, syssetup) the resemblance argument lost, both times to the same
    # mistake: a property-sheet module LOOKS like it wants signed `int` control ids, and spends
    # more of its time handing constants to UNSIGNED Win32 parameters.
    # desk is the third: MEASURED 2026-08-05 on the same tree, i386, one cargo check each.
    #   MACRO_SIGNED=0 -> 1,517 errors (1,276 E0308,  68 E0277)
    #   MACRO_SIGNED=1 -> 1,820 errors (1,508 E0308, 113 E0277)
    # 0 wins by 303. Do not "fix" this by resemblance.
    MACRO_SIGNED=0
    # UNBLOCK so THIS island owns every callback type desk INSTALLS: ntdll-sys spells each with
    # the arch's literal convention, and normalize_island_abi only rewrites a crate's OWN
    # islands, so an -sys-owned slot would stay `stdcall` while the Rust body is
    # `extern "system"` (Rust compares fn-pointer ABIs NOMINALLY). The containers have to come
    # too — a struct whose field type still resolves to the -sys typedef reintroduces the
    # mismatch at the assignment.
    #
    # The TCHAR FAMILY is unblocked for a different and sharper reason: ntdll-sys is built
    # WITHOUT UNICODE, so its glob says `TCHAR = c_char`, and desk.h's own structs are declared
    # `TCHAR szPath[MAX_PATH]`. Inheriting the glob's TCHAR makes _ICON_PATH 260 bytes where the
    # real one is 520 — a layout bug that compiles clean, and the kind that shows up much later
    # as a truncated path. Unblocked, THIS island emits them from its own -DUNICODE compile.
    UNBLOCK="DLGPROC WNDPROC LPFNPSPCALLBACKW LPFNPSPCALLBACKA PFNPROPSHEETCALLBACK
             LPTHREAD_START_ROUTINE PTHREAD_START_ROUTINE
             _PROPSHEETPAGEW _PROPSHEETHEADERW PROPSHEETPAGEW PROPSHEETHEADERW
             TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    # PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union and the C FLATTENS them
    # (`psp.pszTemplate = ..`), which the `__BindgenUnionField<T>` emulation cannot accept.
    REAL_UNIONS=1
    # wtypes.h's `uCLSSPEC` tagged union, which came in with the COM headers above and which
    # desk never touches. bindgen emits its two anonymous member STRUCTS without `Copy`, and a
    # non-Copy field cannot live in a real Rust union — so with REAL_UNIONS=1 the island itself
    # stops compiling (E0740 + E0204) over a type nothing in the module names.
    # The pattern has to reach the NESTED name too (`..._00000009___WIDL_..._00000008`),
    # hence the trailing `.*`. It stays anchored at `__WIDL`, so the `_userCLIPFORMAT___WIDL_`
    # family — which IS used, as STGMEDIUM's wire type — is untouched.
    EXTRA_BL_ITEMS=( '__WIDL_wtypes_generated_name_0000000[89].*' uCLSSPEC )
    DRIVER_DIR="${SRC}/dll/cpl/desk"
    WRAPPER_INC=$'#include "desk.h"\n#include <shellapi.h>\n#include <shlwapi.h>\n#include <windowsx.h>\n#include <debug.h>\n#include <cplext.h>\n#include <winnls.h>\n#define NDEBUG\n#include <cfgmgr32.h>\n#include <uxtheme.h>\n#include <vsstyle.h>\n#include <vssym32.h>\n#include <uxundoc.h>\n#define WIN32_NO_STATUS\n#define _INC_WINDOWS\n#define COM_NO_WINDOWS_H\n#include <windef.h>\n#include <winbase.h>\n#include <wingdi.h>\n#include <objbase.h>\n#include <initguid.h>\n#include <gdiplus.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cfgmgr32\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]commdlg\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]cplext\.h'
                  --allowlist-file '.*[/\\]dll[/\\]desk[/\\]deskcplx\.h'
                  # The gdiplus FAMILY, not just the umbrella header. gdiplus.h's C branch
                  # is nothing but #includes (gdiplusflat.h for the flat API, gdiplusgpstubs
                  # /gdiplustypes/gdiplusenums/gdipluspixelformats/gdiplusimaging for GpImage,
                  # GpStatus, ImageCodecInfo, PixelFormat*, ImageLockMode*), and
                  # --allowlist-file matches where an item is DEFINED. Allowlisting only
                  # `gdiplus\.h` therefore emitted ZERO of them: 19 Gdip* functions and 8
                  # types that dibitmap.c and preview.c call directly.
                  --allowlist-file '.*[/\\]gdiplus[a-z]*\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]objbase\.h'
                  # The widl-GENERATED COM headers, which own three enum families desk uses
                  # by bare member name: DVASPECT_* (wtypes.h), TYMED_* and DATADIR_*
                  # (objidl.h). objbase.h merely #includes them, and --allowlist-file matches
                  # the DEFINITION site, so the island carried `IDataObject` (a struct, pulled
                  # in as a dependency of an allowlisted signature) while every constant those
                  # same call sites pass stayed absent. Hand-typing an enum value is the
                  # §9.112 shape; let bindgen read the header.
                  --allowlist-file '.*[/\\]objidl\.h'
                  --allowlist-file '.*[/\\]unknwn\.h'
                  --allowlist-file '.*[/\\]wtypes\.h'
                  --allowlist-file '.*[/\\]wtypesbase\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]uxtheme\.h'
                  --allowlist-file '.*[/\\]uxundoc\.h'
                  --allowlist-file '.*[/\\]vsstyle\.h'
                  --allowlist-file '.*[/\\]vssym32\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  # The two headers that own the module's actual Win32 surface — desk reaches
                  # user32/kernel32 through ordinary calls, and the derive only adds a header
                  # when it owns a `case` label.
                  --allowlist-file '.*[/\\]winuser\.h'
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*desk[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/desk" -I"${SRC}/build-i386/dll/cpl/desk" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_WIN32 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # ---- Tier-3 batch I: intl.cpl, the Regional and Language Options control panel.
  intl)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-07 from dll/cpl/intl/CMakeLists.txt
    # (12 compiled TU(s), 189 headers in the include closure, 50 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    # Same call as desk/syssetup/umpnpmgr/sysdm, and for the same reason: intl is a thin driver
    # of other people's APIs (user32/comctl32/advapi32/setupapi/shlwapi/shell32 — the locale,
    # geo, calendar and .inf families on top), so BLOCKLIST_FUNCS=1 meant 87 prototypes to
    # hand-write, and a hand-written prototype wrong by one pointer level links and then
    # corrupts silently. The derive defaults to 1 and does not know the module's shape.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # MACRO_SIGNED stays OFF, and that was MEASURED rather than reasoned: one cargo check each,
    # i386, 2026-08-07 — signed 617 errors against unsigned 97. That is FIVE property-sheet
    # modules in a row (netcfgx and aclui aside, umpnpmgr/syssetup/desk/sysdm/intl) landing on
    # UNSIGNED against the "dialog module, so signed int control ids" argument. Generate both
    # and count; the resemblance argument has now been wrong five times.
    # UNBLOCK so THIS island owns every callback type intl INSTALLS — six property-sheet
    # DLGPROCs (currency/date/generalp/numbers/sort/time) plus the sheet callback. ntdll-sys
    # spells each with the arch's literal convention and normalize_island_abi only rewrites a
    # crate's OWN islands, so an -sys-owned slot stays `stdcall` while the Rust body is
    # `extern "system"` (Rust compares fn-pointer ABIs NOMINALLY). The containers have to come
    # too — a struct whose field type still resolves to the -sys typedef reintroduces the
    # mismatch at the assignment. TCHAR family because ntdll-sys is built WITHOUT UNICODE, so
    # its glob says `TCHAR = c_char`, and intl builds -DUNICODE.
    UNBLOCK="DLGPROC WNDPROC LPFNPSPCALLBACKW LPFNPSPCALLBACKA PFNPROPSHEETCALLBACK
             _PROPSHEETPAGEW _PROPSHEETHEADERW PROPSHEETPAGEW PROPSHEETHEADERW
             TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    # PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union and the C FLATTENS them
    # (`psh.nStartPage`, `psp.pszTemplate`), which `__BindgenUnionField<T>` cannot accept.
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/dll/cpl/intl"
    WRAPPER_INC=$'#include "intl.h"\n#include <debug.h>\n#include <shellapi.h>\n#include <shlobj.h>\n#include <shlwapi.h>\n#include <strsafe.h>\n#include <wingdi.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  # The derive allowlists a header when it OWNS a `case` label, and intl's
                  # window messages resolve through the -sys glob instead — so winuser/winbase
                  # never made the list even though every page proc calls GetDlgItem /
                  # SendDlgItemMessageW / CheckDlgButton and the heap+resource helpers. Types
                  # from a glob are not prototypes; without these two it was 20 more functions
                  # to hand-declare. Same pair as sysdm.
                  --allowlist-file '.*[/\\]winuser\.h'
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*intl[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/intl" -I"${SRC}/build-i386/dll/cpl/intl" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # ---- Tier-3 batch H: sysdm.cpl, the System control panel. Plain C, 12 TUs: the
  # General/Hardware/Advanced property sheets, the environment-variable, user-profile,
  # hardware-profile, virtual-memory and startup/recovery dialogs, and the SMBIOS/DMI
  # device-name reader. ----
  sysdm)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-05 from dll/cpl/sysdm/CMakeLists.txt
    # (12 compiled TU(s), 205 headers in the include closure, 77 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). No ERROR findings and no MOSTLY_DEAD_BODIES.
    USERMODE=1
    # Same call as desk/syssetup/umpnpmgr, and for the same reason: sysdm is a thin driver of
    # other people's APIs (user32/gdi32/comctl32/setupapi/shell32/shlwapi/powrprof/udmihelp),
    # so BLOCKLIST_FUNCS=1 would mean hand-writing several hundred prototypes, and a hand-written
    # prototype wrong by one pointer level links and then corrupts silently. The derive defaults
    # to 1 and does not know the module's shape.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # MEASURE, do not resemble. FOUR property-sheet modules in a row (umpnpmgr, syssetup, desk,
    # and this one) have wanted UNSIGNED against the "dialog module, so signed int control ids"
    # argument. Measured 2026-08-05, i386, one cargo check each: see the commit that sets this.
    MACRO_SIGNED=0
    # UNBLOCK so THIS island owns every callback type sysdm INSTALLS. general.c:708 subclasses a
    # static with SetWindowLongPtr(GWLP_WNDPROC, RosImageProc), and the property sheets install
    # DLGPROCs; ntdll-sys spells each with the arch's literal convention and normalize_island_abi
    # only rewrites a crate's OWN islands, so an -sys-owned slot stays `stdcall` while the Rust
    # body is `extern "system"` (Rust compares fn-pointer ABIs NOMINALLY). The containers have to
    # come too — a struct whose field type still resolves to the -sys typedef reintroduces the
    # mismatch at the assignment. TCHAR family for desk's reason: ntdll-sys is built WITHOUT
    # UNICODE, so its glob says `TCHAR = c_char`, and sysdm builds -DUNICODE.
    UNBLOCK="DLGPROC WNDPROC LPFNPSPCALLBACKW LPFNPSPCALLBACKA PFNPROPSHEETCALLBACK
             LPTHREAD_START_ROUTINE PTHREAD_START_ROUTINE
             _PROPSHEETPAGEW _PROPSHEETHEADERW PROPSHEETPAGEW PROPSHEETHEADERW
             TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    # PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union and the C FLATTENS them
    # (`psh.nStartPage`, `psh.pszCaption`, `psp.pszTemplate`), which the `__BindgenUnionField<T>`
    # emulation cannot accept. crate-repair's `anon_union_field` rule routes the call sites.
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/dll/cpl/sysdm"
    WRAPPER_INC=$'#include "precomp.h"\n#define NTOS_MODE_USER\n#include <ndk/pstypes.h> /* For SharedUserData */\n#include <commctrl.h>\n#include <commdlg.h>\n#include <string.h>\n#include <winnls.h>\n#include <powrprof.h>\n#include <buildno.h>\n#include <debug.h>\n#include <shlguid_undoc.h> // CLSID_CRegTreeOptions\n#include <shlobj_undoc.h> // IRegTreeOptions\n#include <udmihelp.h>\n#include <shlwapi.h>\n#include <regstr.h>\n#include <sddl.h>\n#include <ndk/pstypes.h>\n#define NDEBUG'
    ALLOW_FILES=( --allowlist-file '.*[/\\]buildno\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]commdlg\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]powrprof\.h'
                  # Owns PSCB_INITIALIZED, a `case` label in hardprof.c. The derive adds a
                  # header the moment it owns one (§9.112: an omitted const becomes a
                  # catch-all match arm that swallows every case).
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]sddl\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  # performance.c drives IRegTreeOptions by COBJMACROS. shlobj_undoc.h owns the
                  # interface and its WALK_TREE_* constants; shlguid_undoc.h owns
                  # CLSID_CRegTreeOptions / IID_IRegTreeOptions, which sysdm gets as DATA from
                  # the `uuid` import lib — so unlike aclui and desk there is no guid.c here.
                  --allowlist-file '.*[/\\]shlguid_undoc\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlobj_undoc\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  # LoadSMBiosData / TrimDmiStringW / GetSMBiosStringW / FreeSMBiosData, the
                  # sdk/lib/udmihelp static lib smbios.c drives.
                  --allowlist-file '.*[/\\]udmihelp\.h'
                  # udmihelp.h's own dependency, and the owner of the DMI string INDEX enum
                  # (SYS_VENDOR / SYS_PRODUCT / SYS_VERSION / SYS_FAMILY / BOARD_* /
                  # ID_STRINGS_MAX) that smbios.c uses by BARE MEMBER NAME to index
                  # `PCHAR DmiStrings[ID_STRINGS_MAX]`. The derive excluded it as depth-2 SDK
                  # because these are array subscripts rather than `case` labels, so the switch
                  # audit had nothing to say — but an absent one is still a name that does not
                  # resolve, and here it would be 60 unresolved indices.
                  --allowlist-file '.*[/\\]dmilib\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  # The two headers that own the module's actual Win32 surface — sysdm reaches
                  # user32/kernel32 through ordinary calls, and the derive only adds a header
                  # when it owns a `case` label.
                  --allowlist-file '.*[/\\]winuser\.h'
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*sysdm[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/sysdm" -I"${SRC}/build-i386/dll/cpl/sysdm" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/lib/udmihelp" -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES -DNTOS_MODE_USER )
    ;;

  # ---- Tier-3 batch F: syssetup, the system setup DLL. Plain C, no COM substrate: the
  # second-stage GUI installer (the wizard), the device/DLL registration pass, the account
  # and security-policy setup, and the mouse property page. ----
  syssetup)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-04 from dll/win32/syssetup/CMakeLists.txt
    # (8 compiled TU(s), 203 headers in the include closure, 44 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    # Emit the header function surface, minus what the crate defines. syssetup is a thin driver
    # of other people's APIs — setupapi/samlib/advapi32/userenv/comctl32/shell32 — so
    # BLOCKLIST_FUNCS=1 would mean hand-writing several hundred prototypes, and a hand-written
    # prototype wrong by one pointer level links and then corrupts silently.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # MACRO_SIGNED=0, and that was MEASURED, not reasoned — the reasoning was WRONG. syssetup
    # looks exactly like aclui and netcfgx (wizard.c alone is 3.4K lines of property-sheet page
    # procs, so its constants are `int` control ids and PSN_*/DTN_* codes compared against
    # NMHDR.code) and I set it to 1 on that resemblance. Whole-crate check on i386: 483 errors
    # SIGNED against 432 UNSIGNED. What the resemblance misses is that syssetup spends more of
    # its time handing constants to UNSIGNED Win32 and setupapi parameters than comparing them.
    # Second module in a row where the shape argument lost to the measurement (see umpnpmgr):
    # generate both islands and count, it costs one cargo check.
    MACRO_SIGNED=0
    # UNBLOCK so THIS island owns every callback type syssetup INSTALLS: ntdll-sys spells each
    # with the arch's literal convention, and normalize_island_abi only rewrites a crate's OWN
    # islands, so an -sys-owned slot would stay `stdcall` while the Rust body is `extern "system"`
    # (Rust compares fn-pointer ABIs NOMINALLY). The containers have to come too — a struct whose
    # field type still resolves to the -sys typedef reintroduces the mismatch at the assignment.
    #   DLGPROC                    — PROPSHEETPAGE.pfnDlgProc, every wizard page
    #   LPFNPSPCALLBACKW           — PROPSHEETPAGE.pfnCallback (proppage.c's MouseCallback)
    #   PFNPROPSHEETCALLBACK       — PROPSHEETHEADER.pfnCallback
    #   PSP_FILE_CALLBACK_W        — wizard.c's RegistrationNotificationProc, install.c's queue
    #   *THREAD_START_ROUTINE      — install.c's two CreateThread bodies
    UNBLOCK="DLGPROC WNDPROC LPFNPSPCALLBACKW LPFNPSPCALLBACKA PFNPROPSHEETCALLBACK
             PSP_FILE_CALLBACK_W PSP_FILE_CALLBACK_A LPTHREAD_START_ROUTINE PTHREAD_START_ROUTINE
             _PROPSHEETPAGEW _PROPSHEETHEADERW PROPSHEETPAGEW PROPSHEETHEADERW"
    # syssetup.h forward-declares `struct _TIMEZONE_ENTRY*` for two SETUPDATA fields and never
    # completes it; the definition lives in wizard.c. bindgen therefore emits an OPAQUE
    # `_TIMEZONE_ENTRY { _address: u8 }`, which is a different Rust type from the real one the
    # translation carries — every assignment to SetupData.TimeZoneListHead is then an E0308 and
    # nothing says why. Block the placeholder and point the island's own field types at the
    # crate's definition, so there is exactly ONE _TIMEZONE_ENTRY (same shape as shell32's
    # EXTRA_RAW_LINES for its COM interfaces).
    EXTRA_BL_ITEMS=( _TIMEZONE_ENTRY TIMEZONE_ENTRY PTIMEZONE_ENTRY )
    EXTRA_RAW_LINES=( 'pub use crate::syssetup::wizard::_TIMEZONE_ENTRY;' )
    # PROPSHEETPAGE/PROPSHEETHEADER are half anonymous union: pszTemplate/pResource,
    # nStartPage/pStartPage, hbmWatermark/pszbmWatermark, hbmHeader/pszbmHeader. In the C those
    # are FLATTENED (`psp.pszTemplate = ..`), so the translation writes plain field assignments
    # and the `__BindgenUnionField<T>` emulation cannot accept one. New crate, so this is the
    # opt-in case the knob is for.
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/dll/win32/syssetup"
    WRAPPER_INC=$'#include "precomp.h"\n#define NDEBUG\n#include <debug.h>\n#define COBJMACROS\n#include <io.h>\n#include <wincon.h>\n#include <winnls.h>\n#include <winsvc.h>\n#include <userenv.h>\n#include <shlobj.h>\n#include <shlwapi.h>\n#include <shobjidl.h>\n#include <rpcproxy.h>\n#include <ndk/cmfuncs.h>\n#include <stdarg.h>\n#include <rpc.h>\n#include <ntlsa.h>\n#include <ntsecapi.h>\n#include <ntsam.h>\n#include <sddl.h>\n#include <stdlib.h>\n#include <time.h>\n#include <windowsx.h>\n#include <tzlib.h>\n#include <strsafe.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cfgmgr32\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]ntlsa\.h'
                  --allowlist-file '.*[/\\]ntsam\.h'
                  --allowlist-file '.*[/\\]ntsecapi\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]pseh[/\\]pseh2\.h'
                  --allowlist-file '.*[/\\]rpc\.h'
                  --allowlist-file '.*[/\\]rpcproxy\.h'
                  --allowlist-file '.*[/\\]sddl\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]shobjidl\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]syssetup[/\\]syssetup\.h'
                  --allowlist-file '.*[/\\]tzlib\.h'
                  --allowlist-file '.*[/\\]userenv\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*[/\\]winsvc\.h'
                  # The two headers that own the module's actual Win32 surface. The derive adds a
                  # header only when it owns a `case` label, and syssetup reaches kernel32/user32
                  # through ordinary calls — CreateThread/SetTimer/GetDlgItem/SendDlgItemMessageW
                  # and ~200 more.
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*[/\\]winuser\.h'
                  # The account/logon surface security.c IS: NetUserAdd, NetLocalGroupAddMembers.
                  --allowlist-file '.*[/\\]lmaccess\.h'
                  # install.c drives IShellLink/IPersistFile through COBJMACROS, and wizard.c
                  # creates RPC UUIDs. The derive misses both because it only pulls in a header
                  # that owns a `case` label. Emitting them beats hand-writing ~25 COM/RPC
                  # prototypes and two IIDs: a hand-written vtable is a silent-corruption risk in
                  # a way a hand-written function declaration is not.
                  --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*[/\\]objidl\.h'
                  --allowlist-file '.*[/\\]unknwn\.h'
                  --allowlist-file '.*[/\\]rpcdce\.h'
                  --allowlist-file '.*syssetup[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/syssetup" -I"${SRC}/build-i386/dll/win32/syssetup" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/lib/tzlib" -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES -DNTOS_MODE_USER )
    ;;

  shell32)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-07-25 from dll/win32/shell32/CMakeLists.txt
    # (43 compiled TUs, 299 headers in the include closure, 280 switch constants), then adjusted
    # in three places — each noted below.
    #
    # WHY shell32 gets an island at all, this late: it is the one substantial translated module
    # in the tree that never had one. Its substrate was hand-written instead (types.rs + win32.rs
    # + a generated sdk_consts.rs, ~2.7K lines), which costs twice. First, a hand-written
    # `#[repr(C)]` is a silent-corruption risk in a way a hand-written fn declaration is not: a
    # wrong field offset compiles, links, and scribbles. Second — the reason this blocked for
    # months — `verify_ffi` checks extern blocks against `bindgen_i386.rs`, so a crate with no
    # island is INVISIBLE to the fleet's FFI oracle. shell32-rust appears in zero of the 40
    # crates the clashing-extern passes converged (e33a810c9ad, 992af657162), and 4 real ABI
    # disagreements survived in it until a semantic decl comparison went looking by hand.
    USERMODE=1
    # Emit the header function surface, minus the 552 functions the crate defines itself.
    # See BLOCKLIST_FUNCS_FROM_CRATE for why the crate's `#[no_mangle]` set is the right
    # discriminator and shell32.spec is not.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # shell32 bridges TWO -sys crates: ntdll-sys for base Win32/NT types, and com-sys for the
    # COM ABI. Without this the island re-emits its own IUnknown/VARIANT/FORMATETC/IID/ULONG
    # from objbase.h etc., and every module that globs both sees an ambiguous name — 6,232
    # E0659s on the first build, on 216 distinct names. Worse than the error count: a
    # structurally-identical SECOND FORMATETC is a different Rust type, so the COM-class modules
    # that hand com-sys-typed pointers across an interface boundary would stop type-checking
    # against the very vtables they implement. Blocklisting keeps ONE identity.
    SYS_CRATES=( "com-sys:com-sys/src" )
    # Emitted as island_<arch>.rs / island.rs, NOT types_<arch>.rs: shell32-rust already has a
    # hand-written types.rs + shellcom.rs that 40-odd LANDED COM-class modules compile against.
    # Overwriting those is a separate, riskier change than adding the island — see the COM
    # blocklist below for the design conflict that has to be settled first.
    TYPES_STEM=island
    # COM INTERFACES STAY WITH com-sys/shellcom, deliberately. bindgen renders shobjidl's
    # interfaces as C++ classes; the whole translation (and com-sys, which exists for this) uses
    # the C `lpVtbl` idiom — `(*(*This).lpVtbl).Method(...)`. Letting the island emit them makes
    # every such call an E0609 `no field lpVtbl`, ~73 of them on the first build. The boundary
    # that falls out is the right one and worth stating plainly: THE ISLAND OWNS DATA LAYOUTS,
    # com-sys/shellcom OWN VTABLE ABI. Only interfaces that actually collide are listed.
    EXTRA_BL_ITEMS=( IEnumIDList IEnumShellItems IFolderView IPersistFolder2 IPersistIDList
                     IShellFolder IShellFolder2 IShellItem IShellItemArray
                     # Member typedefs of the ATL class TEMPLATES CComEnum<> / CComEnumImpl<>.
                     # bindgen flattens them to non-generic aliases over UNBOUND template
                     # parameters — `pub type ATL_CComEnumImpl_enumeratorClass =
                     # ATL_CComObject<Base>` and `pub type ATL_CComEnum__CComEnumBase =
                     # u8<Base, T, Copy>` — neither of which compiles. Nothing in the
                     # translation names them; the ATL headers are in the closure at all only
                     # because precomp.h drags them in. Checked before widening this: the other
                     # 82 aliases carrying type arguments are legitimate `Option<fn-ptr>`
                     # callbacks, so "alias with a `<`" would have been the wrong rule.
                     # NB: --blocklist-item matches the C++ QUALIFIED name, not bindgen's
                     # flattened Rust spelling (ATL_CComEnumImpl_enumeratorClass).
                     '.*CComEnum.*' )
    # ...and re-export them, because other emitted types still reference these by pointer.
    EXTRA_RAW_LINES=( 'pub use crate::shellcom::{IEnumIDList, IEnumShellItems, IFolderView, IPersistFolder2, IPersistIDList, IShellFolder, IShellFolder2, IShellItem, IShellItemArray};'
                      # com-sys carries the enum TAGS (_APTTYPE/_APTTYPEQUALIFIER, from
                      # objidl.h) but not these typedef aliases, which objbase.h declares —
                      # and objbase.h's CoGetApartmentType takes them by pointer. Aliasing to
                      # the com-sys tags keeps one identity rather than emitting a second enum.
                      'pub type APTTYPE = com_sys::_APTTYPE;'
                      'pub type APTTYPEQUALIFIER = com_sys::_APTTYPEQUALIFIER;' )
    DRIVER_DIR="${SRC}/dll/win32/shell32"
    WRAPPER_INC=$'#include "precomp.h"\n#include <ui/layout.h> // Resizable window\n#include <compat_undoc.h> // RosGetProcessEffectiveVersion\n#include <process.h>\n#include <ndk/obfuncs.h> // For NtQueryObject\n#define _USE_MATH_DEFINES\n#include <math.h>\n#include <devguid.h>\n#define NTOS_MODE_USER\n#include <ndk/iofuncs.h>\n#include <ndk/obfuncs.h>\n#include <atlpath.h>\n#include <precomp.h>\n#include "shell32_version.h"\n#include <lmcons.h>\n#include <lmapibuf.h>\n#include <lmaccess.h>\n#include <lmserver.h>\n#include <secext.h>\n#include "folders/CFSFolder.h" // was "CFSFolder.h" in CDesktopFolder.cpp: a QUOTED include\n// resolves against the INCLUDING file'"'"'s directory, and this wrapper is generated into /tmp.\n#include <mmsystem.h>\n#include <ntquery.h>\n#include <winbase_undoc.h>\n#include <undocshell.h>\n#include <atlwin.h>\n#include <ui/rosctrls.h>\n#include <compat_undoc.h>\n#include <stdarg.h>\n#include <stddef.h>'
    # precomp.h pulls ATL (atlbase/atlcom/atlstr/atlwin), so the parse needs C++. bindgen still
    # emits the plain C structs; `strip_cpp_mangled_externs.py` runs automatically for -x c++
    # cells and drops the `_Z`-mangled decls ATL/C++ member functions produce.
    LANG_ARGS=( -x c++ -std=gnu++17 )
    ALLOW_FILES=( --allowlist-file '.*[/\\]atlbase\.h' --allowlist-file '.*[/\\]atlcoll\.h'
                  --allowlist-file '.*[/\\]atlcom\.h'
                  --allowlist-file '.*[/\\]atlpath\.h'
                  --allowlist-file '.*[/\\]atlsimpcoll\.h'
                  --allowlist-file '.*[/\\]atlstr\.h'
                  --allowlist-file '.*[/\\]atlwin\.h'
                  --allowlist-file '.*[/\\]browseui_undoc\.h'
                  --allowlist-file '.*[/\\]comctl32_undoc\.h'
                  --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]commdlg\.h'
                  --allowlist-file '.*[/\\]compat_undoc\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]ddeml\.h'
                  --allowlist-file '.*[/\\]devguid\.h'
                  --allowlist-file '.*[/\\]emptyvc\.h'
                  --allowlist-file '.*[/\\]fmifs[/\\]fmifs\.h'
                  --allowlist-file '.*[/\\]ishellfolder_helpers\.h'
                  --allowlist-file '.*[/\\]lmaccess\.h'
                  --allowlist-file '.*[/\\]lmapibuf\.h'
                  --allowlist-file '.*[/\\]lmcons\.h'
                  --allowlist-file '.*[/\\]lmserver\.h'
                  --allowlist-file '.*[/\\]mmsystem\.h'
                  --allowlist-file '.*[/\\]ntquery\.h'
                  --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*[/\\]objsafe\.h'
                  --allowlist-file '.*[/\\]powrprof\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]secext\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shellutils\.h'
                  --allowlist-file '.*[/\\]shlguid\.h'
                  --allowlist-file '.*[/\\]shlguid_undoc\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlobj_undoc\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]shlwapi_undoc\.h'
                  --allowlist-file '.*[/\\]shobjidl\.h'
                  --allowlist-file '.*[/\\]shobjidl_undoc\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]ui[/\\]layout\.h'
                  --allowlist-file '.*[/\\]ui[/\\]rosctrls\.h'
                  --allowlist-file '.*[/\\]undocshell\.h'
                  --allowlist-file '.*[/\\]userenv\.h'
                  --allowlist-file '.*[/\\]uxtheme\.h'
                  --allowlist-file '.*[/\\]winbase_undoc\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]wininet\.h'
                  --allowlist-file '.*[/\\]winnetwk\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*[/\\]winver\.h'
                  # winuser.h/winbase.h are "base platform", which profile_derive excludes
                  # because their TYPES belong to ntdll-sys — and they still do: every type
                  # ntdll-sys exports is blocklisted, so allowlisting these files adds their
                  # FUNCTIONS only. Between them they own 38 of the imports the translated TUs
                  # call (CreateDirectoryW, DrawTextW, EnterCriticalSection, GetAncestor, ...),
                  # which no other allowlisted header declares.
                  --allowlist-file '.*[/\\]winuser\.h'
                  --allowlist-file '.*[/\\]winbase\.h'
                  # Same "base platform" reasoning, for the two headers the DISK IOCTL path
                  # needs. ioapiset.h (via winbase.h:21) owns DeviceIoControl, which no other
                  # allowlisted header declares. winioctl.h (via ndk/umtypes.h:49, which
                  # precomp.h reaches through ndk/rtlfuncs.h) owns GET_LENGTH_INFORMATION,
                  # DISK_GEOMETRY, PARTITION_INFORMATION{,_EX} and the PARTITION_STYLE_* enum.
                  # Without it dialogs/drive.cpp's translation has no layout at all for the
                  # buffers it hands DeviceIoControl — the §9.112 shape, one step upstream.
                  #
                  # What actually lands from winioctl.h is only the FILE_DEVICE_* / METHOD_* /
                  # FILE_*_ACCESS constants: ntdll-sys already owns every disk STRUCT and is
                  # blocklisted here so they keep one identity. The IOCTL_DISK_* codes land
                  # NOWHERE — they are CTL_CODE(...) function-like macros, so bindgen emits
                  # nothing and shell32-rust's prelude.rs computes them from the parts above.
                  #
                  # SCOPED TO psdk/ on purpose, unlike its neighbours: sdk/include/wine carries
                  # a SECOND winioctl.h, and an unscoped '.*[/\\]winioctl\.h' would emit from
                  # whichever the -I order resolved. See the LATE_INC note at the top of this
                  # file for what that alternate surface does to _FILE_FS_*_INFORMATION.
                  --allowlist-file '.*[/\\]ioapiset\.h'
                  --allowlist-file '.*[/\\]psdk[/\\]winioctl\.h'
                  --allowlist-file '.*shell32[/\\].*\.h' )
    # The derived profile also proposed -I"${SRC}/build-i386/dll/win32/shell32". Dropped: that
    # directory carries no headers, and baking ONE arch's build dir into a 3-arch generator is
    # exactly the cross-arch header leak AGENTS.md forbids — BUILD_DIR below is already per-arch.
    # sdk/lib/atl is on the include path for every ATL consumer (precomp.h:31 `#include
    # <atlbase.h>`); the compile_commands entry gets it from the atl_classes link target, which
    # the derived profile cannot see.
    EXTRA_INC=( -I"${SRC}/dll/win32/shell32" -I"${SRC}/sdk/lib/atl" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_SHELL32_ -D_WINE -D_DEBUG=1
               -DUNICODE -D_UNICODE -D_ATL_NO_EXCEPTIONS -D_CRT_DECLARE_NONSTDC_NAMES=1
               -D_CRT_NONSTDC_NO_DEPRECATE -D_CRT_NON_CONFORMING_SWPRINTFS -D_DLL
               -D__USE_CRTIMP -D_CRT_NO_POSIX_ERROR_CODES -DNTOS_MODE_USER )
    ;;

  # ---- shell32's private recycle-bin static library (dll/win32/shell32/shellrecyclebin).
  # Its own CMake target, linked into shell32.dll, so it gets its own crate rather than
  # folding into shell32-rust: shell32's island is 44K lines that 40+ landed COM modules
  # compile against, and adding recyclebin_private.h/recyclebin_v5.h to ITS wrapper would
  # perturb all of them to buy two headers used by six TUs. ----
  recyclebin)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-08-07 from
    # dll/win32/shell32/shellrecyclebin/CMakeLists.txt (6 compiled TU(s), 180 headers in the
    # include closure, 0 switch constants — the §9.112 audit found no identifier case labels
    # anywhere in the subtree), then adjusted in the four places noted below.
    USERMODE=1
    # Emit the header function surface minus what the crate defines. The subtree is a thin
    # layer over Win32 file I/O (CreateFileW/ReadFile/SetFilePointer/SHFileOperationW/
    # PathAppendW/SHAlloc), so BLOCKLIST_FUNCS=1 would mean hand-writing ~120 prototypes.
    BLOCKLIST_FUNCS=0
    BLOCKLIST_FUNCS_FROM_CRATE=1
    # ADJUSTMENT 1 — the derive proposed `-x c++` via atlstr.h, and taking it would have been
    # the expensive mistake here. recyclebin.h/recyclebin_v5.h declare all four interfaces
    # (IRecycleBin, IRecycleBin5, IRecycleBinEnumList, IRecycleBinFile) with DECLARE_INTERFACE_.
    # Parsed as C++ that yields bindgen's OPAQUE `__bindgen_vtable` — which you can hold a
    # pointer to but cannot implement, and implementing them is the entire job. Parsed as C it
    # yields the `lpVtbl` struct the COBJMACROS in these very headers already use. The whole
    # cost of staying in C is FOUR CStringW sites (two RecycleBin5 fields, two locals); the cost
    # of C++ would be hand-writing every slot of four interfaces, which is exactly the debt
    # folders/CRecycleBin.rs is already carrying privately.
    LANG_ARGS=()
    # A/W ALIASING REWRITES VTABLE SLOT NAMES, and the island is right where it looks wrong.
    # IRecycleBin5Vtbl's slots 4 and 5 come out `DeleteFileW` and `EmptyRecycleBinW` while the
    # C++ writes `DeleteFile`/`EmptyRecycleBin` — winbase.h makes both A/W macros under UNICODE
    # and the preprocessor rewrites the METHOD NAMES, on the C++ side identically. Same class as
    # the TVN_SELCHANGED -> TVN_SELCHANGEDW alias that §9.112 records. A translation spelling
    # `DeleteFile` gets a loud E0609, so this costs confusion rather than correctness.
    # ADJUSTMENT 2 — com-sys, not a second IUnknown. objbase.h reaches the closure through
    # recyclebin.h, so without this the island re-emits IUnknown/IID/HRESULT and anything
    # bridging to shell32-rust sees two incompatible identities for the same ABI.
    SYS_CRATES=( "com-sys:com-sys/src" )
    # com-sys carries the enum TAGS (_APTTYPE/_APTTYPEQUALIFIER, from objidl.h) but not these
    # typedef aliases, which objbase.h declares — and objbase.h's CoGetApartmentType takes them
    # BY POINTER, so the island emits a signature naming types it never defines. Same two lines
    # shell32's profile carries, for the same reason. Aliasing to the com-sys tags keeps one
    # identity rather than emitting a second enum.
    EXTRA_RAW_LINES=( 'pub type APTTYPE = com_sys::_APTTYPE;'
                      'pub type APTTYPEQUALIFIER = com_sys::_APTTYPEQUALIFIER;' )
    # New usermode crate, so real unions per the REAL_UNIONS note at the top of this file —
    # nothing here is compiled against the __BindgenUnionField accessor API yet, and
    # ULARGE_INTEGER is threaded through the file-size methods of IRecycleBinFile.
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/dll/win32/shell32/shellrecyclebin"
    # ADJUSTMENT 3 — atlstr.h dropped (see ADJUSTMENT 1), and `#define INITGUID` dropped with
    # initguid.h. INITGUID turns every DEFINE_GUID into a DEFINITION; the IIDs are guid.c's job
    # and the Rust replacement publishes them itself, so the island wants the plain
    # `EXTERN_C const IID IID_IRecycleBin;` declarations that recyclebin.h gives without it.
    WRAPPER_INC=$'#include "recyclebin_private.h"\n#include <shlwapi.h>\n#include "sddl.h"\n#include <strsafe.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*[/\\]sddl\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winver\.h'
                  # The API surface the subtree actually calls. The derive leaves these out
                  # because SWITCH_CONST_HEADER_ADDED only pulls a header that owns a `case`
                  # label, and this subtree reaches kernel32/advapi32 by ordinary calls —
                  # CreateFileW, SetFilePointer, MapViewOfFile, ConvertStringSecurityDescriptor*.
                  --allowlist-file '.*[/\\]winbase\.h'
                  --allowlist-file '.*[/\\]winuser\.h'
                  --allowlist-file '.*shellrecyclebin[/\\].*\.h' )
    # ADJUSTMENT 4 — the derive also proposed -I"${SRC}/build-i386/dll/win32/shell32/
    # shellrecyclebin". Dropped for the same reason it was dropped from shell32 above: that
    # directory carries no headers, and baking ONE arch's build dir into a 3-arch generator is
    # the cross-arch header leak AGENTS.md forbids.
    EXTRA_INC=( -I"${SRC}/dll/win32/shell32/shellrecyclebin" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    # sdk/lib/atl is NOT here; see ADJUSTMENT 1.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400
               -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_UNICODE
               -D_ATL_NO_EXCEPTIONS -D_CRT_NON_CONFORMING_SWPRINTFS
               -D_CRT_NO_POSIX_ERROR_CODES )
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
  usbxhci)
    # USB 3 xHCI miniport, written in Rust from the start (no C body). usbxhci.h only
    # includes the SDK surface, so the island is the usbmport.h miniport interface + USB
    # descriptor layouts; the xHCI hardware structs live in the crate's hw.rs.
    DRIVER_DIR="${SRC}/drivers/usb/usbxhci"
    WRAPPER_INC=$'#include "usbxhci.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]usb\.h'
                  --allowlist-file '.*[/\\]usb100\.h'
                  --allowlist-file '.*[/\\]usb200\.h'
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
  legide)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from drivers/storage/ide/legide/CMakeLists.txt
    # (2 compiled TU(s), 70 headers in the include closure, 0 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=0
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/drivers/storage/ide/legide"
    WRAPPER_INC=$'#include <ntddk.h>\n#include <ide.h>\n#include <ata.h>\n#include <scsi.h>\n#include <reactos/drivers/ata/ata_shared.h>\n#define NDEBUG\n#include <debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]ata\.h' --allowlist-file '.*[/\\]ide\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]drivers[/\\]ata[/\\]ata_shared\.h'
                  --allowlist-file '.*[/\\]scsi\.h'
                  --allowlist-file '.*legide[/\\].*\.h'
                  --allowlist-file '.*[/\\]ntddk\.h'
                  --allowlist-file '.*[/\\]ntdef\.h' )
    EXTRA_INC=( -I"${SRC}/drivers/storage/ide/legide" -I"${SRC}/build-i386/drivers/storage/ide/legide" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  setuplib)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-06 from base/setup/lib/CMakeLists.txt
    # (23 compiled TU(s), 298 headers in the include closure, 23 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    # BLOCKLIST_FUNCS, MEASURED: 77 external prototype(s) declared by the include closure, 30 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/setup/lib"
    WRAPPER_INC=$'#include "precomp.h"\n#include "fileqsup.h"\n#define NDEBUG\n#include <debug.h>\n#include "infsupp.h"\n#include "filesup.h"\n#include "partlist.h"\n#include "arcname.h"\n#include "bldrsup.h"\n#include "inicache.h"\n#include "devutils.h"\n#include <pseh/pseh2.h>\n#include <ndk/umfuncs.h> // Ldr* functions\n#include "fsrec.h"\n#include "genlist.h"\n#include <ndk/ldrtypes.h>\n#include <ndk/ldrfuncs.h>\n#include "ntverrsrc.h"\n#include "osdetect.h"\n#include "partinfo.h"\n#include <ntddscsi.h>\n#include <mountdev.h> // For IOCTL_MOUNTDEV_QUERY_DEVICE_NAME\n#include "volutil.h"\n#include "fsrec.h" // For FileSystemToMBRPartitionType()\n#include "registry.h"\n#include "regutil.h"\n#include "bootcode.h"\n#include <ntddstor.h> // For STORAGE_DEVICE_NUMBER\n#include "fsutil.h"\n#include "setuplib.h"\n#include <fslib/vfatlib.h>\n#include "setuplib.h" // HACK for USETUP_DATA\n#include "install.h"\n#include "mui.h"\n#include "muifonts.h"\n#include "muilanguages.h"\n#include "substset.h"\n#include "settings.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]fmifs[/\\]fmifs\.h' --allowlist-file '.*[/\\]fslib[/\\]vfatlib\.h'
                  --allowlist-file '.*[/\\]ldrtypes\.h'
                  --allowlist-file '.*[/\\]mountdev\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h'
                  --allowlist-file '.*[/\\]ntddstor\.h'
                  --allowlist-file '.*[/\\]ntstrsafe\.h'
                  --allowlist-file '.*[/\\]pseh[/\\]pseh2\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]rosioctl\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*lib[/\\].*\.h'
                  --allowlist-file '.*[/\\]mountmgr\.h' )
    EXTRA_INC=( -I"${SRC}/base/setup/lib" -I"${SRC}/build-i386/base/setup/lib" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/base/setup/lib/spapisup" -I"${SRC}/base/setup/lib/utils" -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_WINKD_ -D_SETUPLIB_ -DLANGUAGE_AF_ZA -DLANGUAGE_AR_AE -DLANGUAGE_AR_BH -DLANGUAGE_AR_DZ -DLANGUAGE_AR_EG -DLANGUAGE_AR_IQ -DLANGUAGE_AR_JO -DLANGUAGE_AR_KW -DLANGUAGE_AR_LB -DLANGUAGE_AR_LY -DLANGUAGE_AR_MA -DLANGUAGE_AR_OM -DLANGUAGE_AR_QA -DLANGUAGE_AR_SA -DLANGUAGE_AR_SY -DLANGUAGE_AR_TN -DLANGUAGE_AR_YE -DLANGUAGE_AZ_AZ -DLANGUAGE_BE_BY -DLANGUAGE_BG_BG -DLANGUAGE_BN_BD -DLANGUAGE_BN_IN -DLANGUAGE_CA_ES -DLANGUAGE_CS_CZ -DLANGUAGE_DA_DK -DLANGUAGE_DE_AT -DLANGUAGE_DE_CH -DLANGUAGE_DE_DE -DLANGUAGE_DE_LI -DLANGUAGE_DE_LU -DLANGUAGE_DV_MV -DLANGUAGE_EL_GR -DLANGUAGE_EN_AU -DLANGUAGE_EN_BZ -DLANGUAGE_EN_CA -DLANGUAGE_EN_CB -DLANGUAGE_EN_GB -DLANGUAGE_EN_IE -DLANGUAGE_EN_JM -DLANGUAGE_EN_NZ -DLANGUAGE_EN_PH -DLANGUAGE_EN_TT -DLANGUAGE_EN_US -DLANGUAGE_EN_ZA -DLANGUAGE_EN_ZW -DLANGUAGE_EO_AA -DLANGUAGE_ES_AR -DLANGUAGE_ES_BO -DLANGUAGE_ES_CL -DLANGUAGE_ES_CO -DLANGUAGE_ES_CR -DLANGUAGE_ES_DO -DLANGUAGE_ES_EC -DLANGUAGE_ES_ES -DLANGUAGE_ES_GT -DLANGUAGE_ES_HN -DLANGUAGE_ES_MX -DLANGUAGE_ES_NI -DLANGUAGE_ES_PA -DLANGUAGE_ES_PE -DLANGUAGE_ES_PR -DLANGUAGE_ES_PY -DLANGUAGE_ES_SV -DLANGUAGE_ES_UY -DLANGUAGE_ES_VE -DLANGUAGE_ET_EE -DLANGUAGE_EU_ES -DLANGUAGE_FA_IR -DLANGUAGE_FI_FI -DLANGUAGE_FO_FO -DLANGUAGE_FR_BE -DLANGUAGE_FR_CA -DLANGUAGE_FR_CH -DLANGUAGE_FR_FR -DLANGUAGE_FR_LU -DLANGUAGE_FR_MC -DLANGUAGE_GL_ES -DLANGUAGE_GU_IN -DLANGUAGE_HE_IL -DLANGUAGE_HI_IN -DLANGUAGE_HR_HR -DLANGUAGE_HU_HU -DLANGUAGE_HY_AM -DLANGUAGE_ID_ID -DLANGUAGE_IS_IS -DLANGUAGE_IT_CH -DLANGUAGE_IT_IT -DLANGUAGE_JA_JP -DLANGUAGE_KA_GE -DLANGUAGE_KK_KZ -DLANGUAGE_KN_IN -DLANGUAGE_KOK_IN -DLANGUAGE_KO_KR -DLANGUAGE_KY_KG -DLANGUAGE_LT_LT -DLANGUAGE_LV_LV -DLANGUAGE_MK_MK -DLANGUAGE_MN_MN -DLANGUAGE_MR_IN -DLANGUAGE_MS_BN -DLANGUAGE_MS_MY -DLANGUAGE_MY_MM -DLANGUAGE_NL_BE -DLANGUAGE_NL_NL -DLANGUAGE_NN_NO -DLANGUAGE_PA_IN -DLANGUAGE_PL_PL -DLANGUAGE_PT_BR -DLANGUAGE_PT_PT -DLANGUAGE_RM_CH -DLANGUAGE_RO_RO -DLANGUAGE_RU_RU -DLANGUAGE_SA_IN -DLANGUAGE_SK_SK -DLANGUAGE_SL_SI -DLANGUAGE_SQ_AL -DLANGUAGE_SR_SP -DLANGUAGE_SV_FI -DLANGUAGE_SV_SE -DLANGUAGE_SW_KE -DLANGUAGE_SYR_SY -DLANGUAGE_TA_IN -DLANGUAGE_TE_IN -DLANGUAGE_TH_TH -DLANGUAGE_TR_TR -DLANGUAGE_TT_TA -DLANGUAGE_UK_UA -DLANGUAGE_UR_PK -DLANGUAGE_UZ_UZ -DLANGUAGE_VI_VN -DLANGUAGE_WA_BE -DLANGUAGE_ZH_CN -DLANGUAGE_ZH_HK -DLANGUAGE_ZH_MO -DLANGUAGE_ZH_SG -DLANGUAGE_ZH_TW -D_CRT_NO_POSIX_ERROR_CODES -DNTOS_MODE_USER )
    ;;
  usetup)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-07 from base/setup/usetup/CMakeLists.txt
    # (15 compiled TU(s), 271 headers in the include closure, 74 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 124 external prototype(s) declared by the include closure, 9 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/setup/usetup"
    WRAPPER_INC=$'#include "usetup.h"\n#define Z_SOLO\n#include <zlib.h>\n#define NDEBUG\n#include <debug.h>\n#include <usetup.h>\n#include <blue/ntddblue.h>\n#include "keytrans.h"\n#define INITGUID\n#include <guiddef.h>\n#include <libs/umpnpmgr/sysguid.h>\n#include "muilanguages.h"\n#include <math.h>\n#include <ntstrsafe.h>\n#include "cmdcons.h"\n#include "devinst.h"\n#include "fmtchk.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]\.\.[/\\]lib[/\\]setuplib\.h' --allowlist-file '.*[/\\]blue[/\\]ntddblue\.h'
                  --allowlist-file '.*[/\\]fmifs[/\\]fmifs\.h'
                  --allowlist-file '.*[/\\]fsutil\.h'
                  --allowlist-file '.*[/\\]infros\.h'
                  --allowlist-file '.*[/\\]libs[/\\]umpnpmgr[/\\]sysguid\.h'
                  --allowlist-file '.*[/\\]ntddkbd\.h'
                  --allowlist-file '.*[/\\]ntstrsafe\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]buildno\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]rosioctl\.h'
                  --allowlist-file '.*[/\\]spapisup[/\\]fileqsup\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*[/\\]winuser\.rh'
                  --allowlist-file '.*[/\\]zlib\.h'
                  --allowlist-file '.*usetup[/\\].*\.h'
                  --allowlist-file '.*[/\\]spapisup[/\\]infsupp\.h'
                  --allowlist-file '.*[/\\]wincon\.h' )
    EXTRA_INC=( -I"${SRC}/base/setup/usetup" -I"${SRC}/build-i386/base/setup/usetup" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/reactos/drivers" -I"${SRC}/sdk/lib/inflib" -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/vendor/zlib/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DLANGUAGE_AF_ZA -DLANGUAGE_AR_AE -DLANGUAGE_AR_BH -DLANGUAGE_AR_DZ -DLANGUAGE_AR_EG -DLANGUAGE_AR_IQ -DLANGUAGE_AR_JO -DLANGUAGE_AR_KW -DLANGUAGE_AR_LB -DLANGUAGE_AR_LY -DLANGUAGE_AR_MA -DLANGUAGE_AR_OM -DLANGUAGE_AR_QA -DLANGUAGE_AR_SA -DLANGUAGE_AR_SY -DLANGUAGE_AR_TN -DLANGUAGE_AR_YE -DLANGUAGE_AZ_AZ -DLANGUAGE_BE_BY -DLANGUAGE_BG_BG -DLANGUAGE_BN_BD -DLANGUAGE_BN_IN -DLANGUAGE_CA_ES -DLANGUAGE_CS_CZ -DLANGUAGE_DA_DK -DLANGUAGE_DE_AT -DLANGUAGE_DE_CH -DLANGUAGE_DE_DE -DLANGUAGE_DE_LI -DLANGUAGE_DE_LU -DLANGUAGE_DV_MV -DLANGUAGE_EL_GR -DLANGUAGE_EN_AU -DLANGUAGE_EN_BZ -DLANGUAGE_EN_CA -DLANGUAGE_EN_CB -DLANGUAGE_EN_GB -DLANGUAGE_EN_IE -DLANGUAGE_EN_JM -DLANGUAGE_EN_NZ -DLANGUAGE_EN_PH -DLANGUAGE_EN_TT -DLANGUAGE_EN_US -DLANGUAGE_EN_ZA -DLANGUAGE_EN_ZW -DLANGUAGE_EO_AA -DLANGUAGE_ES_AR -DLANGUAGE_ES_BO -DLANGUAGE_ES_CL -DLANGUAGE_ES_CO -DLANGUAGE_ES_CR -DLANGUAGE_ES_DO -DLANGUAGE_ES_EC -DLANGUAGE_ES_ES -DLANGUAGE_ES_GT -DLANGUAGE_ES_HN -DLANGUAGE_ES_MX -DLANGUAGE_ES_NI -DLANGUAGE_ES_PA -DLANGUAGE_ES_PE -DLANGUAGE_ES_PR -DLANGUAGE_ES_PY -DLANGUAGE_ES_SV -DLANGUAGE_ES_UY -DLANGUAGE_ES_VE -DLANGUAGE_ET_EE -DLANGUAGE_EU_ES -DLANGUAGE_FA_IR -DLANGUAGE_FI_FI -DLANGUAGE_FO_FO -DLANGUAGE_FR_BE -DLANGUAGE_FR_CA -DLANGUAGE_FR_CH -DLANGUAGE_FR_FR -DLANGUAGE_FR_LU -DLANGUAGE_FR_MC -DLANGUAGE_GL_ES -DLANGUAGE_GU_IN -DLANGUAGE_HE_IL -DLANGUAGE_HI_IN -DLANGUAGE_HR_HR -DLANGUAGE_HU_HU -DLANGUAGE_HY_AM -DLANGUAGE_ID_ID -DLANGUAGE_IS_IS -DLANGUAGE_IT_CH -DLANGUAGE_IT_IT -DLANGUAGE_JA_JP -DLANGUAGE_KA_GE -DLANGUAGE_KK_KZ -DLANGUAGE_KN_IN -DLANGUAGE_KOK_IN -DLANGUAGE_KO_KR -DLANGUAGE_KY_KG -DLANGUAGE_LT_LT -DLANGUAGE_LV_LV -DLANGUAGE_MK_MK -DLANGUAGE_MN_MN -DLANGUAGE_MR_IN -DLANGUAGE_MS_BN -DLANGUAGE_MS_MY -DLANGUAGE_MY_MM -DLANGUAGE_NL_BE -DLANGUAGE_NL_NL -DLANGUAGE_NN_NO -DLANGUAGE_PA_IN -DLANGUAGE_PL_PL -DLANGUAGE_PT_BR -DLANGUAGE_PT_PT -DLANGUAGE_RM_CH -DLANGUAGE_RO_RO -DLANGUAGE_RU_RU -DLANGUAGE_SA_IN -DLANGUAGE_SK_SK -DLANGUAGE_SL_SI -DLANGUAGE_SQ_AL -DLANGUAGE_SR_SP -DLANGUAGE_SV_FI -DLANGUAGE_SV_SE -DLANGUAGE_SW_KE -DLANGUAGE_SYR_SY -DLANGUAGE_TA_IN -DLANGUAGE_TE_IN -DLANGUAGE_TH_TH -DLANGUAGE_TR_TR -DLANGUAGE_TT_TA -DLANGUAGE_UK_UA -DLANGUAGE_UR_PK -DLANGUAGE_UZ_UZ -DLANGUAGE_VI_VN -DLANGUAGE_WA_BE -DLANGUAGE_ZH_CN -DLANGUAGE_ZH_HK -DLANGUAGE_ZH_MO -DLANGUAGE_ZH_SG -DLANGUAGE_ZH_TW -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  diskpart)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-07 from base/system/diskpart/CMakeLists.txt
    # (43 compiled TU(s), 141 headers in the include closure, 10 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 56 external prototype(s) declared by the include closure, 5 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/system/diskpart"
    WRAPPER_INC=$'#include "diskpart.h"\n#define NDEBUG\n#include <debug.h>\n#include <fmifs/fmifs.h>\n#define INITGUID\n#include <guiddef.h>\n#include <diskguid.h>\n#include "diskpart_msg.h"\n#include <mountmgr.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]conutils\.h' --allowlist-file '.*[/\\]diskguid\.h'
                  --allowlist-file '.*[/\\]diskpart_msg\.h'
                  --allowlist-file '.*[/\\]fmifs[/\\]fmifs\.h'
                  --allowlist-file '.*[/\\]mountmgr\.h'
                  --allowlist-file '.*[/\\]ntddscsi\.h'
                  --allowlist-file '.*[/\\]ntddstor\.h'
                  --allowlist-file '.*[/\\]ntsecapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]winioctl\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*diskpart[/\\].*\.h'
                  --allowlist-file '.*[/\\]stream\.h' )
    EXTRA_INC=( -I"${SRC}/base/system/diskpart" -I"${SRC}/build-i386/base/system/diskpart" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/reactos/libs/fmifs" -I"${SRC}/sdk/lib/conutils" -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  browseui)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-07 from dll/win32/browseui/CMakeLists.txt
    # (27 compiled TU(s), 231 headers in the include closure, 155 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 473 external prototype(s) declared by the include closure, 83 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/browseui"
    WRAPPER_INC=$'#include "precomp.h"\n#include <commoncontrols.h>\n#include <shlwapi_undoc.h>\n#include <shellapi.h>\n#include <imm.h> // For IMN_OPENCANDIDATE\n#include <process.h> // _beginthreadex\n#include <shlwapi.h>\n#include <strsafe.h>\n#include <htiframe.h>\n#include <shdocvw_undoc.h>\n#include <precomp.h>\n#include <windef.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]atlbase\.h' --allowlist-file '.*[/\\]atlcom\.h'
                  --allowlist-file '.*[/\\]atlsimpcoll\.h'
                  --allowlist-file '.*[/\\]atlstr\.h'
                  --allowlist-file '.*[/\\]atltypes\.h'
                  --allowlist-file '.*[/\\]atlwin\.h'
                  --allowlist-file '.*[/\\]browseui_undoc\.h'
                  --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]commoncontrols\.h'
                  --allowlist-file '.*[/\\]exdispid\.h'
                  --allowlist-file '.*[/\\]htiframe\.h'
                  --allowlist-file '.*[/\\]imm\.h'
                  --allowlist-file '.*[/\\]perhist\.h'
                  --allowlist-file '.*[/\\]shdeprecated\.h'
                  --allowlist-file '.*[/\\]shdocvw_undoc\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shellutils\.h'
                  --allowlist-file '.*[/\\]shlguid_undoc\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlobj_undoc\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]shlwapi_undoc\.h'
                  --allowlist-file '.*[/\\]shobjidl\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]tlogstg\.h'
                  --allowlist-file '.*[/\\]ui[/\\]rosctrls\.h'
                  --allowlist-file '.*[/\\]undocshell\.h'
                  --allowlist-file '.*[/\\]undocuser\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnetwk\.h'
                  --allowlist-file '.*browseui[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/win32/browseui" -I"${SRC}/build-i386/dll/win32/browseui" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/sdk/lib/atl" -I"${SRC}/.build-tools/llvm-mingw/i686-w64-mingw32/include/c++/v1" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_ATL_NO_EXCEPTIONS -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  appwiz)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-07 from dll/cpl/appwiz/CMakeLists.txt
    # (4 compiled TU(s), 180 headers in the include closure, 17 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 78 external prototype(s) declared by the include closure, 7 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/appwiz"
    WRAPPER_INC=$'#include "appwiz.h"\n#include <stdio.h>\n# include <unistd.h>\n#include <msi.h>\n#include <shellapi.h>\n#include <cpl.h>\n#include <wine/unicode.h>\n#include <commctrl.h>\n#include <strsafe.h>\n#include <shlwapi_undoc.h> // for PathFindOnPathExW\n#include "../../win32/shell32/shresdef.h" // for IDS_NEWITEMFORMAT and IDS_LNK_FILE'
    ALLOW_FILES=( --allowlist-file '.*[/\\]\.\.[/\\]\.\.[/\\]win32[/\\]shell32[/\\]shresdef\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]config\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]intshcut\.h'
                  --allowlist-file '.*[/\\]msi\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]shlwapi_undoc\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*appwiz[/\\].*\.h'
                  --allowlist-file '.*[/\\]objidl\.h'
                  --allowlist-file '.*[/\\]shobjidl\.h'
                  --allowlist-file '.*[/\\]urlmon\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/appwiz" -I"${SRC}/build-i386/dll/cpl/appwiz" -I"${SRC}/sdk/include/wine" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_CRT_DECLARE_NONSTDC_NAMES=1 -DUNICODE -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -D__WINESRC__ -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  timedate)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/timedate/CMakeLists.txt
    # (6 compiled TU(s), 92 headers in the include closure, 65 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 94 external prototype(s) declared by the include closure, 2 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/timedate"
    WRAPPER_INC=$'#include "timedate.h"\n#include <math.h>\n#include <stdlib.h>\n#include <windowsx.h>\n#include <tzlib.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]tzlib\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*timedate[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/timedate" -I"${SRC}/build-i386/dll/cpl/timedate" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/lib/tzlib" -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  powercfg)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/powercfg/CMakeLists.txt
    # (6 compiled TU(s), 206 headers in the include closure, 39 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 53 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/powercfg"
    WRAPPER_INC=$'#include "powercfg.h"\n#include <winreg.h>\n#include <regstr.h>\n#include <debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]batclass\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]devguid\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]powrprof\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*powercfg[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/powercfg" -I"${SRC}/build-i386/dll/cpl/powercfg" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  joy)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/joy/CMakeLists.txt
    # (1 compiled TU(s), 143 headers in the include closure, 17 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 11 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/joy"
    WRAPPER_INC=$'#include "joy.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*joy[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/joy" -I"${SRC}/build-i386/dll/cpl/joy" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  hotplug)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/hotplug/CMakeLists.txt
    # (3 compiled TU(s), 99 headers in the include closure, 23 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 42 external prototype(s) declared by the include closure, 1 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/hotplug"
    WRAPPER_INC=$'#include "hotplug.h"\n#define NDEBUG\n#include <debug.h>\n#include <strsafe.h>\n#include <initguid.h>\n#include <devguid.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cfgmgr32\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]dbt\.h'
                  --allowlist-file '.*[/\\]devguid\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*hotplug[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/hotplug" -I"${SRC}/build-i386/dll/cpl/hotplug" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  hdwwiz)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/hdwwiz/CMakeLists.txt
    # (1 compiled TU(s), 150 headers in the include closure, 13 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 23 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/hdwwiz"
    WRAPPER_INC=$'#include "resource.h"\n#include "hdwwiz.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cfgmgr32\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]dll[/\\]devmgr[/\\]devmgr\.h'
                  --allowlist-file '.*[/\\]rpc\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*hdwwiz[/\\].*\.h'
                  --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]rpcdce\.h'
                  --allowlist-file '.*[/\\]winuser\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/hdwwiz" -I"${SRC}/build-i386/dll/cpl/hdwwiz" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  access)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/access/CMakeLists.txt
    # (6 compiled TU(s), 82 headers in the include closure, 74 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 26 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/access"
    WRAPPER_INC=$'#include "access.h"\n#include <cpl.h>\n#include <wingdi.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*access[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/access" -I"${SRC}/build-i386/dll/cpl/access" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  inetcpl)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/inetcpl/CMakeLists.txt
    # (5 compiled TU(s), 164 headers in the include closure, 33 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 45 external prototype(s) declared by the include closure, 8 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/inetcpl"
    WRAPPER_INC=$'#include "precomp.h"\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <winnls.h>\n#include <wininet.h>\n#include <winuser.h>\n#include <winreg.h>\n#include "inetcpl.h"\n#include "wine/debug.h"\n#include "wine/heap.h"\n#include <cryptuiapi.h>\n#include <shlwapi.h>\n#include <prsht.h>\n#include <shlobj.h>\n#define NONAMELESSUNION\n#define COBJMACROS\n#define CONST_VTABLE\n#include <wingdi.h>\n#include <commctrl.h>\n#include <cpl.h>\n#include "ole2.h"\n#include "commctrl.h"\n#include "urlmon.h"\n#include "initguid.h"\n#include "winreg.h"\n#include "shlwapi.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]cryptuiapi\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]ole2\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]urlmon\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]wininet\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*inetcpl[/\\].*\.h'
                  --allowlist-file '.*[/\\]servprov\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/inetcpl" -I"${SRC}/build-i386/dll/cpl/inetcpl" -I"${SRC}/sdk/include/wine" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -D__WINESRC__ -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh inetcpl
  #   TYPES_STEM=types_1 gen_driver_types.sh inetcpl-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).
  ncpa)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/ncpa/CMakeLists.txt
    # (1 compiled TU(s), 69 headers in the include closure, 5 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 2 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/ncpa"
    WRAPPER_INC=$'#define WIN32_NO_STATUS\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <winuser.h>\n#include <shellapi.h>\n#include <cpl.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cpl\.h' --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*ncpa[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/ncpa" -I"${SRC}/build-i386/dll/cpl/ncpa" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  telephon)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/telephon/CMakeLists.txt
    # (1 compiled TU(s), 67 headers in the include closure, 4 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 3 external prototype(s) declared by the include closure, 1 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/telephon"
    WRAPPER_INC=$'#define WIN32_NO_STATUS\n#include <stdarg.h>\n#include <windef.h>\n#include <winbase.h>\n#include <cpl.h>\n#include "resource.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]cpl\.h' --allowlist-file '.*telephon[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/telephon" -I"${SRC}/build-i386/dll/cpl/telephon" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  odbccp32i)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/odbccp32/CMakeLists.txt
    # (1 compiled TU(s), 78 headers in the include closure, 4 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 5 external prototype(s) declared by the include closure, 1 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/odbccp32"
    WRAPPER_INC=$'#include "odbccp32.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*odbccp32[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/odbccp32" -I"${SRC}/build-i386/dll/cpl/odbccp32" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  openglcfg)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/openglcfg/CMakeLists.txt
    # (3 compiled TU(s), 81 headers in the include closure, 16 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 13 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/openglcfg"
    WRAPPER_INC=$'#include "openglcfg.h"\n#include <cpl.h>\n#include <winreg.h>\n#include <debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*openglcfg[/\\].*\.h'
                  --allowlist-file '.*[/\\]prsht\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/openglcfg" -I"${SRC}/build-i386/dll/cpl/openglcfg" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  wined3dcfg)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/wined3dcfg/CMakeLists.txt
    # (3 compiled TU(s), 75 headers in the include closure, 10 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 10 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/wined3dcfg"
    WRAPPER_INC=$'#include "wined3dcfg.h"\n#include <cpl.h>\n#include <winreg.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*wined3dcfg[/\\].*\.h'
                  --allowlist-file '.*[/\\]wined3dcfg\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/wined3dcfg" -I"${SRC}/build-i386/dll/cpl/wined3dcfg" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh inetcpl
  #   TYPES_STEM=types_1 gen_driver_types.sh inetcpl-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).
  console)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/console/CMakeLists.txt
    # (6 compiled TU(s), 94 headers in the include closure, 50 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 77 external prototype(s) declared by the include closure, 3 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/console"
    WRAPPER_INC=$'#include "console.h"\n#define NDEBUG\n#include <debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]concfg\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*console[/\\].*\.h' )
    EXTRA_INC=( -I"${SRC}/dll/cpl/console" -I"${SRC}/build-i386/dll/cpl/console" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/win32ss/user/winsrv/concfg" -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  input)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/input/CMakeLists.txt
    # (10 compiled TU(s), 99 headers in the include closure, 69 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 79 external prototype(s) declared by the include closure, 2 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/input"
    WRAPPER_INC=$'#include "input.h"\n#include "input_list.h"\n#include "layout_list.h"\n#include "locale_list.h"'
    ALLOW_FILES=( --allowlist-file '.*[/\\]\.\.[/\\]\.\.[/\\]\.\.[/\\]base[/\\]setup[/\\]lib[/\\]muifonts\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]imm32_undoc\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*input[/\\].*\.h'
                  --allowlist-file '.*[/\\]input\.h'
                  --allowlist-file '.*[/\\]input_list\.h'
                  --allowlist-file '.*[/\\]layout_list\.h'
                  --allowlist-file '.*[/\\]locale_list\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/input" -I"${SRC}/build-i386/dll/cpl/input" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  main)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/main/CMakeLists.txt
    # (3 compiled TU(s), 167 headers in the include closure, 46 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 46 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/main"
    WRAPPER_INC=$'#include "main.h"\n#include <commdlg.h>\n#include <cplext.h>\n#include <tchar.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]commdlg\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]cplext\.h'
                  --allowlist-file '.*[/\\]devguid\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]regstr\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*main[/\\].*\.h'
                  --allowlist-file '.*[/\\]main\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/main" -I"${SRC}/build-i386/dll/cpl/main" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  mmsys)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/mmsys/CMakeLists.txt
    # (6 compiled TU(s), 155 headers in the include closure, 45 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 94 external prototype(s) declared by the include closure, 2 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/mmsys"
    WRAPPER_INC=$'#include "mmsys.h"\n#include <winsvc.h>\n#include <shlwapi.h>\n#include <debug.h>\n#include <swenum.h>\n#include <newdev.h>\n#include <initguid.h>\n#include <devguid.h>\n#include <commdlg.h>\n#include <windowsx.h>\n#include <shellapi.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]commdlg\.h'
                  --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]devguid\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]mmsystem\.h'
                  --allowlist-file '.*[/\\]newdev\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]setupapi\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]swenum\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*[/\\]winsvc\.h'
                  --allowlist-file '.*mmsys[/\\].*\.h'
                  --allowlist-file '.*[/\\]mmsys\.h'
                  --allowlist-file '.*[/\\]wtypes\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/mmsys" -I"${SRC}/build-i386/dll/cpl/mmsys" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  usrmgr)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/cpl/usrmgr/CMakeLists.txt
    # (7 compiled TU(s), 92 headers in the include closure, 60 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 45 external prototype(s) declared by the include closure, 0 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/cpl/usrmgr"
    WRAPPER_INC=$'#include "usrmgr.h"\n#include <winreg.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]cpl\.h'
                  --allowlist-file '.*[/\\]lmaccess\.h'
                  --allowlist-file '.*[/\\]lmapibuf\.h'
                  --allowlist-file '.*[/\\]lmerr\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]psdk[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*usrmgr[/\\].*\.h'
                  --allowlist-file '.*[/\\]usrmgr\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/dll/cpl/usrmgr" -I"${SRC}/build-i386/dll/cpl/usrmgr" -I"${SRC}/sdk/include/wine" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh inetcpl
  #   TYPES_STEM=types_1 gen_driver_types.sh inetcpl-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh inetcpl
  #   TYPES_STEM=types_1 gen_driver_types.sh inetcpl-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh inetcpl
  #   TYPES_STEM=types_1 gen_driver_types.sh inetcpl-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).

  # MULTI-ISLAND: these TUs do NOT share one legal include set (§9.108), so
  # generate one pass per group — all into the same crate:
  #   gen_driver_types.sh inetcpl
  #   TYPES_STEM=types_1 gen_driver_types.sh inetcpl-types_1
  # Then dispatch them from the crate: one `mod <stem>;` per island, and each
  # translated module imports from the island its own C include set belongs to
  # (names legitimately overlap between islands — never glob both into scope).
  comctl32)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-08 from dll/win32/comctl32/CMakeLists.txt
    # (39 compiled TU(s), 170 headers in the include closure, 1098 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 396 external prototype(s) declared by the include closure, 76 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/dll/win32/comctl32"
    WRAPPER_INC=$'#include "precomp.h"\n#include <stdarg.h>\n#include <string.h>\n#include "windef.h"\n#include "winbase.h"\n#include "wingdi.h"\n#include "winuser.h"\n#include "winnls.h"\n#include "commctrl.h"\n#include "vfw.h"\n#include "mmsystem.h"\n#include "comctl32.h"\n#include "wine/debug.h"\n#include "wine/heap.h"\n#include <stdlib.h>\n#define OEMRESOURCE\n#include "uxtheme.h"\n#include "vssym32.h"\n#include <ctype.h>\n#include <limits.h>\n#define COBJMACROS\n#define NONAMELESSUNION\n#include "winreg.h"\n#include "objbase.h"\n#include "winerror.h"\n#define NO_SHLWAPI_STREAM\n#include "shlwapi.h"\n#include <math.h>\n#include <stdio.h>\n#include "imm.h"\n#include <immdev.h>\n#include <imm32_undoc.h>\n#include "usp10.h"\n#include "vsstyle.h"\n#include "commoncontrols.h"\n#include "wine/exception.h"\n#include <assert.h>\n#include "winnt.h"\n#include "prsht.h"\n#define GLATESTING 0\n#include <stdlib.h> /* atoi */\n#include "wine/list.h"\n#include <windef.h>\n#include <reactos/debug.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]comctl32_undoc\.h' --allowlist-file '.*[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]commoncontrols\.h'
                  --allowlist-file '.*[/\\]imm32_undoc\.h'
                  --allowlist-file '.*[/\\]imm\.h'
                  --allowlist-file '.*[/\\]immdev\.h'
                  --allowlist-file '.*[/\\]mmsystem\.h'
                  --allowlist-file '.*[/\\]objbase\.h'
                  --allowlist-file '.*[/\\]prsht\.h'
                  --allowlist-file '.*[/\\]psdk[/\\]commctrl\.h'
                  --allowlist-file '.*[/\\]reactos[/\\]undocuser\.h'
                  --allowlist-file '.*[/\\]shlwapi\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]usp10\.h'
                  --allowlist-file '.*[/\\]uxtheme\.h'
                  --allowlist-file '.*[/\\]vfw\.h'
                  --allowlist-file '.*[/\\]vsstyle\.h'
                  --allowlist-file '.*[/\\]vssym32\.h'
                  --allowlist-file '.*[/\\]windowsx\.h'
                  --allowlist-file '.*[/\\]winerror\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*comctl32[/\\].*\.h'
                  --allowlist-file '.*[/\\]wtypes\.h'
                  # objidl.h whole is ~3x the island; imagelist.c's ImageList_Read/Write only
                  # call IStream_Read/Write/Seek through the vtable.
                  --allowlist-type 'I(Sequential)?Stream(Vtbl)?'
                  # rebar.c's SystemParametersInfoW(SPI_GETNONCLIENTMETRICS); winuser.h is not
                  # allowlisted, and the c2rust alias named a struct nothing defined.
                  --allowlist-type '(tag)?NONCLIENTMETRICSW' )
    EXTRA_INC=( -I"${SRC}/dll/win32/comctl32" -I"${SRC}/build-i386/dll/win32/comctl32" -I"${SRC}/sdk/include/wine" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x600 -D_WIN32_WINNT=0x600 -DNTDDI_VERSION=0x06000000 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_WINE -D__ROS_LONG64__ -D_COMCTL32_ -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -D__WINESRC__ -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  ndis)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-09 from drivers/network/ndis/CMakeLists.txt
    # (21 compiled TU(s), 86 headers in the include closure, 27 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=0
    REAL_UNIONS=1
    DRIVER_DIR="${SRC}/drivers/network/ndis"
    WRAPPER_INC=$'#include "ndissys.h"\n#include <afilter.h>\n#include <atm.h>\n#include <ndissys.h>\n#include <ndistapi.h>\n#include <ntifs.h>\n#include <ntdef.h>\n#include <initguid.h>\n#include <ioevent.h>\n#include <wdmguid.h>\n#include <ndisguid.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]afilter\.h' --allowlist-file '.*[/\\]atm\.h'
                  --allowlist-file '.*[/\\]initguid\.h'
                  --allowlist-file '.*[/\\]ioevent\.h'
                  --allowlist-file '.*[/\\]ndis\.h'
                  --allowlist-file '.*[/\\]ndisguid\.h'
                  --allowlist-file '.*[/\\]ndistapi\.h'
                  --allowlist-file '.*[/\\]wdmguid\.h'
                  --allowlist-file '.*ndis[/\\].*\.h'
                  --allowlist-file '.*[/\\]buffer\.h'
                  --allowlist-file '.*[/\\]efilter\.h'
                  --allowlist-file '.*[/\\]guiddef\.h'
                  --allowlist-file '.*[/\\]miniport\.h'
                  --allowlist-file '.*[/\\]ntddk\.h'
                  --allowlist-file '.*[/\\]ntddndis\.h'
                  --allowlist-file '.*[/\\]ntdef\.h'
                  --allowlist-file '.*[/\\]protocol\.h'
                  --allowlist-file '.*[/\\]wdm\.h' )
    EXTRA_INC=( -I"${SRC}/drivers/network/ndis" -I"${SRC}/drivers/network/ndis/include" -I"${SRC}/build-i386/drivers/network/ndis" -I"${SRC}/drivers/network/ndis/include" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -DNDIS_WRAPPER -DNDIS51 -DNDIS51_MINIPORT -DNDIS_MINIPORT_DRIVER -DNDIS_LEGACY_DRIVER=1 -DNDIS_LEGACY_MINIPORT=1 -DNDIS_LEGACY_PROTOCOL=1 -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  notepad)
    # AUTO-DERIVED by rustconv.profile_derive on 2026-10-09 from base/applications/notepad/CMakeLists.txt
    # (5 compiled TU(s), 165 headers in the include closure, 70 switch constant(s)).
    # WRAPPER_INC mirrors the TUs' real include prologues verbatim (§9.108: if the .c compiles,
    # that exact sequence is bindgen-able). Review the WHY notes in the report.
    USERMODE=1
    REAL_UNIONS=1
    # BLOCKLIST_FUNCS, MEASURED: 77 external prototype(s) declared by the include closure, 6 called but declared nowhere in it.
    # The count does NOT settle the flag — island it both ways and run one cargo check each. See the report.
    BLOCKLIST_FUNCS=1
    DRIVER_DIR="${SRC}/base/applications/notepad"
    WRAPPER_INC=$'#include "notepad.h"\n#include <assert.h>\n#include <commctrl.h>\n#include <strsafe.h>\n#include <shlobj.h>\n#include <winreg.h>'
    ALLOW_FILES=( --allowlist-file '.*[/\\]commctrl\.h' --allowlist-file '.*[/\\]commdlg\.h'
                  --allowlist-file '.*[/\\]shellapi\.h'
                  --allowlist-file '.*[/\\]shlobj\.h'
                  --allowlist-file '.*[/\\]strsafe\.h'
                  --allowlist-file '.*[/\\]wingdi\.h'
                  --allowlist-file '.*[/\\]winnls\.h'
                  --allowlist-file '.*[/\\]winreg\.h'
                  --allowlist-file '.*notepad[/\\].*\.h'
                  --allowlist-file '.*[/\\]notepad\.h' )
    # UNICODE module: the TCHAR family from THIS -DUNICODE compile, not ntdll-sys's.
    UNBLOCK="TCHAR PTCHAR TBYTE PTBYTE LPTSTR LPCTSTR PTSTR PCTSTR LPTCH PTCH"
    EXTRA_INC=( -I"${SRC}/base/applications/notepad" -I"${SRC}/build-i386/base/applications/notepad" )
    # LOWER priority than the standard SDK dirs -- the real compile lists these
    # after them, and hoisting one ahead swaps which header an include resolves to.
    LATE_INC=( -I"${SRC}/sdk/include/winrt" -I"${SRC}/vendor/opengl-headers/include" -I"${SRC}/vendor/libdxg/include" -I"${SRC}/sdk/lib/tdilib" -I"${SRC}/vendor/zlib/include" -I"${SRC}/vendor/libtiff/include" -I"${SRC}/.build-tools/llvm-mingw/lib/clang/19/include" )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 -DNTDDI_VERSION=0x05020400 -D_WIN32_WINDOWS=0x502 -D_WIN32_IE=0x603 -D_DEBUG=1 -DUNICODE -D_CRT_DECLARE_NONSTDC_NAMES=1 -D_CRT_NONSTDC_NO_DEPRECATE -D_DLL -D_UNICODE -D__USE_CRTIMP -DUNICODE -D_UNICODE -D_CRT_NO_POSIX_ERROR_CODES )
    ;;
  *) echo "unknown driver: $DRIVER" >&2; exit 2 ;;
esac

# Output crate dir defaults to main repo; override with OUT_CRATE_DIR for a worktree.
OUT_CRATE="${OUT_CRATE_DIR:-${SRC}/sdk/lib/rust/${DRIVER}-rust/src}"
# What the crate CONTAINS, as opposed to where this run WRITES. They are the same directory
# unless OUT_CRATE_DIR is set — which exists to regenerate into scratch and diff against the
# committed island before touching the tree. Anything that reads the crate (its #[no_mangle]
# set, its hand-written <stem>_extra.rs) must read CRATE_SRC: reading OUT_CRATE found an empty
# scratch dir, and silently produced an island that blocklisted nothing and dropped the extra
# re-export — 1,300+ unresolved S_OK/E_FAIL/HKEY_* in the crate that consumed it.
CRATE_SRC="${SRC}/sdk/lib/rust/${DRIVER}-rust/src"
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
  # wtypes.h's `uCLSSPEC` tagged union reaches every island that pulls in the COM headers, and
  # bindgen emits its two anonymous member STRUCTS without `Copy`. Under REAL_UNIONS=1 a
  # non-Copy field cannot live in a real Rust union, so the island itself fails to compile
  # (E0740 + E0204) over a type the module never names -- desk hit it, then appwiz. Blocklist
  # it whenever the module's own source does not mention it. The pattern reaches the NESTED
  # name too (`..._00000009___WIDL_..._00000008`) and stays anchored at `__WIDL`, so the
  # `_userCLIPFORMAT___WIDL_` family (STGMEDIUM's wire type) is untouched.
  if [ "${REAL_UNIONS:-0}" = 1 ] && ! grep -rqw uCLSSPEC "${DRIVER_DIR:-/nonexistent}" 2>/dev/null; then
    EXTRA_BL_ITEMS+=( '__WIDL_wtypes_generated_name_0000000[89].*' uCLSSPEC )
  fi
  for _it in "${EXTRA_BL_ITEMS[@]-}"; do [ -n "$_it" ] && ITEM_BL+=( --blocklist-item "$_it" ); done
  local FN_BL=()
  if [ "${BLOCKLIST_FUNCS:-0}" = 1 ]; then FN_BL=( --blocklist-function '.*' ); fi
  # BLOCKLIST_FUNCS_FROM_CRATE: emit the header function surface, MINUS every function the
  # crate itself defines. A cohab crate needs both halves and they are not separable by hand:
  # the island must declare what the module IMPORTS (CreateDirectoryW, DPA_Create, Dde*) but
  # must NOT declare what the module DEFINES, or the definition and the declaration collide.
  # The crate's own `#[no_mangle]` set is the exact discriminator, and it stays correct as
  # translation proceeds. The DLL's .spec is NOT: on shell32 it names 438 exports while the
  # crate defines 552 functions — only 271 in common. Filtering by the spec would miss 281
  # internal cross-TU helpers (a clash each) and suppress 167 declarations the crate genuinely
  # imports (an unresolved name each).
  # KEEP_CRATE_FUNCS=1 (env) skips the filter for one run — ONLY into an OUT_CRATE_DIR
  # scratch dir, to read off bindgen's header-exact declaration of functions the crate now
  # defines itself, for the prelude's hand-kept "shell32's OWN exports" block. Writing such an
  # island into the crate would reintroduce every definition/declaration collision.
  if [ "${KEEP_CRATE_FUNCS:-0}" = 1 ] && [ "$OUT_CRATE" = "$CRATE_SRC" ]; then
    echo "KEEP_CRATE_FUNCS=1 refuses to write into the crate itself; set OUT_CRATE_DIR" >&2
    exit 2
  fi
  if [ "${BLOCKLIST_FUNCS_FROM_CRATE:-0}" = 1 ] && [ "${KEEP_CRATE_FUNCS:-0}" != 1 ]; then
    local CRATE_DEFS="/tmp/${DRIVER}_${STEM}_cratedefs.txt"
    # Scan the CRATE, not the output directory. They are the same path by default, but
    # OUT_CRATE_DIR exists precisely to generate somewhere else — to diff a regenerated island
    # against the committed one before touching the tree — and scanning an empty scratch dir
    # found no definitions, so the grep exited 1 and `pipefail` killed the whole run with
    # nothing on stdout but the arch banner. `|| true`: a crate with no #[no_mangle] yet is a
    # legitimate empty list, not a failure.
    { grep -rhA3 '#\[no_mangle\]' "${CRATE_SRC}" --include=*.rs 2>/dev/null || true; } \
      | grep -oE '\bfn [A-Za-z_][A-Za-z0-9_]*' | sed 's/^fn //' | sort -u > "$CRATE_DEFS" || true
    while IFS= read -r _fn; do [ -n "$_fn" ] && FN_BL+=( --blocklist-function "$_fn" ); done < "$CRATE_DEFS"
    echo "  (blocklisting $(wc -l <"$CRATE_DEFS") functions the crate defines itself)"
  fi
  local RAW_EXTRA=()
  for _rl in "${EXTRA_RAW_LINES[@]-}"; do [ -n "$_rl" ] && RAW_EXTRA+=( --raw-line "$_rl" ); done
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
    "${SYS_USE[@]}" "${RAW_EXTRA[@]}" \
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
    -I"${BUILD_DIR}/sdk/include/reactos/mc" -I"${BUILD_DIR}/sdk/include/dxsdk" \
    -I"${SRC}/sdk/include/dxsdk" \
    -I"${SRC}/sdk/lib/pseh/include" "${LATE_INC[@]}" \
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
local = set(re.findall(r"^pub (?:struct|enum|union|type) ([A-Za-z0-9_]+)", s, re.M))
out = []
for ln in s.splitlines(keepends=True):
    m = re.match(r"^pub use self::([A-Za-z0-9_]+) as ([A-Za-z0-9_]+);", ln)
    if m and m.group(1) not in local:
        if m.group(1).startswith("_"):
            continue
        # ndis.h: `typedef KINTERRUPT_MODE NDIS_INTERRUPT_MODE;` -- the ALIAS is the
        # module-specific name (no kernel crate carries it), so keep it as a type alias, which
        # may name a glob-imported item where a re-export may not.
        ln = f"pub type {m.group(2)} = {m.group(1)};\n"
    out.append(ln)
open(final, "w").write("".join(out))
' "$RAW" "$FINAL"
  # C++ drivers (LANG_ARGS has `-x c++`): the driver's OWN functions are declared without
  # `extern "C"`, so bindgen emits them as `_Z`-mangled extern decls. In an all-Rust cohab
  # those are dead AND collide name-for-name with the translated modules under a flat
  # re-export. Strip every `_Z`-mangled extern block (a rare genuinely-mangled kernel import
  # is re-bound with C linkage in the crate foundation). See strip_cpp_mangled_externs.py.
  if printf '%s\n' "${LANG_ARGS[@]}" | grep -q 'c++'; then
    python3 "$(dirname "$0")/strip_cpp_mangled_externs.py" "$FINAL" >/dev/null
    # bindgen also emits an inherent `impl` of thin wrappers per C++ class, each calling the
    # flattened member function. BLOCKLIST_FUNCS_FROM_CRATE removes those declarations (the
    # translation defines them), which leaves the wrappers calling nothing. Bitfield-accessor
    # impls are kept — see the tool.
    python3 "$(dirname "$0")/strip_cpp_method_impls.py" "$FINAL" >/dev/null
  fi
  if [ "${REAL_UNIONS:-0}" = 1 ]; then
    # See REAL_UNIONS at the top. The tool hard-fails if the rewrite would lose a
    # declaration or unbalance braces, so a silent truncation cannot ship.
    python3 "$(dirname "$0")/bindgen_unions_to_real.py" "$FINAL" >/dev/null
  fi
  # ntoskrnl/hal DATA exports (`extern __declspec(dllimport) ULONG InitSafeBootMode;`) come out
  # as plain statics, an undefined symbol at a driver's link. Give them raw-dylib, which is the
  # dllimport, so the translation reads them as the C does. Kernel islands only. See the tool.
  if [ "${USERMODE:-0}" != 1 ]; then
    python3 "$(dirname "$0")/bindgen_data_imports.py" "$FINAL" "$ARCH" "$SRC"
  fi
  # MS anonymous struct members (`CHANNEL_DATA_COMMON;` as a struct's first member) come out
  # of bindgen as opaque padding; restore them by name from the driver's own headers. A no-op
  # for every driver whose headers have none. See the tool.
  python3 "$(dirname "$0")/bindgen_ms_anon_members.py" "$FINAL" "${DRIVER_DIR}"/*.h \
    "${SRC}"/sdk/include/reactos/drivers/ata/*.h "${SRC}"/sdk/include/ddk/ndis.h
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
    #
    # SUPERSEDED, NOT YET RETIRED. normalize_island_abi.py (run once after all three arches)
    # does this correctly: it projects from the i386 island per symbol, so a genuinely-cdecl
    # callback keeps `C`. The blanket sed below cannot make that distinction and promotes
    # every cdecl fn-pointer to stdcall ON i386 — an ABI break in the opposite direction,
    # affecting 32 alias + 2 field types repo-wide.
    #
    # It still runs because retiring it is NOT provably safe from inspection: the sed also
    # catches fn-pointer shapes the projection does not match (it keys on `Option<...>`
    # aliases and struct fields), so dropping it could reintroduce E0308 in the seven crates
    # that opt in. The two compose safely in the meantime — this leaves nothing spelled
    # `stdcall` on i386, so the later projection finds no witness and is a no-op here.
    # To retire: regenerate those seven crates without it, and fix what fails to build.
    sed -i -E 's/extern "stdcall" fn/extern "system" fn/g; s/extern "C" fn/extern "system" fn/g' "$FINAL"
  fi
  echo "  -> ${FINAL}  ($(grep -cE 'pub struct' "$FINAL") structs, $(wc -l <"$FINAL") lines)"
}

run_arch i386  i686-pc-windows-gnu    '-D_M_IX86 -D__i386__ -D_X86_'
# -mcx16 mirrors sdk/cmake/gcc.cmake:340 (every amd64 TU is compiled with it). Without it
# the island is parsed against DIFFERENT feature flags than the code it describes, and that
# is not cosmetic: intrin_x86.h guards its own _InterlockedCompareExchange128 with
# `!HAS_BUILTIN(...)`, which is `__has_builtin`, which answers 0 for that target builtin
# unless cx16 is enabled — so the header defines the function, and libclang then rejects
# the definition as redefining a builtin. amd64 island generation hard-failed on exactly that.
run_arch amd64 x86_64-pc-windows-gnu  '-D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_ -mcx16'
run_arch arm64 aarch64-pc-windows-gnu '-D_WIN64 -D_M_ARM64 -D__aarch64__ -D__arm64__ -D_ARM64_'

# Spell ABI-identical callback slots `extern "system"` across all three islands, projecting
# from the i386 island. This runs HERE, after all three arches, because i386 is the witness:
# amd64/arm64 have one calling convention so bindgen flattens every NTAPI to `extern "C"`
# and the decoration is gone. It cannot live inside run_arch, which sees one arch at a time.
#
# Unconditional, and scoped to the stem this run just wrote. Without it a translation's
# portable `extern "system" fn` will not unify with the island's literal `stdcall`/`C` in a
# fn-POINTER type (E0308), and the standing workaround is to transmute the address — which
# erases the calling convention along with the rest of the type. That is exactly how buslogic
# shipped six SCSI miniport callbacks defined `extern "C"` into slots the port driver
# dereferences as stdcall. Normalizing here makes `Some(f)` assignable directly, so the same
# mistake is a compile error instead of 8 bytes of stack leaked per SRB dispatch.
python3 "${SCRIPT_DIR}/normalize_island_abi.py" --stem "$STEM" --apply "$OUT_CRATE"

# <stem>.rs dispatches to the per-arch module.
cat > "${OUT_CRATE}/${STEM}.rs" <<RS
//! Driver-private struct layouts (exact, bindgen-generated per arch).
//! Generated by tools/rustconv/gen_driver_types.sh. Do not edit by hand.
#[cfg(target_arch = "x86")]      #[path = "${STEM}_i386.rs"]  mod types_arch;
#[cfg(target_arch = "x86_64")]   #[path = "${STEM}_amd64.rs"] mod types_arch;
#[cfg(target_arch = "aarch64")]  #[path = "${STEM}_arm64.rs"] mod types_arch;
pub use types_arch::*;
RS
# A crate may keep hand-written residue in <stem>_extra.rs, re-exported through the same
# glob so `use crate::types::*` still reaches everything. This exists because bindgen
# CANNOT emit two things a translated module still needs, and regenerating the island must
# not eat them: pointer-valued macros (`#define NULL ((void*)0)`, INVALID_HANDLE_VALUE,
# HKEY_CLASSES_ROOT) and cast-expression constants (`((HRESULT)0x80004001L)`) are macro
# TEXT, not C declarations, so there is nothing in the header for bindgen to bind.
# Regenerating does not touch <stem>_extra.rs; crates without one are unaffected.
if [ -f "${CRATE_SRC}/${STEM}_extra.rs" ]; then
  cat >> "${OUT_CRATE}/${STEM}.rs" <<RS
#[path = "${STEM}_extra.rs"] mod types_extra;
pub use types_extra::*;
RS
  echo "=== ${STEM}_extra.rs found — re-exported through ${STEM}.rs ==="
fi
echo "=== wrote ${OUT_CRATE}/${STEM}.rs (per-arch dispatch) ==="
