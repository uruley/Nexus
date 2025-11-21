from copy import deepcopy
import sys
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from runtime.main import apply_patch


@pytest.fixture
def sample_world():
    return {
        "entities": [
            {
                "id": "entity:cube:001",
                "kind": "cube",
                "transform": {
                    "translation": [0.0, 0.0, 0.0],
                    "rotation": [0.0, 0.0, 0.0],
                    "scale": [1.0, 1.0, 1.0],
                },
                "material": {"color": [1.0, 1.0, 1.0]},
            }
        ],
        "camera": {"translation": [0.0, 5.0, 10.0]},
        "light": {"color": [1.0, 1.0, 1.0], "intensity": 1.0},
    }


def test_move_entity(sample_world):
    patch = {
        "id": "entity:cube:001",
        "type": "move_entity",
        "data": {"dx": 0.0, "dy": 1.0, "dz": 0.0},
    }
    world = deepcopy(sample_world)

    changed = apply_patch(world, patch)

    assert changed is True
    translation = world["entities"][0]["transform"]["translation"]
    assert translation == pytest.approx([0.0, 1.0, 0.0])


def test_set_color(sample_world):
    patch = {
        "id": "entity:cube:001",
        "type": "set_color",
        "data": {"color": [0.0, 1.0, 0.0]},
    }
    world = deepcopy(sample_world)

    changed = apply_patch(world, patch)

    assert changed is True
    color = world["entities"][0]["material"]["color"]
    assert color == pytest.approx([0.0, 1.0, 0.0])


def test_delete_entity(sample_world):
    patch = {
        "id": "entity:cube:001",
        "type": "delete_entity",
        "data": {},
    }
    world = deepcopy(sample_world)

    changed = apply_patch(world, patch)

    assert changed is True
    assert world["entities"] == []


def test_unknown_patch_type(sample_world):
    patch = {"id": "entity:cube:001", "type": "unknown", "data": {}}
    world = deepcopy(sample_world)

    changed = apply_patch(world, patch)

    assert changed is False
    assert world == sample_world
