"""Python plugins: what they get as the file, and the loop that runs them when a native run
pauses before them."""

import inspect
import json
from typing import Any, Callable, Dict, List, Optional

from . import _native
from ._models import Message, Place, Point

Transformer = Callable[[Dict[str, Any], "HostFile"], Any]
"""What an attacher returns: it changes the tree in place or returns a new one, directly or
as an awaitable."""


class HostFile:
    """The file a Python plugin gets, as a vfile: the document, its data and messages."""

    def __init__(self, value: Optional[str]) -> None:
        self.value = value
        self.data: Dict[str, Any] = {}
        self.messages: List[Message] = []

    def message(
        self,
        reason: str,
        place: Any = None,
        rule_id: Optional[str] = None,
        source: Optional[str] = None,
    ) -> Message:
        """Report a warning against the file; `place` is a point, a position or a node."""
        message: Message = {
            "reason": reason,
            "ruleId": rule_id,
            "source": source,
            "place": _read_place(place),
        }
        self.messages.append(message)
        return message

    def fail(
        self,
        reason: str,
        place: Any = None,
        rule_id: Optional[str] = None,
        source: Optional[str] = None,
    ) -> None:
        """Report an error and raise it."""
        self.message(reason, place, rule_id, source)
        error = _native.TextlaneError(reason)
        error.code = "plugin-failed"
        raise error


class Host:
    """A Python plugin a processor uses: its attacher, options and, once attached, its
    transformer."""

    def __init__(self, attacher: Callable[..., Any], parameters: List[Any]) -> None:
        self.attacher = attacher
        self.parameters = parameters
        self.transformer: Optional[Transformer] = None


def finish_sync(run: Any, hosts: Dict[int, Host], file: HostFile, call: str) -> None:
    """Run the host plugins a native run pauses at until it finishes; one returning an
    awaitable fails with `async-in-sync`, as an asynchronous native stage does."""
    sent = len(file.messages)
    while run.hosts():
        tree = json.loads(run.tree())
        for id in run.hosts():
            host = hosts.get(id)
            if host is None or host.transformer is None:
                continue
            result = host.transformer(tree, file)
            if inspect.isawaitable(result):
                close = getattr(result, "close", None)
                if close is not None:
                    close()
                raise _async_error(call)
            tree = _read_tree(result, tree)
        run.resume(json.dumps(tree), json.dumps(file.messages[sent:]))
        sent = len(file.messages)


async def finish(run: Any, hosts: Dict[int, Host], file: HostFile) -> None:
    """Run the host plugins a native run pauses at until it finishes, awaiting each."""
    sent = len(file.messages)
    while run.hosts():
        tree = json.loads(run.tree())
        for id in run.hosts():
            host = hosts.get(id)
            if host is None or host.transformer is None:
                continue
            result = host.transformer(tree, file)
            if inspect.isawaitable(result):
                result = await result
            tree = _read_tree(result, tree)
        run.resume(json.dumps(tree), json.dumps(file.messages[sent:]))
        sent = len(file.messages)


def merge_options(target: Any, source: Any) -> Any:
    """Merge `source` into a copy of `target`, as unified merges a plugin's options: dicts
    key by key and lists index by index, anything else replaced."""
    if isinstance(source, dict):
        merged = dict(target) if isinstance(target, dict) else {}
        for key, value in source.items():
            merged[key] = merge_options(merged.get(key), value)
        return merged
    if isinstance(source, list):
        merged_list = list(target) if isinstance(target, list) else []
        for index, value in enumerate(source):
            if index < len(merged_list):
                merged_list[index] = merge_options(merged_list[index], value)
            else:
                merged_list.append(value)
        return merged_list
    return source


# A transformer that changes the tree in place returns nothing: the tree it got goes on
def _read_tree(result: Any, tree: Dict[str, Any]) -> Dict[str, Any]:
    if isinstance(result, dict) and isinstance(result.get("type"), str):
        return result
    return tree


def _read_place(place: Any) -> Optional[Place]:
    if not isinstance(place, dict):
        return None
    if "position" in place:
        return _read_place(place["position"])
    if "start" in place and "end" in place:
        start, end = _read_point(place["start"]), _read_point(place["end"])
        return {"start": start, "end": end} if start and end else None
    point = _read_point(place)
    return {"start": point, "end": point} if point else None


def _read_point(value: Any) -> Optional[Point]:
    if not isinstance(value, dict) or not isinstance(value.get("line"), int):
        return None
    if not isinstance(value.get("column"), int):
        return None
    offset = value.get("offset")
    offset = offset if isinstance(offset, int) else None
    return {"line": value["line"], "column": value["column"], "offset": offset}


def _async_error(call: str) -> Exception:
    error = _native.TextlaneError(f"`{call}Sync` finished async. Use `{call}` instead")
    error.code = "async-in-sync"
    return error
