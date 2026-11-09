<p align="center">
  <img src="docs/assets/banner.svg" alt="SecretPiece — portable deniable encrypted vault" width="100%">
</p>

<p align="center">
  <a href="README.md"><img alt="Русский" src="https://img.shields.io/badge/RU-Русский-9b6bff?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="English" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
</p>

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/AmazingCoderA/secretpiece/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/actions/workflows/release.yml"><img alt="Build" src="https://github.com/AmazingCoderA/secretpiece/actions/workflows/release.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases"><img alt="Release" src="https://img.shields.io/github/v/release/AmazingCoderA/secretpiece?include_prereleases&label=release&color=6ea8ff"></a>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-4de8ff">
  <a href="#лицензия"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/license-GPL--3.0-blue"></a>
</p>

---

**SecretPiece** — кроссплатформенный инструмент на Rust для создания зашифрованного хранилища внутри обычного файла. Без разметки диска, без форматирования, без драйверов и без прав администратора.

> 🔐 **Your secret, in pieces.** Один файл. Внутри — зашифрованный виртуальный том, скрытый том с отдельным паролем и всё, что нужно, чтобы данные пережили потерю носителя.

<p align="center">
  <img src="https://img.shields.io/badge/status-M0%20·%20spec%20%2B%20skeleton-9b6bff" alt="status">
  <img src="https://img.shields.io/badge/crypto-not%20implemented%20yet-critical" alt="crypto not implemented">
</p>

> ⚠️ **Ранняя стадия.** Криптография и формат ещё не реализованы. Не храните в текущих сборках ничего важного.

## ✨ Почему SecretPiece

| | |
|---|---|
| 🧩 **Не ломает носитель** | Работает только с обычными файлами. MBR/GPT, boot sector и файловая система не изменяются. |
| 💾 **Любой носитель** | Флешка, внешний HDD/SSD, внутренний диск, сетевая папка. Основной сценарий — съёмные USB-носители. |
| 🕵️ **Скрытый том, а не раздел** | Область внутри контейнера выглядит как свободное место. Открывается отдельным паролем. |
| 💻 **Один код — три ОС** | Windows 10/11, macOS 13+, Linux. Общее Rust-ядро, CLI и GUI без драйверов. |
| 🔒 **Честная безопасность** | Открытый код, стандартные примитивы, никакой телеметрии и сетевых вызовов. |
| 🛟 **Восстанавливаемость** | Два заголовка, два суперблока, `doctor`, recovery-код BIP39, схема Шамира. |
| 🧹 **Безопасное удаление** | Shred файлов и свободного места, crypto-erase контейнера — с честным отчётом. |

## 🔐 Шифрование

| Слой | Алгоритм | Статус |
|---|---|---|
| Данные и метаданные | **XChaCha20-Poly1305** (256-bit key, 192-bit nonce) | по умолчанию |
| Альтернативный AEAD | **AES-256-GCM** (через `algorithm_id`) | опционально, позже |
| Парольный KDF | **Argon2id**: light 256 МиБ · default 512 МиБ · paranoid 1 ГиБ | v1 |
| Хеши и MAC | **BLAKE3** (keyed mode) | v1 |
| Обмен ключами | **X25519** для слотов получателей | v1.1 |
| Recovery | **BIP39** (24 слова) + **Shamir** K из N | v1 |
| Слоты разблокировки | пароль · keyfile · FIDO2 hmac-secret · recovery · X25519 | v1 / v1.1 |

- Алгоритм и его параметры фиксируются в заголовке контейнера; менять их молча запрещено.
- `compact`-профиль допускает сжатие и переменные чанки; `balanced`/`paranoid` — фиксированные 64 КиБ.
- Свои шифры, ECB, CBC без MAC, SHA-1/MD5 и «обфускация вместо шифрования» не используются.

## 🗺️ Как это работает

```mermaid
flowchart LR
    subgraph DISK["Носитель: USB · HDD · SSD · сеть"]
        A["обычные файлы<br/>фото, документы"]
        B["personal.spv<br/>обычный файл"]
    end
    B --> C["SPV1 container"]
    C --> D["Внешний том<br/>пароль 1"]
    C --> E["Скрытый том<br/>пароль 2"]
    D --> F["read-only<br/>в deniable-режиме"]
    E --> G["настоящие данные"]
    style B fill:#1b2540,stroke:#6ea8ff,color:#dbe6ff
    style C fill:#101a30,stroke:#4de8ff,color:#dbe6ff
    style D fill:#17213a,stroke:#6ea8ff,color:#dbe6ff
    style E fill:#1d1838,stroke:#9b6bff,color:#e6dcff
    style G fill:#231a3f,stroke:#9b6bff,color:#e6dcff
```

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

## 🚀 Быстрый старт (план CLI)

```bash
# создать 8 ГиБ контейнер со скрытым томом 2 ГиБ
sp init E:\vaults\personal.spv --size 8G --hidden 2G --profile balanced

# открыть интерактивную оболочку
sp shell E:\vaults\personal.spv

sp> put .\Documents docs/
sp> get docs/report.pdf .\report.pdf
sp> snapshot create before-trip
sp> shred docs/old.pdf --passes 3
sp> lock
```

GUI — после стабилизации формата (Tauri 2 + Svelte).

## 🛠️ Сборка и релизы

```bash
git clone https://github.com/AmazingCoderA/secretpiece.git
cd secretpiece
cargo build --workspace
cargo test --workspace
cargo run -p sp-cli --bin sp -- --version
```

Нужен Rust stable (MSRV **1.85**, edition 2024).

**Автосборка** запускается на каждый push в `main` и на каждый PR. Артефакты лежат на странице запуска workflow:

- `windows-x64` — `.zip`;
- `linux-x64` — `.tar.gz`, статический musl;
- `macos-universal` — `.tar.gz` для Intel и Apple Silicon.

Пуш тега `v*` дополнительно создаёт [GitHub Release](https://github.com/AmazingCoderA/secretpiece/releases) с архивами и `SHA256SUMS.txt`.

## 🛡️ Безопасность

- Приложение **не делает сетевых запросов** и не собирает телеметрию.
- Пароли и ключи не попадают в argv, логи и временные файлы.
- Скрытый том неотличим от случайных данных без пароля, **но факт наличия контейнерного файла скрыть нельзя**.
- Затирание на flash/SSD не гарантирует физическое уничтожение из-за wear leveling. Надёжный вариант — **crypto-erase**.
- Уязвимости: приватно через [GitHub Security Advisories](https://github.com/AmazingCoderA/secretpiece/security/advisories/new), см. [`SECURITY.md`](SECURITY.md).

## 📚 Документация

| Документ | О чём |
|---|---|
| [SECURITY.md](SECURITY.md) | раскрытие уязвимостей, scope, гарантии |
| [CONTRIBUTING.md](CONTRIBUTING.md) | правила разработки и ревью |
| [ADR](docs/ADR/README.md) | архитектурные решения |
| [CHANGELOG.md](CHANGELOG.md) | история изменений |

Байтовая спецификация (`docs/FORMAT.md`) и крипто-дизайн (`docs/CRYPTO.md`) появятся вместе с M1.

## 🧭 Дорожная карта

| Этап | Статус | Результат |
|---|---|---|
| **M0** — каркас | ✅ готово | workspace, CI, лицензия GPL-3.0, автосборка релизов |
| **M1** — крипто-ядро и формат | ⏳ | `sp-crypto`, `sp-format`, заголовки, слоты, чанки, тест-векторы |
| **M1.1** — выбор шифрования | ⏳ | AES-256-GCM backend, `sp migrate` |
| **M2** — CLI и ФС | ⏳ | `init/open/ls/put/get/rm`, extents, индекс, FAT32 split |
| **M3** — надёжность | ⏳ | COW-метаданные, снапшоты, корзина, `doctor`, crash-тесты |
| **M4** — скрытый том и recovery | ⏳ | L3/L4, recovery, Shamir, слоты получателей |
| **M5** — затирание и sharding | ⏳ | shred, wipe-free, destroy, Reed-Solomon |
| **M6** — GUI | ⏳ | Tauri 2 + Svelte |
| **M7** — hardening + audit | ⏳ | фаззинг, Miri, внешний аудит, подписи |
| **M8** — v1.0 | ⏳ | релизы, сайт, документация |

## 🤝 Участие

Проект открыт для ревью, фаззинга и аудита. Перед PR прочитайте [`CONTRIBUTING.md`](CONTRIBUTING.md). Криптографические и форматные изменения — только через ADR.

## Лицензия

**GNU General Public License v3.0 only** (GPL-3.0-only). Полный текст — [`LICENSE`](LICENSE).

Инструмент двойного назначения. Используйте только для защиты собственных данных и в рамках закона. Авторы не несут ответственности за потерю данных, неправомерное использование или ограничения криптоэкспорта в вашей юрисдикции.

## Скачать

<p align="center">
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Windows" src="https://img.shields.io/badge/Windows-x64-0078D4?style=for-the-badge&logo=windows11&logoColor=white"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="Linux" src="https://img.shields.io/badge/Linux-x64-FCC624?style=for-the-badge&logo=linux&logoColor=black"></a>
  <a href="https://github.com/AmazingCoderA/secretpiece/releases/latest"><img alt="macOS" src="https://img.shields.io/badge/macOS-universal-000000?style=for-the-badge&logo=apple&logoColor=white"></a>
</p>

Все сборки — в [Releases](https://github.com/AmazingCoderA/secretpiece/releases) вместе с `SHA256SUMS.txt`. Каждый push в `main` собирает свежие артефакты автоматически.

---

<p align="center">
  <a href="README.md"><img alt="Русский" src="https://img.shields.io/badge/RU-Русский-9b6bff?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="English" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
</p>

<p align="center">
  <sub>Сделано с ❤️ и паранойей · <b>SecretPiece</b> · Your secret, in pieces.</sub>
</p>
