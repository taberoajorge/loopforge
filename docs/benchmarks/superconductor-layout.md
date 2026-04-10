# Superconductor Layout Benchmark

## Purpose

This note captures how LoopForge can use Superconductor as a layout-only benchmark during the refactor. The comparison is limited to interface composition for three surfaces: terminal, diff, and guided review.

Superconductor is not an architectural model for the refactor. It must not be used to infer service boundaries, process structure, state ownership, storage design, or event flow.

## In Scope

Only compare the product layout of these surfaces:

- Terminal
- Diff
- Guided review

Any other Superconductor surface or workflow is out of scope for this benchmark.

## Layout Questions

Use the benchmark to answer layout questions only:

- How much screen space should the terminal occupy relative to surrounding controls and status panels?
- Where should diff content sit in relation to file navigation, change summaries, and action controls?
- How should guided review steps sequence on screen so the next action stays obvious without hiding context?
- What information should remain pinned while a user scrolls terminal output, diff hunks, or review guidance?
- When should panels stack, dock, or collapse across narrow and wide window sizes?
- Which controls need persistent placement versus contextual placement inside terminal, diff, and guided review surfaces?

## Out of Scope

This benchmark does not prescribe internal structure. Do not copy architecture, providers, orchestration patterns, background services, or module boundaries from Superconductor.
