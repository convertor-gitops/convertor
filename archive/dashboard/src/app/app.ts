import { ChangeDetectionStrategy, Component, signal } from "@angular/core";
import { Dashboard } from "./page/dashboard/dashboard";

@Component({
    selector: "app-root",
    imports: [ Dashboard ],
    templateUrl: "./app.html",
    styleUrl: "./app.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
    protected readonly title = signal("Convertor Dashboard");
}
