"""The shapes that cross the native boundary: plugin descriptors, files and messages."""

from typing import Any, Dict, List, Optional, TypedDict, Union


class _PluginRequired(TypedDict):
    name: str


class Plugin(_PluginRequired, total=False):
    """A native plugin as a pipeline names it, with its options when given."""

    options: Any


class Point(TypedDict):
    """One place in a file: a 1-indexed line and column and a 0-indexed offset, when known."""

    line: int
    column: int
    offset: Optional[int]


class Place(TypedDict):
    """Where in the file a message is about."""

    start: Point
    end: Point


class Message(TypedDict):
    """A diagnostic a plugin reported, as vfile-message has it."""

    reason: str
    ruleId: Optional[str]
    source: Optional[str]
    place: Optional[Place]


class VFile(TypedDict):
    """What `process` gives back, as a vfile: the output, the messages and the file's data."""

    value: str
    messages: List[Message]
    data: Dict[str, Any]


class _FileValue(TypedDict):
    value: str


Compatible = Optional[Union[str, _FileValue]]
"""A file to run a tree against: its text, or a file holding it as `value`."""


def describe_plugin(name: str, options: Any = None) -> Plugin:
    """The descriptor of a native plugin, with its options only when given."""
    if options is None:
        return {"name": name}
    return {"name": name, "options": options}
