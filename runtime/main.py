from __future__ import annotations

import argparse
import json
import time
from pathlib import Path
from typing import Iterable, List

WORLD_PATH = Path("apps/nexus_desktop/assets/world.json")
POLL_INTERVAL_SECONDS = 0.2


def log(message: str) -> None:
    print(f"[runtime] {message}")


def load_world(path: Path = WORLD_PATH) -> dict:
    """Load the on-disk world file, creating a default one if necessary."""

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
    """Persist the world to disk."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(world, indent=2))


def load_patches(patch_path: Path) -> List[dict]:
    """Load patches from disk, coercing them into a list."""

    try:
        content = json.loads(patch_path.read_text())
    except FileNotFoundError:
        log(f"patch file not found: {patch_path}")
        return []
    except json.JSONDecodeError as exc:
        log(f"invalid patch JSON: {exc}")
        return []

    if isinstance(content, list):
        return [p for p in content if isinstance(p, dict)]
    if isinstance(content, dict):
        return [content]

    log("patch payload must be an object or list")
    return []


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


def apply_patch(world: dict, patch: dict) -> bool:
    """Apply a single patch to the in-memory world.

    Returns True if the patch mutated the world, False otherwise.
    """

    patch_type = patch.get("type")
    data = patch.get("data", {})
    entity_id = patch.get("id")

    start = time.perf_counter()

    if not patch_type:
        log("skipping patch with no type field")
        return False

    if patch_type == "spawn_entity":
        new_entity = {
            "id": entity_id or f"entity:{len(world.get('entities', [])) + 1:03d}",
            "kind": data.get("kind", "cube"),
            "transform": {"translation": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0]},
            "material": {"color": [1.0, 1.0, 1.0]},
        }
        world.setdefault("entities", []).append(new_entity)
        duration_ms = (time.perf_counter() - start) * 1000
        log(f"applied {patch_type} in {duration_ms:.2f}ms")
        return True

    if patch_type == "move_entity" and entity_id:
        entity = find_entity(world, entity_id)
        if entity:
            ensure_entity_defaults(entity)
            translation = entity["transform"]["translation"]
            translation[0] += float(data.get("dx", 0.0))
            translation[1] += float(data.get("dy", 0.0))
            translation[2] += float(data.get("dz", 0.0))
            duration_ms = (time.perf_counter() - start) * 1000
            log(f"applied {patch_type} in {duration_ms:.2f}ms")
            return True
        log(f"move_entity target missing: {entity_id}")
        return False

    if patch_type == "set_color" and entity_id:
        entity = find_entity(world, entity_id)
        if entity:
            ensure_entity_defaults(entity)
            color = data.get("color")
            if isinstance(color, Iterable):
                entity["material"]["color"] = [float(c) for c in color][:3]
                duration_ms = (time.perf_counter() - start) * 1000
                log(f"applied {patch_type} in {duration_ms:.2f}ms")
                return True
        log(f"set_color target missing or invalid payload for {entity_id}")
        return False

    if patch_type == "delete_entity" and entity_id:
        entities = world.get("entities", [])
        before = len(entities)
        world["entities"] = [e for e in entities if e.get("id") != entity_id]
        duration_ms = (time.perf_counter() - start) * 1000
        log(f"applied {patch_type} in {duration_ms:.2f}ms")
        return before != len(world["entities"])

    if patch_type == "move_camera":
        camera = world.setdefault("camera", {"translation": [0.0, 0.0, 0.0]})
        translation = camera.setdefault("translation", [0.0, 0.0, 0.0])
        translation[0] += float(data.get("dx", 0.0))
        translation[1] += float(data.get("dy", 0.0))
        translation[2] += float(data.get("dz", 0.0))
        duration_ms = (time.perf_counter() - start) * 1000
        log(f"applied {patch_type} in {duration_ms:.2f}ms")
        return True

    if patch_type == "set_light":
        light = world.setdefault("light", {"color": [1.0, 1.0, 1.0], "intensity": 1.0})
        if "intensity" in data:
            light["intensity"] = float(data["intensity"])
        if "color" in data and isinstance(data["color"], Iterable):
            light["color"] = [float(c) for c in data["color"]][:3]
        duration_ms = (time.perf_counter() - start) * 1000
        log(f"applied {patch_type} in {duration_ms:.2f}ms")
        return True

    log(f"unhandled patch type: {patch_type}")
    return False


def apply_patches(world: dict, patches: List[dict], world_path: Path = WORLD_PATH) -> dict:
    """Apply patches sequentially, persisting after each patch."""

    for patch in patches:
        apply_patch(world, patch)
        save_world(world, world_path)
    return world


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Apply router patches to world.json")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--patch",
        type=Path,
        help="Path to a JSON patch file (single patch or list).",
    )
    group.add_argument(
        "--watch",
        type=Path,
        help="Watch a patch file for changes and apply automatically.",
    )
    group.add_argument(
        "--simulate",
        type=Path,
        help="Apply patches once and print a concise summary (no watch loop).",
    )
    parser.add_argument(
        "--interval",
        type=float,
        default=POLL_INTERVAL_SECONDS,
        help="Polling interval in seconds when using --watch.",
    )
    parser.add_argument(
        "--world",
        type=Path,
        default=WORLD_PATH,
        help="Override world.json output path.",
    )
    return parser.parse_args()


def watch_patch_file(patch_path: Path, world_path: Path, interval: float) -> None:
    """Continuously poll for changes to patch_path and apply them live."""

    world = load_world(world_path)
    last_modified: float | None = None
    log(f"watching {patch_path} for updates")

    while True:
        try:
            modified = patch_path.stat().st_mtime
        except FileNotFoundError:
            time.sleep(interval)
            continue

        if last_modified is None or modified > last_modified:
            patches = load_patches(patch_path)
            if patches:
                apply_patches(world, patches, world_path)
                log(f"applied {len(patches)} patches from watch loop")
            else:
                log("no valid patches found; clearing command file")

            patch_path.write_text(json.dumps([], indent=2))
            last_modified = patch_path.stat().st_mtime

        time.sleep(interval)


def simulate_patches(patch_path: Path, world_path: Path) -> None:
    """Apply patches once and print a concise summary for debugging."""

    patches = load_patches(patch_path)
    world = load_world(world_path)

    print(f"[simulate] loaded {len(patches)} patches from {patch_path}")

    for patch in patches:
        patch_type = patch.get("type", "<unknown>")
        target_id = patch.get("id", "<no-id>")
        applied = apply_patch(world, patch)
        status = "applied" if applied else "skipped"
        print(f"[simulate] {status} {patch_type} to {target_id}")

    save_world(world, world_path)
    print(f"[simulate] world saved to {world_path}")


def main() -> None:
    args = parse_args()

    if args.watch:
        watch_patch_file(args.watch, args.world, args.interval)
        return

    if args.simulate:
        simulate_patches(args.simulate, args.world)
        return

    patches = load_patches(args.patch)
    world = load_world(args.world)
    updated_world = apply_patches(world, patches, args.world)
    save_world(updated_world, args.world)


if __name__ == "__main__":
    main()
