import { DOCUMENT } from '@angular/common';
import { Dialog, DialogModule, DialogRef } from '@angular/cdk/dialog';
import { Overlay } from '@angular/cdk/overlay';
import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  TemplateRef,
  effect,
  inject,
  input,
  model,
  output,
  signal,
  viewChild,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { UIIconButtonComponent } from '../ui-icon-button/ui-icon-button.component';
import { UIOverlayCloseReason, UIOverlayCoordinator } from '../overlay-coordinator';
import { readOverlaySize, writeOverlaySize } from '../overlay-size-storage';
import { UISplitterComponent } from '../ui-splitter/ui-splitter.component';

const LOWER_SNAP = 0.5;
const UPPER_SNAP = 0.9;
const CLOSE_THRESHOLD = 0.3;
let nextSheetId = 0;

@Component({
  selector: 'app-ui-sheet',
  imports: [DialogModule, UIIconButtonComponent, UISplitterComponent],
  templateUrl: './ui-sheet.component.html',
  styleUrl: './ui-sheet.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UISheetComponent {
  readonly open = model(false);
  readonly heading = input.required<string>();
  readonly persistenceKey = input<string>();
  readonly padding = input<'default' | 'none'>('default');
  readonly closed = output<UIOverlayCloseReason>();

  protected readonly height = signal(0);
  protected readonly headingId = `ui-sheet-heading-${++nextSheetId}`;

  private readonly panel = viewChild.required<TemplateRef<unknown>>('panel');
  private readonly dialog = inject(Dialog);
  private readonly overlay = inject(Overlay);
  private readonly coordinator = inject(UIOverlayCoordinator);
  private readonly destroyRef = inject(DestroyRef);
  private readonly document = inject(DOCUMENT);
  private readonly owner = Symbol('ui-sheet');
  private dialogRef?: DialogRef<unknown>;
  private dragStartHeight = 0;
  private snap = LOWER_SNAP;
  private viewportCleanup?: () => void;
  private closeReason?: UIOverlayCloseReason;

  constructor() {
    effect(() => {
      if (this.open()) this.show();
      else this.hide();
    });

    this.destroyRef.onDestroy(() => {
      this.viewportCleanup?.();
      this.coordinator.release(this.owner);
      this.dialogRef?.close();
    });
  }

  protected close(reason: UIOverlayCloseReason = 'button'): void {
    this.closeReason = reason;
    this.open.set(false);
  }

  protected beginResize(height: number): void {
    this.dragStartHeight = height;
  }

  protected finishPointerResize(height: number): void {
    this.setHeight(height);
    const fraction = height / this.viewportHeight();
    if (fraction < CLOSE_THRESHOLD) {
      this.close('button');
      return;
    }
    this.snap = this.nearestSnap(fraction);
    this.setHeight(this.viewportHeight() * this.snap);
    writeOverlaySize(this.storage(), this.persistenceKey(), this.height());
  }

  protected finishKeyboardResize(height: number): void {
    this.setHeight(height);
    this.snap = height / this.viewportHeight();
    writeOverlaySize(this.storage(), this.persistenceKey(), height);
  }

  protected cancelResize(height: number): void {
    this.setHeight(height);
  }

  private show(): void {
    if (this.dialogRef) return;
    this.coordinator.activate(this.owner, () => this.close('replaced'));
    const viewportHeight = this.viewportHeight();
    const stored = readOverlaySize(this.storage(), this.persistenceKey());
    this.snap = this.nearestSnap(stored ? stored / viewportHeight : LOWER_SNAP);
    this.height.set(viewportHeight * this.snap);
    const ref = this.dialog.open(this.panel(), {
      ariaLabelledBy: this.headingId,
      autoFocus: 'dialog',
      restoreFocus: true,
      hasBackdrop: true,
      backdropClass: ['cdk-overlay-dark-backdrop', 'ui-overlay-backdrop'],
      panelClass: ['ui-kit-overlay', 'ui-sheet-overlay'],
      positionStrategy: this.overlay.position().global().left('0').right('0').bottom('0'),
      width: '100vw',
      maxWidth: '100vw',
      height: `${this.height()}px`,
      maxHeight: '90dvh',
    });
    this.dialogRef = ref;
    this.watchViewport();
    ref.backdropClick.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      this.closeReason = 'backdrop';
    });
    ref.keydownEvents.pipe(takeUntilDestroyed(this.destroyRef)).subscribe((event) => {
      if (event.key === 'Escape') this.closeReason = 'escape';
    });
    ref.closed.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      if (this.dialogRef !== ref) return;
      this.dialogRef = undefined;
      this.viewportCleanup?.();
      this.viewportCleanup = undefined;
      this.coordinator.release(this.owner);
      if (this.open()) this.open.set(false);
      if (this.closeReason) this.closed.emit(this.closeReason);
      this.closeReason = undefined;
    });
  }

  private hide(): void {
    this.coordinator.release(this.owner);
    this.viewportCleanup?.();
    this.viewportCleanup = undefined;
    this.dialogRef?.close();
  }

  private nearestSnap(fraction: number): number {
    return Math.abs(fraction - LOWER_SNAP) <= Math.abs(fraction - UPPER_SNAP)
      ? LOWER_SNAP
      : UPPER_SNAP;
  }

  protected setHeight(height: number): void {
    const next = Math.min(Math.max(1, height), this.viewportHeight() * UPPER_SNAP);
    this.height.set(next);
    this.dialogRef?.overlayRef.updateSize({
      width: '100vw',
      height: `${next}px`,
      maxHeight: '90dvh',
    });
  }

  protected viewportHeight(): number {
    const view = this.document.defaultView;
    return Math.max(1, view?.visualViewport?.height ?? view?.innerHeight ?? 1);
  }

  protected maximumHeight(): number {
    return Math.round(this.viewportHeight() * UPPER_SNAP);
  }

  private watchViewport(): void {
    const viewport = this.document.defaultView?.visualViewport;
    if (!viewport) return;
    const resize = () => this.setHeight(this.viewportHeight() * this.snap);
    viewport.addEventListener('resize', resize);
    this.viewportCleanup = () => viewport.removeEventListener('resize', resize);
  }

  private storage(): Storage | undefined {
    try {
      return this.document.defaultView?.localStorage;
    } catch {
      return undefined;
    }
  }
}
