import { lazy, StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  isRouteErrorResponse,
  RouterProvider,
  useNavigate,
  useRouteError,
} from "react-router";
import { initializeTheme } from "./stores/themeStore";
import "./index.css";
import { Home } from "./features/projects/Home";
import { AppLayout } from "./pages/AppLayout";

const WizardLayout = lazy(() =>
  import("./features/wizard/WizardLayout").then((mod) => ({ default: mod.WizardLayout })),
);
const Describe = lazy(() =>
  import("./features/wizard/Describe").then((mod) => ({ default: mod.Describe })),
);
const Plan = lazy(() => import("./features/wizard/Plan").then((mod) => ({ default: mod.Plan })));
const Atomize = lazy(() =>
  import("./features/wizard/Atomize").then((mod) => ({ default: mod.Atomize })),
);
const Configure = lazy(() =>
  import("./features/wizard/Configure").then((mod) => ({ default: mod.Configure })),
);
const Launch = lazy(() =>
  import("./features/wizard/Launch").then((mod) => ({ default: mod.Launch })),
);
const Monitor = lazy(() =>
  import("./features/monitor/Monitor").then((mod) => ({ default: mod.Monitor })),
);

initializeTheme();

function LazyFallback() {
  return (
    <div className="flex h-full items-center justify-center bg-void">
      <p className="animate-pulse font-mono text-text-dim text-xs">Loading...</p>
    </div>
  );
}

function FrozenPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center bg-void">
      <p className="font-mono text-text-dim text-xs uppercase tracking-widest">
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
    <div className="flex h-screen items-center justify-center bg-void">
      <div className="max-w-md px-6 text-center">
        <p className="mb-4 font-bold font-mono text-5xl text-glow-primary text-primary">
          {isNotFound ? "404" : "ERR"}
        </p>
        <p className="mb-6 font-sans text-sm text-text-muted">
          {isNotFound
            ? "This page doesn't exist or the project has moved."
            : "Something unexpected happened."}
        </p>
        <button
          type="button"
          onClick={() => navigate("/")}
          className="rounded bg-primary/10 px-5 py-2 font-sans text-primary text-sm transition-colors hover:bg-primary/20"
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
        element: (
          <SuspenseWrap>
            <WizardLayout />
          </SuspenseWrap>
        ),
        children: [
          {
            path: "describe",
            element: (
              <SuspenseWrap>
                <Describe />
              </SuspenseWrap>
            ),
          },
          {
            path: "describe/:id",
            element: (
              <SuspenseWrap>
                <Describe />
              </SuspenseWrap>
            ),
          },
          {
            path: "plan/:id",
            element: (
              <SuspenseWrap>
                <Plan />
              </SuspenseWrap>
            ),
          },
          {
            path: "atomize/:id",
            element: (
              <SuspenseWrap>
                <Atomize />
              </SuspenseWrap>
            ),
          },
          {
            path: "configure/:id",
            element: (
              <SuspenseWrap>
                <Configure />
              </SuspenseWrap>
            ),
          },
          {
            path: "launch/:id",
            element: (
              <SuspenseWrap>
                <Launch />
              </SuspenseWrap>
            ),
          },
        ],
      },
      {
        path: "monitor/:id",
        element: (
          <SuspenseWrap>
            <Monitor />
          </SuspenseWrap>
        ),
      },
      { path: "connections", Component: FrozenPlaceholder },
      { path: "plugins", Component: FrozenPlaceholder },
    ],
  },
]);

const rootElement = document.getElementById("root");

if (rootElement === null) {
  throw new Error("Root element not found");
}

createRoot(rootElement).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
