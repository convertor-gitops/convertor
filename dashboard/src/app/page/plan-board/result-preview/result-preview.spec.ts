import { TestBed } from '@angular/core/testing';
import { describe, expect, it, vi } from 'vitest';
import {
  CustomEvaluatedGroupKind,
  EvaluatedGroup,
  EvaluatedGroupRef,
  EvaluatedNodeRef,
  EvaluationReport,
  Trace,
} from '../../../common/model/core/evaluation';
import { Profile } from '../../../common/model/core/profile';
import { ResultPreview } from './result-preview';

describe('ResultPreview', () => {
  it('opens and closes the empty groups panel without losing the empty state', async () => {
    const fixture = TestBed.createComponent(ResultPreview);
    fixture.componentRef.setInput(
      'report',
      new EvaluationReport([], [], [], [], [], Profile.empty()),
    );
    fixture.detectChanges();
    await fixture.whenStable();
    const header: HTMLElement = fixture.nativeElement.querySelector('mat-expansion-panel-header');
    expect(header.getAttribute('aria-expanded')).toBe('true');
    header.click();
    fixture.detectChanges();
    expect(header.getAttribute('aria-expanded')).toBe('false');
    header.click();
    fixture.detectChanges();
    expect(header.getAttribute('aria-expanded')).toBe('true');
    expect(fixture.nativeElement.textContent).toContain('本次评估没有生成策略组');
  });

  it('stops recursive group expansion when it encounters a cycle', () => {
    const first = new EvaluatedGroup(
      'group:1',
      new CustomEvaluatedGroupKind(1),
      '入口',
      [new EvaluatedGroupRef('group:2')],
      true,
      true,
      '入口',
    );
    const second = new EvaluatedGroup(
      'group:2',
      new CustomEvaluatedGroupKind(2),
      '循环组',
      [new EvaluatedGroupRef('group:1'), new EvaluatedNodeRef('missing-node')],
      true,
      true,
      '循环组',
    );
    const report = new EvaluationReport([], [first, second], [], [], [], Profile.empty());
    const fixture = TestBed.createComponent(ResultPreview);
    fixture.componentRef.setInput('report', report);
    fixture.detectChanges();

    const rows = fixture.componentInstance.groupGraph(first);

    expect(rows.map((row) => row.identity)).toEqual(['group:2', 'group:1', 'missing-node']);
    expect(rows[1].cycle).toBe(true);
    expect(rows[2].missing).toBe(true);
  });

  it('emits both resource and path when focusing a trace resource', () => {
    const fixture = TestBed.createComponent(ResultPreview);
    const resourceFocus = vi.fn();
    const pathFocus = vi.fn();
    fixture.componentInstance.resourceFocus.subscribe(resourceFocus);
    fixture.componentInstance.pathFocus.subscribe(pathFocus);

    fixture.componentInstance.focusTraceResource(
      new Trace('groups/0/member_selectors/1', []),
      'node:7',
    );

    expect(resourceFocus).toHaveBeenCalledWith('node:7');
    expect(pathFocus).toHaveBeenCalledWith('groups/0/member_selectors/1');
  });
});
