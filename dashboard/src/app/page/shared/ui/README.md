# Shared UI

This is the only component implementation layer. `ui-component` demonstrates it and
`plan-board` composes it with business data. Consumer pages must not import Material
controls, redefine their internals, or depend on showcase implementations.

## Naming and ownership

TypeScript UI symbols use the `UI` prefix (`UIButtonComponent`, `UIScaffoldComponent`).
Selectors and file names remain kebab-case (`app-ui-button`, `ui-button.component.ts`).
Material and CDK are implementation dependencies of these shared components. CDK
business drag/drop remains allowed in consumers.

`UIScaffold` owns the page header, side regions, main content, responsive docking,
and optional bottom region. `UIPanel` and `UICard` are content surfaces, not page
layouts. Layout resize uses `UISplitter`; pages do not implement pointer handlers.
The divider has a 24px pointer target with a visible grip. Resizing one dock consumes
main-region space without resizing the opposite dock; hidden docks reserve no space.
Default minimum widths are 160px left, 200px right, and 320px main. Main-region
margins follow the same size signals immediately rather than Material autosize
measurements or animated margins. Responsive overlay changes follow container width,
not drag preference sizes.

`UIDrawer` and `UISheet` own focus, modal lifecycle, resizing, and persisted size.
Only one scaffold-level modal opens at a time. Closing does not clear page data.

`UICollapse` is independently expandable. Its action slot is separate from its
expand button. Business pages keep independent expansion state per stable ID.
Node cards distinguish read-only display, selection, and independent highlighting.

## Styling

Use shared theme tokens and explicit component inputs. Consumers may arrange
components with grid/flex and spacing; they must not reach into shared descendants.
New UI behavior must be implemented here and demonstrated in the showcase, including
narrow widths, long text, keyboard interactions, empty/error/disabled states.

## Checks

Run `npm run check:ui`, `npm test -- --watch=false`, and `npm run build` from dashboard.
The architecture check rejects old Ui prefixes, direct Material imports in the two
consumer pages, deep styling overrides, and local details/summary implementations.
