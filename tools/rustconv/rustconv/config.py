"""Shared paths, model config, token budgets."""

import os
from pathlib import Path

# Root paths
REACTOS_ROOT = Path(__file__).resolve().parents[3]  # reactos/
NTOSKRNL_DIR = REACTOS_ROOT / "ntoskrnl"
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

HAND_WRITTEN_MODULES = [
    "base.rs", "ke.rs", "mm.rs", "ex.rs", "ps.rs", "ob.rs",
    "rtl.rs", "interlocked.rs",
]

# ntoskrnl.spec
SPEC_FILE = NTOSKRNL_DIR / "ntoskrnl.spec"

# Output crate
OUTPUT_CRATE = RUST_DIR / "ntoskrnl-rust"

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

# C arch macros → Rust cfg
C_ARCH_MACROS = {
    "_M_ARM64": 'target_arch = "aarch64"',
    "_M_AMD64": 'target_arch = "x86_64"',
    "_M_IX86": 'target_arch = "x86"',
    "_M_ARM": 'target_arch = "arm"',
    "_WIN64": 'target_pointer_width = "64"',
}
