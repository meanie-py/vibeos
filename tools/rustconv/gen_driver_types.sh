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

DRIVER="${1:?usage: gen_driver_types.sh <acpi|classpnp|cdrom|cdfs|disk|pcix|partmgr>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../.." && pwd)"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
NTKRNL_SYS="${SRC}/sdk/lib/rust/ntkrnl-sys/src"

# --- per-driver config -------------------------------------------------------
# WRAPPER_INC : header(s) the wrapper.h #includes (pulls in the private structs)
# ALLOW_FILES : --allowlist-file regexes (types DEFINED here become local)
# DRIVER_INC  : extra -I for the driver's own dir
# VER_DEFS    : version macros matching the driver's CMakeLists.txt
EXTRA_INC=()
# SYS_CRATES: extra sibling -sys crates whose bindgen types must ALSO be blocklisted
# (kept a single identity) and re-exported. Format: "crate_name:src_subdir". Only ACPI
# bridges two -sys crates (ntkrnl-sys + acpica-sys), so it's the only user today.
SYS_CRATES=()
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
  cdrom)
    # KMDF driver (set_module_type cdrom kmdfdriver) — needs the WDF 1.17 headers.
    DRIVER_DIR="${SRC}/drivers/storage/class/cdrom"
    WRAPPER_INC=$'#include <ntddk.h>\n#include "cdrom.h"'
    ALLOW_FILES=( --allowlist-file '.*cdrom\.h' --allowlist-file '.*cdromp\.h' --allowlist-file '.*ioctl\.h' --allowlist-file '.*mmc\.h' --allowlist-file '.*scratch\.h' )
    VER_DEFS=( -DWINVER=0x603 -D_WIN32_WINNT=0x603 -DNTDDI_VERSION=0x06030000
               -DWDF_DEVICE_NO_WDMSEC_H -DDEBUG_USE_KDPRINT -DKMDF_MAJOR_VERSION_STRING=01 -DKMDF_MINOR_VERSION_STRING=017 )
    EXTRA_INC=( -I"${SRC}/sdk/include/wdf/kmdf/1.17" -I"${SRC}/sdk/include/wdf/shared" -I"${SRC}/sdk/include/wdf" )
    ;;
  cdfs)
    DRIVER_DIR="${SRC}/drivers/filesystems/cdfs"
    WRAPPER_INC='#include "cdprocs.h"'
    ALLOW_FILES=( --allowlist-file '.*cdstruc\.h' --allowlist-file '.*cdprocs\.h' --allowlist-file '.*cdfs\.h' )
    VER_DEFS=( -DWINVER=0x502 -D_WIN32_WINNT=0x502 )
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
  *) echo "unknown driver: $DRIVER" >&2; exit 2 ;;
esac

# Output crate dir defaults to main repo; override with OUT_CRATE_DIR for a worktree.
OUT_CRATE="${OUT_CRATE_DIR:-${SRC}/sdk/lib/rust/${DRIVER}-rust/src}"
mkdir -p "$OUT_CRATE"
WRAPPER="/tmp/${DRIVER}_types_wrapper.h"
printf '%s\n' "$WRAPPER_INC" > "$WRAPPER"

run_arch() {
  local ARCH="$1" TARGET="$2" ARCHDEF="$3" BUILD_DIR="${SRC}/build-${1}"
  local NTK="${NTKRNL_SYS}/bindgen_${ARCH}.rs"
  local RAW="/tmp/${DRIVER}_types_${ARCH}.rs"
  echo "=== ${DRIVER} types: ${ARCH} (${TARGET}) ==="

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
  local TYPELIST="/tmp/${DRIVER}_ntktypes_${ARCH}.txt"
  grep -oE "pub (struct|enum|type|union) [A-Za-z0-9_]+" "$NTK" | awk '{print $3}' | sort -u > "$TYPELIST"
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
    local crate="${spec%%:*}" subdir="${spec##*:}"
    local SYSBG="${SRC}/sdk/lib/rust/${subdir}/bindgen_${ARCH}.rs"
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

  bindgen "$WRAPPER" \
    --use-core --no-layout-tests --no-doc-comments \
    "${ALLOW_FILES[@]}" \
    "${BL[@]}" \
    --raw-line '// @generated by tools/rustconv/gen_driver_types.sh — driver-private structs.' \
    --raw-line '#![allow(non_camel_case_types, non_snake_case, dead_code)]' \
    --raw-line '// kernel field types resolved via private ddk glob (NOT re-exported: modules get' \
    --raw-line '// common kernel types from driver_ddk, private structs from this crate::types).' \
    --raw-line 'use ntkrnl_sys::ddk::*;' \
    "${SYS_USE[@]}" \
    -o "$RAW" \
    -- -ferror-limit=0 -Wno-everything \
    -fms-extensions \
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
  local FINAL="${OUT_CRATE}/types_${ARCH}.rs"
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
  echo "  -> ${FINAL}  ($(grep -cE 'pub struct' "$FINAL") structs, $(wc -l <"$FINAL") lines)"
}

run_arch i386  i686-pc-windows-gnu    '-D_M_IX86 -D__i386__ -D_X86_'
run_arch amd64 x86_64-pc-windows-gnu  '-D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_'
run_arch arm64 aarch64-pc-windows-gnu '-D_WIN64 -D_M_ARM64 -D__aarch64__ -D__arm64__ -D_ARM64_'

# types.rs dispatches to the per-arch module.
cat > "${OUT_CRATE}/types.rs" <<'RS'
//! Driver-private struct layouts (exact, bindgen-generated per arch).
//! Generated by tools/rustconv/gen_driver_types.sh. Do not edit by hand.
#[cfg(target_arch = "x86")]      #[path = "types_i386.rs"]  mod types_arch;
#[cfg(target_arch = "x86_64")]   #[path = "types_amd64.rs"] mod types_arch;
#[cfg(target_arch = "aarch64")]  #[path = "types_arm64.rs"] mod types_arch;
pub use types_arch::*;
RS
echo "=== wrote ${OUT_CRATE}/types.rs (per-arch dispatch) ==="
