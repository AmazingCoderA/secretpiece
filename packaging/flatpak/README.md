# Flatpak packaging

`dev.secretpiece.SecretPiece.yml` builds the desktop app (`sp`, `secretpiece`)
and the CLI (`sp-cli`) against `org.freedesktop.Platform//24.08` with the
`rust-stable` SDK extension.

## Build a bundle

The manifest takes a source archive so that `target/` and other local files never
end up in the sandbox:

```bash
git archive --prefix=src/ -o packaging/flatpak/secretpiece-src.tar.gz HEAD
flatpak-builder --user --force-clean --repo=flatpak-repo \
  --default-branch=stable build-flatpak packaging/flatpak/dev.secretpiece.SecretPiece.yml
flatpak build-bundle flatpak-repo secretpiece.flatpak dev.secretpiece.SecretPiece stable
```

Install and run:

```bash
flatpak install --user ./secretpiece.flatpak
flatpak run dev.secretpiece.SecretPiece
```

## Sandbox

The app needs real files, not portals, because a vault is a single file on a USB
drive or a network share. The manifest therefore grants:

- `--filesystem=host` and `--filesystem=/run/media` — open and create vaults
  anywhere the user can reach, including removable media;
- `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`, `--device=dri` —
  the egui/winit window and hardware rendering.

No network access is granted at runtime. The build itself uses
`--share=network` only so that `cargo` can fetch crates; Flathub submissions would
need vendored sources instead.

## Files

| File | Purpose |
|---|---|
| `dev.secretpiece.SecretPiece.yml` | flatpak-builder manifest |
| `dev.secretpiece.SecretPiece.desktop` | menu entry |
| `dev.secretpiece.SecretPiece.metainfo.xml` | AppStream metadata and screenshots |
| `../icons/dev.secretpiece.SecretPiece.svg` | icon source |
| `../icons/dev.secretpiece.SecretPiece.png` | 512×512 icon installed into the bundle |

The same desktop file, icon and metainfo are reused by the `.deb`/`.rpm`
packages (see `../nfpm.yaml`).
