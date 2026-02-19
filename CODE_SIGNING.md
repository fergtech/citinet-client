# Code Signing Policy

Free code signing for Citinet is provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

## Signed Artifacts

The following artifacts are signed for each release:

- Windows installer (`.msi` / `.exe`)
- Main application executable (`Citinet.exe`)

## Team Roles

| Role | Members |
|------|---------|
| Committers & Reviewers | [Members](https://github.com/fergtech/citinet-client/graphs/contributors) |
| Approvers | [Owners](https://github.com/fergtech/citinet-client/people?query=role%3Aowner) |

## Privacy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

See [PRIVACY.md](./PRIVACY.md) for the full privacy policy.

## Third-Party Binaries

Citinet includes or downloads the `cloudflared` binary published by Cloudflare, Inc. under the Apache 2.0 license. This binary is signed by Cloudflare and is not signed under the Citinet/SignPath certificate. It is included as an upstream OSS dependency per SignPath Foundation policy.

## Build & Verification

Releases are built from source via GitHub Actions. Each signing request corresponds to a tagged release commit in this repository and requires manual approval by a project owner before signing proceeds.
