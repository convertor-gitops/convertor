import { UiButtonDirective, UiPanelDirective } from '../../shared/ui';
import { ChangeDetectionStrategy, Component, computed, input, output } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatExpansionModule } from '@angular/material/expansion';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import {
  Diagnostic,
  EvaluatedGroup,
  EvaluatedMemberRef,
  EvaluatedNode,
  EvaluationReport,
  Trace,
} from '../../../common/model/core/evaluation';

const EMPTY_HIGHLIGHTS: ReadonlySet<string> = new Set<string>();
const GRAPH_ROW_LIMIT = 500;

interface GraphRow {
  identity: string;
  kind: EvaluatedMemberRef['kind'] | 'truncated';
  label: string;
  depth: number;
  reachable: boolean;
  cycle: boolean;
  missing: boolean;
}

@Component({
  selector: 'app-result-preview',
  imports: [
    UiButtonDirective,
    UiPanelDirective,
    MatButtonModule,
    MatExpansionModule,
    MatProgressSpinnerModule,
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

  tagAdded(node: EvaluatedNode, tag: string): boolean {
    return !node.original_tags.includes(tag);
  }

  groupGraph(root: EvaluatedGroup): GraphRow[] {
    const report = this.report();
    if (!report) return [];
    const groups = new Map(report.groups.map((group) => [group.identity, group]));
    const nodes = new Map(report.nodes.map((node) => [node.identity, node]));
    const rows: GraphRow[] = [];

    const visit = (
      member: EvaluatedMemberRef,
      depth: number,
      ancestors: ReadonlySet<string>,
    ): void => {
      if (rows.length >= GRAPH_ROW_LIMIT) return;
      if (member.kind === 'builtin') {
        rows.push({
          identity: member.value,
          kind: member.kind,
          label: member.value,
          depth,
          reachable: true,
          cycle: false,
          missing: false,
        });
        return;
      }
      if (member.kind === 'node') {
        const node = nodes.get(member.value);
        rows.push({
          identity: member.value,
          kind: member.kind,
          label: node ? this.nodeName(node) : member.value,
          depth,
          reachable: node?.reachable ?? false,
          cycle: false,
          missing: !node,
        });
        return;
      }

      const group = groups.get(member.value);
      const cycle = ancestors.has(member.value);
      rows.push({
        identity: member.value,
        kind: member.kind,
        label: group ? this.groupName(group) : member.value,
        depth,
        reachable: group?.reachable ?? false,
        cycle,
        missing: !group,
      });
      if (!group || cycle) return;
      const nextAncestors = new Set(ancestors);
      nextAncestors.add(member.value);
      group.members.forEach((child) => visit(child, depth + 1, nextAncestors));
    };

    const ancestors = new Set([root.identity]);
    root.members.forEach((member) => visit(member, 0, ancestors));
    if (rows.length >= GRAPH_ROW_LIMIT)
      rows.push({
        identity: '',
        kind: 'truncated',
        label: `成员过多，仅显示前 ${GRAPH_ROW_LIMIT} 项`,
        depth: 0,
        reachable: false,
        cycle: false,
        missing: false,
      });
    return rows;
  }

  graphPrefix(row: GraphRow): string {
    if (row.kind === 'group') return row.cycle ? '↻' : '▾';
    if (row.kind === 'node') return '●';
    if (row.kind === 'builtin') return '◆';
    return '…';
  }

  graphPadding(row: GraphRow): number {
    return row.depth * 18;
  }

  focusTraceResource(trace: Trace, identity: string): void {
    this.resourceFocus.emit(identity);
    if (trace.path) this.pathFocus.emit(trace.path);
  }

  focusDiagnostic(diagnostic: Diagnostic): void {
    this.pathFocus.emit(diagnostic.path);
  }
}
