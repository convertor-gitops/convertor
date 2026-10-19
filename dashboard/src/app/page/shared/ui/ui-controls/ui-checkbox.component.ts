import { ChangeDetectionStrategy, Component, forwardRef } from '@angular/core';
import { NG_VALUE_ACCESSOR } from '@angular/forms';
import { MatCheckbox } from '@angular/material/checkbox';
import { UIToggleBase } from './ui-toggle-base';

@Component({
  selector: 'app-ui-checkbox',
  imports: [MatCheckbox],
  templateUrl: './ui-checkbox.component.html',
  styleUrl: './ui-checkbox.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => UICheckboxComponent), multi: true },
  ],
  host: { class: 'ui-kit' },
})
export class UICheckboxComponent extends UIToggleBase {}
