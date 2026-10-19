import { ChangeDetectionStrategy, Component, signal } from '@angular/core';
import { UIPanelComponent } from '../../shared/ui/ui-panel/ui-panel.component';
import { UINodeCardComponent } from '../../shared/ui/ui-node-card/ui-node-card.component';
import { UIButtonComponent } from '../../shared/ui/ui-button/ui-button.component';
@Component({
  selector: 'app-panel-showcase',
  imports: [UIPanelComponent, UINodeCardComponent, UIButtonComponent],
  templateUrl: './panel-showcase.component.html',
  styleUrl: './panel-showcase.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelShowcaseComponent {
  readonly levels = [1, 2, 3] as const;
  readonly selected = signal(true);
  readonly elevated = signal(true);
}
