# SIAMBRUK Release Guide

## First repository release

1. Create the GitHub repository and add its remote:

   ```powershell
   git remote add origin https://github.com/<owner>/<repository>.git
   ```

2. The updater endpoint is configured for `Karuki00/SIAMBRUK` in `tauri-app/src-tauri/tauri.conf.json`; update it if the repository is renamed or transferred.
3. Keep the private updater key outside the repository. The local key is `%USERPROFILE%\\.tauri\\siambruk.key`; never commit it.
4. Add the same private key as the GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`.
5. Review the version in the package, Cargo, and Tauri configuration files, then create a matching tag:

   ```powershell
   git tag v0.1.0
   git push origin main --tags
   ```

The release workflow builds signed Windows, macOS, and Linux bundles and uploads `latest.json` for the Tauri updater. The application checks for updates at startup, asks for confirmation, installs the signed artifact, and relaunches.

## Local signed build

Set the private key only in the current shell, then build from `tauri-app/`:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\\.tauri\\siambruk.key"
npm.cmd run tauri build
```

Do not place the private key in `.env`, source control, a release asset, or issue comments. Tauri requires signed updater artifacts; losing this key prevents updates for existing installations.

## Release checks

```powershell
npm.cmd run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```
