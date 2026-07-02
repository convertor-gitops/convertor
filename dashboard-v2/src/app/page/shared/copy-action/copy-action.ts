import { CdkCopyToClipboard } from "@angular/cdk/clipboard";
import { ChangeDetectionStrategy, Component, input, signal } from "@angular/core";

@Component({
    selector: "app-copy-action",
    imports: [CdkCopyToClipboard],
    templateUrl: "./copy-action.html",
    styleUrl: "./copy-action.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CopyAction {
    readonly value = input.required<string>();
    readonly copied = signal(false);
    private timer: ReturnType<typeof setTimeout> | null = null;

    markCopied(): void {
        this.copied.set(true);
        if (this.timer) {
            clearTimeout(this.timer);
        }
        this.timer = setTimeout(() => this.copied.set(false), 1400);
    }
}
