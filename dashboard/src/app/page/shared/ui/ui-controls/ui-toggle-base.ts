import { Directive, input, model, signal } from '@angular/core';
import { ControlValueAccessor } from '@angular/forms';

@Directive()
export abstract class UIToggleBase implements ControlValueAccessor {
  readonly checked = model(false);
  readonly disabled = input(false);
  protected readonly formDisabled = signal(false);
  protected onChange = (_value: boolean) => {};
  protected onTouched = () => {};
  writeValue(value: boolean | null): void {
    this.checked.set(!!value);
  }
  registerOnChange(fn: (value: boolean) => void): void {
    this.onChange = fn;
  }
  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }
  setDisabledState(value: boolean): void {
    this.formDisabled.set(value);
  }
  setChecked(value: boolean): void {
    if (this.disabled() || this.formDisabled()) return;
    this.checked.set(value);
    this.onChange(value);
    this.onTouched();
  }
}
