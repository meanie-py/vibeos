from __future__ import annotations

import json
import multiprocessing
import tempfile
import unittest
from decimal import Decimal
from pathlib import Path

from rustconv.spend_guard import Budget, BudgetExceeded, BudgetOverrun, BudgetStateError


def _reserve_worker(path: str, index: int, queue) -> None:
    try:
        Budget(Path(path), "3.00").reserve(f"attempt-{index}", "0.75")
        queue.put("reserved")
    except BudgetExceeded:
        queue.put("blocked")


class BudgetTests(unittest.TestCase):
    def test_multiprocess_reservations_never_cross_cap(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "ledger.json"
            ctx = multiprocessing.get_context("spawn")
            queue = ctx.Queue()
            workers = [
                ctx.Process(target=_reserve_worker, args=(str(path), i, queue)) for i in range(12)
            ]
            for process in workers:
                process.start()
            for process in workers:
                process.join(10)
                self.assertEqual(process.exitcode, 0)
            results = [queue.get(timeout=2) for _ in workers]
            self.assertEqual(results.count("reserved"), 4)
            self.assertEqual(results.count("blocked"), 8)
            snapshot = json.loads(path.read_text())
            total = sum(Decimal(value) for value in snapshot["reservations"].values())
            self.assertLessEqual(total, Decimal("3.00"))

    def test_unknown_usage_keeps_reservation_across_reopen(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "ledger.json"
            budget = Budget(path, "1.00")
            budget.reserve("lost-usage", "0.80")
            self.assertIsNone(budget.settle("lost-usage", None))
            reopened = Budget(path, "1.00")
            with self.assertRaises(BudgetExceeded):
                reopened.reserve("next", "0.21")
            self.assertEqual(reopened.snapshot()["reservations"], {"lost-usage": "0.80"})

    def test_duplicate_ids_rejected_before_and_after_settlement(self):
        with tempfile.TemporaryDirectory() as temp:
            budget = Budget(Path(temp) / "ledger.json", "1")
            budget.reserve("same", "0.4")
            with self.assertRaises(BudgetStateError):
                budget.reserve("same", "0.1")
            budget.settle("same", "0.3")
            with self.assertRaises(BudgetStateError):
                budget.reserve("same", "0.1")
            with self.assertRaises(BudgetStateError):
                budget.settle("same", "0.3")

    def test_actual_over_reservation_records_debt_then_blocks(self):
        with tempfile.TemporaryDirectory() as temp:
            budget = Budget(Path(temp) / "ledger.json", "1")
            budget.reserve("overrun", "0.2")
            with self.assertRaises(BudgetOverrun):
                budget.settle("overrun", "0.25")
            snapshot = budget.snapshot()
            self.assertEqual(snapshot["spent_usd"], "0.25")
            self.assertEqual(snapshot["debt_usd"], "0.05")
            with self.assertRaises(BudgetExceeded):
                budget.reserve("blocked", "0")

    def test_changed_cap_and_corrupt_json_are_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "ledger.json"
            Budget(path, "1.00")
            with self.assertRaises(BudgetStateError):
                Budget(path, "1.01")
            path.write_text('{"broken":', encoding="utf-8")
            with self.assertRaises(BudgetStateError):
                Budget(path, "1.00")

    def test_amount_validation_rejects_nonfinite_negative_and_malformed(self):
        with tempfile.TemporaryDirectory() as temp:
            for amount in ("NaN", "Infinity", "-0.01", "not-money"):
                with self.subTest(amount=amount), self.assertRaises(ValueError):
                    Budget(Path(temp) / f"{amount}.json", amount)

    def test_known_usage_releases_unused_reservation(self):
        with tempfile.TemporaryDirectory() as temp:
            budget = Budget(Path(temp) / "ledger.json", "1")
            budget.reserve("done", "0.80")
            budget.settle("done", "0.30")
            budget.reserve("after", "0.70")
            snapshot = budget.snapshot()
            self.assertEqual(snapshot["spent_usd"], "0.30")
            self.assertEqual(snapshot["reservations"]["after"], "0.70")


if __name__ == "__main__":
    unittest.main()
