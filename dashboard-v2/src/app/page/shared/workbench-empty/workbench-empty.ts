import { ChangeDetectionStrategy, Component, input } from "@angular/core";

@Component({
    selector: "app-workbench-empty",
    imports: [],
    templateUrl: "./workbench-empty.html",
    styleUrl: "./workbench-empty.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WorkbenchEmpty {
    readonly symbol = input<string>("∅");
    readonly text = input.required<string>();
}
