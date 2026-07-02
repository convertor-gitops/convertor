import { DestroyRef, inject, Injectable } from "@angular/core";
import { Observable } from "rxjs";

export type Hz = number; // 如需收紧为 144|120|60 也可

export type Tick = readonly [ nowMs: number, dtMs: number ];

@Injectable({ providedIn: "root" })
export class TickerService {
    private destroyRef = inject(DestroyRef);

    private rafId = -1;
    private running = false;

    /** 上一次“发出 tick”的 rAF 时间戳（performance 时钟域） */
    private lastEmitTs = 0;

    /** 目标刷新率（可动态切换） */
    private targetHz: Hz = 144;

    /** 帧监听集合：nowMs = rAF ts（perf 时钟），dtMs = 与上次 emit 的时间差（已做大跳时保护） */
    private readonly listeners = new Set<(nowMs: number, dtMs: number) => void>();

    /** 设定目标刷新率（小于等于屏幕刷新率时相当于节流） */
    setTarget(hz: Hz) {
        this.targetHz = hz > 0 ? hz : 60;
    }

    /** 订阅每帧回调；返回 off 函数。自动启停：有订阅→启动，无订阅→停止 */
    onTick(fn: (nowMs: number, dtMs: number) => void): () => void {
        this.listeners.add(fn);
        if (!this.running) this.start();
        return () => {
            this.listeners.delete(fn);
            if (this.listeners.size === 0) this.stop();
        };
    }

    /** 以 Observable 形式订阅帧时钟 */
    tickObservable(): Observable<Tick> {
        return new Observable<Tick>(subscriber => {
            const off = this.onTick((now, dt) => subscriber.next([ now, dt ]));
            return () => off();
        });
    }

    /** 手动启动（通常无需显式调用；onTick 会自动启动） */
    start() {
        if (this.running) return;
        this.running = true;

        const loop = (ts: number) => {
            if (!this.running) return;

            const minInterval = 1000 / this.targetHz;

            // 距离上次“真实发出 tick”的时间
            const dtSinceEmit = this.lastEmitTs ? ts - this.lastEmitTs : minInterval;

            // 是否该在本帧发出一次 tick（基于目标帧率做节流）
            if (this.lastEmitTs === 0 || dtSinceEmit >= minInterval) {
                // 大跳时保护：后台/卡顿回来时不要把巨大的 dt 透传给动画层
                const safeDt = dtSinceEmit > 250 ? minInterval : dtSinceEmit;

                // 发给所有监听者（注意：一次 rAF 只发一次，避免同帧多次 setOption）
                for (const fn of this.listeners) fn(ts, safeDt);

                this.lastEmitTs = ts;
            }

            this.rafId = requestAnimationFrame(loop);
        };

        this.rafId = requestAnimationFrame(loop);

        // Service 销毁时确保停止
        this.destroyRef.onDestroy(() => this.stop());
    }

    /** 停止时钟并复位内部状态 */
    stop() {
        if (!this.running) return;
        this.running = false;
        cancelAnimationFrame(this.rafId);
        this.rafId = -1;
        this.lastEmitTs = 0;
    }
}
