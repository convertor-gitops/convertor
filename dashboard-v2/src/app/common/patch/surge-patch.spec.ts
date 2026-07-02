import { describe, expect, it } from "vitest";
import { Policy } from "../model/core/policy";
import {
    bracketName,
    type RuleEntry,
    renderRuleEntry,
    parseRuleBlock,
    mergeRuleEntries,
    resolveDefault,
    renderRuleBlock,
    patchSurgeHeader,
} from "./surge-patch";

const SAMPLE = `#!MANAGED-CONFIG https://old.example.com/profile interval=43200 strict=true
[General]
loglevel = notify

# 转换器托管 rule-set
# Rule Provider from convertor
// [Subscription] by convertor/2.5.11
RULE-SET,"https://old.example.com/sub",DIRECT
// [BosLife] by convertor/2.5.11
RULE-SET,"https://old.example.com/bos",BosLife
// [DIRECT: no-resolve] by convertor/2.5.11
# RULE-SET,"https://old.example.com/dnr",DIRECT,no-resolve
# End of Rule Provider

[MITM]
skip-server-cert-verify = true`;

describe("bracketName", () => {
    it("subscription / plain / option", () => {
        expect(bracketName(new Policy("DIRECT", true))).toBe("[Subscription]");
        expect(bracketName(new Policy("BosLife", false))).toBe("[BosLife]");
        expect(bracketName(new Policy("BosLife", false, "no-resolve"))).toBe("[BosLife: no-resolve]");
    });
});

describe("parseRuleBlock", () => {
    it("解析出块、保留块外内容、识别禁用前缀", () => {
        const parsed = parseRuleBlock(SAMPLE);
        expect(parsed.hasBlock).toBe(true);
        expect(parsed.entries).toHaveLength(3);

        const sub = parsed.entries[0];
        expect(sub.label).toBe("[Subscription]");
        expect(sub.policyName).toBe("DIRECT");
        expect(sub.option).toBeNull();
        expect(sub.disabled).toBe(false);
        expect(sub.version).toBe("2.5.11");

        const dnr = parsed.entries[2];
        expect(dnr.label).toBe("[DIRECT: no-resolve]");
        expect(dnr.option).toBe("no-resolve");
        expect(dnr.disabled).toBe(true); // 手动禁用

        // 块外内容保留
        expect(parsed.before).toContain("# 转换器托管 rule-set");
        expect(parsed.after.join("\n")).toContain("[MITM]");
    });
});

describe("renderRuleEntry 往返", () => {
    it("渲染后再解析得到等价条目", () => {
        const entry: RuleEntry = {
            label: "[BosLife: no-resolve]",
            policyName: "BosLife",
            option: "no-resolve",
            url: "https://x.example.com/r",
            version: "2.6.20",
            disabled: true,
        };
        const text = renderRuleEntry(entry);
        expect(text).toBe(
            `// [BosLife: no-resolve] by convertor/2.6.20\n# RULE-SET,"https://x.example.com/r",BosLife,no-resolve`,
        );
        const reparsed = parseRuleBlock(
            `# Rule Provider from convertor\n${text}\n# End of Rule Provider`,
        );
        expect(reparsed.entries[0]).toEqual(entry);
    });
});

describe("mergeRuleEntries + resolveDefault", () => {
    const oldEntries = parseRuleBlock(SAMPLE).entries;
    // 模拟一次新 gen：Subscription 变了、BosLife 没了、新增 Netflix、DIRECT:no-resolve 远程现在启用
    const newEntries: RuleEntry[] = [
        { label: "[Subscription]", policyName: "DIRECT", option: null, url: "https://new.example.com/sub", version: "2.6.20", disabled: false },
        { label: "[DIRECT: no-resolve]", policyName: "DIRECT", option: "no-resolve", url: "https://new.example.com/dnr", version: "2.6.20", disabled: false },
        { label: "[Netflix]", policyName: "Netflix", option: null, url: "https://new.example.com/nf", version: "2.6.20", disabled: false },
    ];

    it("三态判定正确", () => {
        const rows = mergeRuleEntries(oldEntries, newEntries);
        const byLabel = new Map(rows.map(r => [r.label, r]));
        expect(byLabel.get("[Subscription]")!.status).toBe("changed");
        expect(byLabel.get("[BosLife]")!.status).toBe("removed");
        expect(byLabel.get("[DIRECT: no-resolve]")!.status).toBe("changed");
        expect(byLabel.get("[Netflix]")!.status).toBe("added");
    });

    it("默认合并：保住手动禁用、丢弃 removed、更新 url", () => {
        const rows = mergeRuleEntries(oldEntries, newEntries);
        const final = resolveDefault(rows);
        const byLabel = new Map(final.map(e => [e.label, e]));

        // 关键：DIRECT:no-resolve 远程虽启用，但本地手动禁用要保住
        const dnr = byLabel.get("[DIRECT: no-resolve]")!;
        expect(dnr.disabled).toBe(true);
        expect(dnr.url).toBe("https://new.example.com/dnr"); // url 仍更新到新值

        // removed 的不出现
        expect(byLabel.has("[BosLife]")).toBe(false);
        // added 的出现且启用
        expect(byLabel.get("[Netflix]")!.disabled).toBe(false);
    });
});

describe("renderRuleBlock 保留块外内容", () => {
    it("只替换块内、before/after 不动", () => {
        const parsed = parseRuleBlock(SAMPLE);
        const final = resolveDefault(mergeRuleEntries(parsed.entries, parsed.entries));
        const out = renderRuleBlock(parsed, final);
        expect(out).toContain("# 转换器托管 rule-set");
        expect(out).toContain("[MITM]");
        expect(out).toContain("# Rule Provider from convertor");
        expect(out).toContain("# End of Rule Provider");
    });
});

describe("patchSurgeHeader", () => {
    it("替换首行为新 header", () => {
        const out = patchSurgeHeader(SAMPLE, "#!MANAGED-CONFIG https://new/profile interval=86400 strict=false");
        expect(out.split("\n")[0]).toBe("#!MANAGED-CONFIG https://new/profile interval=86400 strict=false");
        expect(out).toContain("[General]"); // 其余不动
    });
});
