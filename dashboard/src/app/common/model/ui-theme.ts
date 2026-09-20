import palettes from './ui-palettes.json';

export type UiMode = 'dark' | 'light';
export type UiModePreference = UiMode | 'system';
export type UiDensity = 'ultra' | 'compact' | 'comfortable';
export type UiRadius = 4 | 8 | 12 | 16;
export interface UiPalette {
  id: string;
  name: string;
  mode: UiMode;
  colors: Record<
    | 'background'
    | 'foreground'
    | 'muted'
    | 'selection'
    | 'onSelection'
    | 'button'
    | 'surface'
    | 'disabled'
    | 'contrast'
    | 'active'
    | 'border'
    | 'highlight'
    | 'accent'
    | 'error'
    | 'green'
    | 'yellow'
    | 'blue'
    | 'purple',
    string
  >;
}
export interface UiPreferences {
  mode: UiModePreference;
  dark: string;
  light: string;
  density: UiDensity;
  controlRadius: UiRadius;
  cardRadius: UiRadius;
  overlayRadius: UiRadius;
  panelRadius: UiRadius;
}
export const UI_PALETTES = palettes as UiPalette[];
export const UI_DEFAULTS: UiPreferences = {
  mode: 'dark',
  dark: 'githubdark',
  light: 'github',
  density: 'ultra',
  controlRadius: 8,
  cardRadius: 12,
  overlayRadius: 16,
  panelRadius: 16,
};

/** Validate independently so one stale setting cannot invalidate the rest. */
export function restoreUiPreferences(value: unknown): UiPreferences {
  const raw = value && typeof value === 'object' ? (value as Record<string, unknown>) : {};
  const result = { ...UI_DEFAULTS };
  if (['dark', 'light', 'system'].includes(String(raw['mode'])))
    result.mode = raw['mode'] as UiModePreference;
  if (['ultra', 'compact', 'comfortable'].includes(String(raw['density'])))
    result.density = raw['density'] as UiDensity;
  for (const mode of ['dark', 'light'] as const) {
    if (UI_PALETTES.some((p) => p.id === raw[mode] && p.mode === mode))
      result[mode] = raw[mode] as string;
  }
  for (const key of ['controlRadius', 'cardRadius', 'overlayRadius', 'panelRadius'] as const) {
    if ([4, 8, 12, 16].includes(raw[key] as number)) result[key] = raw[key] as UiRadius;
  }
  return result;
}

export function relativeLuminance(hex: string): number {
  const channels = [1, 3, 5]
    .map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}
export function readableOn(hex: string): string {
  return relativeLuminance(hex) > 0.179 ? '#000000' : '#ffffff';
}
/** Preserve hue, adjusting lightness only when a small label lacks contrast. */
export function readableText(foreground: string, background: string, minimum = 4.5): string {
  const ratio = (a: string, b: string) => {
    const x = relativeLuminance(a),
      y = relativeLuminance(b);
    return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
  };
  if (ratio(foreground, background) >= minimum) return foreground;
  const target = relativeLuminance(background) < 0.179 ? 255 : 0;
  for (let step = 1; step <= 20; step++) {
    const adjusted =
      '#' +
      [1, 3, 5]
        .map((i) =>
          Math.round(
            parseInt(foreground.slice(i, i + 2), 16) * (1 - step / 20) + (target * step) / 20,
          )
            .toString(16)
            .padStart(2, '0'),
        )
        .join('');
    if (ratio(adjusted, background) >= minimum) return adjusted;
  }
  return target ? '#ffffff' : '#000000';
}
