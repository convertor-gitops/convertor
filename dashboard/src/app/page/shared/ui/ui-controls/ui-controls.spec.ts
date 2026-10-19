import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { FormsModule } from '@angular/forms';
import { describe, expect, it, vi } from 'vitest';
import { UICheckboxComponent } from './ui-checkbox.component';
import { UIInfoRowComponent } from './ui-info-row.component';
import { UIOrderedOptionsComponent } from './ui-ordered-options.component';
import { UITabComponent } from './ui-tab.component';
import { UITabsComponent } from './ui-tabs.component';

@Component({
  imports: [FormsModule, UICheckboxComponent, UITabsComponent, UITabComponent],
  template: `<app-ui-checkbox [(ngModel)]="checked">Enabled</app-ui-checkbox>
    <app-ui-tabs [(selectedIndex)]="index"
      ><app-ui-tab label="One"><input data-state value="" /></app-ui-tab
      ><app-ui-tab label="Two">Second</app-ui-tab></app-ui-tabs
    >`,
})
class ControlsHost {
  checked = false;
  index = 0;
}

@Component({
  imports: [UIInfoRowComponent],
  template: `<app-ui-info-row [actionable]="true" (activated)="activateRow()">
    Row content
    <button info-row-actions type="button" (click)="activateAction()">Action</button>
  </app-ui-info-row>`,
})
class InfoRowHost {
  rowActivations = 0;
  actionActivations = 0;
  activateRow(): void {
    this.rowActivations += 1;
  }
  activateAction(): void {
    this.actionActivations += 1;
  }
}

describe('shared controls', () => {
  it('supports forms and preserves tab content while switching', async () => {
    const fixture = TestBed.createComponent(ControlsHost);
    fixture.detectChanges();
    (fixture.nativeElement.querySelector('input') as HTMLInputElement).click();
    fixture.detectChanges();
    expect(fixture.componentInstance.checked).toBe(true);
    const tabs = fixture.nativeElement.querySelectorAll(
      '[role="tab"]',
    ) as NodeListOf<HTMLButtonElement>;
    const state = fixture.nativeElement.querySelector('[data-state]') as HTMLInputElement;
    state.value = 'kept';
    tabs[1].click();
    await fixture.whenStable();
    fixture.detectChanges();
    expect(fixture.componentInstance.index).toBe(1);
    expect(tabs[1].getAttribute('aria-selected')).toBe('true');
    const panels = fixture.nativeElement.querySelectorAll('[role="tabpanel"]');
    expect(panels).toHaveLength(2);
    tabs[0].click();
    await fixture.whenStable();
    fixture.detectChanges();
    expect((fixture.nativeElement.querySelector('[data-state]') as HTMLInputElement).value).toBe(
      'kept',
    );
  });

  it('only emits row activation when actionable', () => {
    const fixture = TestBed.createComponent(UIInfoRowComponent);
    const activated = vi.fn();
    fixture.componentInstance.activated.subscribe(activated);
    fixture.detectChanges();
    (fixture.nativeElement as HTMLElement).click();
    expect(activated).not.toHaveBeenCalled();
    fixture.componentRef.setInput('actionable', true);
    fixture.detectChanges();
    (fixture.nativeElement.querySelector('.trigger') as HTMLButtonElement).click();
    expect(activated).toHaveBeenCalledOnce();
  });

  it('keeps projected actions outside the row trigger', () => {
    const fixture = TestBed.createComponent(InfoRowHost);
    fixture.detectChanges();
    const action = fixture.nativeElement.querySelector('[info-row-actions]') as HTMLButtonElement;
    const trigger = fixture.nativeElement.querySelector('.trigger') as HTMLButtonElement;
    expect(action.closest('.trigger')).toBeNull();
    action.click();
    expect(fixture.componentInstance.actionActivations).toBe(1);
    expect(fixture.componentInstance.rowActivations).toBe(0);
    trigger.click();
    expect(fixture.componentInstance.rowActivations).toBe(1);
  });

  it('keeps ordered option values in click order and removes them by toggle', () => {
    const fixture = TestBed.createComponent(UIOrderedOptionsComponent);
    fixture.componentRef.setInput('choices', [
      { label: 'Region', value: 'region' },
      { label: 'Protocol', value: 'protocol' },
    ]);
    fixture.detectChanges();
    const buttons = [...fixture.nativeElement.querySelectorAll('button')] as HTMLButtonElement[];
    buttons[1].click();
    buttons[0].click();
    fixture.detectChanges();
    expect(fixture.componentInstance.values()).toEqual(['protocol', 'region']);
    expect(
      [...fixture.nativeElement.querySelectorAll('.order')].map((item: Element) =>
        item.textContent?.trim(),
      ),
    ).toEqual(['2', '1']);
    buttons[1].click();
    fixture.detectChanges();
    expect(fixture.componentInstance.values()).toEqual(['region']);
  });
});
