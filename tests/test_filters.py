from pathlib import Path

import pytest

from notifykit import CommonFilter, ModifyDataEvent, DataType, RenameEvent


@pytest.mark.parametrize("path,filtered", [
    (Path("/home/myusr/proj/__pycache__/tmpfie"), True),
    (Path("/home/myusr/proj/.git/HEAD"), True),
    (Path("/home/myusr/proj/.venv/lib/python3/notifykit/testing.py"), True),
    (Path("/home/myusr/proj/main.py"), False),
    (Path("/home/myusr/proj/logs.txt~"), True),
])
def test__event_filter__ignore_paths(path: Path, filtered: bool) -> None:
    filter = CommonFilter()

    assert filter(ModifyDataEvent(path=path, data_type=DataType.CONTENT)) == filtered

@pytest.mark.parametrize("old_path,new_path,filtered", [
    (Path("/home/myusr/proj/__pycache__/tmpfie"), Path("/home/myusr/proj/tmpfile"), False),
    (Path("/home/myusr/proj/logs.txt"), Path("/home/myusr/proj/.git/logs.txt"), False),
    (Path("/home/myusr/proj/__pycache__/abcdg"), Path("/home/myusr/proj/.venv/abcdg"), True),
])
def test__event_filter__ignore_renames(old_path: Path, new_path: Path, filtered: bool) -> None:
    filter = CommonFilter()

    assert filter(RenameEvent(old_path=old_path, new_path=new_path)) == filtered
