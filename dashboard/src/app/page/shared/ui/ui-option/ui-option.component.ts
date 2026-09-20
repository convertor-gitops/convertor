import { ChangeDetectionStrategy, Component, input, TemplateRef, viewChild } from '@angular/core';
import { UiIconComponent, UiIconName } from '../ui-icon/ui-icon.component';
export type UiOptionValue = string | number;
@Component({
  selector: 'app-ui-option',
  imports: [UiIconComponent],
  templateUrl: './ui-option.component.html',
  styleUrl: './ui-option.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UiOptionComponent {
  readonly value = input.required<UiOptionValue>();
  readonly label = input.required<string>();
  readonly description = input('');
  readonly suffix = input('');
  readonly group = input('');
  readonly icon = input<UiIconName>();
  readonly swatch = input('');
  readonly disabled = input(false);
  readonly content = viewChild.required<TemplateRef<unknown>>('content');
}
