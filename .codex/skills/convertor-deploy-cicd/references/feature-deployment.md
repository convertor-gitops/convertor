# Feature Deployment

## Common Preparation

1. Confirm the current branch is `feature/*` or `codex/*`. If it matches neither, remind the user before proceeding. Confirm the requested target is `debug` or `main` (`prod`) and record the starting branch for restoration.
2. Fetch the remote target branch and tags. Inspect the target history, feature commits, and worktree status. If the target already contains the feature tip and there are no pending feature changes, end with “无新增内容可部署”; do not create an empty merge or version bump.
3. Review pending changes and run checks appropriate to the feature. Commit only changes belonging to this feature. If unrelated changes cannot be safely separated, stop and explain the issue. Recheck whether the target contains the resulting feature tip; if so, end with “无新增内容可部署”. Push the feature branch.
4. Ensure the target branch matches its remote before merging. Switch to the target and merge the feature with `git merge --no-ff`. Resolve conflicts based on the intended feature behavior, then verify that the new HEAD is a merge commit whose parents are the former target tip and feature tip. Do not force an empty merge.

## Deploy to Debug

1. Complete common preparation and merge into `debug`.
2. Push `debug`, then switch back to the starting branch. This completes the requested deployment operation. Do not wait for GitHub Actions.

`build-debug.yml` starts automatically on a push to `debug` and delegates to `_build-shared.yml` with the `debug` profile. CI completion and runtime verification are separate tasks when requested.

## Deploy to Main / Prod

1. Run the common branch, fetch, status, and no-new-content checks first. When there is new feature content, inspect the latest `v*` tag reachable in `main` history. Compare its version with the feature's `metadata.json` and check that synchronized version files agree.
2. If the feature version has not advanced, choose the appropriate next version from the feature's changes, update `metadata.json`, and run `just version-sync`. If it has already advanced, confirm that it is newer than the latest `main` tag and matches the intended release. Complete common preparation step 3: review and commit all intended feature and synchronized version changes, validate, and push the feature branch. Check that the intended tag does not already exist before merging.
3. Complete common preparation step 4: merge into `main` with `--no-ff` and verify the new merge commit. Push `main`.
4. On the new `main` merge commit, create the matching `vX.Y.Z` tag and push that tag. Confirm it identifies the merge commit, then switch back to the starting branch. The tag push starts `build.yml` with the `release` profile; pushing `main` alone does not start that workflow.

Never move or replace an existing release tag as part of a normal deployment. If the intended tag already exists, stop and investigate before merging into `main`.
