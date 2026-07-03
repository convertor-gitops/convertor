import { ChangeDetectionStrategy, Component, input } from "@angular/core";

@Component({
    selector: "app-workbench-section",
    imports: [],
    templateUrl: "./workbench-section.html",
    styleUrl: "./workbench-section.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchSection {
    readonly index = input.required<string>();
    readonly title = input.required<string>();
    readonly subtitle = input<string>("");
}
