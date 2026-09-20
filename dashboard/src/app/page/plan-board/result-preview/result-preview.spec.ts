import { TestBed } from '@angular/core/testing';
import { describe, expect, it, vi } from 'vitest';
import {
  CustomEvaluatedGroupKind,
  EvaluatedGroup,
  EvaluatedGroupRef,
  EvaluationReport,
  Trace,
} from '../../../common/model/core/evaluation';
import { Profile } from '../../../common/model/core/profile';
import { ResultPreview } from './result-preview';

describe('ResultPreview', () => {
  it('shows result categories as tabs and omits the node-and-tag panel', async () => {
    const fixture = TestBed.createComponent(ResultPreview);
    fixture.componentRef.setInput(
      'report',
      new EvaluationReport([], [], [], [], [], Profile.empty()),
    );
    fixture.detectChanges();
    await fixture.whenStable();
    const labels = [...fixture.nativeElement.querySelectorAll('[role="tab"]')].map((tab: Element) =>
      tab.textContent?.trim(),
    );
    expect(labels).toEqual(['输出结果', '诊断', '评估定位轨迹', '渲染结果']);
    expect(fixture.nativeElement.textContent).not.toContain('节点与标签');
    expect(fixture.nativeElement.textContent).toContain('本次评估没有生成策略组');
  });

  it('resolves a referenced group without recursively flattening its members', () => {
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
      '被引用组',
      [new EvaluatedGroupRef('group:1')],
      true,
      true,
      '被引用组',
    );
    const report = new EvaluationReport([], [first, second], [], [], [], Profile.empty());
    const fixture = TestBed.createComponent(ResultPreview);
    fixture.componentRef.setInput('report', report);
    fixture.detectChanges();

    expect(fixture.componentInstance.memberGroup(first.members[0])).toBe(second);
    expect(fixture.componentInstance.memberGroup(second.members[0])).toBe(first);
    expect(fixture.componentInstance.memberNode(first.members[0])).toBeNull();
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
