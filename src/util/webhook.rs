// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reqwest::Client;
use serde::Serialize;

use crate::config::{Config, Webhook as WebhookConfig, WebhookLevel};

#[derive(Debug, Clone)]
pub struct Webhook {
    config: WebhookConfig,
    client: Client,
}

pub enum Colour {
    Green = 0x42_f5_98,
    Orange = 0xde_95_3c,
    Blue = 0x64_95_ed,
}

#[derive(Serialize)]
struct WebhookRequest<'a> {
    embeds: [Embed<'a>; 1],
    avatar_url: &'a str,
    username: &'a str,
}

#[derive(Serialize)]
struct Embed<'a> {
    author: Author<'a>,
    title: String,
    description: String,
    color: u32,
}

#[derive(Serialize)]
struct Author<'a> {
    name: &'a str,
    url: &'a str,
    icon_url: &'a str,
}

impl Webhook {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn execute(
        &self,
        title: String,
        message: String,
        level: WebhookLevel,
        colour: Colour,
        config: &Config,
    ) -> Result<(), String> {
        if self.config.level > level {
            return Ok(());
        }
        let wh_url = &self.config.url;
        if wh_url.is_empty() {
            return Ok(());
        }
        let host = match reqwest::Url::parse(&config.live_url) {
            Ok(url) => url.host_str().unwrap_or(&config.server_name).to_string(),
            Err(_) => return Err("Invalid server URL".into()),
        };
        let avatar = format!("{}/favicon.png", &config.live_url);
        let body = WebhookRequest {
            embeds: [Embed {
                author: Author {
                    name: &host,
                    url: &config.live_url,
                    icon_url: &avatar,
                },
                title,
                description: message,
                color: colour as u32,
            }],
            avatar_url: &avatar,
            username: &host,
        };
        let response = match self.client.post(wh_url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => return Err(format!("failed to send webhook: {e}")),
        };
        match response.status().as_u16() {
            200..=299 => Ok(()),
            _ => Err(format!(
                "failed to send webhook: {}",
                response.text().await.unwrap_or_default()
            )),
        }
    }
}
