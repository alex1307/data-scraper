use log::{error, info, warn};
use std::borrow::Cow;
// slack/SlackNotifier.rs
use serde_json::json;
use std::process::Command;
use std::time::Duration;

/// Remove control chars that may break Slack JSON (keep newlines and tabs)
fn sanitize_text(input: &str) -> String {
    input
        .chars()
        .filter(|&c| match c {
            '\n' | '\r' | '\t' => true,
            c if (c as u32) >= 0x20 => true,
            _ => false,
        })
        .collect()
}

fn env_trim(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn mask_webhook(url: &str) -> String {
    // Show only last 6 chars of the full path for debugging, keep host
    // e.g., https://hooks.slack.com/services/XXXX/XXXX/......ABCDEF -> ...ABCDEF
    let tail: String = url
        .chars()
        .rev()
        .take(6)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("***{}", tail)
}

/// Render a url=... line for Slack (plain text is clickable in Slack)
fn line_url_from_opt<'a>(u: Option<&'a str>) -> String {
    match u {
        Some(s) if !s.trim().is_empty() => format!("url={}", s),
        _ => "url=-".to_string(),
    }
}

#[derive(Clone)]
pub struct SlackNotifier {
    /// ако подадеш конкретен webhook, ползва него; иначе чете от env мапинга
    pub default_webhooks: std::collections::HashMap<Channel, String>,
    /// ако зададеш, ще се вика външен скрипт вместо HTTP
    pub external_cmd: Option<String>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Channel {
    Test,
    JobStatus,
    Error,
    AllServices,
}

impl SlackNotifier {
    pub fn from_env() -> Self {
        let mut map = std::collections::HashMap::new();
        if let Some(v) = env_trim("SLACK_TEST_WEBHOOK") {
            map.insert(Channel::Test, v);
        }
        if let Some(v) = env_trim("SLACK_JOB_STATUS_URL") {
            map.insert(Channel::JobStatus, v);
        }
        if let Some(v) = env_trim("SLACK_ERROR_URL") {
            map.insert(Channel::Error, v);
        }
        if let Some(v) = env_trim("SLACK_ALL_SERVICES_URL") {
            map.insert(Channel::AllServices, v);
        }

        Self {
            default_webhooks: map,
            external_cmd: env_trim("SLACK_NOTIFY_CMD"),
        }
    }

    pub async fn notify_text(
        &self,
        channel: Channel,
        text: &str,
        override_webhook: Option<String>,
    ) {
        let mut webhook = override_webhook.or_else(|| self.default_webhooks.get(&channel).cloned());
        // Fallback: if JobStatus is missing, try Test channel
        if webhook.is_none() && matches!(channel, Channel::JobStatus) {
            if let Some(test) = self.default_webhooks.get(&Channel::Test) {
                webhook = Some(test.clone());
                info!("JobStatus webhook missing; falling back to Test channel");
            }
        }
        if webhook.is_none() {
            // Optional global override: if SLACK_FORCE_TEST is set, fallback to Test always
            if std::env::var("SLACK_FORCE_TEST").ok().as_deref() == Some("1") {
                if let Some(test) = self.default_webhooks.get(&Channel::Test) {
                    webhook = Some(test.clone());
                    info!("Force override: sending to Test channel");
                }
            }
        }

        let Some(webhook) = webhook else {
            info!(
                "No webhook for {:?}; skipping notify. msg={}",
                channel, text
            );
            return;
        };

        info!("Webhook selected {:?}: {}", channel, mask_webhook(&webhook));

        let safe_text = sanitize_text(text);

        if let Some(cmd) = &self.external_cmd {
            let _ = Command::new(cmd).arg(&webhook).arg(&safe_text).status();
            return;
        }

        let client = reqwest::Client::new();
        let payload = json!({ "text": safe_text });
        info!("Slack notify to {:?}", channel);
        match client
            .post(webhook)
            .json(&payload)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) => {
                let code = resp.status();
                if code.is_success() {
                    info!("Slack delivered ({})", code);
                }
                if !code.is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    warn!("Slack responded with status {}: {}", code, body);
                }
            }
            Err(e) => {
                error!("Slack request failed: {}", e);
            }
        }
    }

    /// Guess a country/market flag (emoji) from a `source` like `autouncle.ro`, `mobile.bg`, etc.
    fn market_flag(source: &str) -> &'static str {
        let s = source.to_ascii_lowercase();
        // Common ccTLDs we use; default to globe
        if s.contains(".bg") {
            return "🇧🇬";
        }
        if s.contains(".ro") {
            return "🇷🇴";
        }
        if s.contains(".ch") {
            return "🇨🇭";
        }
        if s.contains(".de") {
            return "🇩🇪";
        }
        if s.contains(".it") {
            return "🇮🇹";
        }
        if s.contains(".fr") {
            return "🇫🇷";
        }
        if s.contains(".nl") {
            return "🇳🇱";
        }
        if s.contains(".at") {
            return "🇦🇹";
        }
        if s.contains(".pl") {
            return "🇵🇱";
        }
        if s.contains(".cz") {
            return "🇨🇿";
        }
        if s.contains(".sk") {
            return "🇸🇰";
        }
        if s.contains(".es") {
            return "🇪🇸";
        }
        if s.contains(".dk") {
            return "🇩🇰";
        }
        if s.contains(".se") {
            return "🇸🇪";
        }
        if s.contains(".no") {
            return "🇳🇴";
        }
        if s.contains(".uk") || s.contains(".co.uk") {
            return "🇬🇧";
        }
        "🌍"
    }

    /// Same as `notify_job_finished` but also includes a `url` line.
    pub async fn notify_job_finished_with_url(
        &self,
        source: &str,
        cfg_path: Option<&str>,
        found: i32,
        duration_s: f64,
        host: &str,
        url: Option<&str>,
    ) {
        let flag = Self::market_flag(source);
        let mut msg = format!("✅ {} finished {}\n", source, flag);
        if let Some(cfg) = cfg_path {
            if !cfg.is_empty() {
                msg.push_str(&format!("cfg={}\n", cfg));
            }
        }
        msg.push_str(&format!(
            "found={} duration={:.1}s host={}\n",
            found, duration_s, host
        ));
        // add URL line at the end (plain clickable)
        msg.push_str(&line_url_from_opt(url));
        self.notify_text(Channel::JobStatus, &msg, None).await;
    }

    /// Convenience: send a formatted "job finished" message to JobStatus with a country flag.
    /// Example message:
    /// ✅ autouncle.ro finished 🇷🇴\n
    /// cfg=config/autouncle/ro/premium.yml\n
    /// found=51 duration=8.4s host=matkat-srv
    pub async fn notify_job_finished(
        &self,
        source: &str,
        cfg_path: Option<&str>,
        found: i32,
        duration_s: f64,
        host: &str,
    ) {
        let _ = self
            .notify_job_finished_with_url(source, cfg_path, found, duration_s, host, None)
            .await;
    }
}
