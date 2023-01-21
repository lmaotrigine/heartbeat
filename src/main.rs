/**
 * Copyright (c) 2023 VJ <root@5ht2.me>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use lazy_static::lazy_static;
use rocket::{
    self, catchers,
    fs::{FileServer, Options as FsOptions},
    Config,
};
use rocket_db_pools::{sqlx, Database};
use rocket_dyn_templates::Template;

mod config;
mod guards;
mod models;
mod routes;
mod util;

lazy_static! {
    pub static ref GIT_HASH: &'static str = match option_env!("HB_GIT_COMMIT") {
        Some(hash) => hash,
        None => "",
    };
    pub static ref CONFIG: config::Config = config::Config::try_new().expect("failed to load config file");
}

#[cfg(feature = "webhook")]
lazy_static! {
    pub static ref WEBHOOK: util::Webhook = util::Webhook::new(&CONFIG.webhook);
}

#[derive(Database)]
#[database("main")]
pub struct DbPool(pub sqlx::PgPool);

#[rocket::launch]
async fn launch() -> _ {
    lazy_static::initialize(&GIT_HASH);
    lazy_static::initialize(&CONFIG);
    #[cfg(feature = "webhook")]
    lazy_static::initialize(&WEBHOOK);
    let figment = Config::figment().merge(("databases.main.url", &CONFIG.database.dsn));
    rocket::custom(figment)
        .attach(DbPool::init())
        .register("/", catchers![routes::default_catcher])
        .mount("/", routes::get_routes())
        .mount(
            "/",
            FileServer::new("static/", FsOptions::NormalizeDirs | FsOptions::default()),
        )
        .attach(Template::custom(|engine| {
            engine
                .tera
                .register_filter("format_relative", util::tera::format_relative);
            engine.tera.register_filter("format_num", util::tera::format_num);
        }))
}
