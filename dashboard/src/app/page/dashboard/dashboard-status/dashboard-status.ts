import { AsyncPipe } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject, OnInit } from "@angular/core";
import { MatCardContent, MatCardHeader, MatCardTitle } from "@angular/material/card";
import { BehaviorSubject } from "rxjs";
import { BackendStatus } from "../../../common/model/api/backend-status";
import { MetadataService } from "../../../service/metadata.service";
import { StatusService } from "../../../service/status.service";
import { DashboardPanel } from "../dashboard-panel/dashboard-panel";
import { IconButton } from "../../shared/icon-button/icon-button";

@Component({
  selector: "app-dashboard-status",
  imports: [AsyncPipe, DashboardPanel, MatCardHeader, MatCardTitle, MatCardContent, IconButton],
  templateUrl: "./dashboard-status.html",
  styleUrl: "./dashboard-status.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardStatus implements OnInit {
  private metadataService = inject(MetadataService);
  private statusService = inject(StatusService);

  dashboardVersion = this.metadataService.version;
  backendStatus$ = new BehaviorSubject<BackendStatus | null>(null);
  error$ = new BehaviorSubject<string | null>(null);

  ngOnInit(): void {
    this.refresh();
  }

  refresh(): void {
    this.error$.next(null);
    this.backendStatus$.next(null);
    this.statusService.getStatus().subscribe({
      next: status => this.backendStatus$.next(status),
      error: () => this.error$.next("无法连接后端"),
    });
  }
}
