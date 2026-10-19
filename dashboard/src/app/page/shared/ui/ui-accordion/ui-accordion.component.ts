import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import type { UICollapseComponent } from '../ui-collapse/ui-collapse.component';

@Component({
  selector: 'app-ui-accordion',
  template: '<ng-content />',
  styles: ':host { display: flex; flex-direction: column; gap: 8px; min-width: 0; }',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UIAccordionComponent {
  readonly multi = input(false);
  private active: UICollapseComponent | null = null;
  opened(item: UICollapseComponent): void {
    if (!this.multi() && this.active && this.active !== item) this.active.expanded.set(false);
    this.active = item;
  }
}
