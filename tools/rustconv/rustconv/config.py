"""Shared paths, model config, token budgets."""

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

from dotenv import load_dotenv

from . import project as _project_mod

# The project rustconv is operating on, declared by a `rustconv.toml` at or above CWD.
# None when there is no anchor, in which case every consumer below falls back to the
# legacy inference. Resolved ONCE at import so a chdir mid-run cannot move the tree —
# and the paid-response bank — out from under an in-flight translation.
PROJECT = _project_mod.load()


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

    Order: $RUSTCONV_REACTOS_ROOT → a `rustconv.toml` anchor at or above CWD → CWD's git
    toplevel (if it looks like a ReactOS tree carrying this tool) → module-relative
    (historical behaviour).

    The anchor outranks the sniffing because it is a DECLARATION rather than a guess, and
    because the guess stops working the moment rustconv lives outside the tree it
    translates — it looks for a directory named `ntoskrnl`, which no other project has.
    The sniffing stays as a fallback so a checkout without an anchor is unaffected.
    """
    import subprocess

    env = os.environ.get("RUSTCONV_REACTOS_ROOT")
    if env:
        return Path(env).resolve()

    if PROJECT is not None:
        return PROJECT.root

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
    os.environ.get(
        "RUSTCONV_RESPONSES_DIR",
        # A project that declares its bank in rustconv.toml gets exactly that path; the
        # declaration is the point, since this is where the paid work lives. Only an
        # anchorless checkout falls back to the tool-relative default.
        str(PROJECT.responses if PROJECT is not None else TOOLS_DIR / ".responses"),
    )
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
# The per-arch bindgen islands. Distinct from SHELL32_COM_FILES above, which is the
# hand-written vtable-ABI module and is arch-independent by construction. Only these
# carry per-arch type widths, so only these can settle whether an SDK typedef is an
# integer of the same width on every target (cpp_com.island_integer_aliases).
SHELL32_ISLAND_FILES = {
    arch: SHELL32_RUST_SRC / f"island_{arch}.rs" for arch in ("i386", "amd64", "arm64")
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
    # Real inline bodies outrank bindgen's un-linkable FORCEINLINE declarations.
    "inline/tier1.rs",
]

# ntoskrnl.spec
SPEC_FILE = NTOSKRNL_DIR / "ntoskrnl.spec"

# Output crate (ntoskrnl-rust by default — HAL uses a different one, see Target)
OUTPUT_CRATE = RUST_DIR / "ntoskrnl-rust"


# ---------------------------------------------------------------------------
# Target abstraction — picks ntoskrnl vs hal output routing, system prompt,
# bindgen scope, etc.  See docs/engineering/rustconv-hal-target.md.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Island:
    """One generated FFI binding set, scanned into the catalog under ``module``.

    An "island" is a bindgen-generated Rust file carrying the EXACT layouts and constant
    values of some C header set, per arch. The engine only needs three things about one:
    where its files are, what module path its symbols live under, and whether it outranks
    what is already in the catalog.

    ``authoritative`` decides type/constant collisions. A module's own island wins, because
    that module's C is compiled against exactly those headers — its field offsets and
    constant values are the island's, not some other module's same-named symbol. The shared
    base bindings (ntkrnl-sys "ddk") are NOT authoritative: hand-written substrate outranks
    them. Functions are always add-if-absent regardless, so the shared kernel/NT API stays
    the linkable surface and an island only ADDS its private entry points.

    This replaces four hard-coded, near-identical branches (hal_/win32k_/winsrv_/crate_
    bindgen_files) whose merge policies turned out to be character-for-character the same.
    The engine no longer knows the words "hal", "win32k" or "winsrv".
    """

    module: str  # catalog module path: "ddk", "hal_ddk", "win32k", "winsrv", "types", …
    files: dict  # arch → generated .rs
    authoritative: bool = True  # types/constants override; False = add-if-absent


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
    #: True when this crate's SUBSTRATE still spells fn-pointer slots with the arch's literal
    #: convention (`extern "stdcall"` on i386), which is what forces a translated `static`
    #: system fn to be emitted as a per-arch `#[cfg]` twin pair. False once the substrate is
    #: `extern "system"` throughout, where one definition serves all three arches. Detected
    #: from the island, never assumed -- see `_detect_concrete_stdcall_slots`.
    concrete_stdcall_slots: bool = True
    # The C++ CLASS path's two inputs, which used to be hard-coded to shell32 at the call
    # sites in __main__ — so `--cpp-com` could not run for any other module, `cpp_com_mode`
    # was False there, and with it went the class scaffold (vtable instance, _new/_delete,
    # lpVtbl wiring), the interface rendering AND every audit_worker_body call. The failure
    # was silent: shellrecyclebin's four hand-rolled COM classes translated to bodies with no
    # vtable ever populated, which is a wild jump on the first method call, and nine raw C
    # `goto`s shipped past a check that exists and works.
    #
    # com_object_map: the DLL's class-object map (shell32.cpp's OBJECT_ENTRY table). None for
    # a module whose classes are created by plain `*_Constructor` free functions rather than a
    # COM class factory — which is every non-shell32 case so far.
    com_object_map: Path | None = None
    # com_island_files: arch → the crate's OWN bindgen island, which the class analyzer reads
    # for exact layouts. NOT `crate_bindgen_files`: that names the COM ABI module
    # (shell32 → shellcom), a different thing that happens to live in the same crate.
    com_island_files: dict = field(default_factory=dict)
    # Optional per-module override of the C++ CLASS body prompt. None means the shared
    # template, whose worker contract is target-independent — see load_function_system_prompt.
    com_system_prompt: Path | None = None
    # The generic form of every bindgen hook above. A project may set this directly (and a
    # non-ReactOS one should); when left empty it is DERIVED in __post_init__ from the legacy
    # per-subsystem fields, so the 60-entry registry did not have to be rewritten to get the
    # engine off them. Order is significance order — later islands outrank earlier ones.
    islands: tuple = ()

    def __post_init__(self):
        if self.islands:
            return
        derived = [Island("ddk", self.bindgen_files, authoritative=False)]
        for module, files in (
            ("hal_ddk", self.hal_bindgen_files),
            ("win32k", self.win32k_bindgen_files),
            ("winsrv", self.winsrv_bindgen_files),
            (self.crate_module or "types", self.crate_bindgen_files),
        ):
            if files:
                derived.append(Island(module, files))
        self.islands = tuple(derived)


# ---------------------------------------------------------------------------
# The ReactOS project plugin: TARGETS, detect_target, and the crate/substrate
# resolution rules moved to `rustconv.projects.reactos.targets` — ~1,600 lines of
# vibeos DATA that made the engine look far more coupled than it is.
#
# Re-exported lazily (PEP 562) rather than imported at module scope, because the
# plugin imports THIS module for its path constants: a top-level import here would
# be a cycle. By the time anything asks for TARGETS, config is fully initialised.
#
# NB for anyone adding a config-internal caller: module __getattr__ is NOT consulted
# for bare global lookups inside config.py itself. Use _project() explicitly.
# ---------------------------------------------------------------------------

_PROJECT_EXPORTS = frozenset(
    {
        "TARGETS",
        "TargetResolveError",
        "detect_target",
        "hal_cmake_for_c",
        "make_user_dll_target",
        "_detect_sys_umbrella",
        "_derive_sys_substrate",
        "_resolve_output_crate",
        "_dll_crate_from_path",
        "_find_dll_spec",
        "_detect_com_method_macro",
        "_derive_com_islands",
        "_resolve_sys_name",
        "_user_prompt_addendum",
    }
)


if TYPE_CHECKING:
    # Runtime resolution goes through __getattr__ below, which a type checker cannot see
    # through — it erased the type of every re-exported name, so `TARGETS["ntoskrnl"]`
    # inferred as None and each use became a spurious optional-access error downstream.
    # Re-exported explicitly here (the `X as X` form marks it a deliberate re-export) so
    # checkers get the real types while imports still resolve lazily at runtime.
    from .projects.reactos.targets import (
        TARGETS as TARGETS,
    )
    from .projects.reactos.targets import (
        TargetResolveError as TargetResolveError,
    )
    from .projects.reactos.targets import (
        detect_target as detect_target,
    )
    from .projects.reactos.targets import (
        hal_cmake_for_c as hal_cmake_for_c,
    )
    from .projects.reactos.targets import (
        make_user_dll_target as make_user_dll_target,
    )


def _project():
    from .projects.reactos import targets as _targets

    return _targets


def __getattr__(name: str):
    if name in _PROJECT_EXPORTS:
        return getattr(_project(), name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__():
    return sorted(set(globals()) | _PROJECT_EXPORTS)


def get_backend() -> str:
    if os.environ.get("DEEPSEEK_API_KEY"):
        return "deepseek"
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
# The provisioned key serves the v4 tier; pro is the translation default (flash is the
# cheaper sibling, not a quality peer). Kept in sync with MODEL_ALIASES["deepseek"].
DEEPSEEK_MODEL_V4_PRO = "deepseek-v4-pro"
# Output-token budget. v4 models are REASONING models: reasoning tokens are
# drawn from this same budget before any code `content` is emitted. At 8192 a
# large unit (e.g. a file-scope type preamble with many structs/COM vtables)
# could have its entire budget consumed by reasoning, yielding EMPTY content —
# which silently wiped the mlang/cabinet_main preambles. A large type preamble
# (mlang's is ~30K C chars of structs/COM vtables) needs reasoning + a big code
# body in one call; 16384 still truncated it. 32768 fits both (DeepSeek reasoner
# allows up to 64K output). The empty/truncation guards in batch_runner turn any
# remaining overflow into a loud, retriable failure (never a silent drop).
# 2026-07-18: 32768 was STILL not enough — the iphlpapi full-flip pass lost 3 file
# preambles (media/icmp/address) and 2 large fns (IcmpParseReplies,
# GetInterfacesList) to exactly this mode: finish_reason=stop with EMPTY content
# and a 0-byte .partial, reasoning having eaten the whole budget before any code
# was emitted. Raised to the reasoner's documented 64K ceiling. This is a CAP, not
# a target — you are billed for tokens actually generated, so raising it costs
# nothing on calls that already fit and simply stops truncating the ones that
# don't. Override with DEEPSEEK_MAX_TOKENS for a one-off.
DEEPSEEK_MAX_TOKENS = int(os.environ.get("DEEPSEEK_MAX_TOKENS", "65536"))

# Default model per backend
DEFAULT_MODELS = {
    "deepseek": DEEPSEEK_MODEL_V4_PRO,
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
