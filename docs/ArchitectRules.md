# 🧱 Architect Rules for Nexus

## Purpose
These rules define how AI should operate inside this repository.

---

# Roles

## 1. ChatGPT = Lead System Architect
- Reads ProjectBrain.md before planning.
- Never assume — always ask for STATUS.md.
- Maintains architecture, milestones, and N-level plan.
- Produces specs, workflows, prompts, diagrams.
- Validates design changes.

## 2. Cursor = In-Repo Architect & Engineer
- Reads the entire repo.
- Ensures code consistency with ProjectBrain.md.
- Proposes file changes when architecture updates.
- Handles multi-file edits and refactors.
- Executes engineering tasks precisely.
- Asks questions when design unclear.

## 3. Codex = Senior Code Builder
- Writes large volumes of code.
- Implements subsystems based on Cursor’s plan.
- Follows architecture defined in ProjectBrain.md.

---

# Workflow

### Step 1 — User updates STATUS.md
### Step 2 — ChatGPT checks ProjectBrain + STATUS, creates plan
### Step 3 — Cursor executes plan inside repo
### Step 4 — Codex handles heavy code writing
### Step 5 — ProjectBrain is updated weekly

---

# Rules

1. No file or system is created without matching architecture layer.
2. Every subsystem must appear in ProjectBrain.md.
3. Cursor must ask questions when unclear.
4. Codex must not change architecture — only implement.
5. ChatGPT must maintain consistency across layers.

---
