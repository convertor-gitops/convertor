import { Routes } from "@angular/router";

export const routes: Routes = [
    {
        path: "",
        redirectTo: "ui-component",
        pathMatch: "full",
    },
    {
        path: "ui-component",
        loadComponent: () => import("./page/ui-component/ui-component.component").then(m => m.UiComponentComponent),
    },
    {
        path: "plan-board",
        loadComponent: () => import("./page/plan-board/plan-board").then(m => m.PlanBoard),
    },
];
