// The 24 hour "come back with the address alone" sign in keeps the account's refresh token on this device. It is never stored as plain text: it is sealed with an AES-GCM
// key that the browser holds and that cannot be read out (non extractable, kept in IndexedDB), so a copy of the page's storage (a backup, a synced profile, a dumped file)
// is useless on its own, and it is bound to a fingerprint of the device. Forgetting the key makes the stored text unreadable at once.
const DB = 'streams-resume';
const STORE = 'k';

function open(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function readKey(): Promise<CryptoKey | null> {
  const db = await open();
  return new Promise((resolve, reject) => {
    const get = db.transaction(STORE).objectStore(STORE).get('key');
    get.onsuccess = () => resolve((get.result as CryptoKey) || null);
    get.onerror = () => reject(get.error);
  });
}

async function writeKey(key: CryptoKey | null): Promise<void> {
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, 'readwrite');
    if (key) tx.objectStore(STORE).put(key, 'key'); else tx.objectStore(STORE).delete('key');
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

const b64 = (buf: ArrayBuffer | Uint8Array): string => btoa(String.fromCharCode(...new Uint8Array(buf as ArrayBuffer)));
const unb64 = (s: string) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));

/** Seals text with a fresh device key and returns `iv.cipher` (both base64). */
export async function resumeSeal(plain: string): Promise<string> {
  const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt']);
  await writeKey(key);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const cipher = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, new TextEncoder().encode(plain));
  return b64(iv) + '.' + b64(cipher);
}

/** Opens what `resumeSeal` made; an empty string when the key is gone or the text was changed. */
export async function resumeOpen(sealed: string): Promise<string> {
  try {
    const key = await readKey();
    if (!key) return '';
    const [iv = '', cipher = ''] = sealed.split('.');
    const plain = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: unb64(iv) }, key, unb64(cipher));
    return new TextDecoder().decode(plain);
  } catch {
    return '';
  }
}

/** Deletes the device key: everything sealed before becomes unreadable. */
export async function resumeForget(): Promise<void> {
  try { await writeKey(null); } catch { /* nothing to forget */ }
}

/** A short hash of what identifies this browser on this machine (not secret, only compared with the one stored at sign out). */
export async function resumeFingerprint(): Promise<string> {
  // only things that stay the same on one machine (a window moved to another screen or a zoom change must not look like another device)
  const text = [navigator.userAgent, navigator.platform, Intl.DateTimeFormat().resolvedOptions().timeZone, navigator.hardwareConcurrency || 0].join('|');
  return b64(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text))).slice(0, 22);
}
