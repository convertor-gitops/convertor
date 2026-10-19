import { NgTemplateOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatButton } from '@angular/material/button';
@Component({
  selector: 'app-ui-info-row',
  templateUrl: './ui-info-row.component.html',
  styleUrl: './ui-info-row.component.scss',
  imports: [MatButton, NgTemplateOutlet],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', '[class.actionable]': 'actionable()' },
})
export class UIInfoRowComponent {
  readonly actionable = input(false);
  readonly activated = output<void>();
}
