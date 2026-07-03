export interface DiffSpan {
    text: string;
    type: "same" | "removed" | "added";
}

export interface InlineDiff {
    old: DiffSpan[];
    new: DiffSpan[];
}

/**
 * 轻量字符级 inline diff：取最长公共前缀 + 最长公共后缀，中间段标为 removed/added。
 * 对 rule-provider URL 这类"绝大部分相同、只有 sub_url 一段不同"的场景足够精确，
 * 既能把密文 churn 收敛成一小段高亮，也能让真正的字段变化(版本号、policy)显出来。
 */
export function inlineDiff(oldStr: string, newStr: string): InlineDiff {
    let prefix = 0;
    const max = Math.min(oldStr.length, newStr.length);
    while (prefix < max && oldStr[prefix] === newStr[prefix]) {
        prefix++;
    }

    let suffix = 0;
    while (
        suffix < oldStr.length - prefix &&
        suffix < newStr.length - prefix &&
        oldStr[oldStr.length - 1 - suffix] === newStr[newStr.length - 1 - suffix]
        ) {
        suffix++;
    }

    const head = oldStr.slice(0, prefix);
    const tail = oldStr.slice(oldStr.length - suffix);
    const oldMid = oldStr.slice(prefix, oldStr.length - suffix);
    const newMid = newStr.slice(prefix, newStr.length - suffix);

    const build = (mid: string, midType: "removed" | "added"): DiffSpan[] =>
        [
            { text: head, type: "same" as const },
            { text: mid, type: midType },
            { text: tail, type: "same" as const },
        ].filter(span => span.text.length > 0);

    return {
        old: build(oldMid, "removed"),
        new: build(newMid, "added"),
    };
}
