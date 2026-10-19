import { readOverlaySize, writeOverlaySize } from './overlay-size-storage';

describe('overlay size storage', () => {
  it('round-trips a valid size', () => {
    const storage = new MapStorage();
    writeOverlaySize(storage, 'editor', 679.7);
    expect(readOverlaySize(storage, 'editor')).toBe(680);
  });

  it('ignores invalid data and unavailable storage', () => {
    const storage = new MapStorage();
    storage.setItem('convertor:ui-overlay:editor', 'invalid');
    expect(readOverlaySize(storage, 'editor')).toBeUndefined();
    expect(readOverlaySize(undefined, 'editor')).toBeUndefined();
  });
});

class MapStorage implements Storage {
  private readonly values = new Map<string, string>();
  get length(): number {
    return this.values.size;
  }
  clear(): void {
    this.values.clear();
  }
  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }
  key(index: number): string | null {
    return [...this.values.keys()][index] ?? null;
  }
  removeItem(key: string): void {
    this.values.delete(key);
  }
  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}
