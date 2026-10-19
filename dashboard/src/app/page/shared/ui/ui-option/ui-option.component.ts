import { ChangeDetectionStrategy, Component, input, TemplateRef, viewChild } from '@angular/core';
import { UIIconComponent, UIIconName } from '../ui-icon/ui-icon.component';
export type UIOptionValue = string | number;
@Component({
  selector: 'app-ui-option',
  imports: [UIIconComponent],
  templateUrl: './ui-option.component.html',
  styleUrl: './ui-option.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UIOptionComponent {
  readonly value = input.required<UIOptionValue>();
  readonly label = input.required<string>();
  readonly description = input('');
  readonly suffix = input('');
  readonly group = input('');
  readonly icon = input<UIIconName>();
  readonly swatch = input('');
  readonly disabled = input(false);
  readonly content = viewChild.required<TemplateRef<unknown>>('content');
}
