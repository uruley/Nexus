# ✔ Nexus STATUS (Today)

## Today’s Focus
- [ ] (Optional right now) Track down why /frame is still HTTP 500 under uvicorn
- [ ] Log this as a known issue and move to M2 once we have energy

## Recently Completed
- Runtime loop stable
- Perception server starts successfully
- State file written at perception/state.json
- Runtime reads perception/state.json and logs "Perception: 0 persons"
- Core M1 objective (Perception → Runtime wiring) achieved

## Known Issues (M1 cleanup)
- /frame returns HTTP 500 via uvicorn even though the handler has a safe fallback
- /health still shows "movenet": false (model not fully wired/compatible yet)

## Blockers
- None for continuing to M2 (we can treat /frame 500 + movenet as follow-up bugs)
