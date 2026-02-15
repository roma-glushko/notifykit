"""Tests for structural pattern matching (match statement) support on event classes."""

from pathlib import Path

import pytest

from notifykit import (
    AccessEvent,
    AccessMode,
    AccessType,
    CreateEvent,
    DataType,
    DeleteEvent,
    MetadataType,
    ModifyDataEvent,
    ModifyMetadataEvent,
    ModifyOtherEvent,
    ModifyUnknownEvent,
    ObjectType,
    RenameEvent,
)


def test__match__create_event() -> None:
    event = CreateEvent(path="/tmp/app.py", file_type=ObjectType.FILE)

    match event:
        case CreateEvent(path, file_type):
            assert Path(path) == Path("/tmp/app.py")
            assert file_type == ObjectType.FILE
        case _:
            pytest.fail("CreateEvent did not match")


def test__match__delete_event() -> None:
    event = DeleteEvent(path="/tmp/old.py", file_type=ObjectType.DIR)

    match event:
        case DeleteEvent(path, file_type):
            assert Path(path) == Path("/tmp/old.py")
            assert file_type == ObjectType.DIR
        case _:
            pytest.fail("DeleteEvent did not match")


def test__match__rename_event() -> None:
    event = RenameEvent(old_path="/tmp/a.py", new_path="/tmp/b.py")

    match event:
        case RenameEvent(old_path, new_path):
            assert Path(old_path) == Path("/tmp/a.py")
            assert Path(new_path) == Path("/tmp/b.py")
        case _:
            pytest.fail("RenameEvent did not match")


def test__match__modify_data_event() -> None:
    event = ModifyDataEvent(path="/tmp/data.py", data_type=DataType.CONTENT)

    match event:
        case ModifyDataEvent(path, data_type):
            assert Path(path) == Path("/tmp/data.py")
            assert data_type == DataType.CONTENT
        case _:
            pytest.fail("ModifyDataEvent did not match")


def test__match__modify_metadata_event() -> None:
    event = ModifyMetadataEvent(path="/tmp/meta.py", metadata_type=MetadataType.PERMISSIONS)

    match event:
        case ModifyMetadataEvent(path, metadata_type):
            assert Path(path) == Path("/tmp/meta.py")
            assert metadata_type == MetadataType.PERMISSIONS
        case _:
            pytest.fail("ModifyMetadataEvent did not match")


def test__match__modify_other_event() -> None:
    event = ModifyOtherEvent(path="/tmp/other.py")

    match event:
        case ModifyOtherEvent(path):
            assert Path(path) == Path("/tmp/other.py")
        case _:
            pytest.fail("ModifyOtherEvent did not match")


def test__match__modify_unknown_event() -> None:
    event = ModifyUnknownEvent(path="/tmp/unknown.py")

    match event:
        case ModifyUnknownEvent(path):
            assert Path(path) == Path("/tmp/unknown.py")
        case _:
            pytest.fail("ModifyUnknownEvent did not match")


def test__match__access_event() -> None:
    event = AccessEvent(path="/tmp/access.py", access_type=AccessType.READ, access_mode=AccessMode.READ)

    match event:
        case AccessEvent(path, access_type, access_mode):
            assert Path(path) == Path("/tmp/access.py")
            assert access_type == AccessType.READ
            assert access_mode == AccessMode.READ
        case _:
            pytest.fail("AccessEvent did not match")


def test__match__type_dispatch() -> None:
    """Match statement can dispatch on event type."""
    events = [
        CreateEvent(path="/tmp/new.py", file_type=ObjectType.FILE),
        DeleteEvent(path="/tmp/gone.py", file_type=ObjectType.FILE),
        RenameEvent(old_path="/tmp/a.py", new_path="/tmp/b.py"),
        ModifyDataEvent(path="/tmp/changed.py", data_type=DataType.CONTENT),
    ]

    matched = []
    for event in events:
        match event:
            case CreateEvent():
                matched.append("create")
            case DeleteEvent():
                matched.append("delete")
            case RenameEvent():
                matched.append("rename")
            case ModifyDataEvent():
                matched.append("modify_data")

    assert matched == ["create", "delete", "rename", "modify_data"]
