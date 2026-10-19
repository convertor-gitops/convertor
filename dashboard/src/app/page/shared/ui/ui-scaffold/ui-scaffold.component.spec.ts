import { WritableSignal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { describe, expect, it, vi } from 'vitest';
import { resolveScaffoldHorizontalLayout, UIScaffoldComponent } from './ui-scaffold.component';

const constraints = {
  hasLeft: true,
  hasRight: true,
  leftOpen: true,
  rightOpen: true,
  leftMin: 160,
  leftMax: 720,
  rightMin: 200,
  rightMax: 720,
  mainMin: 320,
};

describe('resolveScaffoldHorizontalLayout', () => {
  it('keeps both preferred widths when the container can accommodate them', () => {
    expect(resolveScaffoldHorizontalLayout(1200, 300, 360, constraints)).toEqual({
      leftOverlay: false,
      rightOverlay: false,
      left: 300,
      right: 360,
    });
  });

  it('moves the right side to an overlay before the left side', () => {
    expect(resolveScaffoldHorizontalLayout(650, 300, 360, constraints)).toMatchObject({
      leftOverlay: false,
      rightOverlay: true,
      left: 300,
    });
  });

  it('moves both sides out of the main flow at narrow widths', () => {
    expect(resolveScaffoldHorizontalLayout(450, 300, 360, constraints)).toMatchObject({
      leftOverlay: true,
      rightOverlay: true,
    });
  });

  it('clamps effective widths without changing the supplied preferences', () => {
    const result = resolveScaffoldHorizontalLayout(1500, 800, 100, constraints);
    expect(result.left).toBe(720);
    expect(result.right).toBe(200);
  });

  it('holds the rendered opposite dock stable throughout pointer and keyboard-sized steps', () => {
    const pointerStep = resolveScaffoldHorizontalLayout(1018, 250, 400, {
      ...constraints,
      preserve: 'right',
    });
    expect(pointerStep.right).toBe(400);

    const keyboardStep = resolveScaffoldHorizontalLayout(1018, pointerStep.left + 8, 400, {
      ...constraints,
      preserve: 'right',
    });
    expect(keyboardStep.right).toBe(400);
    expect(keyboardStep.left).toBe(pointerStep.left + 8);
  });

  it('keeps breakpoint modes stable while hidden docks stop reserving width', () => {
    const visible = resolveScaffoldHorizontalLayout(800, 600, 400, constraints);
    const leftHidden = resolveScaffoldHorizontalLayout(800, 600, 400, {
      ...constraints,
      leftVisible: false,
    });
    expect(leftHidden.leftOverlay).toBe(visible.leftOverlay);
    expect(leftHidden.rightOverlay).toBe(visible.rightOverlay);
    expect(leftHidden.right).toBe(400);
    expect(visible.right).toBe(200);
  });

  it('cancels a bottom resize without replacing lateral preferences', () => {
    vi.stubGlobal(
      'ResizeObserver',
      class {
        observe(): void {}
        disconnect(): void {}
      },
    );
    const fixture = TestBed.createComponent(UIScaffoldComponent);
    fixture.componentRef.setInput('title', 'Test');
    const scaffold = fixture.componentInstance as unknown as {
      preferredLeft: WritableSignal<number>;
      preferredRight: WritableSignal<number>;
      preferredBottom: WritableSignal<number>;
      startBottom(value: number): void;
      resizeBottom(value: number): void;
      completeResize(region: 'bottom', completion: { value: number; reason: 'cancel' }): void;
    };
    scaffold.preferredLeft.set(333);
    scaffold.preferredRight.set(444);
    scaffold.preferredBottom.set(222);
    scaffold.startBottom(222);
    scaffold.resizeBottom(150);
    scaffold.completeResize('bottom', { value: 222, reason: 'cancel' });
    expect(scaffold.preferredLeft()).toBe(333);
    expect(scaffold.preferredRight()).toBe(444);
    expect(scaffold.preferredBottom()).toBe(222);
    fixture.destroy();
    vi.unstubAllGlobals();
  });
});
