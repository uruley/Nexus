from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Iterable, List

WORLD_PATH = Path("apps/nexus_desktop/assets/world.json")


def load_world(path: Path = WORLD_PATH) -> dict:
    if path.exists():
        return json.loads(path.read_text())

    default_world = {
        "entities": [],
        "camera": {"translation": [0.0, 5.0, 10.0]},
        "light": {"color": [1.0, 1.0, 1.0], "intensity": 1.0},
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(default_world, indent=2))
    return default_world


def save_world(world: dict, path: Path = WORLD_PATH) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(world, indent=2))


def load_patches(patch_path: Path) -> List[dict]:
    content = json.loads(patch_path.read_text())
    if isinstance(content, list):
        return content
    return [content]


def ensure_entity_defaults(entity: dict) -> None:
    transform = entity.setdefault("transform", {})
    transform.setdefault("translation", [0.0, 0.0, 0.0])
    transform.setdefault("rotation", [0.0, 0.0, 0.0])
    transform.setdefault("scale", [1.0, 1.0, 1.0])

    material = entity.setdefault("material", {})
    material.setdefault("color", [1.0, 1.0, 1.0])


def find_entity(world: dict, entity_id: str) -> dict | None:
    for entity in world.get("entities", []):
        if entity.get("id") == entity_id:
            return entity
    return None


def apply_patch(world: dict, patch: dict) -> None:
    patch_type = patch.get("type")
    data = patch.get("data", {})
    entity_id = patch.get("id")

    if patch_type == "spawn_entity":
        new_entity = {
            "id": entity_id or f"entity:{len(world.get('entities', [])) + 1:03d}",
            "kind": data.get("kind", "cube"),
            "transform": {"translation": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0]},
            "material": {"color": [1.0, 1.0, 1.0]},
        }
        world.setdefault("entities", []).append(new_entity)
        return

    if patch_type == "move_entity" and entity_id:
        entity = find_entity(world, entity_id)
        if entity:
            ensure_entity_defaults(entity)
            translation = entity["transform"]["translation"]
            translation[0] += float(data.get("dx", 0.0))
            translation[1] += float(data.get("dy", 0.0))
            translation[2] += float(data.get("dz", 0.0))
        return

    if patch_type == "set_color" and entity_id:
        entity = find_entity(world, entity_id)
        if entity:
            ensure_entity_defaults(entity)
            color = data.get("color")
            if isinstance(color, Iterable):
                entity["material"]["color"] = [float(c) for c in color][:3]
        return

    if patch_type == "delete_entity" and entity_id:
        entities = world.get("entities", [])
        world["entities"] = [e for e in entities if e.get("id") != entity_id]
        return

    if patch_type == "move_camera":
        camera = world.setdefault("camera", {"translation": [0.0, 0.0, 0.0]})
        translation = camera.setdefault("translation", [0.0, 0.0, 0.0])
        translation[0] += float(data.get("dx", 0.0))
        translation[1] += float(data.get("dy", 0.0))
        translation[2] += float(data.get("dz", 0.0))
        return

    if patch_type == "set_light":
        light = world.setdefault("light", {"color": [1.0, 1.0, 1.0], "intensity": 1.0})
        if "intensity" in data:
            light["intensity"] = float(data["intensity"])
        if "color" in data and isinstance(data["color"], Iterable):
            light["color"] = [float(c) for c in data["color"]][:3]
        return


def apply_patches(world: dict, patches: List[dict]) -> dict:
    for patch in patches:
        apply_patch(world, patch)
    return world


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Apply router patches to world.json")
    parser.add_argument(
        "--patch",
        type=Path,
        required=True,
        help="Path to a JSON patch file (single patch or list).",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    patches = load_patches(args.patch)
    world = load_world()
    updated_world = apply_patches(world, patches)
    save_world(updated_world)


if __name__ == "__main__":
    main()
