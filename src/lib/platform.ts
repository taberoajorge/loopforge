const APPLE_PLATFORM_REGEX = /Mac|iPhone|iPad|iPod/;

export function isApplePlatform() {
  if (typeof navigator === "undefined") {
    return false;
  }
  return APPLE_PLATFORM_REGEX.test(navigator.userAgent);
}
