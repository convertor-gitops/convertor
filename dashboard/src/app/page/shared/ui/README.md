# Workbench UI

This folder is the small application layer over Angular Material used by the
plan workbench. Material still owns focus, keyboard, disabled and expansion
behavior; this layer supplies the product's visual vocabulary and reusable
interaction wrappers.

## Inventory

| Material primitive | Workbench treatment |
| --- | --- |
| Text, flat and stroked buttons | `UiButtonDirective`; 4px shape and semantic tone hooks |
| Icon button | `UiCloseButton`; labelled 40px close action with an inline SVG |
| Expansion panel / accordion | `UiPanelDirective`; 8px panel and narrow stacked header |
| Form field, input and select | Adapted by `workbench-ui.theme()`; 4px field shape |
| Tabs | Adapted 40px density, typography and active indicator |
| Checkbox and slide toggle | Adapted density plus a 4px switch shape; Material state behavior retained |
| Sidenav / drawer | Adapted by `workbench-ui.theme()`; 12px outer surface shape |
| Tree and progress bar | Adapted row density and 4px progress track |
| Spinner | Material geometry retained so its circle is never reshaped |

The directives deliberately match Material selectors. Import them once in a
standalone component and existing Material markup opts in without duplicate
wrapper elements:

```ts
import {
    UiButtonDirective,
    UiCloseButton,
    UiPanelDirective,
} from "../shared/ui";

@Component({
    imports: [UiButtonDirective, UiCloseButton, UiPanelDirective],
})
export class Example {}
```

```html
<button mat-flat-button uiButtonTone="primary">保存</button>
<mat-expansion-panel uiPanelDensity="comfortable">...</mat-expansion-panel>
<app-ui-close-button (closed)="close()" />
```

Button tones are `default`, `primary`, and `danger`. Panel densities are
`compact` and `comfortable`. The Sass entry point is
`styles/_workbench-ui.scss`; consumers include `workbench-ui.theme()` on the
workbench host so overlays and legacy pages are not restyled accidentally. It
also publishes `--ui-radius-sm` (4px), `--ui-radius-md` (8px), and
`--ui-radius-lg` (12px) for feature-owned surfaces.
