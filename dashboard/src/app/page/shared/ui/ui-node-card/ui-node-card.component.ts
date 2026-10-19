import { NgTemplateOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { UICardComponent } from '../ui-card/ui-card.component';
import { UIIconComponent } from '../ui-icon/ui-icon.component';
import { UIIconButtonComponent } from '../ui-icon-button/ui-icon-button.component';
export type UINodeStatus = 'ready' | 'loading' | 'error' | 'disabled';
export type UINodeCardMode = 'selectable' | 'readonly';
@Component({
  selector: 'app-ui-node-card',
  imports: [NgTemplateOutlet, UIIconComponent, UIIconButtonComponent, UICardComponent],
  templateUrl: './ui-node-card.component.html',
  styleUrl: './ui-node-card.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    class: 'ui-kit',
    '[class.selected]': "mode() === 'selectable' && selected()",
    '[class.highlighted]': 'highlighted()',
    '[class.unavailable]': "status() === 'disabled'",
    '[class.has-error]': "status() === 'error'",
  },
})
export class UINodeCardComponent {
  readonly name = input.required<string>();
  readonly region = input('HK');
  readonly protocol = input('Trojan');
  readonly source = input('');
  readonly tags = input<string[]>([]);
  readonly selected = input(false);
  readonly highlighted = input(false);
  readonly mode = input<UINodeCardMode>('selectable');
  readonly showActions = input(true);
  readonly status = input<UINodeStatus>('ready');
  readonly statusText = input('');
  readonly selectedChange = output<boolean>();
  readonly detailsRequested = output<void>();
  readonly actionRequested = output<void>();
}
