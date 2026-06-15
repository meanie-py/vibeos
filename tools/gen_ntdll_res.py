#!/usr/bin/env python3
"""Regenerate the cargo-driven ntdll resource blobs (PE-COFF `.res`).

Out-of-band replacement for the build-time resource step in
`sdk/lib/rust/ntdll{,_vista}-cargo/build.rs` (and the cmake
`add_library(... ntdll.rc)` rule). Same windres pipeline cmake uses, but
driven from Python and rooted at the repo so the cargo build doesn't need
the full ReactOS `-I` chain on a build variable.

Covers both modules × all three arches:
    ntdll        → data/ntdll-<arch>.rc.res        (dll/ntdll/def/ntdll.rc)
    ntdll_vista  → data/ntdll_vista-<arch>.rc.res   (dll/ntdll/nt_0600/ntdll_vista.rc)

Output is byte-identical to cmake's
`build-<arch>/dll/ntdll/.../<name>.rc.res`. Re-run only when the version
blocks (`common.ver`, `reactos/version.rc`, `ntverp.h`) or `ntstatus.rc`
change. Mirrors `tools/gen_ntoskrnl_res.py`.

Requires `build-<arch>/` configured (for the generated headers:
`build-<arch>/sdk/include/...`, incl. `reactos/mc/ntstatus.rc` + its
`.bin` sidecars). Run `cd build-<arch> && ninja ntdll` first if missing.

Usage:
    python3 tools/gen_ntdll_res.py [--arch i386,amd64,arm64] [--module ntdll,ntdll_vista]
    python3 tools/gen_ntdll_res.py --check     # verify checked-in matches a fresh regen
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LLVM_MINGW = REPO / ".build-tools/llvm-mingw"

# --- per-arch toolchain + arch-specific preprocessor defines -----------------
ARCHES = {
    "i386": {
        "triple": "i686-w64-mingw32",
        "defines": ["_M_IX86", "_USE_32BIT_TIME_T", "_X86_", "__i386__", "i386"],
    },
    "amd64": {
        "triple": "x86_64-w64-mingw32",
        "defines": ["_AMD64_", "_M_AMD64", "_M_X64", "_WIN64", "__x86_64__"],
    },
    "arm64": {
        "triple": "aarch64-w64-mingw32",
        "defines": ["_ARM64_", "_M_ARM64", "_WIN64", "__aarch64__", "__arm64__"],
    },
}

# --- per-module rc source + checked-in output stem ---------------------------
# Include dirs are (kind, relpath): "src" = under REPO, "bld" = under build-<arch>.
_COMMON_TAIL_SRC = [
    ("src", "sdk/include/reactos/subsys"),
    # [module include dir slots in here]
    ("src", "sdk/include"),
    ("src", "sdk/include/crt"),
    ("src", "sdk/include/ddk"),
    ("src", "sdk/include/ndk"),
    ("src", "sdk/include/psdk"),
    ("src", "sdk/include/reactos"),
    ("src", "sdk/include/reactos/libs"),
    ("src", "sdk/include/vcruntime"),
    ("src", "sdk/include/winrt"),
    ("bld", "sdk/include"),
    ("bld", "sdk/include/psdk"),
    ("bld", "sdk/include/ddk"),
    ("bld", "sdk/include/dxsdk"),
    ("bld", "sdk/include/reactos"),
    ("bld", "sdk/include/reactos/mc"),
    ("src", "sdk/include/dxsdk"),
    ("src", "sdk/lib/pseh/include"),
]

MODULES = {
    "ntdll": {
        "rc": "dll/ntdll/def/ntdll.rc",
        "out": "sdk/lib/rust/ntdll-cargo/data/ntdll-{arch}.rc.res",
        # build-<arch>/dll/ntdll, dll/ntdll, <subsys>, dll/ntdll/include, ...
        # tail, then dxsdk/pseh, apisets, crt
        "includes": (
            [
                ("bld", "dll/ntdll"),
                ("src", "dll/ntdll"),
                ("src", "sdk/include/reactos/subsys"),
                ("src", "dll/ntdll/include"),
                *_COMMON_TAIL_SRC[1:],
                ("src", "sdk/lib/apisets"),
                ("src", "sdk/include/crt"),
            ]
        ),
    },
    "ntdll_vista": {
        "rc": "dll/ntdll/nt_0600/ntdll_vista.rc",
        "out": "sdk/lib/rust/ntdll_vista-cargo/data/ntdll_vista-{arch}.rc.res",
        "includes": (
            [
                ("bld", "dll/ntdll/nt_0600"),
                ("src", "dll/ntdll/nt_0600"),
                ("src", "sdk/include/reactos/subsys"),
                ("src", "dll/ntdll/nt_0600/include"),
                *_COMMON_TAIL_SRC[1:],
                ("src", "sdk/include/crt"),
            ]
        ),
    },
}

# Defines shared by every (module, arch). Mirrors the cmake ntdll rc rule.
COMMON_DEFINES = [
    ("RC_INVOKED", None),
    ("__WIN32__", "1"),
    ("__FLAT__", "1"),
]
TAIL_DEFINES = [
    ("CRTDLL", None),
    ("D3D_UMD_INTERFACE_VERSION", "0x000C"),
    ("DBG", "1"),
    ("DLL_EXPORT_VERSION", "0x502"),
    ("DXGKDDI_INTERFACE_VERSION", "0x1052"),
    ("MINGW_HAS_SECURE_API", "1"),
    ("USE_COMPILER_EXCEPTIONS", None),
    ("WINVER", "0x502"),
    ("_CRT_NON_CONFORMING_SWPRINTFS", None),
    ("_CRT_SUPPRESS_RESTRICT", None),
    ("_GLIBCXX_HAVE_BROKEN_VSWPRINTF", None),
    ("_NEW_DELETE_OPERATORS_", None),
    ("_NTOSKRNL_", None),
    ("_SEH_ENABLE_TRACE", None),
    ("_SETUPAPI_VER", "0x502"),
    ("_USE_DUMMY_PSEH", "1"),
    ("_WIN32_IE", "0x603"),
    ("_WIN32_WINDOWS", "0x502"),
    ("_WIN32_WINNT", "0x502"),
    ("__NTDLL__", None),
    ("__REACTOS__", None),
]

# 136 LANGUAGE_* defines required by common.ver / version.rc.
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


def _include_path(kind: str, rel: str, arch: str) -> Path:
    return (REPO / f"build-{arch}" / rel) if kind == "bld" else (REPO / rel)


def regenerate(module: str, arch: str, work: Path) -> bytes:
    acfg = ARCHES[arch]
    mcfg = MODULES[module]
    triple = acfg["triple"]
    windres = LLVM_MINGW / "bin" / f"{triple}-windres"
    clang = LLVM_MINGW / "bin" / f"{triple}-clang"
    rc_source = REPO / mcfg["rc"]
    out_path = work / f"{module}.rc.res"

    cmd = [str(windres), "-O", "coff"]
    for kind, rel in mcfg["includes"]:
        cmd.extend(["-I", str(_include_path(kind, rel, arch))])
    cmd.extend(
        [
            f"--preprocessor={clang}",
            f"--preprocessor-arg=--target={triple}",
            "--preprocessor-arg=-E",
            "--preprocessor-arg=-nostdinc",
            "--preprocessor-arg=-xc-header",
        ]
    )
    for name, value in COMMON_DEFINES:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    for lang in LANGUAGES:
        cmd.append(f"-DLANGUAGE_{lang}")
    for name, value in TAIL_DEFINES:
        cmd.append(f"-D{name}={value}" if value is not None else f"-D{name}")
    for d in acfg["defines"]:
        cmd.append(f"-D{d}")
    cmd.extend([str(rc_source), str(out_path)])

    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"windres failed for {module}/{arch} (exit {proc.returncode})")
    return out_path.read_bytes()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--arch", default="i386,amd64,arm64", help="comma-separated subset of i386,amd64,arm64"
    )
    ap.add_argument(
        "--module", default="ntdll,ntdll_vista", help="comma-separated subset of ntdll,ntdll_vista"
    )
    ap.add_argument(
        "--check", action="store_true", help="verify checked-in files match a fresh regen"
    )
    args = ap.parse_args()

    arches = [a.strip() for a in args.arch.split(",") if a.strip()]
    modules = [m.strip() for m in args.module.split(",") if m.strip()]
    for a in arches:
        if a not in ARCHES:
            raise SystemExit(f"unknown arch {a!r}")
    for m in modules:
        if m not in MODULES:
            raise SystemExit(f"unknown module {m!r}")

    rc = 0
    for module in modules:
        for arch in arches:
            triple = ARCHES[arch]["triple"]
            windres = LLVM_MINGW / "bin" / f"{triple}-windres"
            build_dir = REPO / f"build-{arch}"
            out_file = REPO / MODULES[module]["out"].format(arch=arch)
            for p in [windres, REPO / MODULES[module]["rc"], build_dir]:
                if not p.exists():
                    raise SystemExit(f"missing {p}\n  Run `cd build-{arch} && ninja ntdll` first.")

            with tempfile.TemporaryDirectory(prefix="genres-") as tmp:
                fresh = regenerate(module, arch, Path(tmp))

            if args.check:
                if not out_file.exists():
                    print(f"FAIL: {out_file} does not exist", file=sys.stderr)
                    rc = 2
                    continue
                existing = out_file.read_bytes()
                if existing != fresh:
                    print(
                        f"FAIL: {out_file} differs from fresh regen "
                        f"(expected {len(fresh)} bytes, got {len(existing)})",
                        file=sys.stderr,
                    )
                    rc = rc or 1
                else:
                    print(f"OK: {out_file.name} matches fresh regen ({len(fresh)} bytes)")
            else:
                out_file.parent.mkdir(parents=True, exist_ok=True)
                out_file.write_bytes(fresh)
                print(f"wrote {out_file} ({len(fresh)} bytes)")

    return rc


if __name__ == "__main__":
    sys.exit(main())
