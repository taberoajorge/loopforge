import { lazy, StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  RouterProvider,
  useRouteError,
  isRouteErrorResponse,
  useNavigate,
} from "react-router";
import { AppLayout } from "./pages/AppLayout";
import { Home } from "./features/projects/Home";

const WizardLayout = lazy(() => import("./features/wizard/WizardLayout").then((moduleItem) => ({ default: moduleItem.WizardLayout })));
const Describe = lazy(() => import("./features/wizard/Describe").then((moduleItem) => ({ default: moduleItem.Describe })));
const Plan = lazy(() => import("./features/wizard/Plan").then((moduleItem) => ({ default: moduleItem.Plan })));
const Atomize = lazy(() => import("./features/wizard/Atomize").then((moduleItem) => ({ default: moduleItem.Atomize })));
const Configure = lazy(() => import("./features/wizard/Configure").then((moduleItem) => ({ default: moduleItem.Configure })));
const Launch = lazy(() => import("./features/wizard/Launch").then((moduleItem) => ({ default: moduleItem.Launch })));
const Monitor = lazy(() => import("./features/monitor/Monitor").then((moduleItem) => ({ default: moduleItem.Monitor })));

function LazyFallback() {
  return (
    <div className="flex items-center justify-center h-full bg-void">
      <p className="text-text-dim text-xs font-mono animate-pulse">Loading...</p>
    </div>
  );
}

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
  const routeError = useRouteError();
  const navigate = useNavigate();
  const isNotFound = isRouteErrorResponse(routeError) && routeError.status === 404;

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

function SuspenseWrap({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<LazyFallback />}>{children}</Suspense>;
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
        element: <SuspenseWrap><WizardLayout /></SuspenseWrap>,
        children: [
          { path: "describe", element: <SuspenseWrap><Describe /></SuspenseWrap> },
          { path: "describe/:id", element: <SuspenseWrap><Describe /></SuspenseWrap> },
          { path: "plan/:id", element: <SuspenseWrap><Plan /></SuspenseWrap> },
          { path: "atomize/:id", element: <SuspenseWrap><Atomize /></SuspenseWrap> },
          { path: "configure/:id", element: <SuspenseWrap><Configure /></SuspenseWrap> },
          { path: "launch/:id", element: <SuspenseWrap><Launch /></SuspenseWrap> },
        ],
      },
      { path: "monitor/:id", element: <SuspenseWrap><Monitor /></SuspenseWrap> },
      { path: "connections", Component: FrozenPlaceholder },
      { path: "plugins", Component: FrozenPlaceholder },
    ],
  },
]);

export function mountLegacyRouter(rootElement: HTMLElement) {
  createRoot(rootElement).render(
    <StrictMode>
      <RouterProvider router={router} />
    </StrictMode>,
  );
}
