import { Directive, input } from "@angular/core";

export type UiButtonTone = "default" | "primary" | "danger";

/**
 * Adds the workbench button vocabulary to native Angular Material buttons.
 * Importing this directive is enough to opt every Material button in a
 * standalone component into the shared treatment.
 */
@Directive({
    selector: "button[mat-button], button[mat-flat-button], button[mat-stroked-button]",
    host: {
        class: "ui-button",
        "[class.ui-button--primary]": "uiButtonTone() === 'primary'",
        "[class.ui-button--danger]": "uiButtonTone() === 'danger'",
    },
})
export class UiButtonDirective {
    readonly uiButtonTone = input<UiButtonTone>("default");
}
