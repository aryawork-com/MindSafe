# TODO List

## Languages
- [x] English (Original)
- [ ] Hindi (AI Generated)
- [ ] Spanish (AI Generated)
- [ ] French (AI Generated)
- [ ] German (AI Generated)
- [ ] Portuguese (AI Generated)
- [ ] Italian (AI Generated)

## Task Groups

### Relase 0.9 (*Upcoming*)
> Release on <Date>

14. Pages (*Continued*)
   - [ ] Activities Page
15. Change Password
    - [ ] Core Engine
16. Tabs Controller (*Continued*)
     - [ ] Getting Last Opened Tabs
     - [ ] Drag & Drop Tabs Index
17. Tags
    - [ ] Core Engine
    - [ ] Notes Link
18. Change Master Key
    > Re-encypts all data (DB, Notes, config) with new key */
    - [ ] Core Engine
19. Import (*Continued*)
    - [ ] Import all data from backups
20. Export
    - [ ] Export Note as MD
    - [ ] Export Note as PDF
    - [ ] Export Notes with encryption
    - [ ] Export all notes as MD or PDF
21. Search via Note Name
    - [ ] Core Engine
        > Content search not feasible as all data is only decrypted when the note is accessed (for safety) even embeddings can be used to deduce the overall theme and topic, thus, not feasible
22. Code Clean
    - [ ] Remove Clutter
    - [ ] Clean Functions
    - [ ] Remove unused code
    - [ ] Remove duplicate code
23. Filter
    - [ ] Core Engine
    - [ ] Created Date Range
    - [ ] Updated Date Range
    - [ ] Tags
24. SymLink (*Link one note to another*)
    - [ ] Core Engine
25. Attach Files
    - [ ] Core Engine
    - [ ] Save Encrypted Files
26. Security Audit
    - [ ] Memory Cleaning
    - [ ] Memory Locking for Keys
    - [ ] Side Channel attacks prevention
    - [ ] Secure protection
27. Code Optimization
    - [ ] Memory Audit
    - [ ] Performance Audit
28. Notes History Page
    - [ ] See all history
    - [ ] Manage diff and restore
    - [ ] Save history as new note

### Release 0.5 (*Latest*)
> Released on <Date>

1. Application Core
   - [x] Main State
   - [x] Page Routing
   - [x] Modal Routing
   - [x] Main State Zeroizing
2. Pages
   - [x] Register Page
   - [x] Login Page
   - [x] Editor Page
3. Modal
   - [x] Settings Modal
   - [x] Change Password Modal
   - [x] Change Master Key Modal
   - [x] Update App Modal
4. Languages
   - [x] i18n Configuration
   - [x] English Language
5. Encryption
   - [x] Master Key Derivation using Argon2id from Password
   - [x] DB, File Key Derivation using HKDF, SHA512 from Master Key
   - [x] Zeroizing DB, File, Master Key, Password, Nonce, Key, Salt
6. Database
   - [x] DatabaseService
   - [x] Creating SQLCipher using Db Key
   - [x] Migration Engine
7. Configuration
   - [x] Auto Save
   - [x] Auto Lock
   - [x] Main Directory
   - [x] Backup Directory
   - [x] Safe Copy
   - [x] Syntax Highlight
   - [x] Selected Language
   - [x] Persistence in DB
   - [x] Zeroizing Config
8. Notes
   - [x] Encyption of text using File Key via XChaCha20-Poly1305
   - [x] Decryption of text using File Key
   - [x] CRUD in DB notes table
   - [x] Auto save text
   - [x] Encrypted text history (*recording only; display to be done in #28*)
   - [x] Zeroizing Notes
9. Tabs Controller
   - [x] Creating Core Engine
   - [x] Tabs & Controller
10. Activities (*recording only; display to be done in #14*)
    - [x] Core Engine
    - [ ] Authentication
    - [ ] Configuration
    - [ ] CRUD
11. Import
    - [x] Import Notes from MD
    - [x] Import Notes from TXT
12. Workspace Feature
    - [x] Create Default Workspace
    - [x] Create New Workspace
    - [x] Modify Name of Workspace
    - [x] Delete Workspace
13. Notebook Feature
    - [ ] Create Default Notebook
    - [ ] Create New Notebook
    - [ ] Modify Name of Notebook
    - [ ] Delete Notebook
14. Safe Copy
    - [ ] Core Engine
    - [ ] Safe Paste
15. Update
    - [ ] Check update via GitHub Releases API
16. Backup
    - [ ] Backup DB & State to backup folder at intervals