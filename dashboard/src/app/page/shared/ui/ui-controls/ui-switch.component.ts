import { ChangeDetectionStrategy, Component, forwardRef } from '@angular/core';
import { NG_VALUE_ACCESSOR } from '@angular/forms';
import { MatSlideToggle } from '@angular/material/slide-toggle';
import { UIToggleBase } from './ui-toggle-base';
@Component({
  selector: 'app-ui-switch',
  imports: [MatSlideToggle],
  templateUrl: './ui-switch.component.html',
  styleUrl: './ui-switch.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  providers: [
    { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => UISwitchComponent), multi: true },
  ],
  host: { class: 'ui-kit' },
})
export class UISwitchComponent extends UIToggleBase {}
