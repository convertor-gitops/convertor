import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatButton } from '@angular/material/button';
import { UiIconComponent, UiIconName } from '../ui-icon/ui-icon.component';

export type UiButtonVariant = 'filled' | 'tonal' | 'text' | 'outlined';
export type UiControlSize = 'inherit' | 'small' | 'medium' | 'large';
@Component({
  selector: 'app-ui-button',
  imports: [MatButton, UiIconComponent],
  templateUrl: './ui-button.component.html',
  styleUrl: './ui-button.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: 'ui-kit',
    '[attr.data-size]': 'size()',
    '[class.is-danger]': "tone() === 'danger'",
  },
})
export class UiButtonComponent {
  readonly variant = input<UiButtonVariant>('filled');
  readonly tone = input<'default' | 'danger'>('default');
  readonly size = input<UiControlSize>('inherit');
  readonly icon = input<UiIconName>();
  readonly iconEnd = input<UiIconName>();
  readonly loading = input(false);
  readonly disabled = input(false);
  readonly type = input<'button' | 'submit' | 'reset'>('button');
  readonly pressed = output<void>();
}
