type RetryButtonProps = {
  variant: "inline" | "banner";
  onRetry: () => void;
};

const RETRY_VARIANTS: Record<RetryButtonProps["variant"], string> = {
  inline:
    "px-3 py-1.5 rounded bg-elevated text-text-muted text-xs font-sans hover:text-text transition-colors",
  banner:
    "px-3 py-1 rounded bg-surface border border-border text-text-muted text-xs font-sans hover:text-text transition-colors",
};

export function RetryButton({ variant, onRetry }: RetryButtonProps) {
  return (
    <button type="button" onClick={onRetry} className={RETRY_VARIANTS[variant]}>
      Retry
    </button>
  );
}
