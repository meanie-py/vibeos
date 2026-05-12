"""Shared paths, model config, token budgets."""

import os
from dataclasses import dataclass, field
from pathlib import Path

# Root paths
REACTOS_ROOT = Path(__file__).resolve().parents[3]  # reactos/
NTOSKRNL_DIR = REACTOS_ROOT / "ntoskrnl"
HAL_DIR = REACTOS_ROOT / "hal"
NTKRNL_SYS_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "ntkrnl-sys" / "src"
RUST_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust"
TOOLS_DIR = Path(__file__).resolve().parents[1]  # tools/rustconv/
TEMPLATES_DIR = TOOLS_DIR / "templates"
DATA_DIR = TOOLS_DIR / "data"

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

HAND_WRITTEN_MODULES = [
    "base.rs", "ke.rs", "mm.rs", "ex.rs", "ps.rs", "ob.rs",
    "rtl.rs", "interlocked.rs",
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
}


def detect_target(c_path: Path) -> Target:
    """Pick a target by sniffing the C path. Falls back to ntoskrnl."""
    p = c_path.resolve()
    posix = p.as_posix()
    if "/hal/halx86" in posix or posix.endswith("/hal") or "/hal/" in posix:
        return TARGETS["hal"]
    return TARGETS["ntoskrnl"]


def hal_cmake_for_c(c_path: Path) -> Path | None:
    """Map an HAL C source path to the *.cmake file that lists it.

    hal/halx86/generic/dma.c → hal/halx86/generic.cmake
    hal/halx86/legacy/bus/bushndlr.c → hal/halx86/legacy.cmake (walks up)
    """
    # Walk up looking for a sibling-named .cmake file
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

# Default model per backend
DEFAULT_MODELS = {
    "claude": CLAUDE_MODEL_SONNET,
    "gemini": GEMINI_MODEL,
}

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
