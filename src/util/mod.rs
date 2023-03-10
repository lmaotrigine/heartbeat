/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
pub mod hf_time;
mod plural;
mod snowflake;
pub use snowflake::{Snowflake, SnowflakeGenerator};
mod serializers;
pub use serializers::*;
pub mod tera;
mod token;
pub use token::generate_token;

#[cfg(feature = "badges")]
pub mod badge;
#[cfg(feature = "webhook")]
mod webhook;
#[cfg(feature = "webhook")]
pub use webhook::{Webhook, WebhookColour};
