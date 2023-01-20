/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use reqwest::Client;

use crate::{
    config::{WebhookConfig, WebhookLevel},
    CONFIG,
};

pub struct Webhook {
    config: &'static WebhookConfig,
    client: Client,
}

pub enum WebhookColour {
    Green = 0x42f598,
    Orange = 0xde953c,
    Blue = 0x6495ed,
}

impl Webhook {
    pub fn new(config: &'static WebhookConfig) -> Self {
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
        colour: WebhookColour,
    ) -> Result<(), String> {
        if self.config.level > level {
            return Ok(());
        }
        let server_url = &CONFIG.live_url;
        let wh_url = &self.config.url;
        if wh_url.is_empty() {
            return Ok(());
        }
        let host = match reqwest::Url::parse(server_url) {
            Ok(url) => url.host_str().unwrap_or(&CONFIG.server_name).to_string(),
            Err(_) => return Err("Invalid server URL".into()),
        };
        let avatar = format!("{}/favicon.png", server_url);
        let body = rocket::serde::json::json!({
            "embeds": [{
                "author": {
                    "name": host,
                    "url": server_url,
                    "icon_url": avatar,
                },
                "title": title,
                "description": message,
                "color": colour as u32,
            }],
            "avatar_url": avatar,
            "username": host,
        });
        let response = match self.client.post(wh_url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => return Err(format!("failed to send webhook: {}", e)),
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
