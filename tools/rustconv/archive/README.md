# rustconv archive — retired one-off campaign scripts

These are single-use transform scripts written for specific past translation
campaigns (the freeldr Rust port, ke/ error-fix sweeps, bindgen flattening,
def/import dedup, WDF extern stripping, residual triage). Each had **zero
references** anywhere in the tree (no CLI wiring, no imports, no doc/CI hooks)
as of the 2026-07-04 tooling-consolidation sweep — they were run by hand once
and never again.

They are **archived, not deleted**: the transform logic (varargs/ABI promotion,
field-offset rewrites, cast repairs, etc.) encodes hard-won lessons that may be
worth grepping when a similar campaign recurs. Nothing in the live pipeline
depends on them; do not wire them back into `rustconv` without review.

Retired in: `docs/engineering/tooling-consolidation-plan.md` (T0). Restore any
one with `git mv tools/rustconv/archive/<name>.py tools/rustconv/<name>.py`.
