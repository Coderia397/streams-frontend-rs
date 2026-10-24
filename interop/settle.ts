// When a page has settled: the pictures on screen have arrived and the browser has a quiet moment. Video is the heaviest thing a page
// loads (a YouTube player is about 1 MB of script before any picture of the film) and the least needed to read the page, so players
// start after this: on a slow connection or a slow phone the page draws first, on a fast one the wait is a moment.

type Idle = (fn: () => void, opts: { timeout: number }) => number;

function onScreen(): HTMLImageElement[] {
  return Array.from(document.images).filter((img) => {
    if (img.complete) return false;
    const r = img.getBoundingClientRect();
    return r.width > 0 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth;
  });
}

function arrived(imgs: HTMLImageElement[]): Promise<unknown> {
  const each = imgs.map((img) => new Promise((done) => {
    img.addEventListener('load', done, { once: true });
    img.addEventListener('error', done, { once: true });
  }));
  // a picture that never answers does not hold a player for ever
  return Promise.race([Promise.all(each), new Promise((done) => setTimeout(done, 8000))]);
}

export function whenPageSettled(callback: () => void): void {
  const idle: Idle = (window as unknown as { requestIdleCallback?: Idle }).requestIdleCallback
    ?? ((fn) => window.setTimeout(fn, 200));
  // pictures that arrive while waiting can bring more into view (a row under a billboard): look again, a few times at most
  const look = (left: number) => {
    const pending = onScreen();
    if (pending.length === 0 || left === 0) idle(callback, { timeout: 2000 });
    else arrived(pending).then(() => look(left - 1));
  };
  // a player is usually asked for in the same update that puts the page's pictures in place: look after that update is drawn
  requestAnimationFrame(() => setTimeout(() => look(3), 0));
}
