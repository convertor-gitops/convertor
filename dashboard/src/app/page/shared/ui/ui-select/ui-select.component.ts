import { effect, viewChild } from '@angular/core';
import { NgTemplateOutlet } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  contentChildren,
  forwardRef,
  input,
} from '@angular/core';
import { NG_VALUE_ACCESSOR } from '@angular/forms';
import { MatSelect, MatSelectTrigger } from '@angular/material/select';
import { MatOption, MatOptgroup } from '@angular/material/core';
import { UiValueAccessor } from '../ui-value-accessor';
import { UiOptionComponent, UiOptionValue } from '../ui-option/ui-option.component';
export type UiSelectValue = UiOptionValue | UiOptionValue[] | null;
let nextSelectId = 0;
@Component({
  selector: 'app-ui-select',
  imports: [MatSelect, MatSelectTrigger, MatOption, MatOptgroup, NgTemplateOutlet],
  templateUrl: './ui-select.component.html',
  styleUrl: './ui-select.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => UiSelectComponent), multi: true },
  ],
  host: { class: 'ui-kit' },
})
export class UiSelectComponent extends UiValueAccessor<UiSelectValue> {
  readonly resetAfterSelection = input(false);
  select(value: UiSelectValue): void {
    this.updateValue(value);
    if (this.resetAfterSelection()) {
      this.writeValue(null);
      const control = this.materialControl();
      if (control) control.value = null;
    }
  }
  readonly label = input.required<string>();
  readonly placeholder = input('请选择');
  readonly multiple = input(false);
  readonly hint = input('');
  readonly error = input('');
  private readonly materialControl = viewChild(MatSelect);
  private readonly syncError = effect(() => {
    const control = this.materialControl();
    if (control) {
      control.errorState = !!this.error();
      control.stateChanges.next();
    }
  });
  readonly options = contentChildren(UiOptionComponent, { descendants: true });
  protected readonly id = 'ui-select-' + nextSelectId++;
  readonly groups = computed(() => {
    const groups = new Map<string, UiOptionComponent[]>();
    for (const option of this.options()) {
      const name = option.group();
      groups.set(name, [...(groups.get(name) ?? []), option]);
    }
    return [...groups].map(([name, options]) => ({ name, options }));
  });
  readonly selectedLabel = computed(() => {
    const values = Array.isArray(this.current())
      ? (this.current() as UiOptionValue[])
      : [this.current()];
    return this.options()
      .filter((option) => values.includes(option.value()))
      .map((option) => option.label())
      .join('、');
  });
}
