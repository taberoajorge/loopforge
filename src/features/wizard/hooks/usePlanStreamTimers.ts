import { useEffect, useRef, useState } from "react";

interface UsePlanStreamTimersArgs {
  isRunning: boolean;
  eventCount: number;
}

export function usePlanStreamTimers(args: UsePlanStreamTimersArgs) {
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const [secondsSinceLastEvent, setSecondsSinceLastEvent] = useState(0);
  const startRef = useRef(Date.now());
  const lastEventTimeRef = useRef(Date.now());

  useEffect(() => {
    if (!args.isRunning) {
      setElapsedSeconds(0);
      startRef.current = Date.now();
      return;
    }
    startRef.current = Date.now();
    const timer = setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - startRef.current) / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, [args.isRunning]);

  useEffect(() => {
    lastEventTimeRef.current = Date.now();
  }, []);

  useEffect(() => {
    if (!args.isRunning) {
      setSecondsSinceLastEvent(0);
      return;
    }
    const timer = setInterval(() => {
      setSecondsSinceLastEvent(Math.floor((Date.now() - lastEventTimeRef.current) / 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, [args.isRunning]);

  return { elapsedSeconds, secondsSinceLastEvent };
}
