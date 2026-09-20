import { TestBed } from '@angular/core/testing';
import { describe, expect, it, vi } from 'vitest';
import { UiNodeCardComponent } from './ui-node-card.component';

describe('UiNodeCardComponent action boundaries', () => {
  it('keeps selection, details and menu actions independent', () => {
    const fixture = TestBed.createComponent(UiNodeCardComponent);
    fixture.componentRef.setInput('name', '香港 01');
    const select = vi.fn(),
      details = vi.fn(),
      actions = vi.fn();
    fixture.componentInstance.selectedChange.subscribe(select);
    fixture.componentInstance.detailsRequested.subscribe(details);
    fixture.componentInstance.actionRequested.subscribe(actions);
    fixture.detectChanges();
    const element: HTMLElement = fixture.nativeElement;
    (element.querySelector('.selection') as HTMLButtonElement).click();
    expect(select).toHaveBeenCalledExactlyOnceWith(true);
    (element.querySelector('[aria-label="查看节点 香港 01"]') as HTMLButtonElement).click();
    (element.querySelector('[aria-label="节点操作 香港 01"]') as HTMLButtonElement).click();
    expect(details).toHaveBeenCalledOnce();
    expect(actions).toHaveBeenCalledOnce();
    expect(select).toHaveBeenCalledTimes(1);
  });

  it('disables the selection and both secondary actions for unavailable nodes', () => {
    const fixture = TestBed.createComponent(UiNodeCardComponent);
    fixture.componentRef.setInput('name', '停用节点');
    fixture.componentRef.setInput('status', 'disabled');
    fixture.detectChanges();
    const buttons = [...fixture.nativeElement.querySelectorAll('button')] as HTMLButtonElement[];
    expect(buttons.length).toBe(3);
    expect(buttons.every((button) => button.disabled)).toBe(true);
  });
});
