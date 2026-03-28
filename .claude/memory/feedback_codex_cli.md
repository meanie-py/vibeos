---
name: codex-cli-translation
description: Use OpenAI Codex CLI for C-to-Rust translations as an alternative/complement to rustconv Sonnet pipeline
type: feedback
---

Use `codex` CLI (codex-cli, installed via npm) for C-to-Rust kernel translations. It produces good quality output.

**Why:** Codex (GPT-5.4) scored well on iofunc.rs — 9 functions, 1076 lines, only 2 constant errors (caught by verify_constants.py) and a few missing extern declarations needed. Cross-model translation catches different blind spots than same-model review.

**How to apply:**
- Run `codex exec --full-auto "prompt"` for non-interactive translation
- Default model works (o3 requires paid API, not ChatGPT account)
- Provide detailed translation rules in the prompt (pool tag encoding, NT_ERROR vs NT_SUCCESS, extern "system", DUMMY_PSEH SEH handling, addr_of_mut! for events, etc.)
- Always run `verify_constants.py` on the output to catch wrong constant values
- Always add missing extern declarations, constants, and type casts manually after integration
- The CLI reads files from the repo, so point it at both the C source and existing Rust patterns
