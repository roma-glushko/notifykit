import tempfile
from pathlib import Path

from notifykit import Notifier


async def test__events__create_file() -> None:
    files_to_create = 3
    tmp_dir = Path(tempfile.mkdtemp())

    notifier = Notifier(debounce_ms=100, tick_ms=10, debug=False)
    await notifier.watch([tmp_dir])

    expected_paths = []

    for idx in range(files_to_create):
        file_path = tmp_dir / f'lorem_{idx}.txt'
        file_path.write_text("new lorem ipsum")

        expected_paths.append(str(file_path))