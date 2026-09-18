import { Routes } from "@angular/router";
import { PlanBoard } from "./page/plan-board/plan-board";

export const routes: Routes = [
    {
        path: "",
        component: PlanBoard,
        pathMatch: "full",
    },
];
