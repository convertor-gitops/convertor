import { ChangeDetectionStrategy, Component, input, model } from '@angular/core';
export interface UIOrderedOption {
  label: string;
  value: string;
  disabled?: boolean;
}
@Component({
  selector: 'app-ui-ordered-options',
  templateUrl: './ui-ordered-options.component.html',
  styleUrl: './ui-ordered-options.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit' },
})
export class UIOrderedOptionsComponent {
  readonly values = model<string[]>([]);
  readonly choices = input.required<UIOrderedOption[]>();
  readonly label = input('排序选项');
  order(value: string): number {
    return this.values().indexOf(value) + 1;
  }
  toggle(choice: UIOrderedOption): void {
    if (choice.disabled) return;
    this.values.update((values) =>
      values.includes(choice.value)
        ? values.filter((value) => value !== choice.value)
        : [...values, choice.value],
    );
  }
}
