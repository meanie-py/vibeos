"""LLM API client — supports Claude (Anthropic) and Gemini (Google) backends."""

import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from .config import (
    get_backend, DEFAULT_MODELS,
    CLAUDE_MAX_TOKENS, GEMINI_MAX_TOKENS,
)


@dataclass
class TranslationResult:
    source_file: str
    rust_code: str
    model: str
    input_tokens: int            # UNCACHED input tokens only (full price)
    output_tokens: int
    success: bool
    error: Optional[str] = None
    cache_read_tokens: int = 0   # served from cache (~0.1x price)
    cache_write_tokens: int = 0  # written to cache (~1.25x price)


# Per-1M-token USD pricing (input, output, cache_read, cache_write).
# Cache read = 0.1x input; cache write (5-min ephemeral) = 1.25x input.
PRICING = {
    "claude-opus-4-8":   (5.00, 25.00, 0.50, 6.25),
    "claude-opus-4-7":   (5.00, 25.00, 0.50, 6.25),
    "claude-opus-4-6":   (5.00, 25.00, 0.50, 6.25),
    "claude-sonnet-4-5": (3.00, 15.00, 0.30, 3.75),
    "claude-sonnet-4-6": (3.00, 15.00, 0.30, 3.75),
    "claude-haiku-4-5":  (1.00,  5.00, 0.10, 1.25),
}


def estimate_cost(model: str, input_tokens: int, output_tokens: int,
                  cache_read_tokens: int = 0, cache_write_tokens: int = 0) -> Optional[float]:
    """USD cost for one (or accumulated) call. None if the model isn't priced here."""
    rates = PRICING.get(model)
    if rates is None:
        # tolerate dated/full IDs by prefix match
        rates = next((v for k, v in PRICING.items() if model.startswith(k)), None)
    if rates is None:
        return None
    pin, pout, pread, pwrite = rates
    return (input_tokens * pin + output_tokens * pout
            + cache_read_tokens * pread + cache_write_tokens * pwrite) / 1_000_000


def _submit_claude(prompt: str, model: str, system_prompt: str = "",
                   file_context: str = "") -> TranslationResult:
    """Submit via Anthropic Python SDK.

    Two cached system blocks form the stable prefix:
      1. system_prompt   — file-independent translation rules (cached across the
                            whole run)
      2. file_context    — per-file shared context (import superset, gap fns,
                            globals); byte-identical across all functions of one
                            file, so functions 2..N read it from cache (~0.1x)
                            instead of re-sending it uncached (1x).
    Only the function-specific `prompt` (the user turn) is full price each call.
    """
    import anthropic

    try:
        client = anthropic.Anthropic()  # uses ANTHROPIC_API_KEY env var

        kwargs = {
            "model": model,
            "max_tokens": CLAUDE_MAX_TOKENS,
            "messages": [{"role": "user", "content": prompt}],
        }

        # Build cached system blocks. Each gets its own cache_control breakpoint
        # (max 4 allowed); the prefix system_prompt → file_context is what the
        # cache matches on.
        system_blocks = []
        if system_prompt:
            system_blocks.append({
                "type": "text",
                "text": system_prompt,
                "cache_control": {"type": "ephemeral"},
            })
        if file_context:
            system_blocks.append({
                "type": "text",
                "text": file_context,
                "cache_control": {"type": "ephemeral"},
            })
        if system_blocks:
            kwargs["system"] = system_blocks

        # Use streaming to avoid 10-minute timeout on large outputs
        response_text = ""
        input_tokens = 0
        output_tokens = 0
        cache_read_tokens = 0
        cache_write_tokens = 0
        stop_reason = None

        try:
            with client.messages.stream(**kwargs) as stream:
                for event in stream:
                    if hasattr(event, 'type'):
                        if event.type == 'content_block_delta' and hasattr(event, 'delta'):
                            if hasattr(event.delta, 'text'):
                                response_text += event.delta.text
                        elif event.type == 'message_delta' and hasattr(event, 'usage'):
                            output_tokens = event.usage.output_tokens
                            stop_reason = event.delta.stop_reason if hasattr(event, 'delta') else None
                        elif event.type == 'message_start' and hasattr(event, 'message'):
                            u = event.message.usage
                            input_tokens = u.input_tokens
                            # cache_*_input_tokens are billed separately from input_tokens
                            cache_read_tokens = getattr(u, 'cache_read_input_tokens', 0) or 0
                            cache_write_tokens = getattr(u, 'cache_creation_input_tokens', 0) or 0
        except Exception as stream_err:
            # A mid-stream failure (overload, timeout, truncation) still burned
            # whatever tokens we saw before the break — surface them, don't drop them.
            return TranslationResult(
                source_file="", rust_code=_extract_rust_code(response_text), model=model,
                input_tokens=input_tokens, output_tokens=output_tokens,
                cache_read_tokens=cache_read_tokens, cache_write_tokens=cache_write_tokens,
                success=False, error=f"stream interrupted: {stream_err}",
            )

        rust_code = _extract_rust_code(response_text)

        if stop_reason == "max_tokens":
            print(f"  WARNING: output truncated at {output_tokens} tokens (stop_reason=max_tokens)", file=sys.stderr)

        return TranslationResult(
            source_file="",
            rust_code=rust_code,
            model=model,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            cache_read_tokens=cache_read_tokens,
            cache_write_tokens=cache_write_tokens,
            success=True,
        )

    except Exception as e:
        return TranslationResult(
            source_file="",
            rust_code="",
            model=model,
            input_tokens=0,
            output_tokens=0,
            success=False,
            error=str(e),
        )


def _get_gemini_oauth_token() -> str:
    """Get OAuth access token from gemini-cli's cached credentials."""
    oauth_path = Path.home() / ".gemini" / "oauth_creds.json"
    if not oauth_path.exists():
        raise RuntimeError("No OAuth credentials found. Run 'gemini' once to authenticate.")

    creds_data = json.loads(oauth_path.read_text())

    from google.oauth2.credentials import Credentials
    from google.auth.transport.requests import Request

    creds = Credentials(
        token=creds_data.get("access_token"),
        refresh_token=creds_data.get("refresh_token"),
        token_uri="https://oauth2.googleapis.com/token",
        client_id=creds_data.get("client_id"),
        client_secret=creds_data.get("client_secret"),
    )

    if creds.expired or not creds.valid:
        creds.refresh(Request())

    return creds.token


def _submit_gemini(prompt: str, model: str) -> TranslationResult:
    """Submit via Google GenAI Python SDK using gemini-cli OAuth credentials."""
    try:
        from google import genai

        access_token = _get_gemini_oauth_token()

        client = genai.Client(
            vertexai=False,
            api_key=None,
            http_options={"headers": {"Authorization": f"Bearer {access_token}"}},
        )

        response = client.models.generate_content(
            model=model,
            contents=prompt,
            config=genai.types.GenerateContentConfig(
                max_output_tokens=GEMINI_MAX_TOKENS,
                temperature=0.2,
            ),
        )

        response_text = response.text or ""
        rust_code = _extract_rust_code(response_text)

        usage = response.usage_metadata
        input_tokens = usage.prompt_token_count if usage else 0
        output_tokens = usage.candidates_token_count if usage else 0

        return TranslationResult(
            source_file="",
            rust_code=rust_code,
            model=model,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            success=True,
        )

    except Exception as e:
        return TranslationResult(
            source_file="",
            rust_code="",
            model=model,
            input_tokens=0,
            output_tokens=0,
            success=False,
            error=str(e),
        )


def submit_single(prompt: str, model: str = "", system_prompt: str = "",
                  file_context: str = "") -> TranslationResult:
    """Submit a single translation request.

    Auto-detects backend from model name or ANTHROPIC_API_KEY env var.
    ``file_context`` is the per-file shared context; on Claude it becomes a
    second cached system block, on Gemini (no caching) it's prepended inline.
    """
    backend = get_backend()

    if not model:
        model = DEFAULT_MODELS[backend]

    # Override backend based on model name
    if model.startswith("claude-"):
        backend = "claude"
    elif model.startswith("gemini-"):
        backend = "gemini"

    if backend == "claude":
        return _submit_claude(prompt, model, system_prompt, file_context)
    else:
        # Gemini doesn't support separate system prompt / caching via this path,
        # so we prepend both the rules and the file context to the prompt.
        prefix = "\n\n---\n\n".join(p for p in (system_prompt, file_context) if p)
        full_prompt = (prefix + "\n\n---\n\n" + prompt) if prefix else prompt
        return _submit_gemini(full_prompt, model)


def submit_batch(prompts: dict[str, str], model: str = "", system_prompt: str = "") -> dict[str, TranslationResult]:
    """Submit multiple translation requests sequentially.

    Args:
        prompts: dict of filename → prompt text
        model: model to use (auto-detected if empty)
        system_prompt: shared system prompt (cached for Claude)

    Returns:
        dict of filename → TranslationResult
    """
    results = {}
    total = len(prompts)

    if not model:
        model = DEFAULT_MODELS[get_backend()]

    for i, (filename, prompt) in enumerate(prompts.items(), 1):
        print(f"  [{i}/{total}] Submitting {filename}...", file=sys.stderr)
        result = submit_single(prompt, model=model, system_prompt=system_prompt)
        result.source_file = filename
        results[filename] = result

        if result.success:
            print(f"    ✓ {result.input_tokens} in / {result.output_tokens} out tokens", file=sys.stderr)
        else:
            print(f"    ✗ {result.error}", file=sys.stderr)

    return results


def _extract_rust_code(text: str) -> str:
    """Extract Rust code from markdown fencing if present."""
    lines = text.splitlines()
    in_fence = False
    rust_lines = []
    found_fence = False

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("```rust") or stripped == "```rs":
            in_fence = True
            found_fence = True
            continue
        if in_fence and stripped == "```":
            in_fence = False
            continue
        if in_fence:
            rust_lines.append(line)

    if found_fence and rust_lines:
        return "\n".join(rust_lines)

    # No fencing found — return as-is (might already be pure Rust)
    return text
