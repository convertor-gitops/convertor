import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatIconButton } from '@angular/material/button';
import { MatTooltip } from '@angular/material/tooltip';
import { UIIconComponent, UIIconName } from '../ui-icon/ui-icon.component';
import { UIButtonVariant, UIControlSize } from '../ui-button/ui-button.component';
@Component({
  selector: 'app-ui-icon-button',
  imports: [MatIconButton, MatTooltip, UIIconComponent],
  templateUrl: './ui-icon-button.component.html',
  styleUrl: './ui-icon-button.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', '[attr.data-size]': 'size()' },
})
export class UIIconButtonComponent {
  readonly icon = input<UIIconName>('plus');
  readonly label = input.required<string>();
  readonly variant = input<UIButtonVariant>('text');
  readonly tone = input<'default' | 'danger'>('default');
  readonly size = input<UIControlSize>('inherit');
  readonly disabled = input(false);
  readonly controls = input<string | null>(null);
  readonly expanded = input<boolean | null>(null);
  readonly pressed = output<void>();
}
