# Solitaire

Klondike solitaire, drawn with the card library at `sdk/lib/cardlib`.

## Provenance

This is a **clean-room reimplementation**. The version this replaces was written by
J Brown in 2001 and shipped as the worked example for his *CardLib*, under the same
freeware grant — "permits distribution and nothing else", with a second-hand claim of
public domain relayed by a ReactOS contributor — that caused CardLib itself to be
replaced. It was missed when CardLib went, because the `.cpp` carried no copyright
header and the only statement of provenance was a `ReadMe.txt`; it surfaced from
grepping for the author's name afterwards.

`solitaire.cpp`, `solgame.cpp`, `solcreate.cpp` and `solitaire.h` were written from a
functional specification by an implementer that had never read the originals — the
originals were deleted from the working tree first, so they could not be. What the
implementer worked from: the rules of Klondike, `sdk/lib/cardlib/cardlib.h`, the
resource scripts in this directory, `solundo.cpp`, and Spider (`../spider`) as a
worked example of driving the same API.

The *specification* was written by an agent that had read the originals earlier in the
same session, so it was deliberately confined to observable behaviour and required
interfaces. Integration, review, build and play-testing were done by that same agent.
Recording that plainly is the point: the implementation side of the wall was clean,
the specification side was not blind.

## What is NOT a reimplementation, and stays

- **`solundo.cpp`** — ReactOS's own GPL undo module by Tibor Lajos Füzi, untouched. It
  is the reason the new `solitaire.h` looks the way it does: the header has to declare
  what that file already uses.
- **`resource.h`, `rsrc.rc`, `lang/`, `solitaire.ico`** — ReactOS community work,
  including ~30 translations. Only the `IDS_SOL_ABOUT` string changed, because it
  credited J Brown for a game that is no longer his and named a "CardLib version 1.0"
  that no longer exists. Translator credits in it were preserved.

## Notes

The waste pile is a *view*, not the source of truth: `activepile` holds the real waste
and the visible region shows either the whole pile (draw-one) or the fanned top few
(draw-three). `solundo.cpp` depends on exactly that split, so both shapes have to be
reproduced or undo and normal play disagree.

Winning is handled by posting a message rather than acting inside the add callback: the
card library still touches the source region after the callback returns, so dealing a
fresh game from inside it would have the library turn over a card of the new deal.
