import { UISegmentedComponent } from '../ui/ui-segmented/ui-segmented.component';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { UIThemeService } from '../../../service/ui-theme';
import { UIDensity, UIModePreference, UIRadius } from '../../../common/model/ui-theme';
import { UIButtonComponent } from '../ui/ui-button/ui-button.component';
import { UIIconComponent, UIIconName } from '../ui/ui-icon/ui-icon.component';
import { UIIconButtonComponent } from '../ui/ui-icon-button/ui-icon-button.component';
import { UISelectComponent } from '../ui/ui-select/ui-select.component';
import { UIOptionComponent } from '../ui/ui-option/ui-option.component';
@Component({
  selector: 'app-appearance-panel',
  imports: [
    UISegmentedComponent,
    FormsModule,
    UIButtonComponent,
    UIIconComponent,
    UIIconButtonComponent,
    UISelectComponent,
    UIOptionComponent,
  ],
  templateUrl: './appearance-panel.component.html',
  styleUrl: './appearance-panel.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', 'aria-label': '外观调节' },
})
export class AppearancePanelComponent {
  readonly theme = inject(UIThemeService);
  readonly modes: { value: UIModePreference; label: string; icon: UIIconName }[] = [
    { value: 'dark', label: '深色', icon: 'moon' },
    { value: 'light', label: '浅色', icon: 'sun' },
    { value: 'system', label: '系统', icon: 'monitor' },
  ];
  readonly densities: { value: UIDensity; label: string }[] = [
    { value: 'ultra', label: '极紧凑' },
    { value: 'compact', label: '紧凑' },
    { value: 'comfortable', label: '标准' },
  ];
  readonly radii: UIRadius[] = [4, 8, 12, 16];
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
    value: UIRadius,
  ): void {
    this.theme.update({ [key]: value });
  }
}
