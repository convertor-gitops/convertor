import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { UiCardComponent } from '../ui-card/ui-card.component';
import { UiIconComponent } from '../ui-icon/ui-icon.component';
import { UiIconButtonComponent } from '../ui-icon-button/ui-icon-button.component';
export type UiNodeStatus = 'ready' | 'loading' | 'error' | 'disabled';
@Component({
  selector: 'app-ui-node-card',
  imports: [UiIconComponent, UiIconButtonComponent, UiCardComponent],
  templateUrl: './ui-node-card.component.html',
  styleUrl: './ui-node-card.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: 'ui-kit',
    '[class.selected]': 'selected()',
    '[class.unavailable]': "status() === 'disabled'",
    '[class.has-error]': "status() === 'error'",
  },
})
export class UiNodeCardComponent {
  readonly name = input.required<string>();
  readonly region = input('HK');
  readonly protocol = input('Trojan');
  readonly source = input('');
  readonly tags = input<string[]>([]);
  readonly selected = input(false);
  readonly showActions = input(true);
  readonly status = input<UiNodeStatus>('ready');
  readonly statusText = input('');
  readonly selectedChange = output<boolean>();
  readonly detailsRequested = output<void>();
  readonly actionRequested = output<void>();
}
