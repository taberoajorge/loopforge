import { Link } from "react-router";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { useDisplayVocabularyStore } from "@/stores/displayVocabularyStore";
import type { Project } from "@/stores/projectStore";

type DraftCardProps = {
  project: Project;
  onDiscard: (id: string) => void;
};

export function DraftCard({ project, onDiscard }: DraftCardProps) {
  const vocabulary = useDisplayVocabularyStore((state) => state.vocabulary);
  const wizardStep = project.wizardStep ?? "describe";
  const stepLabel = vocabulary?.wizardStepLabels[wizardStep] ?? wizardStep;
  const title = project.name || "Untitled";
  const description = project.description || "No description";

  return (
    <Card variant="ghost" className="border-dashed" data-testid={`draft-card-${project.id}`}>
      <CardHeader className="gap-3 border-b-0">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 space-y-1">
            <CardTitle className="truncate font-mono" title={title}>
              {title}
            </CardTitle>
            <CardDescription className="truncate" title={description}>
              {description}
            </CardDescription>
          </div>
          <span className="shrink-0 rounded bg-elevated px-1.5 py-0.5 font-mono text-text-dim text-xs">
            {stepLabel}
          </span>
        </div>
      </CardHeader>
      <CardContent className="flex items-center gap-2 border-border/60 border-t px-4 py-3">
        <Link
          to={`/new/${wizardStep}/${project.id}`}
          data-testid={`draft-resume-${project.id}`}
          className={cn(buttonVariants({ variant: "primary", size: "sm" }))}
        >
          Resume
        </Link>
        <Button variant="ghost" size="sm" onClick={() => onDiscard(project.id)}>
          Discard
        </Button>
      </CardContent>
    </Card>
  );
}
