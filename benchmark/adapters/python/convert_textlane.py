import json

import textlane
from common import run_adapter

PIPELINE = [
    ["md.parse"],
    ["md.toHtml", {"allowDangerousHtml": True}],
    ["html.stringify", {"allowDangerousHtml": True}],
]


def convert(markdown: str) -> str:
    # the bindings expose no pipeline yet, so this drives the `testing` build's case runner
    result = json.loads(textlane.run_case(json.dumps({"pipeline": PIPELINE, "input": markdown})))
    if "html" not in result:
        raise RuntimeError(result.get("error", "no html"))
    return result["html"]


if __name__ == "__main__":
    run_adapter(convert)
