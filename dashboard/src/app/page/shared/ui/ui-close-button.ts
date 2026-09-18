import { ChangeDetectionStrategy, Component, input, output } from "@angular/core";
import { MatIconButton } from "@angular/material/button";

@Component({
    selector: "app-ui-close-button",
    imports: [MatIconButton],
    template: `
        <button
            matIconButton
            type="button"
            class="ui-close-button"
            [attr.aria-label]="label()"
            [disabled]="disabled()"
            (click)="closed.emit()">
            <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M6.4 6.4 12 12m0 0 5.6 5.6M12 12l5.6-5.6M12 12l-5.6 5.6" />
            </svg>
        </button>
    `,
    styles: `
        :host {
            display: inline-flex;
            flex: none;
        }

        .ui-close-button {
            width: 40px;
            height: 40px;
            padding: 10px;
            border: 1px solid transparent;
            border-radius: 4px;
            color: var(--app-text-3, currentColor);
        }

        .ui-close-button:hover:not(:disabled) {
            border-color: var(--app-border-2, currentColor);
            background: var(--app-surface-2, transparent);
            color: var(--app-text, currentColor);
        }

        svg {
            display: block;
            width: 18px;
            height: 18px;
            fill: none;
            stroke: currentColor;
            stroke-linecap: round;
            stroke-width: 1.75;
        }
    `,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UiCloseButton {
    readonly label = input("关闭");
    readonly disabled = input(false);
    readonly closed = output<void>();
}
