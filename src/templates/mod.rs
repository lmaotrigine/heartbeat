// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    config::Config,
    stats::Stats,
    util::formats::{format_relative, FormatNum},
};
use chrono::{DateTime, Utc};
use html::{html, Markup, PreEscaped, DOCTYPE};

// We use inline JavaScript here despite the performance hit because it's
// easier to inspect, and page load times are still under 600ms, which is
// not great, but not terrible either.
// If this were a separate file and we load it like we do the stylesheet,
// cloudflare would minify it, rendering this exercise moot.
const JAVASCRIPT: &str = include_str!("./script.mjs");

fn base(title: impl AsRef<str>, include_original_license: bool, extra_head: Option<Markup>, body: &Markup) -> Markup {
    let title = title.as_ref();
    html! {
        (DOCTYPE)
        (PreEscaped(r#"
<!-- Copyright 2023 Isis <root@5ht2.me>
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
                link rel="stylesheet" href="/style.css";
                @if let Some(extra_head) = extra_head {
                    (extra_head)
                }
            }
            (body)
        }
    }
}

pub fn error(message: impl AsRef<str>, method: &str, path: &str, server_name: &str) -> Markup {
    let message = message.as_ref();
    let body = html! {
        body {
            div.spacer {}
            div.privacy {
                div.grid-cell {}
                div {
                    p.centre {
                        (message)": " (method) " on " b { (path) }
                    }
                }
                div.grid-cell {}
            }
            div.spacer {}
            div.links {
                div.grid-cell;
                div {
                    p.centre {
                        a href="/" { "Main Page" }
                    }
                }
                div.grid-cell {}
            }
        }
    };
    base(format!("{message} - {server_name}"), true, None, &body)
}

pub fn index(stats: &Stats, commit: &str, config: &Config) -> Markup {
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
        (PreEscaped(format!(r#"<script type="module">{JAVASCRIPT}</script>"#)))
    });
    let href = format!("{}/tree/{}", config.repo, commit);
    let body = html! {
        body {
            div.spacer {}
            div.preamble {
                div.grid-cell {}
                div {
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
                div.grid-cell {}
            }
            div.times {
                div.grid-cell {}
                div.grid-cell {
                    p.centre {
                        "Last response time:" br;
                        span #last-seen {
                            (last_seen_fmt)
                            " UTC"
                        }
                    }
                }
                div.grid-cell {
                    p.centre {
                        "Time since last response:" br;
                        span #time-difference {
                            (last_seen_relative)
                        }
                    }
                }
                div.grid-cell {
                    p.centre {
                        "Longest absence:" br;
                        span #longest-absence {
                            (longest_absence)
                        }
                    }
                }
                div.grid-cell {
                    p.centre {
                        "Total beats received:" br;
                        span #total-beats {
                            (total_beats)
                        }
                    }
                }
                div.grid-cell {}
            }
            div.spacer {}
            div.links {
                div.grid-cell {}
                div {
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
                div.grid-cell {}
            }
        }
    };
    base(config.server_name.clone(), true, extra_head, &body)
}

pub fn privacy(config: &Config) -> Markup {
    let body = html! {
        body {
            div.spacer {}
            div.privacy {
                div.grid-cell {}
                div {
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
                div.grid-cell {}
            }
            div.spacer {}
            div.links {
                div.grid-cell;
                div {
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
                div.grid-cell {}
            }
        }
    };
    base(format!("Privacy Policy - {}", config.server_name), true, None, &body)
}

pub fn stats(stats: &Stats, config: &Config, server_start_time: DateTime<Utc>) -> Markup {
    let title = format!("Stats - {}", config.server_name);
    let head = html! {
        meta property="og:site_name" content=(title);
        meta property="og:description" content=(format!("Stats for {}", config.server_name));
        meta name="theme-color" content="#6495ed";
        (PreEscaped(format!(r#"<script type="module">{JAVASCRIPT}</script>"#)))
    };
    let uptime = server_start_time.signed_duration_since(Utc::now());
    let body = html! {
        body {
            div.spacer {}
            div.preamble {
                div.grid-cell {}
                div {
                    p.centre {
                        "Statistics for " (config.server_name)
                    }
                }
                div.grid-cell {}
            }
            div.times {
                div.grid-cell {}
                div.grid-cell {
                    p.centre {
                        "Total visits:"
                        br;
                        span #visits {
                            (stats.num_visits.format())
                        }
                    }
                }
                div.grid-cell {
                    p.centre {
                        "Total devices:"
                        br;
                        span #devices {
                            (stats.devices.len().format())
                        }
                    }
                }
                div.grid-cell {
                    p.centre {
                        "Total beats received:"
                        br;
                        span #total-beats {
                            (stats.total_beats.format())
                        }
                    }
                }
                div.grid-cell {
                    p.centre {
                        "Uptime:"
                        br;
                        span #uptime {
                            (format_relative(uptime))
                        }
                    }
                }
                div.grid-cell {}
            }
            div.spacer {}
            div.links {
                div.grid-cell {}
                div {
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
                div.grid-cell {}
            }
        }
    };
    base(format!("Stats - {}", config.server_name), true, Some(head), &body)
}
