use chrono::{DateTime, Utc};
use lexic_core::model::{Entry, Sense};
use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::DictError;
use crate::normalize::normalize;

const DEFAULT_BASE_URL: &str = "https://api.dictionaryapi.dev/api/v2/entries/en";

#[derive(Debug, Clone)]
pub struct LookupResult {
    pub entry: Entry,
    pub senses: Vec<Sense>,
}

#[derive(Debug, Clone)]
pub struct DictionaryClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for DictionaryClient {
    fn default() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            http,
            base_url: String::from(DEFAULT_BASE_URL),
        }
    }
}
impl DictionaryClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            ..Self::default()
        }
    }

    pub async fn lookup(&self, word: &str, now: DateTime<Utc>) -> Result<LookupResult, DictError> {
        let normalized = normalize(word)?;
        let encoded = urlencoding::encode(&normalized);
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), encoded);
        let response = self.http.get(url).send().await.map_err(map_request_error)?;
        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Err(DictError::NotFound);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(DictError::RateLimited);
        }
        if !status.is_success() {
            return Err(DictError::Http(status.as_u16()));
        }
        let entries: Vec<ApiEntry> = response
            .json()
            .await
            .map_err(|error| DictError::InvalidResponse(error.to_string()))?;
        map_entries(entries, normalized, now)
    }
}

fn map_request_error(error: reqwest::Error) -> DictError {
    if error.is_timeout() {
        DictError::Network(String::from("request timed out"))
    } else {
        DictError::Network(error.to_string())
    }
}

#[derive(Deserialize)]
struct ApiEntry {
    word: String,
    phonetic: Option<String>,
    phonetics: Option<Vec<ApiPhonetic>>,
    origin: Option<String>,
    meanings: Option<Vec<ApiMeaning>>,
    license: Option<ApiLicense>,
    #[serde(rename = "sourceUrls")]
    source_urls: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPhonetic {
    text: Option<String>,
    audio: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMeaning {
    part_of_speech: Option<String>,
    definitions: Option<Vec<ApiDefinition>>,
    synonyms: Option<Vec<String>>,
    antonyms: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ApiDefinition {
    definition: String,
    example: Option<String>,
    synonyms: Option<Vec<String>>,
    antonyms: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ApiLicense {
    name: Option<String>,
}

fn map_entries(
    entries: Vec<ApiEntry>,
    normalized: String,
    now: DateTime<Utc>,
) -> Result<LookupResult, DictError> {
    let first = entries.first().ok_or(DictError::NotFound)?;
    let phonetic = first.phonetic.clone().or_else(|| {
        first
            .phonetics
            .as_ref()
            .and_then(|items| items.iter().find_map(|item| item.text.clone()))
    });
    let audio_url = first.phonetics.as_ref().and_then(|items| {
        items
            .iter()
            .find_map(|item| item.audio.clone().filter(|value| !value.trim().is_empty()))
    });
    let source_url = first
        .source_urls
        .as_ref()
        .and_then(|urls| urls.first())
        .cloned();
    let license = first.license.as_ref().and_then(|value| value.name.clone());

    let mut senses = Vec::new();
    let mut order = 0_i32;
    if let Some(meanings) = &first.meanings {
        for meaning in meanings {
            let meaning_synonyms = meaning.synonyms.clone().unwrap_or_default();
            let meaning_antonyms = meaning.antonyms.clone().unwrap_or_default();
            if let Some(definitions) = &meaning.definitions {
                for definition in definitions {
                    let synonyms = if definition
                        .synonyms
                        .as_ref()
                        .map(|items| !items.is_empty())
                        .unwrap_or(false)
                    {
                        definition.synonyms.clone().unwrap_or_default()
                    } else {
                        meaning_synonyms.clone()
                    };
                    let antonyms = if definition
                        .antonyms
                        .as_ref()
                        .map(|items| !items.is_empty())
                        .unwrap_or(false)
                    {
                        definition.antonyms.clone().unwrap_or_default()
                    } else {
                        meaning_antonyms.clone()
                    };
                    senses.push(Sense {
                        id: Uuid::new_v4(),
                        entry_id: Uuid::new_v4(), // placeholder replaced below
                        part_of_speech: meaning.part_of_speech.clone(),
                        definition: definition.definition.clone(),
                        example: definition.example.clone(),
                        synonyms,
                        antonyms,
                        sort_order: order,
                    });
                    order += 1;
                }
            }
        }
    }

    if senses.is_empty() {
        return Err(DictError::NotFound);
    }

    let entry_id = Uuid::new_v4();
    for sense in &mut senses {
        sense.entry_id = entry_id;
    }

    let entry = Entry {
        id: entry_id,
        word: first.word.clone(),
        normalized_word: normalized,
        phonetic,
        audio_url,
        origin: first.origin.clone(),
        source: String::from("dictionaryapi"),
        source_url,
        license,
        created_at: now,
        updated_at: now,
    };

    Ok(LookupResult { entry, senses })
}
