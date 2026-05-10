import json
from pathlib import Path

import typer
from anki.collection import Collection
from anki.sync import SyncAuth
from anki.sync_pb2 import SyncCollectionResponse

AUTH_PATH = Path("~/.config/anki-ai/auth.json").expanduser()

_CR = SyncCollectionResponse.ChangesRequired


def load_hkey() -> str:
    if not AUTH_PATH.exists():
        typer.echo("Not authenticated. Run `anki-ai auth login` first.", err=True)
        raise typer.Exit(1)
    data = json.loads(AUTH_PATH.read_text())
    hkey = data.get("hkey")
    if not hkey:
        typer.echo("auth.json is missing the hkey field.", err=True)
        raise typer.Exit(1)
    return hkey


def save_hkey(hkey: str) -> None:
    AUTH_PATH.parent.mkdir(parents=True, exist_ok=True)
    AUTH_PATH.write_text(json.dumps({"hkey": hkey}))
    AUTH_PATH.chmod(0o600)


def run_sync(
    col: Collection, sync_media: bool = False, upload: bool = False
) -> SyncCollectionResponse:
    hkey = load_hkey()
    auth = SyncAuth(hkey=hkey, endpoint=None)
    output = col.sync_collection(auth, sync_media=sync_media)

    required = output.required

    if required == _CR.NO_CHANGES:
        typer.echo("Already up to date.")
    elif required == _CR.NORMAL_SYNC:
        typer.echo("Sync complete.")
    elif required in (_CR.FULL_SYNC, _CR.FULL_DOWNLOAD, _CR.FULL_UPLOAD):
        direction = "upload" if (upload or required == _CR.FULL_UPLOAD) else "download"
        typer.echo(f"Full sync required — performing full {direction}.")
        col.full_upload_or_download(
            auth=auth,
            server_usn=output.server_media_usn,
            upload=(direction == "upload"),
        )
        typer.echo(f"Full {direction} complete.")
    else:
        typer.echo(f"Sync returned unexpected status: {required}", err=True)

    return output
