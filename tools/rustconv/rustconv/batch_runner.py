"""LLM API client — supports Claude (Anthropic) and Gemini (Google) backends."""

import json
import os
import sys
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path

import anthropic
from openai import OpenAI

from .attempts import Attempt, durable_json
from .config import (
    CLAUDE_MAX_TOKENS,
    DEEPSEEK_BASE_URL,
    DEEPSEEK_MAX_TOKENS,
    DEFAULT_MODELS,
    GEMINI_MAX_TOKENS,
    MODEL_PRICING,
    get_backend,
)


@dataclass
class TranslationResult:
    source_file: str
    rust_code: str
    model: str
    input_tokens: int  # UNCACHED input tokens only (full price)
    output_tokens: int
    success: bool
    error: str | None = None
    cache_read_tokens: int = 0  # served from cache (~0.1x price)
    cache_write_tokens: int = 0  # written to cache (~1.25x price)
    attempt_id: str | None = None
    usage_known: bool = False


# Per-1M-token USD pricing (input, output, cache_read, cache_write).
# Cache read = 0.1x input; cache write (5-min ephemeral) = 1.25x input.
PRICING = {
    "claude-opus-4-8": (5.00, 25.00, 0.50, 6.25),
    "claude-opus-4-7": (5.00, 25.00, 0.50, 6.25),
    "claude-opus-4-6": (5.00, 25.00, 0.50, 6.25),
    "claude-sonnet-4-5": (3.00, 15.00, 0.30, 3.75),
    "claude-sonnet-4-6": (3.00, 15.00, 0.30, 3.75),
    "claude-haiku-4-5": (1.00, 5.00, 0.10, 1.25),
    # The <=100K-prompt tier; estimate_cost() switches to LONG_PROMPT_PRICING above it.
    "claude-haiku-5-5": (0.10, 0.50, 0.01, 0.125),
}

# Models whose rates step up when ONE prompt (input + cache read + cache write) crosses a
# threshold: model -> (threshold tokens, rates above it). Haiku 5.5 verified 2026-10-07 at
# https://www.anthropic.com/claude/haiku (cache rates keep the 0.1x / 1.25x rule).
LONG_PROMPT_PRICING = {
    "claude-haiku-5-5": (100_000, (0.50, 2.50, 0.05, 0.625)),
}


def estimate_cost(
    model: str,
    input_tokens: int,
    output_tokens: int,
    cache_read_tokens: int = 0,
    cache_write_tokens: int = 0,
) -> float | None:
    """USD cost for one (or accumulated) call. None if the model isn't priced here."""
    rates = PRICING.get(model)
    if rates is None:
        # tolerate dated/full IDs by prefix match
        rates = next((v for k, v in PRICING.items() if model.startswith(k)), None)
    if rates is None:
        return None
    tier = next((v for k, v in LONG_PROMPT_PRICING.items() if model.startswith(k)), None)
    if tier is not None and input_tokens + cache_read_tokens + cache_write_tokens > tier[0]:
        rates = tier[1]
    pin, pout, pread, pwrite = rates
    return (
        input_tokens * pin
        + output_tokens * pout
        + cache_read_tokens * pread
        + cache_write_tokens * pwrite
    ) / 1_000_000


def _submit_claude(
    prompt: str, model: str, system_prompt: str = "", file_context: str = "", sink=None
) -> TranslationResult:
    """Submit via Anthropic Python SDK.

    Two cached system blocks form the stable prefix:
      1. system_prompt   — file-independent translation rules (cached across the
                            whole run)
      2. file_context    — per-file shared context (import superset, gap fns,
                            globals); byte-identical across all functions of one
                            file, so functions 2..N read it from cache (~0.1x)
                            instead of re-sending it uncached (1x).
    Only the function-specific `prompt` (the user turn) is full price each call.

    sink: optional ResponseSink. When given, every streamed delta is written to
    disk and flushed AS IT ARRIVES, so a process kill mid-stream never discards
    output you were already billed for.
    """
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
            system_blocks.append(
                {
                    "type": "text",
                    "text": system_prompt,
                    "cache_control": {"type": "ephemeral"},
                }
            )
        if file_context:
            system_blocks.append(
                {
                    "type": "text",
                    "text": file_context,
                    "cache_control": {"type": "ephemeral"},
                }
            )
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
                    if hasattr(event, "type"):
                        if event.type == "content_block_delta" and hasattr(event, "delta"):
                            if hasattr(event.delta, "text"):
                                # Only TextDelta carries .text; the hasattr guard above
                                # excludes the other delta variants in the union.
                                delta_text: str = event.delta.text  # type: ignore[union-attr]
                                response_text += delta_text
                                if sink is not None:
                                    sink.write(delta_text)  # flush to disk now
                        elif event.type == "message_delta" and hasattr(event, "usage"):
                            output_tokens = event.usage.output_tokens
                            stop_reason = (
                                event.delta.stop_reason if hasattr(event, "delta") else None
                            )
                        elif event.type == "message_start" and hasattr(event, "message"):
                            u = event.message.usage
                            input_tokens = u.input_tokens
                            # cache_*_input_tokens are billed separately from input_tokens
                            cache_read_tokens = getattr(u, "cache_read_input_tokens", 0) or 0
                            cache_write_tokens = getattr(u, "cache_creation_input_tokens", 0) or 0
        except Exception as stream_err:
            # A mid-stream failure (overload, timeout, truncation) still burned
            # whatever tokens we saw before the break — surface them, don't drop them.
            return TranslationResult(
                source_file="",
                rust_code=_extract_rust_code(response_text),
                model=model,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                cache_read_tokens=cache_read_tokens,
                cache_write_tokens=cache_write_tokens,
                success=False,
                error=f"stream interrupted: {stream_err}",
            )

        rust_code = _extract_rust_code(response_text)

        # Empty/truncated output is a FAILURE, not a cacheable success — see the
        # detailed note in _submit_deepseek. Same hazard applies to any backend.
        if not rust_code.strip():
            return TranslationResult(
                source_file="",
                rust_code="",
                model=model,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                cache_read_tokens=cache_read_tokens,
                cache_write_tokens=cache_write_tokens,
                success=False,
                error=f"empty output (stop_reason={stop_reason}, {output_tokens} output tokens)",
            )
        if stop_reason == "max_tokens":
            return TranslationResult(
                source_file="",
                rust_code=rust_code,
                model=model,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                cache_read_tokens=cache_read_tokens,
                cache_write_tokens=cache_write_tokens,
                success=False,
                error=(
                    f"truncated output at {output_tokens} tokens "
                    f"(stop_reason=max_tokens) — incomplete, not cached. Raise CLAUDE_MAX_TOKENS."
                ),
            )

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

    # optional gemini extra — not installed in the base env
    from google.auth.transport.requests import Request  # type: ignore[import-not-found]
    from google.oauth2.credentials import Credentials  # type: ignore[import-not-found]

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
        from google import genai  # type: ignore[attr-defined]  # optional gemini extra

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
                source_file="",
                rust_code="",
                model=model,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
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


def _submit_deepseek(
    prompt: str, model: str, system_prompt: str = "", file_context: str = "", sink=None
) -> TranslationResult:
    """Submit via the DeepSeek API (OpenAI-compatible).

    deepseek-chat = V3, deepseek-reasoner = R1. Uses the openai SDK pointed at
    DeepSeek's base URL. system_prompt + file_context become system messages
    (DeepSeek does automatic server-side prefix caching, so re-sending the
    stable prefix per function is cheap — no explicit cache_control needed).
    Streams deltas to `sink` so a kill mid-stream never discards paid output.
    """
    input_tokens = output_tokens = 0
    usage_known = False
    response_text = ""
    try:
        api_key = os.environ.get("DEEPSEEK_API_KEY")
        if not api_key:
            raise RuntimeError("DEEPSEEK_API_KEY not set (add it to .env)")

        client = OpenAI(api_key=api_key, base_url=DEEPSEEK_BASE_URL, max_retries=0)

        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        if file_context:
            messages.append({"role": "system", "content": file_context})
        messages.append({"role": "user", "content": prompt})

        response_text = ""
        input_tokens = 0
        cache_read_tokens = 0
        output_tokens = 0
        finish_reason = None

        stream = client.chat.completions.create(
            model=model,
            messages=messages,
            max_tokens=DEEPSEEK_MAX_TOKENS,
            temperature=0.0,
            stream=True,
            stream_options={"include_usage": True},
            extra_body={
                "thinking": {"type": os.environ.get("RUSTCONV_DEEPSEEK_THINKING", "enabled")}
            },
        )
        for chunk in stream:
            if chunk.choices:
                choice = chunk.choices[0]
                delta = choice.delta
                reasoning = getattr(delta, "reasoning_content", None) if delta is not None else None
                if reasoning and sink is not None and hasattr(sink, "write_reasoning"):
                    sink.write_reasoning(reasoning)
                content = getattr(delta, "content", None) if delta is not None else None
                if content:
                    response_text += content
                    if sink is not None:
                        sink.write(content)  # flush to disk now
                if choice.finish_reason:
                    finish_reason = choice.finish_reason
            # Usage arrives on the final chunk (choices empty) with include_usage.
            usage = getattr(chunk, "usage", None)
            if usage:
                usage_known = True
                # DeepSeek splits the prompt: `prompt_cache_hit_tokens` bill at ~1/30 of a
                # miss. Recorded apart so the cost is the bill, not a ceiling twice its size.
                hit = getattr(usage, "prompt_cache_hit_tokens", 0) or 0
                cache_read_tokens = hit
                input_tokens = usage.prompt_tokens - hit
                output_tokens = usage.completion_tokens

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
                source_file="",
                rust_code="",
                model=model,
                usage_known=usage_known,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                cache_read_tokens=cache_read_tokens,
                success=False,
                error=(
                    f"empty output (finish_reason={finish_reason}, "
                    f"{output_tokens} output tokens). For a reasoning model this "
                    f"usually means reasoning consumed the {DEEPSEEK_MAX_TOKENS}-token "
                    f"budget before any code was emitted — raise DEEPSEEK_MAX_TOKENS "
                    f"or split the unit."
                ),
            )
        if finish_reason == "length":
            return TranslationResult(
                source_file="",
                rust_code=rust_code,
                model=model,
                usage_known=usage_known,
                input_tokens=input_tokens,
                output_tokens=output_tokens,
                cache_read_tokens=cache_read_tokens,
                success=False,
                error=(
                    f"truncated output at {output_tokens} tokens "
                    f"(finish_reason=length) — incomplete; raise DEEPSEEK_MAX_TOKENS "
                    f"or split the unit. Not cached."
                ),
            )

        return TranslationResult(
            source_file="",
            rust_code=rust_code,
            model=model,
            usage_known=usage_known,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            cache_read_tokens=cache_read_tokens,
            success=True,
        )

    except Exception as e:
        return TranslationResult(
            source_file="",
            rust_code=_extract_rust_code(response_text),
            model=model,
            usage_known=usage_known,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            cache_read_tokens=cache_read_tokens,
            success=False,
            error=str(e),
        )


def submit_single(
    prompt: str, model: str = "", system_prompt: str = "", file_context: str = "", sink=None
) -> TranslationResult:
    """Preserve each purchase and reserve its maximum charge before contacting the API."""
    model = model or DEFAULT_MODELS[get_backend()]
    thinking = os.environ.get("RUSTCONV_DEEPSEEK_THINKING", "enabled")
    if thinking not in {"enabled", "disabled"}:
        raise ValueError("RUSTCONV_DEEPSEEK_THINKING must be enabled or disabled")
    budget_file = os.environ.get("RUSTCONV_BUDGET_FILE")
    budget_limit = os.environ.get("RUSTCONV_BUDGET_USD")
    if bool(budget_file) != bool(budget_limit):
        raise ValueError("Set both RUSTCONV_BUDGET_FILE and RUSTCONV_BUDGET_USD")
    budget = None
    upper = None
    pin = pout = Decimal(0)
    if budget_file:
        from .spend_guard import Budget

        if model not in {"deepseek-v4-pro", "deepseek-flash", "deepseek-v4-flash"}:
            raise ValueError(f"No verified reservation bound for {model}; refusing payment")
        if not 0 < DEEPSEEK_MAX_TOKENS <= 384000:
            raise ValueError("DEEPSEEK_MAX_TOKENS outside verified model limit")
        pin, pout = (Decimal(str(v)) for v in MODEL_PRICING[model])
        # Full documented context limit, not a tokenizer estimate. SDK retries disabled.
        upper = (Decimal(1048576) * pin + DEEPSEEK_MAX_TOKENS * pout) / 1000000
        assert budget_limit is not None
        budget = Budget(Path(budget_file), budget_limit)
    attempt = Attempt(
        {
            "model": model,
            "thinking": thinking,
            "prompt": prompt,
            "system_prompt": system_prompt,
            "file_context": file_context,
            "max_output_tokens": DEEPSEEK_MAX_TOKENS if model.startswith("deepseek-") else None,
            "legacy_sink": str(sink.path) if sink is not None and hasattr(sink, "path") else None,
            "reserved_upper_usd": str(upper) if upper is not None else None,
            "price_basis": "peak/cache-miss conservative estimate, verified 2026-09-22",
            "usd_per_million": MODEL_PRICING.get(model),
        },
        downstream=sink,
    )
    try:
        if budget is not None:
            assert upper is not None
            budget.reserve(attempt.id, upper)
        result = _submit_single_backend(prompt, model, system_prompt, file_context, sink=attempt)
        result.attempt_id = attempt.id
        attempt.finish(result)
        if budget is not None:
            actual: Decimal | None = None
            if result.usage_known:
                # The budget stays a CEILING: every prompt token at the peak miss rate,
                # cache hits included. The reported cost (config.cost_usd) is the bill.
                prompt_tokens = result.input_tokens + result.cache_read_tokens
                actual = (prompt_tokens * pin + result.output_tokens * pout) / 1000000
            budget.settle(attempt.id, actual)
            durable_json(
                attempt.path / "accounting.json",
                {
                    "schema": 1,
                    "attempt_id": attempt.id,
                    "estimated_usd": str(actual) if actual is not None else None,
                    "basis": "provider usage at reserved peak/cache-miss rates; not invoice",
                    "budget_file": budget_file,
                },
            )
        return result
    finally:
        attempt.close()


def _submit_single_backend(
    prompt: str, model: str = "", system_prompt: str = "", file_context: str = "", sink=None
) -> TranslationResult:
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


def submit_batch(
    prompts: dict[str, str], model: str = "", system_prompt: str = ""
) -> dict[str, TranslationResult]:
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
            print(
                f"    ✓ {result.input_tokens} in / {result.output_tokens} out tokens",
                file=sys.stderr,
            )
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
