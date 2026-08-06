# Releasing

Releases are published from GitHub Actions
([`.github/workflows/publish.yml`](.github/workflows/publish.yml)). The workflow
runs every release check, uploads the crate, verifies the archive served by
crates.io, and only then creates the matching `<crate>-vX.Y.Z` tag and GitHub
release. Do not run `cargo publish` or create the tag locally.

This workspace holds eight publishable crates that share one version. Two
things follow from that, and they shape everything below:

- **One crate per workflow run.** The dispatch takes a crate selector, because
  crates.io rejects a crate whose path dependencies are not on the registry
  yet. The crates therefore go out bottom-up, one at a time.
- **Tags are `<crate>-vX.Y.Z`**, not `vX.Y.Z` — eight crates cannot share one
  tag.

`Cargo.lock` is not committed, so the release checks intentionally do not use
`--locked`.

## Publish order

| # | Crate | Depends on |
|---|---|---|
| 1 | `prebindgen` | — |
| 2 | `prebindgen-c-runtime` | — (no dependencies at all) |
| 3 | `prebindgen-jni-runtime` | `jni` only |
| 4 | `prebindgen-proc-macro` | `prebindgen` |
| 5 | `prebindgen-flat` | `prebindgen` |
| 6 | `prebindgen-registry` | `prebindgen-flat`, `prebindgen` |
| 7 | `prebindgen-c` | `prebindgen-registry`, `prebindgen-c-runtime` |
| 8 | `prebindgen-jni` | `prebindgen-registry`, `prebindgen-jni-runtime`, `kotlin-codegen` |

1–3 have no workspace dependencies and may go in any order among themselves.
Every crate after them needs its predecessors live on crates.io first, so wait
for each version to appear on the registry before dispatching the next.

The CI `package` job runs `cargo package` for all eight on every pull request,
so metadata breakage — a missing description or license, a path dependency with
no version — surfaces on the PR that causes it rather than at release time. It
tolerates the "dependency not yet published" error, which says nothing about
the crate under test.

### External prerequisite

[`kotlin-codegen`](https://github.com/milyin/kotlin-codegen) lives in its own
repo with its own release cycle and is consumed from crates.io like any other
dependency. `prebindgen-jni` can only be published against a `kotlin-codegen`
version that is already on the registry; if a release needs unreleased
`kotlin-codegen` changes, release that crate first, following
[its own `RELEASING.md`](https://github.com/milyin/kotlin-codegen/blob/main/RELEASING.md).

## First publication: 0.5.0

Trusted Publishing cannot create a crate that does not exist on crates.io yet.
The first publication of each crate therefore uses a scoped crates.io API token
while still running the complete release from CI.

### Configure crates.io and GitHub

1. Sign in to crates.io with the account that will own the `prebindgen` crates.
   Verify the account email if crates.io requests it.
2. Confirm that all eight names are still free on crates.io.
3. Create a crates.io API token that is allowed to publish new crates. Give it
   a short expiration because it is needed only for the first release round.
4. In **GitHub → prebindgen → Settings → Environments**, create an environment
   named `crates-io`.
5. Add the API token to that environment as a secret named
   `CARGO_REGISTRY_TOKEN`. Do not set the `CRATES_IO_TRUSTED_PUBLISHING`
   variable yet.
6. Optionally add a required reviewer to the environment so publication needs
   explicit approval. With eight dispatches in a row, this means eight
   approvals.

### Prepare and publish 0.5.0

1. Confirm that the workspace `Cargo.toml` contains `package.version = "0.5.0"`
   and that every workspace dependency requirement matches it.
2. Run the release checks locally:

   ```console
   cargo fmt --all -- --check \
     --config "unstable_features=true,imports_granularity=Crate,group_imports=StdExternalCrate"
   cargo clippy --all-targets --all-features -- -D warnings
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   cargo test --all --all-features
   ./examples/regen-check.sh
   cargo package -p <crate>
   ```

   Clippy and fmt are worth running on both `1.85.0` (the MSRV) and `stable`,
   as CI does — a stable-only run misses MSRV lints.
3. Merge the release-preparation PR into `main` only after CI passes. The
   workflow refuses to dispatch from any branch but `main`.
4. **Rehearse the whole order first.** For each crate in the order above, open
   **Actions → Publish to crates.io → Run workflow**, select `main`, choose the
   crate, enter `0.5.0`, and enable **dry_run**. This runs every check and
   `cargo publish --dry-run` without publishing, tagging, or releasing.
5. Publish for real, one crate at a time in the same order: same dispatch with
   **dry_run** unchecked.
6. Approve the `crates-io` environment deployment if approval is required.
7. After each run, confirm all three release artifacts:

   - the crate at version `0.5.0` on crates.io;
   - tag `<crate>-v0.5.0` pointing to the published commit;
   - GitHub release `<crate>-v0.5.0`.

8. Wait for the version to be queryable on crates.io before dispatching the
   next crate.

The workflow verifies that the downloaded crates.io archive records the same
Git commit before it creates the tag. If publication succeeds but a later step
fails, rerun the workflow with the same crate and version; it resumes without
uploading the version again.

## Switch to Trusted Publishing after the first release

Once a crate exists on crates.io:

1. Open that crate's **Settings → Trusted Publishing** page.
2. Add a GitHub Actions publisher with these exact values:

   - repository owner: `milyin`
   - repository: `prebindgen`
   - workflow: `publish.yml`
   - environment: `crates-io`

3. Repeat for each of the eight crates — Trusted Publishing is configured per
   crate, not per repository.
4. In the GitHub `crates-io` environment, add the variable
   `CRATES_IO_TRUSTED_PUBLISHING` with the value `true`.
5. Delete the `CARGO_REGISTRY_TOKEN` secret from the environment.
6. After one later release succeeds through Trusted Publishing, optionally
   enable Trusted-Publishing-Only mode in each crate's settings.

Later publication jobs exchange GitHub's OIDC identity for a short-lived
crates.io token. No permanent crates.io credential remains in GitHub.

Set the variable only after **every** crate has a Trusted Publishing entry: it
is repository-wide, so a crate still missing its entry would fail
authentication on the next dispatch.

## Publish a later version

1. In a release-preparation PR:

   - update `package.version` and every `prebindgen*` workspace dependency
     requirement in the workspace `Cargo.toml`;
   - update the README, `docs/`, and API documentation as needed;
   - run the local release checks shown above.

2. Merge the PR into `main` after CI passes.
3. Run **Actions → Publish to crates.io** from `main` once per crate, in the
   publish order above, with the exact manifest version and without the `v`
   prefix. Enable **dry_run** first if you want a rehearsal.
4. Confirm the crates.io version, `<crate>-v<version>` tag, GitHub release, and
   docs.rs documentation for each.

A crate whose code did not change still has to be published: the eight share
one version, and the later crates' dependency requirements point at it.

## Recover from a partial release

Rerunning the workflow with the same crate and version is safe:

- If crates.io does not contain the version, the workflow publishes it.
- If it is already published from the current commit, publication is skipped
  and release creation resumes.
- If it was published from another commit, the workflow stops.
- An existing tag is reused only if it points to the published commit.

If the sequence breaks partway through, resume at the first crate that is not
yet on crates.io — the ones already published need nothing.

Published versions cannot be replaced. Correct a bad release by publishing a
new version; never move a published version's tag.
