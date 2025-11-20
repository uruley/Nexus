"""Runtime entry point for applying world patches.

This module loads a world description, applies patch operations, writes the
updated world back to disk, and prints timing information for the overall
frame as well as each patch.
"""
from __future__ import annotations

import argparse
import json
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Sequence, Optional

DEFAULT_WORLD = {
    "entities": [],
    "camera": {
        "position": [0.0, 1.0, 5.0],
        "target": [0.0, 0.0, 0.0],
    },
    "lighting": {},
}


@dataclass
class Patch:
    """Structured representation of an incoming world patch."""

    id: str
    type: str
    data: Dict[str, Any]

    @classmethod
    def from_raw(cls, raw: Dict[str, Any]) -> "Patch":
        if not isinstance(raw, dict):
            raise TypeError(f"Patch must be an object, received: {type(raw)!r}")
        try:
            patch_id = raw["id"]
            patch_type = raw["type"]
            data = raw["data"]
        except KeyError as exc:  # pragma: no cover - defensive programming
            missing = exc.args[0]
            raise KeyError(f"Patch is missing required field: {missing}") from exc

        if not isinstance(patch_id, str):
            raise TypeError("Patch 'id' must be a string")
        if not isinstance(patch_type, str):
            raise TypeError("Patch 'type' must be a string")
        if not isinstance(data, dict):
            raise TypeError("Patch 'data' must be an object")
        return cls(id=patch_id, type=patch_type, data=data)


def ensure_world_exists(world_path: Path) -> Dict[str, Any]:
    """Load the world file, creating a default one if it is missing."""
    if not world_path.exists():
        world_path.parent.mkdir(parents=True, exist_ok=True)
        with world_path.open("w", encoding="utf-8") as handle:
            json.dump(DEFAULT_WORLD, handle, indent=2)
            handle.write("\n")
        return json.loads(json.dumps(DEFAULT_WORLD))

    with world_path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_world(world_path: Path, world: Dict[str, Any]) -> None:
    """Persist the updated world back to the filesystem."""
    world_path.parent.mkdir(parents=True, exist_ok=True)
    with world_path.open("w", encoding="utf-8") as handle:
        json.dump(world, handle, indent=2)
        handle.write("\n")


def load_patches(paths: Iterable[Path]) -> List[Patch]:
    """Load patches from a collection of file paths."""
    patches: List[Patch] = []
    for path in paths:
        with path.open("r", encoding="utf-8") as handle:
            payload = json.load(handle)
        if isinstance(payload, list):
            for item in payload:
                patches.append(Patch.from_raw(item))
        else:
            patches.append(Patch.from_raw(payload))
    return patches


def read_perception_state() -> Optional[Dict[str, Any]]:
    """Attempt to read perception/state.json, returning its contents or None.

    Never raises; logs a short warning instead.
    """
    perception_state_path = Path(__file__).parents[1] / "perception" / "state.json"
    if not perception_state_path.exists():
        print("Perception: no state.json available")
        return None

    try:
        with perception_state_path.open("r", encoding="utf-8") as handle:
            return json.load(handle)
    except Exception as exc:  # pragma: no cover - defensive I/O
        print(f"Perception: failed to read state.json: {exc}")
        return None


def apply_patch(world: Dict[str, Any], patch: Patch) -> None:
    """Mutate the world in-place based on a single patch."""
    if patch.type == "spawn_entity":
        entity = patch.data.copy()
        entity.setdefault("id", patch.id)
        world.setdefault("entities", [])
        world["entities"].append(entity)
    elif patch.type == "move_camera":
        world.setdefault("camera", {})
        world["camera"].update(patch.data)
    elif patch.type == "set_light":
        light_name = patch.data.get("name", patch.id)
        light_state = patch.data.copy()
        light_state.pop("name", None)
        lighting = world.setdefault("lighting", {})
        existing = lighting.get(light_name, {})
        merged = {**existing, **light_state}
        lighting[light_name] = merged
    else:
        raise ValueError(f"Unsupported patch type: {patch.type}")


def discover_patch_files(patch_arguments: Sequence[str], patch_dir: Path | None) -> List[Path]:
    """Resolve patch file inputs either from CLI arguments or a directory scan."""
    if patch_arguments:
        return [Path(arg) for arg in patch_arguments]

    if patch_dir and patch_dir.exists():
        return sorted(patch_dir.glob("*.json"))

    return []


def main(argv: Sequence[str] | None = None) -> Dict[str, Any]:
    parser = argparse.ArgumentParser(description="Apply world patches to a runtime world file.")
    parser.add_argument("--world", type=Path, default=Path(__file__).with_name("world.json"), help="Path to the world JSON file.")
    parser.add_argument(
        "--patch",
        dest="patches",
        action="append",
        default=[],
        help="Path to a patch JSON file. Repeat for multiple patches.",
    )
    parser.add_argument(
        "--patch-dir",
        dest="patch_dir",
        type=Path,
        default=Path(__file__).parents[1] / "router" / "examples",
        help="Directory containing patch JSON files to apply if --patch is not specified.",
    )
    args = parser.parse_args(argv)

    world_path: Path = args.world
    patch_paths = discover_patch_files(args.patches, args.patch_dir)

    world = ensure_world_exists(world_path)
    patches = load_patches(patch_paths)

    frame_start = time.perf_counter()
    for patch in patches:
        patch_start = time.perf_counter()
        apply_patch(world, patch)
        patch_elapsed = (time.perf_counter() - patch_start) * 1000
        print(f"Applied {patch.id} ({patch.type}) in {patch_elapsed:.3f} ms")

    write_world(world_path, world)

    # After world update, read latest perception state (if available)
    perception_state = read_perception_state()
    if perception_state is not None:
        persons = perception_state.get("persons") or []
        count = len(persons)
        if count == 0:
            print("Perception: 0 persons")
        else:
            first = persons[0]
            keypoints = first.get("keypoints") or []
            if keypoints:
                avg_score = sum(k.get("c", 0.0) for k in keypoints) / len(keypoints)
                print(f"Perception: {count} persons, avg_score={avg_score:.2f}")
            else:
                print(f"Perception: {count} persons, avg_score=N/A")

    frame_elapsed = (time.perf_counter() - frame_start) * 1000
    print(f"Frame processed in {frame_elapsed:.3f} ms for {len(patches)} patch(es)")
    return world


if __name__ == "__main__":  # pragma: no cover - manual execution entry point
    main()
