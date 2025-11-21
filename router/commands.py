from __future__ import annotations

from typing import List

DEFAULT_ENTITY_ID = "entity:cube:001"


def text_to_patches(text: str) -> List[dict]:
    """Convert a natural language command into schema-compliant patches."""

    normalized = " ".join(text.lower().split())

    move_vectors = {
        "move cube up": (0.0, 1.0, 0.0),
        "move cube down": (0.0, -1.0, 0.0),
        "move cube left": (-1.0, 0.0, 0.0),
        "move cube right": (1.0, 0.0, 0.0),
        "move cube forward": (0.0, 0.0, 1.0),
        "move cube back": (0.0, 0.0, -1.0),
    }

    if normalized in move_vectors:
        dx, dy, dz = move_vectors[normalized]
        return [
            {
                "id": DEFAULT_ENTITY_ID,
                "type": "move_entity",
                "data": {"dx": dx, "dy": dy, "dz": dz},
            }
        ]

    if normalized == "spawn cube":
        entity_id = DEFAULT_ENTITY_ID
        return [
            {
                "id": entity_id,
                "type": "spawn_entity",
                "data": {"kind": "cube"},
            },
            {
                "id": entity_id,
                "type": "move_entity",
                "data": {"dx": 0.0, "dy": 1.0, "dz": 0.0},
            },
            {
                "id": entity_id,
                "type": "set_color",
                "data": {"color": [1.0, 1.0, 1.0]},
            },
        ]

    if normalized == "delete cube":
        return [
            {
                "id": DEFAULT_ENTITY_ID,
                "type": "delete_entity",
                "data": {},
            }
        ]

    color_map = {
        "make cube red": [1.0, 0.0, 0.0],
        "make cube blue": [0.0, 0.0, 1.0],
        "make cube green": [0.0, 1.0, 0.0],
    }

    if normalized in color_map:
        return [
            {
                "id": DEFAULT_ENTITY_ID,
                "type": "set_color",
                "data": {"color": color_map[normalized]},
            }
        ]

    return []
