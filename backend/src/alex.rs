use crate::{
    config::AlexConfig,
    error::AppError,
    models::product::{Branch, Candidate, SearchInput, SearchMode, satang, valid_id},
    state::AppState,
};
use reqwest::Url;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub struct Session {
    token: String,
    pub branches: Vec<Branch>,
}

pub fn config(state: &AppState) -> Result<&AlexConfig, AppError> {
    state.config.alex.as_ref().ok_or(AppError::AlexUnavailable)
}

async fn response_json(mut response: reqwest::Response) -> Result<Value, AppError> {
    if !response.status().is_success() {
        return Err(AppError::AlexFailed);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| AppError::AlexFailed)? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err(AppError::AlexFailed);
        }
        bytes.extend(chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| AppError::AlexFailed)
}

pub async fn login(state: &AppState) -> Result<Session, AppError> {
    let cfg = config(state)?;
    let response = state
        .http
        .post(format!("{}/api/user/login", cfg.api_url))
        .json(&json!({"username":cfg.username,"password":cfg.password}))
        .send()
        .await
        .map_err(|_| AppError::AlexFailed)?;
    let value = response_json(response).await?;
    if value["shopInfoId"].as_str() != Some(&cfg.shop_info_id) {
        return Err(AppError::AlexScope);
    }
    let token = value["token"]
        .as_str()
        .filter(|v| !v.is_empty() && v.len() <= 16384)
        .ok_or(AppError::AlexFailed)?
        .to_string();
    let mut branches = Vec::new();
    let mut seen = BTreeSet::new();
    for branch in value["userBranch"].as_array().ok_or(AppError::AlexFailed)? {
        let id = branch["branchId"]
            .as_str()
            .filter(|v| valid_id(v))
            .ok_or(AppError::AlexFailed)?;
        let name = branch["branchName"]
            .as_str()
            .filter(|v| !v.trim().is_empty() && v.len() <= 1000)
            .ok_or(AppError::AlexFailed)?;
        if (cfg.branch_ids.is_empty() || cfg.branch_ids.iter().any(|allowed| allowed == id))
            && seen.insert(id.to_string())
        {
            branches.push(Branch {
                id: id.into(),
                name: name.into(),
            });
        }
    }
    if branches.is_empty() {
        return Err(AppError::AlexScope);
    }
    Ok(Session { token, branches })
}

pub fn check_branch(session: &Session, branch: &str) -> Result<(), AppError> {
    if session.branches.iter().any(|allowed| allowed.id == branch) {
        Ok(())
    } else {
        Err(AppError::AlexScope)
    }
}

pub fn product_code_from_url(cfg: &AlexConfig, value: &str) -> Result<String, AppError> {
    let prefix = cfg.product_url_prefix.as_ref().ok_or(AppError::Validation(
        "Product URL mapping is not configured",
    ))?;
    let prefix = Url::parse(prefix).map_err(|_| AppError::AlexUnavailable)?;
    let url = Url::parse(value).map_err(|_| AppError::Validation("Invalid product URL"))?;
    if prefix.scheme() != "https"
        || url.origin() != prefix.origin()
        || !url.path().starts_with(prefix.path())
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::Validation(
            "Product URL is outside the allowed prefix",
        ));
    }
    let codes: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "code" || key == "productCode")
        .map(|(_, v)| v.into_owned())
        .collect();
    let code = if codes.len() == 1 {
        codes[0].clone()
    } else if codes.is_empty() && prefix.path().ends_with('/') {
        url.path()
            .strip_prefix(prefix.path())
            .unwrap_or("")
            .to_string()
    } else {
        return Err(AppError::Validation(
            "Product URL must identify exactly one code",
        ));
    };
    if !valid_id(&code) {
        return Err(AppError::Validation("Product URL has no valid code"));
    }
    Ok(code)
}

pub async fn search_page(
    state: &AppState,
    session: &Session,
    input: &SearchInput,
) -> Result<(Vec<Candidate>, bool), AppError> {
    check_branch(session, &input.branch_id)?;
    let cfg = config(state)?;
    let keyword = match input.mode {
        SearchMode::ProductUrl => product_code_from_url(cfg, input.query.as_deref().unwrap_or(""))?,
        _ => input.query.clone().unwrap_or_default().trim().to_string(),
    };
    let response = state.http.post(format!("{}/api/v2/GetProductItems", cfg.api_url))
        .bearer_auth(&session.token).json(&json!({"filter":true,"pageNumber":input.page,"pageSize":15,
            "showNotAvailable":false,"branchId":input.branch_id,"keyword":keyword,
            "color":filter_value(&input.color),"size":filter_value(&input.size),
            "category":input.category.as_deref().unwrap_or(""),"texture":input.texture.as_deref().unwrap_or(""),"simpleSearch":false}))
        .send().await.map_err(|_| AppError::AlexFailed)?;
    let response = response_json(response).await?;
    let items = response["itemList"]
        .as_array()
        .filter(|items| items.len() <= 15)
        .ok_or(AppError::AlexFailed)?;
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for item in items {
        if item
            .get("shopInfoId")
            .is_some_and(|id| id.as_str() != Some(&cfg.shop_info_id))
            || item
                .get("branchId")
                .is_some_and(|id| id.as_str() != Some(&input.branch_id))
        {
            return Err(AppError::AlexScope);
        }
        let text = |key: &str| {
            item[key]
                .as_str()
                .filter(|v| !v.trim().is_empty() && v.len() <= 2000)
                .map(String::from)
                .ok_or(AppError::AlexFailed)
        };
        let optional = |key: &str| -> Result<Option<String>, AppError> {
            if item[key].is_null() || item[key] == "" {
                Ok(None)
            } else {
                text(key).map(Some)
            }
        };
        let id = text("productItemId")?;
        if !valid_id(&id) || !seen.insert(id.clone()) {
            return Err(AppError::AlexFailed);
        }
        let code = text("code")?;
        let color = optional("color")?;
        let size = optional("size")?;
        if matches!(input.mode, SearchMode::Code | SearchMode::ProductUrl)
            && !code.eq_ignore_ascii_case(&keyword)
            || !matches_filter(&input.color, &color)
            || !matches_filter(&input.size, &size)
            || item["isActive"] == false
        {
            continue;
        }
        result.push(Candidate {
            product_item_id: id,
            code,
            name: text("name")?,
            color,
            size,
            rental_fee_satang: satang(&item["rentalPrice"])?,
            deposit_satang: satang(&item["bail"])?,
            image_url: item["mainImage"]
                .as_str()
                .filter(|url| !url.trim().is_empty())
                .and_then(|url| allowed_image(cfg, url).ok())
                .map(|url| url.to_string()),
            score: None,
        });
    }
    let more = response["itemNumber"]
        .as_u64()
        .map_or(items.len() == 15, |count| {
            u64::from(input.page) * 15 < count
        });
    Ok((result, more))
}

fn filter_value(value: &Option<String>) -> String {
    value
        .as_ref()
        .map(|v| v.trim_matches('|'))
        .filter(|v| !v.is_empty())
        .map(|v| format!("{v}|"))
        .unwrap_or_default()
}

fn matches_filter(filter: &Option<String>, value: &Option<String>) -> bool {
    let choices: Vec<_> = filter
        .as_deref()
        .unwrap_or("")
        .split('|')
        .filter(|v| !v.is_empty())
        .collect();
    choices.is_empty()
        || value
            .as_ref()
            .is_some_and(|v| choices.iter().any(|choice| v.eq_ignore_ascii_case(choice)))
}

fn allowed_image(cfg: &AlexConfig, value: &str) -> Result<Url, AppError> {
    let api = Url::parse(&cfg.api_url).map_err(|_| AppError::AlexUnavailable)?;
    let url = api.join(value).map_err(|_| AppError::AlexFailed)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || (url.scheme() != "https" && url.origin() != api.origin())
        || (url.origin() != api.origin()
            && !cfg.image_origins.iter().any(|origin| {
                Url::parse(origin).is_ok_and(|allowed| allowed.origin() == url.origin())
            }))
    {
        return Err(AppError::AlexScope);
    }
    Ok(url)
}

pub async fn image(
    state: &AppState,
    session: &Session,
    value: &str,
) -> Result<(String, Vec<u8>), AppError> {
    let cfg = config(state)?;
    let url = allowed_image(cfg, value)?;
    let mut request = state.http.get(url.clone());
    if Url::parse(&cfg.api_url).is_ok_and(|api| api.origin() == url.origin()) {
        request = request.bearer_auth(&session.token);
    }
    let mut response = request.send().await.map_err(|_| AppError::AlexFailed)?;
    if !response.status().is_success() {
        return Err(AppError::AlexFailed);
    }
    let mime = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .to_string();
    if !["image/jpeg", "image/png", "image/webp", "image/gif"].contains(&mime.as_str()) {
        return Err(AppError::AlexFailed);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| AppError::AlexFailed)? {
        if bytes.len() + chunk.len() > 8 * 1024 * 1024 {
            return Err(AppError::AlexFailed);
        }
        bytes.extend(chunk);
    }
    Ok((mime, bytes))
}
