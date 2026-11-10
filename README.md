<p align="center">
  <img src="docs/assets/hero.svg" alt="SecretPiece — encrypted vault in a single file" width="100%">
</p>

<p align="center">
  <a href="README.md"><img alt="Русский" src="https://img.shields.io/badge/RU-Русский-9b6bff?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="English" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
</p>

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/AmazingCoderA/secretpiece/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/actions/workflows/release.yml"><img alt="Build" src="https://github.com/AmazingCoderA/secretpiece/actions/workflows/release.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases"><img alt="Release" src="https://img.shields.io/github/v/release/AmazingCoderA/secretpiece?label=latest&color=6ea8ff"></a>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20BSD-4de8ff">
  <a href="#лицензия"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/license-GPL--3.0-blue"></a>
</p>

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Download latest release" src="https://img.shields.io/badge/⬇_Скачать_последнюю_версию-2ea043?style=for-the-badge&logo=github&logoColor=white"></a>
  <a href="#установка"><img alt="Install" src="https://img.shields.io/badge/Установка-по_платформам-9b6bff?style=for-the-badge"></a>
</p>

---

**SecretPiece** — кроссплатформенный инструмент на Rust для создания зашифрованного хранилища внутри обычного файла. Работает на флешке, внешнем HDD/SSD, внутреннем диске и в сетевой папке. Без разметки диска, без форматирования, без драйверов и без прав администратора. Открытый код, лицензия GPL-3.0.

> 🔐 **Your secret, in pieces.** Один файл. Внутри — зашифрованный виртуальный том, скрытый том с отдельным паролем и всё, что нужно, чтобы данные пережили потерю носителя.

<p align="center">
  <img src="https://img.shields.io/badge/status-v1.2%20·%20skeleton-9b6bff" alt="status">
  <img src="https://img.shields.io/badge/crypto-not%20implemented%20yet-critical" alt="crypto not implemented">
</p>

> ⚠️ **Ранняя стадия.** Криптография и формат ещё не реализованы. Не храните в текущих сборках ничего важного.

## ✨ Возможности

| | |
|---|---|
| 🧩 **Не ломает носитель** | Только обычные файлы. MBR/GPT, boot sector и файловая система не изменяются. |
| 💾 **Любой носитель** | USB-флешка, внешний HDD/SSD, внутренний диск, SMB/NFS. Основной сценарий — съёмные носители. |
| 🕵️ **Скрытый том** | Область внутри контейнера выглядит как свободное место. Открывается отдельным паролем. |
| 🔐 **Выбор шифрования** | XChaCha20-Poly1305 по умолчанию, ChaCha20-Poly1305 и AES-256-GCM опционально, параметры в заголовке. |
| 🗂️ **Разные форматы** | Один файл, набор частей для FAT32, папка-хранилище, nested-контейнер, k/n shards. |
| 🚀 **Любые платформы** | Windows x64/ARM64, macOS universal, Linux x64/ARM64, FreeBSD/OpenBSD/NetBSD, Arch Linux. |
| 🛟 **Восстанавливаемость** | Два заголовка, два суперблока, `doctor`, recovery BIP39, схема Шамира, снапшоты. |
| 🧹 **Безопасное удаление** | Shred файлов и свободного места, crypto-erase контейнера — с честным отчётом. |
| 🌍 **RU / EN** | Интерфейс и подсказки на русском и английском, автодетект языка, `--lang`. |

## 📦 Установка

<p align="center">
  <a href="#windows"><img alt="Windows" src="https://img.shields.io/badge/Windows-x64_/_ARM64-0078D4?style=for-the-badge&logo=windows11&logoColor=white"></a>
  <a href="#macos"><img alt="macOS" src="https://img.shields.io/badge/macOS-universal-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="#android"><img alt="Android" src="https://img.shields.io/badge/Android-APK-3DDC84?style=for-the-badge&logo=android&logoColor=white"></a>
  <a href="#linux"><img alt="Linux" src="https://img.shields.io/badge/Linux-x64_/_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black"></a>
  <a href="#arch-linux"><img alt="Arch Linux" src="https://img.shields.io/badge/Arch_Linux-pkg-1793D1?style=for-the-badge&logo=archlinux&logoColor=white"></a>
  <a href="#bsd"><img alt="BSD" src="https://img.shields.io/badge/BSD-FreeBSD_/_OpenBSD_/_NetBSD-AB2B28?style=for-the-badge&logo=freebsd&logoColor=white"></a>
</p>

Все сборки лежат в [Releases](https://github.com/AmazingCoderA/secretpiece/releases/latest) вместе с `SHA256SUMS.txt`. Каждый push в `main` собирает свежие артефакты автоматически.

### Windows

```powershell
# скачать secretpiece-<version>-windows-x64.zip и распаковать
.\sp.exe --version
.\sp.exe --help
```

Windows ARM64 — архив `windows-arm64`. Портативная сборка: ничего не пишет в реестр, конфиг — рядом с бинарником.

### macOS

```bash
# universal: Intel + Apple Silicon
tar -xzf secretpiece-<version>-macos-universal.tar.gz
xattr -dr com.apple.quarantine ./sp
./sp --version
```

### Android

```bash
# APK из Releases
adb install -r secretpiece-<version>-android.apk
```

Или включите «Установка из неизвестных источников» и откройте APK на телефоне
(Android 8+, ARM64). Сценарий: флешка через OTG/USB-C → «Выбрать файл .spv» →
пароль → работа с файлами. Без root и без разрешения `INTERNET`.
Подробнее — [`docs/ANDROID.md`](docs/ANDROID.md).

> Сейчас в APK готовы UI и доступ к носителю через SAF; криптоядро подключается
> в v1.3 через `sp-ffi`.

<p align="center">
  <img src="docs/assets/android.svg" alt="SecretPiece для Android: флешка через OTG, выбор .spv и пароль" width="100%">
</p>

### Linux

```bash
# статический musl, без зависимостей
tar -xzf secretpiece-<version>-linux-x64.tar.gz
./sp --version
```

Для ARM64 — `linux-arm64`. Работает на Ubuntu 22.04+, Fedora, Debian, Arch, Alpine.

### Arch Linux

```bash
# готовый pacman-пакет из Releases (x86_64)
sudo pacman -U secretpiece-1.2-1-x86_64.pkg.tar.zst

# AUR (после публикации рецепта)
yay -S secretpiece-bin    # готовый бинарник
yay -S secretpiece        # сборка из исходников
# или: paru -S secretpiece-bin

# вручную из исходников
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece/packaging/arch
makepkg -si
```

Рецепты: [`packaging/arch/PKGBUILD`](packaging/arch/PKGBUILD),
[`packaging/aur/secretpiece-bin`](packaging/aur/secretpiece-bin/PKGBUILD) и
[`packaging/aur/secretpiece`](packaging/aur/secretpiece/PKGBUILD). В релизах есть
`arch-x64` архив и настоящий `.pkg.tar.zst`.

### BSD

```sh
# FreeBSD
pkg install rust
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece && cargo build --release --locked -p sp-cli

# OpenBSD
pkg_add rust

# NetBSD
pkgin install rust
```

BSD-сборки в CI помечены как экспериментальные: релиз выходит, даже если VM-раннер BSD недоступен.

## 🔐 Шифрование

| Слой | Алгоритм | Статус |
|---|---|---|
| Данные и метаданные | **XChaCha20-Poly1305** (256-bit key, 192-bit nonce) | по умолчанию |
| Альтернатива | **ChaCha20-Poly1305** (RFC 8439, 96-bit nonce) | опционально |
| Альтернатива | **AES-256-GCM** (AES-NI / ARMv8 Crypto) | опционально |
| Альтернатива | **AES-256-GCM-SIV** (nonce-misuse resistant) | опционально, позже |
| Парольный KDF | **Argon2id**: light 256 МиБ · default 512 МиБ · paranoid 1 ГиБ | v1.2 |
| Хеши и MAC | **BLAKE3** (keyed mode) | v1.2 |
| Обмен ключами | **X25519** для слотов получателей | v1.5 |
| Recovery | **BIP39** (24 слова) + **Shamir** K из N | v1.5 |
| Слоты разблокировки | пароль · keyfile · FIDO2 hmac-secret · recovery · X25519 | v1.2–v1.5 |
| Подписи манифестов | **Ed25519** (только релизы, не тома) | v2.0 |

- `algorithm_id` и все параметры KDF записываются в заголовок; молча менять их запрещено.
- Смена алгоритма — только через `sp migrate` с созданием нового контейнера.
- Свои шифры, ECB, CBC без MAC, SHA-1/MD5 и «обфускация вместо шифрования» не используются.
- Слоты и payload заголовка неотличимы от случайных данных без ключа.

## 🗂️ Форматы контейнера

| Формат | Что создаётся | Ограничения |
|---|---|---|
| **SPV1 · single file** | один `*.spv` | до 4 ТиБ, лучший выбор для exFAT/NTFS/APFS/ext4 |
| **SPV1 · split** | `*.spv.000`, `.001`, … | FAT32, части по 3.5 ГиБ |
| **Vault folder (L2)** | папка блоков + индекс | удобно для облака и больших объёмов |
| **Nested (L4)** | контейнер внутри внешнего тома | нет overlap свободного места |
| **Sharded (L5)** | k/n Reed-Solomon шарды | каждая часть < 4 ГиБ, без FAT32-сплита |
| **Archive export** | зашифрованная копия для бэкапа | требует пароль/recovery |

## 🗺️ Архитектура

<p align="center">
  <img src="docs/assets/architecture.svg" alt="SecretPiece architecture: interfaces, vault, format, crypto, SPV1 layout" width="100%">
</p>

```
0x00000  header A       ← внешний том
0x10000  header A'      ← резерв
0x20000  header B       ← скрытый том
0x30000  header B'      ← резерв
0x40000  superblock A
0x50000  superblock B
0x60000  chunks…        ← данные и метаданные
N−hidden hidden area    ← перекрывается свободным местом (deniable)
```

## 🎛️ Режимы хранения

| Уровень | Что это | Для чего |
|---|---|---|
| **L1 · Sealed file** | один `*.spv` | бэкапы, облако, передача |
| **L2 · Vault folder** | папка с блоками и индексом | большие объёмы, синхронизация |
| **L3 · Hidden volume** | внешний + скрытый том в конце файла | основной режим для флешки |
| **L4 · Nested vault** | контейнер внутри внешнего тома | без overlap свободного места |
| **L5 · Sharded vault** | k/n Reed-Solomon части | disaster recovery, раздельное хранение |
| **L6 · Paranoid** | L3/L5 + снапшоты + 2FA + реплики | максимальная сохранность |

Профили сохранности: **Fast** · **Balanced** (по умолчанию) · **Paranoid** · **Custom**. Профиль меняет политику версий, корзины и проверок — но не криптографию и не формат.

## 🚀 Быстрый старт

```bash
# язык интерфейса: ru или en (автодетект по системной локали)
sp --lang ru init E:\vaults\personal.spv --size 8G --hidden 2G --profile balanced
sp shell E:\vaults\personal.spv

sp> put .\Documents docs/
sp> get docs/report.pdf .\report.pdf
sp> snapshot create before-trip
sp> shred docs/old.pdf --passes 3
sp> lock
```

GUI — после стабилизации формата (Tauri 2 + Svelte).

## 🌍 Языки

- Русский и английский — полностью, включая подсказки и сообщения об ошибках.
- Автодетект: `LC_ALL`, `LC_MESSAGES`, `LANG`, `SP_LANG`.
- Явно: `sp --lang ru …` или `sp --lang en …`.
- В GUI — переключатель в настройках и на экране входа.
- Новые языки — через файлы ресурсов, без правки ядра.

<p align="center">
  <img src="docs/assets/cli.svg" alt="SecretPiece CLI: русский и английский интерфейс" width="100%">
</p>

## 🧯 Если закончилось место

SecretPiece проверяет свободное место **до** записи и не оставляет частичных коммитов. Если место закончилось:

| Ситуация | Что делает приложение | Что делать пользователю |
|---|---|---|
| Свободно < 10% | предупреждение + прогресс | очистить корзину, удалить старые версии |
| Свободно 0 | отказ **до** записи, код `E_NOSPC` (9), том → read-only | освободить место или увеличить контейнер |
| Импорт больше свободного | preflight-отказ без мусора | импортировать частями или во внешний том |
| Кончилось место на носителе | `E_NOSPC`, read-only, без повреждения индекса | очистить корзину/снапшоты, экспортировать данные |
| FAT32 и файл > 4 ГиБ | автосплит на части 3.5 ГиБ | использовать exFAT/NTFS |
| Нужно больше объёма | `sp resize` (план v1.4) | создать новый контейнер и перенести данные |

Подсказки есть везде: в CLI-ошибках, в `sp df`, в GUI-баннере и в `sp doctor --explain`. Резерв 64 МиБ всегда держится под метаданные и суперблоки.

## 🛡️ Безопасность

- Приложение **не делает сетевых запросов** и не собирает телеметрию.
- Пароли и ключи не попадают в argv, логи и временные файлы.
- Скрытый том неотличим от случайных данных без пароля, **но факт наличия контейнерного файла скрыть нельзя**.
- Затирание на flash/SSD не гарантирует физическое уничтожение из-за wear leveling. Надёжный вариант — **crypto-erase**.
- Уязвимости: приватно через [GitHub Security Advisories](https://github.com/AmazingCoderA/secretpiece/security/advisories/new), см. [`SECURITY.md`](SECURITY.md).

## 🖥️ Платформы и архитектуры

| ОС | Готовые сборки | Статус |
|---|---|---|
| Windows 10/11 | x64, ARM64 | v1.2 |
| macOS 13+ | universal (Intel + Apple Silicon) | v1.2 |
| Android 8+ | APK (UI + SAF), ARM64 | v1.2 (каркас) |
| Linux | x64, ARM64 (static musl) | v1.2 |
| Arch Linux | x64, pacman-пакет, PKGBUILD, AUR (yay/paru) | v1.2 |
| FreeBSD | x64 (VM build) | экспериментально |
| OpenBSD | x64 (VM build) | экспериментально |
| NetBSD | x64 (VM build) | экспериментально |
| Linux ARMv7 / RISC-V | сборка из исходников | план v1.3 |
| Flatpak / AppImage | песочница и переносимый образ | план v1.4 |

## 🛠️ Сборка и релизы

```bash
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece
cargo build --workspace
cargo test --workspace
cargo run -p sp-cli --bin sp -- --version
```

Нужен Rust stable (MSRV **1.85**, edition 2024).

- **Auto-build** — на каждый push в `main` и на каждый PR: артефакты на странице запуска workflow.
- **Release** — на тег `v*`: архивы + `SHA256SUMS.txt` в GitHub Releases.
- **Ассеты релиза:** `windows-x64`, `windows-arm64`, `linux-x64`, `linux-arm64`, `macos-universal`, `arch-x64`, `*.pkg.tar.zst`, `android.apk` + `SHA256SUMS.txt`.
- Версии — `1.1`, `1.2`, `1.3`, `2.0` (без микро-патчей в тегах).

## 📚 Документация

| Документ | О чём |
|---|---|
| [SECURITY.md](SECURITY.md) | раскрытие уязвимостей, scope, гарантии |
| [CONTRIBUTING.md](CONTRIBUTING.md) | правила разработки и ревью |
| [ADR](docs/ADR/README.md) | архитектурные решения |
| [CHANGELOG.md](CHANGELOG.md) | история изменений |

Байтовая спецификация (`docs/FORMAT.md`) и крипто-дизайн (`docs/CRYPTO.md`) появятся вместе с v1.3.

## 🧭 Дорожная карта

| Версия | Статус | Результат |
|---|---|---|
| **v1.1** | ✅ | каркас, CI, автосборка, GPL-3.0, любые носители |
| **v1.2** | ✅ | визуал, RU/EN CLI, pacman/AUR, Android-каркас, Windows ARM64, Linux ARM64, BSD |
| **v1.3** | ⏳ | крипто-ядро и формат SPV1: Argon2id, XChaCha20-Poly1305, слоты, чанки |
| **v1.4** | ⏳ | CLI и ФС: `init/open/ls/put/get/rm`, extents, индекс, FAT32 split, ARMv7/RISC-V |
| **v1.5** | ⏳ | надёжность: COW, снапшоты, корзина, `doctor`, resize, crash-тесты |
| **v1.6** | ⏳ | скрытый том, recovery, Shamir, слоты получателей X25519 |
| **v1.7** | ⏳ | затирание, free-space wipe, destroy, Reed-Solomon sharding |
| **v1.8** | ⏳ | GUI: Tauri 2 + Svelte, мастер создания, менеджер файлов |
| **v2.0** | ⏳ | hardening, фаззинг, Miri, внешний аудит, подписи, SBOM |

## 🤝 Участие

Проект открыт для ревью, фаззинга и аудита. Перед PR прочитайте [`CONTRIBUTING.md`](CONTRIBUTING.md). Криптографические и форматные изменения — только через ADR.

## Лицензия

**GNU General Public License v3.0 only** (GPL-3.0-only). Полный текст — [`LICENSE`](LICENSE).

Инструмент двойного назначения. Используйте только для защиты собственных данных и в рамках закона. Авторы не несут ответственности за потерю данных, неправомерное использование или ограничения криптоэкспорта в вашей юрисдикции.

## Скачать

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Windows" src="https://img.shields.io/badge/Windows-x64_/_ARM64-0078D4?style=for-the-badge&logo=windows11&logoColor=white"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Linux" src="https://img.shields.io/badge/Linux-x64_/_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="macOS" src="https://img.shields.io/badge/macOS-universal-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="BSD" src="https://img.shields.io/badge/BSD-FreeBSD_/_OpenBSD_/_NetBSD-AB2B28?style=for-the-badge&logo=freebsd&logoColor=white"></a>
</p>

<p align="center"><sub>encrypted USB vault · hidden volume · VeraCrypt alternative · Rust · GPL-3.0 · open source</sub></p>

---

<p align="center">
  <a href="README.md"><img alt="Русский" src="https://img.shields.io/badge/RU-Русский-9b6bff?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="English" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
</p>

<p align="center">
  <sub>Сделано с ❤️ и паранойей · <b>SecretPiece</b> · Your secret, in pieces.</sub>
</p>
