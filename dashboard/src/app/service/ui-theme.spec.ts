import { TestBed } from '@angular/core/testing';
import { beforeEach, describe, expect, it } from 'vitest';
import { UiThemeService } from './ui-theme';
import {
  readableOn,
  readableText,
  relativeLuminance,
  restoreUiPreferences,
  UI_DEFAULTS,
  UI_PALETTES,
} from '../common/model/ui-theme';

describe('UI palette registry and preferences', () => {
  beforeEach(() => {
    localStorage.clear();
  });
  it('contains all 32 official palettes with valid colors and explicit modes', () => {
    expect(UI_PALETTES).toHaveLength(32);
    expect(new Set(UI_PALETTES.map((p) => p.id)).size).toBe(32);
    expect(UI_PALETTES.filter((p) => p.mode === 'light')).toHaveLength(9);
    for (const palette of UI_PALETTES) {
      for (const value of Object.values(palette.colors))
        expect(value).toMatch(/^#[\da-f]{6}([\da-f]{2})?$/i);
    }
  });

  it('recovers malformed and cross-mode preferences without dropping valid radius settings', () => {
    expect(restoreUiPreferences(null)).toEqual(UI_DEFAULTS);
    expect(
      restoreUiPreferences({ dark: 'github', light: 'dracula', controlRadius: 7, cardRadius: 16 }),
    ).toEqual({ ...UI_DEFAULTS, cardRadius: 16 });
  });

  it('keeps an independent palette for each mode and preserves geometry', () => {
    const service = TestBed.inject(UiThemeService);
    service.selectPalette('dracula');
    service.update({ cardRadius: 16, density: 'compact' });
    service.update({ mode: 'light' });
    expect(service.palette().id).toBe('github');
    service.selectPalette('lighter');
    service.selectPalette('dracula'); // invalid in light mode
    expect(service.palette().id).toBe('lighter');
    service.update({ mode: 'dark' });
    expect(service.palette().id).toBe('dracula');
    expect(service.preferences().cardRadius).toBe(16);
    expect(service.preferences().density).toBe('compact');
  });

  it('restores saved settings and applies overlay-inheritable tokens', () => {
    localStorage.setItem(
      'convertor.ui.preferences.v1',
      JSON.stringify({ ...UI_DEFAULTS, dark: 'oceanic', controlRadius: 12 }),
    );
    const service = TestBed.inject(UiThemeService);
    TestBed.tick();
    expect(service.palette().id).toBe('oceanic');
    expect(document.documentElement.dataset['uiTheme']).toBe('oceanic');
    expect(document.documentElement.style.getPropertyValue('--ui-radius-control')).toBe('12px');
    expect(document.documentElement.style.getPropertyValue('--ui-bg')).toBe('#263238');
  });

  it('maintains readable accent-button and secondary-label contrast across all palettes', () => {
    const contrast = (a: string, b: string) =>
      (Math.max(relativeLuminance(a), relativeLuminance(b)) + 0.05) /
      (Math.min(relativeLuminance(a), relativeLuminance(b)) + 0.05);
    for (const { colors } of UI_PALETTES) {
      expect(contrast(readableOn(colors.accent), colors.accent)).toBeGreaterThanOrEqual(4.5);
      expect(
        contrast(readableText(colors.muted, colors.surface), colors.surface),
      ).toBeGreaterThanOrEqual(4.5);
    }
  });
});
