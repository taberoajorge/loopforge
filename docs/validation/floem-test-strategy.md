# Floem Phase 1 Validation Strategy

## Decision

LoopForge does not assume AccessKit-native E2E automation for Floem Phase 1. If that capability becomes real and stable later, it can extend coverage, but it is not a prerequisite for shipping or validating the Phase 1 UI port.

The first-line plan for Phase 1 E2E coverage is a headless-first stack with three required layers:

1. Headless harness tests
2. State tests
3. Snapshot tests

These layers are the baseline validation strategy now. They are not deferred to a later phase while waiting on native accessibility automation.

## Required Validation Layers

### Headless Harness Tests

Headless harness tests drive the Floem app through deterministic flows without assuming OS-level automation hooks. They cover the primary user journeys, event wiring, and cross-panel transitions that would otherwise be treated as E2E checks.

Phase 1 expectations:

- launch the app shell in a test harness
- execute core workflows through programmatic events
- verify visible route, panel, and action transitions
- confirm critical artifacts or side effects after each flow

### State Tests

State tests validate the application state machine beneath the UI surface. They prove that user actions, async events, and recovery paths produce the correct session, project, and workflow state even when native E2E tooling is unavailable.

Phase 1 expectations:

- assert initial, loading, success, failure, and recovery states
- cover reducer, store, or controller transitions that feed the UI
- verify that persisted state and in-memory state stay aligned

### Snapshot Tests

Snapshot tests lock down the rendered structure of the key Floem surfaces so regressions are visible early. They complement headless harness coverage by checking layout composition, conditional rendering, and stable output for known states.

Phase 1 expectations:

- snapshot primary screens and major panels
- snapshot important empty, active, blocked, and error states
- review snapshot changes as part of normal regression triage

## Phase 1 Coverage Rule

For Floem Phase 1, E2E validation starts with headless harness tests, state tests, and snapshot tests. This is the required coverage plan for launch readiness. AccessKit-native automation is explicitly outside the Phase 1 assumption set and must not block execution of the validation plan.
