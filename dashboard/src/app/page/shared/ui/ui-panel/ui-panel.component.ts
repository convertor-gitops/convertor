import { MatCard } from '@angular/material/card';
import { ChangeDetectionStrategy, Component, input } from '@angular/core';
@Component({
  selector: 'app-ui-panel',
  imports: [MatCard],
  templateUrl: './ui-panel.component.html',
  styleUrl: './ui-panel.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', '[attr.data-elevation]': 'elevation()' },
})
export class UiPanelComponent {
  readonly heading = input('');
  readonly description = input('');
  readonly elevation = input<1 | 2 | 3>(1);
}
