import pytest

from rustconv.argument_facts import require_argument_facts
from rustconv.call_seal import CallSealError, seal_calls
from rustconv.fact_seal import seal_facts


def test_permutation_with_every_token_present_is_rejected():
    facts = seal_facts("first(10); second(20);", dialect="c")
    calls = seal_calls(facts.text, {"first", "second"})
    a, b = facts.facts
    first, second = calls.calls
    with pytest.raises(CallSealError, match="first argument 1"):
        require_argument_facts(
            f"{first.placeholder}({b.placeholder}); {second.placeholder}({a.placeholder});", calls
        )


def test_constant_cast_and_declaration_are_accepted():
    facts = seal_facts("math_error(_DOMAIN, x);", dialect="c")
    calls = seal_calls(facts.text, {"math_error"})
    token = calls.calls[0].placeholder
    require_argument_facts(
        f'extern "C" {{ fn {token}(kind: i32, x: f64); }}\n'
        f"{token}({facts.facts[0].placeholder} as i32, x);",
        calls,
    )
