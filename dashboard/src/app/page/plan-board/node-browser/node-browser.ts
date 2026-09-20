import { UiNodeCardComponent } from '../../shared/ui';
import { UiButtonComponent, UiTextFieldComponent } from '../../shared/ui';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  input,
  output,
  signal,
  viewChild,
  afterRenderEffect,
  untracked,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { CdkDrag, CdkDragPreview, CdkDropList } from '@angular/cdk/drag-drop';
import { MatTree, MatTreeModule } from '@angular/material/tree';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatTooltipModule } from '@angular/material/tooltip';
import { BrowseDimension, groupInputNodes, InputNodeRow, BrowseBranch } from '../board-operations';

interface TreeItem {
  key: string;
  label: string;
  count: number;
  children: TreeItem[];
  node?: InputNodeRow;
}
@Component({
  selector: 'app-node-browser',
  imports: [
    UiNodeCardComponent,
    UiButtonComponent,
    UiTextFieldComponent,
    FormsModule,
    CdkDrag,
    CdkDragPreview,
    CdkDropList,
    MatTreeModule,
    MatButtonModule,
    MatCheckboxModule,
    MatTooltipModule,
  ],
  templateUrl: './node-browser.html',
  styleUrl: './node-browser.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodeBrowser {
  readonly nodes = input.required<InputNodeRow[]>();
  readonly highlighted = input<ReadonlySet<string>>(new Set());
  readonly dropTargets = input<string[]>([]);
  readonly createGroup = output<InputNodeRow[]>();
  readonly focusNode = output<string>();
  readonly tree = viewChild<MatTree<TreeItem>>(MatTree);
  readonly search = signal('');
  readonly dimensions = signal<BrowseDimension[]>([]);
  readonly selected = signal<ReadonlySet<string>>(new Set());
  readonly filtered = computed(() => {
    const query = this.search().trim().toLocaleLowerCase();
    return this.nodes().filter(
      (node) =>
        !query ||
        [
          node.proxy.name,
          node.proxy.server,
          node.sourceName,
          node.proxy.protocol,
          ...node.proxy.tags,
        ]
          .join(' ')
          .toLocaleLowerCase()
          .includes(query),
    );
  });
  readonly picked = computed(() => this.nodes().filter((node) => this.selected().has(node.key)));
  readonly data = computed(() => {
    const nodes = this.filtered();
    const dimensions = this.dimensions();
    return dimensions.length
      ? this.toTree(groupInputNodes(nodes, dimensions))
      : this.toLeaves(nodes);
  });
  readonly children = (item: TreeItem): TreeItem[] => item.children;
  readonly isBranch = (_: number, item: TreeItem): boolean => !item.node;
  readonly track = (_: number, item: TreeItem): string => item.key;
  readonly expansionKey = (item: TreeItem): string => item.key;
  readonly labels = { source: '来源', protocol: '协议', region: '地区', has_tag: '原始标签' };
  readonly dimensionKinds: BrowseDimension['kind'][] = ['source', 'protocol', 'region', 'has_tag'];
  constructor() {
    afterRenderEffect(() => {
      const highlighted = this.highlighted();
      const tree = this.tree();
      const data = this.data();
      if (!tree || !highlighted.size) return;
      untracked(() => {
        const reveal = (item: TreeItem): boolean => {
          if (item.node) return highlighted.has(item.node.identity);
          const hits = item.children.map(reveal).some(Boolean);
          if (hits) tree.expand(item);
          return hits;
        };
        data.forEach(reveal);
      });
    });
    effect(() => {
      const keys = new Set(this.nodes().map((node) => node.key));
      this.selected.update((selected) => new Set([...selected].filter((key) => keys.has(key))));
    });
  }
  regionBadge(region: string): string {
    return region.match(/\p{Regional_Indicator}{2}/u)?.[0] ?? (region.slice(0, 2) || '—');
  }
  toggle(node: InputNodeRow, checked: boolean): void {
    this.selected.update((current) => {
      const next = new Set(current);
      checked ? next.add(node.key) : next.delete(node.key);
      return next;
    });
  }
  selectVisible(): void {
    this.selected.update(
      (current) => new Set([...current, ...this.filtered().map((node) => node.key)]),
    );
  }
  expandAll(): void {
    const tree = this.tree();
    const expand = (item: TreeItem): void => {
      if (!item.node) tree?.expand(item);
      item.children.forEach(expand);
    };
    this.data().forEach(expand);
  }
  clear(): void {
    this.selected.set(new Set());
  }
  dragNodes(node: InputNodeRow): InputNodeRow[] {
    return this.selected().has(node.key) ? this.picked() : [node];
  }
  toggleDimension(kind: BrowseDimension['kind']): void {
    this.dimensions.update((items) => {
      const index = items.findIndex((item) => item.kind === kind);
      if (index >= 0) return items.filter((_, itemIndex) => itemIndex !== index);
      return [...items, { kind, ...(kind === 'has_tag' ? { tag: '' } : {}) }];
    });
  }
  dimensionOrder(kind: BrowseDimension['kind']): number {
    return this.dimensions().findIndex((item) => item.kind === kind) + 1;
  }
  tag(index: number, tag: string): void {
    this.dimensions.update((items) =>
      items.map((item, i) => (i === index ? { ...item, tag } : item)),
    );
  }
  private toTree(branches: BrowseBranch[]): TreeItem[] {
    return branches.map((branch) => ({
      ...branch,
      children: branch.children.length
        ? this.toTree(branch.children)
        : branch.nodes.map((node) => ({
            key: node.key,
            label: node.proxy.name,
            count: 1,
            children: [],
            node,
          })),
    }));
  }
  private toLeaves(nodes: InputNodeRow[]): TreeItem[] {
    return nodes.map((node) => ({
      key: node.key,
      label: node.proxy.name,
      count: 1,
      children: [],
      node,
    }));
  }
}
