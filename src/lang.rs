use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    Japanese,
    Chinese,
    Semitic,
    Brahmic,
    Cyrillic,
    Greek,
    Hangul,
    English,
    Other,
}

impl Script {
    pub fn detect(text: &str) -> Script {
        for ch in text.chars() {
            let cp = ch as u32;
            for (script, ranges) in SCRIPT_RANGES {
                if ranges.iter().any(|(lo, hi)| (*lo..=*hi).contains(&cp)) {
                    return *script;
                }
            }
        }
        Script::Other
    }
}

const KANA: &[(u32, u32)] = &[(0x3040, 0x309F), (0x30A0, 0x30FF)];
const CJK: &[(u32, u32)] = &[(0x4E00, 0x9FFF), (0x3400, 0x4DBF)];
const HANGUL: &[(u32, u32)] = &[(0xAC00, 0xD7AF)];
const CYRILLIC: &[(u32, u32)] = &[(0x0400, 0x04FF), (0x0500, 0x052F)];
const GREEK: &[(u32, u32)] = &[(0x0370, 0x03FF), (0x1F00, 0x1FFF)];
const LATIN: &[(u32, u32)] = &[(0x0041, 0x005A), (0x0061, 0x007A)];
const ARABIC: &[(u32, u32)] = &[(0x0600, 0x06FF), (0x0750, 0x077F)];
const HEBREW: &[(u32, u32)] = &[(0x0590, 0x05FF)];
const BRAHMIC: &[(u32, u32)] = &[(0x0900, 0x097F), (0x0980, 0x09FF)];

const SCRIPT_RANGES: &[(Script, &[(u32, u32)])] = &[
    (Script::Japanese, KANA),
    (Script::Chinese, CJK),
    (Script::Semitic, ARABIC),
    (Script::Semitic, HEBREW),
    (Script::Brahmic, BRAHMIC),
    (Script::Cyrillic, CYRILLIC),
    (Script::Greek, GREEK),
    (Script::Hangul, HANGUL),
    (Script::English, LATIN),
];

pub fn triage(text: &str) -> (Script, Vec<String>) {
    let script = Script::detect(text);
    let fb = tokens::get_fallbacks();
    let key = match script {
        Script::Other => "other",
        s => match s {
            Script::Japanese => "japanese",
            Script::Chinese => "chinese",
            Script::Semitic => "semitic",
            Script::Brahmic => "brahmic",
            Script::Cyrillic => "cyrillic",
            Script::Greek => "greek",
            Script::Hangul => "hangul",
            Script::English => "english",
            Script::Other => "other",
        },
    };
    let chain = fb.get(key).cloned().unwrap_or_default();
    (script, chain)
}

pub fn is_alpha(text: &str) -> bool {
    Script::detect(text) != Script::Other
}
