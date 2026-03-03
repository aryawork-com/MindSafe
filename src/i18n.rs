use egui::Ui;
use egui_i18n::tr;

// Uncomment once Language translation done
// Add more if required
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Language {
    English,
    Spanish,
    // Hindi,
    // French,
    // German,
    // Portuguese,
    // Italian,
    // Dutch,
    // Russian,
    // Chinese,
    // Japanese,
    // Korean,
    // Arabic,
    // Turkish,
    // Bengali,
    // Urdu,
    // Persian,
    // Hebrew,
    // Greek,
    // Polish,
    // Swedish,
    // Danish,
    // Finnish,
    // Norwegian,
    // Czech,
    // Hungarian,
    // Thai,
    // Vietnamese,
    // Indonesian,
    // Malay,
    // Tamil,
    // Telugu,
    // Kannada,
}

impl Language {
    pub fn get_name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            // Language::Hindi => "हिन्दी",
            // Language::French => "Français",
            // Language::German => "Deutsch",
            // Language::Portuguese => "Português",
            // Language::Italian => "Italiano",
            // Language::Dutch => "Nederlands",
            // Language::Russian => "Русский",
            // Language::Chinese => "中文",
            // Language::Japanese => "日本語",
            // Language::Korean => "한국어",
            // Language::Arabic => "العربية",
            // Language::Turkish => "Türkçe",
            // Language::Bengali => "বাংলা",
            // Language::Urdu => "اردو",
            // Language::Persian => "فارسی",
            // Language::Hebrew => "עברית",
            // Language::Greek => "Ελληνικά",
            // Language::Polish => "Polski",
            // Language::Swedish => "Svenska",
            // Language::Danish => "Dansk",
            // Language::Finnish => "Suomi",
            // Language::Norwegian => "Norsk",
            // Language::Czech => "Čeština",
            // Language::Hungarian => "Magyar",
            // Language::Thai => "ไทย",
            // Language::Vietnamese => "Tiếng Việt",
            // Language::Indonesian => "Bahasa Indonesia",
            // Language::Malay => "Bahasa Melayu",
            // Language::Tamil => "தமிழ்",
            // Language::Telugu => "తెలుగు",
            // Language::Kannada => "ಕನ್ನಡ",
        }
    }
    pub fn get_code(&self) -> &str {
        match self {
            Language::English => "en-US",
            Language::Spanish => "es-ES",
            // Language::Hindi => "hi-IN",
            // Language::French => "fr-FR",
            // Language::German => "de-DE",
            // Language::Portuguese => "pt-BR",
            // Language::Italian => "it-IT",
            // Language::Dutch => "nl-NL",
            // Language::Russian => "ru-RU",
            // Language::Chinese => "zh-CN",
            // Language::Japanese => "ja-JP",
            // Language::Korean => "ko-KR",
            // Language::Arabic => "ar-SA",
            // Language::Turkish => "tr-TR",
            // Language::Bengali => "bn-IN",
            // Language::Urdu => "ur-IN",
            // Language::Persian => "fa-IR",
            // Language::Hebrew => "he-IL",
            // Language::Greek => "el-GR",
            // Language::Polish => "pl-PL",
            // Language::Swedish => "sv-SE",
            // Language::Danish => "da-DK",
            // Language::Finnish => "fi-FI",
            // Language::Norwegian => "nb-NO",
            // Language::Czech => "cs-CZ",
            // Language::Hungarian => "hu-HU",
            // Language::Thai => "th-TH",
            // Language::Vietnamese => "vi-VN",
            // Language::Indonesian => "id-ID",
            // Language::Malay => "ms-MY",
            // Language::Tamil => "ta-IN",
            // Language::Telugu => "te-IN",
            // Language::Kannada => "kn-IN",
        }
    }

    pub fn parse_code(code: &str) -> Self {
        match code {
            "en-US" => Language::English,
            "es-ES" => Language::Spanish,
            // "hi-IN" => Language::Hindi,
            // "fr-FR" => Language::French,
            // "de-DE" => Language::German,
            // "pt-BR" => Language::Portuguese,
            // "it-IT" => Language::Italian,
            // "nl-NL" => Language::Dutch,
            // "ru-RU" => Language::Russian,
            // "zh-CN" => Language::Chinese,
            // "ja-JP" => Language::Japanese,
            // "ko-KR" => Language::Korean,
            // "ar-SA" => Language::Arabic,
            // "tr-TR" => Language::Turkish,
            // "bn-BD" => Language::Bengali,
            // "ur-IN" => Language::Urdu,
            // "fa-IR" => Language::Persian,
            // "he-IL" => Language::Hebrew,
            // "el-GR" => Language::Greek,
            // "pl-PL" => Language::Polish,
            // "sv-SE" => Language::Swedish,
            // "da-DK" => Language::Danish,
            // "fi-FI" => Language::Finnish,
            // "nb-NO" => Language::Norwegian,
            // "cs-CZ" => Language::Czech,
            // "hu-HU" => Language::Hungarian,
            // "th-TH" => Language::Thai,
            // "vi-VN" => Language::Vietnamese,
            // "id-ID" => Language::Indonesian,
            // "ms-MY" => Language::Malay,
            // "ta-IN" => Language::Tamil,
            // "te-IN" => Language::Telugu,
            // "kn-IN" => Language::Kannada,
            _ => Language::English,
        }
    }
}

pub fn language_dropdown(ui: &mut Ui, selected: &mut Language) {
    let before = *selected;
    ui.columns_const(|[_col0, col1, col2, _col3]| {
        col1.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.label(tr!("language"));
        });
        col2.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            egui::ComboBox::from_id_salt("language-box")
                .selected_text(selected.get_name().to_string())
                // .width(f32::MAX)
                .show_ui(ui, |ui| {
                    ui.selectable_value(selected, Language::English, Language::English.get_name());
                    ui.selectable_value(selected, Language::Spanish, Language::Spanish.get_name());
                    // ui.selectable_value(selected, Language::Hindi, Language::Hindi.get_name());
                    // ui.selectable_value(selected, Language::French, Language::French.get_name());
                    // ui.selectable_value(selected, Language::German, Language::German.get_name());
                    // ui.selectable_value(
                    //     selected,
                    //     Language::Portuguese,
                    //     Language::Portuguese.get_name(),
                    // );
                    // ui.selectable_value(selected, Language::Italian, Language::Italian.get_name());
                    // ui.selectable_value(selected, Language::Dutch, Language::Dutch.get_name());
                    // ui.selectable_value(selected, Language::Russian, Language::Russian.get_name());
                    // ui.selectable_value(selected, Language::Chinese, Language::Chinese.get_name());
                    // ui.selectable_value(
                    //     selected,
                    //     Language::Japanese,
                    //     Language::Japanese.get_name(),
                    // );
                    // ui.selectable_value(selected, Language::Korean, Language::Korean.get_name());
                    // ui.selectable_value(selected, Language::Arabic, Language::Arabic.get_name());
                    // ui.selectable_value(selected, Language::Turkish, Language::Turkish.get_name());
                    // ui.selectable_value(selected, Language::Bengali, Language::Bengali.get_name());
                    // ui.selectable_value(selected, Language::Urdu, Language::Urdu.get_name());
                    // ui.selectable_value(selected, Language::Persian, Language::Persian.get_name());
                    // ui.selectable_value(selected, Language::Hebrew, Language::Hebrew.get_name());
                    // ui.selectable_value(selected, Language::Greek, Language::Greek.get_name());
                    // ui.selectable_value(selected, Language::Polish, Language::Polish.get_name());
                    // ui.selectable_value(selected, Language::Swedish, Language::Swedish.get_name());
                    // ui.selectable_value(selected, Language::Danish, Language::Danish.get_name());
                    // ui.selectable_value(selected, Language::Finnish, Language::Finnish.get_name());
                    // ui.selectable_value(
                    //     selected,
                    //     Language::Norwegian,
                    //     Language::Norwegian.get_name(),
                    // );
                    // ui.selectable_value(selected, Language::Czech, Language::Czech.get_name());
                    // ui.selectable_value(
                    //     selected,
                    //     Language::Hungarian,
                    //     Language::Hungarian.get_name(),
                    // );
                    // ui.selectable_value(selected, Language::Thai, Language::Thai.get_name());
                    // ui.selectable_value(
                    //     selected,
                    //     Language::Vietnamese,
                    //     Language::Vietnamese.get_name(),
                    // );
                    // ui.selectable_value(
                    //     selected,
                    //     Language::Indonesian,
                    //     Language::Indonesian.get_name(),
                    // );
                    // ui.selectable_value(selected, Language::Malay, Language::Malay.get_name());
                    // ui.selectable_value(selected, Language::Tamil, Language::Tamil.get_name());
                    // ui.selectable_value(selected, Language::Telugu, Language::Telugu.get_name());
                    // ui.selectable_value(selected, Language::Kannada, Language::Kannada.get_name());
                });
        });
    });

    if *selected != before {
        egui_i18n::set_language(selected.get_code());
        // ctx.request_repaint();
    }
}

/// Loading Languages
pub fn init_languages() {
    let en_us = String::from_utf8_lossy(include_bytes!("../assets/languages/en-US.ftl"));
    let es_es = String::from_utf8_lossy(include_bytes!("../assets/languages/es-ES.ftl"));
    // let hi_IN = String::from_utf8_lossy(include_bytes!("../assets/languages/hi-IN.ftl"));
    // let fr_FR = String::from_utf8_lossy(include_bytes!("../assets/languages/fr-FR.ftl"));
    // let de_DE = String::from_utf8_lossy(include_bytes!("../assets/languages/de-DE.ftl"));
    // let pt_BR = String::from_utf8_lossy(include_bytes!("../assets/languages/pt-BR.ftl"));
    // let it_IT = String::from_utf8_lossy(include_bytes!("../assets/languages/it-IT.ftl"));
    // let nl_NL = String::from_utf8_lossy(include_bytes!("../assets/languages/nl-NL.ftl"));
    // let ru_RU = String::from_utf8_lossy(include_bytes!("../assets/languages/ru-RU.ftl"));
    // let zh_CN = String::from_utf8_lossy(include_bytes!("../assets/languages/zh-CN.ftl"));
    // let ja_JP = String::from_utf8_lossy(include_bytes!("../assets/languages/ja-JP.ftl"));
    // let ko_KR = String::from_utf8_lossy(include_bytes!("../assets/languages/ko-KR.ftl"));
    // let ar_SA = String::from_utf8_lossy(include_bytes!("../assets/languages/ar-SA.ftl"));
    // let tr_TR = String::from_utf8_lossy(include_bytes!("../assets/languages/tr-TR.ftl"));
    // let bn_BD = String::from_utf8_lossy(include_bytes!("../assets/languages/bn-BD.ftl"));
    // let ur_IN = String::from_utf8_lossy(include_bytes!("../assets/languages/ur-IN.ftl"));
    // let fa_IR = String::from_utf8_lossy(include_bytes!("../assets/languages/fa-IR.ftl"));
    // let he_IL = String::from_utf8_lossy(include_bytes!("../assets/languages/he-IL.ftl"));
    // let el_GR = String::from_utf8_lossy(include_bytes!("../assets/languages/el-GR.ftl"));
    // let pl_PL = String::from_utf8_lossy(include_bytes!("../assets/languages/pl-PL.ftl"));
    // let sv_SE = String::from_utf8_lossy(include_bytes!("../assets/languages/sv-SE.ftl"));
    // let da_DK = String::from_utf8_lossy(include_bytes!("../assets/languages/da-DK.ftl"));
    // let fi_FI = String::from_utf8_lossy(include_bytes!("../assets/languages/fi-FI.ftl"));
    // let nb_NO = String::from_utf8_lossy(include_bytes!("../assets/languages/nb-NO.ftl"));
    // let cs_CZ = String::from_utf8_lossy(include_bytes!("../assets/languages/cs-CZ.ftl"));
    // let hu_HU = String::from_utf8_lossy(include_bytes!("../assets/languages/hu-HU.ftl"));
    // let th_TH = String::from_utf8_lossy(include_bytes!("../assets/languages/th-TH.ftl"));
    // let vi_VN = String::from_utf8_lossy(include_bytes!("../assets/languages/vi-VN.ftl"));
    // let id_ID = String::from_utf8_lossy(include_bytes!("../assets/languages/id-ID.ftl"));
    // let ms_MY = String::from_utf8_lossy(include_bytes!("../assets/languages/ms-MY.ftl"));
    // let ta_IN = String::from_utf8_lossy(include_bytes!("../assets/languages/ta-IN.ftl"));
    // let te_IN = String::from_utf8_lossy(include_bytes!("../assets/languages/te-IN.ftl"));
    // let kn_IN = String::from_utf8_lossy(include_bytes!("../assets/languages/kn-IN.ftl"));
    egui_i18n::load_translations_from_text("en-US", en_us).unwrap();
    egui_i18n::load_translations_from_text("es-ES", es_es).unwrap();
    // egui_i18n::load_translations_from_text("hi-IN", hi_IN).unwrap();
    // egui_i18n::load_translations_from_text("fr-FR", fr_FR).unwrap();
    // egui_i18n::load_translations_from_text("de-DE", de_DE).unwrap();
    // egui_i18n::load_translations_from_text("pt-BR", pt_BR).unwrap();
    // egui_i18n::load_translations_from_text("it-IT", it_IT).unwrap();
    // egui_i18n::load_translations_from_text("nl-NL", nl_NL).unwrap();
    // egui_i18n::load_translations_from_text("ru-RU", ru_RU).unwrap();
    // egui_i18n::load_translations_from_text("zh-CN", zh_CN).unwrap();
    // egui_i18n::load_translations_from_text("ja-JP", ja_JP).unwrap();
    // egui_i18n::load_translations_from_text("ko-KR", ko_KR).unwrap();
    // egui_i18n::load_translations_from_text("ar-SA", ar_SA).unwrap();
    // egui_i18n::load_translations_from_text("tr-TR", tr_TR).unwrap();
    // egui_i18n::load_translations_from_text("bn-BD", bn_BD).unwrap();
    // egui_i18n::load_translations_from_text("ur-IN", ur_IN).unwrap();
    // egui_i18n::load_translations_from_text("fa-IR", fa_IR).unwrap();
    // egui_i18n::load_translations_from_text("he-IL", he_IL).unwrap();
    // egui_i18n::load_translations_from_text("el-GR", el_GR).unwrap();
    // egui_i18n::load_translations_from_text("pl-PL", pl_PL).unwrap();
    // egui_i18n::load_translations_from_text("sv-SE", sv_SE).unwrap();
    // egui_i18n::load_translations_from_text("da-DK", da_DK).unwrap();
    // egui_i18n::load_translations_from_text("fi-FI", fi_FI).unwrap();
    // egui_i18n::load_translations_from_text("nb-NO", nb_NO).unwrap();
    // egui_i18n::load_translations_from_text("cs-CZ", cs_CZ).unwrap();
    // egui_i18n::load_translations_from_text("hu-HU", hu_HU).unwrap();
    // egui_i18n::load_translations_from_text("th-TH", th_TH).unwrap();
    // egui_i18n::load_translations_from_text("vi-VN", vi_VN).unwrap();
    // egui_i18n::load_translations_from_text("id-ID", id_ID).unwrap();
    // egui_i18n::load_translations_from_text("ms-MY", ms_MY).unwrap();
    // egui_i18n::load_translations_from_text("ta-IN", ta_IN).unwrap();
    // egui_i18n::load_translations_from_text("te-IN", te_IN).unwrap();
    // egui_i18n::load_translations_from_text("kn-IN", kn_IN).unwrap();

    // Fall back to English
    egui_i18n::set_language("en-US");
    egui_i18n::set_fallback("en-US");
}
