import { UiTextFieldComponent, UiSelectComponent, UiOptionComponent } from '../../shared/ui';
import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import {
  GroupStrategy,
  SelectGroupStrategy,
  UrlTestGroupStrategy,
} from '../../../common/model/core/plan';
@Component({
  selector: 'app-strategy-editor',
  imports: [UiTextFieldComponent, UiSelectComponent, UiOptionComponent, FormsModule],
  templateUrl: './strategy-editor.html',
  styleUrl: './strategy-editor.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StrategyEditor {
  readonly strategy = input.required<GroupStrategy>();
  readonly strategyChange = output<GroupStrategy>();
  urlTest(): UrlTestGroupStrategy | null {
    const value = this.strategy();
    return value instanceof UrlTestGroupStrategy ? value : null;
  }
  setKind(kind: string): void {
    this.strategyChange.emit(
      kind === 'url_test'
        ? new UrlTestGroupStrategy('https://www.gstatic.com/generate_204', 300, 50)
        : new SelectGroupStrategy(),
    );
  }
  update(field: 'url' | 'interval_secs' | 'tolerance_ms', value: string | number | null): void {
    const current = this.urlTest();
    if (!current) return;
    const next = new UrlTestGroupStrategy(current.url, current.interval_secs, current.tolerance_ms);
    if (field === 'url') next.url = String(value ?? '');
    else next[field] = Number(value);
    this.strategyChange.emit(next);
  }
}
