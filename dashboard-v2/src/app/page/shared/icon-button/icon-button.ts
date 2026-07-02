import { Component } from "@angular/core";
import { MatIconButton } from "@angular/material/button";
import { MatIcon } from "@angular/material/icon";

@Component({
    selector: "app-icon-button",
    imports: [
        MatIcon,
        MatIconButton,
    ],
    templateUrl: "./icon-button.html",
    styleUrl: "./icon-button.scss",
})
export class IconButton {

}
