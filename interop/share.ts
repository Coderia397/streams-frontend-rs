// Sharing (src/components/share): the system share sheet with the title's picture, and copying a link.
//
// A browser lets a page open the system share sheet only during a tap, for a few seconds. Fetching the picture inside the tap could
// outlast that, so the sheet's opening fetches it (sharePrepare) and the tap shares what is already there (shareSystem).

const prepared = new Map<string, File>();

/** Fetches the picture at `imageUrl` for a later share; any failure leaves the share without a picture. */
export function sharePrepare(imageUrl: string, fileName: string): void {
  if (!imageUrl || prepared.has(imageUrl)) return;
  // past the cache: the page's own <img> of the same poster is cached without the CORS header, and a fetch answered from it fails
  fetch(imageUrl, { mode: 'cors', cache: 'no-store' })
    .then((r) => (r.ok ? r.blob() : Promise.reject(new Error(String(r.status)))))
    .then((blob) => {
      prepared.set(imageUrl, new File([blob], fileName, { type: blob.type || 'image/jpeg' }));
    })
    .catch(() => {});
}

/**
 * Opens the system share sheet with the picture (when it arrived and the browser can share files) and the title and link, or with the
 * title and link alone. Resolves to 'shared', 'cancelled' (the person closed the sheet), 'failed' or 'unsupported' (no share sheet).
 */
export function shareSystem(title: string, url: string, imageUrl: string): Promise<string> {
  if (typeof navigator.share !== 'function') return Promise.resolve('unsupported');
  const file = prepared.get(imageUrl);
  const withPicture: ShareData | null = file && navigator.canShare?.({ files: [file] }) ? { files: [file], title, text: `${title} ${url}` } : null;
  return navigator.share(withPicture ?? { title, text: title, url }).then(
    () => 'shared',
    (e: unknown) => ((e as { name?: string })?.name === 'AbortError' ? 'cancelled' : 'failed'),
  );
}

/** Copies `text`; resolves to whether it was copied. The clipboard API first, then the older selection copy (some in-app browsers). */
export function copyText(text: string): Promise<boolean> {
  const fallback = (): boolean => {
    const area = document.createElement('textarea');
    area.value = text;
    area.setAttribute('readonly', '');
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    let ok = false;
    try {
      ok = document.execCommand('copy');
    } catch {
      ok = false;
    }
    area.remove();
    return ok;
  };
  if (!navigator.clipboard?.writeText) return Promise.resolve(fallback());
  return navigator.clipboard.writeText(text).then(() => true, () => fallback());
}
