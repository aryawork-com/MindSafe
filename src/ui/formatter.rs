use ::egui::Stroke;
use egui::{Color32, FontId, TextFormat, text::LayoutJob};

fn md_color(kind: &str) -> Color32 {
    match kind {
        "h1" => Color32::from_rgb(255, 180, 80),
        "h2" => Color32::from_rgb(255, 140, 80),
        "h3" => Color32::from_rgb(255, 100, 80),
        "h4" => Color32::from_rgb(200, 160, 255),
        "h5" => Color32::from_rgb(180, 180, 255),
        "bold" => Color32::from_rgb(255, 220, 200),
        "highlight" => Color32::from_rgb(252, 239, 180),
        "highlight_error" => Color32::from_rgb(251, 177, 189),
        "highlight_success" => Color32::from_rgb(146, 230, 167),
        "checked" => Color32::from_rgb(146, 230, 167),
        "unchecked" => Color32::from_rgb(100, 100, 100),
        "cross-checked" => Color32::from_rgb(251, 177, 189),
        "italic" => Color32::from_rgb(200, 220, 255),
        "strike" => Color32::from_rgb(255, 150, 220),
        "quote" => Color32::from_rgb(150, 200, 255),
        "olist" => Color32::from_rgb(220, 220, 170),
        "ulist" => Color32::from_rgb(170, 220, 220),
        "code" => Color32::from_rgb(200, 180, 255),
        "table" => Color32::from_rgb(120, 255, 200),
        "link" => Color32::from_rgb(120, 200, 255),
        "warn" => Color32::from_rgb(255, 180, 180),
        "info" => Color32::from_rgb(180, 200, 255),
        "error" => Color32::from_rgb(255, 100, 100),
        "success" => Color32::from_rgb(150, 255, 180),
        "reminder" => Color32::from_rgb(255, 220, 150),
        "sep" => Color32::from_rgb(100, 100, 100),
        _ => Color32::from_rgb(220, 220, 220),
    }
}

pub fn md_formatter(text: &str, syntax_highlight: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    if !syntax_highlight {
        job.append(text, 0.0, TextFormat::default());
        return job;
    }
    let mut sections: Vec<(usize, usize, TextFormat)> = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;

    while i < chars.len() {
        let (byte, ch) = chars[i];

        // 1. Headings (#, ##, ...)
        if ch == '#' {
            let mut level = 1;
            let mut j = i + 1;
            while j < chars.len() && chars[j].1 == '#' {
                level += 1;
                j += 1;
            }
            // find end of line
            let mut k = j;
            while k < chars.len() && chars[k].1 != '\n' {
                k += 1;
            }
            let end = if k < chars.len() {
                chars[k].0
            } else {
                text.len()
            };
            let fmt = TextFormat {
                font_id: FontId::proportional(26.0 - (level as f32 * 2.0)),
                color: md_color(&format!("h{level}")),
                ..Default::default()
            };
            sections.push((byte, end, fmt));
            i = k;
            continue;
        }

        // 2. Bold **text**
        if text[byte..].starts_with("**")
            && let Some(rel) = text[byte + 2..].find("**")
        {
            let end = byte + 2 + rel;
            sections.push((
                byte,
                end + 2,
                TextFormat {
                    font_id: FontId::proportional(15.0),
                    color: md_color("bold"),
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b <= end + 1);
            continue;
        }

        // 3. Highlight ==text==
        if text[byte..].starts_with("==") {
            if let Some(rel) = text[byte + 2..].find("==") {
                let end = byte + 2 + rel;
                sections.push((
                    byte,
                    end + 2,
                    TextFormat {
                        font_id: FontId::proportional(15.0),
                        color: Color32::BLACK,
                        background: md_color("highlight"),
                        ..Default::default()
                    },
                ));
                i = chars.partition_point(|(b, _)| *b <= end + 1);
                continue;
            }
        } else if text[byte..].starts_with("=x=") {
            if let Some(rel) = text[byte + 1..].find("=x=") {
                let end = byte + 2 + rel;
                sections.push((
                    byte,
                    end + 2,
                    TextFormat {
                        font_id: FontId::proportional(15.0),
                        color: Color32::BLACK,
                        background: md_color("highlight_error"),
                        ..Default::default()
                    },
                ));
                i = chars.partition_point(|(b, _)| *b <= end + 1);
                continue;
            }
        } else if text[byte..].starts_with("=+=")
            && let Some(rel) = text[byte + 1..].find("=+=")
        {
            let end = byte + 2 + rel;
            sections.push((
                byte,
                end + 2,
                TextFormat {
                    font_id: FontId::proportional(15.0),
                    color: Color32::BLACK,
                    background: md_color("highlight_success"),
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b <= end + 1);
            continue;
        }

        // 4. Italic * or _
        if (ch == '*' || ch == '_')
            && let Some(rel) = text[byte + 1..].find(ch)
        {
            let end = byte + 1 + rel;
            sections.push((
                byte,
                end + 1,
                TextFormat {
                    font_id: FontId::proportional(15.0),
                    color: md_color("italic"),
                    italics: true,
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b <= end);
            continue;
        }

        // 5. Strikethrough ~~
        if text[byte..].starts_with("~~")
            && let Some(rel) = text[byte + 2..].find("~~")
        {
            let end = byte + 2 + rel;
            sections.push((
                byte,
                end + 2,
                TextFormat {
                    font_id: FontId::proportional(15.0),
                    color: md_color("strike"),
                    strikethrough: Stroke::new(1.0, Color32::GRAY),
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b <= end + 1);
            continue;
        }

        // 6. Horizontal rules --- *** ___
        if text[byte..].starts_with("---")
            || text[byte..].starts_with("***")
            || text[byte..].starts_with("___")
        {
            let end = byte + 3;
            sections.push((
                byte,
                end,
                TextFormat {
                    font_id: FontId::monospace(16.0),
                    color: md_color("sep"),
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b < end);
            continue;
        }

        // NOTE: Commented earlier logic, as added kinds in this only
        // 7. Blockquote >
        // if ch == '>' {
        //     let mut j = i;
        //     while j < chars.len() && chars[j].1 != '\n' {
        //         j += 1;
        //     }
        //     let end = if j < chars.len() {
        //         chars[j].0
        //     } else {
        //         text.len()
        //     };
        //     sections.push((
        //         byte,
        //         end,
        //         TextFormat {
        //             font_id: FontId::proportional(16.0),
        //             color: md_color("quote"),
        //             ..Default::default()
        //         },
        //     ));
        //     i = j;
        //     continue;
        // }
        // Blockquote with kind (!>, i>, x>, +>, *>)
        if ch == '>'
            || text[byte..].starts_with("!>")
            || text[byte..].starts_with("i>")
            || text[byte..].starts_with("x>")
            || text[byte..].starts_with("+>")
            || text[byte..].starts_with("*>")
        {
            let mut j = i;
            // checking if flowing char after symbol is ">"
            // find end of current line
            while j < chars.len() && chars[j].1 != '\n' {
                j += 1;
            }
            let line_end = if j < chars.len() {
                chars[j].0
            } else {
                text.len()
            };
            let line = &text[byte..line_end].to_lowercase();

            // determine kind based on prefix
            let kind = if line.starts_with("!>") {
                "warn"
            } else if line.starts_with("i>") {
                "info"
            } else if line.starts_with("x>") {
                "error"
            } else if line.starts_with("+>") {
                "success"
            } else if line.starts_with("*>") {
                "reminder"
            } else {
                "quote" // default for normal blockquotes
            };

            sections.push((
                byte,
                line_end,
                TextFormat {
                    font_id: FontId::proportional(16.0),
                    color: md_color(kind),
                    ..Default::default()
                },
            ));

            i = j; // move to next line
            continue;
        }

        // 12. Checkbox List
        if text[byte..].starts_with("- [_]")
            || text[byte..].starts_with("- [*]")
            || text[byte..].starts_with("- [x]")
        {
            let mut j = i;
            // checking if flowing char after symbol is ">"
            // find end of current line
            while j < chars.len() && chars[j].1 != '\n' {
                j += 1;
            }
            let line_end = if j < chars.len() {
                chars[j].0
            } else {
                text.len()
            };
            let line = &text[byte..line_end].to_lowercase();

            // determine kind based on prefix
            let kind = if line.starts_with("- [_]") {
                "unchecked"
            } else if line.starts_with("- [x]") {
                "checked"
            } else {
                "cross-checked"
            };

            sections.push((
                byte,
                line_end,
                TextFormat {
                    font_id: FontId::proportional(14.0),
                    color: md_color(kind),
                    ..Default::default()
                },
            ));

            i = j; // move to next line
            continue;
        }

        // 8. Lists - unordered / ordered
        if ch == '-' || ch == '*' {
            if i + 1 < chars.len() && chars[i + 1].1 == ' ' {
                let end = if i + 2 < chars.len() {
                    chars[i + 2].0
                } else {
                    text.len()
                };
                sections.push((
                    byte,
                    end,
                    TextFormat {
                        font_id: FontId::proportional(14.0),
                        color: md_color("ulist"),
                        ..Default::default()
                    },
                ));
            }
        } else if ch.is_ascii_digit() {
            let mut j = i;
            while j < chars.len() && chars[j].1.is_ascii_digit() {
                j += 1;
            }
            if j < chars.len() && chars[j].1 == '.' {
                let end = if j + 1 < chars.len() {
                    chars[j + 1].0
                } else {
                    text.len()
                };
                sections.push((
                    byte,
                    end,
                    TextFormat {
                        font_id: FontId::proportional(14.0),
                        color: md_color("olist"),
                        ..Default::default()
                    },
                ));
            }
        }

        // 9. Inline code `...`
        if ch == '`'
            && let Some(rel) = text[byte + 1..].find('`')
        {
            let end = byte + 1 + rel;
            sections.push((
                byte,
                end + 1,
                TextFormat {
                    font_id: FontId::monospace(15.0),
                    color: md_color("code"),
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b <= end);
            continue;
        }

        // 10. Tables | ... |
        if ch == '|' {
            let mut j = i;
            while j < chars.len() && chars[j].1 != '\n' {
                j += 1;
            }
            let end = if j < chars.len() {
                chars[j].0
            } else {
                text.len()
            };
            sections.push((
                byte,
                end,
                TextFormat {
                    font_id: FontId::monospace(14.0),
                    color: md_color("table"),
                    ..Default::default()
                },
            ));
            i = j;
            continue;
        }

        // 11. Links [..](..)
        if ch == '['
            && let Some(close) = text[byte..].find(')')
        {
            let end = byte + close + 1;
            sections.push((
                byte,
                end,
                TextFormat {
                    font_id: FontId::proportional(15.0),
                    color: md_color("link"),
                    ..Default::default()
                },
            ));
            i = chars.partition_point(|(b, _)| *b < end);
            continue;
        }

        // Note: Combined with blockquotes
        // 13. Notes ::: warning/info/etc.
        // if text[byte..].starts_with(":::") {
        //     // find end of the ::: start line
        //     let mut j = i;
        //     while j < chars.len() && chars[j].1 != '\n' {
        //         j += 1;
        //     }
        //     let line_end = if j < chars.len() {
        //         chars[j].0
        //     } else {
        //         text.len()
        //     };
        //     let line = &text[byte..line_end].to_lowercase();

        //     let kind = if line.contains("warning") {
        //         "warn"
        //     } else if line.contains("info") {
        //         "info"
        //     } else if line.contains("error") {
        //         "error"
        //     } else if line.contains("success") {
        //         "success"
        //     } else if line.contains("reminder") {
        //         "reminder"
        //     } else {
        //         "_"
        //     };

        //     // find the closing ::: line (after start line)
        //     let mut k = j + 1;
        //     let mut block_end = text.len();
        //     while k < chars.len() {
        //         if text[chars[k].0..].starts_with(":::") {
        //             block_end = chars[k].0; // stop BEFORE closing line
        //             break;
        //         }
        //         k += 1;
        //     }

        //     // color from the beginning of ::: line through block content,
        //     // but NOT the closing :::
        //     sections.push((
        //         byte,
        //         block_end,
        //         TextFormat {
        //             font_id: FontId::proportional(15.0),
        //             color: md_color(kind),
        //             ..Default::default()
        //         },
        //     ));

        //     // jump i to after the closing ::: if it exists
        //     if block_end < text.len() {
        //         // skip over the closing marker line entirely
        //         let mut after = k;
        //         while after < chars.len() && chars[after].1 != '\n' {
        //             after += 1;
        //         }
        //         i = after;
        //     } else {
        //         i = k;
        //     }

        //     continue;
        // }

        i += 1;
    }

    // Now build job
    let mut last = 0;
    for (start, end, fmt) in sections {
        if start > last {
            job.append(&text[last..start], 0.0, TextFormat::default());
        }
        if end <= text.len() {
            job.append(&text[start..end], 0.0, fmt);
            last = end;
        }
    }
    if last < text.len() {
        job.append(&text[last..], 0.0, TextFormat::default());
    }

    job
}
