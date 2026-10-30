import json
from pathlib import Path

import pytest
import textlane

# The conformance runner goes through `run_case`; this samples the suite through the public
# API instead, so a mapping the runner never sees cannot drift unnoticed

SUITE = Path(__file__).resolve().parents[2] / "conformance"
TESTS = SUITE / "tests"
PENDING = json.loads((SUITE / "pending.json").read_text(encoding="utf-8"))
SAMPLE_PER_FILE = 3
OUTPUTS = ["html", "md", "js", "txt"]


def is_pending(file_id, group_id, test_id):
    for key in PENDING:
        if key.endswith("*") and file_id.startswith(key[:-1]):
            return True
        if key in (file_id, group_id, test_id):
            return True
    return False


def collect_cases():
    for file in sorted(TESTS.rglob("*.json")):
        file_id = file.relative_to(TESTS).with_suffix("").as_posix()
        sample = []
        for group in json.loads(file.read_text(encoding="utf-8")):
            group_id = f"{file_id}#{group['description']}"
            for test in group["tests"]:
                test_id = f"{group_id}#{test['description']}"
                # host plugins need the runner's stand-ins: conformance_unit.py runs them publicly
                hosted = any(str(entry[0]).startswith("host:") for entry in group["pipeline"])
                if "input" in test and "operations" not in test and not hosted:
                    if not is_pending(file_id, group_id, test_id):
                        sample.append(pytest.param(group, test, id=test_id))
        yield from sample[:SAMPLE_PER_FILE]


@pytest.mark.parametrize("group,test", list(collect_cases()))
def test_api(group, test):
    plugins = [{"name": entry[0], "options": entry[1]} if len(entry) > 1 else {"name": entry[0]} for entry in group["pipeline"]]

    def processor():
        return textlane.textlane().use(plugins)

    source = test["input"]
    expected = test["expected"]
    output = next((key for key in OUTPUTS if key in expected), None)
    if "error" in expected:
        with pytest.raises(textlane.TextlaneError) as error:
            if output:
                processor().process(source)
            else:
                processor().run(processor().parse(source), source)
        assert error.value.code == expected["error"]["code"]
        return
    if output:
        assert processor().process(source)["value"] == expected[output]
        return
    # What plugins stored in the file is the vfile's `data`
    if "fileData" in expected:
        assert processor().process_sync(source)["data"] == expected["fileData"]
        return
    if "tree" in expected:
        tree = processor().run(processor().parse(source), source)
        # A tree read back from JSON keeps unist positions, which a bridge into nlcst cannot
        # place its nodes by: only `process` keeps them in that case
        bridged = any(entry[0].endswith(".toNaturalLanguage") for entry in group["pipeline"])
        if group.get("positions") is False or bridged:
            assert strip_positions(tree) == strip_positions(expected["tree"])
        else:
            assert tree == expected["tree"]
        return
    # Messages-only cases whose pipeline has no compiler are the runner's alone
    if not any(entry[0].endswith("stringify") for entry in group["pipeline"]):
        return
    messages = [
        {key: message[key] for key in ("ruleId", "source", "place")}
        for message in processor().process(source)["messages"]
    ]
    assert messages == expected["messages"]


def strip_positions(value):
    if isinstance(value, dict):
        return {key: strip_positions(item) for key, item in value.items() if key != "position"}
    if isinstance(value, list):
        return [strip_positions(item) for item in value]
    return value
