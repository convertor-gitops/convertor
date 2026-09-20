import { TestBed } from '@angular/core/testing';
import { describe, expect, it } from 'vitest';
import { NodeBrowser } from './node-browser';
import { InputNodeRow } from '../board-operations';
import { Proxy } from '../../../common/model/core/profile';

const row: InputNodeRow = {
  key: 'snapshot-a',
  identity: 's1/n0',
  source: 1,
  sourceName: '来源',
  region: '香港',
  proxy: Proxy.deserialize({
    name: '香港',
    protocol: 'trojan',
    server: 'example.com',
    port: 443,
    password: 'test',
    cipher: null,
    sni: null,
    udp: null,
    tfo: null,
    skip_cert_verify: null,
    tags: [],
    extra: {},
    comment: null,
  }),
};

describe('NodeBrowser snapshot and tree state', () => {
  it('renders leaf rows directly when no browsing dimension is selected', async () => {
    const fixture = TestBed.createComponent(NodeBrowser);
    fixture.componentRef.setInput('nodes', [row]);
    fixture.detectChanges();
    await fixture.whenStable();

    expect(fixture.nativeElement.querySelectorAll('.branch-toggle')).toHaveLength(0);
    expect(fixture.nativeElement.querySelectorAll('app-ui-node-card')).toHaveLength(1);
    expect(fixture.nativeElement.textContent).not.toContain('全部节点');
    const select = fixture.nativeElement.querySelector('app-ui-node-card button.selection');
    select.click();
    fixture.detectChanges();
    expect(fixture.componentInstance.picked()).toHaveLength(1);
    expect(select.getAttribute('aria-pressed')).toBe('true');
  });

  it('appends dimensions in click order and removes a selected dimension on the next click', () => {
    const fixture = TestBed.createComponent(NodeBrowser);
    fixture.componentRef.setInput('nodes', [row]);
    const component = fixture.componentInstance;

    component.toggleDimension('protocol');
    component.toggleDimension('source');
    component.toggleDimension('has_tag');
    expect(component.dimensions()).toEqual([
      { kind: 'protocol' },
      { kind: 'source' },
      { kind: 'has_tag', tag: '' },
    ]);
    expect(component.dimensionOrder('protocol')).toBe(1);
    expect(component.dimensionOrder('has_tag')).toBe(3);

    component.toggleDimension('source');
    expect(component.dimensions()).toEqual([{ kind: 'protocol' }, { kind: 'has_tag', tag: '' }]);
    expect(component.dimensionOrder('has_tag')).toBe(2);
  });

  it('expands every nested level and removes selected rows when the snapshot changes', async () => {
    const fixture = TestBed.createComponent(NodeBrowser);
    fixture.componentRef.setInput('nodes', [row]);
    const component = fixture.componentInstance;
    component.dimensions.set([{ kind: 'source' }, { kind: 'protocol' }, { kind: 'region' }]);
    fixture.detectChanges();
    await fixture.whenStable();
    component.expandAll();
    fixture.detectChanges();
    await fixture.whenStable();
    expect(fixture.nativeElement.querySelectorAll('app-ui-node-card')).toHaveLength(1);
    component.toggle(row, true);
    expect(component.picked()).toHaveLength(1);
    fixture.componentRef.setInput('nodes', [{ ...row, key: 'snapshot-b' }]);
    fixture.detectChanges();
    await fixture.whenStable();
    expect(component.picked()).toHaveLength(0);
    expect(component.selected().size).toBe(0);
  });
});
