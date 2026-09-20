import { ChangeDetectionStrategy, Component, signal } from '@angular/core';
import { UiPanelComponent } from '../../shared/ui/ui-panel/ui-panel.component';
import { UiNodeCardComponent } from '../../shared/ui/ui-node-card/ui-node-card.component';
import { UiButtonComponent } from '../../shared/ui/ui-button/ui-button.component';
@Component({
  selector: 'app-panel-showcase',
  imports: [UiPanelComponent, UiNodeCardComponent, UiButtonComponent],
  templateUrl: './panel-showcase.component.html',
  styleUrl: './panel-showcase.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelShowcaseComponent {
  readonly levels = [1, 2, 3] as const;
  readonly selected = signal(true);
  readonly elevated = signal(true);
}
