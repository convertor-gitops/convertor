import { ChangeDetectionStrategy, Component, computed, input, model } from '@angular/core';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { UiIconComponent, UiIconName } from '../ui-icon/ui-icon.component';
export interface UiSegment {
  value: string | number;
  label: string;
  icon?: UiIconName;
  disabled?: boolean;
}
@Component({
  selector: 'app-ui-segmented',
  imports: [MatButtonToggleModule, UiIconComponent],
  templateUrl: './ui-segmented.component.html',
  styleUrl: './ui-segmented.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit' },
})
export class UiSegmentedComponent<T extends string | number> {
  readonly label = input.required<string>();
  readonly options = input.required<readonly UiSegment[]>();
  readonly value = model.required<T>();
  readonly selectedIndex = computed(() =>
    Math.max(
      0,
      this.options().findIndex((o) => o.value === this.value()),
    ),
  );
}
