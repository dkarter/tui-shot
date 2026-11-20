# Development

## Build and test

Use a current stable Rust toolchain:

```sh
cargo build --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

To install from a local checkout, run `cargo install --locked --path .`. The
CLI exposes its machine-readable Usage spec with `tui-shot __usage_spec__`.

## README screenshots

Use `tui-shot` to capture live panes with sample data. Upload the PNGs to
[the gallery issue](https://github.com/dkarter/tui-shot/issues/3) with
`gh issue comment --attach`, then embed the generated URLs in the README
instead of checking image files into Git.

## Releases

Conventional Commits drive the Release Please PR. Merging it creates a draft
release. CI builds and attests each archive and attaches the signed Packslip
bundle. Keep immutable releases enabled and allow GitHub Actions to create
pull requests. GitHub may hold CI on automation-created PRs for approval.

GitHub's default workflow token cannot read the immutability setting. Without
extra credentials, CI leaves the signed release as a draft. A maintainer can
publish it after checking the setting:

```sh
repo=dkarter/tui-shot
tag=v0.1.0 # Replace with the draft's tag.
test "$(gh api "repos/$repo/immutable-releases" --jq .enabled)" = true &&
  gh release edit "$tag" --repo "$repo" --verify-tag --draft=false
gh release verify "$tag" --repo "$repo"
```

For automatic publishing, set the `IMMUTABILITY_READ_TOKEN` repository secret
to a fine-grained token scoped to this repository with **Administration: Read**
only. CI uses it only to check the setting; the default workflow token publishes
the draft.

To resume a failed draft, dispatch the `Release` workflow on the tag itself:

```sh
gh workflow run release.yml --ref "$tag" -f resume_tag="$tag"
```
