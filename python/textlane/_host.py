"""Python plugins: what they get as the file, and the loop that runs them when a native run
pauses before them."""

import inspect
import json
import os
import posixpath
from typing import Any, Callable, Dict, List, NoReturn, Optional, Union

from . import _native
from ._models import Message, Place, Point

Transformer = Callable[[Dict[str, Any], "HostFile"], Any]
"""What an attacher returns: it changes the tree in place or returns a new one, directly or
as an awaitable."""


class HostFile:
    """The file a Python plugin gets, as a vfile: the document, its path, data and messages.

    Only what plugins report here reaches the result; native stages report there directly.
    """

    def __init__(self, value: Optional[str], path: Optional[str] = None) -> None:
        self.value = value
        self.data: Dict[str, Any] = {}
        self.messages: List["HostMessage"] = []
        self.history: List[str] = [] if path is None else [path]
        self.cwd = os.getcwd()

    @property
    def path(self) -> Optional[str]:
        """The current path; setting it adds to `history`."""
        return self.history[-1] if self.history else None

    @path.setter
    def path(self, path: Optional[str]) -> None:
        if path is not None and path != self.path:
            self.history.append(path)

    @property
    def dirname(self) -> Optional[str]:
        return None if self.path is None else posixpath.dirname(self.path)

    @property
    def basename(self) -> Optional[str]:
        return None if self.path is None else posixpath.basename(self.path)

    @property
    def extname(self) -> Optional[str]:
        return None if self.path is None else posixpath.splitext(self.path)[1]

    @property
    def stem(self) -> Optional[str]:
        return None if self.path is None else posixpath.splitext(self.basename or "")[0]

    def message(
        self,
        reason: Union[str, BaseException],
        place: Any = None,
        rule_id: Optional[str] = None,
        source: Optional[str] = None,
        ancestors: Optional[List[Any]] = None,
        cause: Optional[BaseException] = None,
    ) -> "HostMessage":
        """Report a warning against the file; `place` is a point, a position or a node."""
        message = HostMessage(reason, place, rule_id, source, ancestors, cause)
        message.file = self.path or ""
        message.fatal = False
        self.messages.append(message)
        return message

    def info(
        self, reason: Union[str, BaseException], place: Any = None, **options: Any
    ) -> "HostMessage":
        """Report a note against the file, neither a warning nor an error."""
        message = self.message(reason, place, **options)
        message.fatal = None
        return message

    def fail(
        self, reason: Union[str, BaseException], place: Any = None, **options: Any
    ) -> NoReturn:
        """Report an error against the file and raise it."""
        message = self.message(reason, place, **options)
        message.fatal = True
        raise message

    def __str__(self) -> str:
        return self.value or ""


class HostMessage(Exception):
    """A diagnostic a Python plugin reported, as vfile-message has it."""

    def __init__(
        self,
        reason: Union[str, BaseException],
        place: Any = None,
        rule_id: Optional[str] = None,
        source: Optional[str] = None,
        ancestors: Optional[List[Any]] = None,
        cause: Optional[BaseException] = None,
    ) -> None:
        text = reason if isinstance(reason, str) else str(reason)
        super().__init__(text)
        self.reason = text
        self.rule_id = rule_id
        self.source = source
        self.ancestors = ancestors
        self.__cause__ = cause if cause is not None or isinstance(reason, str) else reason
        self.place = _read_place(place if place is not None else (ancestors or [None])[-1])
        self.line = self.place["start"]["line"] if self.place else None
        self.column = self.place["start"]["column"] if self.place else None
        self.name = _write_place(self.place)
        self.fatal: Optional[bool] = None
        self.file = ""

    def to_json(self) -> Message:
        """The message as it crosses into the crate."""
        return {
            "reason": self.reason,
            "ruleId": self.rule_id,
            "source": self.source,
            "place": self.place,
        }


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
        messages = [message.to_json() for message in file.messages[sent:]]
        run.resume(json.dumps(tree), json.dumps(messages))
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
        messages = [message.to_json() for message in file.messages[sent:]]
        run.resume(json.dumps(tree), json.dumps(messages))
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


# A transformer that changes the tree in place returns nothing: the tree it got goes on;
# one returning an exception fails the run, as unified's returning an `Error` does
def _read_tree(result: Any, tree: Dict[str, Any]) -> Dict[str, Any]:
    if isinstance(result, BaseException):
        raise result
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


# unist-util-stringify-position: `1:1-1:2`, or `1:1` for an unknown place
def _write_place(place: Optional[Place]) -> str:
    if not place:
        return "1:1"
    start = f"{place['start']['line']}:{place['start']['column']}"
    end = f"{place['end']['line']}:{place['end']['column']}"
    return start if start == end else f"{start}-{end}"


def _async_error(call: str) -> Exception:
    error = _native.TextlaneError(f"`{call}Sync` finished async. Use `{call}` instead")
    error.code = "async-in-sync"
    return error
