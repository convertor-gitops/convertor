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

export type UIDrawerSide = 'start' | 'end';

const INITIAL_WIDTH = 680;
const MIN_WIDTH = 320;
const MAX_WIDTH = 960;
let nextDrawerId = 0;

@Component({
  selector: 'app-ui-drawer',
  imports: [DialogModule, UIIconButtonComponent, UISplitterComponent],
  templateUrl: './ui-drawer.component.html',
  styleUrl: './ui-drawer.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UIDrawerComponent {
  readonly open = model(false);
  readonly heading = input.required<string>();
  readonly side = input<UIDrawerSide>('end');
  readonly persistenceKey = input<string>();
  readonly padding = input<'default' | 'none'>('default');
  readonly closed = output<UIOverlayCloseReason>();

  protected readonly width = signal(INITIAL_WIDTH);
  protected readonly headingId = `ui-drawer-heading-${++nextDrawerId}`;

  private readonly panel = viewChild.required<TemplateRef<unknown>>('panel');
  private readonly dialog = inject(Dialog);
  private readonly overlay = inject(Overlay);
  private readonly coordinator = inject(UIOverlayCoordinator);
  private readonly destroyRef = inject(DestroyRef);
  private readonly document = inject(DOCUMENT);
  private readonly owner = Symbol('ui-drawer');
  private dialogRef?: DialogRef<unknown>;
  private resizeStartWidth?: number;
  private closeReason?: UIOverlayCloseReason;

  constructor() {
    effect(() => {
      if (this.open()) this.show();
      else this.hide();
    });

    this.destroyRef.onDestroy(() => {
      this.coordinator.release(this.owner);
      this.dialogRef?.close();
    });
  }

  protected close(reason: UIOverlayCloseReason = 'button'): void {
    this.closeReason = reason;
    this.open.set(false);
  }

  protected beginResize(width: number): void {
    this.resizeStartWidth = width;
  }

  protected finishResize(width: number): void {
    this.setWidth(width);
    if (this.resizeStartWidth === undefined || width !== this.resizeStartWidth) this.persistWidth();
    this.resizeStartWidth = undefined;
  }

  private show(): void {
    if (this.dialogRef) return;
    this.coordinator.activate(this.owner, () => this.close('replaced'));
    this.width.set(this.clampWidth(this.storedWidth() ?? INITIAL_WIDTH));
    const position = this.overlay.position().global().top('0').bottom('0');
    if (this.side() === 'end') position.right('0');
    else position.left('0');
    const ref = this.dialog.open(this.panel(), {
      ariaLabelledBy: this.headingId,
      autoFocus: 'dialog',
      restoreFocus: true,
      hasBackdrop: true,
      backdropClass: ['cdk-overlay-dark-backdrop', 'ui-overlay-backdrop'],
      panelClass: ['ui-kit-overlay', 'ui-drawer-overlay'],
      positionStrategy: position,
      width: `${this.width()}px`,
      maxWidth: '100vw',
      height: '100dvh',
    });
    this.dialogRef = ref;
    ref.backdropClick.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      this.closeReason = 'backdrop';
    });
    ref.keydownEvents.pipe(takeUntilDestroyed(this.destroyRef)).subscribe((event) => {
      if (event.key === 'Escape') this.closeReason = 'escape';
    });
    ref.closed.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => {
      if (this.dialogRef !== ref) return;
      this.dialogRef = undefined;
      this.coordinator.release(this.owner);
      if (this.open()) this.open.set(false);
      if (this.closeReason) this.closed.emit(this.closeReason);
      this.closeReason = undefined;
    });
  }

  private hide(): void {
    this.coordinator.release(this.owner);
    this.dialogRef?.close();
  }

  protected setWidth(width: number): void {
    const next = this.clampWidth(width);
    this.width.set(next);
    this.dialogRef?.overlayRef.updateSize({ width: `${next}px`, height: '100dvh' });
  }

  private maximumWidth(): number {
    return Math.max(1, Math.min(MAX_WIDTH, this.document.defaultView?.innerWidth ?? MAX_WIDTH));
  }

  private clampWidth(width: number): number {
    const maximum = this.maximumWidth();
    return Math.min(Math.max(Math.min(MIN_WIDTH, maximum), width), maximum);
  }

  private storedWidth(): number | undefined {
    return readOverlaySize(this.storage(), this.persistenceKey());
  }

  private persistWidth(): void {
    writeOverlaySize(this.storage(), this.persistenceKey(), this.width());
  }

  private storage(): Storage | undefined {
    try {
      return this.document.defaultView?.localStorage;
    } catch {
      return undefined;
    }
  }
}
