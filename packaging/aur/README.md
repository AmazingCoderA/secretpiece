# AUR packaging

Two recipes are ready to publish on the [Arch User Repository](https://aur.archlinux.org):

| Package | Type | Source |
|---|---|---|
| `secretpiece-bin` | prebuilt binary | GitHub release `linux-x64` / `linux-arm64` |
| `secretpiece` | build from source | release tag tarball |

They are **not published yet** — publishing requires an AUR account with an SSH
key registered at <https://aur.archlinux.org> (only the package maintainer can
push). Until then, install the pacman package attached to a GitHub release:

```bash
sudo pacman -U secretpiece-1.4-1-x86_64.pkg.tar.zst
```

## Publish

```bash
# once per machine
ssh-keygen -t ed25519 -C "aur"
# add ~/.ssh/id_ed25519.pub at https://aur.archlinux.org/account

git clone ssh://aur@aur.archlinux.org/secretpiece-bin.git
cp packaging/aur/secretpiece-bin/PKGBUILD packaging/aur/secretpiece-bin/.SRCINFO secretpiece-bin/
cd secretpiece-bin
git add PKGBUILD .SRCINFO
git commit -m "1.4"
git push

# same for secretpiece
git clone ssh://aur@aur.archlinux.org/secretpiece.git
cp ../packaging/aur/secretpiece/PKGBUILD ../packaging/aur/secretpiece/.SRCINFO secretpiece/
cd secretpiece
git add PKGBUILD .SRCINFO
git commit -m "1.4"
git push
```

Users then install with:

```bash
yay -S secretpiece-bin
# or
paru -S secretpiece-bin
```

## Checksums

`secretpiece-bin` uses `sha256sums=('SKIP')` because the checksum depends on the
release asset. Replace it with the value from `SHA256SUMS.txt` in the release
before publishing. For `secretpiece`, generate the tarball hash with
`updpkgsums` after a tag exists (`makepkg --printsrcinfo > .SRCINFO`).
