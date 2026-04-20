import type { FitAddon } from "@xterm/addon-fit";
import { type RefObject, useEffect } from "react";

type UseTerminalFitArgs = {
  containerRef: RefObject<HTMLElement | null>;
  fitAddonRef: RefObject<FitAddon | null>;
};

function runFit(fitAddonRef: RefObject<FitAddon | null>) {
  fitAddonRef.current?.fit();
}

export function useTerminalFit({ containerRef, fitAddonRef }: UseTerminalFitArgs) {
  useEffect(() => {
    const containerElement = containerRef.current;
    if (!containerElement) {
      return;
    }

    let animationFrameId = 0;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    const scheduleFit = () => {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = requestAnimationFrame(() => runFit(fitAddonRef));
    };

    const scheduleFitBurst = () => {
      scheduleFit();
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      timeoutId = setTimeout(scheduleFit, 120);
    };

    const resizeObserver = new ResizeObserver(scheduleFit);
    resizeObserver.observe(containerElement);

    const mutationObserver = new MutationObserver(scheduleFitBurst);
    mutationObserver.observe(containerElement, {
      attributes: true,
      attributeFilter: ["class", "style", "hidden"],
    });

    const parentElement = containerElement.parentElement;
    if (parentElement) {
      mutationObserver.observe(parentElement, {
        attributes: true,
        attributeFilter: ["class", "style", "hidden", "data-state"],
      });
    }

    const handleResize = () => scheduleFitBurst();
    const handleVisibility = () => {
      if (document.visibilityState === "visible") {
        scheduleFitBurst();
      }
    };

    window.addEventListener("resize", handleResize);
    window.addEventListener("focus", handleResize);
    document.addEventListener("visibilitychange", handleVisibility);

    scheduleFitBurst();

    return () => {
      resizeObserver.disconnect();
      mutationObserver.disconnect();
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("focus", handleResize);
      document.removeEventListener("visibilitychange", handleVisibility);
      cancelAnimationFrame(animationFrameId);
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [containerRef, fitAddonRef]);
}
