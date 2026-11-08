use reqwest::Client;
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const BASE: &str = "https://autorace.jp";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";
/// Minimum interval between requests (human-like pace).
const MIN_INTERVAL: Duration = Duration::from_millis(1200);

#[derive(Default)]
struct Session {
    /// "name=value; name=value" cookie header value
    cookie: String,
    /// Decoded XSRF-TOKEN value for the X-XSRF-TOKEN header
    xsrf: String,
}

pub struct ApiClient {
    client: Client,
    session: Mutex<Session>,
    last_request: Mutex<Instant>,
}

#[derive(Debug)]
pub enum ApiError {
    Http(String),
    SessionExpired,
    Api { code: String, message: String },
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Http(m) => write!(f, "HTTP error: {m}"),
            ApiError::SessionExpired => write!(f, "session expired"),
            ApiError::Api { code, message } => write!(f, "API error {code}: {message}"),
        }
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn extract_csrf(html: &str) -> Option<String> {
    let marker = "csrf-token\" content=\"";
    let start = html.find(marker)? + marker.len();
    let end = html[start..].find('"')? + start;
    Some(html[start..end].to_string())
}

fn cookie_pair(set_cookie: &str) -> Option<(String, String)> {
    let pair = set_cookie.split(';').next()?;
    let (name, value) = pair.split_once('=')?;
    Some((name.trim().to_string(), value.trim().to_string()))
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(20))
            .build()
            .expect("reqwest client");
        Self {
            client,
            session: Mutex::new(Session::default()),
            last_request: Mutex::new(Instant::now() - MIN_INTERVAL),
        }
    }

    async fn throttle(&self) {
        let wait = {
            let mut last = self.last_request.lock().unwrap();
            let elapsed = last.elapsed();
            if elapsed >= MIN_INTERVAL {
                *last = Instant::now();
                return;
            }
            let w = MIN_INTERVAL - elapsed;
            *last = Instant::now() + w;
            w
        };
        sleep(wait).await;
    }

    async fn init_session(&self) -> Result<(), ApiError> {
        self.throttle().await;
        let resp = self
            .client
            .get(format!("{BASE}/race_info/"))
            .send()
            .await
            .map_err(|e| ApiError::Http(e.to_string()))?;
        let headers = resp.headers().clone();
        let html = resp.text().await.map_err(|e| ApiError::Http(e.to_string()))?;

        let mut session = Session::default();
        for value in headers.get_all("set-cookie") {
            if let Ok(sc) = value.to_str() {
                if let Some((name, val)) = cookie_pair(sc) {
                    if name == "XSRF-TOKEN" || name == "race_info_session" {
                        if !session.cookie.is_empty() {
                            session.cookie.push_str("; ");
                        }
                        session.cookie.push_str(&format!("{name}={val}"));
                        if name == "XSRF-TOKEN" {
                            session.xsrf = percent_decode(&val);
                        }
                    }
                }
            }
        }
        // CSRF token meta is not strictly needed for X-XSRF-TOKEN flow, but
        // keep session only if we got the cookies.
        let _ = extract_csrf(&html);
        if session.cookie.is_empty() {
            return Err(ApiError::Http("no session cookies".into()));
        }
        *self.session.lock().unwrap() = session;
        Ok(())
    }

    async fn request(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value, ApiError> {
        if self.session.lock().unwrap().cookie.is_empty() {
            self.init_session().await?;
        }
        for attempt in 0..2 {
            match self.send(method, path, body).await {
                Ok(v) => return Ok(v),
                Err(ApiError::SessionExpired) if attempt == 0 => {
                    *self.session.lock().unwrap() = Session::default();
                    self.init_session().await?;
                }
                Err(e) => return Err(e),
            }
        }
        Err(ApiError::SessionExpired)
    }

    async fn send(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value, ApiError> {
        self.throttle().await;
        let (cookie, xsrf) = {
            let s = self.session.lock().unwrap();
            (s.cookie.clone(), s.xsrf.clone())
        };
        let url = format!("{BASE}{path}");
        let mut req = match method {
            "GET" => self.client.get(&url),
            _ => self.client.post(&url),
        };
        req = req
            .header("Cookie", cookie)
            .header("X-Requested-With", "XMLHttpRequest");
        if method != "GET" {
            req = req
                .header("Content-Type", "application/json")
                .header("X-XSRF-TOKEN", xsrf)
                .json(body.unwrap_or(&Value::Null));
        }

        let resp = req.send().await.map_err(|e| ApiError::Http(e.to_string()))?;
        let status = resp.status();
        let is_html = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false);
        let text = resp.text().await.map_err(|e| ApiError::Http(e.to_string()))?;

        if status.as_u16() == 419 || text.contains("Page Expired") {
            return Err(ApiError::SessionExpired);
        }
        if !status.is_success() {
            return Err(ApiError::Http(format!("status {status}")));
        }
        if is_html {
            return Err(ApiError::SessionExpired);
        }
        serde_json::from_str(&text).map_err(|e| ApiError::Http(format!("invalid json: {e}")))
    }

    /// GET endpoint returning the API envelope; returns `body` on Success.
    pub async fn get(&self, path: &str) -> Result<Value, ApiError> {
        let env = self.request("GET", path, None).await?;
        envelope_body(env)
    }

    /// POST endpoint returning the API envelope; returns `body` on Success.
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value, ApiError> {
        let env = self.request("POST", path, Some(body)).await?;
        envelope_body(env)
    }
}

pub fn envelope_body(env: Value) -> Result<Value, ApiError> {
    let result = env.get("result").and_then(|v| v.as_str()).unwrap_or("");
    if result == "Success" {
        Ok(env.get("body").cloned().unwrap_or(Value::Null))
    } else {
        let err = env
            .get("errors")
            .and_then(|e| e.as_array())
            .and_then(|a| a.first())
            .cloned()
            .unwrap_or(Value::Null);
        Err(ApiError::Api {
            code: err.get("code").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
            message: err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    #[ignore = "hits the live autorace.jp API"]
    async fn smoke_session_and_post() {
        let api = ApiClient::new();
        let today = api.get("/race_info/XML/Hold/Today").await.unwrap();
        assert!(today.get("today").is_some());

        let body = api
            .post(
                "/race_info/OtherRaceInfo",
                &json!({"placeCode": 2, "raceDate": "2026-11-08", "raceNo": 1}),
            )
            .await
            .unwrap();
        assert_eq!(body.get("placeKey").and_then(|v| v.as_str()), Some("kawaguchi"));
    }
}
