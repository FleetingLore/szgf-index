//! 国际化模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub type LocalePack = HashMap<String, String>;
pub type Locale = String;

#[derive(Debug, Clone)]
pub struct I18nManager {
    default_locale: Locale,
    packs: HashMap<Locale, LocalePack>,
    locale_names: HashMap<Locale, String>,
}

impl I18nManager {
    pub fn load(config_dir: &PathBuf, default_locale: &str) -> Self {
        let mut packs: HashMap<Locale, LocalePack> = HashMap::new();
        let mut locale_names = HashMap::new();

        let i18n_dir = config_dir.join("i18n");
        eprintln!("I18n: loading from {:?}", i18n_dir);
        match std::fs::read_dir(&i18n_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    eprintln!("I18n: found file {:?}", path);
                    if path.extension().map_or(false, |e| e == "json") {
                        let code = path.file_stem().unwrap().to_string_lossy().to_string();
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(pack) = serde_json::from_str::<LocalePack>(&content) {
                                if let Some(name) = pack.get("@meta.name") {
                                    locale_names.insert(code.clone(), name.clone());
                                }
                                packs.insert(code, pack);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("I18n: failed to read i18n dir: {}", e);
            }
        }

        eprintln!("I18n: loaded {} language packs", packs.len());

        Self {
            default_locale: default_locale.to_string(),
            packs,
            locale_names,
        }
    }

    pub fn resolve(&self, query_lang: Option<&str>, accept_lang: Option<&str>) -> String {
        if let Some(lang) = query_lang {
            if self.packs.contains_key(lang) {
                return lang.to_string();
            }
            if let Some(full) = self.find_by_prefix(lang) {
                return full.to_string();
            }
        }
        if let Some(header) = accept_lang {
            for part in header.split(',') {
                let lang = part.split(';').next().unwrap_or("").trim();
                if self.packs.contains_key(lang) {
                    return lang.to_string();
                }
                if let Some(full) = self.find_by_prefix(lang) {
                    return full.to_string();
                }
            }
        }
        self.default_locale.clone()
    }

    fn find_by_prefix(&self, prefix: &str) -> Option<&str> {
        let prefix_lower = prefix.to_lowercase();
        self.packs
            .keys()
            .find(|k| k.to_lowercase().starts_with(&prefix_lower))
            .map(|s| s.as_str())
    }

    pub fn t(&self, locale: &str, key: &str) -> String {
        self.packs
            .get(locale)
            .and_then(|pack| pack.get(key))
            .cloned()
            .or_else(|| {
                self.packs
                    .get(&self.default_locale)
                    .and_then(|pack| pack.get(key))
                    .cloned()
            })
            .unwrap_or_else(|| format!("[{}]", key))
    }

    pub fn tf(&self, locale: &str, key: &str, params: &HashMap<&str, &str>) -> String {
        let mut text = self.t(locale, key);
        for (k, v) in params {
            text = text.replace(&format!("{{{{{}}}}}", k), v);
        }
        text
    }

    pub fn available_locales(&self) -> Vec<LocaleInfo> {
        self.packs
            .keys()
            .map(|code| LocaleInfo {
                code: code.clone(),
                name: self
                    .locale_names
                    .get(code)
                    .cloned()
                    .unwrap_or_else(|| code.clone()),
            })
            .collect()
    }

    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocaleInfo {
    pub code: String,
    pub name: String,
}
