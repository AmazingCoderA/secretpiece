# Что такое файл `.spv`

`.spv` — это **обычный файл**, внутри которого лежит зашифрованный сейф
SecretPiece. Не раздел диска, не драйвер, не виртуальный диск, который надо
монтировать правами администратора. Один файл: его можно скопировать на
флешку, в облако, отправить по почте, положить рядом с фотографиями.

> Коротко: `.spv` = контейнер SecretPiece (SPV1). Расширение `.spv` —
> **косметика**. Формат определяется заголовком внутри файла, а не именем.

## Зачем это нужно

- **Любой носитель.** USB-флешка, внешний HDD/SSD, внутренний диск, сетевая
  папка, облачная синхронизация — везде, где можно хранить файл.
- **Без форматирования.** Не нужно переразбивать диск, ставить драйверы,
  получать права администратора или «монтировать» что-либо.
- **Переносимость.** Один и тот же файл открывается на Windows, Linux, macOS,
  BSD и Android.
- **Правдоподобное отрицание.** В файле может быть внешний том и скрытый том;
  без пароля нельзя доказать, что скрытая часть вообще существует.

## Как устроен контейнер SPV1

```
0x00000  header A       ← внешний том (основной)
0x10000  header A'      ← резервная копия заголовка A
0x20000  header B       ← скрытый том
0x30000  header B'      ← резервная копия заголовка B
0x40000  superblock A   ← метаданные внешнего тома
0x50000  superblock B   ← метаданные скрытого тома
0x60000  chunks…        ← зашифрованные данные и индекс
N−hidden hidden area    ← скрытая зона в конце файла
```

- **Заголовки A/A′ и B/B′** — фиксированные области в начале файла. Каждый
  заголовок содержит key slots: 512-байтовые записи с обёрнутым мастер-ключом.
- **Key slots** позволяют иметь несколько способов открытия: пароль, keyfile,
  recovery-фраза, позже — FIDO2. Слоты выглядят как случайные байты.
- **Superblock** — зашифрованный снимок метаданных (дерево каталогов, версии).
- **Chunks** — данные, нарезанные на блоки и зашифрованные
  XChaCha20-Poly1305 (по умолчанию) или ChaCha20-Poly1305 / AES-256-GCM /
  AES-256-GCM-SIV.
- **Hidden area** — скрытый том, размещённый в конце файла. Свободное место
  внешнего тома перекрывается со скрытым, поэтому «хвост» файла не выглядит
  подозрительно.

Ключи выводятся из пароля через **Argon2id**, целостность проверяется
**BLAKE3**. Параметры KDF хранятся в заголовке, а не зашиты в бинарник — их
можно усиливать без смены формата.

## Почему расширение не важно

Внутри файла есть магия SPV1 и версия формата. Приложение читает их, а не имя
файла. Поэтому `personal.spv`, `personal.jpg`, `backup.dat` и `invoice.pdf` с
одинаковым содержимым — один и тот же сейф.

Это и есть **маскировка файла**: переименуйте контейнер в безобидное имя,
и он не будет выделяться среди обычных файлов на флешке. Содержимое при этом
не меняется, и пароль остаётся прежним.

> Не путайте с маскировкой приложения на Android — это отдельная опция,
> см. [`ANDROID.md`](ANDROID.md#маскировка-приложения).

## Как создать и открыть

```bash
# создать контейнер 8 ГиБ со скрытым томом 2 ГиБ
sp init E:\vaults\personal.spv --size 8G --hidden 2G

# открыть интерактивную оболочку
sp shell E:\vaults\personal.spv

# проверить состояние и свободное место
sp doctor E:\vaults\personal.spv --explain
sp df E:\vaults\personal.spv
```

На Android: подключите флешку по OTG/USB-C → «Выбрать файл .spv» → пароль.
Файл открывается через Storage Access Framework, без root.

## Чего `.spv` не делает

- Не шифрует весь диск целиком — это файл-контейнер, а не full-disk encryption.
- Не скрывает сам факт наличия файла. Если нужно спрятать и его, используйте
  маскировку имени/расширения и скрытый том.
- Не является заменой резервным копиям: потерянный пароль восстановить нельзя,
  кроме заранее настроенной recovery-фразы BIP39.

## Статус

Формат SPV1 описан и зафиксирован в коде-каркасе (`crates/sp-format`), но
криптография подключается в **v1.4**. До этого сборки помечены как каркас:
не храните в них реальные секреты.

---

# What is an `.spv` file

`.spv` is an **ordinary file** that holds an encrypted SecretPiece vault. It is
not a disk partition, not a driver, not a virtual volume that needs admin rights
to mount. Just a file you can copy to a USB stick, a cloud folder, or an email.

> In short: `.spv` is a SecretPiece (SPV1) container. The extension is
> **cosmetic** — the format is detected from the header inside the file.

## Layout

```
0x00000  header A       outer volume
0x10000  header A'      backup of A
0x20000  header B       hidden volume
0x30000  header B'      backup of B
0x40000  superblock A   outer volume metadata
0x50000  superblock B   hidden volume metadata
0x60000  chunks…        encrypted data and index
N−hidden hidden area    hidden tail (deniable)
```

- **Key slots** are 512-byte records with a wrapped master key: password,
  keyfile, BIP39 recovery, later FIDO2. Slots are indistinguishable from random
  bytes without the key.
- **Superblocks** are encrypted metadata snapshots (directory tree, versions).
- **Chunks** hold data encrypted with XChaCha20-Poly1305 (default),
  ChaCha20-Poly1305, AES-256-GCM, or AES-256-GCM-SIV.
- **Hidden area** overlaps the outer volume's free space, so the tail of the
  file gives nothing away.
- Keys are derived with **Argon2id**, integrity uses **BLAKE3**. KDF parameters
  live in the header, so they can be raised without a format change.

Because format detection uses the internal magic, `personal.spv`,
`personal.jpg`, and `backup.dat` with the same bytes are the same vault. Rename
the file to something boring — that is file-level masking, and it does not
change the contents or the password.

Status: the SPV1 layout is fixed in `crates/sp-format`, crypto is wired in
**v1.4**. Builds before that are skeletons — do not store real secrets.
