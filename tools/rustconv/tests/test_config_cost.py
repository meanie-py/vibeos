"""Cost/model resolution — pure arithmetic the `estimate`/`translate` banners
depend on. Pinned as invariants (linear in tokens, batch halves, unknown model
over-estimates) so they survive price-table edits without churning the test."""

import pytest

from rustconv import config
from rustconv.__main__ import resolve_model


class TestCostUsd:
    def test_linear_in_tokens(self):
        one = config.cost_usd("claude-opus-4-8", 1000, 1000)
        two = config.cost_usd("claude-opus-4-8", 2000, 2000)
        assert two == 2 * one > 0

    def test_batch_applies_half_discount(self):
        full = config.cost_usd("claude-opus-4-8", 5000, 3000, batch=False)
        batch = config.cost_usd("claude-opus-4-8", 5000, 3000, batch=True)
        assert batch == full * config.BATCH_DISCOUNT

    def test_zero_tokens_zero_cost(self):
        assert config.cost_usd("claude-opus-4-8", 0, 0) == 0.0

    def test_tiered_model_steps_up_per_prompt(self):
        # Haiku 5.5 bills a prompt over 100K tokens (uncached + cached) at the long rate; the
        # step is per call, so 100K exactly stays short and one token more goes long.
        threshold, short, long = config.CLAUDE_TIERED_PRICING["claude-haiku-5-5"]
        at = config.cost_usd("claude-haiku-5-5", threshold, 1000)
        assert at == pytest.approx((threshold * short[0] + 1000 * short[1]) / 1e6)
        over = config.cost_usd("claude-haiku-5-5", threshold - 10, 1000, cache_read_tokens=11)
        expect = ((threshold - 10) * long[0] + 11 * long[0] * 0.1 + 1000 * long[1]) / 1e6
        assert over == pytest.approx(expect)
        assert long[0] > short[0] and long[1] > short[1]

    def test_claude_cache_reads_and_writes_bill_at_their_own_rates(self):
        # 0.1x for a read, 1.25x for a write -- the same rule batch_runner.estimate_cost uses.
        from rustconv.batch_runner import estimate_cost

        args = ("claude-haiku-5-5", 2000, 500)
        got = config.cost_usd(*args, cache_read_tokens=150_000, cache_write_tokens=8000)
        assert got == pytest.approx(estimate_cost(*args, 150_000, 8000))
        pin = config.CLAUDE_TIERED_PRICING["claude-haiku-5-5"][2][0]
        assert config.cost_usd("claude-haiku-5-5", 0, 0, 200_000) == pytest.approx(
            200_000 * pin * 0.1 / 1e6
        )

    def test_batch_runner_tier_matches(self):
        from rustconv.batch_runner import LONG_PROMPT_PRICING, estimate_cost

        threshold, long = LONG_PROMPT_PRICING["claude-haiku-5-5"]
        short = estimate_cost("claude-haiku-5-5", threshold, 0)
        over = estimate_cost("claude-haiku-5-5", threshold + 1, 0)
        assert over == pytest.approx((threshold + 1) * long[0] / 1e6)
        assert short == pytest.approx(threshold * config.MODEL_PRICING["claude-haiku-5-5"][0] / 1e6)

    def test_haiku_alias_is_the_newest_haiku(self):
        assert resolve_model("haiku") == "claude-haiku-5-5"
        assert resolve_model("haiku-4-5") == "claude-haiku-4-5-20251001"

    def test_unknown_model_over_estimates_to_opus(self):
        # Unknown → Opus pricing (the expensive tier), never a lowball.
        assert config.model_pricing("totally-made-up") == config.model_pricing("claude-opus-4-8")


class TestBackendDefault:
    """The unflagged default must land on the cheap lane.

    Not a style preference — this is the money path. When DEFAULT_MODELS had no "deepseek"
    key, `-m deepseek` was the only route to it, and the shell32 lane drifted to 69 Sonnet
    runs against 1 DeepSeek (382 calls, $3.81 where DeepSeek was $0.31). A tautological
    "default == DEFAULT_MODELS[get_backend()]" assertion cannot catch that, so pin the
    VALUE and the precedence.
    """

    def test_anthropic_key_wins_and_defaults_to_haiku_5_5(self, monkeypatch):
        # Haiku 5.5 is the translation default since 2026-10-09: cheaper per call than
        # DeepSeek v4-pro and less integration rework (rustconv-pipeline.md).
        monkeypatch.setenv("DEEPSEEK_API_KEY", "x")
        monkeypatch.setenv("ANTHROPIC_API_KEY", "y")
        assert config.get_backend() == "claude"
        assert resolve_model(None) == "claude-haiku-5-5"

    def test_falls_back_to_deepseek_without_anthropic_key(self, monkeypatch):
        monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
        monkeypatch.setenv("DEEPSEEK_API_KEY", "x")
        assert config.get_backend() == "deepseek"
        assert resolve_model(None) == config.DEEPSEEK_MODEL_V4_PRO

    def test_every_backend_has_a_default_model(self):
        # A backend get_backend() can return but DEFAULT_MODELS lacks is a KeyError at the
        # first paid call — the failure mode that kept "deepseek" unreachable.
        for backend in ("deepseek", "claude", "gemini"):
            assert backend in config.DEFAULT_MODELS

    def test_default_model_is_priced(self):
        # An unpriced default silently over-estimates to Opus in the pre-payment banner.
        assert config.DEEPSEEK_MODEL_V4_PRO in config.MODEL_PRICING
        assert config.model_pricing("claude-haiku-5-5") < config.model_pricing(
            config.DEEPSEEK_MODEL_V4_PRO
        )
        assert config.model_pricing(config.DEEPSEEK_MODEL_V4_PRO) < config.model_pricing(
            config.CLAUDE_MODEL_SONNET
        )

    def test_alias_and_constant_agree(self):
        # MODEL_ALIASES["deepseek"] and DEEPSEEK_MODEL_V4_PRO are two spellings of one
        # decision; a drift between them makes `-m deepseek` and the default disagree.
        assert resolve_model("deepseek") == config.DEEPSEEK_MODEL_V4_PRO


class TestResolveModel:
    def test_none_falls_back_to_backend_default(self):
        assert resolve_model(None) == config.DEFAULT_MODELS[config.get_backend()]

    def test_escalation_aliases_still_reach_claude(self):
        # Switching the default must not strand the documented escalation route.
        assert resolve_model("sonnet") == config.CLAUDE_MODEL_SONNET
        assert resolve_model("opus").startswith("claude-opus")

    def test_known_alias_expands(self):
        assert resolve_model("opus").startswith("claude-opus")
        assert resolve_model("sonnet").startswith("claude-sonnet")

    def test_unknown_passes_through_unchanged(self):
        assert resolve_model("some-future-model-v9") == "some-future-model-v9"

    def test_full_id_passes_through(self):
        assert resolve_model("claude-opus-4-8") == "claude-opus-4-8"


def test_concurrency_gate_is_by_price_not_vendor():
    from rustconv.__main__ import _cheap_enough_to_fan_out

    assert _cheap_enough_to_fan_out("deepseek-v4-pro")
    assert _cheap_enough_to_fan_out("claude-haiku-5-5")
    assert not _cheap_enough_to_fan_out("claude-sonnet-4-6")
    assert not _cheap_enough_to_fan_out("no-such-model")
