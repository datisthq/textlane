import json
import sys
import time
from collections.abc import Callable


def is_gfm() -> bool:
    """Whether the third argument, the case's flavor, asks for GFM."""
    return sys.argv[3:4] == ["gfm"]


def run_adapter(convert: Callable[[str], str]) -> None:
    """Convert `<input> <repeat>` times and print `{elapsedMs, outputBytes}`; imports and one
    warm-up conversion are untimed."""
    path, repeat = sys.argv[1:3]
    with open(path, encoding="utf-8") as file:
        markdown = file.read()
    html = convert(markdown)
    started = time.perf_counter()
    for _ in range(int(repeat)):
        html = convert(markdown)
    elapsed_ms = (time.perf_counter() - started) * 1000
    print(json.dumps({"elapsedMs": elapsed_ms, "outputBytes": len(html.encode())}))
