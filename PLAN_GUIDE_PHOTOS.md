# PLAN_GUIDE_PHOTOS.md — Workstream G21: Photos Authorization Guides

## Status

G21 portable and iOS guides are implemented and reviewed against the D22/B27 APIs and validation limits. They state the exact host key and iOS 14.0 floor, preserve Limited, and make no live permission or asset-access claim

## Objective

Document the D22 portable read/write status contract and B27 iOS PhotoKit API boundary without implying asset access or host-app configuration

## Write scope

- `PLAN_GUIDE_PHOTOS.md`
- `docs/capabilities/photos.md`
- `docs/ios/photos.md`

Do not edit docs indexes, aggregate plans, canonical capability metadata, CI, or other capability guides

## Required guide facts

- Distinguish `Limited` from `Authorized` in all examples and status tables
- State that the iOS backend uses only the read/write access level and requires host `NSPhotoLibraryUsageDescription`
- State the iOS 14 API floor and link Apple's public PhotoKit and Info.plist references
- Keep status results separate from guarantees about later access or operation outcome
- Exclude asset enumeration, image requests, editing, picker UI, and all runtime or permission outcome claims
- State that target compilation and link/import checks do not demonstrate live app behavior or host plist configuration

## Validation

- Review both guides against D22 and B27 scope
- Run `git diff --check`
- Do not add the guides to shared documentation indexes; root owns index integration
