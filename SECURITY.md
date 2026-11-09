# Security Policy

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

- Preferred: [GitHub Security Advisories](https://github.com/AmazingCoderA/secretpiece/security/advisories/new) (private).
- Alternative: open a minimal issue asking for a private contact channel, without details.

Include:

- affected version / commit;
- platform and filesystem;
- reproduction steps or a proof of concept;
- impact assessment (confidentiality, integrity, availability);
- whether the issue affects the container format, key handling, or only the UI.

We aim to acknowledge within **72 hours** and to provide an initial assessment
within **7 days**. Coordinated disclosure is **90 days** by default, or sooner
by agreement.

## Scope

In scope:

- cryptography and key hierarchy (`sp-crypto`);
- container format and parsing (`sp-format`);
- chunk I/O, locking, fsync (`sp-storage`);
- namespace, versions, snapshots (`sp-fs`);
- high-level vault operations and policies (`sp-vault`);
- wipe/destroy logic (`sp-wipe`);
- CLI and GUI handling of secrets.

Out of scope (see the README security section):

- compromised OS, keyloggers, memory dumps, cold boot;
- malicious USB controllers or firmware;
- physical destruction guarantees on flash/SSD (wear leveling);
- coercion to reveal a password;
- hiding the fact that a container file exists.

## No backdoors

SecretPiece will never include backdoors, universal passwords, or hidden keys.
Recovery is possible only through mechanisms the user explicitly configured
(recovery key, Shamir shares, recipients).

## PGP

A project PGP key will be published here before the first stable release.
