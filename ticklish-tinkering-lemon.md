# Unified Wizard Flow — Merge Wizard + Onboarding

## Context
Currently there are two separate flows: a 6-step **wizard** (install, dirs, DB) and a 7-step **onboarding** (profile, keys, contribution, network check). The user wants a single unified flow that includes node naming, slug generation, Cloudflare tunnel setup (with skip option), and all the onboarding steps. The old two-phase system creates a disconnect where tunnel setup only happens post-setup in AdminPanel.

## New Unified Flow (10 steps)

| # | Step | Component | Source |
|---|------|-----------|--------|
| 0 | Welcome | WelcomeStep | existing wizard |
| 1 | License | LicenseStep | existing wizard |
| 2 | Node Identity | **NodeIdentityStep** (NEW) | replaces CreateProfile + adds slug |
| 3 | Location | LocationStep | existing wizard |
| 4 | Contribution | ContributionSlider | from onboarding (adapted) |
| 5 | Service | ServiceStep | existing wizard |
| 6 | Tunnel Setup | **TunnelStep** (NEW) | moved from AdminPanel, with skip |
| 7 | Installing | ProgressStep | existing wizard (enhanced) |
| 8 | Network Check | NetworkCheck | from onboarding (adapted) |
| 9 | Complete | **CompleteStep** (NEW) | replaces SuccessStep + ReadyPanel |

## Detailed Step Designs

### Step 2 — NodeIdentityStep (NEW)
- **Node Name** input (e.g., "Merryweather Commons") → stored in configStore.nodeName
- **Node Slug** auto-generated from name (lowercase, hyphens, alphanumeric only)
  - e.g., "Merryweather Commons" → "merryweather-commons"
  - Editable but auto-derived
- **Node Type** selector (hub / client / personal) — moved from wizardStore where it was defaulting to 'client' silently
- Shows preview: `merryweather-commons.citinet.io`
- Validation: slug format (lowercase, hyphens, 3-63 chars)

### Step 6 — TunnelStep (NEW)
- Shows what will happen: "Your node will be accessible at `{slug}.citinet.io`"
- **CF API Token** input (password field)
- **"Skip for now"** button → proceeds without tunnel (local-only mode)
- **"Set Up Tunnel"** button → triggers:
  1. Install cloudflared (if needed)
  2. Create tunnel via `cloudflared tunnel create {slug}`
  3. Create DNS CNAME: `{slug}.citinet.io` → `{tunnel-id}.cfargotunnel.com`
  4. Generate tunnel config YAML (port 9090 — hub service port)
  5. Start tunnel
- Progress indicators for each sub-step
- On success: shows green checkmark + public URL
- On skip: stores flag `tunnelSkipped = true`

### Step 7 — ProgressStep (Enhanced)
- Same as current but now also:
  - Passes `nodeName` (from NodeIdentityStep) instead of default
  - Passes contribution values synced from ContributionSlider
  - If tunnel was set up in step 6, starts the tunnel service

### Step 9 — CompleteStep (NEW)
- Replaces both SuccessStep and ReadyPanel
- Shows summary: node name, type, storage pledged, public URL (or "local only")
- Single "Open Dashboard" button → `setPhase("dashboard")` (no more "onboarding" phase)

## Files to Modify

### Stores
- **`src/stores/wizardStore.ts`** — Add fields: `nodeName`, `nodeSlug`, `cfApiToken`, `tunnelSkipped`. Update TOTAL_STEPS to 10.
- **`src/stores/appStore.ts`** — Remove "onboarding" phase. Wizard goes directly to "dashboard".
- **`src/stores/onboardingStore.ts`** — DELETE (no longer needed; fields moved to wizardStore)

### New Components
- **`src/components/wizard/NodeIdentityStep.tsx`** — Node name + slug + type selector
- **`src/components/wizard/TunnelStep.tsx`** — CF tunnel setup with skip option
- **`src/components/wizard/CompleteStep.tsx`** — Final summary → dashboard

### Modified Components
- **`src/components/wizard/Wizard.tsx`** — New step array (10 steps)
- **`src/components/wizard/WizardLayout.tsx`** — Update STEP_LABELS to 10 steps
- **`src/components/wizard/ProgressStep.tsx`** — Use wizardStore.nodeName, sync contribution
- **`src/components/onboarding/ContributionSlider.tsx`** — Move to wizard dir, use wizardStore instead of onboardingStore
- **`src/components/onboarding/NetworkCheck.tsx`** — Move to wizard dir, use wizardStore
- **`src/App.tsx`** — Remove onboarding import/case

### Components to DELETE
- `src/components/wizard/SuccessStep.tsx` — Replaced by CompleteStep
- `src/components/onboarding/Onboarding.tsx` — No longer needed
- `src/components/onboarding/OnboardingLayout.tsx` — No longer needed
- `src/components/onboarding/WelcomePanel.tsx` — Redundant with wizard welcome
- `src/components/onboarding/CreateProfile.tsx` — Merged into NodeIdentityStep
- `src/components/onboarding/IdentityKeys.tsx` — Moved into ProgressStep (auto-generate during install)
- `src/components/onboarding/CommunityPanel.tsx` — Stats shown in CompleteStep
- `src/components/onboarding/ReadyPanel.tsx` — Merged into CompleteStep

### Backend
- No Rust changes needed — all tunnel commands already exist (`install_cloudflared`, `setup_tunnel`, `start_tunnel`)
- The frontend TunnelStep will call existing APIs in sequence

## Key Implementation Details

### Slug Generation (frontend utility)
```typescript
function generateSlug(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 63);
}
```

### Tunnel Port
- Hardcode to 9090 (hub_service.rs default port) — no user input needed

### Phase Simplification
- `AppPhase = "wizard" | "dashboard"` (remove "onboarding")
- Wizard completes → dashboard directly

### WizardStore Additions
```typescript
nodeName: string;      // "Merryweather Commons"
nodeSlug: string;      // "merryweather-commons"
cfApiToken: string;    // Cloudflare API token
tunnelSkipped: boolean; // true if user skipped tunnel step
```

## Verification
1. `cargo build` — should still pass (no backend changes)
2. `npx tsc --noEmit` — TypeScript check
3. `npm run test` — existing tests pass
4. `npm run dev` — walk through full wizard:
   - Welcome → License → Node Identity (enter name, see slug preview) → Location → Contribution → Service → Tunnel (test skip + test with token) → Installing → Network Check → Complete → Dashboard
5. Verify localStorage `citinet-phase` goes from "wizard" to "dashboard" (no "onboarding")
6. Verify node name appears in dashboard after setup
