use chrono::Utc;
use lexic_dict::{DictError, DictionaryClient};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn lookup_parses_dictionary_payload() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/entries/en/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "word": "hello",
            "phonetic": "/həˈləʊ/",
            "phonetics": [{
                "text": "/həˈləʊ/",
                "audio": "https://example.com/hello.mp3"
            }],
            "origin": "early 19th century",
            "meanings": [{
                "partOfSpeech": "exclamation",
                "definitions": [{
                    "definition": "used as a greeting or to begin a phone conversation",
                    "example": "hello there",
                    "synonyms": ["hi"],
                    "antonyms": []
                }],
                "synonyms": ["hi", "hey"],
                "antonyms": []
            }],
            "license": {"name": "CC BY-SA 3.0", "url": "https://creativecommons.org/licenses/by-sa/3.0/"},
            "sourceUrls": ["https://en.wiktionary.org/wiki/hello"]
        }])))
        .mount(&server)
        .await;

    let client = DictionaryClient::new(format!("{}/api/v2/entries/en", server.uri()));
    let result = client.lookup("hello", Utc::now()).await.unwrap();

    assert_eq!(result.entry.word, "hello");
    assert_eq!(result.entry.normalized_word, "hello");
    assert_eq!(result.entry.phonetic.as_deref(), Some("/həˈləʊ/"));
    assert_eq!(
        result.entry.audio_url.as_deref(),
        Some("https://example.com/hello.mp3")
    );
    assert_eq!(result.entry.origin.as_deref(), Some("early 19th century"));
    assert_eq!(result.entry.license.as_deref(), Some("CC BY-SA 3.0"));
    assert_eq!(
        result.entry.source_url.as_deref(),
        Some("https://en.wiktionary.org/wiki/hello")
    );
    assert_eq!(result.senses.len(), 1);
    assert_eq!(
        result.senses[0].part_of_speech.as_deref(),
        Some("exclamation")
    );
    assert_eq!(result.senses[0].synonyms, vec!["hi".to_string()]);
}

#[tokio::test]
async fn lookup_reports_missing_word() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/entries/en/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "title": "No Definitions Found",
            "message": "Sorry pal, we couldn't find definitions for the word you were looking for.",
            "resolution": "You can try the search again"
        })))
        .mount(&server)
        .await;

    let client = DictionaryClient::new(format!("{}/api/v2/entries/en", server.uri()));
    let error = client.lookup("missing", Utc::now()).await.unwrap_err();

    assert_eq!(error, DictError::NotFound);
}

#[tokio::test]
async fn lookup_reports_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/entries/en/limited"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let client = DictionaryClient::new(format!("{}/api/v2/entries/en", server.uri()));
    let error = client.lookup("limited", Utc::now()).await.unwrap_err();

    assert_eq!(error, DictError::RateLimited);
}

#[tokio::test]
async fn lookup_normalizes_before_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/entries/en/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "word": "hello",
            "meanings": [{
                "partOfSpeech": "noun",
                "definitions": [{"definition": "a greeting"}]
            }]
        }])))
        .mount(&server)
        .await;

    let client = DictionaryClient::new(format!("{}/api/v2/entries/en", server.uri()));
    let result = client.lookup("  HeLLo  ", Utc::now()).await.unwrap();

    assert_eq!(result.entry.normalized_word, "hello");
}
