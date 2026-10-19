import { NgTemplateOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, contentChildren, input, model } from '@angular/core';
import { MatTabsModule } from '@angular/material/tabs';
import { UITabComponent } from './ui-tab.component';

@Component({
  selector: 'app-ui-tabs',
  imports: [NgTemplateOutlet, MatTabsModule],
  templateUrl: './ui-tabs.component.html',
  styleUrl: './ui-tabs.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit', '[class.fill]': 'fill()' },
})
export class UITabsComponent {
  readonly fill = input(false);
  readonly selectedIndex = model(0);
  readonly preserveContent = input(true);
  readonly tabs = contentChildren(UITabComponent);
}
