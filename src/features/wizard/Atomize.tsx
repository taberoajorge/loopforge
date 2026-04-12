import { useRef, useState, type DragEvent } from "react";
import { useNavigate, useParams } from "react-router";
import { discardDraft, saveDraft, savePrd, type Prd } from "../../lib/tauri";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "../../components/ui/card";
import { Progress } from "../../components/ui/progress";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../../components/ui/tabs";
import { useWizardStore, type UserStory } from "../../stores/wizardStore";
import { useAtomizerPipeline, STAGE_BADGE, STAGE_LABEL } from "../../hooks/useAtomizerPipeline";
import { useAtomizerActivity } from "../../hooks/useAtomizerActivity";
import { buildDraftPayload } from "../../lib/draft-payload";
import { AtomizeConfirmationDialog } from "./components/AtomizeConfirmationDialog";
import { AtomizeStreamPanel } from "./components/AtomizeStreamPanel";
import { AtomizeStoryList } from "./components/AtomizeStoryList";

function makeBlankStory(existingCount: number): UserStory {
  const paddedId = String(existingCount + 1).padStart(3, "0");
  return {
    id: `S-${paddedId}`,
    title: "New story",
    description: "",
    acceptanceCriteria: [],
    scope: { filesToModify: [], filesToCreate: [], filesToAvoid: [] },
    verification: { commands: [], assertions: [] },
    priority: "medium",
    estimatedComplexity: "medium",
    estimatedMinutes: 30,
    dependsOn: [],
    passes: false,
    blocked: false,
    attempts: 0,
    notes: null,
  };
}

export function Atomize() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const stories = useWizardStore((state) => state.stories);
  const reorderStories = useWizardStore((state) => state.reorderStories);
  const addStory = useWizardStore((state) => state.addStory);
  const updateStory = useWizardStore((state) => state.updateStory);
  const removeStory = useWizardStore((state) => state.removeStory);
  const advanceStep = useWizardStore((state) => state.advanceStep);
  const dragIndexRef = useRef<number | null>(null);
  const [discardOpen, setDiscardOpen] = useState(false);
  const [storyToRemove, setStoryToRemove] = useState<UserStory | null>(null);

  const pipeline = useAtomizerPipeline(id);
  const activityEvents = useAtomizerActivity(id);
  const totalMinutes = stories.reduce((sum, story) => sum + story.estimatedMinutes, 0);
  const totalHours = (totalMinutes / 60).toFixed(1);
  const addStoryDisabled = !pipeline.atomizeStarted || pipeline.isRunning;
  const nextDisabled = pipeline.isRunning || stories.length === 0;
  const footerStatus = !pipeline.atomizeStarted || pipeline.isRunning
    ? `Processing · ${pipeline.pipelinePercent}% · ${pipeline.formatElapsed(pipeline.elapsedSeconds)}`
    : pipeline.atomizeError
      ? "Atomization failed"
      : pipeline.isDone && stories.length === 0
        ? "Atomization Complete · 0 Stories"
        : `Atomization Complete · ${stories.length} Stories · ${totalHours}h`;

  const handleDragStart = (index: number) => { dragIndexRef.current = index; };
  const handleDragOver = (event: DragEvent) => { event.preventDefault(); };
  const handleDrop = (toIndex: number) => {
    if (dragIndexRef.current === null || dragIndexRef.current === toIndex) return;
    reorderStories(dragIndexRef.current, toIndex);
    dragIndexRef.current = null;
  };

  async function persistStoriesToDisk() {
    const snap = useWizardStore.getState();
    if (!id || snap.stories.length === 0) return;
    const prd: Prd = {
      projectName: snap.projectData.name,
      generatedAt: new Date().toISOString(),
      totalEstimatedMinutes: totalMinutes,
      stories: snap.stories,
    };
    await savePrd(id, JSON.stringify(prd, null, 2)).catch(() => {});
  }

  async function handleNext() {
    if (!id) return;
    await persistStoriesToDisk();
    await saveDraft(id, buildDraftPayload(id, "configure")).catch(() => {});
    advanceStep(4);
    navigate(`/new/configure/${id}`);
  }

  async function confirmDiscardDraft() {
    if (!id) return;
    await discardDraft(id).catch(() => {});
    navigate("/");
  }

  return (
    <div className="flex h-full min-h-0 gap-4 p-4">
      <Card className="flex w-80 shrink-0 flex-col overflow-hidden">
        <Tabs defaultValue="queue" className="flex min-h-0 flex-1 flex-col">
          <CardHeader className="pb-2">
            <TabsList className="w-full">
              <TabsTrigger value="queue" className="flex-1">Queue</TabsTrigger>
              <TabsTrigger value="activity" className="flex-1">
                Activity{activityEvents.length > 0 ? ` (${activityEvents.length})` : ""}
              </TabsTrigger>
            </TabsList>
          </CardHeader>
          <TabsContent value="queue" className="flex flex-col px-4 pb-4">
            <div className="space-y-4">
              <Progress value={pipeline.pipelinePercent} label="System Health" valueLabel={`${pipeline.pipelinePercent}%`} />
              <ScrollArea className="max-h-48">
                <ScrollViewport className="h-full">
                  <ScrollContent className="space-y-2">
                    {pipeline.stages.map((stage) => (
                      <div key={stage.number} className="flex items-center justify-between">
                        <span className="text-xs text-text-muted">{stage.label}</span>
                        <Badge variant={STAGE_BADGE[stage.status]} className={`min-w-[4rem] justify-center text-center${stage.status === "running" ? " animate-pulse" : ""}`}>
                          {STAGE_LABEL[stage.status]}
                        </Badge>
                      </div>
                    ))}
                  </ScrollContent>
                </ScrollViewport>
              </ScrollArea>
              {pipeline.stageMessage ? (
                <p className="text-xs text-text-muted">
                  {pipeline.stageMessage}
                  {pipeline.isRunning ? ` · ${pipeline.formatElapsed(pipeline.elapsedSeconds)}` : ""}
                </p>
              ) : null}
              {pipeline.atomizeError ? <p className="text-xs text-blocked">{pipeline.atomizeError}</p> : null}
            </div>
          </TabsContent>
          <TabsContent value="activity" className="min-h-0 flex-1">
            <AtomizeStreamPanel
              events={activityEvents}
              isRunning={pipeline.isRunning}
              isDone={pipeline.isDone}
              hasError={Boolean(pipeline.atomizeError)}
            />
          </TabsContent>
        </Tabs>
      </Card>
      <Card className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <CardHeader className="flex-row items-center justify-between gap-3">
          <CardTitle>Atomic Blueprint</CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="neutral">{stories.length} stories</Badge>
            <Badge variant="neutral">{totalHours}h est</Badge>
            <Button variant="outline" size="sm" disabled={addStoryDisabled} onClick={() => addStory(makeBlankStory(stories.length))}>Add story</Button>
          </div>
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-0">
          <AtomizeStoryList
            stories={stories}
            atomizeStarted={pipeline.atomizeStarted}
            isDone={pipeline.isDone}
            atomizeError={pipeline.atomizeError}
            onUpdateStory={updateStory}
            onRequestRemoveStory={setStoryToRemove}
            onDragStart={handleDragStart}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
          />
        </CardContent>
        <CardFooter className="justify-between">
          <Button variant="secondary" size="sm" onClick={() => setDiscardOpen(true)}>Discard draft</Button>
          <span className={`text-xs text-text-muted${pipeline.isRunning ? " animate-pulse" : ""}`}>{footerStatus}</span>
          <Button variant="primary" onClick={handleNext} disabled={nextDisabled}>Proceed to validation</Button>
        </CardFooter>
      </Card>
      <AtomizeConfirmationDialog open={discardOpen} onOpenChange={setDiscardOpen} title="Discard this draft?" description="Plan and PRD artifacts will be deleted." confirmLabel="Discard draft" onConfirm={confirmDiscardDraft} />
      <AtomizeConfirmationDialog
        open={Boolean(storyToRemove)}
        onOpenChange={(open) => { if (!open) setStoryToRemove(null); }}
        title="Remove story?"
        description={storyToRemove ? `${storyToRemove.id} will be removed from the atomization list.` : "This story will be removed from the atomization list."}
        confirmLabel="Remove story"
        onConfirm={() => { if (!storyToRemove) return; removeStory(storyToRemove.id); setStoryToRemove(null); }}
      />
    </div>
  );
}
