import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import * as P from '../../../common/model/core/plan';
import { PredicateEditor } from '../predicate-editor/predicate-editor';
@Component({
  selector: 'app-member-editor',
  imports: [FormsModule, MatFormFieldModule, MatSelectModule, PredicateEditor],
  templateUrl: './member-editor.html',
  styles: `
    :host {
      display: grid;
      gap: 10px;
    }
    .row {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
    }
    mat-form-field {
      flex: 1 1 140px;
      min-width: 0;
      max-width: 100%;
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MemberEditor {
  readonly selector = input.required<P.MemberSelector>();
  readonly sources = input.required<P.Source[]>();
  readonly groups = input.required<P.CustomGroup[]>();
  readonly policies = input.required<P.GroupingPolicy[]>();
  readonly owner = input.required<number>();
  readonly selectorChange = output<P.MemberSelector>();
  readonly kinds = [
    ['nodes', '按条件选节点'],
    ['nodes_from_groups', '从原始组展开节点'],
    ['import_groups', '保留原始组'],
    ['base_groups', '引用自动组'],
    ['group', '引用节点组'],
    ['builtin', '内置动作'],
  ];
  nodeSelection(): P.NodeSelection | null {
    const s = this.selector();
    return s instanceof P.NodesMemberSelector ? s.selection : null;
  }
  sourceSelection(): P.SourceGroupSelection | null {
    const s = this.selector();
    return s instanceof P.NodesFromGroupsMemberSelector || s instanceof P.ImportGroupsMemberSelector
      ? s.selection
      : null;
  }
  baseSelection(): P.BaseGroupSelection | null {
    const s = this.selector();
    return s instanceof P.BaseGroupsMemberSelector ? s.selection : null;
  }
  groupValue(): number {
    return (this.selector() as P.GroupMemberSelector).group;
  }
  builtinValue(): string {
    return (this.selector() as P.BuiltinMemberSelector).builtin;
  }
  depth(): string {
    return (this.selector() as P.NodesFromGroupsMemberSelector).depth;
  }
  changeKind(kind: string): void {
    const source = this.sources()[0]?.id ?? 0;
    let next: P.MemberSelector;
    switch (kind) {
      case 'nodes_from_groups':
        next = new P.NodesFromGroupsMemberSelector(
          new P.SourceGroupSelection(source, new P.AllPredicate([])),
          'Recursive',
        );
        break;
      case 'import_groups':
        next = new P.ImportGroupsMemberSelector(
          new P.SourceGroupSelection(source, new P.AllPredicate([])),
        );
        break;
      case 'base_groups':
        next = new P.BaseGroupsMemberSelector(
          new P.BaseGroupSelection(this.policies()[0]?.id ?? 0, 'Roots', new P.AllPredicate([])),
        );
        break;
      case 'group':
        next = new P.GroupMemberSelector(
          this.groups().find((group) => group.id !== this.owner())?.id ?? 0,
        );
        break;
      case 'builtin':
        next = new P.BuiltinMemberSelector('Direct');
        break;
      default:
        next = new P.NodesMemberSelector(
          new P.NodeSelection(
            source,
            new P.AtomPredicate(new P.NodeNamePredicate(new P.EqualsStringMatch(''))),
          ),
        );
    }
    this.selectorChange.emit(next);
  }
  changeSource(source: number): void {
    const s = P.MemberSelector.deserialize(this.selector().serialize());
    if (
      s instanceof P.NodesMemberSelector ||
      s instanceof P.NodesFromGroupsMemberSelector ||
      s instanceof P.ImportGroupsMemberSelector
    )
      s.selection.source = source;
    this.selectorChange.emit(s);
  }
  changePredicate(
    predicate: P.Predicate<P.NodePredicate | P.GroupPredicate | P.BaseGroupPredicate>,
  ): void {
    const s = P.MemberSelector.deserialize(this.selector().serialize());
    if (s instanceof P.NodesMemberSelector)
      s.selection.predicate = P.Predicate.deserialize(
        predicate.serialize(),
        P.NodePredicate.deserialize,
      );
    else if (
      s instanceof P.NodesFromGroupsMemberSelector ||
      s instanceof P.ImportGroupsMemberSelector
    )
      s.selection.predicate = P.Predicate.deserialize(
        predicate.serialize(),
        P.GroupPredicate.deserialize,
      );
    else if (s instanceof P.BaseGroupsMemberSelector)
      s.selection.predicate = P.Predicate.deserialize(
        predicate.serialize(),
        P.BaseGroupPredicate.deserialize,
      );
    this.selectorChange.emit(s);
  }
  changeBase(field: 'policy' | 'scope', value: number | P.GroupScope): void {
    const s = P.MemberSelector.deserialize(this.selector().serialize());
    if (s instanceof P.BaseGroupsMemberSelector) {
      if (field === 'policy') s.selection.policy = Number(value);
      else s.selection.scope = value as P.GroupScope;
      this.selectorChange.emit(s);
    }
  }
  changeDepth(depth: 'Direct' | 'Recursive'): void {
    const s = P.MemberSelector.deserialize(this.selector().serialize());
    if (s instanceof P.NodesFromGroupsMemberSelector) {
      s.depth = depth;
      this.selectorChange.emit(s);
    }
  }
  changeGroup(value: number): void {
    this.selectorChange.emit(new P.GroupMemberSelector(value));
  }
  changeBuiltin(value: P.BuiltinValue): void {
    this.selectorChange.emit(new P.BuiltinMemberSelector(value));
  }
}
