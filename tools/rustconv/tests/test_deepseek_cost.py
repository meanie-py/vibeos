"""DeepSeek cost = cache-hit, cache-miss and output rates, halved off-peak (UTC bands)."""

import datetime as dt

import pytest

from rustconv.config import cost_usd, deepseek_peak

UTC = dt.timezone.utc
PEAK = dt.datetime(2026, 10, 5, 7, 0, tzinfo=UTC)  # Monday 07:00 UTC
OFF = dt.datetime(2026, 10, 5, 18, 0, tzinfo=UTC)  # Monday 18:00 UTC (setuplib's runs)


def test_bands():
    assert deepseek_peak(PEAK) and deepseek_peak(dt.datetime(2026, 10, 5, 1, 0, tzinfo=UTC))
    assert not deepseek_peak(OFF) and not deepseek_peak(dt.datetime(2026, 10, 5, 4, 0, tzinfo=UTC))
    assert not deepseek_peak(dt.datetime(2026, 10, 3, 7, 0, tzinfo=UTC))  # Saturday


def test_hits_and_off_peak_are_priced_apart():
    # 1M miss + 1M hit + 1M out at peak: 1.32 + 0.044 + 3.96
    assert cost_usd("deepseek-v4-pro", 10**6, 10**6, 10**6, when=PEAK) == pytest.approx(5.324)
    assert cost_usd("deepseek-v4-pro", 10**6, 10**6, 10**6, when=OFF) == pytest.approx(2.662)
    # other models are unchanged: hits counted at the input rate as before
    assert cost_usd("claude-haiku-4-5-20251001", 10**6, 0, 0) == pytest.approx(1.0)
