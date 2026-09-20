import { ChangeDetectionStrategy, Component, input, model, output } from '@angular/core';
import { UiPanelComponent } from '../ui-panel/ui-panel.component';
import { UiIconComponent } from '../ui-icon/ui-icon.component';
import { MatExpansionModule } from '@angular/material/expansion';
@Component({
  selector: 'app-ui-collapse',
  imports: [UiPanelComponent, UiIconComponent, MatExpansionModule],
  templateUrl: './ui-collapse.component.html',
  styleUrl: './ui-collapse.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UiCollapseComponent {
  readonly heading = input.required<string>();
  readonly description = input('');
  readonly elevation = input<1 | 2 | 3>(1);
  readonly disabled = input(false);
  readonly expanded = model(false);
  readonly opened = output<void>();
  readonly closed = output<void>();
}
