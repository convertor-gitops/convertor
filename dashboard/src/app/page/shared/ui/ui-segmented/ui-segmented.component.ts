import { ChangeDetectionStrategy, Component, computed, input, model } from '@angular/core';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { UIIconComponent, UIIconName } from '../ui-icon/ui-icon.component';
export interface UISegment {
  value: string | number;
  label: string;
  icon?: UIIconName;
  disabled?: boolean;
}
@Component({
  selector: 'app-ui-segmented',
  imports: [MatButtonToggleModule, UIIconComponent],
  templateUrl: './ui-segmented.component.html',
  styleUrl: './ui-segmented.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit' },
})
export class UISegmentedComponent<T extends string | number> {
  readonly label = input.required<string>();
  readonly options = input.required<readonly UISegment[]>();
  readonly value = model.required<T>();
  readonly selectedIndex = computed(() =>
    Math.max(
      0,
      this.options().findIndex((o) => o.value === this.value()),
    ),
  );
}
