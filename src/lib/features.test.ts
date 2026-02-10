import { describe, it, expect } from 'vitest';
import { resolveFlags, PROFILE_FLAGS, type FeatureFlag } from './features';

describe('PROFILE_FLAGS', () => {
  it('hub profile includes hub_mode and admin_panel', () => {
    expect(PROFILE_FLAGS.hub).toContain('hub_mode');
    expect(PROFILE_FLAGS.hub).toContain('admin_panel');
  });

  it('client profile includes contribution but not hub_mode', () => {
    expect(PROFILE_FLAGS.client).toContain('contribution');
    expect(PROFILE_FLAGS.client).not.toContain('hub_mode');
    expect(PROFILE_FLAGS.client).not.toContain('admin_panel');
  });

  it('personal profile has no contribution, hub_mode, or admin_panel', () => {
    expect(PROFILE_FLAGS.personal).not.toContain('contribution');
    expect(PROFILE_FLAGS.personal).not.toContain('hub_mode');
    expect(PROFILE_FLAGS.personal).not.toContain('admin_panel');
  });

  it('all profiles include common flags', () => {
    const commonFlags: FeatureFlag[] = ['file_sync', 'discussions', 'discover_tab', 'local_storage', 'auth', 'diagnostics'];
    for (const profile of ['hub', 'client', 'personal'] as const) {
      for (const flag of commonFlags) {
        expect(PROFILE_FLAGS[profile], `${profile} should include ${flag}`).toContain(flag);
      }
    }
  });
});

describe('resolveFlags', () => {
  it('resolves hub flags correctly', () => {
    const flags = resolveFlags('hub');
    expect(flags.hub_mode).toBe(true);
    expect(flags.admin_panel).toBe(true);
    expect(flags.contribution).toBe(false);
    expect(flags.file_sync).toBe(true);
    expect(flags.discussions).toBe(true);
  });

  it('resolves client flags correctly', () => {
    const flags = resolveFlags('client');
    expect(flags.hub_mode).toBe(false);
    expect(flags.admin_panel).toBe(false);
    expect(flags.contribution).toBe(true);
    expect(flags.file_sync).toBe(true);
  });

  it('resolves personal flags correctly', () => {
    const flags = resolveFlags('personal');
    expect(flags.hub_mode).toBe(false);
    expect(flags.admin_panel).toBe(false);
    expect(flags.contribution).toBe(false);
    expect(flags.file_sync).toBe(true);
    expect(flags.diagnostics).toBe(true);
  });

  it('returns a boolean for every flag', () => {
    const allFlags: FeatureFlag[] = [
      'hub_mode', 'file_sync', 'discussions', 'contribution',
      'admin_panel', 'discover_tab', 'local_storage', 'auth', 'diagnostics',
    ];
    for (const profile of ['hub', 'client', 'personal'] as const) {
      const flags = resolveFlags(profile);
      for (const flag of allFlags) {
        expect(typeof flags[flag], `${profile}.${flag} should be boolean`).toBe('boolean');
      }
    }
  });
});
