import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  effect,
  inject,
  input,
  output,
  signal,
} from '@angular/core';

export type UISplitterOrientation = 'vertical' | 'horizontal';
export type UISplitterAppearance = 'line' | 'handle';
export type UISplitterCompletionReason = 'pointer' | 'keyboard' | 'reset' | 'cancel';
export interface UISplitterCompletion {
  value: number;
  reason: UISplitterCompletionReason;
}

export function constrainSplitterValue(value: number, min: number, max: number): number {
  const safeMin = Number.isFinite(min) ? min : 0;
  const safeMax = Number.isFinite(max) ? Math.max(safeMin, max) : safeMin;
  if (!Number.isFinite(value)) return safeMin;
  return Math.min(safeMax, Math.max(safeMin, value));
}

@Component({
  selector: 'app-ui-splitter',
  templateUrl: './ui-splitter.component.html',
  styleUrl: './ui-splitter.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '[attr.data-orientation]': 'orientation()',
    '[attr.data-appearance]': 'appearance()',
    '[class.is-resizing]': 'dragging()',
    '[class.is-collapsed]': 'collapsed()',
  },
})
export class UISplitterComponent {
  readonly orientation = input<UISplitterOrientation>('vertical');
  readonly direction = input<1 | -1>(1);
  readonly appearance = input<UISplitterAppearance>('line');
  readonly value = input(0);
  readonly min = input(0);
  readonly keyboardMin = input<number | undefined>(undefined);
  readonly max = input(Number.MAX_SAFE_INTEGER);
  readonly defaultValue = input(0);
  readonly label = input('调整面板大小');
  readonly controls = input('');
  readonly collapsed = input(false);

  readonly valueChange = output<number>();
  readonly resizeStart = output<number>();
  readonly resizeEnd = output<number>();
  readonly pointerResizeEnd = output<number>();
  readonly keyboardResizeEnd = output<number>();
  readonly resizeCanceled = output<number>();
  readonly completed = output<UISplitterCompletion>();
  readonly collapsedChange = output<boolean>();

  protected readonly renderedValue = signal(0);
  protected readonly dragging = signal(false);

  private readonly destroyRef = inject(DestroyRef);
  private activePointerId: number | null = null;
  private startCoordinate = 0;
  private startValue = 0;
  private pendingValue: number | null = null;
  private frameId: number | null = null;
  private handle: HTMLElement | null = null;
  private previousUserSelect = '';
  private previousCursor = '';

  constructor() {
    effect(() => {
      const next = constrainSplitterValue(this.value(), this.min(), this.max());
      if (!this.dragging()) this.renderedValue.set(next);
    });
    this.destroyRef.onDestroy(() => this.finishPointer(false, true));
  }

  protected pointerDown(event: PointerEvent): void {
    if (event.button !== 0 || this.activePointerId !== null) return;
    const target = event.currentTarget as HTMLElement;
    this.activePointerId = event.pointerId;
    this.handle = target;
    target.focus({ preventScroll: true });
    this.startCoordinate = this.coordinate(event);
    this.startValue = constrainSplitterValue(this.value(), this.min(), this.max());
    this.renderedValue.set(this.startValue);
    this.dragging.set(true);
    target.setPointerCapture(event.pointerId);
    this.lockDocumentInteraction();
    window.addEventListener('blur', this.windowBlur);
    this.resizeStart.emit(this.startValue);
    event.preventDefault();
  }

  protected pointerMove(event: PointerEvent): void {
    if (event.pointerId !== this.activePointerId) return;
    const delta = (this.coordinate(event) - this.startCoordinate) * this.direction();
    this.pendingValue = constrainSplitterValue(this.startValue + delta, this.min(), this.max());
    if (this.frameId === null) {
      this.frameId = requestAnimationFrame(() => {
        this.frameId = null;
        if (this.pendingValue === null) return;
        const next = this.pendingValue;
        this.pendingValue = null;
        this.renderedValue.set(next);
        this.valueChange.emit(next);
      });
    }
  }

  protected pointerUp(event: PointerEvent): void {
    if (event.pointerId === this.activePointerId) this.finishPointer(true, false);
  }

  protected pointerCancel(event: PointerEvent): void {
    if (event.pointerId === this.activePointerId) this.finishPointer(false, true);
  }

  protected lostPointerCapture(event: PointerEvent): void {
    if (event.pointerId === this.activePointerId) this.finishPointer(false, true);
  }

  protected keyDown(event: KeyboardEvent): void {
    if (event.key === 'Escape' && this.dragging()) {
      event.preventDefault();
      event.stopPropagation();
      this.finishPointer(false, true);
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.key === 'Home') {
      event.preventDefault();
      this.reset();
      return;
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      this.collapsedChange.emit(!this.collapsed());
      return;
    }

    const step = event.shiftKey ? 32 : 8;
    let next: number | null = null;
    switch (event.key) {
      case 'Home':
        next = this.effectiveKeyboardMin();
        break;
      case 'End':
        next = this.max();
        break;
      case 'ArrowLeft':
        if (this.orientation() === 'vertical')
          next = this.renderedValue() - step * this.direction();
        break;
      case 'ArrowRight':
        if (this.orientation() === 'vertical')
          next = this.renderedValue() + step * this.direction();
        break;
      case 'ArrowUp':
        if (this.orientation() === 'horizontal')
          next = this.renderedValue() - step * this.direction();
        break;
      case 'ArrowDown':
        if (this.orientation() === 'horizontal')
          next = this.renderedValue() + step * this.direction();
        break;
    }
    if (next === null) return;
    event.preventDefault();
    const constrained = constrainSplitterValue(next, this.effectiveKeyboardMin(), this.max());
    this.renderedValue.set(constrained);
    this.valueChange.emit(constrained);
    this.resizeEnd.emit(constrained);
    this.keyboardResizeEnd.emit(constrained);
    this.completed.emit({ value: constrained, reason: 'keyboard' });
  }

  protected reset(): void {
    const next = constrainSplitterValue(this.defaultValue(), this.min(), this.max());
    this.renderedValue.set(next);
    if (this.collapsed()) this.collapsedChange.emit(false);
    this.valueChange.emit(next);
    this.resizeEnd.emit(next);
    this.keyboardResizeEnd.emit(next);
    this.completed.emit({ value: next, reason: 'reset' });
  }

  private coordinate(event: PointerEvent): number {
    return this.orientation() === 'vertical' ? event.clientX : event.clientY;
  }

  private effectiveKeyboardMin(): number {
    return Math.min(this.max(), Math.max(this.min(), this.keyboardMin() ?? this.min()));
  }

  private finishPointer(commit: boolean, restore: boolean): void {
    if (this.activePointerId === null) return;
    if (this.frameId !== null) {
      cancelAnimationFrame(this.frameId);
      this.frameId = null;
    }
    if (commit && this.pendingValue !== null) {
      this.renderedValue.set(this.pendingValue);
      this.valueChange.emit(this.pendingValue);
    }
    const pointerId = this.activePointerId;
    this.activePointerId = null;
    this.dragging.set(false);
    this.restoreDocumentInteraction();
    if (this.handle?.hasPointerCapture(pointerId)) this.handle.releasePointerCapture(pointerId);
    this.handle = null;
    window.removeEventListener('blur', this.windowBlur);
    const finalValue = restore
      ? this.startValue
      : constrainSplitterValue(this.renderedValue(), this.min(), this.max());
    if (restore) {
      this.renderedValue.set(finalValue);
      this.valueChange.emit(finalValue);
    }
    this.pendingValue = null;
    if (commit) {
      this.resizeEnd.emit(finalValue);
      this.pointerResizeEnd.emit(finalValue);
      this.completed.emit({ value: finalValue, reason: 'pointer' });
    }
    if (restore) {
      this.resizeEnd.emit(finalValue);
      this.resizeCanceled.emit(finalValue);
      this.completed.emit({ value: finalValue, reason: 'cancel' });
    }
  }

  private lockDocumentInteraction(): void {
    this.previousUserSelect = document.documentElement.style.userSelect;
    this.previousCursor = document.documentElement.style.cursor;
    document.documentElement.style.userSelect = 'none';
    document.documentElement.style.cursor =
      this.orientation() === 'vertical' ? 'col-resize' : 'row-resize';
  }

  private restoreDocumentInteraction(): void {
    document.documentElement.style.userSelect = this.previousUserSelect;
    document.documentElement.style.cursor = this.previousCursor;
  }

  private readonly windowBlur = () => this.finishPointer(false, true);
}
