import { useState } from "react";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from "../../../components/ui/alert-dialog";
import { Button } from "../../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../../components/ui/card";

type LaunchActionsProps = {
  launching: boolean;
  onBack: () => void;
  onCancel: () => void;
  onLaunch: () => void;
};

export function LaunchActions({ launching, onBack, onCancel, onLaunch }: LaunchActionsProps) {
  const [confirmOpen, setConfirmOpen] = useState(false);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Execution actions</CardTitle>
        <CardDescription>Launch, go back, or cancel this step.</CardDescription>
      </CardHeader>
      <CardContent className="flex items-center gap-3">
        <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
          <AlertDialogTrigger asChild>
            <Button variant="ghost" disabled={launching}>
              Cancel
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Leave launch review?</AlertDialogTitle>
              <AlertDialogDescription>
                You can return to this step later from the wizard flow.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel asChild>
                <Button variant="secondary">Stay</Button>
              </AlertDialogCancel>
              <AlertDialogAction
                asChild
                onClick={() => {
                  onCancel();
                }}
              >
                <Button variant="primary" disabled={launching}>
                  Leave
                </Button>
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
        <Button variant="secondary" onClick={onBack} disabled={launching}>
          Back
        </Button>
        <Button variant="primary" className="flex-1" disabled={launching} onClick={onLaunch}>
          {launching ? "Initializing..." : "Launch loop"}
        </Button>
      </CardContent>
    </Card>
  );
}
