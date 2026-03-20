# MindSafe

> Your notes deserve absolute privacy.

MindSafe is a secure, offline-first notebook/editor built with [Rust](https://www.rust-lang.org/) and [egui-eframe](https://www.egui.rs/), designed so that **ONLY** you can access and read your data. Whether you’re writing ideas, keeping a personal journal, or storing sensitive information, this app ensures your words remain yours — **private, secure, and under your control.**

## Features

### Security

1. 🗝️ **Password-protected key** — A key is generated on your device (*can be regenerated if needed*) and wrapped with your password using [Argon2id](https://en.wikipedia.org/wiki/Argon2), a state-of-the-art password hardening algorithm. Without your password, the data is useless to anyone else.
2. 🔐 **End-to-end encryption** — Every note is encrypted individually with modern cryptography ([XChaCha20-Poly1305](https://en.wikipedia.org/wiki/ChaCha20-Poly1305)). Even if someone copies your data, they only see meaningless ciphertext.
3. 🛡️ **Resistant to attacks** — Strong key derivation, random nonces, per-note keys, and automatic zeroization of secrets mean your data is safeguarded against brute force, device theft-hacking-compromise and forensic analysis.
4. activities <a name="activities"></a>
5. safe copy

> Unless you display your password, or they guess it, or your device's OS is infected with a malware causing memory leak while when you unlock the app, your data is safe and protected.

### Performance
4. ⚡ **Fast and portable** — Built in [Rust](https://www.rust-lang.org/) for performance and safety, packaged as a lightweight desktop app. Works across Windows, macOS, and Linux.
5. **Powered by SQLite** - Uses [SQLite](https://sqlite.org/) as db, which is highly stable, [ACID compliant](https://en.wikipedia.org/wiki/ACID) and best in performance.

### Privacy
6. 💾 **Local Storage** — All notes and configuration stay on your device. There is no server, no cloud sync, and no hidden copies.
7. **No Data Collection** - MindSafe does not collect data in any shape or form from its users. No telemetry, no data theft, and no analytics (*[activities](#activities) recorded in app, are for the security, the data remains on device in encypted format and is never sent anywhere, this can be turned off*). No data is sent/collected.
8. **Completely Offline** - It does not make any connections to the internet to avoid any form of data loss, it strictly an offline app.


tags
auto save 
history
export

## FOSS Complaint

MindSafe is [Free and Open Source Software (FOSS)](https://en.wikipedia.org/wiki/Free_and_open-source_software) compliant, entire code base is released under [GNU Affero General Public License version 3 or later](/LICENSE.md).