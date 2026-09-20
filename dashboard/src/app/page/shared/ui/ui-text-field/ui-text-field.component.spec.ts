import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { FormControl, ReactiveFormsModule } from '@angular/forms';
import { beforeEach, describe, expect, it } from 'vitest';
import { UiTextFieldComponent } from './ui-text-field.component';

@Component({
  imports: [ReactiveFormsModule, UiTextFieldComponent],
  template: '<app-ui-text-field label="来源" [formControl]="control" error="请检查名称" />',
})
class InputHost {
  control = new FormControl('主力订阅');
}

describe('UiTextFieldComponent form contract', () => {
  beforeEach(() => TestBed.configureTestingModule({ imports: [InputHost] }));
  it('propagates typing and blur, and links its accessible label and validation message', async () => {
    const fixture = TestBed.createComponent(InputHost);
    fixture.detectChanges();
    await fixture.whenStable();
    const input: HTMLInputElement = fixture.nativeElement.querySelector('input');
    expect(input.value).toBe('主力订阅');
    input.value = '备用订阅';
    input.dispatchEvent(new Event('input'));
    input.dispatchEvent(new Event('blur'));
    await fixture.whenStable();
    expect(fixture.componentInstance.control.value).toBe('备用订阅');
    expect(fixture.componentInstance.control.touched).toBe(true);
    expect(fixture.nativeElement.querySelector('label').htmlFor).toBe(input.id);
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(
      fixture.nativeElement.querySelector('#' + input.getAttribute('aria-describedby')).textContent,
    ).toContain('请检查名称');
    fixture.componentInstance.control.disable();
    fixture.detectChanges();
    expect(input.disabled).toBe(true);
  });
});

@Component({
  imports: [ReactiveFormsModule, UiTextFieldComponent],
  template: '<app-ui-text-field label="间隔" type="number" [min]="1" [formControl]="control" />',
})
class NumberHost {
  control = new FormControl<number | null>(300);
}
it('preserves numeric values and empty input for strategy editing', async () => {
  const fixture = TestBed.createComponent(NumberHost);
  fixture.detectChanges();
  await fixture.whenStable();
  const input: HTMLInputElement = fixture.nativeElement.querySelector('input');
  expect(input.min).toBe('1');
  input.value = '60';
  input.dispatchEvent(new Event('input'));
  expect(fixture.componentInstance.control.value).toBe(60);
  input.value = '';
  input.dispatchEvent(new Event('input'));
  expect(fixture.componentInstance.control.value).toBeNull();
});
