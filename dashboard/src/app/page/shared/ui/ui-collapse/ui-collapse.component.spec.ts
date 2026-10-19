import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { describe, expect, it, vi } from 'vitest';
import { UIAccordionComponent } from '../ui-accordion/ui-accordion.component';
import { UICollapseComponent } from './ui-collapse.component';

@Component({
  imports: [UIAccordionComponent, UICollapseComponent],
  template: `<app-ui-accordion>
    <app-ui-collapse heading="One"
      ><button collapse-trailing type="button">Action</button>One body</app-ui-collapse
    >
    <app-ui-collapse heading="Two">Two body</app-ui-collapse>
  </app-ui-accordion>`,
})
class CollapseHost {}

describe('UICollapseComponent', () => {
  it('keeps trailing actions independent of the toggle', () => {
    const fixture = TestBed.createComponent(UICollapseComponent);
    fixture.componentRef.setInput('heading', 'Details');
    const opened = vi.fn();
    fixture.componentInstance.opened.subscribe(opened);
    fixture.detectChanges();
    const toggle = fixture.nativeElement.querySelector('.toggle') as HTMLButtonElement;
    toggle.click();
    fixture.detectChanges();
    expect(toggle.getAttribute('aria-expanded')).toBe('true');
    expect(opened).toHaveBeenCalledOnce();
  });

  it('coordinates exclusive items and leaves the action outside the toggle', () => {
    const fixture = TestBed.createComponent(CollapseHost);
    fixture.detectChanges();
    const toggles = [...fixture.nativeElement.querySelectorAll('.toggle')] as HTMLButtonElement[];
    toggles[0].click();
    fixture.detectChanges();
    toggles[1].click();
    fixture.detectChanges();
    expect(toggles[0].getAttribute('aria-expanded')).toBe('false');
    expect(toggles[1].getAttribute('aria-expanded')).toBe('true');
    const action = fixture.nativeElement.querySelector('[collapse-trailing]') as HTMLButtonElement;
    const heading = fixture.nativeElement.querySelector('.heading') as HTMLElement;
    const indicator = fixture.nativeElement.querySelector('.indicator') as HTMLElement;
    expect(
      heading.compareDocumentPosition(indicator) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(action.closest('.toggle')).toBeNull();
  });
});
