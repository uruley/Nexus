from __future__ import annotations

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from runtime.main import Patch, apply_patch, discover_patch_files, load_patches, main as runtime_main


def test_apply_patch_spawn_and_update(tmp_path: Path) -> None:
    world = {"entities": [], "camera": {}, "lighting": {}}
    spawn = Patch(id="spawn-1", type="spawn_entity", data={"mesh": "cube", "position": [0, 0, 0]})
    apply_patch(world, spawn)
    assert world["entities"][0]["mesh"] == "cube"

    move_cam = Patch(id="cam-1", type="move_camera", data={"position": [1, 2, 3]})
    apply_patch(world, move_cam)
    assert world["camera"]["position"] == [1, 2, 3]

    set_light = Patch(id="light:key", type="set_light", data={"name": "key", "intensity": 1.2})
    apply_patch(world, set_light)
    assert world["lighting"]["key"]["intensity"] == 1.2


def test_main_creates_world_and_applies_patch(tmp_path: Path) -> None:
    world_path = tmp_path / "world.json"
    patch_path = tmp_path / "patch.json"
    patch_path.write_text(
        json.dumps(
            {
                "id": "cam-2",
                "type": "move_camera",
                "data": {"position": [9, 9, 9]},
            }
        )
    )

    result = runtime_main(["--world", str(world_path), "--patch", str(patch_path)])
    assert result["camera"]["position"] == [9, 9, 9]

    persisted = json.loads(world_path.read_text())
    assert persisted["camera"]["position"] == [9, 9, 9]


def test_discover_patch_files_prefers_cli_arguments(tmp_path: Path) -> None:
    patch_a = tmp_path / "a.json"
    patch_b = tmp_path / "b.json"
    patch_a.write_text("{}"); patch_b.write_text("{}")

    discovered = discover_patch_files([str(patch_b), str(patch_a)], tmp_path)
    assert [Path(path) for path in [patch_b, patch_a]] == discovered


def test_load_patches_handles_list(tmp_path: Path) -> None:
    payload_path = tmp_path / "bundle.json"
    payload_path.write_text(
        json.dumps(
            [
                {"id": "1", "type": "spawn_entity", "data": {"mesh": "cube", "position": [0, 0, 0]}},
                {"id": "2", "type": "move_camera", "data": {"position": [0, 1, 2]}},
            ]
        )
    )

    patches = load_patches([payload_path])
    assert [patch.id for patch in patches] == ["1", "2"]
