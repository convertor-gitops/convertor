const STORAGE_PREFIX = 'convertor:ui-overlay:';

export function readOverlaySize(
  storage: Storage | undefined,
  key: string | undefined,
): number | undefined {
  if (!storage || !key) return undefined;

  try {
    const value = Number(storage.getItem(`${STORAGE_PREFIX}${key}`));
    return Number.isFinite(value) && value > 0 ? value : undefined;
  } catch {
    return undefined;
  }
}

export function writeOverlaySize(
  storage: Storage | undefined,
  key: string | undefined,
  value: number,
): void {
  if (!storage || !key || !Number.isFinite(value) || value <= 0) return;

  try {
    storage.setItem(`${STORAGE_PREFIX}${key}`, String(Math.round(value)));
  } catch {
    // Private browsing and storage policies can reject writes. Resizing must still work.
  }
}
