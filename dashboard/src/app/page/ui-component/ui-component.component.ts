import { MatAccordion } from '@angular/material/expansion';
import { UiSegmentedComponent } from '../shared/ui/ui-segmented/ui-segmented.component';
import { UiCollapseComponent } from '../shared/ui/ui-collapse/ui-collapse.component';
import { UiPanelComponent } from '../shared/ui/ui-panel/ui-panel.component';
import { PanelShowcaseComponent } from './panel-showcase/panel-showcase.component';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  DestroyRef,
  inject,
  signal,
  TemplateRef,
} from '@angular/core';
import { ThemeGalleryComponent } from './theme-gallery/theme-gallery.component';
import { AppearancePanelComponent } from './appearance-panel/appearance-panel.component';
import { DOCUMENT } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatDialog, MatDialogClose } from '@angular/material/dialog';
import { UiThemeService } from '../../service/ui-theme';
import { UI_PALETTES } from '../../common/model/ui-theme';
import { UiButtonComponent, UiButtonVariant } from '../shared/ui/ui-button/ui-button.component';
import { UiIconComponent, UiIconName } from '../shared/ui/ui-icon/ui-icon.component';
import { UiIconButtonComponent } from '../shared/ui/ui-icon-button/ui-icon-button.component';
import { UiTextFieldComponent } from '../shared/ui/ui-text-field/ui-text-field.component';
import { UiSelectComponent } from '../shared/ui/ui-select/ui-select.component';
import { UiOptionComponent } from '../shared/ui/ui-option/ui-option.component';
import {
  UiNodeCardComponent,
  UiNodeStatus,
} from '../shared/ui/ui-node-card/ui-node-card.component';

interface SampleNode {
  id: number;
  name: string;
  region: string;
  protocol: string;
  source: string;
  tags: string[];
  status: UiNodeStatus;
  statusText: string;
}
@Component({
  selector: 'app-ui-component',
  imports: [
    MatAccordion,
    UiSegmentedComponent,
    UiCollapseComponent,
    UiPanelComponent,
    PanelShowcaseComponent,
    ThemeGalleryComponent,
    AppearancePanelComponent,
    FormsModule,
    MatDialogClose,
    UiButtonComponent,
    UiIconComponent,
    UiIconButtonComponent,
    UiTextFieldComponent,
    UiSelectComponent,
    UiOptionComponent,
    UiNodeCardComponent,
  ],
  templateUrl: './ui-component.component.html',
  styleUrl: './ui-component.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'ui-kit' },
})
export class UiComponentComponent {
  readonly theme = inject(UiThemeService);
  private readonly document = inject(DOCUMENT);
  private readonly dialog = inject(MatDialog);
  readonly paletteCount = UI_PALETTES.length;
  readonly sections: { id: string; label: string; icon: UiIconName; count: string }[] = [
    { id: 'panels', label: '容器层级', icon: 'layers', count: '03' },
    { id: 'buttons', label: '按钮', icon: 'bolt', count: '05' },
    { id: 'fields', label: '输入与选择', icon: 'sliders', count: '08' },
    { id: 'collapse', label: '折叠面板', icon: 'chevron', count: '02' },
    { id: 'nodes', label: '节点卡片', icon: 'server', count: '12' },
    { id: 'compositions', label: '组合预览', icon: 'layers', count: '02' },
    { id: 'colors', label: '主题色板', icon: 'palette', count: '32' },
  ];
  readonly activeSection = signal('panels');
  readonly variants: { value: UiButtonVariant; label: string }[] = [
    { value: 'filled', label: 'Filled · 实色' },
    { value: 'tonal', label: 'Tonal · 淡色' },
    { value: 'outlined', label: 'Outlined · 描边' },
    { value: 'text', label: 'Text · 文字' },
  ];
  readonly activity = signal('预览设置自动保存');
  readonly loading = signal(false);
  readonly selected = signal(new Set([1, 3]));
  readonly emptySelection = new Set<number>();
  readonly nodeQuery = signal('');
  readonly nodeLayout = signal<'tiles' | 'list'>('tiles');
  readonly selectedCount = computed(() => this.selected().size);
  readonly samples: SampleNode[] = [
    {
      id: 1,
      name: '香港 · 中环 01',
      region: 'HK',
      protocol: 'Trojan',
      source: '主力订阅',
      tags: ['流媒体'],
      status: 'ready',
      statusText: '',
    },
    {
      id: 2,
      name: '香港 · 九龙 02',
      region: 'HK',
      protocol: 'SS',
      source: '主力订阅',
      tags: [],
      status: 'ready',
      statusText: '',
    },
    {
      id: 3,
      name: '日本 · 东京 01',
      region: 'JP',
      protocol: 'VMess',
      source: '备用订阅',
      tags: ['低延迟'],
      status: 'ready',
      statusText: '',
    },
    {
      id: 4,
      name: '日本 · 大阪 02',
      region: 'JP',
      protocol: 'Trojan',
      source: '主力订阅',
      tags: [],
      status: 'ready',
      statusText: '',
    },
    {
      id: 5,
      name: '新加坡 · 滨海湾 01',
      region: 'SG',
      protocol: 'VLESS',
      source: '主力订阅',
      tags: ['专线'],
      status: 'ready',
      statusText: '',
    },
    {
      id: 6,
      name: '台湾 · 台北 01',
      region: 'TW',
      protocol: 'Trojan',
      source: '备用订阅',
      tags: [],
      status: 'loading',
      statusText: '加载中',
    },
    {
      id: 7,
      name: '美国 · 洛杉矶 01',
      region: 'US',
      protocol: 'Hysteria2',
      source: '备用订阅',
      tags: [],
      status: 'ready',
      statusText: '',
    },
    {
      id: 8,
      name: '美国 · 西雅图 02',
      region: 'US',
      protocol: 'SS',
      source: '主力订阅',
      tags: [],
      status: 'error',
      statusText: '加载失败',
    },
    {
      id: 9,
      name: '英国 · 伦敦 01',
      region: 'GB',
      protocol: 'Trojan',
      source: '备用订阅',
      tags: [],
      status: 'ready',
      statusText: '',
    },
    {
      id: 10,
      name: '德国 · 法兰克福 01',
      region: 'DE',
      protocol: 'VLESS',
      source: '主力订阅',
      tags: [],
      status: 'disabled',
      statusText: '已停用',
    },
    {
      id: 11,
      name: '香港 · 超长节点名称与多标签的展示效果检查 · 高速专线 03',
      region: 'HK',
      protocol: 'Trojan',
      source: '主力订阅',
      tags: ['专线', '流媒体'],
      status: 'ready',
      statusText: '',
    },
    {
      id: 12,
      name: 'DIRECT',
      region: '↗',
      protocol: '直连',
      source: '内置策略',
      tags: [],
      status: 'ready',
      statusText: '',
    },
  ];
  readonly visibleNodes = computed(() =>
    this.samples.filter((node) =>
      [node.name, node.source, node.protocol]
        .join(' ')
        .toLowerCase()
        .includes(this.nodeQuery().trim().toLowerCase()),
    ),
  );
  sourceName = '主力订阅';
  sourceUrl = 'https://example.com/subscription';
  search = '';
  notes = '';
  client = 'surge';
  region = 'hk';
  protocols = ['trojan', 'ss'];
  groupName = '流媒体';
  private timer?: ReturnType<typeof setTimeout>;

  constructor() {
    inject(DestroyRef).onDestroy(() => {
      clearTimeout(this.timer);
      this.dialog.closeAll();
    });
  }
  navigate(id: string): void {
    this.activeSection.set(id);
    this.document.getElementById(id)?.scrollIntoView({ block: 'start', behavior: 'instant' });
  }
  announce(label: string): void {
    this.activity.set(label);
  }
  loadExample(): void {
    clearTimeout(this.timer);
    this.loading.set(true);
    this.timer = setTimeout(() => {
      this.loading.set(false);
      this.announce('示例加载完成');
    }, 1400);
  }
  toggleNode(id: number, selected: boolean): void {
    this.selected.update((current) => {
      const next = new Set(current);
      selected ? next.add(id) : next.delete(id);
      return next;
    });
  }
  selectVisible(): void {
    this.selected.update(
      (current) =>
        new Set([
          ...current,
          ...this.visibleNodes()
            .filter((n) => n.status !== 'disabled')
            .map((n) => n.id),
        ]),
    );
  }
  openNode(template: TemplateRef<unknown>, node: SampleNode): void {
    this.dialog.open(template, {
      data: node,
      panelClass: 'ui-kit-overlay',
      width: '400px',
      maxWidth: 'calc(100vw - 32px)',
    });
  }
  paletteModeLabel(): string {
    return this.theme.mode() === 'dark' ? '深色' : '浅色';
  }

  dialogClose(): void {
    this.dialog.closeAll();
  }
}
