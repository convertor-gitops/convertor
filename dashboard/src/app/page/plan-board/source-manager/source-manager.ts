import { UIProgressComponent, UISwitchComponent } from '../../shared/ui/ui-controls/ui-controls';
import {
  UIButtonComponent,
  UITextFieldComponent,
  UISelectComponent,
  UIOptionComponent,
  UICollapseComponent,
} from '../../shared/ui';
import { ChangeDetectionStrategy, Component, effect, inject, output, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import {
  AllPredicate,
  InlineSourceInput,
  NodePredicate,
  Predicate,
  RemoteSourceInput,
  Source,
  SourceInput,
} from '../../../common/model/core/plan';
import { ProxyClient } from '../../../common/model/core/proxy-client';
import { PlanBoardService } from '../../../service/plan-board.service';
import { EditablePredicate, PredicateEditor } from '../predicate-editor/predicate-editor';

type SourceKind = 'remote' | 'inline';

@Component({
  selector: 'app-source-manager',
  imports: [
    UIProgressComponent,
    UISwitchComponent,
    UIButtonComponent,
    UITextFieldComponent,
    UISelectComponent,
    UIOptionComponent,
    UICollapseComponent,
    FormsModule,
    PredicateEditor,
  ],
  templateUrl: './source-manager.html',
  styleUrl: './source-manager.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SourceManager {
  readonly collapsedSources = signal<ReadonlySet<number>>(new Set());
  setSourceExpanded(id: number, expanded: boolean): void {
    this.collapsedSources.update((s) => {
      const next = new Set(s);
      expanded ? next.delete(id) : next.add(id);
      return next;
    });
  }
  readonly board = inject(PlanBoardService);
  readonly closeRequested = output<void>();

  readonly draftName = signal('');
  readonly draftKind = signal<SourceKind>('remote');
  readonly draftContent = signal('');
  readonly clientOptions = [ProxyClient.Surge, ProxyClient.Clash];
  private readonly lastLoadedInputs = new Map<number, string>();

  constructor() {
    effect(() => {
      const snapshots = this.board.sourceProfiles();
      for (const source of this.board.plan().sources) {
        if (snapshots.has(source.id))
          this.lastLoadedInputs.set(source.id, this.inputValue(source.input));
      }
    });
  }

  addAndLoad(): void {
    const content = this.draftContent().trim();
    if (!content) return;
    const source = this.board.addSource(this.draftKind());
    this.board.updateSource(source.id, (item) => {
      item.name = this.draftName().trim() || `来源 ${source.id}`;
      item.input = this.createInput(this.draftKind(), content);
    });
    this.draftName.set('');
    this.draftContent.set('');
    this.board.loadSource(source.id);
  }

  setClient(client: ProxyClient): void {
    if (client === this.board.plan().client) return;
    this.captureLoadedInputs();
    this.board.setClient(client);
    for (const source of this.board.plan().sources) {
      if (this.inputReady(source.input)) this.board.loadSource(source.id);
    }
  }

  updateName(source: Source, name: string): void {
    this.board.updateSource(source.id, (item) => (item.name = name));
  }

  updateKind(source: Source, kind: SourceKind): void {
    if (source.input.kind === kind) return;
    this.rememberLoadedInput(source);
    this.board.updateSource(source.id, (item) => (item.input = this.createInput(kind, '')));
  }

  updateInput(source: Source, value: string): void {
    if (value === this.inputValue(source.input)) return;
    this.rememberLoadedInput(source);
    this.board.updateSource(source.id, (item) => {
      item.input = this.createInput(item.input.kind, value);
    });
  }

  load(source: Source, refresh: boolean): void {
    if (!this.inputReady(source.input)) return;
    this.board.loadSource(source.id, refresh);
  }

  remove(source: Source): void {
    this.lastLoadedInputs.delete(source.id);
    this.board.removeSource(source.id);
  }

  setFilterEnabled(source: Source, enabled: boolean): void {
    this.board.updateSource(source.id, (item) => {
      item.node_filter = enabled ? (item.node_filter ?? new AllPredicate<NodePredicate>([])) : null;
    });
  }

  updateFilter(source: Source, predicate: EditablePredicate): void {
    this.board.updateSource(source.id, (item) => {
      item.node_filter = predicate as Predicate<NodePredicate>;
    });
  }

  inputValue(input: SourceInput): string {
    return input instanceof RemoteSourceInput ? input.url : (input as InlineSourceInput).content;
  }

  loadedInput(source: Source): string | null {
    return this.lastLoadedInputs.get(source.id) ?? null;
  }

  inputChangedSinceLoad(source: Source): boolean {
    const loaded = this.loadedInput(source);
    return loaded !== null && loaded !== this.inputValue(source.input);
  }

  loadedAt(sourceId: number): string {
    const value = this.board.sourceProfiles().get(sourceId)?.loaded_at;
    if (!value) return '';
    const millis = value < 10_000_000_000 ? value * 1000 : value;
    return new Date(millis).toLocaleString('zh-CN');
  }

  private captureLoadedInputs(): void {
    for (const source of this.board.plan().sources) this.rememberLoadedInput(source);
  }

  private rememberLoadedInput(source: Source): void {
    if (this.board.sourceProfiles().has(source.id) && !this.lastLoadedInputs.has(source.id)) {
      this.lastLoadedInputs.set(source.id, this.inputValue(source.input));
    }
  }

  private createInput(kind: SourceKind, value: string): SourceInput {
    return kind === 'remote' ? new RemoteSourceInput(value) : new InlineSourceInput(value);
  }

  private inputReady(input: SourceInput): boolean {
    return this.inputValue(input).trim().length > 0;
  }
}
