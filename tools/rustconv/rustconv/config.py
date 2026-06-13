"""Shared paths, model config, token budgets."""

import os
import re
from dataclasses import dataclass, field
from pathlib import Path

# Root paths
REACTOS_ROOT = Path(__file__).resolve().parents[3]  # reactos/

# Load API keys from the repo-root .env (ANTHROPIC_API_KEY / DEEPSEEK_API_KEY).
# rustconv historically relied on these being exported in the shell; load them
# here so `python -m rustconv ...` Just Works from a clean shell. Existing
# environment values win (override=False) so an explicit export still takes
# precedence.
try:
    from dotenv import load_dotenv as _load_dotenv
    _load_dotenv(REACTOS_ROOT / ".env", override=False)
except Exception:
    pass
NTOSKRNL_DIR = REACTOS_ROOT / "ntoskrnl"
HAL_DIR = REACTOS_ROOT / "hal"
# On the llvm-mingw (non-MSVC) build, _WINKD_=FALSE+KDBG=TRUE makes
# drivers/base/CMakeLists.txt build kdcom.dll from kdrosdbg/kdrosdbg.c,
# NOT kdcom/. The Target points here so it validates on the unchanged
# green smoke baseline. See memory feedback_kdcom_is_kdrosdbg.
KDCOM_DIR = REACTOS_ROOT / "drivers" / "base" / "kdrosdbg"
# kdgdb: the GDB-stub KD transport (also builds kdcom.dll, but only under
# _WINKD_=TRUE + GDB=TRUE — never on the default llvm-mingw build, which
# forces _WINKD_=FALSE). Translated for completeness; the kdgdb-rust
# staticlib validates standalone via `cargo build -p kdgdb-rust` (i386+amd64;
# kdgdb has no arm64 path). i386_sup.c/amd64_sup.c are arch-gated.
KDGDB_DIR = REACTOS_ROOT / "drivers" / "base" / "kdgdb"
# Serial kdcom: the classic NS16550 serial KD transport (drivers/base/kdcom,
# distinct from the default kdrosdbg-as-kdcom.dll above). _WINKD_-only, never
# built on llvm-mingw. Uses cportlib (= cport-rust). → kdcom-serial-rust.
KDCOM_SERIAL_DIR = REACTOS_ROOT / "drivers" / "base" / "kdcom"
# kdvm: the VirtualBox KD transport (builds kdvbox.dll, not kdcom.dll).
# _WINKD_-only. Has a tiny KdVmExchange asm (VBox backdoor port I/O). → kdvm-rust.
KDVM_DIR = REACTOS_ROOT / "drivers" / "base" / "kdvm"
BOOTVID_DIR = REACTOS_ROOT / "drivers" / "base" / "bootvid"
FREELDR_DIR = REACTOS_ROOT / "boot" / "freeldr" / "freeldr"
ARM64INPUT_DIR = REACTOS_ROOT / "drivers" / "input" / "arm64input"
KBDCLASS_DIR = REACTOS_ROOT / "drivers" / "input" / "kbdclass"
MOUCLASS_DIR = REACTOS_ROOT / "drivers" / "input" / "mouclass"
I8042PRT_DIR = REACTOS_ROOT / "drivers" / "input" / "i8042prt"
# LiveCD read-stack drivers (boot path: ISO9660 → cdrom → classpnp → cdfs).
CLASSPNP_DIR = REACTOS_ROOT / "drivers" / "storage" / "class" / "classpnp"
CDROM_DIR = REACTOS_ROOT / "drivers" / "storage" / "class" / "cdrom"
CDFS_DIR = REACTOS_ROOT / "drivers" / "filesystems" / "cdfs"
SWENUM_DIR = REACTOS_ROOT / "drivers" / "ksfilter" / "swenum"
USBD_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbd"
HIDPARSER_DIR = REACTOS_ROOT / "sdk" / "lib" / "drivers" / "hidparser"
PL011_DIR = REACTOS_ROOT / "drivers" / "serial" / "pl011"
PROCESSR_DIR = REACTOS_ROOT / "drivers" / "processor" / "processr"
ACPI_DIR = REACTOS_ROOT / "drivers" / "bus" / "acpi"
CSQ_DIR = REACTOS_ROOT / "sdk" / "lib" / "drivers" / "csq"
ARBITER_DIR = REACTOS_ROOT / "sdk" / "lib" / "drivers" / "arbiter"
PCIX_DIR = REACTOS_ROOT / "drivers" / "bus" / "pcix"
DISK_DIR = REACTOS_ROOT / "drivers" / "storage" / "class" / "disk"
PARTMGR_DIR = REACTOS_ROOT / "drivers" / "storage" / "partmgr"
FS_REC_DIR = REACTOS_ROOT / "drivers" / "filesystems" / "fs_rec"
MOUNTMGR_DIR = REACTOS_ROOT / "drivers" / "storage" / "mountmgr"
PCIIDEX_DIR = REACTOS_ROOT / "drivers" / "storage" / "ide" / "pciidex"
SCSIPORT_DIR = REACTOS_ROOT / "drivers" / "storage" / "port" / "scsiport"
NTKRNL_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "ntkrnl-sys" / "src"
RUST_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust"
TOOLS_DIR = Path(__file__).resolve().parents[1]  # tools/rustconv/
TEMPLATES_DIR = TOOLS_DIR / "templates"
DATA_DIR = TOOLS_DIR / "data"

# Durable archive for EVERY paid API response. Output is streamed here as it
# arrives (see sink.py) so killing a translate mid-run never discards work you
# already paid for. NEVER /tmp — this lives in-repo (gitignored) so the copy
# survives reboots, tmp-reaping, and process kills. RUSTCONV_RESPONSES_DIR can
# be overridden via env for tests.
RUSTCONV_RESPONSES_DIR = Path(
    os.environ.get("RUSTCONV_RESPONSES_DIR", str(TOOLS_DIR / ".responses"))
)

# ntkrnl-sys source files
BINDGEN_FILES = {
    "arm64": NTKRNL_SYS_DIR / "bindgen_arm64.rs",
    "amd64": NTKRNL_SYS_DIR / "bindgen_amd64.rs",
    "i386": NTKRNL_SYS_DIR / "bindgen_i386.rs",
}

HAL_BINDGEN_FILES = {
    "amd64": NTKRNL_SYS_DIR / "hal_bindgen_amd64.rs",
    "i386": NTKRNL_SYS_DIR / "hal_bindgen_i386.rs",
}

# win32k subsystem + its dedicated FFI crate (analog of ntkrnl-sys/HAL).
# win32k calls BOTH ntoskrnl APIs (covered by BINDGEN_FILES) AND its own
# internal surface (_WND/_DESKTOP/Int*/Gre*/co_* — covered here).
WIN32SS_DIR = REACTOS_ROOT / "win32ss"
WIN32K_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "win32k-sys" / "src"
WIN32K_BINDGEN_FILES = {
    "arm64": WIN32K_SYS_DIR / "bindgen_arm64.rs",
    "amd64": WIN32K_SYS_DIR / "bindgen_amd64.rs",
    "i386": WIN32K_SYS_DIR / "bindgen_i386.rs",
}


# SMSS→CSRSS user-mode subsystem spine + its dedicated FFI crate
# (ntkrnl-sys analog for *user mode*). These native processes link
# ntdll, not the kernel — ntdll-sys is their primary "ddk" surface
# (Nt*/Zw*/Rtl*/Ldr*/Dbg* + the SM/CSR/BASE subsystem-protocol
# structs). See docs/engineering/smss-csrss-translation-audit.md.
# NOTE: winsrv (win32ss/user/winsrv) is intentionally NOT routed here
# — it stays under the /win32ss/ → win32k rule until the Phase-5
# pause-and-check ownership gate in the audit is resolved.
SMSS_DIR = REACTOS_ROOT / "base" / "system" / "smss"
CSR_DIR = REACTOS_ROOT / "subsystems" / "csr"
BASESRV_DIR = REACTOS_ROOT / "subsystems" / "win" / "basesrv"
# ntdll.dll itself — the user-mode loader/dispatch/rtl-glue body. Same
# user-mode substrate as smss-csrss (ntdll-sys), output to its own
# ntdll-rust crate. See docs/engineering/ntdll-translation-audit.md.
NTDLL_DIR = REACTOS_ROOT / "dll" / "ntdll"
NTDLL_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "ntdll-sys" / "src"
NTDLL_BINDGEN_FILES = {
    "arm64": NTDLL_SYS_DIR / "bindgen_arm64.rs",
    "amd64": NTDLL_SYS_DIR / "bindgen_amd64.rs",
    "i386": NTDLL_SYS_DIR / "bindgen_i386.rs",
}

# winsrv (Phase 5 of the SMSS→CSRSS spine — win32ss/user/winsrv/). Built
# atop ntdll-sys: winsrv-sys re-exports `ntdll_sys::nt` and adds the
# wincon/winuser/wingdi/imm + consrv/usersrv/concfg internal surface.
WINSRV_DIR = REACTOS_ROOT / "win32ss" / "user" / "winsrv"
WINSRV_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "winsrv-sys" / "src"
WINSRV_BINDGEN_FILES = {
    "arm64": WINSRV_SYS_DIR / "bindgen_arm64.rs",
    "amd64": WINSRV_SYS_DIR / "bindgen_amd64.rs",
    "i386": WINSRV_SYS_DIR / "bindgen_i386.rs",
}

# userinit.exe — the post-login shell launcher (winlogon → userinit →
# explorer.exe), last hop before the desktop. Plain user-mode GUI exe (not
# a CSR server): documented user32/gdi32/advapi32/kernel32 surface, no RPC /
# COM / GINA. Reuses the winsrv-sys substrate (a superset that already covers
# its Win32 surface) + the generic user-mode prompt addendum. Output to its
# own userinit-rust crate. See docs/engineering/smss-csrss-translation-audit.md.
USERINIT_DIR = REACTOS_ROOT / "base" / "system" / "userinit"

# winlogon.exe — the logon manager (winlogon → userinit → explorer). GINA-
# independent LEAF files only (setup/environment/shutdown/screensaver/security);
# the GINA core (wlx.c), SAS state machine (sas.c), main (winlogon.c) and RPC
# (rpcserver.c) stay C. Own winlogon-sys substrate (self-contained: Win32 GUI/
# registry/security + winwlx GINA + ndk + the winlogon-private WLSESSION/GINA
# structs, size-asserted). Own winlogon-rust crate. See
# docs/engineering/smss-csrss-translation-audit.md + memory project_winlogon_track.
WINLOGON_DIR = REACTOS_ROOT / "base" / "system" / "winlogon"
WINLOGON_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "winlogon-sys" / "src"
WINLOGON_BINDGEN_FILES = {
    "arm64": WINLOGON_SYS_DIR / "bindgen_arm64.rs",
    "amd64": WINLOGON_SYS_DIR / "bindgen_amd64.rs",
    "i386": WINLOGON_SYS_DIR / "bindgen_i386.rs",
}

HAND_WRITTEN_MODULES = [
    "base.rs", "ke.rs", "mm.rs", "ex.rs", "ps.rs", "ob.rs",
    "rtl.rs", "interlocked.rs",
    # constants.rs (~685 STATUS_*/OBJ_*/ACCESS_* consts) and driver_ddk.rs
    # (plain-driver DDK surface: DO_*/IRP_MJ_*/FILE_DEVICE_*/RTL_REGISTRY_* +
    # IoCompleteRequest/IoCallDriver/DbgPrint wrappers) were previously
    # invisible to the catalog, so the resolver flagged every driver DDK
    # symbol as unresolved and each translated .rs needed a hand-written shim.
    # Scanning them here lets resolve() route those symbols to the right glob.
    "constants.rs", "driver_ddk.rs",
]

# ntoskrnl.spec
SPEC_FILE = NTOSKRNL_DIR / "ntoskrnl.spec"

# Output crate (ntoskrnl-rust by default — HAL uses a different one, see Target)
OUTPUT_CRATE = RUST_DIR / "ntoskrnl-rust"


# ---------------------------------------------------------------------------
# Target abstraction — picks ntoskrnl vs hal output routing, system prompt,
# bindgen scope, etc.  See docs/engineering/rustconv-hal-target.md.
# ---------------------------------------------------------------------------

@dataclass
class Target:
    """Describes one C source root + corresponding Rust output crate.

    Fields that are None / empty for ntoskrnl are HAL-specific output-routing
    hooks (auto-editing CMakeLists.txt, choosing src/x86 vs src/{subsystem},
    etc.).
    """
    name: str                              # "ntoskrnl" or "hal"
    source_dir: Path                       # NTOSKRNL_DIR or HAL_DIR
    output_crate: Path                     # ntoskrnl-rust or hal-rust
    bindgen_files: dict                    # arch → bindgen .rs (mandatory kernel bindings)
    hal_bindgen_files: dict                # arch → hal_bindgen .rs (empty for ntoskrnl)
    spec_file: Path | None                 # ntoskrnl.spec (None for HAL — no spec exports)
    system_prompt: Path                    # templates/system_prompt.md or *_hal.md
    src_subdir: Path | None                # "src/x86" for hal-rust, None for ntoskrnl
    cmake_deps_file: Path | None           # hal/halx86/CMakeLists.txt (DEPENDS list to append)
    cmake_source_files: list = field(default_factory=list)  # generic.cmake etc. to edit
    arches: tuple = ("i386", "amd64", "arm64")  # supported arches
    win32k_bindgen_files: dict = field(default_factory=dict)  # arch → win32k-sys .rs (win32k only)
    winsrv_bindgen_files: dict = field(default_factory=dict)  # arch → winsrv-sys .rs (winsrv only)
    # FFI substrate crate the translated Rust imports from. Default is
    # the kernel crate with per-module paths (ntkrnl_sys::ddk/base/…).
    # A user-mode target (smss-csrss) sets sys_crate="ntdll_sys" +
    # sys_umbrella="nt": every `ntkrnl_sys::<mod>::` is deterministically
    # collapsed to a single `ntdll_sys::nt::*` glob in postprocess
    # (LLM-independent, mirrors win32k enforce_win32k_abi philosophy).
    sys_crate: str = "ntkrnl_sys"
    sys_umbrella: str | None = None
    # Optional target-specific rules appended to the (shared) system
    # prompt — mirrors system_prompt_hal.md but additive, so the big
    # kernel ruleset isn't duplicated. smss-csrss uses it for the
    # user-mode specifics (ntdll_sys::nt umbrella, FORCEINLINE
    # NtCurrent*/_PEB/_TEB, user-mode not kernel-mode).
    system_prompt_addendum: Path | None = None


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
        name="classpnp", source_dir=CLASSPNP_DIR,
        output_crate=RUST_DIR / "classpnp-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
    ),
    "cdrom": Target(
        name="cdrom", source_dir=CDROM_DIR,
        output_crate=RUST_DIR / "cdrom-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
    ),
    "cdfs": Target(
        name="cdfs", source_dir=CDFS_DIR,
        output_crate=RUST_DIR / "cdfs-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
    ),
    # pcix.sys — the full MS-architecture PCI bus driver (drivers/bus/pcix, 11K LOC,
    # arb/intrface/pci subdirs). NOT loaded at boot (pci.sys is the live PNP0A03
    # driver), so link-clean ×3 is the integration bar, not smoke boot. Private
    # structs from gen_driver_types.sh substrate (crate::types); kernel/PnP types
    # from ntkrnl-sys ddk. Links arbiter (Rust) + pciclass.
    "pcix": Target(
        name="pcix", source_dir=PCIX_DIR,
        output_crate=RUST_DIR / "pcix-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
    ),
    # disk.sys — the SCSI disk class driver (drivers/storage/class/disk, 12.6K
    # LOC: data/disk/diskwmi/geometry/pnp.c). WDM class driver built on classpnp
    # (imports ClassInitialize/Class* exports + the storage-DDK/SRB/sense
    # substrate landed with classpnp-rust). Boot-loaded (GenDisk→disk). Private
    # structs from gen_driver_types.sh substrate (crate::types); kernel/PnP/SCSI
    # types from ntkrnl-sys ddk.
    "disk": Target(
        name="disk", source_dir=DISK_DIR,
        output_crate=RUST_DIR / "disk-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
    ),
    # partmgr.sys — the partition manager (drivers/storage/partmgr, ~2.9K LOC, WDM
    # kernelmodedriver). BOOT-loaded storage infra (drive-letter/partition enum),
    # so it IS smoke-exercised (unlike pcix). Private device-extension structs
    # (FDO_EXTENSION/PARTITION_EXTENSION/BASIC_VOLUME_UNIQUE_ID/DISK_GEOMETRY_EX_
    # INTERNAL) from gen_driver_types.sh substrate (crate::types); MOUNTDEV_*/
    # ioevent / disk-geometry / volume types come from the same allowlisted driver
    # headers; kernel/PnP types from ntkrnl-sys ddk.
    "partmgr": Target(
        name="partmgr", source_dir=PARTMGR_DIR,
        output_crate=RUST_DIR / "partmgr-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
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
        name="fs_rec", source_dir=FS_REC_DIR,
        output_crate=RUST_DIR / "fs_rec-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
    ),
    # mountmgr.sys — the mount manager (drivers/storage/mountmgr, 9.4K LOC, 7
    # files: database/device/mountmgr/notify/point/symlink/uniqueid). Boot-start
    # WDM driver (no .spec, DriverEntry-only). Manages drive letters / mount
    # points / volume symlinks via MOUNTDEV/MOUNTMGR ioctls. Private structs from
    # mntmgr.h (gen_driver_types substrate, crate::types); MOUNTMGR_*/MOUNTDEV_*
    # from the mountmgr/mountdev headers. Sibling of partmgr/disk.
    "mountmgr": Target(
        name="mountmgr", source_dir=MOUNTMGR_DIR,
        output_crate=RUST_DIR / "mountmgr-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={}, spec_file=None,
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
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
        name="pciidex", source_dir=PCIIDEX_DIR,
        output_crate=RUST_DIR / "pciidex-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={},
        spec_file=PCIIDEX_DIR / "pciidex.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
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
        name="scsiport", source_dir=SCSIPORT_DIR,
        output_crate=RUST_DIR / "scsiport-rust",
        bindgen_files=BINDGEN_FILES, hal_bindgen_files={},
        spec_file=SCSIPORT_DIR / "scsiport.spec",
        system_prompt=TEMPLATES_DIR / "system_prompt.md", src_subdir=Path("src"),
        cmake_deps_file=None, cmake_source_files=[], arches=("i386", "amd64", "arm64"),
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


def detect_target(c_path: Path) -> Target:
    """Pick a target by sniffing the C path. Falls back to ntoskrnl."""
    p = c_path.resolve()
    posix = p.as_posix()
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
    if ("/base/system/smss" in posix or "/subsystems/csr" in posix
            or "/subsystems/win/basesrv" in posix
            or "/sdk/lib/smlib" in posix):
        return TARGETS["smss-csrss"]
    # userinit.exe (post-login shell launcher). Own userinit-rust crate on the
    # winsrv-sys substrate. Keep before any generic base/system fallthrough.
    if "/base/system/userinit" in posix:
        return TARGETS["userinit"]
    # winlogon.exe (logon manager). Own winlogon-rust crate on the self-contained
    # winlogon-sys substrate. Leaf files only — keep before base/system fallthrough.
    if "/base/system/winlogon" in posix:
        return TARGETS["winlogon"]
    # ntdll.dll body (loader/dispatch/rtl-glue). Its own ntdll-rust crate
    # on the same user-mode substrate. Note: the shared rtl algorithms it
    # links (sdk/lib/rtl/*) are NOT routed here — those are the rtl-rust /
    # rtl-strings crates; only dll/ntdll/'s own .c belong to this target.
    if "/dll/ntdll/" in posix:
        return TARGETS["ntdll"]
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
    if "/drivers/storage/partmgr" in posix:
        return TARGETS["partmgr"]
    if "/drivers/filesystems/fs_rec" in posix:
        return TARGETS["fs_rec"]
    if "/drivers/storage/mountmgr" in posix:
        return TARGETS["mountmgr"]
    if "/drivers/storage/ide/pciidex" in posix:
        return TARGETS["pciidex"]
    if "/drivers/storage/port/scsiport" in posix:
        return TARGETS["scsiport"]
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
        return make_user_dll_target(*dll)
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
    if "dll" not in parts:
        return None
    rest = parts[parts.index("dll") + 1:]
    if not rest:
        return None
    if rest[0] == "win32" and len(rest) >= 2:
        name, sub = rest[1], ("dll", "win32", rest[1])
    else:
        name, sub = rest[0], ("dll", rest[0])
    if not re.fullmatch(r"[A-Za-z0-9_]+", name):
        return None
    return name, REACTOS_ROOT.joinpath(*sub)


def make_user_dll_target(name: str, source_dir: Path) -> "Target":
    """A self-contained user-mode-DLL target (modeled on userinit/winlogon):
    output to sdk/lib/rust/<name>-rust, ntdll_sys substrate (provides the NT/Rtl
    types Wine code uses via winternl; Win32 APIs get forward-declared by the
    mechanical prompt). Used for standalone dll/win32/* DLLs not in TARGETS."""
    return Target(
        name=name,
        source_dir=source_dir,
        output_crate=RUST_DIR / f"{name}-rust",
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
    )


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

# Backend selection: "claude" (default if ANTHROPIC_API_KEY set), "gemini"
def get_backend() -> str:
    if os.environ.get("ANTHROPIC_API_KEY"):
        return "claude"
    return "gemini"

# Claude models
CLAUDE_MODEL_SONNET = "claude-sonnet-4-5-20250929"
CLAUDE_MODEL_HAIKU = "claude-haiku-4-5-20251001"
CLAUDE_MAX_TOKENS = 32768

# Gemini
GEMINI_CMD = "gemini"
GEMINI_MODEL = "gemini-2.5-pro"
GEMINI_MAX_TOKENS = 65536

# DeepSeek (OpenAI-compatible API). deepseek-chat = V3, deepseek-reasoner = R1.
DEEPSEEK_BASE_URL = "https://api.deepseek.com"
DEEPSEEK_MODEL_CHAT = "deepseek-chat"
DEEPSEEK_MODEL_REASONER = "deepseek-reasoner"
# Output-token budget. v4 models are REASONING models: reasoning tokens are
# drawn from this same budget before any code `content` is emitted. At 8192 a
# large unit (e.g. a file-scope type preamble with many structs/COM vtables)
# could have its entire budget consumed by reasoning, yielding EMPTY content —
# which silently wiped the mlang/cabinet_main preambles. A large type preamble
# (mlang's is ~30K C chars of structs/COM vtables) needs reasoning + a big code
# body in one call; 16384 still truncated it. 32768 fits both (DeepSeek reasoner
# allows up to 64K output). The empty/truncation guards in batch_runner turn any
# remaining overflow into a loud, retriable failure (never a silent drop).
DEEPSEEK_MAX_TOKENS = 32768

# Default model per backend
DEFAULT_MODELS = {
    "claude": CLAUDE_MODEL_SONNET,
    "gemini": GEMINI_MODEL,
}

# FULL (synchronous) list price per 1M tokens, USD: (input, output).
# rustconv's `translate` / `submit` / `probe-migrate` are SYNCHRONOUS calls and
# pay THESE prices — NOT the 50%-off batch price. The old estimator quoted only
# batch Sonnet ($1.50/$7.50); when you run `translate -m opus` you actually pay
# the Opus row below, per call, once per function. That mismatch is why a "$1"
# estimate turned into $15. Keep these current via the claude-api skill.
# Verified 2026-06-06 against the claude-api skill model table.
MODEL_PRICING = {
    "claude-opus-4-8":             (5.00, 25.00),
    "claude-opus-4-7":             (5.00, 25.00),
    "claude-opus-4-6":             (5.00, 25.00),
    "claude-opus-4-5-20251101":    (5.00, 25.00),
    "claude-sonnet-4-6":           (3.00, 15.00),
    "claude-sonnet-4-5-20250929":  (3.00, 15.00),
    "claude-haiku-4-5-20251001":   (1.00,  5.00),
    "gemini-2.5-pro":              (1.25, 10.00),
    "gemini-2.5-flash":            (0.30,  2.50),
    # DeepSeek list prices (USD/1M, cache-miss). Server-side context caching
    # makes repeated prefixes far cheaper (cache-hit input ~$0.07/1M), so these
    # are a ceiling for the per-function re-send cost, not the typical bill.
    "deepseek-chat":               (0.27,  1.10),
    "deepseek-reasoner":           (0.55,  2.19),
    # v4 tier (cache-miss list prices; server-side caching makes the real bill far
    # lower on agentic re-sends — ~99% cache hit). Approximate; dashboard is truth.
    "deepseek-v4-pro":             (0.28,  1.10),
    "deepseek-v4-flash":           (0.14,  0.55),
}
# Anthropic Message Batches API runs at 50% of the list prices above. The
# synchronous `translate`/`submit` path does NOT get this discount.
BATCH_DISCOUNT = 0.5


def model_pricing(model: str) -> tuple[float, float]:
    """(input, output) $/1M for a model id. Falls back to Opus (the most
    expensive tier) when unknown — so an unrecognized model OVER-estimates
    rather than lulling you with a lowball."""
    return MODEL_PRICING.get(model, MODEL_PRICING["claude-opus-4-8"])


def cost_usd(model: str, input_tokens: int, output_tokens: int,
             batch: bool = False) -> float:
    """USD cost for a single call. batch=True applies the 50% batch discount."""
    pin, pout = model_pricing(model)
    cost = input_tokens * pin / 1_000_000 + output_tokens * pout / 1_000_000
    return cost * (BATCH_DISCOUNT if batch else 1.0)

# Arch mappings
ARCH_MAP = {
    "arm64": "aarch64",
    "amd64": "x86_64",
    "i386": "i686",
}

RUST_TARGETS = {
    "arm64": "aarch64-pc-windows-gnullvm",
    "amd64": "x86_64-pc-windows-gnu",
    "i386": "i686-pc-windows-gnu",
}

# Toolchain and build paths
LLVM_MINGW = REACTOS_ROOT.parent / "llvm-mingw-20250114-ucrt-ubuntu-20.04-x86_64"
BUILD_DIRS = {
    "i386": REACTOS_ROOT.parent / "build-i386",
    "amd64": REACTOS_ROOT.parent / "build-amd64",
    "arm64": REACTOS_ROOT.parent / "build-arm64",
}
CLANG_TARGETS = {
    "i386": "i686-w64-mingw32",
    "amd64": "x86_64-w64-mingw32",
    "arm64": "aarch64-w64-mingw32",
}
NTOS_CMAKE = NTOSKRNL_DIR / "ntos.cmake"

# C arch macros → Rust cfg
C_ARCH_MACROS = {
    "_M_ARM64": 'target_arch = "aarch64"',
    "_M_AMD64": 'target_arch = "x86_64"',
    "_M_IX86": 'target_arch = "x86"',
    "_M_ARM": 'target_arch = "arm"',
    "_WIN64": 'target_pointer_width = "64"',
}
