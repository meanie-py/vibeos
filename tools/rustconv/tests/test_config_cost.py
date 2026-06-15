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


class TestResolveModel:
    def test_none_falls_back_to_backend_default(self):
        assert resolve_model(None) == config.DEFAULT_MODELS[config.get_backend()]

    def test_known_alias_expands(self):
        assert resolve_model("opus").startswith("claude-opus")
        assert resolve_model("sonnet").startswith("claude-sonnet")

    def test_unknown_passes_through_unchanged(self):
        assert resolve_model("some-future-model-v9") == "some-future-model-v9"

    def test_full_id_passes_through(self):
        assert resolve_model("claude-opus-4-8") == "claude-opus-4-8"
