import { ConvUrl } from "../model/core/conv-url";
import { Policy } from "../model/core/policy";

// 与 Rust 侧 surge_renderer.rs 的常量保持一致
export const RULE_BLOCK_START = "# Rule Provider from convertor";
export const RULE_BLOCK_END = "# End of Rule Provider";
export const MANAGED_CONFIG_PREFIX = "#!MANAGED-CONFIG";

/**
 * 一条 rule-provider 条目：由「// 注释行」+「RULE-SET 行」成对构成。
 * label 是对齐 key（= Policy.bracket_name），合并时按它匹配新旧。
 */
export interface RuleEntry {
    /** [Subscription] / [BosLife: no-resolve] —— 对齐 key */
    label: string;
    /** RULE-SET 行末的策略名（DIRECT / BosLife …） */
    policyName: string;
    /** 策略选项（no-resolve / force-remote-dns …），无则 null */
    option: string | null;
    /** rule-provider URL（不含两侧引号） */
    url: string;
    /** 注释里的 `by convertor/<version>` 版本，无则 null */
    version: string | null;
    /** RULE-SET 行是否以 `# ` 开头（手动禁用） */
    disabled: boolean;
}

export type MergeStatus = "unchanged" | "changed" | "added" | "removed";

/** 一行合并对比：旧条目 / 新条目 / 状态。removed 时 newEntry 为 null，added 时 oldEntry 为 null。 */
export interface MergeRow {
    label: string;
    status: MergeStatus;
    oldEntry: RuleEntry | null;
    newEntry: RuleEntry | null;
}

/** 复现 Rust `Policy::bracket_name`：[Subscription] / [name] / [name: option] */
export function bracketName(policy: Policy): string {
    const parts = policy.is_subscription ? ["Subscription"] : [policy.name];
    if (policy.option) {
        parts.push(policy.option);
    }
    return `[${parts.join(": ")}]`;
}

/** 把一条 RuleEntry 渲染成两行文本（注释行 + RULE-SET 行，禁用则 RULE-SET 行加 `# ` 前缀） */
export function renderRuleEntry(entry: RuleEntry): string {
    const suffix = entry.version ? ` by convertor/${entry.version}` : "";
    const comment = `// ${entry.label}${suffix}`;
    const policyPart = entry.option ? `${entry.policyName},${entry.option}` : entry.policyName;
    const rule = `${entry.disabled ? "# " : ""}RULE-SET,"${entry.url}",${policyPart}`;
    return `${comment}\n${rule}`;
}

/** 从 gen 得到的 rule-provider ConvUrl 列表构造新条目（默认启用，版本统一为 version） */
export function entriesFromUrls(urls: ConvUrl[], version: string | null): RuleEntry[] {
    const entries: RuleEntry[] = [];
    for (const url of urls) {
        const policy = url.query?.policy;
        if (!policy) {
            continue;
        }
        entries.push({
            label: bracketName(policy),
            policyName: policy.name,
            option: policy.option ?? null,
            url: url.toString(),
            version,
            disabled: false,
        });
    }
    return entries;
}

const COMMENT_RE = /^\/\/\s*(\[[^\]]*\])(?:\s+by\s+convertor\/(\S+))?\s*$/;
const RULE_RE = /^(#\s*)?RULE-SET\s*,\s*"([^"]*)"\s*,\s*(.+?)\s*$/;

/** 解析一对「注释行 + RULE-SET 行」为 RuleEntry；任一不匹配则返回 null */
function parseEntry(commentLine: string, ruleLine: string): RuleEntry | null {
    const cm = COMMENT_RE.exec(commentLine.trim());
    const rm = RULE_RE.exec(ruleLine.trim());
    if (!cm || !rm) {
        return null;
    }
    const label = cm[1];
    const version = cm[2] ?? null;
    const disabled = !!rm[1];
    const url = rm[2];
    const tail = rm[3].split(",").map(s => s.trim()).filter(s => s.length > 0);
    const policyName = tail[0] ?? "";
    const option = tail.length > 1 ? tail[1] : null;
    return { label, policyName, option, url, version, disabled };
}

export interface ParsedBlock {
    /** 是否找到了 START/END 成对标记 */
    hasBlock: boolean;
    /** START 标记之前的所有行（含用户手写的 `# 转换器托管 rule-set` 等） */
    before: string[];
    /** 块内解析出的条目 */
    entries: RuleEntry[];
    /** END 标记之后的所有行 */
    after: string[];
}

/** 解析 rules.dconf：定位 START/END 块、提取其间的条目，块外内容原样保留 */
export function parseRuleBlock(content: string): ParsedBlock {
    const lines = content.split("\n");
    const startIdx = lines.findIndex(l => l.trim() === RULE_BLOCK_START);
    const endIdx = lines.findIndex(l => l.trim() === RULE_BLOCK_END);
    if (startIdx === -1 || endIdx === -1 || endIdx < startIdx) {
        return { hasBlock: false, before: lines, entries: [], after: [] };
    }

    const before = lines.slice(0, startIdx);
    const after = lines.slice(endIdx + 1);
    const inner = lines.slice(startIdx + 1, endIdx);

    const entries: RuleEntry[] = [];
    for (let i = 0; i < inner.length; i++) {
        const line = inner[i].trim();
        if (line.startsWith("//")) {
            const entry = parseEntry(inner[i], inner[i + 1] ?? "");
            if (entry) {
                entries.push(entry);
                i++; // 跳过已消费的 RULE-SET 行
            }
        }
    }
    return { hasBlock: true, before, entries, after };
}

/** 旧条目与新条目按 label 三态合并：保留旧顺序，新增追加到末尾，旧有新无标记 removed */
export function mergeRuleEntries(oldEntries: RuleEntry[], newEntries: RuleEntry[]): MergeRow[] {
    const newByLabel = new Map(newEntries.map(e => [e.label, e]));
    const oldByLabel = new Map(oldEntries.map(e => [e.label, e]));
    const rows: MergeRow[] = [];

    // 先按旧块顺序处理（保留/变更/删除），让 diff 噪音最小
    for (const oldEntry of oldEntries) {
        const newEntry = newByLabel.get(oldEntry.label) ?? null;
        if (!newEntry) {
            rows.push({ label: oldEntry.label, status: "removed", oldEntry, newEntry: null });
        } else {
            const status = entryBodyEquals(oldEntry, newEntry) ? "unchanged" : "changed";
            rows.push({ label: oldEntry.label, status, oldEntry, newEntry });
        }
    }
    // 再追加全新条目
    for (const newEntry of newEntries) {
        if (!oldByLabel.has(newEntry.label)) {
            rows.push({ label: newEntry.label, status: "added", oldEntry: null, newEntry });
        }
    }
    return rows;
}

/**
 * 默认合并策略：removed 丢弃、added 采用新条目、unchanged/changed 采用新条目但**继承旧的禁用状态**
 * （保住用户手动 `# ` 掉的规则）。三栏合并编辑器在此基础上让用户逐条覆盖。
 */
export function resolveDefault(rows: MergeRow[]): RuleEntry[] {
    const result: RuleEntry[] = [];
    for (const row of rows) {
        if (row.status === "removed") {
            continue;
        }
        if (row.status === "added") {
            result.push(row.newEntry!);
            continue;
        }
        // unchanged / changed：用新条目，但继承旧的 disabled
        result.push({ ...row.newEntry!, disabled: row.oldEntry!.disabled });
    }
    return result;
}

/** 比较两条是否实质相同（忽略 disabled，因为禁用是合并时单独决定的维度） */
function entryBodyEquals(a: RuleEntry, b: RuleEntry): boolean {
    return a.url === b.url && a.policyName === b.policyName && a.option === b.option && a.version === b.version;
}

/**
 * 把一组最终条目渲染回 START/END 块内，块外内容（before/after）原样保留。
 * 若原文件没有块，则在末尾追加一个新块。
 */
export function renderRuleBlock(parsed: ParsedBlock, finalEntries: RuleEntry[]): string {
    const body = finalEntries.map(renderRuleEntry).join("\n");
    const block = [RULE_BLOCK_START, body, RULE_BLOCK_END].filter(s => s.length > 0).join("\n");
    if (!parsed.hasBlock) {
        const head = parsed.before.length > 0 ? parsed.before.join("\n") + "\n" : "";
        return head + block + "\n";
    }
    const before = parsed.before.join("\n");
    const after = parsed.after.join("\n");
    return [before, block, after].filter(s => s.length > 0).join("\n");
}

/** 只渲染 START/END 之间的受管辖块（含首尾 marker），用于总预览——不含块外 before/after */
export function renderManagedBlock(entries: RuleEntry[]): string {
    const body = entries.map(renderRuleEntry).join("\n");
    return [RULE_BLOCK_START, body, RULE_BLOCK_END].filter(s => s.length > 0).join("\n");
}

/** 替换 Surge 主配置首行为新的 MANAGED-CONFIG header（对应 Rust update_surge_conf） */
export function patchSurgeHeader(content: string, headerLine: string): string {
    const lines = content.split("\n");
    if (lines.length === 0 || lines[0].trim().length === 0) {
        return [headerLine, ...lines.slice(lines.length === 0 ? 0 : 1)].join("\n");
    }
    lines[0] = headerLine;
    return lines.join("\n");
}

/** 从 profile_url（含 interval/strict 的 query）构造 MANAGED-CONFIG header 行 */
export function buildHeaderLine(profileUrl: ConvUrl): string {
    const query = profileUrl.query;
    const interval = query?.interval ?? 86400;
    const strict = query?.strict ?? true;
    return `${MANAGED_CONFIG_PREFIX} ${profileUrl.toString()} interval=${interval} strict=${strict}`;
}
