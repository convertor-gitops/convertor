import { DOCUMENT } from '@angular/common';
import { computed, DestroyRef, effect, inject, Injectable, signal } from '@angular/core';
import {
  readableOn,
  readableText,
  restoreUiPreferences,
  UI_DEFAULTS,
  UI_PALETTES,
  UiPreferences,
} from '../common/model/ui-theme';

@Injectable({ providedIn: 'root' })
export class UiThemeService {
  private readonly document = inject(DOCUMENT);
  private readonly window = this.document.defaultView;
  private readonly media = this.window?.matchMedia?.('(prefers-color-scheme: dark)');
  private readonly systemDark = signal(this.media?.matches ?? true);
  readonly preferences = signal(this.read());
  readonly mode = computed(() =>
    this.preferences().mode === 'system'
      ? this.systemDark()
        ? 'dark'
        : 'light'
      : (this.preferences().mode as 'dark' | 'light'),
  );
  readonly palettes = computed(() => UI_PALETTES.filter((p) => p.mode === this.mode()));
  readonly palette = computed(
    () =>
      this.palettes().find((p) => p.id === this.preferences()[this.mode()]) ?? this.palettes()[0],
  );

  constructor() {
    const changed = (event: MediaQueryListEvent) => this.systemDark.set(event.matches);
    this.media?.addEventListener('change', changed);
    inject(DestroyRef).onDestroy(() => this.media?.removeEventListener('change', changed));
    effect(() => {
      try {
        this.window?.localStorage.setItem(
          'convertor.ui.preferences.v1',
          JSON.stringify(this.preferences()),
        );
      } catch {
        /* Storage can be unavailable. */
      }
      this.apply();
    });
  }
  update(patch: Partial<UiPreferences>): void {
    this.preferences.update((current) => restoreUiPreferences({ ...current, ...patch }));
  }
  selectPalette(id: string): void {
    if (this.palettes().some((p) => p.id === id)) this.update({ [this.mode()]: id });
  }
  reset(): void {
    this.preferences.set({ ...UI_DEFAULTS });
  }
  private read(): UiPreferences {
    try {
      return restoreUiPreferences(
        JSON.parse(this.window?.localStorage.getItem('convertor.ui.preferences.v1') ?? 'null'),
      );
    } catch {
      return { ...UI_DEFAULTS };
    }
  }
  private apply(): void {
    const root = this.document.documentElement;
    const p = this.palette(),
      c = p.colors,
      settings = this.preferences();
    root.dataset['uiTheme'] = p.id;
    root.dataset['uiMode'] = p.mode;
    root.dataset['uiDensity'] = settings.density;
    const values: Record<string, string> = {
      bg: c.background,
      surface: c.surface,
      'surface-low': c.contrast,
      'surface-high': c.button,
      text: readableText(c.foreground, c.surface),
      muted: readableText(c.muted, c.surface),
      accent: c.accent,
      'accent-text': readableText(c.accent, c.surface),
      'on-accent': readableOn(c.accent),
      selection: c.selection,
      'on-selection': readableText(c.onSelection, c.active),
      active: c.active,
      border: c.border,
      hover: c.highlight,
      error: readableText(c.error, c.surface),
      'on-error': readableOn(readableText(c.error, c.surface)),
      success: readableText(c.green, c.surface),
      warning: readableText(c.yellow, c.surface),
      blue: readableText(c.blue, c.surface),
      purple: readableText(c.purple, c.surface),
      disabled: c.disabled,
      'radius-control': settings.controlRadius + 'px',
      'radius-card': settings.cardRadius + 'px',
      'radius-overlay': settings.overlayRadius + 'px',
      'radius-panel': settings.panelRadius + 'px',
    };
    for (const [key, value] of Object.entries(values)) root.style.setProperty('--ui-' + key, value);
  }
}
