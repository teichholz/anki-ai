from anki.collection import Collection


def list_tags(col: Collection) -> list[str]:
    return sorted(col.tags.all())


def bulk_add_tags(col: Collection, query: str, tags: list[str]) -> int:
    nids = col.find_notes(query)
    if not nids:
        return 0
    result = col.tags.bulk_add(list(nids), " ".join(tags))
    return result.count


def bulk_remove_tags(col: Collection, query: str, tags: list[str]) -> int:
    nids = col.find_notes(query)
    if not nids:
        return 0
    result = col.tags.bulk_remove(list(nids), " ".join(tags))
    return result.count


def rename_tag(col: Collection, old: str, new: str) -> int:
    result = col.tags.rename(old, new)
    return result.count
