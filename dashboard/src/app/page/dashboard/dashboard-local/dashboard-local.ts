import { ChangeDetectionStrategy, Component, computed, inject, signal } from "@angular/core";
import { NavigationEnd, Router, RouterOutlet } from "@angular/router";
import { filter } from "rxjs";
import { PreviewContextService } from "../../../service/preview-context.service";
import { WorkbenchSection } from "../../shared/workbench-section/workbench-section";

@Component({
    selector: "app-dashboard-local",
    imports: [RouterOutlet, WorkbenchSection],
    templateUrl: "./dashboard-local.html",
    styleUrl: "./dashboard-local.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardLocal {
    private router = inject(Router);
    private previewCtx = inject(PreviewContextService);

    readonly url = signal(this.router.url);
    readonly isPreview = computed(() => this.url().endsWith("/preview"));
    readonly title = computed(() => this.isPreview() ? "注入预览" : "本地配置目录");
    readonly subtitle = computed(() => this.isPreview() ? "Injection Preview" : "Local Config Access");

    constructor() {
        this.router.events.pipe(
            filter((event): event is NavigationEnd => event instanceof NavigationEnd),
        ).subscribe(event => this.url.set(event.urlAfterRedirects));
    }

    back(): void {
        this.previewCtx.clear();
        void this.router.navigate(["/"]);
    }
}
