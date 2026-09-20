import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { UiThemeService } from '../../../service/ui-theme';
import { UiIconComponent } from '../../shared/ui/ui-icon/ui-icon.component';
@Component({
  selector: 'app-theme-gallery',
  imports: [UiIconComponent],
  templateUrl: './theme-gallery.component.html',
  styleUrl: './theme-gallery.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ThemeGalleryComponent {
  readonly theme = inject(UiThemeService);
}
