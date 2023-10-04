/**
 * Copyright 2023 Isis <root@5ht2.me>
 * 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/**
 * @typedef {Object} Stats
 * @property {number} num_visits
 * @property {number} total_beats
 * @property {number} uptime
 * @property {number} last_seen
 * @property {number} last_seen_relative
 */

const $i = document.getElementById.bind(document);
const months = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October','November','December'];
const units = {Y: 'year', m: 'month', w: 'week', d: 'day', H: 'hour', M: 'minute', S: 'second'};

/**
 * Pad a number with a leading zero if it is less than 10.
 * @param {number} n The number to pad. 
 * @returns {string} The padded number.
 */
function zeroPad(n) {
  return n < 10 ? '0' + n : n;
}

/**
 * Pluralize a string based on a number.
 * @param {number} n The number of items.
 * @param {string} s The singular form of the item.
 * @returns {string}
 */
function plural(n, s) {
  return `${n} ${s}${n === 1 ? '' : 's'}`;
}

/**
 * Format a Unix timestamp as a human-readable date.
 * @param {number} secs A Unix timestamp.
 * @returns {string} The formatted date.
 */
function formatDate(secs) {
  const dt = new Date(secs * 1000);
  const Y = dt.getUTCFullYear();
  const m = months[dt.getUTCMonth()];
  const d = zeroPad(dt.getUTCDate());
  const H = zeroPad(dt.getUTCHours());
  const M = zeroPad(dt.getUTCMinutes());
  const S = zeroPad(dt.getUTCSeconds());
  return `${d} ${m} ${Y} ${H}:${M}:${S} UTC`;
}

/**
 * Format `secs` seconds as a human-readable duration.
 * @param {number} secs The number of seconds in the duration.
 * @returns {string} The formatted duration.
 */
function formatRelativeTime(secs) {
  let Y, m, w, d, H, M, S, rem;
  [Y, rem] = [Math.floor(secs / 31536000), secs % 31536000];
  [m, rem] = [Math.floor(rem / 2592000), rem % 2592000];
  [w, rem] = [Math.floor(rem / 604800), rem % 604800];
  [d, rem] = [Math.floor(rem / 86400), rem % 86400];
  [H, rem] = [Math.floor(rem / 3600), rem % 3600];
  [M, S] = [Math.floor(rem / 60), Math.round(rem % 60)];
  const fmt = { Y, m, w, d, H, M, S };
  const arr = [];
  Object.entries(fmt).filter(([, v]) => v > 0).forEach(([k, v]) => arr.push(plural(v, units[k])));
  if (arr.length === 0) {
    return '0 seconds';
  }
  if (arr.length === 1) {
    return arr[0];
  }
  if (arr.length === 2) {
    return `${arr[0]} and ${arr[1]}`;
  }
  return arr.slice(0, -1).join(', ') + ', and ' + arr.slice(-1);
}

/**
 * Refresh components on the stats page.
 * This is a lightweight and more retro version of what Preact does.
 * @param {Stats} stats
 * @returns {void} 
 */
function Stats(stats) {
  $i('visits').innerText = stats.num_visits.toLocaleString('en-GB');
  $i('devices').innerText = stats.devices.length.toLocaleString('en-GB');
  $i('total-beats').innerText = stats.total_beats.toLocaleString('en-GB');
  $i('uptime').innerText = formatRelativeTime(stats.uptime);
}

/**
 * Refresh components on the index page.
 * This is a lightweight and more retro version of what Preact does.
 * @param {Stats} stats
 * @returns {void} 
 */
function Index(stats) {
  $i('last-seen').innerText = formatDate(stats.last_seen);
  $i('time-difference').innerText = formatRelativeTime(stats.last_seen_relative, true);
  $i('longest-absence').innerText = formatRelativeTime(stats.longest_absence);
  $i('total-beats').innerText = stats.total_beats.toLocaleString('en-GB');
}

document.addEventListener('DOMContentLoaded', () => {
  const path = window.location.pathname;
  const component = path === '/' ? Index : Stats;
  const url = new URL('/api/stats/ws', window.location.href);
  // http -> ws
  // https -> wss
  url.protocol = url.protocol.replace('http', 'ws');
  const ws = new WebSocket(url.href);

  ws.onmessage = (ev) => {
    /** @type {Stats} */
    let json = JSON.parse(ev.data);
    component(json);
  };
});
