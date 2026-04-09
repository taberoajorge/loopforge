import { render } from "@testing-library/react";
import {
  MemoryRouter,
  Route,
  Routes,
  type RouteObject,
} from "react-router";

function renderRoutes(routes: RouteObject[], parentKey: string = "route") {
  return routes.map((route, index) => {
    const routeKey = `${parentKey}-${route.path ?? "index"}-${index}`;
    const routeElement =
      route.element ??
      (route.Component ? <route.Component /> : undefined);
    if (route.index) {
      return <Route key={routeKey} index element={routeElement} />;
    }
    return (
      <Route key={routeKey} path={route.path} element={routeElement}>
        {route.children ? renderRoutes(route.children, routeKey) : null}
      </Route>
    );
  });
}

export function renderRoute(
  routes: RouteObject[],
  initialEntries: string[] = ["/"],
) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <Routes>{renderRoutes(routes)}</Routes>
    </MemoryRouter>,
  );
}
