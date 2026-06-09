<!--
  deepdrive GRIND PRESET — bounded compile-error squashing for a BANKED translation.
  Copy this file, fill the <<...>> slots, and pass it via:
    deepdrive run --dir <your worktree> --prompt-file <filled copy> \
                  --verify "cargo check -p <<CRATE>> --target <<TARGET>> 2>&1"
  Drive it in BOUNDED turns: after each turn, read verify.error_count + diff, then either
  send the next bounded instruction or STEER on whatever class it's stuck on.
  Delete these HTML comments in your filled copy if you like; DeepCode ignores them anyway.
-->

# Task: drive `<<CRATE>>` to `cargo check` green — GRIND ONLY, do not re-translate

The crate `<<CRATE>>` is a **banked translation** (already produced by a stronger model). It does not
yet compile. Your job is to make `cargo check -p <<CRATE>> --target <<TARGET>>` pass by fixing the
**compiler errors**, with the **smallest possible edits**. This is a grind, not a rewrite.

## HARD RULES — do not violate these

1. **NEVER rewrite a function wholesale.** Make the minimal edit that satisfies the specific compiler
   error at that site. Preserve the existing structure, control flow, naming, and idiom.
2. **Do not change program logic or behavior.** You are fixing types/imports/syntax/ABI, not redesigning.
3. **Do not re-translate, re-generate, or "improve" code.** The translation is intentional and paid for.
4. **Stay in this crate.** Do not edit other crates, `CMakeLists`, the workspace `Cargo.toml` beyond
   adding `<<CRATE>>` to `members` if it is missing, or anything outside `<<CRATE>>`.
5. **No stubs.** Never replace a body with `todo!()`/`unimplemented!()`/`panic!()` to dodge an error.

## Method (cheap-first, then targeted)

1. Run `cargo check -p <<CRATE>> --target <<TARGET>> 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn`
   to see the error classes and counts. Work the largest mechanical class first.
2. For machine-applicable fixes (missing `mut`, unused, simple coercions), you MAY run
   `cargo fix -p <<CRATE>> --target <<TARGET>> --lib --broken-code --allow-no-vcs-commit --allow-dirty`
   to auto-apply rustc's suggestions. Then re-check.
3. For the rest, fix per error using the rules below. Batch several edits, THEN re-check once
   (don't re-run `cargo check` after every single edit — it wastes turns).
4. If you make **no progress on a class for 2 rounds**, STOP and report which class/site is stuck so the
   human can give you a global rule. Do not thrash by toggling edits back and forth.

## Subsystem-specific rules (provided by the steering engineer)

<<SUBSYSTEM_RULES — e.g. "DWORD=u32/BOOL=i32, add `as` casts don't change typedefs"; bindgen union
fields are `__bindgen_anon_N`; FASTCALL on i386 = extern "fastcall"; this struct's layout is
size-asserted so do not reorder fields; etc. Leave blank on the first turn and fill in as you learn
what it's stuck on.>>

## When done (or blocked)

Stop and report: the final `error_count`, the remaining error classes (with counts), and for anything
you could not fix under the rules, the exact site + error so the engineer can steer the next turn.
