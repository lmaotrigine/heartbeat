// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    models::Stats,
    util::formats::{format_relative, Formattable},
    CONFIG, GIT_HASH, SERVER_START_TIME,
};
use chrono::Utc;
use html::{html, Markup, PreEscaped, DOCTYPE};

fn base(title: impl AsRef<str>, include_original_license: bool, extra_head: Option<Markup>) -> Markup {
    let title = title.as_ref();
    html! {
        (DOCTYPE)
        (PreEscaped(r#"
<!-- Copyright 2023 VJ <root@5ht2.me>
   -
   - This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/."#
        ))
        @if include_original_license {
            (PreEscaped(r#"
   -
   - This file incorporates work covered by the following copyright and
   - permission notice:
   -
   -   Copyright 2020-2023 Liv <liv@frogg.ie>
   -
   -   Permission to use, copy, modify, and/or distribute this software for any
   -   purpose with or without fee is hereby granted, provided that the above
   -   copyright notice and this permission notice appear in all copies.
   -
   -   THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
   -   WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
   -   MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
   -   ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
   -   WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
   -   ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
   -   OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE."#
            ))
        }
        (PreEscaped("\n-->\n"))
        html lang="en" {
            head {
                title { (title) }
                meta property="og:image" content="/favicon.png";
                link rel="icon" type="image/x-icon" href="/favicon.ico";
                link rel="stylesheet" href="/grids.min.css";
                link rel="stylesheet" href="/style.css";
                @if let Some(extra_head) = extra_head {
                    (extra_head)
                }
            }
        }
    }
}

pub fn error(message: impl AsRef<str>, method: &str, path: &str) -> Markup {
    let message = message.as_ref();
    let config = CONFIG.get().expect("config to be initialized").clone();
    html! {
        (base(format!("{message} - {}", config.server_name), true, None))
        body {
            div.spacer;
            div.pure-g.privacy {
                div."pure-g-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        (message)": " (method) " on " b { (path) }
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
            div.spacer;
            div.pure-g.links {
                div."pure-g-u-0"."pure-u-lg-1-6";
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        a href="/" { "Main Page" }
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
        }
    }
}

pub fn index(stats: &Stats) -> Markup {
    let commit = *GIT_HASH.get().expect("GIT_HASH to be initialized");
    let config = CONFIG.get().expect("config to be initialized").clone();
    let now = Utc::now();
    let last_seen = stats.last_seen.unwrap_or_else(|| std::time::UNIX_EPOCH.into());
    let last_seen_relative = format_relative(now - last_seen);
    let longest_absence = format_relative(stats.longest_absence);
    let total_beats = stats.total_beats.format();
    let now_fmt = now.format("%d %B %Y %H:%M:%S");
    let last_seen_fmt = last_seen.format("%d %B %Y %H:%M:%S");
    let extra_head = Some(html! {
        meta property="og:site_name" content=(config.server_name);
        meta property="og:description" content=(format!(r#"Last seen at {last_seen_fmt}.
This embed was generated at {now_fmt}.
Due to caching, you will have to check the website if the embed generation time is old."#));
        meta name="theme-color" content="#6495ed";
        (PreEscaped(format!(r#"<script type="module">{}</script>"#, include_str!("./script.mjs"))))
    });
    let href = format!("{}/tree/{}", config.repo, commit);
    html! {
        (base(config.server_name.clone(), true, extra_head))
        body {
            div.spacer {}
            div.pure-g.preamble {
                div."pure-g-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-lg-4-6" {
                    p.centre {
                        "Welcome to " (config.server_name)"." br;
                        "This page displays the last timestamp that they have unlocked and used any of their devices." br;
                        "If they have been absent for more than 48 hours, something is probably wrong." br;
                        "This website is running on version "
                        a href=(href) {
                            code {
                                (commit)
                            }
                        }
                        " of "
                        a href=(config.repo) {
                            "Heartbeat"
                        }
                        "."
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
            div.pure-g.times {
                div."pure-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Last response time:" br;
                        span #last-seen {
                            (last_seen_fmt)
                            " UTC"
                        }
                    }
                }
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Time since last response:" br;
                        span #time-difference {
                            (last_seen_relative)
                            " ago"
                        }
                    }
                }
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Longest absence:" br;
                        span #longest-absence {
                            (longest_absence)
                        }
                    }
                }
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Total beats received:" br;
                        span #total-beats {
                            (total_beats)
                        }
                    }
                }
                div."pure-u-0"."pure-u-lg-1-6" {}
            }
            div.spacer {}
            div.pure-g.links {
                div."pure-g-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        a href="/stats"{
                            "Stats"
                        }
                        " - "
                        a href="/privacy" {
                            "Privacy Policy"
                        }
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
        }
    }
}

pub fn privacy() -> Markup {
    let config = CONFIG.get().expect("config to be initialized").clone();
    html! {
        (base(format!("Privacy Policy - {}", config.server_name), true, None))
        body {
            div.spacer {}
            div.pure-g.privacy {
                div."pure-g-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        "Heartbeat Privacy Information"
                        br;br;
                        "Heartbeat only keeps logs to stdout (terminal output)."
                        br;
                        "IP addresses are only logged on:"
                        br;
                    }
                    p {
                        "- Any POST requests"
                        br;
                        "- Non-GET requests on anything except the main page"
                        br;
                        "- GET requests on non-existent pages"
                    }
                    p.centre {
                        b {
                            "Your IP address will not be logged for normal requests."
                        }
                        br;
                        "Logs are not shared with anybody."
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
            div.spacer;
            div.pure-g.links {
                div."pure-g-u-0"."pure-u-lg-1-6";
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        a href="/" {
                            "Main Page"
                        }
                        " - "
                        a href="/stats" {
                            "Stats"
                        }
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
        }
    }
}

pub async fn stats(stats: &Stats) -> Markup {
    let config = CONFIG.get().expect("config to be initialized").clone();
    let title = format!("Stats - {}", config.server_name);
    let head = html! {
        meta property="og:site_name" content=(title);
        meta property="og:description" content=(format!("Stats for {}", config.server_name));
        meta name="theme-color" content="#6495ed";
        (PreEscaped(format!(r#"<script type="module">{}</script>"#, include_str!("./script.mjs"))))
    };
    let uptime = SERVER_START_TIME
        .get_or_init(|| async { Utc::now() })
        .await
        .signed_duration_since(Utc::now());
    html! {
        (base(format!("Stats - {}", config.server_name), true, Some(head)))
        body {
            div.spacer {}
            div.pure-g.preamble {
                div."pure-g-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        "Statistics for " (config.server_name)
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
            div.pure-g.times {
                div."pure-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Total visits:"
                        br;
                        span #visits {
                            (stats.num_visits.format())
                        }
                    }
                }
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Total devices:"
                        br;
                        span #devices {
                            (stats.devices.len().format())
                        }
                    }
                }
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Total beats received:"
                        br;
                        span #total-beats {
                            (stats.total_beats.format())
                        }
                    }
                }
                div."pure-u-1"."pure-u-lg-1-6" {
                    p.centre {
                        "Uptime:"
                        br;
                        span #uptime {
                            (format_relative(uptime))
                        }
                    }
                }
                div."pure-u-0"."pure-u-lg-1-6" {}
            }
            div.spacer {}
            div.pure-g.links {
                div."pure-g-u-0"."pure-u-lg-1-6" {}
                div."pure-u-1"."pure-u-lg-4-6" {
                    p.centre {
                        a href="/" {
                            "Main Page"
                        }
                        " - "
                        a href="/privacy" {
                            "Privacy Policy"
                        }
                    }
                }
                div."pure-g-u-0"."pure-u-lg-1-6" {}
            }
        }
    }
}
