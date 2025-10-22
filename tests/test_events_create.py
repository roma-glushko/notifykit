import asyncio
import tempfile
from pathlib import Path

from notifykit import Notifier, NotifierT
from tests.conftest import EventCollector


async def test__events__create_file() -> None:
    files_to_create = 3
    expected_events = files_to_create * 3 # each file triggers 3 events: Create, ModifyMetadata, ModifyData
    tmp_dir = Path(tempfile.mkdtemp())

    await asyncio.sleep(0.1)  # avoid catching directory creation event

    notifier: NotifierT = Notifier(debounce_ms=100, tick_ms=10, debug=False)
    await notifier.watch([tmp_dir])

    collector = EventCollector()

    async with collector.collect(notifier) as c:
        expected_paths = []

        for idx in range(files_to_create):
            file_path = tmp_dir / f'lorem_{idx}.txt'
            file_path.write_text("new lorem ipsum")

            expected_paths.append(str(file_path))

        await c.wait_for_events(expected_events, timeout=3)

    assert len(collector.events) == expected_events, collector.events
