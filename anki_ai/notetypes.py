from anki.collection import Collection


def list_notetypes(col: Collection) -> list[dict]:
    return [
        {"id": nt.id, "name": nt.name, "use_count": nt.use_count}
        for nt in col.models.all_use_counts()
    ]


def get_notetype_fields(col: Collection, name: str) -> list[str]:
    nt = col.models.by_name(name)
    if nt is None:
        raise ValueError(f"Note type '{name}' not found.")
    return col.models.field_names(nt)
