import { effect, viewChild } from '@angular/core';
import { ChangeDetectionStrategy, Component, forwardRef, input } from '@angular/core';
import { NG_VALUE_ACCESSOR } from '@angular/forms';
import { MatInput } from '@angular/material/input';
import { UiValueAccessor } from '../ui-value-accessor';
import { UiIconComponent, UiIconName } from '../ui-icon/ui-icon.component';
let nextFieldId = 0;
@Component({
  selector: 'app-ui-text-field',
  imports: [MatInput, UiIconComponent],
  templateUrl: './ui-text-field.component.html',
  styleUrl: './ui-text-field.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      useExisting: forwardRef(() => UiTextFieldComponent),
      multi: true,
    },
  ],
  host: { class: 'ui-kit' },
})
export class UiTextFieldComponent extends UiValueAccessor<string | number | null> {
  readonly label = input.required<string>();
  readonly placeholder = input('');
  readonly hint = input('');
  readonly error = input('');
  private readonly materialControl = viewChild(MatInput);
  private readonly syncError = effect(() => {
    const control = this.materialControl();
    if (control) {
      control.errorState = !!this.error();
      control.stateChanges.next();
    }
  });
  readonly icon = input<UiIconName>();
  readonly type = input<'text' | 'search' | 'url' | 'password' | 'number'>('text');
  readonly multiline = input(false);
  readonly rows = input(3);
  readonly min = input<number | null>(null);
  readonly max = input<number | null>(null);
  readonly readonly = input(false);
  readonly required = input(false);
  protected readonly id = 'ui-field-' + nextFieldId++;
  protected readInput(event: Event): void {
    const field = event.target as HTMLInputElement;
    this.updateValue(
      this.type() === 'number' ? (field.value === '' ? null : field.valueAsNumber) : field.value,
    );
  }
}
