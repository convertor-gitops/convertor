import { Directive, input } from "@angular/core";

export type UiPanelDensity = "compact" | "comfortable";

/** Shared panel behavior hook; Material continues to own expansion semantics. */
@Directive({
    selector: "mat-expansion-panel",
    host: {
        class: "ui-panel",
        "[class.ui-panel--comfortable]": "uiPanelDensity() === 'comfortable'",
    },
})
export class UiPanelDirective {
    readonly uiPanelDensity = input<UiPanelDensity>("compact");
}
