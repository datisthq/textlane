"""Textlane: a Rust-native text processing pipeline, a unified replacement."""

from . import html, md, mdx, nl
from ._models import Message, Place, Plugin, Point, VFile
from ._native import (
    TextlaneError,
    plugin_names,
    retained_memory,
    set_retained_memory,
    version,
)
from ._processor import Processor, textlane

__all__ = [
    "Message",
    "Place",
    "Plugin",
    "Point",
    "Processor",
    "TextlaneError",
    "VFile",
    "html",
    "md",
    "mdx",
    "plugin_names",
    "nl",
    "retained_memory",
    "set_retained_memory",
    "textlane",
    "version",
]

# Only wheels built with the crate's `testing` feature carry it, for the conformance suite
try:
    from ._native import run_case  # noqa: F401
except ImportError:
    pass
