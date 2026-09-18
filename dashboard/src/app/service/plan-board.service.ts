import { computed, Injectable, OnDestroy, signal } from '@angular/core';
import { Subscription } from 'rxjs';
import {
  BuildPlanUrlRequest,
  DecodePlanUrlRequest,
  EvaluatePlanRequest,
  LoadSourceRequest,
} from '../common/model/api/plan-board';
import { Diagnostic, EvaluationReport, SourceProfile } from '../common/model/core/evaluation';
import {
  CustomGroup,
  GroupingPolicy,
  InlineSourceInput,
  Plan,
  RemoteSourceInput,
  RuleBlock,
  Source,
  SourceInput,
} from '../common/model/core/plan';
import { ProxyClient } from '../common/model/core/proxy-client';
import { PlanBoardApiService } from './plan-board-api.service';

const DRAFT_KEY = 'convertor.plan-board.v1';

@Injectable()
/**
 * Component-scoped PlanBoard state and command service.
 *
 * Only the Plan is persisted. Loaded profiles and evaluation output always
 * come from the current session and are invalidated by semantic input changes.
 */
export class PlanBoardService implements OnDestroy {
  private readonly planState = signal(Plan.empty());
  private readonly sourceProfilesState = signal(new Map<number, SourceProfile>());
  private readonly selectedSourceIdState = signal<number | null>(null);
  private readonly reportState = signal<EvaluationReport | null>(null);
  private readonly renderedContentState = signal<string | null>(null);
  private readonly subscriptionUrlState = signal<string | null>(null);
  private readonly loadingSourceIdsState = signal(new Set<number>());
  private readonly sourceErrorsState = signal(new Map<number, string>());
  private readonly previewStaleState = signal(false);
  private readonly evaluatingState = signal(false);
  private readonly buildingState = signal(false);
  private readonly restoringState = signal(false);
  private readonly errorState = signal<string | null>(null);
  private readonly dirtyState = signal(false);
  private evaluationTimer: ReturnType<typeof setTimeout> | null = null;
  private evaluationSubscription: Subscription | null = null;
  private readonly sourceSubscriptions = new Map<number, Subscription>();
  private buildSubscription: Subscription | null = null;
  private restoreSubscription: Subscription | null = null;
  private planRevision = 0;
  private evaluationGeneration = 0;
  private buildGeneration = 0;
  private restoreGeneration = 0;
  private readonly sourceGenerations = new Map<number, number>();
  private destroyed = false;

  readonly plan = this.planState.asReadonly();
  readonly sourceProfiles = this.sourceProfilesState.asReadonly();
  readonly selectedSourceId = this.selectedSourceIdState.asReadonly();
  readonly report = this.reportState.asReadonly();
  readonly renderedContent = this.renderedContentState.asReadonly();
  readonly subscriptionUrl = this.subscriptionUrlState.asReadonly();
  readonly loadingSourceIds = this.loadingSourceIdsState.asReadonly();
  readonly sourceErrors = this.sourceErrorsState.asReadonly();
  readonly previewStale = this.previewStaleState.asReadonly();
  readonly evaluating = this.evaluatingState.asReadonly();
  readonly building = this.buildingState.asReadonly();
  readonly restoring = this.restoringState.asReadonly();
  readonly error = this.errorState.asReadonly();
  readonly dirty = this.dirtyState.asReadonly();

  readonly selectedSource = computed(
    () => this.plan().sources.find((source) => source.id === this.selectedSourceId()) ?? null,
  );
  readonly selectedSourceProfile = computed(() => {
    const id = this.selectedSourceId();
    return id === null ? null : (this.sourceProfiles().get(id) ?? null);
  });
  readonly sourceProfilesComplete = computed(() => {
    const profiles = this.sourceProfiles();
    return (
      this.plan().sources.length > 0 &&
      this.plan().sources.every((source) => profiles.has(source.id))
    );
  });
  readonly diagnostics = computed<Diagnostic[]>(() => [
    ...Array.from(this.sourceProfiles().values()).flatMap((profile) => profile.diagnostics),
    ...(this.report()?.diagnostics ?? []),
  ]);
  readonly canBuildUrl = computed(
    () =>
      this.sourceProfilesComplete() &&
      this.report() !== null &&
      !this.report()!.hasErrors &&
      this.renderedContent() !== null &&
      !this.previewStale() &&
      this.loadingSourceIds().size === 0 &&
      this.sourceErrors().size === 0 &&
      !this.evaluating() &&
      !this.building() &&
      !this.restoring(),
  );

  constructor(private readonly api: PlanBoardApiService) {
    const restored = this.readDraft();
    if (restored) {
      this.planState.set(restored);
      this.selectedSourceIdState.set(restored.sources[0]?.id ?? null);
      if (
        restored.sources.length > 0 &&
        restored.sources.every((source) => sourceInputReady(source.input))
      ) {
        queueMicrotask(() => this.reloadAllSources());
      }
    }
  }

  selectSource(sourceId: number | null): void {
    this.selectedSourceIdState.set(sourceId);
  }

  setClient(client: ProxyClient): void {
    if (this.plan().client === client) return;
    this.changePlan((plan) => {
      plan.client = client;
    }, true);
  }

  addSource(kind: 'remote' | 'inline' = 'remote'): Source {
    const plan = this.plan().clone();
    const id = Math.max(0, ...plan.sources.map((source) => source.id)) + 1;
    const input: SourceInput =
      kind === 'remote' ? new RemoteSourceInput('') : new InlineSourceInput('');
    const source = new Source(id, `来源 ${id}`, input, [], null);
    plan.sources.push(source);
    if (plan.sources.length === 1) plan.output.settings_source = id;
    this.commitPlan(plan);
    this.selectedSourceIdState.set(id);
    return source;
  }

  updateSource(sourceId: number, update: (source: Source) => void): void {
    const plan = this.plan().clone();
    const source = plan.sources.find((item) => item.id === sourceId);
    if (!source) return;
    const previousInput = JSON.stringify(source.input.serialize());
    update(source);
    const inputChanged = previousInput !== JSON.stringify(source.input.serialize());
    if (inputChanged) this.dropSourceProfile(sourceId);
    this.commitPlan(plan);
  }

  removeSource(sourceId: number): void {
    const plan = this.plan().clone();
    plan.sources = plan.sources.filter((source) => source.id !== sourceId);
    if (plan.output.settings_source === sourceId)
      plan.output.settings_source = plan.sources[0]?.id ?? 0;
    plan.output.extra_nodes = plan.output.extra_nodes.filter(
      (selection) => selection.source !== sourceId,
    );
    this.dropSourceProfile(sourceId);
    this.commitPlan(plan);
    if (this.selectedSourceId() === sourceId)
      this.selectedSourceIdState.set(plan.sources[0]?.id ?? null);
  }

  moveSource(sourceId: number, offset: -1 | 1): void {
    this.changePlan((plan) => moveById(plan.sources, sourceId, offset));
  }

  setGroupingPolicies(policies: GroupingPolicy[]): void {
    this.changePlan((plan) => {
      plan.grouping_policies = policies;
    });
  }

  setGroups(groups: CustomGroup[]): void {
    this.changePlan((plan) => {
      plan.groups = groups;
    });
  }

  setRules(rules: RuleBlock[]): void {
    this.changePlan((plan) => {
      plan.rules = rules;
    });
  }

  replaceEditedPlan(plan: Plan): void {
    const previous = this.plan();
    const profiles = new Map(this.sourceProfiles());
    if (previous.client !== plan.client) {
      this.invalidateAllSourceRequests();
      profiles.clear();
      this.sourceErrorsState.set(new Map());
    } else {
      const inputs = new Map(
        previous.sources.map((source) => [source.id, JSON.stringify(source.input.serialize())]),
      );
      for (const source of plan.sources) {
        if (inputs.get(source.id) !== JSON.stringify(source.input.serialize())) {
          this.invalidateSourceRequest(source.id);
          profiles.delete(source.id);
          this.setSourceError(source.id, null);
        }
      }
      for (const source of previous.sources) {
        if (!plan.sources.some((item) => item.id === source.id)) {
          this.invalidateSourceRequest(source.id);
          profiles.delete(source.id);
          this.setSourceError(source.id, null);
        }
      }
    }
    this.sourceProfilesState.set(profiles);
    this.commitPlan(plan.clone());
  }

  updateOutput(update: (plan: Plan) => void): void {
    this.changePlan(update);
  }

  loadSource(sourceId: number, refresh = false): void {
    const source = this.plan().sources.find((item) => item.id === sourceId);
    if (!source) return;
    this.cancelEvaluation();
    this.buildGeneration++;
    this.buildSubscription?.unsubscribe();
    this.buildSubscription = null;
    this.buildingState.set(false);
    this.subscriptionUrlState.set(null);
    this.markPreviewStale();
    const generation = this.nextSourceGeneration(sourceId);
    const signature = this.sourceSignature(source);
    this.setSourceLoading(sourceId, true);
    this.setSourceError(sourceId, null);
    this.errorState.set(null);
    const subscription = this.api
      .loadSource(
        new LoadSourceRequest(
          source.id,
          this.plan().client,
          source.input,
          refresh ? 'refresh' : 'use',
        ),
      )
      .subscribe({
        next: (profile) => {
          if (!this.isCurrentSourceRequest(sourceId, generation, signature)) return;
          const profiles = new Map(this.sourceProfiles());
          profiles.set(sourceId, profile);
          this.sourceProfilesState.set(profiles);
          this.setSourceLoading(sourceId, false);
          this.sourceSubscriptions.delete(sourceId);
          this.scheduleEvaluation();
        },
        error: (error) => {
          if (!this.isCurrentSourceRequest(sourceId, generation, signature)) return;
          this.setSourceLoading(sourceId, false);
          this.sourceSubscriptions.delete(sourceId);
          const message = errorMessage(error);
          this.setSourceError(sourceId, message);
          this.errorState.set(message);
        },
      });
    if (!subscription.closed) this.sourceSubscriptions.set(sourceId, subscription);
  }

  reloadAllSources(refresh = false): void {
    const plan = this.plan();
    if (plan.sources.length === 0) return;
    this.cancelEvaluation();
    this.markPreviewStale();
    this.errorState.set(null);
    for (const source of plan.sources) this.loadSource(source.id, refresh);
  }

  evaluateNow(): void {
    this.cancelEvaluation();
    if (
      !this.sourceProfilesComplete() ||
      this.loadingSourceIds().size > 0 ||
      this.sourceErrors().size > 0
    )
      return;
    const plan = this.plan().clone();
    const sourceProfiles = plan.sources.map((source) => this.sourceProfiles().get(source.id)!);
    const revision = this.planRevision;
    const generation = ++this.evaluationGeneration;
    this.evaluatingState.set(true);
    this.previewStaleState.set(true);
    this.errorState.set(null);
    this.evaluationSubscription = this.api
      .evaluatePlan(new EvaluatePlanRequest(plan, sourceProfiles))
      .subscribe({
        next: (response) => {
          if (!this.isCurrentEvaluation(generation, revision)) return;
          this.evaluatingState.set(false);
          this.evaluationSubscription = null;
          this.reportState.set(response.report);
          this.renderedContentState.set(
            response.report.hasErrors ? null : response.rendered_content,
          );
          this.previewStaleState.set(false);
          if (response.report.hasErrors) this.subscriptionUrlState.set(null);
        },
        error: (error) => {
          if (!this.isCurrentEvaluation(generation, revision)) return;
          this.evaluatingState.set(false);
          this.evaluationSubscription = null;
          this.previewStaleState.set(true);
          this.subscriptionUrlState.set(null);
          this.errorState.set(errorMessage(error));
        },
      });
  }

  buildUrl(): void {
    if (!this.canBuildUrl()) return;
    const revision = this.planRevision;
    const generation = ++this.buildGeneration;
    this.buildSubscription?.unsubscribe();
    this.buildingState.set(true);
    this.errorState.set(null);
    this.buildSubscription = this.api
      .buildUrl(new BuildPlanUrlRequest(this.plan().clone()))
      .subscribe({
        next: (response) => {
          if (!this.isCurrentBuild(generation, revision)) return;
          this.buildingState.set(false);
          this.buildSubscription = null;
          this.subscriptionUrlState.set(response.url);
          this.dirtyState.set(false);
        },
        error: (error) => {
          if (!this.isCurrentBuild(generation, revision)) return;
          this.buildingState.set(false);
          this.buildSubscription = null;
          this.subscriptionUrlState.set(null);
          this.errorState.set(errorMessage(error));
        },
      });
  }

  restoreFromUrl(url: string): void {
    const revision = this.planRevision;
    const generation = ++this.restoreGeneration;
    this.restoreSubscription?.unsubscribe();
    this.restoringState.set(true);
    this.errorState.set(null);
    this.restoreSubscription = this.api.decodePlan(new DecodePlanUrlRequest(url)).subscribe({
      next: (response) => {
        if (!this.isCurrentRestore(generation, revision)) return;
        this.restoringState.set(false);
        this.restoreSubscription = null;
        this.replacePlan(response.plan, true);
        this.reloadAllSources();
      },
      error: (error) => {
        if (!this.isCurrentRestore(generation, revision)) return;
        this.restoringState.set(false);
        this.restoreSubscription = null;
        this.errorState.set(errorMessage(error));
      },
    });
  }

  clearError(): void {
    this.errorState.set(null);
  }

  ngOnDestroy(): void {
    this.destroyed = true;
    this.cancelEvaluation();
    for (const subscription of this.sourceSubscriptions.values()) subscription.unsubscribe();
    this.sourceSubscriptions.clear();
    this.buildSubscription?.unsubscribe();
    this.restoreSubscription?.unsubscribe();
  }

  clearDraft(): void {
    this.removeDraft();
    this.replacePlan(Plan.empty(), false);
    this.dirtyState.set(false);
  }

  restoreDraft(): void {
    const draft = this.readDraft();
    if (!draft) return;
    this.replacePlan(draft, false);
    this.reloadAllSources();
  }

  private changePlan(update: (plan: Plan) => void, clearProfiles = false): void {
    const plan = this.plan().clone();
    update(plan);
    if (clearProfiles) {
      this.invalidateAllSourceRequests();
      this.sourceProfilesState.set(new Map());
      this.sourceErrorsState.set(new Map());
    }
    this.commitPlan(plan);
  }

  private commitPlan(plan: Plan): void {
    this.bumpPlanRevision();
    this.planState.set(plan);
    this.markPreviewStale();
    this.subscriptionUrlState.set(null);
    this.dirtyState.set(true);
    this.writeDraft(plan);
    this.scheduleEvaluation();
  }

  private replacePlan(plan: Plan, generated: boolean): void {
    this.cancelEvaluation();
    this.invalidateAllSourceRequests();
    this.bumpPlanRevision();
    this.planState.set(plan);
    this.sourceProfilesState.set(new Map());
    this.sourceErrorsState.set(new Map());
    this.reportState.set(null);
    this.renderedContentState.set(null);
    this.previewStaleState.set(false);
    this.subscriptionUrlState.set(null);
    this.selectedSourceIdState.set(plan.sources[0]?.id ?? null);
    this.dirtyState.set(!generated);
    this.writeDraft(plan);
  }

  private dropSourceProfile(sourceId: number): void {
    this.invalidateSourceRequest(sourceId);
    const profiles = new Map(this.sourceProfiles());
    profiles.delete(sourceId);
    this.sourceProfilesState.set(profiles);
    this.setSourceError(sourceId, null);
  }

  private setSourceLoading(sourceId: number, loading: boolean): void {
    const ids = new Set(this.loadingSourceIds());
    if (loading) ids.add(sourceId);
    else ids.delete(sourceId);
    this.loadingSourceIdsState.set(ids);
  }

  private scheduleEvaluation(): void {
    this.cancelEvaluation();
    if (
      !this.sourceProfilesComplete() ||
      this.loadingSourceIds().size > 0 ||
      this.sourceErrors().size > 0
    )
      return;
    this.evaluationTimer = setTimeout(() => this.evaluateNow(), 300);
  }

  private cancelEvaluation(): void {
    this.evaluationGeneration++;
    if (this.evaluationTimer !== null) clearTimeout(this.evaluationTimer);
    this.evaluationTimer = null;
    this.evaluationSubscription?.unsubscribe();
    this.evaluationSubscription = null;
    this.evaluatingState.set(false);
  }

  private bumpPlanRevision(): void {
    this.planRevision++;
    this.buildGeneration++;
    this.restoreGeneration++;
    if (this.buildingState()) {
      this.buildSubscription?.unsubscribe();
      this.buildSubscription = null;
      this.buildingState.set(false);
    }
    if (this.restoringState()) {
      this.restoreSubscription?.unsubscribe();
      this.restoreSubscription = null;
      this.restoringState.set(false);
    }
  }

  private markPreviewStale(): void {
    if (this.reportState() !== null || this.renderedContentState() !== null)
      this.previewStaleState.set(true);
  }

  private nextSourceGeneration(sourceId: number): number {
    this.sourceSubscriptions.get(sourceId)?.unsubscribe();
    const generation = (this.sourceGenerations.get(sourceId) ?? 0) + 1;
    this.sourceGenerations.set(sourceId, generation);
    return generation;
  }

  private invalidateSourceRequest(sourceId: number): void {
    this.sourceGenerations.set(sourceId, (this.sourceGenerations.get(sourceId) ?? 0) + 1);
    this.sourceSubscriptions.get(sourceId)?.unsubscribe();
    this.sourceSubscriptions.delete(sourceId);
    this.setSourceLoading(sourceId, false);
  }

  private invalidateAllSourceRequests(): void {
    const sourceIds = new Set([
      ...this.sourceGenerations.keys(),
      ...this.sourceSubscriptions.keys(),
      ...this.loadingSourceIds(),
    ]);
    for (const sourceId of sourceIds) this.invalidateSourceRequest(sourceId);
  }

  private sourceSignature(source: Source): string {
    return JSON.stringify({ client: this.plan().client, input: source.input.serialize() });
  }

  private isCurrentSourceRequest(sourceId: number, generation: number, signature: string): boolean {
    if (this.destroyed || this.sourceGenerations.get(sourceId) !== generation) return false;
    const source = this.plan().sources.find((item) => item.id === sourceId);
    return source !== undefined && this.sourceSignature(source) === signature;
  }

  private isCurrentEvaluation(generation: number, revision: number): boolean {
    return (
      !this.destroyed && generation === this.evaluationGeneration && revision === this.planRevision
    );
  }

  private isCurrentBuild(generation: number, revision: number): boolean {
    return !this.destroyed && generation === this.buildGeneration && revision === this.planRevision;
  }

  private isCurrentRestore(generation: number, revision: number): boolean {
    return (
      !this.destroyed && generation === this.restoreGeneration && revision === this.planRevision
    );
  }

  private setSourceError(sourceId: number, error: string | null): void {
    const errors = new Map(this.sourceErrors());
    if (error === null) errors.delete(sourceId);
    else errors.set(sourceId, error);
    this.sourceErrorsState.set(errors);
  }

  private readDraft(): Plan | null {
    try {
      const value = globalThis.localStorage?.getItem(DRAFT_KEY);
      return value ? Plan.deserialize(JSON.parse(value)) : null;
    } catch {
      this.removeDraft();
      return null;
    }
  }

  private writeDraft(plan: Plan): void {
    try {
      globalThis.localStorage?.setItem(DRAFT_KEY, JSON.stringify(plan.serialize()));
    } catch {
      /* storage may be unavailable */
    }
  }

  private removeDraft(): void {
    try {
      globalThis.localStorage?.removeItem(DRAFT_KEY);
    } catch {
      /* storage may be unavailable */
    }
  }
}

function moveById<T extends { id: number }>(items: T[], id: number, offset: -1 | 1): void {
  const index = items.findIndex((item) => item.id === id);
  const target = index + offset;
  if (index < 0 || target < 0 || target >= items.length) return;
  [items[index], items[target]] = [items[target], items[index]];
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function sourceInputReady(input: SourceInput): boolean {
  if (input instanceof RemoteSourceInput) return input.url.trim().length > 0;
  if (input instanceof InlineSourceInput) return input.content.trim().length > 0;
  return false;
}
