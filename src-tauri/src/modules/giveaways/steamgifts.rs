use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::modules::http_curl;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointsInfo {
    pub points: u32,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GiveawayItem {
    pub name: String,
    pub code: String,
    pub cost: u32,
    pub image_url: String,
    pub is_entered: bool,
    pub ends_at: Option<i64>,
    pub ends_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WonGiveaway {
    pub name: String,
    pub code: String,
    pub image_url: String,
    pub source: String,
    pub url: String,
}

pub struct SteamgiftsService {
    cookie_header: String,
}

/// Cookie header uses only `PHPSESSID=<value>` (no extra cookie names).
pub fn normalize_cookie(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    for part in trimmed.split(';') {
        let part = part.trim();
        if part.starts_with("PHPSESSID=") {
            return part.to_string();
        }
    }

    if let Some(value) = trimmed.strip_prefix("PHPSESSID=") {
        return format!("PHPSESSID={}", value.trim());
    }

    format!("PHPSESSID={trimmed}")
}

impl SteamgiftsService {
    pub fn new(cookie: &str) -> Self {
        Self {
            cookie_header: normalize_cookie(cookie),
        }
    }

    fn headers(&self) -> Vec<(&str, &str)> {
        vec![
            ("Cookie", &self.cookie_header),
            ("Referer", "https://www.steamgifts.com/"),
            (
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        ]
    }

    fn fetch(&self, url: &str) -> Result<String, String> {
        if self.cookie_header.is_empty() {
            return Err("PHPSESSID cookie is required. Set it in Settings.".into());
        }

        let response = http_curl::request(url, "GET", &self.headers(), None)?;

        if response.status == 403 {
            return Err(
                "HTTP 403 Forbidden — invalid or expired PHPSESSID. Paste a fresh PHPSESSID from steamgifts.com cookies in Settings."
                    .into(),
            );
        }

        if response.status != 200 {
            return Err(format!("HTTP {}", response.status));
        }

        if response.body.contains("Just a moment") || response.body.contains("cf-browser-verification")
        {
            return Err(
                "Cloudflare blocked the request. Paste a fresh PHPSESSID from your browser."
                    .into(),
            );
        }

        Ok(response.body)
    }

    pub async fn fetch_points(&self) -> Result<PointsInfo, String> {
        let html = self.fetch("https://www.steamgifts.com/")?;
        Ok(parse_points(&html))
    }

    pub async fn fetch_search_page(
        &self,
        page: u32,
    ) -> Result<(u32, String, Vec<GiveawayItem>), String> {
        let html = self.fetch(&format!(
            "https://www.steamgifts.com/giveaways/search?page={page}"
        ))?;
        Ok(parse_search_page(&html))
    }

    pub async fn fetch_won(&self) -> Result<Vec<WonGiveaway>, String> {
        let html = self.fetch("https://www.steamgifts.com/giveaways/won")?;
        Ok(parse_won_page(&html))
    }

    pub async fn claim_won(&self, code: &str) -> Result<(), String> {
        let html = self.fetch(&format!("https://www.steamgifts.com/giveaway/{code}/"))?;
        let xsrf_sel = Selector::parse("[name='xsrf_token']").unwrap();
        let document = Html::parse_document(&html);
        let xsrf = document
            .select(&xsrf_sel)
            .next()
            .and_then(|el| el.value().attr("value"))
            .ok_or("Could not read XSRF token for gift claim.")?;

        let payload = format!("xsrf_token={xsrf}&do=redeem&code={code}");
        let mut headers = self.headers();
        headers.push(("Origin", "https://www.steamgifts.com"));
        headers.push(("Content-Type", "application/x-www-form-urlencoded"));

        let response = http_curl::request(
            "https://www.steamgifts.com/ajax.php",
            "POST",
            &headers,
            Some(&payload),
        )?;

        if response.status == 403 {
            return Err("HTTP 403 — refresh your PHPSESSID cookie.".into());
        }

        if response.status != 200 {
            return Err(format!("HTTP {}", response.status));
        }

        if response.body.contains("\"type\":\"success\"") || response.body.contains("success") {
            Ok(())
        } else {
            Err(format!(
                "Claim failed: {}",
                response.body.chars().take(120).collect::<String>()
            ))
        }
    }

    pub async fn enter_giveaway(&self, code: &str, xsrf_token: &str) -> Result<(), String> {
        let payload = format!("xsrf_token={xsrf_token}&do=entry_insert&code={code}");
        let mut headers = self.headers();
        headers.push(("Origin", "https://www.steamgifts.com"));

        let response = http_curl::request(
            "https://www.steamgifts.com/ajax.php",
            "POST",
            &headers,
            Some(&payload),
        )?;

        if response.status == 403 {
            return Err("HTTP 403 — refresh your PHPSESSID cookie in Settings.".into());
        }

        if response.status != 200 {
            return Err(format!("HTTP {}", response.status));
        }

        Ok(())
    }
}

fn digits_only(text: &str) -> u32 {
    text.chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

pub fn parse_points(html: &str) -> PointsInfo {
    let document = Html::parse_document(html);
    let points_sel = Selector::parse(".nav__points").unwrap();
    let user_sel = Selector::parse(".nav__user-name").unwrap();

    let points = document
        .select(&points_sel)
        .next()
        .map(|el| digits_only(&el.text().collect::<String>()))
        .unwrap_or(0);

    let username = document
        .select(&user_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    PointsInfo { points, username }
}

pub fn parse_search_page(html: &str) -> (u32, String, Vec<GiveawayItem>) {
    let document = Html::parse_document(html);
    let points_sel = Selector::parse(".nav__points").unwrap();
    let xsrf_sel = Selector::parse("[name='xsrf_token']").unwrap();
    let row_sel = Selector::parse(".giveaway__row-inner-wrap").unwrap();

    let points = document
        .select(&points_sel)
        .next()
        .map(|el| digits_only(&el.text().collect::<String>()))
        .unwrap_or(0);

    let xsrf = document
        .select(&xsrf_sel)
        .next()
        .and_then(|el| el.value().attr("value"))
        .unwrap_or("")
        .to_string();

    let giveaways = document
        .select(&row_sel)
        .map(parse_giveaway_row)
        .collect();

    (points, xsrf, giveaways)
}

fn parse_giveaway_row(element: scraper::ElementRef<'_>) -> GiveawayItem {
    let cost_sel = Selector::parse(".giveaway__heading__thin").unwrap();
    let name_sel = Selector::parse(".giveaway__heading__name").unwrap();

    let cost = element
        .select(&cost_sel)
        .last()
        .map(|el| digits_only(&el.text().collect::<String>()))
        .unwrap_or(0);

    let name_el = element.select(&name_sel).next();
    let name = name_el
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown".into());

    let href = name_el
        .and_then(|el| el.value().attr("href"))
        .unwrap_or("");
    let code = href.split('/').nth(2).unwrap_or("").to_string();

    let app_id_re = Regex::new(r"/app/(\d+)").unwrap();
    let image_url = element
        .select(&Selector::parse("a.giveaway__icon[href*='steampowered.com/app/']").unwrap())
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(|href| app_id_re.captures(href))
        .map(|caps| {
            format!(
                "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/capsule_184x69.jpg",
                &caps[1]
            )
        })
        .unwrap_or_default();

    let (ends_at, ends_label) = parse_giveaway_end(&element);
    let is_entered = element
        .value()
        .attr("class")
        .map(|c| c.contains("is-faded"))
        .unwrap_or(false);

    GiveawayItem {
        name,
        code,
        cost,
        image_url,
        is_entered,
        ends_at,
        ends_label,
    }
}

fn parse_giveaway_end(element: &scraper::ElementRef<'_>) -> (Option<i64>, String) {
    let ts_sel = Selector::parse("[data-timestamp]").unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut best_ts: Option<i64> = None;
    let mut best_label = String::new();

    for el in element.select(&ts_sel) {
        let raw = el.value().attr("data-timestamp").unwrap_or("");
        if let Ok(ts) = raw.parse::<i64>() {
            if ts > now && (best_ts.is_none() || Some(ts) < best_ts) {
                best_ts = Some(ts);
                best_label = el.text().collect::<String>().trim().to_string();
            }
        }
    }

    (best_ts, best_label)
}

pub fn parse_won_page(html: &str) -> Vec<WonGiveaway> {
    let document = Html::parse_document(html);
    let row_sel = Selector::parse(".table__row-inner-wrap").unwrap();
    let heading_sel = Selector::parse(".table__column__heading").unwrap();

    document
        .select(&row_sel)
        .filter_map(|row| {
            if row
                .select(&Selector::parse("form input[name='do'][value='entry_delete']").unwrap())
                .next()
                .is_some()
            {
                return None;
            }

            let heading = row.select(&heading_sel).next()?;
            let href = heading.value().attr("href").unwrap_or("");
            let name = heading.text().collect::<String>().trim().to_string();
            let code = href.split('/').nth(2).unwrap_or("").to_string();
            if code.is_empty() {
                return None;
            }

            let url = if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://www.steamgifts.com{href}")
            };

            Some(WonGiveaway {
                name,
                code,
                image_url: String::new(),
                source: "steamgifts".into(),
                url,
            })
        })
        .collect()
}

pub fn within_end_window(giveaway: &GiveawayItem, max_hours: u32) -> bool {
    if max_hours == 0 {
        return true;
    }
    let Some(ends_at) = giveaway.ends_at else {
        return false;
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    ends_at - now <= max_hours as i64 * 3600
}
