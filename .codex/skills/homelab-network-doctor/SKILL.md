---
name: homelab-network-doctor
description: Diagnose and recover convertor-gitops homelab network incidents across Tailscale subnet routing, mihomo, gt-ac5300/gs7 routes, HAProxy/NPM, DNS fake-ip, Pod networking, coredns-vultr, DERP, NOTRACK, static routes, and Vultr Talos NotReady. Use when the user reports network unreachable, kubectl/talosctl/Ingress/Harbor failures, home to Vultr connectivity, Pod DNS/outbound problems, or asks to record/update homelab network cases.
---

# Homelab Network Doctor

## Overview

Use this skill for homelab network incidents that affect the `convertor` service or the sibling GitOps repository at `/Users/bppleman/RustroverProjects/convertor-gitops`. When invoked from `/Users/bppleman/RustroverProjects/convertor`, keep the Rust service source in the current repository, but use `../convertor-gitops` as the source of truth for Kubernetes, Helm, ArgoCD, Talos, NixOS, Tailscale, mihomo, DNS, and case evidence.

## Operating Rules

- Prefer Chinese in user-facing replies.
- Start from the current repository root for service-code inspection. For homelab infrastructure evidence, switch/read from `../convertor-gitops` unless the user explicitly provides another checkout.
- Read `AGENTS.md` first if it has not already been loaded in the current turn.
- Keep diagnosis read-only until the failure branch is isolated. When the user explicitly asks to fix, execute the smallest reversible repair that fits repository policy; ask or coordinate for cloud power actions, production-impacting changes, credentials, or missing tokens.
- Never try SSH for Talos nodes. Use `talosctl --talosconfig talos/homelab/base/talosconfig -n <ip>`.
- Do not print secrets. In particular, redact Tailscale `TS_AUTHKEY`, Kubernetes Secret `data`, DERP keys, and API tokens.
- Do not commit or push Git changes.
- For internet-facing commands, use the repository proxy convention such as `pr curl ...`; SSH does not use a proxy.

## Workflow

1. Establish the minimal incident shape: source, target, symptom, start time, and recent changes. If the user already supplied enough context, proceed without stopping.
2. Load `references/diagnostic-runbook.md` before running network probes. Use it to separate F1-F6, E2E, and NPM flows.
3. Check matching prior cases before deep exploration. Load `references/casebook.md`, then open the relevant JSON files under `.github/agents/homelab-network-doctor/cases/`.
4. Run only targeted probes. Keep each result tied to a branch: confirmed, ruled out, or still unknown.
5. If repair is requested, state the exact action and why it is the smallest useful action. Then execute it when it is automatable and allowed by `AGENTS.md`; otherwise give copyable commands or ask for the missing credential/action.
6. Verify at the layer that failed and at the user-visible layer. For example, after a Vultr Talos recovery, verify Tailscale peer state, `talosctl`, `kubectl get nodes`, and affected pods.
7. End with a concise diagnosis report: symptom, locked flow, verified facts, ruled-out branches, root cause, repair, and residual risk.

## Reference Loading

- `references/diagnostic-runbook.md`: load for any live network diagnosis or recovery.
- `references/casebook.md`: load when symptoms match prior incidents, when the user asks to record a case, or after a recovery that may be worth preserving.
- Repository Talos guidance: load `.github/skills/homelab-talos/SKILL.md` when touching Talos lifecycle, config rendering, `talos-wrapper`, `config-node`, `apply-node`, or Vultr Talos details.
- Repository NixOS guidance: load `.github/skills/homelab-nixos/SKILL.md` when touching `nixos/tailscale`, `nixos/mihomo`, `nixos/infra`, NFS mounts, or NixOS service deployment.
- Repository overview: load `.github/skills/homelab-overview/references/architecture.md` when topology or ownership is uncertain.

## Installation Diagnostic

The repository source for this skill should live at `.codex/skills/homelab-network-doctor`. For automatic Codex discovery, `~/.codex/skills/homelab-network-doctor` should be a symlink to that repository path. If a future session is told to use `$homelab-network-doctor` but cannot find it under `~/.codex/skills`, assume the current machine may be missing the symlink and inspect the repository copy.

## Case Recording

Only record a case when the user explicitly asks to沉淀, record, or update the network case set. Then:

1. Create a new `.json` file under `.github/agents/homelab-network-doctor/cases/`; do not edit existing cases unless the user explicitly asks and the repository policy allows it.
2. Follow `.github/agents/homelab-network-doctor/case.schema.json`.
3. Use `approved` only when the user confirmed the root cause or the repair was verified. Otherwise use `pending-review`.
4. Include at least one failure signal and one ruled-out branch.
5. Run:

```bash
just verify-network-doctor-cases
```

## High-Value Pitfalls

- `dig` returning `198.18.x.x` from home Pods is expected mihomo fake-ip behavior; use `curl` to verify actual connectivity.
- `kubectl` or `talosctl` failing does not always mean Kubernetes is broken. Check infra HAProxy, Tailscale path, static routes, and Talos API separately.
- Fixing Tailscale does not prove Vultr Talos recovered. Retest `talosctl` and K8s readiness after the tailnet path is restored.
- `tailscale-autoconnect` can fail with `invalid key: API key does not exist` when `TS_AUTHKEY` is stale. Long-term fix is disabling key expiry for the subnet router machine or using a tagged device, not relying on a permanent auth key.
