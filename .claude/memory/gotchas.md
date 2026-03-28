# Gotchas

## DO NOT Read PNG/Image Files in Long Sessions
**Problem**: Reading `.png` files (e.g., smoke test screenshots like `qemu/i386_smoke.png`) via the Read tool embeds base64 image data into the conversation context. In long sessions with many content blocks (300+), this can cause API errors:
```
Image format image/png not supported
```
This crashes the Claude Code CLI session and requires manually editing internal JSON files to recover.

**Prevention**:
- Use `file <path>` or `identify <path>` to check image metadata instead of reading the raw file
- If you must inspect a screenshot, use a subagent (isolates the image from main context)
- Never read `.png`/`.jpg`/`.bmp` files during long debugging sessions
- QEMU `screendump` produces PPM format — these may be renamed to `.png` but aren't actually PNG
