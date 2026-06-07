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
    DEEPSEEK_BASE_URL, DEEPSEEK_MAX_TOKENS,
)


@dataclass
class TranslationResult:
    source_file: str
    rust_code: str
    model: str
    input_tokens: int
    output_tokens: int
    success: bool
    error: Optional[str] = None


def _submit_claude(prompt: str, model: str, system_prompt: str = "",
                   file_context: str = "", sink=None) -> TranslationResult:
    """Submit via Anthropic Python SDK.

    sink: optional ResponseSink. When given, every streamed delta is written to
    disk and flushed AS IT ARRIVES, so a process kill mid-stream never discards
    output you were already billed for.
    """
    import anthropic

    try:
        client = anthropic.Anthropic()  # uses ANTHROPIC_API_KEY env var

        kwargs = {
            "model": model,
            "max_tokens": CLAUDE_MAX_TOKENS,
            "messages": [{"role": "user", "content": prompt}],
        }

        # Up to TWO cached system blocks form the stable prefix:
        #   1. system_prompt — file-independent translation rules
        #   2. file_context  — per-file shared context (import superset, gap fns,
        #      globals); byte-identical across all functions of one file, so
        #      functions 2..N read it from cache (~0.1x) instead of re-sending it
        #      uncached (1x). Only the per-function `prompt` is full price.
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
        stop_reason = None

        with client.messages.stream(**kwargs) as stream:
            for event in stream:
                if hasattr(event, 'type'):
                    if event.type == 'content_block_delta' and hasattr(event, 'delta'):
                        if hasattr(event.delta, 'text'):
                            response_text += event.delta.text
                            if sink is not None:
                                sink.write(event.delta.text)  # flush to disk now
                    elif event.type == 'message_delta' and hasattr(event, 'usage'):
                        output_tokens = event.usage.output_tokens
                        stop_reason = event.delta.stop_reason if hasattr(event, 'delta') else None
                    elif event.type == 'message_start' and hasattr(event, 'message'):
                        input_tokens = event.message.usage.input_tokens

        rust_code = _extract_rust_code(response_text)

        # Empty/truncated output is a FAILURE, not a cacheable success — see the
        # detailed note in _submit_deepseek. Same hazard applies to any backend.
        if not rust_code.strip():
            return TranslationResult(
                source_file="", rust_code="", model=model,
                input_tokens=input_tokens, output_tokens=output_tokens,
                success=False,
                error=f"empty output (stop_reason={stop_reason}, {output_tokens} output tokens)",
            )
        if stop_reason == "max_tokens":
            return TranslationResult(
                source_file="", rust_code=rust_code, model=model,
                input_tokens=input_tokens, output_tokens=output_tokens,
                success=False,
                error=(f"truncated output at {output_tokens} tokens "
                       f"(stop_reason=max_tokens) — incomplete, not cached. Raise CLAUDE_MAX_TOKENS."),
            )

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

        # Empty output is a FAILURE, not a cacheable success — see the detailed
        # note in _submit_deepseek.
        if not rust_code.strip():
            return TranslationResult(
                source_file="", rust_code="", model=model,
                input_tokens=input_tokens, output_tokens=output_tokens,
                success=False,
                error=f"empty output ({output_tokens} output tokens)",
            )

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


def _submit_deepseek(prompt: str, model: str, system_prompt: str = "",
                     file_context: str = "", sink=None) -> TranslationResult:
    """Submit via the DeepSeek API (OpenAI-compatible).

    deepseek-chat = V3, deepseek-reasoner = R1. Uses the openai SDK pointed at
    DeepSeek's base URL. system_prompt + file_context become system messages
    (DeepSeek does automatic server-side prefix caching, so re-sending the
    stable prefix per function is cheap — no explicit cache_control needed).
    Streams deltas to `sink` so a kill mid-stream never discards paid output.
    """
    from openai import OpenAI

    try:
        api_key = os.environ.get("DEEPSEEK_API_KEY")
        if not api_key:
            raise RuntimeError("DEEPSEEK_API_KEY not set (add it to .env)")

        client = OpenAI(api_key=api_key, base_url=DEEPSEEK_BASE_URL)

        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        if file_context:
            messages.append({"role": "system", "content": file_context})
        messages.append({"role": "user", "content": prompt})

        response_text = ""
        input_tokens = 0
        output_tokens = 0
        finish_reason = None

        stream = client.chat.completions.create(
            model=model,
            messages=messages,
            max_tokens=DEEPSEEK_MAX_TOKENS,
            temperature=0.0,
            stream=True,
            stream_options={"include_usage": True},
        )
        for chunk in stream:
            if chunk.choices:
                choice = chunk.choices[0]
                delta = choice.delta
                if delta is not None and getattr(delta, "content", None):
                    response_text += delta.content
                    if sink is not None:
                        sink.write(delta.content)  # flush to disk now
                if choice.finish_reason:
                    finish_reason = choice.finish_reason
            # Usage arrives on the final chunk (choices empty) with include_usage.
            if getattr(chunk, "usage", None):
                input_tokens = chunk.usage.prompt_tokens
                output_tokens = chunk.usage.completion_tokens

        rust_code = _extract_rust_code(response_text)

        # An EMPTY or TRUNCATED translation is a FAILURE, never a cacheable
        # result. DeepSeek v4 reasoning models can spend the entire max_tokens
        # budget on reasoning (which streams as `reasoning_content`, not the
        # `content` we capture) and emit NO code at all — or get cut off
        # mid-output. Returning success here caches the empty/partial string as
        # a valid $0 entry and SILENTLY drops the unit forever (this is exactly
        # what wiped the mlang/cabinet_main type preambles + map_font). Make it
        # loud and retriable instead.
        if not rust_code.strip():
            return TranslationResult(
                source_file="", rust_code="", model=model,
                input_tokens=input_tokens, output_tokens=output_tokens,
                success=False,
                error=(f"empty output (finish_reason={finish_reason}, "
                       f"{output_tokens} output tokens). For a reasoning model this "
                       f"usually means reasoning consumed the {DEEPSEEK_MAX_TOKENS}-token "
                       f"budget before any code was emitted — raise DEEPSEEK_MAX_TOKENS "
                       f"or split the unit."),
            )
        if finish_reason == "length":
            return TranslationResult(
                source_file="", rust_code=rust_code, model=model,
                input_tokens=input_tokens, output_tokens=output_tokens,
                success=False,
                error=(f"truncated output at {output_tokens} tokens "
                       f"(finish_reason=length) — incomplete; raise DEEPSEEK_MAX_TOKENS "
                       f"or split the unit. Not cached."),
            )

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
                  file_context: str = "", sink=None) -> TranslationResult:
    """Submit a single translation request.

    Auto-detects backend from model name or ANTHROPIC_API_KEY env var.

    file_context: optional per-file shared context; on Claude it becomes a
        second cached system block (read at ~0.1x across a file's functions),
        on DeepSeek it's a second system message (server-side prefix caching),
        on Gemini (no caching) it's prepended inline.
    sink: optional ResponseSink for durable stream-to-disk (Claude/DeepSeek).
    """
    backend = get_backend()

    if not model:
        model = DEFAULT_MODELS[backend]

    # Override backend based on model name
    if model.startswith("claude-"):
        backend = "claude"
    elif model.startswith("deepseek-"):
        backend = "deepseek"
    elif model.startswith("gemini-"):
        backend = "gemini"

    if backend == "claude":
        return _submit_claude(prompt, model, system_prompt, file_context, sink=sink)
    elif backend == "deepseek":
        return _submit_deepseek(prompt, model, system_prompt, file_context, sink=sink)
    else:
        # Gemini has no separate system prompt / caching here, so prepend both
        # the rules and the per-file context to the prompt.
        prefix = "\n\n---\n\n".join(p for p in (system_prompt, file_context) if p)
        full_prompt = (prefix + "\n\n---\n\n" + prompt) if prefix else prompt
        result = _submit_gemini(full_prompt, model)
        # Gemini isn't streamed here, but still persist the result durably.
        if sink is not None and result.rust_code:
            sink.write(result.rust_code)
        return result


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
