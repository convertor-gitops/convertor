import { ChangeDetectionStrategy, Component, input, TemplateRef, viewChild } from '@angular/core';

@Component({
  selector: 'app-ui-tab',
  template: '<ng-template><ng-content /></ng-template>',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UITabComponent {
  readonly label = input.required<string>();
  readonly disabled = input(false);
  readonly content = viewChild.required(TemplateRef);
}
