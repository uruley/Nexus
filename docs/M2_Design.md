# M2 — Entity System Upgrade: Design Specification

**Status:** Design Phase (Cursor)  
**Target:** Implementation Phase (Codex)  
**Date:** 2025-11-20

---

## 1. Current Entity & Collider Situation (Recon)

### 1.1 Existing Components

**Location:** `crates/anchor/src/lib.rs`

- **`Velocity`** (Component, lines 169-177)
  - Type: `Vec3` (wrapped in `Velocity(Vec3)`)
  - Purpose: Stores linear velocity
  - Default: `Vec3::ZERO`
  - Already integrated: `integrate_velocity` system applies velocity to `Transform::translation`

- **`BodySize`** (Component, lines 179-181)
  - Type: `Vec3` (wrapped in `BodySize(Vec3)`)
  - Purpose: Stores full entity dimensions (width, height, depth)
  - Used in: `spawn_entity` to create mesh and store size
  - **Issue:** Stores full size, not half-extents. For floor clamping, we need to know the entity's "height" (half-extents.y) to position the bottom correctly.

### 1.2 Current Physics Behavior

**Location:** `crates/anchor/src/lib.rs` (lines 339-348)

- **`integrate_velocity` system:**
  - Runs in `FixedUpdate` within `AnchorSystemSet::Integrate`
  - Reads `Transform` + `Velocity`
  - Applies: `transform.translation += velocity.0 * delta`
  - **No gravity applied**
  - **No floor clamping**
  - Entities can move freely in 3D space, including below y=0

### 1.3 Entity Spawning

**Location:** `crates/anchor/src/lib.rs` (lines 275-302)

- `spawn_entity` creates entities with:
  - `PbrBundle` (mesh, material, transform)
  - `Velocity` component (from `SpawnArgs.vel`)
  - `BodySize` component (from `SpawnArgs.size`)
  - Position set via `Transform::from_translation(position)`

**Location:** `apps/app/src/main.rs` (lines 156-167)

- Static cube spawned at `Vec3::ZERO` with:
  - No `Velocity` component
  - No `BodySize` component
  - Just `PbrBundle` + `Name`

### 1.4 Floor Clamping Status

**Current State:** **NO floor clamping exists.**

- No system checks `transform.translation.y`
- No system prevents entities from going below y=0
- The mentioned "bug" (entities clamped with center on floor) suggests there was previous clamping logic that was removed or never committed.

### 1.5 Summary

- Entities have `Velocity` and `BodySize` components
- `BodySize` stores full dimensions (not half-extents)
- Velocity integration works but has no constraints
- No gravity system
- No floor clamping
- Static entities in `apps/app/src/main.rs` don't participate in physics

---

## 2. M2 Design: High-Level Plan

### 2.1 Goals

1. **Add `Collider` component** with half-extents for accurate floor clamping
2. **Implement gravity** (simple downward acceleration)
3. **Implement floor clamping** so entity bottoms sit on y=0
4. **Wire static cube** to demonstrate the system
5. **Add debug metrics** to HUD (optional but useful)

### 2.2 Component Design

#### `Collider` Component

**Location:** `crates/world_state/src/lib.rs` (new file or add to existing)

```rust
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Collider {
    pub half_extents: Vec3,  // Half-width, half-height, half-depth
}
```

**Rationale:**
- Half-extents are standard for AABB (Axis-Aligned Bounding Box) colliders
- For floor clamping: `bottom_y = transform.translation.y - collider.half_extents.y`
- We clamp so `bottom_y >= 0.0`, meaning `transform.translation.y >= collider.half_extents.y`

**Migration:**
- `BodySize` can remain for mesh creation, but `Collider` is the source of truth for physics
- When spawning, derive `Collider` from `BodySize`: `Collider { half_extents: size * 0.5 }`

#### `Velocity` Component

**Status:** Already exists in `crates/anchor/src/lib.rs`

- No changes needed
- Will be used by gravity system

### 2.3 System Design

#### Gravity System

**Location:** `crates/anchor/src/lib.rs` (new system)

**Name:** `apply_gravity`

**Behavior:**
- Runs in `FixedUpdate`, before `integrate_velocity`
- Queries entities with `Velocity` component (optional `Collider` for future use)
- Applies downward acceleration: `velocity.y -= GRAVITY * delta`
- Constant: `const GRAVITY: f32 = 9.81;` (or configurable via resource)

**System Order:**
- `apply_gravity` → `integrate_velocity` → `clamp_to_floor`

#### Floor Clamping System

**Name:** `clamp_to_floor`

**Behavior:**
- Runs in `FixedUpdate`, after `integrate_velocity`
- Queries entities with `Transform` + `Collider`
- For each entity:
  - Calculate `bottom_y = transform.translation.y - collider.half_extents.y`
  - If `bottom_y < 0.0`:
    - Set `transform.translation.y = collider.half_extents.y` (so bottom sits at y=0)
    - If entity has `Velocity`, set `velocity.y = 0.0` (stop falling)
- Count clamped entities (for debug/metrics)

**System Order:**
- After `integrate_velocity` in `AnchorSystemSet::Integrate`

### 2.4 Metrics/Debug Info

**Optional Enhancement:**
- Add a resource `PhysicsMetrics` with:
  - `entities_with_velocity: usize`
  - `entities_clamped_this_frame: usize`
- Expose to HUD (future work, not required for M2)

---

## 3. Concrete File-Level Changes

### 3.1 `crates/world_state/src/lib.rs`

**Action:** Add `Collider` component

**Changes:**
- Add `Collider` struct (see design above)
- Ensure it implements `Component`, `Debug`, `Clone`, `Copy`, `Deref`, `DerefMut`, `Reflect`
- Export it: `pub use collider::Collider;` (if in a module) or just `pub struct Collider { ... }`

**Dependencies:**
- `bevy::prelude::*` (for `Component`, `Reflect`, `Vec3`)
- `bevy::reflect::Reflect`

### 3.2 `crates/anchor/src/lib.rs`

**Action:** Add gravity and floor clamping systems

**Changes:**

1. **Add gravity constant:**
   ```rust
   const GRAVITY: f32 = 9.81;
   ```

2. **Add `apply_gravity` system:**
   - Query: `Query<&mut Velocity>`
   - Apply: `velocity.y -= GRAVITY * time.delta_seconds()`
   - Run in `FixedUpdate`, before `integrate_velocity`

3. **Add `clamp_to_floor` system:**
   - Query: `Query<(&mut Transform, &Collider, Option<&mut Velocity>)>`
   - For each entity:
     - `let bottom_y = transform.translation.y - collider.half_extents.y;`
     - If `bottom_y < 0.0`:
       - `transform.translation.y = collider.half_extents.y;`
       - If `Velocity` present: `velocity.y = 0.0;`
   - Run in `FixedUpdate`, after `integrate_velocity`

4. **Update `AnchorPlugin::build`:**
   - Register `Collider` type: `.register_type::<Collider>()`
   - Add systems in order:
     - `apply_gravity` (before `integrate_velocity`)
     - `clamp_to_floor` (after `integrate_velocity`)
   - Both in `AnchorSystemSet::Integrate`

5. **Update `spawn_entity`:**
   - After creating entity, add `Collider`:
     ```rust
     .insert(Collider {
         half_extents: size * 0.5,
     })
     ```

**Dependencies:**
- Import `Collider` from `world_state`
- Import `Time<Fixed>` (already present)

### 3.3 `apps/app/src/main.rs`

**Action:** Wire static cube to use physics

**Changes:**

1. **Import `Collider` and `Velocity`:**
   ```rust
   use anchor::{Velocity, BodySize};  // Already imported or add
   use world_state::Collider;
   ```

2. **Update `setup` function:**
   - Modify cube spawn (lines 156-167) to:
     - Set initial position above floor: `Transform::from_xyz(0.0, 1.0, 0.0)` (or higher)
     - Add `Velocity(Vec3::ZERO)` component
     - Add `Collider { half_extents: Vec3::new(0.5, 0.5, 0.5) }` (for 1x1x1 cube)
     - Optionally add `BodySize(Vec3::new(1.0, 1.0, 1.0))` for consistency

**Example spawn:**
```rust
commands
    .spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.85, 1.0),
            perceptual_roughness: 0.5,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),  // Start above floor
        ..default()
    })
    .insert(Name::new("cube_1"))
    .insert(Velocity(Vec3::ZERO))  // Start with zero velocity
    .insert(Collider {
        half_extents: Vec3::new(0.5, 0.5, 0.5),  // Half of 1x1x1
    });
```

### 3.4 `crates/world_state/Cargo.toml`

**Action:** Ensure Bevy dependencies

**Check:**
- `bevy` dependency includes `bevy_reflect` feature (should be in workspace deps)
- No changes needed if workspace deps are correct

### 3.5 `crates/anchor/Cargo.toml`

**Action:** Add `world_state` dependency

**Changes:**
- Add: `world_state = { path = "../world_state" }`
- Verify it's already present (check existing deps)

---

## 4. Example Entity Setup

### 4.1 Spawned via Intent (HTTP API)

When `Spawn` intent is received, `spawn_entity` will automatically:
- Create `Collider` from `BodySize` (half-extents = size * 0.5)
- Apply gravity if `Velocity` is non-zero
- Clamp to floor after integration

### 4.2 Static Cube in `apps/app/src/main.rs`

**Current:** Spawned at `Vec3::ZERO` with no physics components.

**After M2:** Spawned at `(0.0, 2.0, 0.0)` with:
- `Velocity(Vec3::ZERO)` — will fall due to gravity
- `Collider { half_extents: Vec3::new(0.5, 0.5, 0.5) }` — bottom will clamp to y=0

**Expected Behavior:**
- Cube starts at y=2.0
- Gravity applies downward acceleration
- Cube falls until bottom hits y=0.0 (center at y=0.5)
- Velocity.y becomes 0.0 on contact

---

## 5. Codex-Ready Implementation Prompt

```
=== M2 Entity System Upgrade Implementation ===

You are implementing Milestone M2 for the Nexus engine: Entity System Upgrade.

CONTEXT:
- M1 (Perception → Runtime) is complete.
- Current state: Entities have Velocity and BodySize components, but no gravity or floor clamping.
- Goal: Add Collider component, gravity system, and floor clamping so entity bottoms sit on y=0.

TASKS:

1. Add Collider Component
   - File: `crates/world_state/src/lib.rs`
   - Add `Collider` struct:
     ```rust
     #[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
     #[reflect(Component)]
     pub struct Collider {
         pub half_extents: Vec3,
     }
     ```
   - Ensure it's exported (public)

2. Update Anchor Plugin
   - File: `crates/anchor/src/lib.rs`
   - Add gravity constant: `const GRAVITY: f32 = 9.81;`
   - Add `apply_gravity` system:
     - Queries `Query<&mut Velocity>`
     - Applies: `velocity.y -= GRAVITY * time.delta_seconds()`
     - Runs in `FixedUpdate`, before `integrate_velocity`
   - Add `clamp_to_floor` system:
     - Queries `Query<(&mut Transform, &Collider, Option<&mut Velocity>)>`
     - For each entity:
       - Calculate `bottom_y = transform.translation.y - collider.half_extents.y`
       - If `bottom_y < 0.0`:
         - Set `transform.translation.y = collider.half_extents.y`
         - If `Velocity` present: `velocity.y = 0.0`
     - Runs in `FixedUpdate`, after `integrate_velocity`
   - In `AnchorPlugin::build`:
     - Register `Collider`: `.register_type::<Collider>()`
     - Add systems in order: `apply_gravity` → `integrate_velocity` → `clamp_to_floor`
     - Both new systems in `AnchorSystemSet::Integrate`
   - Update `spawn_entity`:
     - After inserting `BodySize`, also insert:
       ```rust
       .insert(Collider {
           half_extents: size * 0.5,
       })
       ```

3. Wire Static Cube
   - File: `apps/app/src/main.rs`
   - Import: `use world_state::Collider;`
   - In `setup` function, update cube spawn (around line 156):
     - Change position to: `Transform::from_xyz(0.0, 2.0, 0.0)`
     - Add: `.insert(Velocity(Vec3::ZERO))`
     - Add: `.insert(Collider { half_extents: Vec3::new(0.5, 0.5, 0.5) })`

4. Verify Dependencies
   - File: `crates/anchor/Cargo.toml`
   - Ensure `world_state = { path = "../world_state" }` is present

CONSTRAINTS:
- Don't break existing HUD (it's a no-op, but don't remove it)
- Keep floor at y=0 (hardcoded for now)
- Follow existing code style (tracing for logs, etc.)
- Use `FixedUpdate` timing for physics systems
- Ensure `Collider` is registered for reflection

EXPECTED BEHAVIOR:
- Cube spawns at y=2.0
- Falls due to gravity
- Stops when bottom hits y=0.0 (center at y=0.5)
- No entities can go below y=0

VERIFICATION:
- Run `cargo run -p app`
- Observe cube falling and stopping at floor
- Check logs for any errors
```

---

## 6. Suggested STATUS.md Updates

```markdown
# ✔ Nexus STATUS (Daily)

## Today's Focus
- [ ] Implement Collider component in world_state
- [ ] Add gravity system to anchor
- [ ] Add floor clamping system to anchor
- [ ] Wire static cube to use Collider and demonstrate clamping
- [ ] Test: cube falls and stops at floor (y=0)

## Recently Completed
- M1 — Perception → Runtime Integration
  - perception/server.py hardened and serving /health + /frame (200) with MiDaS + MoveNet
  - /frame writes perception/state.json
  - Runtime reads perception/state.json and logs persons
  - Bevy renderer builds and shows basic world

## Blockers
None.

---
```

---

## 7. Design Decisions Log

**2025-11-20 — M2 Design:**
- **Collider uses half-extents** (not full size) for standard AABB representation
- **Gravity constant: 9.81 m/s²** (standard Earth gravity, can be made configurable later)
- **Floor at y=0** (hardcoded for simplicity, can be made configurable later)
- **Gravity applies to all entities with Velocity** (no "grounded" flag yet; can be added in M3)
- **BodySize remains** for mesh creation; Collider is physics source of truth
- **System order:** `apply_gravity` → `integrate_velocity` → `clamp_to_floor` (all in FixedUpdate)

---

## 8. Future Enhancements (Post-M2)

- Configurable gravity (via resource)
- Configurable floor height (via resource)
- "Grounded" flag to disable gravity when on floor
- Collision detection between entities (M3+)
- Physics metrics in HUD
- Support for non-cuboid colliders (spheres, capsules)

---

**End of Design Document**

