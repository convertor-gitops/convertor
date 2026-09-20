import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { TestbedHarnessEnvironment } from '@angular/cdk/testing/testbed';
import { MatSelectHarness } from '@angular/material/select/testing';
import { beforeEach, describe, expect, it } from 'vitest';
import { UiSelectComponent, UiSelectValue } from './ui-select.component';
import { UiOptionComponent } from '../ui-option/ui-option.component';

@Component({
  imports: [ReactiveFormsModule, UiSelectComponent, UiOptionComponent],
  template:
    '<app-ui-select label="地区" [formControl]="control" [multiple]="multiple" [resetAfterSelection]="reset"><app-ui-option value="hk" label="香港" description="主力订阅" group="亚洲" suffix="HK" /><app-ui-option value="jp" label="日本" description="备用订阅" group="亚洲" /><app-ui-option value="de" label="德国" [disabled]="true" group="欧洲" /></app-ui-select>',
})
class SelectHost {
  control = new FormControl<UiSelectValue>('hk');
  multiple = false;
  reset = false;
}

describe('UiSelectComponent Angular Forms and real Material options', () => {
  beforeEach(() => TestBed.configureTestingModule({ imports: [SelectHost] }));
  async function setup(multiple = false) {
    const fixture = TestBed.createComponent(SelectHost);
    fixture.componentInstance.multiple = multiple;
    if (multiple) fixture.componentInstance.control.setValue(['hk']);
    fixture.detectChanges();
    await fixture.whenStable();
    const harness = await TestbedHarnessEnvironment.loader(fixture).getHarness(MatSelectHarness);
    return { fixture, harness, control: fixture.componentInstance.control };
  }

  it('registers projected option definitions with Material and uses only labels in the trigger', async () => {
    const { harness } = await setup();
    expect(await harness.getValueText()).toBe('香港');
    await harness.open();
    expect((await harness.getOptions()).length).toBe(3);
    expect(await (await harness.getOptions())[2].isDisabled()).toBe(true);
    await harness.clickOptions({ text: /日本/ });
    expect(await harness.getValueText()).toBe('日本');
  });

  it('writes user selections back to the parent form and marks it touched', async () => {
    const { harness, control } = await setup();
    await harness.open();
    await harness.clickOptions({ text: /日本/ });
    expect(control.value).toBe('jp');
    expect(control.touched).toBe(true);
  });

  it('updates the trigger on programmatic writes without emitting another form change', async () => {
    const { fixture, harness, control } = await setup();
    let emissions = 0;
    control.valueChanges.subscribe(() => emissions++);
    control.setValue('jp');
    fixture.detectChanges();
    await fixture.whenStable();
    expect(await harness.getValueText()).toBe('日本');
    expect(emissions).toBe(1);
  });

  it('supports array values and preserves selection when opening a multiple select', async () => {
    const { harness, control } = await setup(true);
    await harness.open();
    await harness.clickOptions({ text: /日本/ });
    expect(control.value).toEqual(['hk', 'jp']);
    await harness.close();
    expect(await harness.getValueText()).toBe('香港、日本');
  });

  it('reflects a disabled reactive form control', async () => {
    const { fixture, harness, control } = await setup();
    control.disable();
    fixture.detectChanges();
    await fixture.whenStable();
    expect(await harness.isDisabled()).toBe(true);
  });
  it('allows repeating an add action while clearing the visible selection', async () => {
    const { fixture, harness, control } = await setup();
    fixture.componentInstance.reset = true;
    fixture.detectChanges();
    const values: UiSelectValue[] = [];
    control.valueChanges.subscribe((value) => values.push(value));
    await harness.open();
    await harness.clickOptions({ text: /日本/ });
    expect(await harness.getValueText()).toBe('请选择');
    await harness.open();
    await harness.clickOptions({ text: /日本/ });
    expect(values).toEqual(['jp', 'jp']);
  });
});
