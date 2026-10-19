import { UIOverlayCoordinator } from './overlay-coordinator';

describe('UIOverlayCoordinator', () => {
  it('closes the previous scaffold overlay when another activates', () => {
    const coordinator = new UIOverlayCoordinator();
    const first = Symbol('first');
    const second = Symbol('second');
    let firstClosed = 0;

    coordinator.activate(first, () => firstClosed++);
    coordinator.activate(second, () => undefined);

    expect(firstClosed).toBe(1);
  });

  it('does not let a stale overlay release the active owner', () => {
    const coordinator = new UIOverlayCoordinator();
    const first = Symbol('first');
    const second = Symbol('second');
    let secondClosed = 0;

    coordinator.activate(first, () => undefined);
    coordinator.activate(second, () => secondClosed++);
    coordinator.release(first);
    coordinator.activate(Symbol('third'), () => undefined);

    expect(secondClosed).toBe(1);
  });
});
