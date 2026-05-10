from anki.collection import Collection


def _tree_to_list(node, parent_name: str = "") -> list[dict]:
    results = []
    full_name = f"{parent_name}::{node.name}" if parent_name else node.name
    if node.deck_id:
        results.append(
            {
                "id": node.deck_id,
                "name": full_name,
                "new": node.new_count,
                "learning": node.learn_count,
                "review": node.review_count,
            }
        )
        child_parent = full_name
    else:
        child_parent = ""
    for child in node.children:
        results.extend(_tree_to_list(child, child_parent))
    return results


def list_decks(col: Collection) -> list[dict]:
    tree = col.sched.deck_due_tree()
    if tree is None:
        return []
    return _tree_to_list(tree)


def create_deck(col: Collection, name: str) -> dict:
    result = col.decks.add_normal_deck_with_name(name)
    return {"id": result.id, "name": name}


def delete_deck(col: Collection, name: str) -> None:
    did = col.decks.id_for_name(name)
    if did is None:
        raise ValueError(f"Deck '{name}' not found.")
    col.decks.remove([did])
