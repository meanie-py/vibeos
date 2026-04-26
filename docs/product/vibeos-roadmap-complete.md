# Completed Roadmap Items

## 0b. Repo Restructure ✅ (2026-03-21)

Flatten `os-playground/reactos/` → `vibeos/` as a single git repo. Move build dirs, qemu scripts, docs, and toolchain inside the repo root. Update all paths in AGENTS.md, cmake, and scripts. Migrate Claude Code memory directory.

**Commit:** `1d570b10` — restructure: vibeos layout with docs/, qemu/, AGENTS.md

**What was done:**
- `reactos/` → `vibeos/` (git repo root)
- Build dirs moved into `vibeos/build-*/` (gitignored)
- Toolchain moved into `vibeos/.build-tools/llvm-mingw/` (gitignored)
- `qemu/` scripts tracked in git (binary images gitignored)
- `docs/product/` and `docs/engineering/` for working documents
- `CLAUDE.md` → `AGENTS.md` (shared agent instructions), `CLAUDE.md` and `GEMINI.md` reference it via `@AGENTS.md`
- `toolchain-llvm-mingw.cmake` updated to find toolchain at `.build-tools/llvm-mingw`
- `tests/qemu/vm.py` PROJECT_DIR updated for new layout
- Python venv recreated
- Claude Code memory migrated to new project path
