import { Injectable } from '@angular/core';

export type UIOverlayOwner = symbol;
export type UIOverlayCloseReason = 'button' | 'backdrop' | 'escape' | 'replaced';

/** Coordinates scaffold-level overlays without affecting nested CDK dialogs/selects. */
@Injectable({ providedIn: 'root' })
export class UIOverlayCoordinator {
  private active: { owner: UIOverlayOwner; close: () => void } | undefined;

  activate(owner: UIOverlayOwner, close: () => void): void {
    if (this.active?.owner === owner) {
      this.active.close = close;
      return;
    }

    const previous = this.active;
    this.active = { owner, close };
    previous?.close();
  }

  release(owner: UIOverlayOwner): void {
    if (this.active?.owner === owner) {
      this.active = undefined;
    }
  }
}
