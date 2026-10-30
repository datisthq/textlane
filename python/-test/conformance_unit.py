import asyncio
import json
from pathlib import Path

import pytest
import textlane
from hosts import create_hosts

SUITE = Path(__file__).resolve().parents[2] / "conformance"
TESTS = SUITE / "tests"

# "<area>/<file>#<group>" to the open issue whose fix unblocks it in this runner alone;
# what the crate does not implement yet is conformance/pending.json, shared by every runner
KNOWN_DIVERGENCES: dict = {}
PENDING = json.loads((SUITE / "pending.json").read_text(encoding="utf-8"))
matched_entries = set()
failing_entries = set()


# An entry names a file, a group, one test as `<group>#<test>`, or an area as `<area>/*`
def matches_entry(entry, file_id, group_id, test_id):
    if entry.endswith("*"):
        return file_id.startswith(entry[:-1])
    return entry in (file_id, group_id, test_id)


def collect_cases():
    for file in sorted(TESTS.rglob("*.json")):
        file_id = file.relative_to(TESTS).with_suffix("").as_posix()
        for group in json.loads(file.read_text(encoding="utf-8")):
            group_id = f"{file_id}#{group['description']}"
            if group_id in KNOWN_DIVERGENCES:
                continue
            for test in group["tests"]:
                test_id = f"{group_id}#{test['description']}"
                entry = next(
                    (key for key in PENDING if matches_entry(key, file_id, group_id, test_id)),
                    None,
                )
                if entry:
                    matched_entries.add(entry)
                yield pytest.param(group, test, entry, id=test_id)


@pytest.mark.parametrize("group,test,entry", list(collect_cases()))
def test_conformance(group, test, entry):
    actual = run_hosted(group, test) if is_hosted(group) else run_native(group, test)
    if group.get("positions") is False:
        actual = strip_positions(actual)
    if not entry:
        assert_outcome(actual, test["expected"])
        return
    try:
        assert_outcome(actual, test["expected"])
    except AssertionError:
        failing_entries.add(entry)


def run_native(group, test):
    case = {"pipeline": group["pipeline"]}
    for key in ("input", "tree", "operations"):
        if key in test:
            case[key] = test[key]
    return json.loads(textlane.run_case(json.dumps(case)))


# `host:` plugins stand for Python ones, so their groups go through the public API with this
# runner's stand-ins, processing asynchronously as the Rust runner does
def is_hosted(group):
    return any(str(entry[0]).startswith("host:") for entry in group["pipeline"])


def run_hosted(group, test):
    processor = textlane.textlane()
    host_for = create_hosts()
    for name, *options in group["pipeline"]:
        processor.use(host_for(name) if name.startswith("host:") else name, *options)
    try:
        file = asyncio.run(processor.process(test.get("input", "")))
    except Exception as error:
        return {"error": {"code": getattr(error, "code", "unknown"), "message": str(error)}}
    messages = [
        {"ruleId": message["ruleId"], "source": message["source"], "place": message["place"]}
        for message in file["messages"]
    ]
    outputs = {key: file["value"] for key in ("html", "md", "js", "txt")}
    return {**outputs, "data": processor.data(), "messages": messages}


# A group with `positions: false` compares trees without them
def strip_positions(value):
    if isinstance(value, dict):
        return {key: strip_positions(item) for key, item in value.items() if key != "position"}
    if isinstance(value, list):
        return [strip_positions(item) for item in value]
    return value


# A pending entry whose cases all pass is stale and fails the run
def test_pending_lists_only_failing_cases():
    assert sorted(matched_entries - failing_entries) == []


def assert_outcome(actual, expected):
    # A run may report several outcomes; the one `expected` names is compared, and an unexpected
    # error always fails. An expected error matches on its code, its message only as a substring
    if "error" in expected:
        error = actual.get("error", {})
        assert error.get("code") == expected["error"]["code"]
        assert expected["error"].get("message", "") in error.get("message", "")
    else:
        assert "error" not in actual
        for key, value in expected.items():
            assert actual.get(key) == value