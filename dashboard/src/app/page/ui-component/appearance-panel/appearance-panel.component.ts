import { UiSegmentedComponent } from '../../shared/ui/ui-segmented/ui-segmented.component';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { UiThemeService } from '../../../service/ui-theme';
import { UiDensity, UiModePreference, UiRadius } from '../../../common/model/ui-theme';
import { UiButtonComponent } from '../../shared/ui/ui-button/ui-button.component';
import { UiIconComponent, UiIconName } from '../../shared/ui/ui-icon/ui-icon.component';
import { UiIconButtonComponent } from '../../shared/ui/ui-icon-button/ui-icon-button.component';
import { UiSelectComponent } from '../../shared/ui/ui-select/ui-select.component';
import { UiOptionComponent } from '../../shared/ui/ui-option/ui-option.component';
@Component({
  selector: 'app-appearance-panel',
  imports: [
    UiSegmentedComponent,
    FormsModule,
    UiButtonComponent,
    UiIconComponent,
    UiIconButtonComponent,
    UiSelectComponent,
    UiOptionComponent,
  ],
  templateUrl: './appearance-panel.component.html',
  styleUrl: './appearance-panel.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', 'aria-label': '外观调节' },
})
export class AppearancePanelComponent {
  readonly theme = inject(UiThemeService);
  readonly modes: { value: UiModePreference; label: string; icon: UiIconName }[] = [
    { value: 'dark', label: '深色', icon: 'moon' },
    { value: 'light', label: '浅色', icon: 'sun' },
    { value: 'system', label: '系统', icon: 'monitor' },
  ];
  readonly densities: { value: UiDensity; label: string }[] = [
    { value: 'ultra', label: '极紧凑' },
    { value: 'compact', label: '紧凑' },
    { value: 'comfortable', label: '标准' },
  ];
  readonly radii: UiRadius[] = [4, 8, 12, 16];
  readonly radiusOptions = this.radii.map((value) => ({ value, label: value + 'px' }));
  readonly radiusSettings = [
    { key: 'controlRadius' as const, label: '输入控件圆角' },
    { key: 'cardRadius' as const, label: '卡片圆角' },
    { key: 'panelRadius' as const, label: '容器圆角' },
    { key: 'overlayRadius' as const, label: '浮层圆角' },
  ];
  paletteModeLabel(): string {
    return this.theme.mode() === 'dark' ? '深色' : '浅色';
  }
  setRadius(
    key: 'controlRadius' | 'cardRadius' | 'overlayRadius' | 'panelRadius',
    value: UiRadius,
  ): void {
    this.theme.update({ [key]: value });
  }
}
