use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::process::Command;
use scraper::{Html, Selector};

const DEFAULT_PROMPT: &str = "Answer concisely based on the user's selected text. \
 If the text is a question, provide a clear and direct answer with one or few sentences. \
 If the text is a word or phrase, give a brief definition or explanation. \
 If the text is a code snippet, explain its purpose in simple terms.";
const DEFAULT_OPENAI_MODEL: &str = "gpt-5-mini";
const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash";
const DEFAULT_WIKI_BASE_URL: &str = "http://localhost:8080";
const DEFAULT_WIKIPEDIA_MODEL: &str = "kiwix-wikipedia";

#[derive(Serialize)]
pub struct OverlayResponse {
    pub text: String,
    pub provider: String,
    pub model: String,
}

#[derive(Clone, Copy)]
enum Provider {
    OpenAi,
    Gemini,
    Wikipedia,
}

impl Provider {
    fn from_option(input: Option<String>) -> Self {
        if let Some(value) = input {
            if value.eq_ignore_ascii_case("gemini") {
                return Provider::Gemini;
            }

            if value.eq_ignore_ascii_case("wikipedia") {
                return Provider::Wikipedia;
            }
        }

        Provider::OpenAi
    }

    fn label(&self) -> &'static str {
        match self {
            Provider::OpenAi => "openai",
            Provider::Gemini => "gemini",
            Provider::Wikipedia => "wikipedia",
        }
    }
}

pub async fn generate(provider: Option<String>) -> Result<OverlayResponse, String> {
    let selection = fetch_selection()?;

    let provider = Provider::from_option(provider);
    let prompt = env::var("OVERLAY_PROMPT").unwrap_or_else(|_| DEFAULT_PROMPT.to_string());

    if selection.trim().is_empty() {
        let model = match provider {
            Provider::Gemini => {
                env::var("GEMINI_MODEL").unwrap_or_else(|_| DEFAULT_GEMINI_MODEL.to_string())
            }
            Provider::OpenAi => {
                env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_OPENAI_MODEL.to_string())
            }
            Provider::Wikipedia => DEFAULT_WIKIPEDIA_MODEL.to_string(),
        };

        return Ok(OverlayResponse {
            text: "(empty)".to_string(),
            provider: provider.label().to_string(),
            model,
        });
    }

    let client = Client::new();

    let (text, model) = match provider {
        Provider::OpenAi => {
            let api_key =
                env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY is not set".to_string())?;

            let model =
                env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_OPENAI_MODEL.to_string());

            let reply = query_openai(&client, &api_key, &model, &prompt, &selection).await?;
            (reply, model)
        }
        Provider::Gemini => {
            let api_key = env::var("GEMINI_API_KEY")
                .or_else(|_| env::var("GEMINI_API_TOKEN"))
                .map_err(|_| "GEMINI_API_KEY or GEMINI_API_TOKEN is not set".to_string())?;

            let model =
                env::var("GEMINI_MODEL").unwrap_or_else(|_| DEFAULT_GEMINI_MODEL.to_string());

            let reply = query_gemini(&client, &api_key, &model, &prompt, &selection).await?;
            (reply, model)
        }
        Provider::Wikipedia => {
            let model = DEFAULT_WIKIPEDIA_MODEL.to_string();
            let reply = query_wikipedia(&client, &selection).await?;
            (reply, model)
        }
    };

    Ok(OverlayResponse {
        text,
        provider: provider.label().to_string(),
        model,
    })
}

fn fetch_selection() -> Result<String, String> {
    if let Ok(Some(primary)) = run_wl_paste(true) {
        return Ok(primary);
    }

    if let Some(clipboard) = run_wl_paste(false)? {
        return Ok(clipboard);
    }

    Ok(String::new())
}

fn run_wl_paste(primary: bool) -> Result<Option<String>, String> {
    let mut cmd = Command::new("wl-paste");
    if primary {
        cmd.arg("--primary");
    }

    let output = cmd
        .output()
        .map_err(|err| format!("failed to run wl-paste: {err}"))?;

    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

async fn query_openai(
    client: &Client,
    api_key: &str,
    model: &str,
    prompt: &str,
    selection: &str,
) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": prompt },
            { "role": "user", "content": selection }
        ]
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("api request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("api http error: {e}"))?;

    let parsed: Value = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse api response: {e}"))?;

    if let Some(reply) = parsed
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|s| s.as_str())
    {
        let trimmed = reply.trim().to_string();
        if trimmed.is_empty() {
            Err("api response missing content".to_string())
        } else {
            Ok(trimmed)
        }
    } else {
        Err(format!("unexpected response shape: {}", parsed))
    }
}

async fn query_gemini(
    client: &Client,
    api_key_or_token: &str,
    model: &str,
    prompt: &str,
    selection: &str,
) -> Result<String, String> {
    let base_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let has_token = env::var("GEMINI_API_TOKEN").is_ok();
    let url = if has_token {
        base_url
    } else {
        format!("{}?key={}", base_url, api_key_or_token)
    };

    let body = serde_json::json!({
        "contents": [
            {
                "parts": [{ "text": selection }]
            }
        ],
        "systemInstruction": {
            "parts": [{ "text": prompt }]
        },
        "tools": [
        {
            "google_search": serde_json::json!({})
        }
        ]
    });

    let mut req = client.post(&url).json(&body);
    if has_token {
        req = req.bearer_auth(api_key_or_token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("api request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("api http error: {e}"))?;

    let parsed: Value = resp
        .json()
        .await
        .map_err(|e| format!("failed to parse api response: {e}"))?;

    if let Some(text) = parsed
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|cand| cand.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.get(0))
        .and_then(|p0| p0.get("text"))
        .and_then(|t| t.as_str())
    {
        let out = text.trim().to_string();
        if out.is_empty() {
            Err("api response missing content".to_string())
        } else {
            Ok(out)
        }
    } else if let Some(text) = parsed
        .get("response")
        .and_then(|r| r.get("text"))
        .and_then(|t| t.as_str())
    {
        Ok(text.trim().to_string())
    } else {
        Err(format!("unexpected response shape: {}", parsed))
    }
}

fn extract_content_path(search_html: &str) -> Option<String> {
    for needle in ["href=\"/content/", "href=\"/resources/"] {
        if let Some(idx) = search_html.find(needle) {
            let start = idx + "href=\"".len();
            if let Some(rest) = search_html.get(start..) {
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].trim_start_matches('/').to_string());
                }
            }
        }
    }
    None
}

pub async fn query_wikipedia(client: &Client, selection: &str) -> Result<String, String> {
    let base = DEFAULT_WIKI_BASE_URL.trim_end_matches('/');

    let search_html = client
        .get(format!("{base}/search"))
        .query(&[("pattern", selection)])
        .send()
        .await
        .map_err(|e| format!("search request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("search http error: {e}"))?
        .text()
        .await
        .map_err(|e| format!("failed to read search response: {e}"))?;

    let content_path = extract_content_path(&search_html)
        .ok_or_else(|| "no wikipedia matches found in search results".to_string())?;

    let article_resp = client
        .get(format!("{base}/{content_path}"))
        .send()
        .await
        .map_err(|e| format!("article request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("article http error: {e}"))?;

    let body = article_resp
        .text()
        .await
        .map_err(|e| format!("failed to read article body: {e}"))?;

    let text = html_to_text(&body);
    if text.is_empty() {
        Err("article content was empty".to_string())
    } else {
        Ok(text)
    }
}

fn html_to_text(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut chunks = Vec::new();

    if let Ok(sel) = Selector::parse("h1") {
        if let Some(node) = document.select(&sel).next() {
            let text = node.text().collect::<Vec<_>>().join(" ");
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !text.is_empty() {
                chunks.push(text);
            }
        }
    }

    if let Ok(sel) = Selector::parse("p") {
        for node in document.select(&sel).take(4) {
            let text = node.text().collect::<Vec<_>>().join(" ");
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !text.is_empty() {
                chunks.push(text);
            }
        }
    }

    if chunks.is_empty() {
        let fallback = document.root_element().text().collect::<Vec<_>>().join(" ");
        return fallback
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
    }

    chunks.join("\n\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{extract_content_path, html_to_text};

    #[test]
    fn parses_first_content_link_from_search_html() {
        let sample = r#"
        <html>
          <body>
            <div class="results">
              <ul>
                <li>
                  <a href="/content/wikipedia_en_100_mini_2025-10/Bird">Bird</a>
                </li>
                <li>
                  <a href="/content/wikipedia_en_100_mini_2025-10/Other">Other</a>
                </li>
              </ul>
            </div>
          </body>
        </html>
        "#;

        let path = extract_content_path(sample).expect("expected first link");
        assert_eq!(path, "content/wikipedia_en_100_mini_2025-10/Bird");
    }

    #[test]
    fn returns_none_when_no_links_found() {
        let sample = "<html><body><p>No results</p></body></html>";
        assert!(extract_content_path(sample).is_none());
    }

    #[test]
    fn strips_html_and_normalizes_whitespace() {
        let sample = r#"
        <html>
          <body>
            <h1>Bird</h1>
            <p>Birds are warm-blooded <b>animals</b>.</p>
          </body>
        </html>
        "#;

        let text = html_to_text(sample);
        assert!(text.contains("Bird"));
        assert!(text.contains("Birds are warm-blooded animals."));
        assert!(!text.contains('<'));
    }
}
