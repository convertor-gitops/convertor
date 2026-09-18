import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import {
  GroupStrategy,
  SelectGroupStrategy,
  UrlTestGroupStrategy,
} from '../../../common/model/core/plan';
@Component({
  selector: 'app-strategy-editor',
  imports: [FormsModule, MatFormFieldModule, MatInputModule, MatSelectModule],
  template: `
    <mat-form-field appearance="outline" subscriptSizing="dynamic"
      ><mat-label>组行为</mat-label>
      <mat-select [ngModel]="strategy().kind" (ngModelChange)="setKind($event)">
        <mat-option value="select">手动选择</mat-option
        ><mat-option value="url_test">自动测速</mat-option>
      </mat-select>
    </mat-form-field>
    @if (urlTest(); as test) {
      <mat-form-field appearance="outline" subscriptSizing="dynamic"
        ><mat-label>测速 URL</mat-label
        ><input matInput [ngModel]="test.url" (ngModelChange)="update('url', $event)"
      /></mat-form-field>
      <mat-form-field appearance="outline" subscriptSizing="dynamic"
        ><mat-label>间隔（秒）</mat-label
        ><input
          matInput
          type="number"
          min="1"
          [ngModel]="test.interval_secs"
          (ngModelChange)="update('interval_secs', $event)"
      /></mat-form-field>
      <mat-form-field appearance="outline" subscriptSizing="dynamic"
        ><mat-label>容差（毫秒）</mat-label
        ><input
          matInput
          type="number"
          min="0"
          [ngModel]="test.tolerance_ms"
          (ngModelChange)="update('tolerance_ms', $event)"
      /></mat-form-field>
    }
  `,
  styles: `
    :host {
      display: flex;
      flex-wrap: wrap;
      gap: 10px;
    }
    mat-form-field {
      flex: 1 1 130px;
      min-width: 0;
    }
  `,
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
