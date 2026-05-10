import os
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path

from anki import backend_pb2
from anki._backend import RustBackend, backend_exception_to_pylib
from anki.collection import Collection


def _silence_backend_noise() -> None:
    # anki's _run_command prints "blocked main thread for Xms" whenever a
    # synchronous call takes >200ms — a GUI-dev diagnostic that is pure noise
    # in a CLI. Replace the method with an identical copy that omits the print.
    def _patched(self: RustBackend, service: int, method: int, input: bytes) -> bytes:
        try:
            return self._backend.command(service, method, input)
        except Exception as error:
            error_bytes = bytes(error.args[0])
        err = backend_pb2.BackendError()
        err.ParseFromString(error_bytes)
        raise backend_exception_to_pylib(err)

    setattr(RustBackend, "_run_command", _patched)


_silence_backend_noise()


def get_collection_path() -> str:
    env_path = os.environ.get("ANKI_COLLECTION_PATH")
    if env_path:
        return str(Path(env_path).expanduser())

    anki_base = Path("~/.local/share/Anki2").expanduser()
    if anki_base.exists():
        for entry in sorted(anki_base.iterdir()):
            if entry.is_dir() and not entry.name.startswith("."):
                candidate = entry / "collection.anki2"
                if candidate.exists():
                    return str(candidate)

    return str(anki_base / "User 1" / "collection.anki2")


@contextmanager
def open_collection(path: str | None = None) -> Generator[Collection, None, None]:
    col_path = path or get_collection_path()
    col = Collection(col_path)
    try:
        yield col
    finally:
        col.close()
