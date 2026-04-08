import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  RouterProvider,
  useRouteError,
  isRouteErrorResponse,
  useNavigate,
} from "react-router";
import { initializeTheme } from "./stores/themeStore";
import "./index.css";
import { AppLayout } from "./pages/AppLayout";
import { Home } from "./features/projects/Home";
import { WizardLayout } from "./features/wizard/WizardLayout";
import { Describe } from "./features/wizard/Describe";
import { Plan } from "./features/wizard/Plan";
import { Atomize } from "./features/wizard/Atomize";
import { Configure } from "./features/wizard/Configure";
import { Launch } from "./features/wizard/Launch";
import { Monitor } from "./features/monitor/Monitor";

initializeTheme();

function FrozenPlaceholder() {
  return (
    <div className="flex items-center justify-center h-full bg-void">
      <p className="text-text-dim text-xs font-mono uppercase tracking-widest">
        Feature frozen — pending reboot slice
      </p>
    </div>
  );
}

function RouteError() {
  const error = useRouteError();
  const navigate = useNavigate();
  const isNotFound = isRouteErrorResponse(error) && error.status === 404;

  return (
    <div className="flex items-center justify-center h-screen bg-void">
      <div className="text-center max-w-md px-6">
        <p className="text-5xl font-bold font-mono text-primary text-glow-primary mb-4">
          {isNotFound ? "404" : "ERR"}
        </p>
        <p className="text-sm font-sans text-text-muted mb-6">
          {isNotFound
            ? "This page doesn't exist or the project has moved."
            : "Something unexpected happened."}
        </p>
        <button
          onClick={() => navigate("/")}
          className="px-5 py-2 rounded bg-primary/10 text-primary text-sm font-sans hover:bg-primary/20 transition-colors"
        >
          ← Back to Home
        </button>
      </div>
    </div>
  );
}

const router = createBrowserRouter([
  {
    path: "/",
    Component: AppLayout,
    ErrorBoundary: RouteError,
    children: [
      { index: true, Component: Home },
      {
        path: "new",
        Component: WizardLayout,
        children: [
          { path: "describe", Component: Describe },
          { path: "describe/:id", Component: Describe },
          { path: "plan/:id", Component: Plan },
          { path: "atomize/:id", Component: Atomize },
          { path: "configure/:id", Component: Configure },
          { path: "launch/:id", Component: Launch },
        ],
      },
      { path: "monitor/:id", Component: Monitor },
      { path: "connections", Component: FrozenPlaceholder },
      { path: "plugins", Component: FrozenPlaceholder },
    ],
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
