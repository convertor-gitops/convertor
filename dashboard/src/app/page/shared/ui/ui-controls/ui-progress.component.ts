import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { MatProgressBar } from '@angular/material/progress-bar';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
@Component({
  selector: 'app-ui-progress',
  templateUrl: './ui-progress.component.html',
  styleUrl: './ui-progress.component.scss',
  imports: [MatProgressBar, MatProgressSpinner],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit' },
})
export class UIProgressComponent {
  readonly kind = input<'spinner' | 'bar'>('spinner');
  readonly indeterminate = input(true);
  readonly value = input(0);
  readonly label = input('加载中');
  readonly diameter = input(24);
}
