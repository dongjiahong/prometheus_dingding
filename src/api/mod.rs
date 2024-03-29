use std::collections::HashMap;

use anyhow::Result;
use axum::{response::IntoResponse, Extension, Json};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PrometeusPost {
    version: String,
    #[serde(rename = "groupKey")]
    group_key: String,
    status: String,
    receiver: String,
    #[serde(rename = "groupLabels")]
    group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels")]
    common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations")]
    common_annotations: HashMap<String, String>,
    #[serde(rename = "externalURL")]
    external_url: String,
    alerts: Vec<Alert>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Alert {
    status: String,
    labels: HashMap<String, String>,
    annotations: HashMap<String, String>,
    #[serde(rename = "startsAt")]
    starts_at: String,
    #[serde(rename = "endsAt")]
    ends_at: String,
    #[serde(rename = "generatorURL")]
    generator_url: String,
    fingerprint: String,
}

#[derive(Serialize)]
struct RequestBodyText<'a> {
    msgtype: &'a str,
    text: ContentText<'a>,
}

#[derive(Serialize)]
struct ContentText<'a> {
    title: &'a str,
    content: String,
}

#[derive(Serialize)]
struct RequestBodyMarkdown<'a> {
    msgtype: &'a str,
    markdown: ContentMarkdown<'a>,
}

#[derive(Serialize)]
struct ContentMarkdown<'a> {
    title: &'a str,
    text: String,
}

#[derive(Serialize)]
struct CustomResponse<T> {
    msg: String,
    data: Option<T>,
}

impl<T> CustomResponse<T> {
    #[allow(dead_code)]
    fn new(msg: &str, data: Option<T>) -> Self {
        CustomResponse {
            msg: msg.to_string(),
            data,
        }
    }

    fn ok(data: Option<T>) -> Self {
        CustomResponse {
            msg: "ok".to_string(),
            data,
        }
    }
    fn err(msg: &str) -> Self {
        CustomResponse {
            msg: msg.to_string(),
            data: None,
        }
    }
    fn to_json(self) -> Json<CustomResponse<T>> {
        Json(self)
    }
}

#[derive(Deserialize)]
struct DingResp {
    errcode: i32,
    errmsg: String,
}

pub async fn ding_text(
    Json(input): Json<PrometeusPost>,
    Extension((title, ding_url)): Extension<(String, String)>,
) -> impl IntoResponse {
    if input.alerts.len() == 0 {
        info!("no alert will send!");
        return CustomResponse::<i32>::err("no alert").to_json();
    }

    if let Ok(c) = serde_json::to_string_pretty(&input.alerts) {
        match send_text(&ding_url, &title, c.as_str()).await {
            Ok(resp) => {
                info!("send ding msg: {}", resp.errmsg);
                return CustomResponse::<i32>::ok(Some(resp.errcode)).to_json();
            }
            Err(err) => {
                error!("send ding ding err: {}, url: {}", err, ding_url);
                return CustomResponse::<i32>::err(err.to_string().as_str()).to_json();
            }
        }
    } else {
        return CustomResponse::<i32>::err("serialize alert to json err").to_json();
    };
}

async fn send_text(ding_url: &str, title: &str, c: &str) -> Result<DingResp> {
    let content = ContentText {
        title,
        content: c.to_string(),
    };
    let req_body = RequestBodyText {
        msgtype: "text",
        text: content,
    };

    let client = reqwest::Client::new();
    let res = client
        .post(ding_url)
        .json(&req_body)
        .send()
        .await?
        .json::<DingResp>()
        .await?;
    Ok(res)
}

pub async fn ding_markdown(
    Json(input): Json<PrometeusPost>,
    Extension((title, ding_url)): Extension<(String, String)>,
) -> impl IntoResponse {
    if input.alerts.len() == 0 {
        info!("no alert will send!");
        return CustomResponse::<i32>::err("no alert").to_json();
    }

    let mut mds = vec![];
    let local = Local::now().to_string();
    let mut custom_ding_url = ding_url.clone();

    for alert in &input.alerts {
        mds.push(format!(
            "## {}\n* date:{}\n* summary:{}\n* description:{}\n",
            alert.labels["alertname"],
            local,
            alert.annotations["summary"],
            alert.annotations["description"],
        ));
        if alert.annotations.contains_key("ding_url") {
            let url = alert.annotations["ding_url"].clone();
            custom_ding_url = url;
        }
    }

    match send_markdown(&custom_ding_url, &title, mds.join("\n").as_str()).await {
        Ok(resp) => {
            info!("send ding msg: {}", resp.errmsg);
            return CustomResponse::<i32>::ok(Some(resp.errcode)).to_json();
        }
        Err(err) => {
            error!("send ding ding err: {}, url: {}", err, custom_ding_url);
            return CustomResponse::<i32>::err(err.to_string().as_str()).to_json();
        }
    }
}

async fn send_markdown(ding_url: &str, title: &str, c: &str) -> Result<DingResp> {
    let content = ContentMarkdown {
        title,
        text: c.to_string(),
    };
    let req_body = RequestBodyMarkdown {
        msgtype: "markdown",
        markdown: content,
    };

    let client = reqwest::Client::new();
    let res = client
        .post(ding_url)
        .json(&req_body)
        .send()
        .await?
        .json::<DingResp>()
        .await?;
    Ok(res)
}
