import json
from pathlib import Path

import pytest
import textlane

TESTS = Path(__file__).resolve().parents[2] / "conformance" / "tests"

# "<area>/<file>#<group>" to the open issue whose fix unblocks it; nothing else skips
KNOWN_DIVERGENCES: dict = {}


def collect_cases():
    for file in sorted(TESTS.rglob("*.json")):
        file_id = file.relative_to(TESTS).with_suffix("").as_posix()
        for group in json.loads(file.read_text(encoding="utf-8")):
            group_id = f"{file_id}#{group['description']}"
            if group_id in KNOWN_DIVERGENCES:
                continue
            for test in group["tests"]:
                yield pytest.param(group, test, id=f"{group_id} / {test['description']}")


@pytest.mark.parametrize("group,test", list(collect_cases()))
def test_conformance(group, test):
    case = {"pipeline": group["pipeline"]}
    for key in ("input", "tree"):
        if key in test:
            case[key] = test[key]
    actual = json.loads(textlane.run_case(json.dumps(case)))
    expected = test["expected"]
    # An expected error matches on its code, and on its message only as a substring
    if "error" in expected:
        error = actual.get("error", {})
        assert error.get("code") == expected["error"]["code"]
        assert expected["error"].get("message", "") in error.get("message", "")
    else:
        assert actual == expected
