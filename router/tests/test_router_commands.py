import sys
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from router.commands import DEFAULT_ENTITY_ID, text_to_patches


def assert_patch_list(patches):
    assert isinstance(patches, list)
    for patch in patches:
        assert isinstance(patch, dict)
        assert all(key in patch for key in ("id", "type", "data"))


@pytest.mark.parametrize(
    "command,axis,expected",
    [
        ("move cube up", "dy", 1.0),
        ("move cube down", "dy", -1.0),
        ("move cube left", "dx", -1.0),
        ("move cube right", "dx", 1.0),
        ("move cube forward", "dz", 1.0),
        ("move cube back", "dz", -1.0),
    ],
)
def test_move_commands(command, axis, expected):
    patches = text_to_patches(command)
    assert_patch_list(patches)
    assert len(patches) == 1
    patch = patches[0]
    assert patch["id"] == DEFAULT_ENTITY_ID
    assert patch["type"] == "move_entity"
    delta = patch["data"]
    assert delta[axis] == pytest.approx(expected)


@pytest.mark.parametrize(
    "command,expected_color",
    [
        ("make cube red", [1.0, 0.0, 0.0]),
        ("make cube blue", [0.0, 0.0, 1.0]),
        ("make cube green", [0.0, 1.0, 0.0]),
    ],
)
def test_color_commands(command, expected_color):
    patches = text_to_patches(command)
    assert_patch_list(patches)
    assert len(patches) == 1
    patch = patches[0]
    assert patch["id"] == DEFAULT_ENTITY_ID
    assert patch["type"] == "set_color"
    assert patch["data"]["color"] == expected_color


def test_spawn_cube_command():
    patches = text_to_patches("spawn cube")
    assert_patch_list(patches)
    assert len(patches) >= 1
    assert any(p["type"] == "spawn_entity" for p in patches)


def test_delete_cube_command():
    patches = text_to_patches("delete cube")
    assert_patch_list(patches)
    assert len(patches) >= 1
    assert patches[0]["id"] == DEFAULT_ENTITY_ID
    assert any(p["type"] == "delete_entity" for p in patches)


def test_unknown_command_returns_empty_list():
    patches = text_to_patches("dance cube")
    assert patches == []
