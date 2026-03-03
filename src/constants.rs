// Internal modules usage
use crate::services::database::Migration;

// Migration constants
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        serial_id: 1,
        name: "create_config_table",
        query: "CREATE TABLE IF NOT EXISTS config (
            id INTEGER PRIMARY KEY,
            selected_language TEXT NOT NULL DEFAULT 'English',
            main_directory TEXT DEFAULT NULL,
            backup_directory TEXT DEFAULT NULL,
            safe_copy BOOLEAN NOT NULL DEFAULT 1,
            syntax_highlight BOOLEAN NOT NULL DEFAULT 1,
            auto_save_duration INTEGER NOT NULL DEFAULT 5,
            auto_lock_duration INTEGER NOT NULL DEFAULT 30,
            sorting INTEGER NOT NULL DEFAULT 4,
            last_opened_note TEXT DEFAULT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc')),
            updated_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc'))
        );",
        version: "1.0.0",
    },
    Migration {
        serial_id: 2,
        name: "create_config_triggers",
        query: "CREATE TRIGGER IF NOT EXISTS update_config_updated_at
            BEFORE UPDATE ON config
            FOR EACH ROW
            BEGIN
                SELECT NEW.updated_at = DATETIME('now', 'utc');
            END;",
        version: "1.0.0",
    },
    Migration {
        serial_id: 3,
        name: "create_notes_table",
        query: "CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            blob TEXT DEFAULT NULL,
            sha_hash TEXT DEFAULT NULL,
            starred BOOLEAN NOT NULL DEFAULT 0,
            disable_auto_save BOOLEAN NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc')),
            updated_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc'))
        );",
        version: "1.0.0",
    },
    Migration {
        serial_id: 4,
        name: "create_notes_triggers",
        query: "CREATE TRIGGER IF NOT EXISTS update_notes_updated_at
            BEFORE UPDATE ON notes
            FOR EACH ROW
            BEGIN
                SELECT NEW.updated_at = DATETIME('now', 'utc');
            END;",
        version: "1.0.0",
    },
    Migration {
        serial_id: 5,
        name: "create_notes_history_table",
        query: "CREATE TABLE IF NOT EXISTS notes_history (
            id TEXT PRIMARY KEY,
            note_id TEXT NOT NULL,
            blob TEXT DEFAULT NULL,
            sha_hash TEXT NOT NULL,
            duplicate_id TEXT DEFAULT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc'))
        );",
        version: "1.0.0",
    },
    Migration {
        serial_id: 6,
        name: "create_activities_table",
        query: "CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT DEFAULT NULL,
            activity_type INTEGER NOT NULL,
            retain_duration INTEGER NOT NULL DEFAULT 30,
            enabled BOOLEAN NOT NULL DEFAULT 0,
            requires_review BOOLEAN NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc')),
            updated_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc'))
        );",
        version: "1.0.0",
    },
    Migration {
        serial_id: 7,
        name: "create_activities_triggers",
        query: "CREATE TRIGGER IF NOT EXISTS update_activities_updated_at
            BEFORE UPDATE ON activities
            FOR EACH ROW
            BEGIN
                SELECT NEW.updated_at = DATETIME('now', 'utc');
            END;",
        version: "1.0.0",
    },
    Migration {
        serial_id: 8,
        name: "create_activity_logs_table",
        query: "CREATE TABLE IF NOT EXISTS activity_logs (
            id TEXT PRIMARY KEY,
            activity_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            item_id TEXT DEFAULT NULL,
            details TEXT DEFAULT NULL,
            reviewed BOOLEAN NOT NULL DEFAULT 0,
            starred BOOLEAN NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc')),
            updated_at TIMESTAMP NOT NULL DEFAULT (DATETIME('now', 'utc'))
        );",
        version: "1.0.0",
    },
    Migration {
        serial_id: 9,
        name: "create_activity_logs_triggers",
        query: "CREATE TRIGGER IF NOT EXISTS update_activity_logs_updated_at
            BEFORE UPDATE ON activity_logs
            FOR EACH ROW
            BEGIN
                SELECT NEW.updated_at = DATETIME('now', 'utc');
            END;",
        version: "1.0.0",
    },
];

pub const HELP_TEXT: &str = r#"# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5

**This is bold text**

==This is normal highlighted text==  or  =x=and error highlighted text=x=  or  =+=also success highlighted text=+=

*This is italic text* and also _another italic style_  

~~This is strikethrough~~

---

> Blockquote level 1
>> Nested blockquote level 2
>>> Even deeper blockquote level 3

- Unordered list item
- Another bullet

1. Ordered item one
2. Ordered item two

- [_] Unchecked checklist item
- [x] Checked checklist item
- [*] Crossed checklist item

Inline `code sample` inside text.

```rust
fn main() {
    println!("This is fenced code block with language tag");
}
```

| Column A | Column B | Column C |
| -------- | -------- | -------- |
| Row 1    | Data     | More     |
| Row 2    | Values   | Here     |

[This is a link](https://example.com)

!> This is a warning note

i> This is an info note

x> This is an error note

+> This is a success note

*> This is a reminder note"#;
