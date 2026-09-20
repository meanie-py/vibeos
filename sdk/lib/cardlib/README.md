# cardlib

The playing-card rendering library behind `sol.exe` and `spider.exe`: piles of
cards on a green table, with drag and drop, fanning, animation and the
per-pile rules the games plug in as callbacks.

- **Interface:** `cardlib.h` — plain C ABI, so C and C++ callers link it directly.
- **Implementation:** `sdk/lib/rust/cardlib-rust/` — `no_std`, allocation-free
  Rust, built as a staticlib for i386, amd64 and arm64 on the pinned stable
  toolchain.
- **Artwork:** loaded at runtime from `cards.dll` (`dll/win32/cards`), which is
  a separate ReactOS component and is not part of this library.

## Provenance

This is a **clean-room reimplementation** of the interface of *CardLib* by
J Brown (catch22.net), which this project previously carried under
`sdk/lib/3rdparty/cardlib`. That code was distributed as freeware — it grants
redistribution and nothing else, with a second-hand claim of public domain
that could not be substantiated — so it has been replaced rather than kept.

Credit for the original design of this interface is his. The implementation
here was written without reading, grepping or otherwise consulting any of the
original `.cpp` sources. What *was* consulted, deliberately and of necessity:

- the original public headers, as the interface contract;
- both consumers (`base/applications/games/solitaire`, `.../spider`) in full,
  which are the real behavioural specification;
- `dll/win32/cards/cards.c`, ReactOS's own LGPL card-art DLL, for how the
  artwork expects to be drawn.

The public interface is **not** a copy of the original headers either. It was
rewritten from the outside in against the ~45 entry points the two games
actually use, as a C API with opaque handles, and the games were converted to
it. Constant *values* (`CS_*`, `ecb*`) and the 0-51 card encoding are preserved
because the card artwork and the games' own arithmetic depend on them.

## Notes for the curious

**Card indices.** A card is `(value - 1) * 4 + suit`, value-major, so 0..3 are
the four aces. The artwork in `cards.dll` is stored suit-major, so loading
transposes it: strip slot `i` takes resource `(i % 4) * 13 + (i / 4) + 1`.
After that, a face card's strip slot *is* its card index.

**Strip slots.** All 68 bitmaps are composed once into one wide DC, at
`slot = resource id - 1`. Slots 0..51 are the faces, 52 the free-cell
placeholder, 53..64 the twelve backs, 65 the joker, 66 and 67 the X and O
placeholders. The `ecb*` names for the last four are inherited and wrong for
ReactOS's artwork — only 53..64 are used, and those are right.

**Rounded corners.** The artwork has square corners (white on the faces,
yellow on the backs). Cards are drawn by saving the three pixels at each
corner, blitting, and putting them back, which is what `cdtDrawExt` does.

**Table green.** The X and O placeholders have `RGB(0, 128, 0)` baked into
them, so that is the table colour whether we like it or not.

## Deliberate differences from the original

- **No `HPALETTE`.** The original exposed a `__hPalette` global for 256-colour
  displays. Nothing ever used it, and the artwork reaches us as a
  screen-depth DDB, so it is gone. 8bpp displays are not supported.
- **Trimmed surface.** `CardButton`, `DropZone`, `CardCount`, `DistributeStacks`,
  `SetResizeProc`, per-region locking and the free `CardBlt` had no callers and
  are not reimplemented. The card-preview dialogs blit out of
  `CardLib_CardBitmapsDC()` directly, which is what they did before.
- **Explicit lifetime.** `CardLib_Init()` must be called before anything else;
  the original relied on C++ static initialisation. It returns false if
  `cards.dll` is missing, and the games treat that as fatal rather than
  drawing an empty table.
