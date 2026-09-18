import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  AllPredicate,
  AtomPredicate,
  EqualsStringMatch,
  NodeAnnotation,
  NodeNamePredicate,
  RegionDimension,
  RemoteSourceInput,
  SelectGroupStrategy,
} from '../../common/model/core/plan';
import { PlanBoard } from './plan-board';
import {
  DirectNodeOrigin,
  InputNode,
  ResolvedDependencies,
  SourceProfile,
} from '../../common/model/core/evaluation';
import { Profile, Proxy } from '../../common/model/core/profile';
import { ProxyClient } from '../../common/model/core/proxy-client';

describe('PlanBoard workbench behavior', () => {
  let component: PlanBoard;

  beforeEach(() => {
    localStorage.clear();
    TestBed.configureTestingModule({
      imports: [PlanBoard],
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    component = TestBed.createComponent(PlanBoard).componentInstance;
  });

  it('does not edit the last entered group when a drag ends outside its container', () => {
    const setGroups = vi.spyOn(component.board, 'setGroups');
    component.dropNodes(1, {
      isPointerOverContainer: false,
      previousContainer: { id: 'input-nodes' },
      item: { data: [] },
    } as never);
    expect(setGroups).not.toHaveBeenCalled();
  });

  it('wraps an automatic policy as an output root group', () => {
    component.addPolicy();
    const policy = component.board.plan().grouping_policies[0];
    component.wrapPolicy(policy);

    const group = component.board.plan().groups[0];
    expect(group.name).toBe(`自动组 ${policy.id}`);
    expect(group.member_selectors[0].kind).toBe('base_groups');
    expect(component.board.plan().output.roots).toContain(group.id);
  });

  it('reorders policy dimensions through service-backed mutations', () => {
    component.addPolicy();
    const policy = component.board.plan().grouping_policies[0];
    component.addDimension(policy.id, 'source');
    component.moveDimension(policy.id, 1, -1);

    expect(component.board.plan().grouping_policies[0].group_by[0].kind).toBe('source');
    expect(component.board.plan().grouping_policies[0].group_by[1]).toBeInstanceOf(RegionDimension);
  });

  it('removes one tag from a multi-tag annotation without deleting the other tag', () => {
    const source = component.board.addSource('remote');
    component.board.updateSource(source.id, (item) => {
      item.input = new RemoteSourceInput('https://example.com/sub');
      item.annotations = [new NodeAnnotation(new AllPredicate([]), ['家宽', '低倍率'])];
    });
    const row = component.tags().find((item) => item.tag === '家宽')!.rows[0];

    component.removeTagSelector('家宽', row);

    expect(component.board.plan().sources[0].annotations).toHaveLength(1);
    expect(component.board.plan().sources[0].annotations[0].add_tags).toEqual(['低倍率']);
  });

  it('splits a multi-tag annotation before editing one tag predicate', () => {
    const source = component.board.addSource('remote');
    component.board.updateSource(source.id, (item) => {
      item.annotations = [new NodeAnnotation(new AllPredicate([]), ['家宽', '低倍率'])];
    });
    const row = component.tags().find((item) => item.tag === '家宽')!.rows[0];

    component.updateTagPredicate(
      '家宽',
      row,
      new AtomPredicate(new NodeNamePredicate(new EqualsStringMatch('节点 A'))),
    );

    const annotations = component.board.plan().sources[0].annotations;
    expect(annotations).toHaveLength(2);
    expect(annotations[0].add_tags).toEqual(['低倍率']);
    expect(annotations[0].when).toBeInstanceOf(AllPredicate);
    expect(annotations[1].add_tags).toEqual(['家宽']);
    expect(annotations[1].when).toBeInstanceOf(AtomPredicate);
  });

  it('rejects node rows from an expired source snapshot', () => {
    component.createGroup([
      {
        key: '[1,"old","node"]',
        identity: 'node',
        source: 1,
        sourceName: '旧来源',
        proxy: {} as never,
        region: '',
      },
    ]);

    expect(component.board.plan().groups).toHaveLength(0);
    expect(component.operationError()).toContain('快照已过期');
  });

  it('creates a visible output root group from current selected nodes', () => {
    const source = component.board.addSource('remote');
    const proxy = new Proxy(
      '香港 01',
      'ss',
      'hk.example.com',
      443,
      null,
      null,
      null,
      null,
      null,
      null,
      [],
      {},
      null,
    );
    const profile = new SourceProfile(
      source.id,
      ProxyClient.Surge,
      'input',
      'snapshot',
      1,
      '',
      Profile.empty(),
      new ResolvedDependencies([], []),
      [],
      [new InputNode('node-1', source.id, proxy, [new DirectNodeOrigin(0)], '香港')],
      [],
    );
    (
      component.board as unknown as {
        sourceProfilesState: { set(value: Map<number, SourceProfile>): void };
      }
    ).sourceProfilesState.set(new Map([[source.id, profile]]));

    component.createGroup(component.nodes());

    const group = component.board.plan().groups[0];
    expect(component.board.plan().output.roots).toContain(group.id);
    expect(component.centerTab()).toBe(1);
    expect(component.expandedGroupId()).toBe(group.id);
  });

  it('uses the selected group strategy without mutating the old instance', () => {
    component.addGroup();
    const group = component.board.plan().groups[0];
    const previous = group.strategy;
    component.setGroupStrategy(group.id, new SelectGroupStrategy());
    expect(component.board.plan().groups[0].strategy).not.toBe(previous);
  });

  it('opens and focuses the selector addressed by a custom trace path', () => {
    component.addGroup();
    const group = component.board.plan().groups[0];

    component.focusPath(`custom/${group.id}/member_selectors/0`);

    expect(component.centerTab()).toBe(1);
    expect(component.expandedGroupId()).toBe(group.id);
    expect(component.focusedPath()).toBe(`custom/${group.id}/member_selectors/0`);
  });

  it('does not use stale report paths to navigate the current plan', () => {
    (
      component.board as unknown as { previewStaleState: { set(value: boolean): void } }
    ).previewStaleState.set(true);

    component.focusPath('custom/12/member_selectors/0');

    expect(component.centerTab()).toBe(0);
    expect(component.expandedGroupId()).toBeNull();
    expect(component.operationError()).toContain('预览正在更新');
  });
});
