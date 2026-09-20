import { computed, Directive, input, signal } from '@angular/core';
import { ControlValueAccessor } from '@angular/forms';

/** Shared Angular Forms contract. Programmatic writes never emit user changes. */
@Directive()
export abstract class UiValueAccessor<T> implements ControlValueAccessor {
  readonly disabled = input(false);
  protected readonly formDisabled = signal(false);
  readonly isDisabled = computed(() => this.disabled() || this.formDisabled());
  readonly current = signal<T | null>(null);
  protected onChange: (value: T | null) => void = () => {};
  protected onTouched: () => void = () => {};
  writeValue(value: T | null): void {
    this.current.set(value);
  }
  registerOnChange(fn: (value: T | null) => void): void {
    this.onChange = fn;
  }
  registerOnTouched(fn: () => void): void {
    this.onTouched = fn;
  }
  setDisabledState(value: boolean): void {
    this.formDisabled.set(value);
  }
  touch(): void {
    this.onTouched();
  }
  updateValue(value: T | null): void {
    if (this.isDisabled()) return;
    this.current.set(value);
    this.onChange(value);
  }
}
