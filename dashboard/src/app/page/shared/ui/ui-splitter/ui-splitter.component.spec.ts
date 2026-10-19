import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { describe, expect, it, vi } from 'vitest';
import {
  constrainSplitterValue,
  UISplitterCompletion,
  UISplitterComponent,
} from './ui-splitter.component';

@Component({
  imports: [UISplitterComponent],
  template:
    '<app-ui-splitter [value]="value" [min]="100" [keyboardMin]="keyboardMin" [max]="300" [defaultValue]="180" [direction]="direction" (valueChange)="value = $event" (completed)="completions.push($event)" />',
})
class SplitterHost {
  value = 200;
  direction: 1 | -1 = 1;
  keyboardMin = 100;
  completions: UISplitterCompletion[] = [];
}

describe('UISplitterComponent', () => {
  it('constrains invalid and out-of-range values', () => {
    expect(constrainSplitterValue(40, 100, 300)).toBe(100);
    expect(constrainSplitterValue(400, 100, 300)).toBe(300);
    expect(constrainSplitterValue(Number.NaN, 100, 300)).toBe(100);
    expect(constrainSplitterValue(200, 300, 100)).toBe(300);
  });

  it('supports keyboard resize and reset', () => {
    const fixture = TestBed.createComponent(SplitterHost);
    fixture.detectChanges();
    const separator = fixture.nativeElement.querySelector('[role="separator"]') as HTMLElement;
    separator.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(208);
    expect(separator.getAttribute('aria-valuenow')).toBe('208');
    expect(fixture.componentInstance.completions.at(-1)?.reason).toBe('keyboard');
    separator.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(300);
    separator.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Home', ctrlKey: true, bubbles: true }),
    );
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(180);
    expect(fixture.componentInstance.completions.at(-1)).toEqual({ value: 180, reason: 'reset' });
  });

  it('reverses physical movement for end and bottom edges', () => {
    const fixture = TestBed.createComponent(SplitterHost);
    fixture.componentInstance.direction = -1;
    fixture.detectChanges();
    const separator = fixture.nativeElement.querySelector('[role="separator"]') as HTMLElement;
    separator.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(208);
  });

  it('uses a separate minimum for keyboard resizing', () => {
    const fixture = TestBed.createComponent(SplitterHost);
    fixture.componentInstance.keyboardMin = 180;
    fixture.detectChanges();
    const separator = fixture.nativeElement.querySelector('[role="separator"]') as HTMLElement;
    separator.dispatchEvent(new KeyboardEvent('keydown', { key: 'Home', bubbles: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(180);
    separator.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(180);
  });

  it('restores the starting value when Escape cancels a pointer drag', () => {
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    });
    vi.stubGlobal('cancelAnimationFrame', vi.fn());
    const fixture = TestBed.createComponent(SplitterHost);
    fixture.detectChanges();
    const separator = fixture.nativeElement.querySelector('[role="separator"]') as HTMLElement;
    Object.defineProperties(separator, {
      setPointerCapture: { value: vi.fn() },
      hasPointerCapture: { value: vi.fn(() => false) },
    });
    separator.dispatchEvent(
      new PointerEvent('pointerdown', { pointerId: 7, button: 0, clientX: 100, bubbles: true }),
    );
    separator.dispatchEvent(
      new PointerEvent('pointermove', { pointerId: 7, clientX: 150, bubbles: true }),
    );
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(250);
    separator.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.value).toBe(200);
    expect(fixture.componentInstance.completions.at(-1)).toEqual({ value: 200, reason: 'cancel' });
    expect(document.documentElement.style.userSelect).toBe('');
    expect(document.documentElement.style.cursor).toBe('');
    vi.unstubAllGlobals();
  });
});
