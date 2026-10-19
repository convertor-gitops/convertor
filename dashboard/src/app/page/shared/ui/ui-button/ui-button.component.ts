import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatButton } from '@angular/material/button';
import { UIIconComponent, UIIconName } from '../ui-icon/ui-icon.component';

export type UIButtonVariant = 'filled' | 'tonal' | 'text' | 'outlined';
export type UIControlSize = 'inherit' | 'small' | 'medium' | 'large';
@Component({
  selector: 'app-ui-button',
  imports: [MatButton, UIIconComponent],
  templateUrl: './ui-button.component.html',
  styleUrl: './ui-button.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: 'ui-kit',
    '[attr.data-size]': 'size()',
    '[class.is-danger]': "tone() === 'danger'",
  },
})
export class UIButtonComponent {
  readonly variant = input<UIButtonVariant>('filled');
  readonly tone = input<'default' | 'danger'>('default');
  readonly size = input<UIControlSize>('inherit');
  readonly icon = input<UIIconName>();
  readonly iconEnd = input<UIIconName>();
  readonly loading = input(false);
  readonly disabled = input(false);
  readonly type = input<'button' | 'submit' | 'reset'>('button');
  readonly pressed = output<void>();
}
