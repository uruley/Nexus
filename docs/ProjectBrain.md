# 🧠 Nexus Project Brain  
The single source of truth for architecture, design, and project direction.

---

# 0. What Nexus Is
Nexus is an AI-native game engine built with a 6-layer architecture:
1. Runtime — JSON patch-based engine loop controlling world state  
2. Perception Kit — depth, pose, and flow via ONNX models  
3. Neural Motion Compiler — video → skeleton → animation  
4. Neural Renderer — neural/splat/NeRF rendering  
5. Command Router — NL → schema → JSON patches for runtime  
6. Docs & Tools — project brain, status, prompts, workflows  

All engineering must align with this architecture.  
All decisions must be logged in Section 5.

---

# 1. Current Architecture (High-Level)

## 1.1 Runtime
**Purpose:** World state, entity updates, JSON patch engine.  
**Files:**  
- `runtime/main.py`  
- `runtime/systems/*.py`  
- `schema.json`  
**Status:** Basic engine loop works. Render window works. Collider system WIP.

## 1.2 Perception Kit
**Purpose:** Use MoveNet & MiDaS to produce pose & depth.  
**Files:**  
- `perception/server.py`  
- `perception/models/movenet_thunder.onnx`  
**Status:** Server live. Model missing. Not wired to runtime.

## 1.3 Neural Motion Compiler
**Purpose:** Video → pose → rig → animation clip.  
**Files:**  
- (planned) `motion_compiler/`  
**Status:** Not implemented.

## 1.4 Neural Renderer
**Purpose:** Neural or splat-based rendering experiments.  
**Files:**  
- (planned) `renderer/`  
**Status:** Not implemented.

## 1.5 Command Router
**Purpose:** Natural language → structured command → patch.  
**Files:**  
- `router/schema.json`  
- (planned) `router/server.py`  
**Status:** Schema exists; router missing.

## 1.6 Docs & Tools
**Purpose:** Maintain architectural clarity across months.  
**Files:**  
- `docs/ProjectBrain.md`  
- `docs/STATUS.md`  
- `docs/FileIndex.md`  
- `docs/ArchitectRules.md`  
**Status:** Architect Mode initializing.

---

# 2. Active Milestone (M1)
**M1 — Perception → Runtime Integration**

Goal:  
Pipe pose and/or depth into runtime world state.

Tasks:  
- [ ] Add `movenet_thunder.onnx`  
- [ ] Fix `/health` to pass  
- [ ] Write pose/depth → `perception/state.json`  
- [ ] Runtime reads `perception/state.json`  
- [ ] Attach to test entity (e.g., “player”)  

Outputs:  
- Working perception → runtime link  
- First AI-driven entity

---

# 3. Next Milestones

## M2 — Entity System Upgrade
- Collider fix  
- Physics basics  
- Floor clamp  

## M3 — Command Router MVP
- NL → action schema  
- Router server  
- Runtime patch interpreter  

## M4 — Neural Renderer Prototype
- Basic splat renderer prototype  

---

# 4. Repository Standards
All files belong to one of the 6 layers.  
Every change must be reflected in ProjectBrain.md.  
Changes to architecture must update Section 5 below.

---

# 5. Decisions Log
**2025-11-18** — Architect Mode enabled.  
**2025-11-18** — Cursor designated In-Repo Architect.  
**2025-11-18** — Codex designated Senior Builder.  
**2025-11-18** — ChatGPT acts as Lead System Architect.  
**2025-11-18** — Unity optional, Nexus remains custom engine first.  

---

# 6. How to Work With Project Brain
- ChatGPT reads this entire document before planning.  
- Cursor uses this to validate code structure.  
- You update milestone status or file lists weekly.  
- STATUS.md handles daily tasks.  

---
