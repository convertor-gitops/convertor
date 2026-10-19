import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { UIThemeService } from '../../../service/ui-theme';
import { UIIconComponent } from '../../shared/ui/ui-icon/ui-icon.component';
@Component({
  selector: 'app-theme-gallery',
  imports: [UIIconComponent],
  templateUrl: './theme-gallery.component.html',
  styleUrl: './theme-gallery.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ThemeGalleryComponent {
  readonly theme = inject(UIThemeService);
}
