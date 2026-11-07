export function isMacDesktop(): boolean {
  if (typeof navigator === "undefined") {
    return false;
  }
  return /Macintosh|Mac OS X/.test(navigator.userAgent);
}
