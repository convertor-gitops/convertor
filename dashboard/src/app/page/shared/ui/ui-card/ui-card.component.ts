import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { MatCard } from '@angular/material/card';
@Component({
  selector: 'app-ui-card',
  imports: [MatCard],
  templateUrl: './ui-card.component.html',
  styleUrl: './ui-card.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', '[class.subtle]': "appearance() === 'subtle'" },
})
export class UiCardComponent {
  readonly heading = input('');
  readonly description = input('');
  readonly appearance = input<'outlined' | 'subtle'>('outlined');
  readonly padded = input(true);
}
