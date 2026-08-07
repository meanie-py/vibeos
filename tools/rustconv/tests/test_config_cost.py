"""Cost/model resolution — pure arithmetic the `estimate`/`translate` banners
depend on. Pinned as invariants (linear in tokens, batch halves, unknown model
over-estimates) so they survive price-table edits without churning the test."""

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

    def test_deepseek_key_wins_over_anthropic(self, monkeypatch):
        monkeypatch.setenv("DEEPSEEK_API_KEY", "x")
        monkeypatch.setenv("ANTHROPIC_API_KEY", "y")
        assert config.get_backend() == "deepseek"
        assert resolve_model(None) == config.DEEPSEEK_MODEL_V4_PRO

    def test_falls_back_to_claude_without_deepseek_key(self, monkeypatch):
        monkeypatch.delenv("DEEPSEEK_API_KEY", raising=False)
        monkeypatch.setenv("ANTHROPIC_API_KEY", "y")
        assert config.get_backend() == "claude"

    def test_every_backend_has_a_default_model(self):
        # A backend get_backend() can return but DEFAULT_MODELS lacks is a KeyError at the
        # first paid call — the failure mode that kept "deepseek" unreachable.
        for backend in ("deepseek", "claude", "gemini"):
            assert backend in config.DEFAULT_MODELS

    def test_default_model_is_priced(self):
        # An unpriced default silently over-estimates to Opus in the pre-payment banner.
        assert config.DEEPSEEK_MODEL_V4_PRO in config.MODEL_PRICING
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
