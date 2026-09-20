import { ChangeDetectionStrategy, Component, input } from '@angular/core';

export const UI_ICONS = {
  plus: 'M12 5v14M5 12h14',
  close: 'm6 6 12 12M6 18 18 6',
  check: 'm5 12 4 4L19 6',
  chevron: 'm9 5 7 7-7 7',
  down: 'm6 9 6 6 6-6',
  search: 'm21 21-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0',
  more: 'M5 12h.01M12 12h.01M19 12h.01',
  grip: 'M8 6h.01M16 6h.01M8 12h.01M16 12h.01M8 18h.01M16 18h.01',
  refresh: 'M20 7v5h-5M4 17v-5h5M6 6a8 8 0 0 1 13 2M5 16a8 8 0 0 0 13 2',
  copy: 'M9 9h11v11H9zM15 5V3H3v12h2',
  trash: 'M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7',
  globe: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0M3 12h18M12 3c-5 5-5 13 0 18M12 3c5 5 5 13 0 18',
  layers: 'm12 3 10 5-10 5L2 8zM2 12l10 5 10-5M2 16l10 5 10-5',
  sliders: 'M4 4v6M4 14v6M12 4v10M12 18v2M20 4v2M20 10v10M1 10h6M9 18h6M17 6h6',
  moon: 'M20 15A9 9 0 0 1 9 4a9 9 0 1 0 11 11',
  sun: 'M12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1 1M18 18l1 1M5 19l1-1M18 6l1-1',
  monitor: 'M3 4h18v13H3zM8 21h8M12 17v4',
  server: 'M3 3h18v7H3zM3 14h18v7H3zM6 6h.01M6 17h.01M10 6h7M10 17h7',
  arrow: 'M4 12h16m-6-6 6 6-6 6',
  info: 'M12 8h.01M12 11v6M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0',
  code: 'm8 6-6 6 6 6m8-12 6 6-6 6M14 3l-4 18',
  clock: 'M12 7v5l3 2M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0',
  bolt: 'm13 2-9 12h7l-1 8 10-12h-7z',
  palette:
    'M12 3a9 9 0 1 0 0 18h2a2 2 0 0 0 0-4h-1a2 2 0 0 1 0-4h4a4 4 0 0 0 4-4c0-4-5-6-9-6M7 9h.01M10 6h.01M15 6h.01M6 14h.01',
} as const;
export type UiIconName = keyof typeof UI_ICONS;
@Component({
  selector: 'app-ui-icon',
  templateUrl: './ui-icon.component.html',
  styleUrl: './ui-icon.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { 'aria-hidden': 'true' },
})
export class UiIconComponent {
  readonly name = input<UiIconName>('plus');
  protected readonly paths = UI_ICONS;
}
