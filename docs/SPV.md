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
  BSD и Android (на телефоне — только чтение).
- **Один пароль.** Ключ выводится из пароля и живёт только в памяти; в файле
  нет ни пароля, ни его хеша в открытом виде.

## Как устроен контейнер SPV1

```
0x0000  header slot 0   4096 байт
0x1000  header slot 1   4096 байт
0x2000  данные          зашифрованный индекс + чанки файлов по 64 КиБ
```

- **Два слота заголовка.** Каждый коммит пишется в слот, который сейчас не
  используется, и увеличивает номер поколения (`generation`). Если питание
  пропадёт посреди записи, предыдущее поколение останется целым — контейнер
  откроется.
- **Заголовок** содержит магию `SPV1`, версию формата, поколение, параметры
  Argon2id (соль, память, итерации, параллелизм), nonce и обёрнутый
  мастер-ключ, а также смещение, длину и nonce зашифрованного индекса.
  Контрольная сумма **BLAKE3** покрывает весь слот целиком.
- **Мастер-ключ** случайный (32 байта). На диск он попадает только
  завёрнутым: `XChaCha20-Poly1305` под ключом, выведенным из пароля через
  **Argon2id**. Смена пароля переворачивает только обёртку, не данные.
- **Индекс** — JSON-дерево каталогов и файлов, зашифрованное мастер-ключом.
  При каждом изменении пишется новая зашифрованная версия, а заголовок
  переключается на неё.
- **Чанки** — файлы нарезаны по **64 КиБ** и зашифрованы
  `XChaCha20-Poly1305`. В associated data входят путь и номер чанка, поэтому
  переставить или подменить куски между файлами нельзя: проверка не пройдёт.

Ключи выводятся из пароля через **Argon2id**, целостность проверяется
**BLAKE3**. Параметры KDF хранятся в заголовке, а не зашиты в бинарник — их
можно усиливать без смены формата. Профили: `light` 16 МиБ, `default` 64 МиБ,
`paranoid` 256 МиБ.

> Скрытый том, recovery-фраза и несколько key slots (keyfile, FIDO2, X25519) —
> это следующие версии. В v1.4 у контейнера один пароль и один том; скрытая
> область пока не создаётся.

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
# CLI: создать контейнер (пароль спросят дважды)
sp-cli init personal.spv

# добавить файл, посмотреть список, извлечь
sp-cli put personal.spv .\Documents\report.pdf docs/report.pdf
sp-cli ls personal.spv
sp-cli get personal.spv docs/report.pdf .\report.pdf

# состояние и проверка целостности
sp-cli df personal.spv
sp-cli verify personal.spv
```

```powershell
# GUI: отдельное приложение
.\sp.exe            # или secretpiece.exe
```

На Android: подключите флешку по OTG/USB-C → «Выбрать файл .spv» → пароль.
Файл читается напрямую через Storage Access Framework и JNI, без root и без
копирования во внутреннюю память. Подробности — в [`ANDROID.md`](ANDROID.md).

## Чего `.spv` не делает

- Не шифрует весь диск целиком — это файл-контейнер, а не full-disk encryption.
- Не скрывает сам факт наличия файла. Если нужно спрятать и его, переименуйте
  файл; скрытый том появится в v1.7.
- Не является заменой резервным копиям: потерянный пароль восстановить нельзя.

## Статус

Формат SPV1 и криптоядро реализованы в **v1.4** (`crates/sp-crypto`,
`crates/sp-format`, `crates/sp-vault`) и покрыты тестами. Внешний аудит ещё
не проводился — держите резервную копию контейнера.

---

# What is an `.spv` file

`.spv` is an **ordinary file** that holds an encrypted SecretPiece vault. It is
not a disk partition, not a driver, not a virtual volume that needs admin rights
to mount. Just a file you can copy to a USB stick, a cloud folder, or an email.

> In short: `.spv` is a SecretPiece (SPV1) container. The extension is
> **cosmetic** — the format is detected from the header inside the file.

## Layout

```
0x0000  header slot 0   4096 bytes
0x1000  header slot 1   4096 bytes
0x2000  data area       encrypted index + 64 KiB file chunks
```

- **Two header slots.** Every commit is written to the slot that is not in use
  and bumps the generation number. If power is lost mid-write, the previous
  generation stays intact and the container still opens.
- **The header** carries the `SPV1` magic, format version, generation, Argon2id
  parameters (salt, memory, iterations, parallelism), the wrapped master key,
  and the offset, length and nonce of the encrypted index. A **BLAKE3**
  checksum covers the whole slot.
- **The master key** is random (32 bytes) and is stored only wrapped:
  `XChaCha20-Poly1305` under a key derived from the password with **Argon2id**.
  Changing the password rewraps the key and leaves the data untouched.
- **The index** is a JSON directory tree encrypted with the master key. Each
  change writes a new encrypted snapshot and points the header at it.
- **Chunks** are files split into **64 KiB** pieces and encrypted with
  `XChaCha20-Poly1305`. The path and chunk number go into the associated data,
  so chunks cannot be swapped between files — authentication fails.

Keys are derived with **Argon2id**, integrity uses **BLAKE3**. KDF parameters
live in the header, so they can be raised without a format change. Profiles:
`light` 16 MiB, `default` 64 MiB, `paranoid` 256 MiB.

> The hidden volume, BIP39 recovery, and multiple key slots (keyfile, FIDO2,
> X25519) are planned for later versions. v1.4 has one password and one volume;
> no hidden area is created yet.

Because format detection uses the internal magic, `personal.spv`,
`personal.jpg`, and `backup.dat` with the same bytes are the same vault. Rename
the file to something boring — that is file-level masking, and it does not
change the contents or the password.

```bash
sp-cli init personal.spv
sp-cli put personal.spv ./report.pdf docs/report.pdf
sp-cli ls personal.spv
sp-cli get personal.spv docs/report.pdf ./report.pdf
sp-cli df personal.spv
sp-cli verify personal.spv
```

On Android, plug the drive in over OTG/USB-C, tap **Choose .spv file**, and
enter the password. The container is read straight from the drive through the
Storage Access Framework and JNI — no root, no copy into app storage.

Status: the SPV1 format and the crypto core are implemented in **v1.4**
(`crates/sp-crypto`, `crates/sp-format`, `crates/sp-vault`) and covered by
tests. No external audit has been performed yet — keep a backup.
