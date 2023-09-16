// Copyright (c) 2023 Isis <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// overengineering reeee

pub struct Plural<'a> {
    singular: &'a str,
    plural: String,
}

impl<'a> Plural<'a> {
    pub fn from(singular: &'a str) -> Self {
        Self {
            singular,
            plural: format!("{singular}s"),
        }
    }

    #[allow(dead_code)] // we don't deal with non-standard plurals for now
    pub fn plural(mut self, plural: &str) -> Self {
        self.plural = plural.to_owned();
        self
    }

    pub fn compute(&self, n: i64) -> String {
        if n == 1 {
            format!("{} {}", n, self.singular)
        } else {
            format!("{} {}", n, self.plural)
        }
    }
}

pub struct Rough<'a> {
    article: &'a str,
    singular: &'a str,
    plural: String,
}

impl<'a> Rough<'a> {
    pub fn from(singular: &'a str) -> Self {
        Self {
            article: "a",
            singular,
            plural: format!("{singular}s"),
        }
    }

    pub const fn article(mut self, article: &'a str) -> Self {
        self.article = article;
        self
    }

    pub fn compute(&self, n: i64) -> String {
        if n == 1 {
            format!("{} {}", self.article, self.singular)
        } else {
            format!("{} {}", n, self.plural)
        }
    }
}
