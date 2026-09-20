import {
  UiButtonComponent,
  UiTextFieldComponent,
  UiSelectComponent,
  UiOptionComponent,
} from '../../shared/ui';
import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import {
  AllPredicate,
  AnyPredicate,
  AtomPredicate,
  BaseGroupDepthPredicate,
  BaseGroupDimensionPredicate,
  BaseGroupNamePredicate,
  BaseGroupPredicate,
  ContainsStringMatch,
  EndsWithStringMatch,
  EqualsStringMatch,
  GroupKindPredicate,
  GroupNamePredicate,
  GroupPredicate,
  HasTagDimension,
  NodeHasTagPredicate,
  NodeNamePredicate,
  NodePortPredicate,
  NodePredicate,
  NodeProtocolPredicate,
  NodeServerPredicate,
  NotPredicate,
  OneOfStringMatch,
  Predicate,
  ProtocolDimension,
  RegexStringMatch,
  RegionDimension,
  SourceDimension,
  StartsWithStringMatch,
  StringMatch,
} from '../../../common/model/core/plan';

export type PredicateDomain = 'node' | 'source-group' | 'base-group';
export type EditableAtom = NodePredicate | GroupPredicate | BaseGroupPredicate;
export type EditablePredicate = Predicate<EditableAtom>;
type PredicateOp = EditablePredicate['op'];
type StringMatchOp = StringMatch['op'];

@Component({
  selector: 'app-predicate-editor',
  imports: [
    UiButtonComponent,
    UiTextFieldComponent,
    UiSelectComponent,
    UiOptionComponent,
    FormsModule,
    MatButtonModule,
    MatCheckboxModule,
  ],
  templateUrl: './predicate-editor.html',
  styleUrl: './predicate-editor.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PredicateEditor {
  readonly predicate = input.required<EditablePredicate>();
  readonly domain = input<PredicateDomain>('node');
  readonly predicateChange = output<EditablePredicate>();

  readonly operations: readonly { value: PredicateOp; label: string }[] = [
    { value: 'all', label: '全部满足' },
    { value: 'any', label: '任一满足' },
    { value: 'not', label: '不满足' },
    { value: 'atom', label: '条件' },
  ];
  readonly matchOperations: readonly { value: StringMatchOp; label: string }[] = [
    { value: 'equals', label: '等于' },
    { value: 'one_of', label: '属于' },
    { value: 'contains', label: '包含任一' },
    { value: 'starts_with', label: '开头是' },
    { value: 'ends_with', label: '结尾是' },
    { value: 'regex', label: '正则' },
  ];

  asList(predicate: EditablePredicate): EditablePredicate[] {
    return predicate instanceof AllPredicate || predicate instanceof AnyPredicate
      ? predicate.args
      : [];
  }

  asNot(predicate: EditablePredicate): EditablePredicate {
    return predicate instanceof NotPredicate ? predicate.args : this.defaultAtom();
  }

  asAtom(predicate: EditablePredicate): EditableAtom {
    return predicate instanceof AtomPredicate ? predicate.args : this.defaultAtom().args;
  }

  setOperation(op: PredicateOp): void {
    const current = this.predicate();
    if (current.op === op) return;
    if (op === 'all') this.emit(new AllPredicate(this.childrenFor(current)));
    else if (op === 'any') this.emit(new AnyPredicate(this.childrenFor(current)));
    else if (op === 'not') this.emit(new NotPredicate(this.firstChildFor(current)));
    else this.emit(this.defaultAtom());
  }

  addChild(): void {
    const current = this.predicate();
    if (current instanceof AllPredicate)
      this.emit(new AllPredicate([...current.args, this.defaultAtom()]));
    else if (current instanceof AnyPredicate)
      this.emit(new AnyPredicate([...current.args, this.defaultAtom()]));
  }

  replaceChild(index: number, child: EditablePredicate): void {
    const current = this.predicate();
    if (!(current instanceof AllPredicate || current instanceof AnyPredicate)) return;
    const args = current.args.map((item, itemIndex) => (itemIndex === index ? child : item));
    this.emit(current instanceof AllPredicate ? new AllPredicate(args) : new AnyPredicate(args));
  }

  removeChild(index: number): void {
    const current = this.predicate();
    if (!(current instanceof AllPredicate || current instanceof AnyPredicate)) return;
    const args = current.args.filter((_, itemIndex) => itemIndex !== index);
    this.emit(current instanceof AllPredicate ? new AllPredicate(args) : new AnyPredicate(args));
  }

  replaceNot(child: EditablePredicate): void {
    this.emit(new NotPredicate(child));
  }

  atomField(atom: EditableAtom): string {
    return atom instanceof BaseGroupPredicate ? atom.kind : atom.field;
  }

  setAtomField(field: string): void {
    this.emit(new AtomPredicate(this.defaultAtomValue(field)));
  }

  isStringAtom(atom: EditableAtom): boolean {
    return (
      atom instanceof NodeNamePredicate ||
      atom instanceof NodeProtocolPredicate ||
      atom instanceof NodeServerPredicate ||
      atom instanceof GroupNamePredicate ||
      atom instanceof GroupKindPredicate ||
      atom instanceof BaseGroupNamePredicate
    );
  }

  stringTest(atom: EditableAtom): StringMatch {
    if (this.isStringAtom(atom)) return (atom as { test: StringMatch }).test;
    return new EqualsStringMatch('');
  }

  setMatchOperation(atom: EditableAtom, op: StringMatchOp): void {
    const current = this.stringTest(atom);
    const text = this.matchText(current);
    const match =
      op === 'one_of'
        ? new OneOfStringMatch(text ? [text] : [])
        : op === 'contains'
          ? new ContainsStringMatch(text)
          : op === 'starts_with'
            ? new StartsWithStringMatch(text)
            : op === 'ends_with'
              ? new EndsWithStringMatch(text)
              : op === 'regex'
                ? new RegexStringMatch(text, false)
                : new EqualsStringMatch(text);
    this.emitAtomWithTest(atom, match);
  }

  matchText(match: StringMatch): string {
    if (match instanceof RegexStringMatch) return match.pattern;
    if (match instanceof OneOfStringMatch) return match.value.join(', ');
    if (
      match instanceof EqualsStringMatch ||
      match instanceof ContainsStringMatch ||
      match instanceof StartsWithStringMatch ||
      match instanceof EndsWithStringMatch
    )
      return match.value;
    return '';
  }

  setMatchText(atom: EditableAtom, value: string): void {
    const match = this.stringTest(atom);
    let next: StringMatch;
    if (match instanceof OneOfStringMatch)
      next = new OneOfStringMatch(
        value
          .split(',')
          .map((item) => item.trim())
          .filter(Boolean),
      );
    else if (match instanceof RegexStringMatch)
      next = new RegexStringMatch(value, match.case_insensitive);
    else if (match instanceof ContainsStringMatch) next = new ContainsStringMatch(value);
    else if (match instanceof StartsWithStringMatch) next = new StartsWithStringMatch(value);
    else if (match instanceof EndsWithStringMatch) next = new EndsWithStringMatch(value);
    else next = new EqualsStringMatch(value);
    this.emitAtomWithTest(atom, next);
  }

  setRegexCaseInsensitive(atom: EditableAtom, checked: boolean): void {
    const match = this.stringTest(atom);
    if (match instanceof RegexStringMatch)
      this.emitAtomWithTest(atom, new RegexStringMatch(match.pattern, checked));
  }

  regexCaseInsensitive(match: StringMatch): boolean {
    return match instanceof RegexStringMatch && match.case_insensitive;
  }

  scalarValue(atom: EditableAtom): string | number {
    if (atom instanceof NodePortPredicate) return atom.test;
    if (atom instanceof NodeHasTagPredicate) return atom.test;
    if (atom instanceof BaseGroupDepthPredicate) return atom.depth;
    if (atom instanceof BaseGroupDimensionPredicate) return atom.value;
    return '';
  }

  setScalarValue(atom: EditableAtom, value: string): void {
    if (atom instanceof NodePortPredicate)
      this.emit(new AtomPredicate(new NodePortPredicate(this.integer(value))));
    else if (atom instanceof NodeHasTagPredicate)
      this.emit(new AtomPredicate(new NodeHasTagPredicate(value)));
    else if (atom instanceof BaseGroupDepthPredicate)
      this.emit(new AtomPredicate(new BaseGroupDepthPredicate(this.integer(value))));
    else if (atom instanceof BaseGroupDimensionPredicate)
      this.emit(new AtomPredicate(new BaseGroupDimensionPredicate(atom.dimension, value)));
  }

  dimensionKind(atom: EditableAtom): string {
    return atom instanceof BaseGroupDimensionPredicate ? atom.dimension.kind : 'region';
  }

  dimensionTag(atom: EditableAtom): string {
    return atom instanceof BaseGroupDimensionPredicate && atom.dimension instanceof HasTagDimension
      ? atom.dimension.tag
      : '';
  }

  setDimensionKind(atom: EditableAtom, kind: string): void {
    if (!(atom instanceof BaseGroupDimensionPredicate)) return;
    const dimension =
      kind === 'source'
        ? new SourceDimension()
        : kind === 'protocol'
          ? new ProtocolDimension()
          : kind === 'has_tag'
            ? new HasTagDimension('')
            : new RegionDimension();
    this.emit(new AtomPredicate(new BaseGroupDimensionPredicate(dimension, atom.value)));
  }

  setDimensionTag(atom: EditableAtom, tag: string): void {
    if (atom instanceof BaseGroupDimensionPredicate)
      this.emit(
        new AtomPredicate(new BaseGroupDimensionPredicate(new HasTagDimension(tag), atom.value)),
      );
  }

  private childrenFor(predicate: EditablePredicate): EditablePredicate[] {
    if (predicate instanceof AllPredicate || predicate instanceof AnyPredicate)
      return [...predicate.args];
    if (predicate instanceof NotPredicate) return [predicate.args];
    return [predicate];
  }

  private firstChildFor(predicate: EditablePredicate): EditablePredicate {
    if (predicate instanceof AllPredicate || predicate instanceof AnyPredicate)
      return predicate.args[0] ?? this.defaultAtom();
    if (predicate instanceof NotPredicate) return predicate.args;
    return predicate;
  }

  private defaultAtom(): AtomPredicate<EditableAtom> {
    return new AtomPredicate(
      this.defaultAtomValue(this.domain() === 'base-group' ? 'name' : 'name'),
    );
  }

  private defaultAtomValue(field: string): EditableAtom {
    if (this.domain() === 'source-group')
      return field === 'kind'
        ? new GroupKindPredicate(new EqualsStringMatch(''))
        : new GroupNamePredicate(new EqualsStringMatch(''));
    if (this.domain() === 'base-group') {
      if (field === 'depth') return new BaseGroupDepthPredicate(0);
      if (field === 'dimension') return new BaseGroupDimensionPredicate(new RegionDimension(), '');
      return new BaseGroupNamePredicate(new EqualsStringMatch(''));
    }
    if (field === 'protocol') return new NodeProtocolPredicate(new EqualsStringMatch(''));
    if (field === 'server') return new NodeServerPredicate(new EqualsStringMatch(''));
    if (field === 'port') return new NodePortPredicate(0);
    if (field === 'has_tag') return new NodeHasTagPredicate('');
    return new NodeNamePredicate(new EqualsStringMatch(''));
  }

  private emitAtomWithTest(atom: EditableAtom, test: StringMatch): void {
    const next =
      atom instanceof NodeProtocolPredicate
        ? new NodeProtocolPredicate(test)
        : atom instanceof NodeServerPredicate
          ? new NodeServerPredicate(test)
          : atom instanceof GroupNamePredicate
            ? new GroupNamePredicate(test)
            : atom instanceof GroupKindPredicate
              ? new GroupKindPredicate(test)
              : atom instanceof BaseGroupNamePredicate
                ? new BaseGroupNamePredicate(test)
                : new NodeNamePredicate(test);
    this.emit(new AtomPredicate(next));
  }

  private integer(value: string): number {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  private emit(predicate: EditablePredicate): void {
    this.predicateChange.emit(predicate);
  }
}
