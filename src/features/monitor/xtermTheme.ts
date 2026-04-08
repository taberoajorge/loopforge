import type { ITheme } from "@xterm/xterm";

function getTerminalToken(name: string): string {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(`--lf-terminal-${name}`)
    .trim();
}

export function buildXtermTheme(): ITheme {
  return {
    background: getTerminalToken("bg"),
    foreground: getTerminalToken("fg"),
    cursor: getTerminalToken("cursor"),
    cursorAccent: getTerminalToken("cursor-accent"),
    selectionBackground: getTerminalToken("selection"),
    black: getTerminalToken("black"),
    brightBlack: getTerminalToken("bright-black"),
    red: getTerminalToken("red"),
    brightRed: getTerminalToken("bright-red"),
    green: getTerminalToken("green"),
    brightGreen: getTerminalToken("bright-green"),
    yellow: getTerminalToken("yellow"),
    brightYellow: getTerminalToken("bright-yellow"),
    blue: getTerminalToken("blue"),
    brightBlue: getTerminalToken("bright-blue"),
    magenta: getTerminalToken("magenta"),
    brightMagenta: getTerminalToken("bright-magenta"),
    cyan: getTerminalToken("cyan"),
    brightCyan: getTerminalToken("bright-cyan"),
    white: getTerminalToken("white"),
    brightWhite: getTerminalToken("bright-white"),
  };
}
