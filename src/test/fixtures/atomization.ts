import type { AtomizeArgs, AtomizeProgress } from "../../lib/tauri";
import { type DeepPartial, mergeFixture } from "./shared";
import { createPrd, createUserStory } from "./wizard";

export function createAtomizeArgs(overrides?: DeepPartial<AtomizeArgs>): AtomizeArgs {
  return mergeFixture<AtomizeArgs>(
    {
      projectId: "project-001",
      projectName: "LoopForge",
      projectDir: "/work/loopforge",
      agent: "codex",
      model: "gpt-5.4",
      effort: "medium",
    },
    overrides,
  );
}

export function createAtomizeProgress(overrides?: DeepPartial<AtomizeProgress>): AtomizeProgress {
  return mergeFixture<AtomizeProgress>(
    {
      stage: 2,
      stageName: "Chunk",
      message: "Chunking plan",
      projectId: "project-001",
      elapsedMs: 0,
    },
    overrides,
  );
}

export function createAtomizedPrd() {
  return createPrd({
    stories: [
      createUserStory({
        id: "S-005",
        title: "Create typed IPC and event test doubles",
      }),
      createUserStory({
        id: "S-006",
        title: "Add integration tests for the wizard shell",
      }),
    ],
  });
}
