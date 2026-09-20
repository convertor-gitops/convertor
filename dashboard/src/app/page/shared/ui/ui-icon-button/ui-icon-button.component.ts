import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatIconButton } from '@angular/material/button';
import { MatTooltip } from '@angular/material/tooltip';
import { UiIconComponent, UiIconName } from '../ui-icon/ui-icon.component';
import { UiButtonVariant, UiControlSize } from '../ui-button/ui-button.component';
@Component({
  selector: 'app-ui-icon-button',
  imports: [MatIconButton, MatTooltip, UiIconComponent],
  templateUrl: './ui-icon-button.component.html',
  styleUrl: './ui-icon-button.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', '[attr.data-size]': 'size()' },
})
export class UiIconButtonComponent {
  readonly icon = input<UiIconName>('plus');
  readonly label = input.required<string>();
  readonly variant = input<UiButtonVariant>('text');
  readonly size = input<UiControlSize>('inherit');
  readonly disabled = input(false);
  readonly pressed = output<void>();
}
