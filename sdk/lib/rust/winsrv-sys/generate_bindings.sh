#!/bin/bash
# Generate the winsrv.dll (consrv + usersrv + concfg + top-level)
# FFI surface for all 3 architectures (Phase 5 of the SMSS+CSRSS
# translation track).
#
# Lifts the actual per-arch compile flags from build-<arch>/build.ninja's
# consrv rule — consrv has the widest header set (wincon/conmsg/imm/
# ntuser/csrsrv + its private include/) so it covers what concfg and
# usersrv need too, mirroring the win32k-sys and ntdll-sys pattern.
#
# Prerequisites: cargo install bindgen-cli; a configured build-<arch>/.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
# build-<arch>/ dirs are gitignored — a fresh worktree has none, and
# vibeos has no cold-cmake-build path (warm dirs only). Point at a tree
# that HAS warm build dirs (typically the main checkout) via
# VIBEOS_BUILD_ROOT; headers + ninja flags are all bindgen needs.
BUILD_ROOT="${VIBEOS_BUILD_ROOT:-$SRC}"
RES="${SRC}/.build-tools/llvm-mingw/lib/clang/19/include"
OUT_DIR="${SCRIPT_DIR}/src"
WRAPPER="${SCRIPT_DIR}/wrapper.h"

# The ninja build statement we lift DEFINES/INCLUDES/FLAGS from (the consrv target).
# A TU that is still COMPILED as C. The original rule's .c went to Rust when the module
# was cohab-flipped, the grep below then found nothing, and bindgen ran with no
# -I/-D at all (host mingw headers) -- so this island could not be regenerated.
NINJA_RULE='win32ss/user/winsrv/CMakeFiles/consrv.dir/consrv/consrv_rust_shim.c.obj:'

# Type/var allowlist '.+' (struct field types chain everywhere).
# Function allowlist covers the winsrv naming families:
#   - ConDrv*/ConSrv*/ConsoleClient*/ConsoleInput/ConsoleOutput
#   - Cn(s|c)l[A-Za-z]+  catches Cnsl-style helpers if present
#   - Term[A-Za-z]+      TERMINAL_VTBL impls (TermBeep/TermResize/…)
#   - Concfg|TT|Font|Settings[A-Za-z]+  concfg surface
#   - UserSrv*/UserClient*/UserSound*/UserInit*  usersrv surface
#   - Base[A-Za-z]+      cohabits with basesrv via _UserSoundSentry
# The ntoskrnl/ntdll APIs winsrv also calls (Nt/Zw/Rtl/Csr/Sm/Base) stay
# in ntdll-sys to avoid clashing-extern noise.
# NT/CSR/SM/BASE user-mode surface (mirror ntdll-sys allowlist). With
# this in place winsrv-sys becomes a true SUPERSET FFI crate — one
# glob `use winsrv_sys::winsrv::*` covers everything winsrv-rust
# touches without name-conflict shadows against a parallel
# `ntdll_sys::nt::*` glob (HKEY/HANDLE/LPWSTR/MAX_PATH duplicated
# across the two bindgens were the Phase 5.0b spike-blocker).
NTDLL_FNS='(Nt|Zw)[A-Z][A-Za-z0-9_]*|Rtl[A-Z][A-Za-z0-9_]*|Ldr[A-Z][A-Za-z0-9_]*|Dbg[A-Z][A-Za-z0-9_]*|Csr[A-Za-z0-9_]+|Sm[A-Za-z0-9_]+|Sb[A-Za-z0-9_]+|Base[A-Za-z0-9_]+'
# winsrv-internal naming families (consrv/usersrv/concfg).
WINSRV_FNS='Con(Drv|Srv|sole)[A-Za-z0-9_]*|Cnsl[A-Za-z0-9_]*|Term[A-Za-z0-9_]+|Concfg[A-Za-z0-9_]*|TT[A-Z][A-Za-z0-9_]*|Settings[A-Za-z0-9_]+|UserSrv[A-Za-z0-9_]*|UserClient[A-Za-z0-9_]*|UserSound[A-Za-z0-9_]*|UserInit[A-Za-z0-9_]*|UserServer[A-Za-z0-9_]*|FindTopLevelWnd|GetTimeouts'
# Win32 imports winsrv consumes (advapi32 registry, kernel32 console+
# nls+locale, user32+gdi32 GUI surface, strsafe macros, msvcrt). These
# are the "outer ring" of FFI consrv/usersrv/concfg/winsrv top-level
# call into for their actual work.
WIN32_FNS='Reg[A-Z][A-Za-z0-9_]+|String(Cb|Cch)[A-Z][A-Za-z0-9_]+|(Get|Set|Is|Format|Find|Translate|Beep|Activate|Show|Update|Invalidate|Get|Send|Post|Register|Unregister|Create|Destroy|Read|Write|Open|Close|Delete|Enum|Query|Map|Unmap|Wait|Release|Acquire|Lock|Unlock|Load|Free|Kill|Heap|Elf|Message|Empty|Select|Drag|Peek|Real|Pat|Ext|Scroll|Change|Bit|Begin|End|Invert|Combine|Track|Append|Draw|Default|Window|Move|Adjust|Client|Screen|Equal|Inflate|Offset|Pt|Sleep|Notify|Animate|Capture|Validate|Enable|Disable|Hide|Print|Process|Cut|Copy|Paste|Choose|Clip|Char|Get|Set|To|From|Update|Display|Cursor|Caret|Icon|Brush|Pen|Pal|Font|Mouse|Keyboard|Key|Vk|Map|Adjust|Defer|Reply|Compute|Setup|Tear|Render|Reset|Rect|Region|Cancel|Tag|Map|Set|Get|Cct|Make|System|Console|Output|Input|Buffer|Mode|Color|Title)[A-Z][A-Za-z0-9_]*|MultiByteToWideChar|WideCharToMultiByte|TranslateCharsetInfo|TranslateMessage|DispatchMessageW|DefWindowProcW|CallWindowProcW|GdiTransparentBlt|PolyTextOutW|ExtTextOutW|TextOutW|CharNextW|CharPrevW|MAKE[A-Z][A-Za-z0-9_]+|IMM[A-Z][A-Za-z0-9_]+|CoCreateInstance|CoInitializeEx|CoUninitialize|GlobalAlloc|GlobalFree|GlobalLock|GlobalUnlock|GlobalSize|GlobalReAlloc|GlobalHandle|VkKeyScanW|VkKeyScanExW|MapVirtualKeyW|MapVirtualKeyExW|ToUnicodeEx|WindowFromPoint|TrackPopupMenuEx|AppendMenuW|DefDlgProcW|DragQueryFileW|DragFinish|DragAcceptFiles|StringCchCatW|StringCchCopyW|StringCchCopyNW|SendNotifyMessageW|RealizePalette|MonitorFromWindow|MonitorFromRect|MonitorFromPoint|RegisterClassExW|UnregisterClassW|SystemParametersInfoW|UpdateLayeredWindow|Beep|MessageBeep|GetVersion|GetSystemMetrics|FillRect|FrameRect|CreateDIBSection|CreateBitmap|CreatePalette|CreateRectRgn|CreateCompatibleBitmap|CreateCompatibleDC|DeleteDC|DeleteObject|DeleteFontFamily|CreateFontIndirectW|CreateFontW|StretchDIBits|SetDIBitsToDevice|GetDIBits|GetPaletteEntries|GetTextExtentPoint32W|GetTextMetricsW|TextMetricsCachedFlush|CombineRgn|InvertRgn|FillRgn|PtInRect|EqualRect|SetRect|SetRectEmpty|IntersectRect|UnionRect|SubtractRect|OffsetRect|InflateRect|CopyRect|GetClipBox|GetClipRgn|ScreenToClient|ClientToScreen|GetCursorPos|SetCursorPos|GetClientRect|GetWindowRect|GetWindow|GetForegroundWindow|GetParent|GetActiveWindow|GetFocus|SetFocus|GetDesktopWindow|GetWindowDC|ReleaseDC|GetDC|UpdateWindow|InvalidateRect|ValidateRect|ShowWindow|SetWindowText|SetWindowPos|MoveWindow|EnableWindow|IsZoomed|IsIconic|IsWindow|IsWindowVisible|IsWindowUnicode|GetMenu|GetSystemMenu|CreateMenu|CreatePopupMenu|DestroyMenu|GetMenuItemInfoW|SetMenuItemInfoW|RemoveMenu|DeleteMenu|InsertMenuW|InsertMenuItemW|CheckMenuItem|EnableMenuItem|GetSubMenu|GetMenuItemCount|GetMenuItemID|GetMenuState|GetMenuStringW|TrackPopupMenu|GetSysColor|GetSysColorBrush|FreeLibrary|GetModuleHandleW|GetModuleHandleA|GetProcAddress|GetLastError|SetLastError|LocalAlloc|LocalFree|LocalLock|LocalUnlock|LocalSize|LocalReAlloc|LocalHandle|LocalReSize|LoadLibraryW|LoadLibraryExW|LoadStringW|LoadAcceleratorsW|LoadBitmapW|LoadImageW|LoadCursorW|LoadIconW|LoadMenuW|LoadMenuIndirectW|TranslateAcceleratorW|CharNextW|CharPrevW|lstrlenW|lstrcpynW|lstrcpyW|lstrcatW|lstrcmpW|GetThreadDesktop|OpenInputDesktop|CloseDesktop|GetUserObjectInformationW|SwitchDesktop|MapViewOfFile|UnmapViewOfFile|CreateFileMappingW|OpenFileMappingW|VirtualAlloc|VirtualFree|VirtualProtect|GetCurrentThreadId|GetCurrentProcessId|GetCurrentProcess|GetCurrentThread|TerminateProcess|TerminateThread|ExitProcess|ExitThread|ResumeThread|SuspendThread|CreateThread|GetExitCodeThread|WaitForSingleObject|WaitForMultipleObjects|MsgWaitForMultipleObjects|CloseHandle|DuplicateHandle|GetTickCount|QueryPerformanceCounter|QueryPerformanceFrequency|Sleep|GetConsoleInputBufferMode|GetConsoleOutputBufferMode|RtlInitUnicodeString|RtlUnicodeStringToInteger|RtlIntegerToUnicodeString|swscanf|wsprintfW|wvsprintfW|wnsprintfW'
FN_ALLOW="${NTDLL_FNS}|${WINSRV_FNS}|${WIN32_FNS}"

run_bindgen() {
    local ARCH=$1 TARGET=$2 BUILD_DIR="${BUILD_ROOT}/build-$1"
    local RAW="/tmp/winsrv_bindgen_${ARCH}_raw.rs"
    local FINAL="${OUT_DIR}/bindgen_${ARCH}.rs"
    local NINJA="${BUILD_DIR}/build.ninja"

    if [ ! -f "$NINJA" ]; then
        echo "!! ${ARCH}: ${NINJA} missing — skipping (configure build-${ARCH})"
        return 0
    fi
    echo "=== Generating ${ARCH} (target: ${TARGET}) ==="

    local DEFINES INCLUDES RAWFLAGS SANITIZED
    DEFINES=$(grep -A8 "$NINJA_RULE" "$NINJA" | grep -m1 -E '^  DEFINES' | sed 's/^  DEFINES = //')
    INCLUDES=$(grep -A8 "$NINJA_RULE" "$NINJA" | grep -m1 -E '^  INCLUDES' | sed 's/^  INCLUDES = //')
    RAWFLAGS=$(grep -A8 "$NINJA_RULE" "$NINJA" | grep -m1 -E '^  FLAGS' | sed 's/^  FLAGS = //')
    if [ -z "$DEFINES" ] || [ -z "$INCLUDES" ]; then
        echo "!! ${ARCH}: no DEFINES/INCLUDES for $NINJA_RULE in $NINJA" >&2
        exit 1
    fi
    # Drop codegen/PCH-only flags that break or are irrelevant to bindgen
    # parsing. NEVER add -fms-compatibility.
    SANITIZED=$(echo "$RAWFLAGS" | tr ' ' '\n' \
        | grep -vE '^-g|^-O|^-Winvalid-pch|^-ffile-prefix-map|^-include-pch|\.pch$|^-fno-omit|^-mstackrealign|^-fno-optimize|^-march|^-mtune' \
        | grep -vE '^-Xclang$|^-include$|cmake_pch' | tr '\n' ' ')

    bindgen "$WRAPPER" \
        --use-core --no-layout-tests --no-doc-comments \
        --allowlist-type '.+' \
        --allowlist-var '.+' \
        --allowlist-function "$FN_ALLOW" \
        -o "$RAW" \
        -- -ferror-limit=0 -Wno-everything $SANITIZED $DEFINES $INCLUDES \
        -I"${SRC}/win32ss/user/winsrv" \
        -I"${SRC}/win32ss/user/winsrv/concfg" \
        -I"${SRC}/win32ss/user/winsrv/consrv" \
        -I"${SRC}/win32ss/user/winsrv/usersrv" \
        -isystem "$RES" --target="${TARGET}"

    python3 "${SCRIPT_DIR}/postprocess_bindgen.py" "$RAW" "$ARCH" "$FINAL"
}

run_bindgen i386  i686-pc-windows-gnu
run_bindgen amd64 x86_64-pc-windows-gnu
run_bindgen arm64 aarch64-pc-windows-gnu

echo ""
echo "=== Done. Files in ${OUT_DIR}/ ==="
ls -la "${OUT_DIR}"/bindgen_*.rs 2>/dev/null

# Spell ABI-identical callback slots `extern "system"` across the three islands this run just
# wrote, projecting per symbol from the i386 witness (the only arch that still records which
# callbacks are stdcall). Without this the slots keep the arch's literal convention, a
# translated `extern "system"` callback will not unify with them, and the standing workaround
# is a transmute that erases the whole fn type. Idempotent — safe to re-run.
#
# This lives in EVERY generator on purpose: normalization that only some of them apply is
# normalization that the next regeneration of any other crate silently undoes.
_isl_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "${_isl_dir}/../../../../tools/rustconv/normalize_island_abi.py" --apply "${_isl_dir}"
