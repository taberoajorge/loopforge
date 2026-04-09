import { useState, type DragEvent } from "react";
import type { UserStory } from "../../../stores/wizardStore";
import { Badge } from "../../../components/ui/badge";
import { Button } from "../../../components/ui/button";
import { Card, CardContent, CardHeader } from "../../../components/ui/card";
import { Input } from "../../../components/ui/input";
import { ScrollArea, ScrollContent, ScrollViewport } from "../../../components/ui/scroll-area";
import { Textarea } from "../../../components/ui/textarea";

const PRIORITY_BADGE: Record<UserStory["priority"], "danger" | "warning" | "info" | "neutral"> = {
  critical: "danger",
  high: "warning",
  medium: "info",
  low: "neutral",
};

type AtomizeStoryListProps = {
  stories: UserStory[];
  atomizeStarted: boolean;
  atomizeError: string | null;
  onUpdateStory: (storyId: string, patch: Partial<UserStory>) => void;
  onRequestRemoveStory: (story: UserStory) => void;
  onDragStart: (index: number) => void;
  onDragOver: (event: DragEvent) => void;
  onDrop: (index: number) => void;
};

function StoryCard({
  story,
  index,
  onUpdateStory,
  onRequestRemoveStory,
  onDragStart,
  onDragOver,
  onDrop,
}: {
  story: UserStory;
  index: number;
  onUpdateStory: (storyId: string, patch: Partial<UserStory>) => void;
  onRequestRemoveStory: (story: UserStory) => void;
  onDragStart: (index: number) => void;
  onDragOver: (event: DragEvent) => void;
  onDrop: (index: number) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [titleDraft, setTitleDraft] = useState(story.title);
  const [descriptionDraft, setDescriptionDraft] = useState(story.description);

  function commitEdit() {
    onUpdateStory(story.id, {
      title: titleDraft.trim() || story.title,
      description: descriptionDraft.trim(),
    });
    setEditing(false);
  }

  return (
    <Card draggable role="article" aria-label={`Story ${story.id}`} data-testid={`atomize-story-${story.id}`} onDragStart={() => onDragStart(index)} onDragOver={onDragOver} onDrop={() => onDrop(index)}>
      <CardHeader className="flex flex-row items-center justify-between gap-3 p-3">
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono font-semibold text-primary">{story.id}</span>
          <Badge variant={PRIORITY_BADGE[story.priority]}>{story.priority}</Badge>
        </div>
        <Button variant="ghost" size="sm" data-testid={`atomize-story-remove-${story.id}`} onClick={() => onRequestRemoveStory(story)}>
          Remove
        </Button>
      </CardHeader>
      <CardContent className="space-y-2 p-3">
        {editing ? (
          <>
            <Input value={titleDraft} onChange={(event) => setTitleDraft(event.target.value)} />
            <Textarea rows={2} value={descriptionDraft} onChange={(event) => setDescriptionDraft(event.target.value)} />
            <div className="flex items-center gap-2">
              <Button variant="primary" size="sm" onClick={commitEdit}>
                Save
              </Button>
              <Button variant="secondary" size="sm" onClick={() => setEditing(false)}>
                Cancel
              </Button>
            </div>
          </>
        ) : (
          <>
            <button className="w-full text-left" aria-label={`Edit story ${story.id}`} data-testid={`atomize-story-edit-${story.id}`} onClick={() => setEditing(true)}>
              <p className="text-sm font-medium text-text">{story.title}</p>
              {story.description ? <p className="text-xs text-text-muted">{story.description}</p> : null}
            </button>
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="neutral">EST {story.estimatedMinutes}m</Badge>
              <Badge variant="neutral">{story.acceptanceCriteria.length} criteria</Badge>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

export function AtomizeStoryList({
  stories,
  atomizeStarted,
  atomizeError,
  onUpdateStory,
  onRequestRemoveStory,
  onDragStart,
  onDragOver,
  onDrop,
}: AtomizeStoryListProps) {
  if (!atomizeStarted || stories.length === 0) {
    const message = atomizeError
      ? "Atomization failed — add stories manually or go back."
      : "Atomizing plan into stories...";
    return <div className="flex h-full items-center justify-center p-6 text-sm text-text-dim" role="status" data-testid="atomize-story-list-status">{message}</div>;
  }

  return (
    <ScrollArea className="h-full" role="region" aria-label="Atomized stories" data-testid="atomize-story-list">
      <ScrollViewport className="h-full" padding="md">
        <ScrollContent className="space-y-3">
          {stories.map((story, index) => (
            <StoryCard
              key={story.id}
              story={story}
              index={index}
              onUpdateStory={onUpdateStory}
              onRequestRemoveStory={onRequestRemoveStory}
              onDragStart={onDragStart}
              onDragOver={onDragOver}
              onDrop={onDrop}
            />
          ))}
        </ScrollContent>
      </ScrollViewport>
    </ScrollArea>
  );
}
