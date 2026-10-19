import {
  UITabsComponent,
  UITabComponent,
  UIProgressComponent,
  UIInfoRowComponent,
} from '../../shared/ui/ui-controls/ui-controls';
import { UICollapseComponent } from '../../shared/ui';
import { UIButtonComponent, UICardComponent, UINodeCardComponent } from '../../shared/ui';
import { ChangeDetectionStrategy, Component, computed, input, output } from '@angular/core';
import {
  Diagnostic,
  EvaluatedGroup,
  EvaluatedMemberRef,
  EvaluatedNode,
  EvaluationReport,
  Trace,
} from '../../../common/model/core/evaluation';

const EMPTY_HIGHLIGHTS: ReadonlySet<string> = new Set<string>();
interface GroupDisplayRow {
  group: EvaluatedGroup;
  depth: number;
}
@Component({
  selector: 'app-result-preview',
  imports: [
    UITabsComponent,
    UITabComponent,
    UIProgressComponent,
    UIInfoRowComponent,
    UICollapseComponent,
    UICardComponent,
    UINodeCardComponent,
    UIButtonComponent,
  ],
  templateUrl: './result-preview.html',
  styleUrl: './result-preview.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ResultPreview {
  readonly report = input<EvaluationReport | null>(null);
  readonly rendered = input<string | null>(null);
  readonly stale = input(false);
  readonly evaluating = input(false);
  readonly highlighted = input<ReadonlySet<string>>(EMPTY_HIGHLIGHTS);
  readonly resourceFocus = output<string>();
  readonly pathFocus = output<string>();

  readonly reachableNodes = computed(
    () => this.report()?.nodes.filter((node) => node.reachable) ?? [],
  );
  readonly generatedNodes = computed(() => this.report()?.nodes ?? []);
  readonly reachableGroups = computed(
    () => this.report()?.groups.filter((group) => group.reachable) ?? [],
  );
  readonly generatedGroups = computed(() => this.report()?.groups ?? []);
  readonly nodesByIdentity = computed(
    () => new Map((this.report()?.nodes ?? []).map((node) => [node.identity, node])),
  );
  readonly groupsByIdentity = computed(
    () => new Map((this.report()?.groups ?? []).map((group) => [group.identity, group])),
  );
  readonly groupRows = computed<GroupDisplayRow[]>(() => {
    const report = this.report();
    if (!report) return [];

    const groups = new Map(report.groups.map((group) => [group.identity, group]));
    const children = new Map<string | null, string[]>();
    for (const base of report.base_groups) {
      const siblings = children.get(base.parent) ?? [];
      siblings.push(base.identity);
      children.set(base.parent, siblings);
    }

    const rows: GroupDisplayRow[] = [];
    const visited = new Set<string>();
    const visit = (identity: string, depth: number): void => {
      if (visited.has(identity)) return;
      visited.add(identity);
      const group = groups.get(identity);
      if (group) rows.push({ group, depth });
      for (const child of children.get(identity) ?? []) visit(child, depth + 1);
    };

    for (const root of children.get(null) ?? []) visit(root, 0);
    for (const group of report.groups) {
      if (!visited.has(group.identity)) rows.push({ group, depth: 0 });
    }
    return rows;
  });
  readonly errorCount = computed(
    () => this.report()?.diagnostics.filter((item) => item.severity === 'error').length ?? 0,
  );
  readonly warningCount = computed(
    () => this.report()?.diagnostics.filter((item) => item.severity === 'warning').length ?? 0,
  );

  focusResource(identity: string): void {
    this.resourceFocus.emit(identity);
  }

  focusPath(path: string): void {
    this.pathFocus.emit(path);
  }

  isHighlighted(identity: string): boolean {
    return this.highlighted().has(identity);
  }

  nodeName(node: EvaluatedNode): string {
    return node.output_name ?? node.proxy.name;
  }

  groupName(group: EvaluatedGroup): string {
    return group.output_name ?? group.name;
  }

  groupKind(group: EvaluatedGroup): string {
    return group.kind.kind === 'base'
      ? '基础组'
      : group.kind.kind === 'custom'
        ? '自定义组'
        : '导入组';
  }

  resourceState(resource: EvaluatedNode | EvaluatedGroup): string {
    return resource.reachable ? '可达输出' : '仅生成';
  }

  memberNode(member: EvaluatedMemberRef): EvaluatedNode | null {
    return member.kind === 'node' ? (this.nodesByIdentity().get(member.value) ?? null) : null;
  }

  memberGroup(member: EvaluatedMemberRef): EvaluatedGroup | null {
    return member.kind === 'group' ? (this.groupsByIdentity().get(member.value) ?? null) : null;
  }

  nodeBadge(node: EvaluatedNode): string {
    const name = this.nodeName(node).trim();
    const first = name.split(/\s+/u)[0] ?? '';
    if (/\p{Regional_Indicator}/u.test(first)) return first;
    return Array.from(name).slice(0, 2).join('').toUpperCase();
  }

  focusTraceResource(trace: Trace, identity: string): void {
    this.resourceFocus.emit(identity);
    if (trace.path) this.pathFocus.emit(trace.path);
  }

  focusDiagnostic(diagnostic: Diagnostic): void {
    this.pathFocus.emit(diagnostic.path);
  }
}
