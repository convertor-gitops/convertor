import {
  ChangeDetectionStrategy,
  Component,
  Directive,
  TemplateRef,
  contentChild,
  inject,
  input,
  viewChild,
} from '@angular/core';
import { NgTemplateOutlet } from '@angular/common';
import { MatTree, MatTreeModule } from '@angular/material/tree';
import { MatButtonModule } from '@angular/material/button';
import { UIIconComponent } from '../ui-icon/ui-icon.component';
export interface UITreeItem {
  key: string;
  label: string;
  count: number;
  children: UITreeItem[];
}
@Directive({ selector: 'ng-template[uiTreeLeaf]' })
export class UITreeLeafDirective {
  readonly template = inject(TemplateRef);
}
@Component({
  selector: 'app-ui-tree',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [MatTreeModule, MatButtonModule, NgTemplateOutlet, UIIconComponent],
  host: { class: 'ui-kit' },
  template: `<mat-tree
    #tree
    [dataSource]="nodes()"
    [childrenAccessor]="children"
    [trackBy]="track"
    [expansionKey]="key"
  >
    <mat-tree-node *matTreeNodeDef="let item" matTreeNodePadding [matTreeNodePaddingIndent]="12">
      <ng-container
        [ngTemplateOutlet]="leaf()?.template ?? null"
        [ngTemplateOutletContext]="{ $implicit: item }"
      />
    </mat-tree-node>
    <mat-tree-node
      *matTreeNodeDef="let item; when: branch"
      matTreeNodePadding
      [matTreeNodePaddingIndent]="12"
      [isExpandable]="true"
    >
      <button
        mat-button
        matTreeNodeToggle
        [attr.aria-label]="'展开或折叠 ' + item.label"
        [attr.aria-expanded]="tree.isExpanded(item)"
      >
        <span>{{ item.label }}</span
        ><small>{{ item.count }}</small
        ><app-ui-icon name="chevron" [class.expanded]="tree.isExpanded(item)" />
      </button> </mat-tree-node
  ></mat-tree>`,
  styles: [
    `
      :host {
        display: block;
        min-width: 0;
      }
      mat-tree {
        background: transparent;
      }
      mat-tree-node {
        min-height: 32px;
        min-width: 0;
        box-sizing: border-box;
        max-width: 100%;
        padding-block: 4px;
      }
      button {
        width: 100%;
        min-width: 0;
        text-align: left;
        padding: 4px 8px;
        --mat-button-text-container-shape: var(--ui-radius-control);
      }
      button span {
        white-space: normal;
        overflow-wrap: anywhere;
      }
      small {
        margin-inline: 8px;
        color: var(--ui-muted);
      }
      app-ui-icon {
        width: 14px;
        transform: rotate(-90deg);
      }
      app-ui-icon.expanded {
        transform: rotate(0);
      }
      ::ng-deep app-ui-tree .mdc-button__label {
        display: flex;
        align-items: center;
        width: 100%;
        gap: 4px;
      }
      ::ng-deep app-ui-tree .mdc-button__label > span {
        flex: 1;
      }
    `,
  ],
})
export class UITreeComponent {
  readonly nodes = input<UITreeItem[]>([]);
  readonly leaf = contentChild(UITreeLeafDirective);
  readonly tree = viewChild(MatTree<UITreeItem>);
  readonly children = (item: UITreeItem) => item.children;
  readonly key = (item: UITreeItem) => item.key;
  readonly track = (_: number, item: UITreeItem) => item.key;
  readonly branch = (_: number, item: UITreeItem) => item.children.length > 0;
  expand(item: UITreeItem) {
    this.tree()?.expand(item);
  }
  isExpanded(item: UITreeItem) {
    return this.tree()?.isExpanded(item) ?? false;
  }
  collapseAll() {
    this.tree()?.collapseAll();
  }
  expandAll() {
    this.tree()?.expandAll();
  }
}
