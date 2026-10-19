import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  ElementRef,
  afterNextRender,
  computed,
  effect,
  inject,
  input,
  signal,
} from '@angular/core';
import { MatSidenavModule } from '@angular/material/sidenav';
import { UIIconButtonComponent } from '../ui-icon-button/ui-icon-button.component';
import { UIOverlayCoordinator } from '../overlay-coordinator';
import {
  UISplitterCompletion,
  UISplitterComponent,
  constrainSplitterValue,
} from '../ui-splitter/ui-splitter.component';

const STORAGE_VERSION = 1;
const MAIN_MIN_HEIGHT = 240;

export interface UIScaffoldHorizontalLayout {
  leftOverlay: boolean;
  rightOverlay: boolean;
  left: number;
  right: number;
}

export interface UIScaffoldHorizontalConstraints {
  hasLeft: boolean;
  hasRight: boolean;
  leftOpen: boolean;
  rightOpen: boolean;
  leftVisible?: boolean;
  rightVisible?: boolean;
  leftMin: number;
  leftMax: number;
  rightMin: number;
  rightMax: number;
  mainMin: number;
  splitterSize?: number;
  preserve?: 'left' | 'right';
}

export function resolveScaffoldHorizontalLayout(
  containerWidth: number,
  preferredLeft: number,
  preferredRight: number,
  constraints: UIScaffoldHorizontalConstraints,
): UIScaffoldHorizontalLayout {
  const width = Math.max(0, containerWidth);
  const splitter = constraints.splitterSize ?? 4;
  const leftActive = constraints.hasLeft && constraints.leftOpen;
  const rightActive = constraints.hasRight && constraints.rightOpen;
  const leftVisible = constraints.hasLeft && (constraints.leftVisible ?? constraints.leftOpen);
  const rightVisible = constraints.hasRight && (constraints.rightVisible ?? constraints.rightOpen);
  const rightOverlay =
    rightActive &&
    width <
      constraints.mainMin +
        (leftActive ? constraints.leftMin + splitter : 0) +
        constraints.rightMin +
        splitter;
  const leftOverlay = leftActive && width < constraints.mainMin + constraints.leftMin + splitter;

  let left = constrainSplitterValue(preferredLeft, constraints.leftMin, constraints.leftMax);
  let right = constrainSplitterValue(preferredRight, constraints.rightMin, constraints.rightMax);
  const inlineLeft = leftVisible && !leftOverlay;
  const inlineRight = rightVisible && !rightOverlay;
  const availableForSides = Math.max(
    0,
    width - constraints.mainMin - (inlineLeft ? splitter : 0) - (inlineRight ? splitter : 0),
  );
  let overflow = (inlineLeft ? left : 0) + (inlineRight ? right : 0) - availableForSides;
  const reduceRight = (): void => {
    if (overflow <= 0 || !inlineRight) return;
    const reduction = Math.min(overflow, Math.max(0, right - constraints.rightMin));
    right -= reduction;
    overflow -= reduction;
  };
  const reduceLeft = (): void => {
    if (overflow <= 0 || !inlineLeft) return;
    const reduction = Math.min(overflow, Math.max(0, left - constraints.leftMin));
    left -= reduction;
    overflow -= reduction;
  };
  if (constraints.preserve === 'right') {
    reduceLeft();
    reduceRight();
  } else {
    reduceRight();
    reduceLeft();
  }

  return { leftOverlay, rightOverlay, left, right };
}

interface StoredLayout {
  version: typeof STORAGE_VERSION;
  left: number;
  right: number;
  bottom: number;
  leftCollapsed: boolean;
  rightCollapsed: boolean;
  bottomCollapsed: boolean;
}

@Component({
  selector: 'app-ui-scaffold',
  imports: [MatSidenavModule, UIIconButtonComponent, UISplitterComponent],
  templateUrl: './ui-scaffold.component.html',
  styleUrl: './ui-scaffold.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit' },
})
export class UIScaffoldComponent {
  readonly title = input.required<string>();
  readonly subtitle = input('');
  readonly persistenceKey = input('');
  readonly hasLeft = input(false);
  readonly hasRight = input(false);
  readonly hasBottom = input(false);
  readonly leftWidth = input(256);
  readonly leftMin = input(160);
  readonly leftMax = input(720);
  readonly rightWidth = input(320);
  readonly rightMin = input(200);
  readonly rightMax = input(720);
  readonly mainMin = input(320);
  readonly bottomHeight = input(240);
  readonly bottomMin = input(120);
  readonly bottomMax = input(Number.MAX_SAFE_INTEGER);

  protected readonly preferredLeft = signal(256);
  protected readonly preferredRight = signal(320);
  protected readonly preferredBottom = signal(240);
  protected readonly leftDockCollapsed = signal(false);
  protected readonly rightDockCollapsed = signal(false);
  protected readonly leftOverlayOpen = signal(false);
  protected readonly rightOverlayOpen = signal(false);
  protected readonly bottomCollapsed = signal(false);
  protected readonly containerWidth = signal(0);
  protected readonly workspaceHeight = signal(0);
  private readonly activeResize = signal<'left' | 'right' | 'bottom' | null>(null);
  protected readonly horizontal = computed(() =>
    resolveScaffoldHorizontalLayout(
      this.containerWidth(),
      this.preferredLeft(),
      this.preferredRight(),
      {
        hasLeft: this.hasLeft(),
        hasRight: this.hasRight(),
        // Breakpoints describe the available slots, independent of whether a dock is currently hidden.
        // This keeps a hidden sidebar in overlay mode so its header toggle opens it on narrow screens.
        leftOpen: this.hasLeft(),
        rightOpen: this.hasRight(),
        leftVisible: !this.leftDockCollapsed(),
        rightVisible: !this.rightDockCollapsed(),
        leftMin: this.leftMin(),
        leftMax: this.leftMax(),
        rightMin: this.rightMin(),
        rightMax: this.rightMax(),
        mainMin: this.mainMin(),
        preserve:
          this.activeResize() === 'left'
            ? 'right'
            : this.activeResize() === 'right'
              ? 'left'
              : undefined,
      },
    ),
  );
  protected readonly effectiveBottomMax = computed(() =>
    Math.max(0, Math.min(this.bottomMax(), this.workspaceHeight() - MAIN_MIN_HEIGHT)),
  );
  protected readonly effectiveBottomMin = computed(() =>
    Math.min(this.bottomMin(), this.effectiveBottomMax()),
  );
  protected readonly effectiveBottom = computed(() =>
    constrainSplitterValue(
      this.preferredBottom(),
      this.effectiveBottomMin(),
      this.effectiveBottomMax(),
    ),
  );
  protected readonly leftOpened = computed(() =>
    this.horizontal().leftOverlay ? this.leftOverlayOpen() : !this.leftDockCollapsed(),
  );
  protected readonly rightOpened = computed(() =>
    this.horizontal().rightOverlay ? this.rightOverlayOpen() : !this.rightDockCollapsed(),
  );
  protected readonly effectiveLeftMax = computed(() => {
    if (this.horizontal().leftOverlay) return this.leftMax();
    const right =
      this.rightOpened() && !this.horizontal().rightOverlay ? this.horizontal().right : 0;
    const splitters = (this.rightOpened() && !this.horizontal().rightOverlay ? 4 : 0) + 4;
    return Math.max(
      this.leftMin(),
      Math.min(this.leftMax(), this.containerWidth() - this.mainMin() - right - splitters),
    );
  });
  protected readonly effectiveRightMax = computed(() => {
    if (this.horizontal().rightOverlay) return this.rightMax();
    const left = this.leftOpened() && !this.horizontal().leftOverlay ? this.horizontal().left : 0;
    const splitters = (this.leftOpened() && !this.horizontal().leftOverlay ? 4 : 0) + 4;
    return Math.max(
      this.rightMin(),
      Math.min(this.rightMax(), this.containerWidth() - this.mainMin() - left - splitters),
    );
  });
  protected readonly ids = computed(() => {
    const prefix = (this.persistenceKey() || 'ui-scaffold').replace(/[^a-zA-Z0-9_-]/g, '-');
    return {
      left: `${prefix}-left`,
      main: `${prefix}-main`,
      right: `${prefix}-right`,
      bottom: `${prefix}-bottom`,
    };
  });
  protected readonly hasOverlay = computed(
    () => this.horizontal().leftOverlay || this.horizontal().rightOverlay,
  );

  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private readonly destroyRef = inject(DestroyRef);
  private readonly overlayCoordinator = inject(UIOverlayCoordinator);
  private readonly overlayOwner = Symbol('ui-scaffold');
  private resizeObserver: ResizeObserver | null = null;
  private preferredLeftBeforeDrag = 0;
  private preferredRightBeforeDrag = 0;
  private preferredBottomBeforeDrag = 0;
  private previousLeftOverlay = false;
  private previousRightOverlay = false;

  constructor() {
    effect(() => {
      const { leftOverlay, rightOverlay } = this.horizontal();
      if (leftOverlay && !this.previousLeftOverlay) this.leftOverlayOpen.set(false);
      if (rightOverlay && !this.previousRightOverlay) this.rightOverlayOpen.set(false);
      this.previousLeftOverlay = leftOverlay;
      this.previousRightOverlay = rightOverlay;
    });
    effect(() => {
      if (this.leftOverlayOpen() || this.rightOverlayOpen()) {
        this.overlayCoordinator.activate(this.overlayOwner, () => {
          this.leftOverlayOpen.set(false);
          this.rightOverlayOpen.set(false);
        });
      } else {
        this.overlayCoordinator.release(this.overlayOwner);
      }
    });
    afterNextRender(() => {
      this.preferredLeft.set(this.leftWidth());
      this.preferredRight.set(this.rightWidth());
      this.preferredBottom.set(this.bottomHeight());
      this.restore();
      const workspace = this.host.nativeElement.querySelector<HTMLElement>('.workspace');
      this.resizeObserver = new ResizeObserver(() => {
        this.containerWidth.set(this.host.nativeElement.clientWidth);
        this.workspaceHeight.set(workspace?.clientHeight ?? this.host.nativeElement.clientHeight);
      });
      this.resizeObserver.observe(this.host.nativeElement);
      if (workspace) this.resizeObserver.observe(workspace);
      this.containerWidth.set(this.host.nativeElement.clientWidth);
      this.workspaceHeight.set(workspace?.clientHeight ?? this.host.nativeElement.clientHeight);
      window.addEventListener('storage', this.storageListener);
    });
    this.destroyRef.onDestroy(() => {
      this.overlayCoordinator.release(this.overlayOwner);
      this.resizeObserver?.disconnect();
      if (typeof window !== 'undefined')
        window.removeEventListener('storage', this.storageListener);
    });
  }

  toggleLeft(): void {
    if (this.horizontal().leftOverlay) {
      const next = !this.leftOverlayOpen();
      this.rightOverlayOpen.set(false);
      this.leftOverlayOpen.set(next);
      return;
    }
    this.leftDockCollapsed.update((value) => !value);
    this.persist();
  }

  toggleRight(): void {
    if (this.horizontal().rightOverlay) {
      const next = !this.rightOverlayOpen();
      this.leftOverlayOpen.set(false);
      this.rightOverlayOpen.set(next);
      return;
    }
    this.rightDockCollapsed.update((value) => !value);
    this.persist();
  }

  toggleBottom(): void {
    this.bottomCollapsed.update((value) => !value);
    this.persist();
  }

  resetLayout(): void {
    this.preferredLeft.set(this.leftWidth());
    this.preferredRight.set(this.rightWidth());
    this.preferredBottom.set(this.bottomHeight());
    this.leftDockCollapsed.set(false);
    this.rightDockCollapsed.set(false);
    this.leftOverlayOpen.set(false);
    this.rightOverlayOpen.set(false);
    this.bottomCollapsed.set(false);
    try {
      localStorage.removeItem(this.storageKey());
    } catch {
      // Storage can be unavailable in privacy modes; reset still applies in memory.
    }
  }

  protected startLeft(value: number): void {
    this.beginHorizontalResize('left');
  }

  protected resizeLeft(value: number): void {
    if (this.activeResize() !== 'left') this.beginHorizontalResize('left');
    this.preferredLeft.set(value);
  }

  protected startRight(value: number): void {
    this.beginHorizontalResize('right');
  }

  protected resizeRight(value: number): void {
    if (this.activeResize() !== 'right') this.beginHorizontalResize('right');
    this.preferredRight.set(value);
  }

  protected startBottom(value: number): void {
    this.activeResize.set('bottom');
    this.preferredBottomBeforeDrag = this.preferredBottom();
  }

  protected resizeBottom(value: number): void {
    this.preferredBottom.set(value);
  }

  protected completeResize(
    region: 'left' | 'right' | 'bottom',
    completion: UISplitterCompletion,
  ): void {
    if (completion.reason === 'cancel') {
      if (region === 'bottom') {
        this.preferredBottom.set(this.preferredBottomBeforeDrag);
      } else {
        this.preferredLeft.set(this.preferredLeftBeforeDrag);
        this.preferredRight.set(this.preferredRightBeforeDrag);
      }
      this.activeResize.set(null);
      return;
    }
    this.activeResize.set(null);
    this.persist();
  }

  protected setLeftCollapsed(value: boolean): void {
    this.leftDockCollapsed.set(value);
    this.persist();
  }

  protected setRightCollapsed(value: boolean): void {
    this.rightDockCollapsed.set(value);
    this.persist();
  }

  protected setBottomCollapsed(value: boolean): void {
    this.bottomCollapsed.set(value);
    this.persist();
  }

  protected setLeftOpened(value: boolean): void {
    // Opening is controlled by the header toggle. Ignore a late `true` emitted while a
    // side drawer changes to `over`, otherwise crossing a breakpoint resurrects it.
    if (!value && this.horizontal().leftOverlay) this.leftOverlayOpen.set(false);
  }

  protected setRightOpened(value: boolean): void {
    if (!value && this.horizontal().rightOverlay) this.rightOverlayOpen.set(false);
  }

  private storageKey(): string {
    return `convertor.ui.scaffold.${this.persistenceKey() || 'default'}.v${STORAGE_VERSION}`;
  }

  private beginHorizontalResize(region: 'left' | 'right'): void {
    if (this.activeResize() === region) return;
    this.preferredLeftBeforeDrag = this.preferredLeft();
    this.preferredRightBeforeDrag = this.preferredRight();
    const rendered = this.horizontal();
    this.preferredLeft.set(rendered.left);
    this.preferredRight.set(rendered.right);
    this.activeResize.set(region);
  }

  private persist(): void {
    try {
      const value: StoredLayout = {
        version: STORAGE_VERSION,
        left: this.preferredLeft(),
        right: this.preferredRight(),
        bottom: this.preferredBottom(),
        leftCollapsed: this.leftDockCollapsed(),
        rightCollapsed: this.rightDockCollapsed(),
        bottomCollapsed: this.bottomCollapsed(),
      };
      localStorage.setItem(this.storageKey(), JSON.stringify(value));
    } catch {
      // Layout remains usable when storage is full, blocked, or unavailable.
    }
  }

  private restore(raw?: string | null): void {
    try {
      const stored = JSON.parse(
        raw ?? localStorage.getItem(this.storageKey()) ?? 'null',
      ) as Partial<StoredLayout> | null;
      if (!stored || stored.version !== STORAGE_VERSION) return;
      if (typeof stored.left === 'number') this.preferredLeft.set(stored.left);
      if (typeof stored.right === 'number') this.preferredRight.set(stored.right);
      if (typeof stored.bottom === 'number') this.preferredBottom.set(stored.bottom);
      if (typeof stored.leftCollapsed === 'boolean')
        this.leftDockCollapsed.set(stored.leftCollapsed);
      if (typeof stored.rightCollapsed === 'boolean')
        this.rightDockCollapsed.set(stored.rightCollapsed);
      if (typeof stored.bottomCollapsed === 'boolean')
        this.bottomCollapsed.set(stored.bottomCollapsed);
    } catch {
      // Ignore malformed or inaccessible persisted state.
    }
  }

  private readonly storageListener = (event: StorageEvent): void => {
    if (event.key !== this.storageKey() || this.activeResize() !== null) return;
    this.restore(event.newValue);
  };
}
