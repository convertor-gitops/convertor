import { UiButtonDirective, UiPanelDirective } from '../shared/ui';
import { CdkDragDrop, CdkDropList } from '@angular/cdk/drag-drop';
import {
  ChangeDetectionStrategy,
  afterNextRender,
  Injector,
  Component,
  computed,
  effect,
  inject,
  signal,
  viewChild,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatExpansionModule } from '@angular/material/expansion';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatDrawer, MatSidenavModule } from '@angular/material/sidenav';
import { MatTabsModule } from '@angular/material/tabs';
import { SurgeBinding } from '../../common/model/local/surge-binding';
import { patchSurgeHeader } from '../../common/patch/surge-patch';
import * as P from '../../common/model/core/plan';
import { ProxyClient } from '../../common/model/core/proxy-client';
import {
  BaseEvaluatedGroupKind,
  CustomEvaluatedGroupKind,
} from '../../common/model/core/evaluation';
import { LocalFsService } from '../../service/local-fs.service';
import { MetadataService } from '../../service/metadata.service';
import { PlanBoardApiService } from '../../service/plan-board-api.service';
import { PlanBoardService } from '../../service/plan-board.service';
import { CopyAction } from '../shared/copy-action/copy-action';
import {
  acceptsPickedNodes as groupAcceptsPickedNodes,
  appendPickedNodes,
  inputNodes,
  InputNodeRow,
  referencedNodes,
} from './board-operations';
import { MemberEditor } from './member-editor/member-editor';
import { NodeBrowser } from './node-browser/node-browser';
import { EditablePredicate, PredicateEditor } from './predicate-editor/predicate-editor';
import { ResultPreview } from './result-preview/result-preview';
import { SourceManager } from './source-manager/source-manager';
import { StrategyEditor } from './strategy-editor/strategy-editor';

interface TagSelectorRow {
  source: P.Source;
  annotationIndex: number;
  predicate: P.Predicate<P.NodePredicate>;
}
interface TagContainer {
  tag: string;
  rows: TagSelectorRow[];
}

@Component({
  selector: 'app-plan-board',
  imports: [
    UiButtonDirective,
    UiPanelDirective,
    FormsModule,
    CdkDropList,
    MatButtonModule,
    MatCheckboxModule,
    MatExpansionModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    MatSidenavModule,
    MatTabsModule,
    CopyAction,
    MemberEditor,
    NodeBrowser,
    PredicateEditor,
    ResultPreview,
    SourceManager,
    StrategyEditor,
  ],
  providers: [PlanBoardApiService, PlanBoardService],
  templateUrl: './plan-board.html',
  styleUrl: './plan-board.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PlanBoard {
  private readonly injector = inject(Injector);
  readonly board = inject(PlanBoardService);
  readonly metadata = inject(MetadataService);
  readonly localFs = inject(LocalFsService);
  readonly restoreUrl = signal('');
  readonly highlighted = signal<ReadonlySet<string>>(new Set());
  readonly centerTab = signal(0);
  readonly expandedPolicyId = signal<number | null>(null);
  readonly expandedGroupId = signal<number | null>(null);
  readonly focusedNodeName = signal<string | null>(null);
  readonly focusedTag = signal<string | null>(null);
  readonly focusedPath = signal<string | null>(null);
  readonly tagDrafts = signal<ReadonlyMap<string, string>>(new Map());
  readonly nodeBrowser = viewChild(NodeBrowser);
  readonly sourceDrawer = viewChild(MatDrawer);
  readonly localBinding = signal<SurgeBinding | null>(null);
  readonly localFiles = signal<string[]>([]);
  readonly localMessage = signal<string | null>(null);
  readonly localPreview = signal<string | null>(null);
  readonly operationError = signal<string | null>(null);
  readonly acceptsPickedNodes = groupAcceptsPickedNodes;
  readonly nodes = computed(() => inputNodes(this.board.plan(), this.board.sourceProfiles()));
  readonly dropTargets = computed(() =>
    this.board
      .plan()
      .groups.filter(groupAcceptsPickedNodes)
      .map((group) => this.groupDropId(group.id)),
  );
  readonly tags = computed<TagContainer[]>(() => {
    const containers = new Map<string, TagSelectorRow[]>();
    for (const source of this.board.plan().sources) {
      source.annotations.forEach((annotation, annotationIndex) => {
        for (const tag of annotation.add_tags) {
          const rows = containers.get(tag) ?? [];
          rows.push({ source, annotationIndex, predicate: annotation.when });
          containers.set(tag, rows);
        }
      });
    }
    return [...containers].map(([tag, rows]) => ({ tag, rows }));
  });

  constructor() {
    effect(() => {
      if (this.board.previewStale()) {
        this.highlighted.set(new Set());
        this.focusedPath.set(null);
      }
    });
    void this.restoreLocalBinding();
  }

  createGroup(nodes: InputNodeRow[]): void {
    if (!nodes.length) return;
    const groups = this.cloneGroups();
    const id = Math.max(0, ...groups.map((group) => group.id)) + 1;
    groups.push(
      new P.CustomGroup(
        id,
        `节点组 ${id}`,
        new P.SelectGroupStrategy(),
        [],
        new P.ErrorEmptyGroupPolicy(),
      ),
    );
    const plan = this.board.plan().clone();
    plan.groups = groups;
    if (this.applyPickedNodes(plan, id, nodes)) {
      this.board.updateOutput((next) => {
        next.output.roots = [...new Set([...next.output.roots, id])];
      });
      this.focusGroup(id);
    }
  }

  dropNodes(groupId: number, event: CdkDragDrop<unknown>): void {
    if (!event.isPointerOverContainer || event.previousContainer.id !== 'input-nodes') return;
    const nodes = event.item.data as InputNodeRow[];
    if (!Array.isArray(nodes)) return;
    if (this.applyPickedNodes(this.board.plan(), groupId, nodes)) this.focusGroup(groupId);
  }

  groupDropId(id: number): string {
    return `plan-group-${id}`;
  }

  addPolicy(): void {
    const policies = this.clonePolicies();
    const id = Math.max(0, ...policies.map((policy) => policy.id)) + 1;
    policies.push(new P.GroupingPolicy(id, [new P.RegionDimension()], new P.SelectGroupStrategy()));
    this.board.setGroupingPolicies(policies);
    this.focusPolicy(id);
  }

  removePolicy(id: number): void {
    this.board.setGroupingPolicies(
      this.board.plan().grouping_policies.filter((policy) => policy.id !== id),
    );
  }

  addDimension(policyId: number, kind: P.NodeDimension['kind']): void {
    const policies = this.clonePolicies();
    const policy = policies.find((item) => item.id === policyId);
    if (!policy) return;
    policy.group_by.push(this.dimension(kind));
    this.board.setGroupingPolicies(policies);
  }

  changeDimension(policyId: number, index: number, kind: P.NodeDimension['kind']): void {
    this.mutatePolicy(policyId, (policy) => (policy.group_by[index] = this.dimension(kind)));
  }

  setDimensionTag(policyId: number, index: number, tag: string): void {
    this.mutatePolicy(policyId, (policy) => {
      if (policy.group_by[index] instanceof P.HasTagDimension)
        (policy.group_by[index] as P.HasTagDimension).tag = tag;
    });
  }

  dimensionTag(dimension: P.NodeDimension): string {
    return dimension instanceof P.HasTagDimension ? dimension.tag : '';
  }

  moveDimension(policyId: number, index: number, offset: -1 | 1): void {
    this.mutatePolicy(policyId, (policy) => move(policy.group_by, index, offset));
  }

  removeDimension(policyId: number, index: number): void {
    this.mutatePolicy(policyId, (policy) => policy.group_by.splice(index, 1));
  }

  setPolicyStrategy(policyId: number, strategy: P.GroupStrategy): void {
    this.mutatePolicy(policyId, (policy) => (policy.strategy = strategy));
  }

  wrapPolicy(policy: P.GroupingPolicy): void {
    const groups = this.cloneGroups();
    const id = Math.max(0, ...groups.map((group) => group.id)) + 1;
    groups.push(
      new P.CustomGroup(
        id,
        `自动组 ${policy.id}`,
        new P.SelectGroupStrategy(),
        [
          new P.BaseGroupsMemberSelector(
            new P.BaseGroupSelection(policy.id, 'Roots', new P.AllPredicate([])),
          ),
        ],
        new P.ErrorEmptyGroupPolicy(),
      ),
    );
    this.board.setGroups(groups);
    this.board.updateOutput(
      (plan) => (plan.output.roots = [...new Set([...plan.output.roots, id])]),
    );
  }

  addGroup(): void {
    const groups = this.cloneGroups();
    const id = Math.max(0, ...groups.map((group) => group.id)) + 1;
    groups.push(
      new P.CustomGroup(
        id,
        `节点组 ${id}`,
        new P.SelectGroupStrategy(),
        [new P.BuiltinMemberSelector('Direct')],
        new P.ErrorEmptyGroupPolicy(),
      ),
    );
    this.board.setGroups(groups);
    this.focusGroup(id);
  }

  updateGroup(id: number, update: (group: P.CustomGroup) => void): void {
    const groups = this.cloneGroups();
    const group = groups.find((item) => item.id === id);
    if (!group) return;
    update(group);
    this.board.setGroups(groups);
  }

  setGroupName(id: number, name: string): void {
    this.updateGroup(id, (group) => (group.name = name));
  }

  setGroupStrategy(id: number, strategy: P.GroupStrategy): void {
    this.updateGroup(id, (group) => (group.strategy = strategy));
  }

  removeGroup(id: number): void {
    this.board.setGroups(this.board.plan().groups.filter((group) => group.id !== id));
    this.board.updateOutput(
      (plan) => (plan.output.roots = plan.output.roots.filter((root) => root !== id)),
    );
  }

  addSelector(groupId: number): void {
    const source = this.board.plan().sources[0]?.id ?? 0;
    this.updateGroup(groupId, (group) =>
      group.member_selectors.push(
        new P.NodesMemberSelector(
          new P.NodeSelection(
            source,
            new P.AtomPredicate(new P.NodeNamePredicate(new P.EqualsStringMatch(''))),
          ),
        ),
      ),
    );
  }

  setSelector(groupId: number, index: number, selector: P.MemberSelector): void {
    this.updateGroup(groupId, (group) => (group.member_selectors[index] = selector));
  }

  moveSelector(groupId: number, index: number, offset: -1 | 1): void {
    this.updateGroup(groupId, (group) => move(group.member_selectors, index, offset));
  }

  removeSelector(groupId: number, index: number): void {
    this.updateGroup(groupId, (group) => group.member_selectors.splice(index, 1));
  }

  emptyPolicyValue(group: P.CustomGroup): string {
    if (group.on_empty instanceof P.ErrorEmptyGroupPolicy) return 'error';
    const target = (group.on_empty as P.UseEmptyGroupPolicy).target;
    return target instanceof P.GroupTarget
      ? `group:${target.group}`
      : `builtin:${(target as P.BuiltinTarget).builtin}`;
  }

  setEmptyPolicy(groupId: number, value: string): void {
    this.updateGroup(groupId, (group) => {
      group.on_empty =
        value === 'error'
          ? new P.ErrorEmptyGroupPolicy()
          : new P.UseEmptyGroupPolicy(
              value.startsWith('group:')
                ? new P.GroupTarget(Number(value.slice(6)))
                : new P.BuiltinTarget(value.endsWith('Reject') ? 'Reject' : 'Direct'),
            );
    });
  }

  toggleRoot(groupId: number, checked: boolean): void {
    this.board.updateOutput((plan) => {
      plan.output.roots = checked
        ? [...new Set([...plan.output.roots, groupId])]
        : plan.output.roots.filter((id) => id !== groupId);
    });
  }

  addTag(): void {
    const source = this.board.plan().sources[0];
    if (!source) return;
    const existing = new Set(this.tags().map((item) => item.tag));
    let suffix = 1;
    while (existing.has(`新标签 ${suffix}`)) suffix++;
    const tag = `新标签 ${suffix}`;
    this.board.updateSource(source.id, (item) =>
      item.annotations.push(new P.NodeAnnotation(new P.AllPredicate([]), [tag])),
    );
  }

  renameTag(oldTag: string, newTag: string): void {
    const next = newTag.trim();
    if (!next || next === oldTag) return;
    for (const source of this.board.plan().sources) {
      if (source.annotations.some((annotation) => annotation.add_tags.includes(oldTag))) {
        this.board.updateSource(source.id, (item) => {
          for (const annotation of item.annotations)
            annotation.add_tags = annotation.add_tags.map((tag) => (tag === oldTag ? next : tag));
        });
      }
    }
    this.tagDrafts.update((current) => {
      const next = new Map(current);
      next.delete(oldTag);
      return next;
    });
    if (this.focusedTag() === oldTag) {
      this.focusedTag.set(next);
      this.scrollTo(this.tagPanelId(next));
    }
  }

  tagDraft(tag: string): string {
    return this.tagDrafts().get(tag) ?? tag;
  }

  setTagDraft(tag: string, value: string): void {
    this.tagDrafts.update((current) => new Map(current).set(tag, value));
  }

  addTagSelector(tag: string, sourceId: number): void {
    this.board.updateSource(sourceId, (source) =>
      source.annotations.push(new P.NodeAnnotation(new P.AllPredicate([]), [tag])),
    );
  }

  updateTagPredicate(tag: string, row: TagSelectorRow, predicate: EditablePredicate): void {
    this.board.updateSource(row.source.id, (source) => {
      const annotation = source.annotations[row.annotationIndex];
      const next = predicate as P.Predicate<P.NodePredicate>;
      if (annotation.add_tags.length > 1) {
        annotation.add_tags = annotation.add_tags.filter((item) => item !== tag);
        source.annotations.push(new P.NodeAnnotation(next, [tag]));
      } else {
        annotation.when = next;
      }
    });
  }

  removeTagSelector(tag: string, row: TagSelectorRow): void {
    this.board.updateSource(row.source.id, (source) => {
      const annotation = source.annotations[row.annotationIndex];
      annotation.add_tags = annotation.add_tags.filter((item) => item !== tag);
      if (!annotation.add_tags.length) source.annotations.splice(row.annotationIndex, 1);
    });
  }

  focusResource(identity: string): void {
    if (this.board.previewStale()) {
      this.operationError.set('预览正在更新，请等待最新结果后再定位资源。');
      return;
    }
    const report = this.board.report();
    this.highlighted.set(
      report ? new Set([identity, ...referencedNodes(report, [identity])]) : new Set([identity]),
    );
    const group = report?.groups.find((item) => item.identity === identity);
    if (group?.kind instanceof CustomEvaluatedGroupKind) this.focusGroup(group.kind.id);
    else if (group?.kind instanceof BaseEvaluatedGroupKind) this.focusPolicy(group.kind.policy);
    else {
      const node = report?.nodes.find((item) => item.identity === identity);
      if (node) {
        this.focusedNodeName.set(node.proxy.name);
        this.nodeBrowser()?.search.set(node.proxy.name);
        this.scrollTo('input-node-browser');
      }
    }
  }

  focusPath(path: string): void {
    if (this.board.previewStale()) {
      this.operationError.set('预览正在更新，请等待最新结果后再定位诊断或命中结果。');
      return;
    }
    this.focusedPath.set(path);
    this.locatePlanPath(path);
    const report = this.board.report();
    const exact = report?.trace.filter((item) => item.path === path) ?? [];
    const traces = exact.length
      ? exact
      : (report?.trace.filter((item) => relatedPath(path, item.path)) ?? []);
    const resources = traces.flatMap((item) => item.resources);
    this.highlighted.set(
      report ? new Set([...resources, ...referencedNodes(report, resources)]) : new Set(),
    );
  }

  restore(): void {
    const url = this.restoreUrl().trim();
    if (url) this.board.restoreFromUrl(url);
  }

  clearNodeFocus(): void {
    this.focusedNodeName.set(null);
    this.nodeBrowser()?.search.set('');
  }

  setSettingsSource(source: number): void {
    this.board.updateOutput((plan) => (plan.output.settings_source = source));
  }

  fallbackValue(): string {
    const fallback = this.board.plan().output.fallback;
    return fallback instanceof P.GroupTarget
      ? `group:${fallback.group}`
      : `builtin:${(fallback as P.BuiltinTarget).builtin}`;
  }

  setFallback(value: string): void {
    this.board.updateOutput((plan) => {
      plan.output.fallback = value.startsWith('group:')
        ? new P.GroupTarget(Number(value.slice(6)))
        : new P.BuiltinTarget(value.endsWith('Reject') ? 'Reject' : 'Direct');
    });
  }

  async pickLocalDirectory(): Promise<void> {
    try {
      const dirHandle = await this.localFs.pickDirectory();
      const files = await this.localFs.listFiles(dirHandle);
      const binding: SurgeBinding = {
        dirHandle,
        dirName: dirHandle.name,
        client: ProxyClient.Surge,
        mainProfile: files.find((file) => file.endsWith('.conf')) ?? files[0] ?? null,
        rulesProfile: null,
      };
      await this.localFs.saveBinding(binding);
      this.localBinding.set(binding);
      this.localFiles.set(files);
      this.localMessage.set(null);
    } catch (error) {
      this.localMessage.set(message(error));
    }
  }

  async setLocalMainFile(file: string): Promise<void> {
    const binding = this.localBinding();
    if (!binding) return;
    const updated = { ...binding, mainProfile: file || null };
    await this.localFs.saveBinding(updated);
    this.localBinding.set(updated);
    this.localPreview.set(null);
  }

  async previewLocalInjection(): Promise<void> {
    const binding = this.localBinding();
    const url = this.board.subscriptionUrl();
    if (!binding?.mainProfile || !url) return;
    try {
      if (!(await this.localFs.ensurePermission(binding.dirHandle, 'readwrite')))
        throw new Error('没有目录读写权限');
      const content = await this.localFs.readFile(binding.dirHandle, binding.mainProfile);
      this.localPreview.set(
        patchSurgeHeader(content, `#!MANAGED-CONFIG ${url} interval=86400 strict=true`),
      );
      this.localMessage.set(null);
    } catch (error) {
      this.localMessage.set(message(error));
    }
  }

  async applyLocalInjection(): Promise<void> {
    const binding = this.localBinding();
    const preview = this.localPreview();
    if (!binding?.mainProfile || preview === null) return;
    try {
      if (!(await this.localFs.ensurePermission(binding.dirHandle, 'readwrite')))
        throw new Error('没有目录读写权限');
      await this.localFs.writeFile(binding.dirHandle, binding.mainProfile, preview);
      this.localMessage.set(`已更新 ${binding.mainProfile}`);
    } catch (error) {
      this.localMessage.set(message(error));
    }
  }

  private applyPickedNodes(plan: P.Plan, groupId: number, rows: InputNodeRow[]): boolean {
    const current = new Map(this.nodes().map((node) => [node.key, node]));
    if (rows.some((row) => !current.has(row.key))) {
      this.operationError.set('来源已刷新，所选节点快照已过期，请重新选择。');
      return false;
    }
    const currentRows = rows.map((row) => current.get(row.key)!);
    try {
      const group = plan.groups.find((item) => item.id === groupId);
      if (!group || !groupAcceptsPickedNodes(group))
        throw new Error('该组包含条件或组引用，不能拖入逐个选取的节点。');
      this.board.setGroups(appendPickedNodes(plan, groupId, currentRows).groups);
      this.operationError.set(null);
      return true;
    } catch (error) {
      this.operationError.set(message(error));
      return false;
    }
  }

  private locatePlanPath(path: string): void {
    const policyMatch = /(?:^|\/)grouping_policies\/(\d+)/.exec(path);
    if (policyMatch) {
      this.focusPolicy(Number(policyMatch[1]));
      return;
    }
    const customMatch = /^(?:custom|groups)\/(\d+)(?:\/member_selectors\/(\d+))?/.exec(path);
    if (customMatch) {
      const id = Number(customMatch[1]);
      this.focusGroup(id, customMatch[2] === undefined ? null : Number(customMatch[2]));
      return;
    }
    const annotationMatch = /^sources?\/(\d+)\/annotations\/(\d+)/.exec(path);
    if (annotationMatch) {
      const sourceId = Number(annotationMatch[1]);
      const annotationIndex = Number(annotationMatch[2]);
      const annotation = this.board.plan().sources.find((source) => source.id === sourceId)
        ?.annotations[annotationIndex];
      this.centerTab.set(2);
      const tag = annotation?.add_tags[0] ?? null;
      this.focusedTag.set(tag);
      if (tag) this.scrollTo(this.tagPanelId(tag));
      return;
    }
    const sourceMatch = /^sources?\/(\d+)/.exec(path);
    if (sourceMatch) {
      this.board.selectSource(Number(sourceMatch[1]));
      void this.sourceDrawer()?.open();
      this.scrollTo(`source-${sourceMatch[1]}`);
    }
  }

  private focusPolicy(id: number): void {
    this.centerTab.set(0);
    this.expandedPolicyId.set(id);
    this.scrollTo(`policy-${id}`);
  }

  private focusGroup(id: number, selectorIndex: number | null = null): void {
    this.centerTab.set(1);
    this.expandedGroupId.set(id);
    this.scrollTo(
      selectorIndex === null ? this.groupDropId(id) : `selector-${id}-${selectorIndex}`,
    );
  }

  tagPanelId(tag: string): string {
    return `tag-${encodeURIComponent(tag)}`;
  }

  private scrollTo(id: string): void {
    afterNextRender(
      () =>
        globalThis.document
          ?.getElementById(id)
          ?.scrollIntoView?.({ block: 'nearest', inline: 'nearest' }),
      { injector: this.injector },
    );
  }

  private clonePolicies(): P.GroupingPolicy[] {
    return this.board
      .plan()
      .grouping_policies.map((item) => P.GroupingPolicy.deserialize(item.serialize()));
  }

  private mutatePolicy(id: number, update: (policy: P.GroupingPolicy) => void): void {
    const policies = this.clonePolicies();
    const policy = policies.find((item) => item.id === id);
    if (!policy) return;
    update(policy);
    this.board.setGroupingPolicies(policies);
  }

  private cloneGroups(): P.CustomGroup[] {
    return this.board.plan().groups.map((item) => P.CustomGroup.deserialize(item.serialize()));
  }

  private dimension(kind: P.NodeDimension['kind']): P.NodeDimension {
    if (kind === 'source') return new P.SourceDimension();
    if (kind === 'protocol') return new P.ProtocolDimension();
    if (kind === 'has_tag') return new P.HasTagDimension('');
    return new P.RegionDimension();
  }

  private async restoreLocalBinding(): Promise<void> {
    try {
      const binding = await this.localFs.loadBinding();
      if (!binding) return;
      this.localBinding.set(binding);
      this.localFiles.set(await this.localFs.listFiles(binding.dirHandle));
    } catch {
      // A stale file-system handle is optional and does not block the workbench.
    }
  }
}

function move<T>(items: T[], index: number, offset: -1 | 1): void {
  const target = index + offset;
  if (target < 0 || target >= items.length) return;
  [items[index], items[target]] = [items[target], items[index]];
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function relatedPath(left: string, right: string): boolean {
  return left === right || left.startsWith(`${right}/`) || right.startsWith(`${left}/`);
}
