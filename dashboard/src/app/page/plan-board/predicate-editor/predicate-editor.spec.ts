import { ComponentRef } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { describe, expect, it } from 'vitest';
import {
  AllPredicate,
  AtomPredicate,
  NodeNamePredicate,
  NodePortPredicate,
  NodePredicate,
  NotPredicate,
  RegexStringMatch,
} from '../../../common/model/core/plan';
import { EditablePredicate, PredicateEditor } from './predicate-editor';

describe('PredicateEditor', () => {
  function create(predicate: EditablePredicate): ComponentRef<PredicateEditor> {
    const fixture = TestBed.createComponent(PredicateEditor);
    fixture.componentRef.setInput('predicate', predicate);
    fixture.detectChanges();
    return fixture.componentRef;
  }

  it('emits a new atom without mutating the input', () => {
    const original = new AtomPredicate(new NodePortPredicate(80));
    const component = create(original).instance;
    let changed: EditablePredicate | undefined;
    component.predicateChange.subscribe((value) => (changed = value));

    component.setScalarValue(original.args, '443');

    expect((original.args as NodePortPredicate).test).toBe(80);
    expect(changed).toBeInstanceOf(AtomPredicate);
    expect((changed as AtomPredicate<NodePortPredicate>).args.test).toBe(443);
  });

  it('replaces a nested condition while retaining the surrounding tree', () => {
    const first = new AtomPredicate(new NodePortPredicate(80));
    const nested = new NotPredicate(
      new AtomPredicate(new NodeNamePredicate(new RegexStringMatch('old', true))),
    );
    const original = new AllPredicate([first, nested]);
    const component = create(original).instance;
    let changed: EditablePredicate | undefined;
    component.predicateChange.subscribe((value) => (changed = value));
    const replacement = new AtomPredicate(new NodePortPredicate(443));

    component.replaceChild(1, replacement);

    expect(changed).toBeInstanceOf(AllPredicate);
    expect((changed as AllPredicate<NodePredicate>).args[0]).toBe(first);
    expect((changed as AllPredicate<NodePredicate>).args[1]).toBe(replacement);
    expect(original.args[1]).toBe(nested);
  });

  it('preserves regex flags while editing its pattern', () => {
    const atom = new NodeNamePredicate(new RegexStringMatch('HK-.*', true));
    const original = new AtomPredicate(atom);
    const component = create(original).instance;
    let changed: EditablePredicate | undefined;
    component.predicateChange.subscribe((value) => (changed = value));

    component.setMatchText(atom, 'JP-.*');

    const match = (changed as AtomPredicate<NodeNamePredicate>).args.test;
    expect(match).toBeInstanceOf(RegexStringMatch);
    expect((match as RegexStringMatch).pattern).toBe('JP-.*');
    expect((match as RegexStringMatch).case_insensitive).toBe(true);
    expect((atom.test as RegexStringMatch).pattern).toBe('HK-.*');
  });
});
