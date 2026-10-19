import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  model,
  output,
} from '@angular/core';
import { UIPanelComponent } from '../ui-panel/ui-panel.component';
import { UIIconComponent } from '../ui-icon/ui-icon.component';
import { UIAccordionComponent } from '../ui-accordion/ui-accordion.component';
let nextCollapseId = 0;
@Component({
  selector: 'app-ui-collapse',
  imports: [UIPanelComponent, UIIconComponent],
  templateUrl: './ui-collapse.component.html',
  styleUrl: './ui-collapse.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UICollapseComponent {
  readonly heading = input.required<string>();
  readonly description = input('');
  readonly elevation = input<1 | 2 | 3>(1);
  readonly disabled = input(false);
  readonly expanded = model(false);
  readonly opened = output<void>();
  readonly closed = output<void>();
  protected readonly contentId = `ui-collapse-content-${nextCollapseId++}`;
  private readonly accordion = inject(UIAccordionComponent, { optional: true, skipSelf: true });
  private previousExpanded = this.expanded();
  private readonly expandedEffect = effect(() => {
    const expanded = this.expanded();
    if (expanded === this.previousExpanded) return;
    this.previousExpanded = expanded;
    if (expanded) {
      this.accordion?.opened(this);
      this.opened.emit();
    } else {
      this.closed.emit();
    }
  });
  toggle(): void {
    if (!this.disabled()) this.expanded.update((value) => !value);
  }
}
