#!/usr/bin/env python3
"""Regenerate `ntoskrnl.rc.res` (PE-COFF resource section) per arch.

Out-of-band replacement for the build-time `compile_resource()` helper in
`sdk/lib/rust/ntoskrnl-cargo/build.rs`. Same pipeline (windres on
`ntoskrnl/ntoskrnl.rc` with the kernel `-I` chain), but driven from
Python so the build doesn't need `ntoskrnl`, `sdk/include/psdk`, and
`sdk/include/vcruntime` on `KERNEL_INCLUDES`.

Output is checked in to `sdk/lib/rust/ntoskrnl-cargo/data/ntoskrnl-<arch>.rc.res`.
Re-run when the version blocks (`common.ver`, `ntverp.h`), the bootanim
resource set, or the bugcode message table (`bugcodes.mc`, or how the build
turns windmc output into resources) change. `scan_res_named_types` in
`vibeos-cli check` catches the one staleness that broke every lookup: a
MESSAGETABLE stored under a string type name instead of RT id 11.

Output is byte-identical to cmake's
`build-<arch>/ntoskrnl/CMakeFiles/ntoskrnl.dir/ntoskrnl.rc.res`.

Usage:
    python3 tools/gen_ntoskrnl_res.py --arch amd64          # Regenerate + write
    python3 tools/gen_ntoskrnl_res.py --arch i386 --check   # Verify checked-in
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"
RC_SOURCE = REPO / "ntoskrnl/ntoskrnl.rc"

# Per-arch: llvm-mingw triple + the `_M_*`/`_X_`/`__arch__` resource defines.
# The arch define sets mirror reactos-link's {I386,AMD64,ARM64}_ARCH_DEFINES
# (i386 additionally gets _USE_32BIT_TIME_T, exactly like the kernel build).
ARCHES = {
    "i386": {
        "triple": "i686-w64-mingw32",
        "defines": [
            ("_M_IX86", None),
            ("_X86_", None),
            ("__i386__", None),
            ("i386", None),
            ("_USE_32BIT_TIME_T", None),
        ],
    },
    "amd64": {
        "triple": "x86_64-w64-mingw32",
        "defines": [
            ("_M_AMD64", None),
            ("_AMD64_", None),
            ("_WIN64", None),
            ("__x86_64__", None),
        ],
    },
    "arm64": {
        "triple": "aarch64-w64-mingw32",
        "defines": [
            ("_M_ARM64", None),
            ("_ARM64_", None),
            ("_WIN64", None),
            ("__aarch64__", None),
        ],
    },
}

# Arch-agnostic kernel resource defines (mirror KERNEL_DEFINES_COMMON in the
# reactos-link engine; the `_M_*`/`__arch__` set is added per-arch above).
KERNEL_DEFINES_COMMON = [
    ("D3D_UMD_INTERFACE_VERSION", "0x000C"),
    ("DBG", "1"),
    ("DLL_EXPORT_VERSION", "0x502"),
    ("DXGKDDI_INTERFACE_VERSION", "0x1052"),
    ("KDBG", None),
    ("MINGW_HAS_SECURE_API", "1"),
    ("NTDDI_VERSION", "0x05020400"),
    ("NTKRNLVISTA", None),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", "0x502"),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTOSKRNL_", None),
    ("_NTSYSTEM_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", "0x502"),
    ("_USE_DUMMY_PSEH", "1"),
    ("_WIN32_IE", "0x603"),
    ("_WIN32_WINDOWS", "0x502"),
    ("_WIN32_WINNT", "0x502"),
    ("__REACTOS__", None),
    ("__ROS_ROSSYM__", None),
]

# Mirrors LANGUAGES in build.rs (136 LANGUAGE_* defines required by
# common.ver to emit every language's StringFileInfo block).
LANGUAGES = [
    "AF_ZA",
    "AR_AE",
    "AR_BH",
    "AR_DZ",
    "AR_EG",
    "AR_IQ",
    "AR_JO",
    "AR_KW",
    "AR_LB",
    "AR_LY",
    "AR_MA",
    "AR_OM",
    "AR_QA",
    "AR_SA",
    "AR_SY",
    "AR_TN",
    "AR_YE",
    "AZ_AZ",
    "BE_BY",
    "BG_BG",
    "BN_BD",
    "BN_IN",
    "CA_ES",
    "CS_CZ",
    "DA_DK",
    "DE_AT",
    "DE_CH",
    "DE_DE",
    "DE_LI",
    "DE_LU",
    "DV_MV",
    "EL_GR",
    "EN_AU",
    "EN_BZ",
    "EN_CA",
    "EN_CB",
    "EN_GB",
    "EN_IE",
    "EN_JM",
    "EN_NZ",
    "EN_PH",
    "EN_TT",
    "EN_US",
    "EN_ZA",
    "EN_ZW",
    "EO_AA",
    "ES_AR",
    "ES_BO",
    "ES_CL",
    "ES_CO",
    "ES_CR",
    "ES_DO",
    "ES_EC",
    "ES_ES",
    "ES_GT",
    "ES_HN",
    "ES_MX",
    "ES_NI",
    "ES_PA",
    "ES_PE",
    "ES_PR",
    "ES_PY",
    "ES_SV",
    "ES_UY",
    "ES_VE",
    "ET_EE",
    "EU_ES",
    "FA_IR",
    "FI_FI",
    "FO_FO",
    "FR_BE",
    "FR_CA",
    "FR_CH",
    "FR_FR",
    "FR_LU",
    "FR_MC",
    "GL_ES",
    "GU_IN",
    "HE_IL",
    "HI_IN",
    "HR_HR",
    "HU_HU",
    "HY_AM",
    "ID_ID",
    "IS_IS",
    "IT_CH",
    "IT_IT",
    "JA_JP",
    "KA_GE",
    "KK_KZ",
    "KN_IN",
    "KOK_IN",
    "KO_KR",
    "KY_KG",
    "LT_LT",
    "LV_LV",
    "MK_MK",
    "MN_MN",
    "MR_IN",
    "MS_BN",
    "MS_MY",
    "MY_MM",
    "NL_BE",
    "NL_NL",
    "NN_NO",
    "PA_IN",
    "PL_PL",
    "PT_BR",
    "PT_PT",
    "RM_CH",
    "RO_RO",
    "RU_RU",
    "SA_IN",
    "SK_SK",
    "SL_SI",
    "SQ_AL",
    "SR_SP",
    "SV_FI",
    "SV_SE",
    "SW_KE",
    "SYR_SY",
    "TA_IN",
    "TE_IN",
    "TH_TH",
    "TR_TR",
    "TT_TA",
    "UK_UA",
    "UR_PK",
    "UZ_UZ",
    "VI_VN",
    "WA_BE",
    "ZH_CN",
    "ZH_HK",
    "ZH_MO",
    "ZH_SG",
    "ZH_TW",
]

KERNEL_INCLUDES = [
    "ntoskrnl",
    "sdk/include/crt",
    "sdk/include/psdk",
    "sdk/include/vcruntime",
    "ntoskrnl/include",
]


def regenerate(
    work: Path, arch: str, triple: str, clang: Path, windres: Path, bugcodes_rc: Path, defines: list
) -> bytes:
    # Stage the cmake-built bugcodes.rc + its .bin sidecars next to a
    # symlink/copy so windres can resolve `<bugcodes.rc>` (and the .bin
    # files referenced from it via relative paths).
    mc_stage = work / "mc-stage"
    mc_stage.mkdir()
    bugcodes_dir = bugcodes_rc.parent
    for f in bugcodes_dir.glob("bugcodes*"):
        if f.is_file():
            shutil.copy2(f, mc_stage / f.name)

    out_path = work / "ntoskrnl.rc.res"
    cmd = [str(windres), "-O", "coff"]
    for inc in KERNEL_INCLUDES:
        cmd.extend(["-I", str(REPO / inc)])
    cmd.extend(["-I", str(mc_stage)])
    cmd.extend(
        [
            f"--preprocessor={clang}",
            f"--preprocessor-arg=--target={triple}",
            "--preprocessor-arg=-E",
            "--preprocessor-arg=-nostdinc",
            "--preprocessor-arg=-xc-header",
            "-DRC_INVOKED",
            "-D__WIN32__=1",
            "-D__FLAT__=1",
        ]
    )
    for lang in LANGUAGES:
        cmd.append(f"-DLANGUAGE_{lang}")
    for name, value in KERNEL_DEFINES_COMMON + defines:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    cmd.extend([str(RC_SOURCE), str(out_path)])

    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"windres failed (exit {proc.returncode})")
    return out_path.read_bytes()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--arch", choices=sorted(ARCHES), default="i386", help="target arch (default: i386)"
    )
    ap.add_argument(
        "--check", action="store_true", help="verify checked-in file matches a fresh regen"
    )
    args = ap.parse_args()

    cfg = ARCHES[args.arch]
    triple = cfg["triple"]
    clang = LLVM_MINGW / f"bin/{triple}-clang"
    windres = LLVM_MINGW / f"bin/{triple}-windres"
    out_file = REPO / f"sdk/lib/rust/ntoskrnl-cargo/data/ntoskrnl-{args.arch}.rc.res"
    # `bugcodes.rc` is generated at cmake-build time; borrow it (and its .bin
    # sidecars) from `build-<arch>/sdk/include/reactos/mc/`.
    bugcodes_rc = REPO / f"build-{args.arch}/sdk/include/reactos/mc/bugcodes.rc"

    for p in [windres, clang, RC_SOURCE, bugcodes_rc]:
        if not p.exists():
            raise SystemExit(f"missing {p}\n  Run `cd build-{args.arch} && ninja ntoskrnl` first.")

    with tempfile.TemporaryDirectory(prefix="genres-") as tmp:
        fresh = regenerate(
            Path(tmp), args.arch, triple, clang, windres, bugcodes_rc, cfg["defines"]
        )

    if args.check:
        if not out_file.exists():
            print(f"FAIL: {out_file} does not exist", file=sys.stderr)
            return 2
        existing = out_file.read_bytes()
        if existing != fresh:
            print(f"FAIL: {out_file} differs from fresh regen", file=sys.stderr)
            print(f"  expected {len(fresh)} bytes, got {len(existing)} bytes", file=sys.stderr)
            return 1
        print(f"OK: {out_file} matches fresh regen ({len(fresh)} bytes)")
        return 0

    out_file.parent.mkdir(parents=True, exist_ok=True)
    out_file.write_bytes(fresh)
    print(f"wrote {out_file} ({len(fresh)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
