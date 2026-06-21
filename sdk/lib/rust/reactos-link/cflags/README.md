# Committed C-compile flag templates — cmake-off fallback (WS1 DEEP)

These are per-arch flag templates that let `reactos-link`'s
`c_from_compile_db(<name>, …)` compile its C floor **without** a configured
`build-<arch>/compile_commands.json` — the cmake-off path (a cargo build with no
cmake configure/ninja step).

| file | consumer | sources |
|---|---|---|
| `blrtl.{i386,amd64,arm64}.flags` | `rosload` / `uefildr` loaders | the boot-RTL C floor: `sdk/lib/rtl/{bitmap,compress,crc32,image,largeint,random,unicode,bitmap64,byteswap,mem}.c` + `boot/rtl/nlsboot.c` |

## How it works

`reactos-link::compile_c_db_committed` (the `else` branch of the `c_db_libs`
loop) reads `<name>.<arch>.flags`, substitutes `{REPO}` → the repo root, appends
the native-hpp-generated xdk include dirs (see below) plus the source + `-o`, and
runs clang per source — then archives + links exactly like the
compile_commands.json replay branch. **cmake's compile_commands.json wins when a
build is configured** (the replay branch — so the committed flags can never make a
live build stale); these fill in only when there is no cmake build at all. Same
fallback shape as WS2 (`sdk/include/asm/committed/ks*.inc`) and WS3 (the committed
`.res`).

## What the template is

Each `<name>.<arch>.flags` is cmake's **exact** recorded blrtl compile flags with
the two *inert* bits stripped:

- the ninja **PCH** machinery (`-Xclang -include-pch …` + the
  `-Xclang -include …/cmake_pch.h` header force-include) — droppable because every
  blrtl `.c` already `#include <rtl.h>` textually; and
- the `build-<arch>/…` **`-I`** paths — empirically inert for blrtl *except* for
  the one generated header `psdk/winnt.h`, which the cmake-off path regenerates
  itself via `native-hpp` (`sdk/tools/hpp/hpp.c`, self-hosted like spec2def) over
  `sdk/include/xdk/winnt.template.h` — proven byte-identical to cmake's.

Objects built from the template are **code + data + relocation identical** to
cmake's (only `.debug_*` `comp_dir` strings differ, runtime-irrelevant); the
linked `uefildr.exe` has a **byte-identical `.text`** to the shipping cmake loader.

## Regenerating (when the blrtl cmake flags change — rare)

```
# from a configured build-<arch>/ :
python3 tools/gen_blrtl_flags.py --arch i386 --arch amd64 --arch arm64
```

A CI diff-check (committed vs freshly-generated) catches drift cheaply. See the
roadmap `docs/engineering/cargo-drive-cmake-off-roadmap.md` (WS1 DEEP).
