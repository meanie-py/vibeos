"""Shared paths, model config, token budgets."""

import os
import re
from dataclasses import dataclass, field
from pathlib import Path

from dotenv import load_dotenv


# Root paths
def _detect_reactos_root() -> Path:
    """The ReactOS tree rustconv operates on.

    Prefer the git worktree of the CURRENT directory, so a `pip install -e`'d
    rustconv (whose module physically lives in ONE checkout) still reads
    sources, writes the output crate, and caches paid responses in the tree you
    invoke it FROM — not wherever the module happens to be installed. This kills
    the silent "fell back to the main checkout" footgun that let a worktree
    `python -m rustconv translate` assemble into the main repo (and park paid
    .responses there).

    Order: $RUSTCONV_REACTOS_ROOT → CWD's git toplevel (if it looks like a
    ReactOS tree carrying this tool) → module-relative (historical behaviour).
    """
    import subprocess

    env = os.environ.get("RUSTCONV_REACTOS_ROOT")
    if env:
        return Path(env).resolve()
    module_root = Path(__file__).resolve().parents[3]
    try:
        top = subprocess.run(
            ["git", "-C", os.getcwd(), "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout.strip()
    except Exception:
        top = ""
    if top:
        cwd_root = Path(top)
        # Only trust CWD if it's actually a ReactOS tree carrying this tool —
        # otherwise an unrelated git repo as CWD would mis-route everything.
        if (cwd_root / "ntoskrnl").is_dir() and (
            cwd_root / "tools" / "rustconv" / "rustconv"
        ).is_dir():
            return cwd_root
    return module_root


REACTOS_ROOT = _detect_reactos_root()  # the tree we operate on (CWD worktree, not install dir)


# Load API keys from .env (ANTHROPIC_API_KEY / DEEPSEEK_API_KEY). .env is
# gitignored, so it lives in the MAIN checkout and is usually ABSENT from a
# worktree. Try, in order (all override=False so a shell export always wins):
#   1. the invoked tree's own .env (a worktree may carry its own keys),
#   2. the MAIN checkout's .env, found via the shared git common-dir (the usual
#      case: worktrees share MAIN's .git, and MAIN holds the gitignored .env),
#   3. the module-install tree's .env (the pip-install-e location).
def _load_env_keys(root: Path) -> None:
    candidates = [root / ".env"]
    import subprocess

    try:
        common = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--git-common-dir"],
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout.strip()
        if common:
            candidates.append((root / common).resolve().parent / ".env")  # .git → checkout → .env
    except Exception:
        pass
    candidates.append(Path(__file__).resolve().parents[3] / ".env")
    seen: set[Path] = set()
    for env in candidates:
        if env not in seen and env.exists():
            seen.add(env)
            load_dotenv(env, override=False)


_load_env_keys(REACTOS_ROOT)
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
# afd.sys — the Ancillary Function Driver: the kernel bridge from usermode
# Winsock (ws2_32 → msafd) down to TDI, which netstack now provides
# (\Device\Tcp|Udp|Ip). The keystone of the netstack→winsock vertical.
AFD_DIR = REACTOS_ROOT / "drivers" / "network" / "afd" / "afd"
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
FASTFAT_DIR = REACTOS_ROOT / "drivers" / "filesystems" / "fastfat"
MOUNTMGR_DIR = REACTOS_ROOT / "drivers" / "storage" / "mountmgr"
PCIIDEX_DIR = REACTOS_ROOT / "drivers" / "storage" / "ide" / "pciidex"
SCSIPORT_DIR = REACTOS_ROOT / "drivers" / "storage" / "port" / "scsiport"
STORPORT_DIR = REACTOS_ROOT / "drivers" / "storage" / "port" / "storport"
STORAHCI_DIR = REACTOS_ROOT / "drivers" / "storage" / "port" / "storahci"
BUSLOGIC_DIR = REACTOS_ROOT / "drivers" / "storage" / "port" / "buslogic"
SERIAL_DIR = REACTOS_ROOT / "drivers" / "serial" / "serial"
RAMDISK_DIR = REACTOS_ROOT / "drivers" / "storage" / "class" / "ramdisk"
FLTMGR_DIR = REACTOS_ROOT / "drivers" / "filters" / "fltmgr"
KSECDD_DIR = REACTOS_ROOT / "drivers" / "crypto" / "ksecdd"
USBSTOR_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbstor"
USBCCGP_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbccgp"
USBHUB_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbhub"
USBEHCI_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbehci"
USBOHCI_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbohci"
USBUHCI_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbuhci"
USBPORT_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbport"
USBAUDIO_DIR = REACTOS_ROOT / "drivers" / "usb" / "usbaudio"
KS_DIR = REACTOS_ROOT / "drivers" / "ksfilter" / "ks"
NTKRNL_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "ntkrnl-sys" / "src"
RUST_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust"
TOOLS_DIR = REACTOS_ROOT / "tools" / "rustconv"  # follows REACTOS_ROOT (invoked tree)
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
# kernel32.dll — the Win32 base API DLL. Per-file Rust cohabitation into the
# cmake-built kernel32.dll (proc.c was the first flip; see project memory).
# Same user-mode substrate as ntdll/smss-csrss (ntdll-sys → ntdll_sys::nt).
KERNEL32_DIR = REACTOS_ROOT / "dll" / "win32" / "kernel32"
# msafd.dll — the AFD Winsock service provider (dll/win32/msafd, ~6.3K LOC:
# dllmain/sndrcv/helpers/event/stubs.c). The next rung up the netstack→winsock
# vertical above afd.sys: ws2_32's WSP* calls land here and become IOCTL_AFD_*
# DeviceIoControls down to afd.sys. Usermode DLL, same ntdll-sys substrate as
# kernel32 (Nt*/Rtl*); winsock2/WSP + AFD shared.h IOCTL types added per-crate.
MSAFD_DIR = REACTOS_ROOT / "dll" / "win32" / "msafd"
# ws2_32.dll — the WinSock 2.2 library (dll/win32/ws2_32, ~14.2K LOC / 35 .c / 187
# spec exports). The TOP of the netstack→winsock vertical, one rung above msafd:
# apps call the public sockets API here and ws2_32 dispatches through its
# transport-provider catalog into msafd's WSP* entry points. ROS-NATIVE (not
# Wine-derived), so the clean shape the pipeline handles best.
#
# The .c files include NOTHING — inc/ws2_32.h is a PCH (add_pch in CMakeLists) that
# pulls windef/winbase/winreg/winnls/winuser + ws2spi + NDK rtlfuncs + pseh2 +
# ws2help + nsp_dns + iptypes + the private inc/ws2_32p.h. The scanner must resolve
# against that PCH, and the object model it defines (TCATALOG/NSCATALOG/WSPROCESS/
# WSTHREAD/WSSOCKET/WSASYNCBLOCK) is bindgen-generated into ws2_32-rust's own `types`
# module by gen_driver_types.sh — imported per-file at integration, never transcribed.
WS2_32_DIR = REACTOS_ROOT / "dll" / "win32" / "ws2_32"
LSASRV_DIR = REACTOS_ROOT / "dll" / "win32" / "lsasrv"
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

# explorer.exe — the desktop shell (taskbar/tray/start-menu/systray). Last
# boot-path component still fully C++. Same user-mode GUI-exe substrate as
# userinit (winsrv-sys Win32 superset, ntdll_sys::nt base). Own explorer-rust
# crate; per-file cohabitation into the cmake-built explorer.exe. Most files are
# ATL/WTL/COM-heavy (CWindowImpl/CComObject) — see
# docs/engineering/explorer-rust-translation-plan.md for the tiered ordering
# (leaf-procedural first, COM monsters last).
SHELL_EXPLORER_DIR = REACTOS_ROOT / "base" / "shell" / "explorer"

# shell32.dll C++/ATL COM classes.  Unlike the generic user-DLL fallback this
# target has a crate-local COM substrate and a dedicated body-only prompt; COM
# interfaces/vtables/factories are generated deterministically by cpp_com.py.
SHELL32_DIR = REACTOS_ROOT / "dll" / "win32" / "shell32"
SHELL32_RUST_SRC = RUST_DIR / "shell32-rust" / "src"
SHELL32_COM_FILES = {
    "arm64": SHELL32_RUST_SRC / "shellcom.rs",
    "amd64": SHELL32_RUST_SRC / "shellcom.rs",
    "i386": SHELL32_RUST_SRC / "shellcom.rs",
}

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
    "base.rs",
    "ke.rs",
    "mm.rs",
    "ex.rs",
    "ps.rs",
    "ob.rs",
    "rtl.rs",
    "interlocked.rs",
    # constants.rs (~685 STATUS_*/OBJ_*/ACCESS_* consts) and driver_ddk.rs
    # (plain-driver DDK surface: DO_*/IRP_MJ_*/FILE_DEVICE_*/RTL_REGISTRY_* +
    # IoCompleteRequest/IoCallDriver/DbgPrint wrappers) were previously
    # invisible to the catalog, so the resolver flagged every driver DDK
    # symbol as unresolved and each translated .rs needed a hand-written shim.
    # Scanning them here lets resolve() route those symbols to the right glob.
    "constants.rs",
    "driver_ddk.rs",
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

    name: str  # "ntoskrnl" or "hal"
    source_dir: Path  # NTOSKRNL_DIR or HAL_DIR
    output_crate: Path  # ntoskrnl-rust or hal-rust
    bindgen_files: dict  # arch → bindgen .rs (mandatory kernel bindings)
    hal_bindgen_files: dict  # arch → hal_bindgen .rs (empty for ntoskrnl)
    spec_file: Path | None  # ntoskrnl.spec (None for HAL — no spec exports)
    system_prompt: Path  # templates/system_prompt.md or *_hal.md
    src_subdir: Path | None  # "src/x86" for hal-rust, None for ntoskrnl
    cmake_deps_file: Path | None  # hal/halx86/CMakeLists.txt (DEPENDS list to append)
    cmake_source_files: list = field(default_factory=list)  # generic.cmake etc. to edit
    arches: tuple = ("i386", "amd64", "arm64")  # supported arches
    win32k_bindgen_files: dict = field(default_factory=dict)  # arch → win32k-sys .rs (win32k only)
    winsrv_bindgen_files: dict = field(default_factory=dict)  # arch → winsrv-sys .rs (winsrv only)
    # A CRATE-LOCAL bindgen island (arch → .rs), as produced by
    # tools/rustconv/gen_driver_types.sh into <crate>/src/types_<arch>.rs. This is the
    # GENERIC form of the win32k_/winsrv_ hooks above: any DLL/driver whose private
    # object model is bindgen-generated rather than shared (ws2_32's TCATALOG/WSSOCKET/
    # NSQUERY, msafd's SOCKET_INFORMATION, the driver fleet's device extensions) points
    # here, and the resolver puts those symbols in the catalog under `crate_module`.
    #
    # WHY IT MATTERS: without this the island's names come back UNRESOLVED, and an
    # unresolved name is not a no-op — the model dutifully invents a forward declaration
    # for it (a guessed struct, a guessed const value), which compiles, links, and is
    # silently wrong. Feeding the island to the resolver instead means the model is TOLD
    # the exact generated type and imports it. Pair with crate_module="types".
    crate_bindgen_files: dict = field(default_factory=dict)
    crate_module: str = "types"  # module path the island is imported from (crate::<crate_module>)
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
    # Name of the crate's stdcall-wrapper macro for COM vtbl methods, if it has
    # one. A C `static … WINAPI` method is __stdcall on i386, and rustc will not
    # unify `extern "system"` with the `extern "stdcall"` fn type bindgen gives
    # the vtbl slot — so each body needs a per-arch cfg pair. With a macro named
    # here, abi_enforce emits one `stdmethod! { … }`; without it, the body is
    # duplicated under two cfgs (correct, just verbose). See abi_enforce.py.
    com_method_macro: str | None = None


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


def make_user_dll_target(name: str, source_dir: Path) -> "Target":
    """A self-contained user-mode-DLL target (modeled on userinit/winlogon):
    output to sdk/lib/rust/<name>-rust, ntdll_sys substrate (provides the NT/Rtl
    types Wine code uses via winternl; Win32 APIs get forward-declared by the
    mechanical prompt). Used for standalone dll/win32/* DLLs not in TARGETS."""
    crate = RUST_DIR / f"{name}-rust"
    return Target(
        name=name,
        source_dir=source_dir,
        output_crate=crate,
        bindgen_files=NTDLL_BINDGEN_FILES,
        hal_bindgen_files={},
        spec_file=_find_dll_spec(name, source_dir),
        system_prompt=TEMPLATES_DIR / "system_prompt.md",
        src_subdir=None,
        cmake_deps_file=None,
        cmake_source_files=[],
        arches=("i386", "amd64", "arm64"),
        sys_crate="ntdll_sys",
        sys_umbrella="nt",
        system_prompt_addendum=TEMPLATES_DIR / "system_prompt_user.md",
        com_method_macro=_detect_com_method_macro(crate),
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
    "claude-opus-4-8": (5.00, 25.00),
    "claude-opus-4-7": (5.00, 25.00),
    "claude-opus-4-6": (5.00, 25.00),
    "claude-opus-4-5-20251101": (5.00, 25.00),
    "claude-sonnet-4-6": (3.00, 15.00),
    "claude-sonnet-4-5-20250929": (3.00, 15.00),
    "claude-haiku-4-5-20251001": (1.00, 5.00),
    "gemini-2.5-pro": (1.25, 10.00),
    "gemini-2.5-flash": (0.30, 2.50),
    # DeepSeek list prices (USD/1M, cache-miss). Server-side context caching
    # makes repeated prefixes far cheaper (cache-hit input ~$0.07/1M), so these
    # are a ceiling for the per-function re-send cost, not the typical bill.
    "deepseek-chat": (0.27, 1.10),
    "deepseek-reasoner": (0.55, 2.19),
    # v4 tier (cache-miss list prices; server-side caching makes the real bill far
    # lower on agentic re-sends — ~99% cache hit). Approximate; dashboard is truth.
    "deepseek-v4-pro": (0.28, 1.10),
    "deepseek-v4-flash": (0.14, 0.55),
}
# Anthropic Message Batches API runs at 50% of the list prices above. The
# synchronous `translate`/`submit` path does NOT get this discount.
BATCH_DISCOUNT = 0.5


def model_pricing(model: str) -> tuple[float, float]:
    """(input, output) $/1M for a model id. Falls back to Opus (the most
    expensive tier) when unknown — so an unrecognized model OVER-estimates
    rather than lulling you with a lowball."""
    return MODEL_PRICING.get(model, MODEL_PRICING["claude-opus-4-8"])


def cost_usd(model: str, input_tokens: int, output_tokens: int, batch: bool = False) -> float:
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
