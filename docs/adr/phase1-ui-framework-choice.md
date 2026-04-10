# Phase 1 UI Framework Choice

## Status

Accepted for Phase 1

## Decision

Floem remains the Phase 1 choice while IDE-like custom surfaces are the higher priority.

Slint becomes the preferred option only if accessibility automation is a hard requirement for Phase 1. This is the switch trigger. Without that trigger, the project stays with Floem.

## Rationale

This is a trigger-based decision, not an open-ended comparison. The team is choosing between accessibility automation strength and IDE-like custom surfaces based on the primary Phase 1 constraint.

- Choose Floem when the Phase 1 priority is IDE-like custom surfaces and product-specific interface control.
- Choose Slint only when accessibility automation is the non-negotiable requirement that outweighs custom surface flexibility.
