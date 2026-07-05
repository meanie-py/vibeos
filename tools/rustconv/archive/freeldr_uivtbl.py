#!/usr/bin/env python3
"""freeldr_uivtbl — normalize rustconv's UIVTBL access mistranslation.

C: `UIVTBL UiVtbl;` is a *value* and `UiVtbl.DrawText(a,b)` calls a plain
function pointer. rustconv variously emitted:
  * `(*UiVtbl).Field`                  — deref of a value (E0614)
  * `(*core::ptr::addr_of!(UiVtbl))…`   — same, verbose
  * `(UiVtbl.Field)(args)`              — Option<fn> not unwrapped (E0618)
The Rust `UIVTBL` (ui_keystone) has `Option<extern "C" fn>` slots, so the
correct form is `(UiVtbl.Field.unwrap())(args)` with NO deref.

Two deterministic passes, idempotent:
  1. strip the bogus value-deref:  `(*core::ptr::addr_of!(IDENT))` → `IDENT`
     and `(*IDENT)` → `IDENT` for the known value-statics (UiVtbl /
     SavedNoUiVtbl / *Vtbl / UiProgressBar) only.
  2. for a call `(<recv>.<Slot>)(`  where <Slot> is one of the 24 UIVTBL
     fn-ptr slots and it isn't already `.unwrap()`, insert `.unwrap()`.

Reusable across the ui unit + any parked unit that touches UiVtbl.
Usage: python3 freeldr_uivtbl.py <file.rs> [...]
"""
import re
import sys

SLOTS = ("Initialize", "UnInitialize", "DrawBackdrop", "FillArea",
         "DrawShadow", "DrawBox", "DrawText", "DrawText2",
         "DrawCenteredText", "DrawStatusText", "UpdateDateTime",
         "MessageBox", "MessageBoxCritical", "DrawProgressBarCenter",
         "DrawProgressBar", "SetProgressBarText", "TickProgressBar",
         "EditBox", "TextToColor", "TextToFillStyle", "FadeInBackdrop",
         "FadeOut", "DisplayMenu", "DrawMenu")
# value-statics that rustconv wrongly treated as pointers
VALUES = ("UiVtbl", "SavedNoUiVtbl", "TuiVtbl", "MiniTuiVtbl",
          "NoUiVtbl", "UiProgressBar")
SLOT_RE = "|".join(SLOTS)
VAL_RE = "|".join(VALUES)


def fix(path):
    s = open(path).read()
    o = s
    # 1a. (*core::ptr::addr_of!(V)) / (*core::ptr::addr_of_mut!(V)) → V
    s = re.sub(
        r'\(\*\s*core::ptr::addr_of(?:_mut)?!\(\s*(' + VAL_RE + r')\s*\)\s*\)',
        r'\1', s)
    # 1b. (*V) → V   (only the known value-statics)
    s = re.sub(r'\(\*\s*(' + VAL_RE + r')\s*\)', r'\1', s)
    # 2. (recv.Slot)(  →  (recv.Slot.unwrap())(   unless already unwrapped.
    #    recv = an ident path possibly with .field chain ending at the Slot.
    def unwrap(m):
        inner = m.group(1)
        if inner.endswith(".unwrap()"):
            return m.group(0)
        return "(" + inner + ".unwrap())("
    s = re.sub(
        r'\(([A-Za-z_][\w.]*\.(?:' + SLOT_RE + r'))\)\(',
        unwrap, s)
    if s != o:
        open(path, "w").write(s)
    print(f"{path}: uivtbl {'changed' if s != o else 'noop'}")


if __name__ == "__main__":
    for p in sys.argv[1:]:
        fix(p)
