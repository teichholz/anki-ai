import json
from pathlib import Path
from typing import Annotated

import typer

from anki_ai.cards import find_cards, get_card_info, suspend_cards, unsuspend_cards
from anki_ai.collection import open_collection
from anki_ai.decks import create_deck, delete_deck, list_decks
from anki_ai.media import add_media_file
from anki_ai.notes import add_note, delete_note, get_note, search_notes, update_note
from anki_ai.notetypes import get_notetype_fields, list_notetypes
from anki_ai.sync import run_sync, save_hkey
from anki_ai.tags import bulk_add_tags, bulk_remove_tags, list_tags, rename_tag

cli = typer.Typer(no_args_is_help=True)
auth_app = typer.Typer(no_args_is_help=True)
decks_app = typer.Typer(no_args_is_help=True)
notes_app = typer.Typer(no_args_is_help=True)
cards_app = typer.Typer(no_args_is_help=True)
tags_app = typer.Typer(no_args_is_help=True)
notetypes_app = typer.Typer(no_args_is_help=True)
media_app = typer.Typer(no_args_is_help=True)

cli.add_typer(auth_app, name="auth", help="Manage AnkiWeb authentication.")
cli.add_typer(decks_app, name="decks", help="Deck operations.")
cli.add_typer(notes_app, name="notes", help="Note operations.")
cli.add_typer(cards_app, name="cards", help="Card operations.")
cli.add_typer(tags_app, name="tags", help="Tag operations.")
cli.add_typer(notetypes_app, name="notetypes", help="Note type introspection.")
cli.add_typer(media_app, name="media", help="Media file operations.")


def _parse_fields(field_args: list[str]) -> dict[str, str]:
    result: dict[str, str] = {}
    for arg in field_args:
        if "=" not in arg:
            raise typer.BadParameter(f"Field must be 'Name=Value', got: {arg!r}")
        name, _, value = arg.partition("=")
        result[name] = value
    return result


def _exit_on_error(exc: Exception, msg: str) -> None:
    typer.echo(f"{msg}: {exc}", err=True)
    raise typer.Exit(1)


# ---------------------------------------------------------------------------
# auth
# ---------------------------------------------------------------------------


@auth_app.command("login")
def auth_login() -> None:
    """Exchange AnkiWeb credentials for an auth token and store it."""
    email = typer.prompt("AnkiWeb email")
    password = typer.prompt("AnkiWeb password", hide_input=True)
    try:
        with open_collection() as col:
            auth = col.sync_login(email, password, None)
            save_hkey(auth.hkey)
        typer.echo("Logged in successfully.")
    except Exception as exc:
        _exit_on_error(exc, "Login failed")


# ---------------------------------------------------------------------------
# sync
# ---------------------------------------------------------------------------


@cli.command("skill")
def skill_cmd() -> None:
    """Print the path to the installed Claude skill file."""
    from importlib.resources import files

    print(files("anki_ai") / "skill" / "anki.md")


@cli.command("sync")
def sync_cmd(
    media: Annotated[bool, typer.Option("--media/--no-media")] = True,
    upload: Annotated[
        bool, typer.Option("--upload/--download", help="Direction for full sync.")
    ] = False,
) -> None:
    """Pull from (and push to) AnkiWeb."""
    try:
        with open_collection() as col:
            run_sync(col, sync_media=media, upload=upload)
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Sync failed")


# ---------------------------------------------------------------------------
# info
# ---------------------------------------------------------------------------


@cli.command("info")
def info_cmd() -> None:
    """Show collection statistics."""
    try:
        with open_collection() as col:
            data = {
                "path": col.path,
                "notes": col.note_count(),
                "cards": col.card_count(),
                "studied_today": col.studied_today(),
            }
        print(json.dumps(data, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to get info")


# ---------------------------------------------------------------------------
# decks
# ---------------------------------------------------------------------------


@decks_app.command("list")
def decks_list() -> None:
    """List decks with due card counts (new / learning / review)."""
    try:
        with open_collection() as col:
            decks = list_decks(col)
        print(json.dumps(decks, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to list decks")


@decks_app.command("create")
def decks_create(
    name: Annotated[str, typer.Argument(help="Deck name (use '::' for nested decks).")],
) -> None:
    """Create a deck. Returns the existing deck if the name is already taken."""
    try:
        with open_collection() as col:
            deck = create_deck(col, name)
        print(json.dumps(deck, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to create deck")


@decks_app.command("delete")
def decks_delete(
    name: Annotated[str, typer.Argument(help="Deck name to delete.")],
    yes: Annotated[
        bool, typer.Option("--yes", "-y", help="Skip confirmation prompt.")
    ] = False,
) -> None:
    """Delete a deck and all its cards."""
    if not yes:
        typer.confirm(f"Delete deck '{name}' and all its cards?", abort=True)
    try:
        with open_collection() as col:
            delete_deck(col, name)
        typer.echo(f"Deleted deck '{name}'.")
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to delete deck")


# ---------------------------------------------------------------------------
# notes
# ---------------------------------------------------------------------------


@notes_app.command("add")
def notes_add(
    deck: Annotated[str, typer.Option("--deck", help="Target deck name.")],
    note_type: Annotated[str, typer.Option("--type", help="Note type name.")] = "Basic",
    fields: Annotated[
        list[str],
        typer.Option("--field", help="Field as Name=Value. Repeat for each field."),
    ] = [],
) -> None:
    """Add a note to a deck."""
    if not fields:
        typer.echo("Provide at least one --field Name=Value.", err=True)
        raise typer.Exit(1)
    try:
        parsed = _parse_fields(fields)
        with open_collection() as col:
            note_id = add_note(col, deck, note_type, parsed)
        print(json.dumps({"id": note_id}, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to add note")


@notes_app.command("get")
def notes_get(
    note_id: Annotated[int, typer.Argument(help="Note ID.")],
) -> None:
    """Get a note by ID."""
    try:
        with open_collection() as col:
            note = get_note(col, note_id)
        print(json.dumps(note, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to get note")


@notes_app.command("delete")
def notes_delete(
    note_id: Annotated[int, typer.Argument(help="Note ID to delete.")],
    yes: Annotated[
        bool, typer.Option("--yes", "-y", help="Skip confirmation prompt.")
    ] = False,
) -> None:
    """Delete a note by ID."""
    if not yes:
        typer.confirm(f"Delete note {note_id}?", abort=True)
    try:
        with open_collection() as col:
            delete_note(col, note_id)
        typer.echo(f"Deleted note {note_id}.")
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to delete note")


@notes_app.command("update")
def notes_update(
    note_id: Annotated[int, typer.Argument(help="Note ID to update.")],
    fields: Annotated[
        list[str],
        typer.Option("--field", help="Field as Name=Value. Repeat for each field."),
    ] = [],
) -> None:
    """Update one or more fields of an existing note."""
    if not fields:
        typer.echo("Provide at least one --field Name=Value.", err=True)
        raise typer.Exit(1)
    try:
        parsed = _parse_fields(fields)
        with open_collection() as col:
            result = update_note(col, note_id, parsed)
        print(json.dumps(result, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to update note")


@notes_app.command("search")
def notes_search(
    query: Annotated[str, typer.Argument(help="Anki search query.")],
) -> None:
    """Search notes and return results as JSON."""
    try:
        with open_collection() as col:
            results = search_notes(col, query)
        print(json.dumps(results, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Search failed")


# ---------------------------------------------------------------------------
# cards
# ---------------------------------------------------------------------------


@cards_app.command("list")
def cards_list(
    query: Annotated[str, typer.Argument(help="Anki search query.")],
) -> None:
    """Find cards matching a search query."""
    try:
        with open_collection() as col:
            cards = find_cards(col, query)
        print(json.dumps(cards, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to list cards")


@cards_app.command("info")
def cards_info(
    card_id: Annotated[int, typer.Argument(help="Card ID.")],
) -> None:
    """Show scheduling info for a card."""
    try:
        with open_collection() as col:
            info = get_card_info(col, card_id)
        print(json.dumps(info, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to get card info")


@cards_app.command("suspend")
def cards_suspend(
    card_ids: Annotated[list[int], typer.Argument(help="Card IDs to suspend.")],
) -> None:
    """Suspend cards by ID."""
    try:
        with open_collection() as col:
            count = suspend_cards(col, card_ids)
        print(json.dumps({"suspended": count}, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to suspend cards")


@cards_app.command("unsuspend")
def cards_unsuspend(
    card_ids: Annotated[list[int], typer.Argument(help="Card IDs to unsuspend.")],
) -> None:
    """Unsuspend cards by ID."""
    try:
        with open_collection() as col:
            unsuspend_cards(col, card_ids)
        print(json.dumps({"unsuspended": len(card_ids)}, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to unsuspend cards")


# ---------------------------------------------------------------------------
# tags
# ---------------------------------------------------------------------------


@tags_app.command("list")
def tags_list() -> None:
    """List all tags in the collection."""
    try:
        with open_collection() as col:
            tags = list_tags(col)
        print(json.dumps(tags, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to list tags")


@tags_app.command("add")
def tags_add(
    tags: Annotated[list[str], typer.Argument(help="Tags to add.")],
    query: Annotated[
        str, typer.Option("--query", "-q", help="Anki search query to select notes.")
    ] = "deck:current",
) -> None:
    """Add tags to notes matching a search query."""
    try:
        with open_collection() as col:
            count = bulk_add_tags(col, query, tags)
        print(json.dumps({"updated": count}, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to add tags")


@tags_app.command("remove")
def tags_remove(
    tags: Annotated[list[str], typer.Argument(help="Tags to remove.")],
    query: Annotated[
        str, typer.Option("--query", "-q", help="Anki search query to select notes.")
    ],
) -> None:
    """Remove tags from notes matching a search query."""
    try:
        with open_collection() as col:
            count = bulk_remove_tags(col, query, tags)
        print(json.dumps({"updated": count}, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to remove tags")


@tags_app.command("rename")
def tags_rename(
    old: Annotated[str, typer.Argument(help="Existing tag name.")],
    new: Annotated[str, typer.Argument(help="New tag name.")],
) -> None:
    """Rename a tag across all notes."""
    try:
        with open_collection() as col:
            count = rename_tag(col, old, new)
        print(json.dumps({"updated": count}, ensure_ascii=False))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to rename tag")


# ---------------------------------------------------------------------------
# notetypes
# ---------------------------------------------------------------------------


@notetypes_app.command("list")
def notetypes_list() -> None:
    """List all note types with their note counts."""
    try:
        with open_collection() as col:
            nts = list_notetypes(col)
        print(json.dumps(nts, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to list note types")


@notetypes_app.command("fields")
def notetypes_fields(
    name: Annotated[str, typer.Argument(help="Note type name.")],
) -> None:
    """List the fields of a note type."""
    try:
        with open_collection() as col:
            field_names = get_notetype_fields(col, name)
        print(json.dumps(field_names, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to get fields")


# ---------------------------------------------------------------------------
# media
# ---------------------------------------------------------------------------


@media_app.command("upload")
def media_upload(
    paths: Annotated[list[Path], typer.Argument(help="File(s) to copy into the collection media folder.")],
) -> None:
    """Copy local files into the collection media folder.

    Returns the stored filename for each file. Use the filename to embed media
    in note fields: '<img src="file.jpg">' for images, '[sound:file.mp3]' for audio.
    """
    try:
        with open_collection() as col:
            results = [{"filename": add_media_file(col, p)} for p in paths]
        print(json.dumps(results, ensure_ascii=False, indent=2))
    except typer.Exit:
        raise
    except Exception as exc:
        _exit_on_error(exc, "Failed to upload media")
