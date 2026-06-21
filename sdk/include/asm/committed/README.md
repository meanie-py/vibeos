# Committed `ks*.inc` — cmake-off fallback (WS2)

These are byte-stable copies of the per-arch kernel asm offset includes that
cmake's `asm` target generates into `build-<arch>/sdk/include/asm/`:

| file | arch | generator |
|---|---|---|
| `ks386.inc`   | i386  | `native-geninc` over the i386 `genincdata` PE module |
| `ksamd64.inc` | amd64 | `native-geninc` over the amd64 `genincdata` PE module |
| `ksarm64.h`   | arm64 | *(not yet committed — needs a configured `build-arm64`)* |

## Why they exist

The generation is **deterministic per (source, arch)** — `geninc` parses the
`.asmdef` section of a PE module compiled from `sdk/include/asm/genincdata.c`
(which pulls the per-arch `ks*.template.h`). It is NOT config-dependent, so a
committed copy is safe.

`reactos-link`'s `add_kernel_asm_includes` adds the **build-dir generated copy
first** (it always wins when a cmake build dir is present, so a configured build
can never go stale against these). It falls back to *this* directory only when
the generated `ks<arch>.{inc,h}` is absent — i.e. the cmake-off path, where the
cargo kernel build runs with no cmake configure/ninja step. See the
`cargo-drive-cmake-off-roadmap.md` (WS2).

## Regenerating (when `ks*.template.h` / `genincdata.c` change — rare)

```
# from a configured build-<arch>/ :
ninja asm                                   # runs native-geninc
cp build-i386/sdk/include/asm/ks386.inc     sdk/include/asm/committed/ks386.inc
cp build-amd64/sdk/include/asm/ksamd64.inc  sdk/include/asm/committed/ksamd64.inc
cp build-arm64/sdk/include/asm/ksarm64.h    sdk/include/asm/committed/ksarm64.h
```

A CI diff-check (committed vs freshly-generated) catches drift cheaply.
