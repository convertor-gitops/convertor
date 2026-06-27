# Casebook

## Purpose

Use this reference for known network incident patterns and for recording new cases in the `convertor-gitops` repository.

Repository case store:

```text
.github/agents/homelab-network-doctor/cases/
```

Schema:

```text
.github/agents/homelab-network-doctor/case.schema.json
```

Validation:

```bash
just verify-network-doctor-cases
```

## Recording Rules

Only record a case when the user explicitly asks to沉淀, record, or update the case set.

When recording:

- Create a new JSON file named like `case-id.json` under `.github/agents/homelab-network-doctor/cases/`.
- Do not edit existing case files unless the user explicitly asks for an update.
- Do not modify the legacy Copilot agent file, schema, guard script, hook file, justfile, or business configuration as part of case recording.
- Include real evidence from the current incident.
- Include at least one failure signal and one ruled-out branch.
- Use `approved` only when the user confirmed the root cause or the repair was verified.
- Run `just verify-network-doctor-cases` before final response.

## Known Cases

Load the matching JSON file before diagnosing a similar incident.

### `vultr-notready-tailscale-down`

File:

```text
.github/agents/homelab-network-doctor/cases/vultr-notready-tailscale-down.json
```

Pattern:

- `node-vultr-worker-101` becomes NotReady.
- `talosctl` to `100.64.100.31` fails with no route to host.
- `tailscale` subnet router has Tailscale stopped or not running.
- `gt-ac5300` still has `100.64.0.0/10 via 10.0.1.7`.

Recovery:

- Start or bring up Tailscale on `10.0.1.7`.
- Verify peer state, Talos services, and K8s node readiness.

### `vultr-notready-tailscale-authkey-expired-then-power-reboot`

File:

```text
.github/agents/homelab-network-doctor/cases/vultr-notready-tailscale-authkey-expired-then-power-reboot.json
```

Pattern:

- `node-vultr-worker-101` is NotReady.
- `talosctl -n 100.64.100.31` initially fails with no route to host.
- `tailscaled` on `tailscale` reports `Needs login`.
- `tailscale-autoconnect` fails with `invalid key: API key does not exist`.
- `gt-ac5300` static route is present.
- Interactive `tailscale up` restores the peer, but `talosctl -n 100.64.100.31` still times out.
- Public Talos API may return `authentication handshake failed: EOF`.

Recovery:

- Restore `tailscale` login with interactive `tailscale up` or a fresh auth key.
- Retest `talosctl`; do not stop after Tailscale peer recovery.
- If Talos API remains unavailable, coordinate a Vultr power reboot.
- Verify `talosctl version`, Talos services, `kubectl get nodes`, and Vultr-hosted pods.
- Long-term fix: disable key expiry on the `tailscale` machine or use a tagged device.

## Case ID Guidance

Use stable, lowercase, hyphenated IDs that encode symptom and root cause:

```text
vultr-notready-tailscale-authkey-expired-then-power-reboot
home-pod-vultr-pod-mihomo-notrack-missing
ingress-502-infra-haproxy-nfs-missing
```
