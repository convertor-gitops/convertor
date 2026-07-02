import { Routes } from "@angular/router";
import { Dashboard } from "./page/dashboard/dashboard";
import { DashboardPreview } from "./page/dashboard/dashboard-preview/dashboard-preview";
import { DashboardSync } from "./page/dashboard/dashboard-sync/dashboard-sync";

export const routes: Routes = [
    {
        path: "",
        component: Dashboard,
        children: [
            { path: "", component: DashboardSync, pathMatch: "full" },
            { path: "preview", component: DashboardPreview },
        ],
    },
];
