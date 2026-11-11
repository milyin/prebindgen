//! Domain-value conversion coverage; these types deliberately have no foreign class.
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicI64, Ordering};

#[prebindgen]
pub struct CanonMillis {
    pub value: i64,
}
#[prebindgen]
pub fn canonical_millis_new(value: i64) -> CanonMillis {
    CanonMillis { value }
}
#[prebindgen]
pub fn canonical_millis_value(value: &CanonMillis) -> i64 {
    value.value
}
#[prebindgen]
pub fn canonical_millis_echo(value: CanonMillis) -> CanonMillis {
    value
}
#[prebindgen]
pub fn canonical_millis_optional(value: Option<CanonMillis>) -> Option<CanonMillis> {
    value
}
#[prebindgen]
pub fn canonical_millis_rows(value: Vec<CanonMillis>) -> Vec<CanonMillis> {
    value
}
#[prebindgen]
pub fn canonical_millis_callback(callback: impl Fn(CanonMillis) -> CanonMillis) -> CanonMillis {
    callback(CanonMillis { value: 19 })
}
#[prebindgen]
pub fn canonical_millis_notify(callback: impl Fn(CanonMillis)) {
    callback(CanonMillis { value: 23 })
}
#[prebindgen]
pub fn canonical_millis_static() -> &'static CanonMillis {
    static VALUE: CanonMillis = CanonMillis { value: 31 };
    &VALUE
}

#[prebindgen]
#[derive(Clone, Copy)]
pub struct CanonCelsius {
    pub value: i32,
}
impl From<i32> for CanonCelsius {
    fn from(value: i32) -> Self {
        Self { value }
    }
}
impl From<CanonCelsius> for i32 {
    fn from(value: CanonCelsius) -> Self {
        value.value
    }
}
#[prebindgen]
pub fn canonical_celsius(value: CanonCelsius) -> CanonCelsius {
    CanonCelsius {
        value: value.value + 1,
    }
}

#[prebindgen]
pub struct CanonPercent {
    pub value: i32,
}
impl TryFrom<i32> for CanonPercent {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, String> {
        if (0..=100).contains(&value) {
            Ok(Self { value })
        } else {
            Err(format!("percent outside range: {value}"))
        }
    }
}
impl TryFrom<CanonPercent> for i32 {
    type Error = String;
    fn try_from(value: CanonPercent) -> Result<i32, String> {
        if value.value <= 100 {
            Ok(value.value)
        } else {
            Err(format!("invalid output percent: {}", value.value))
        }
    }
}
#[prebindgen]
pub fn canonical_percent(value: Option<CanonPercent>) -> Option<CanonPercent> {
    value
}
#[prebindgen]
pub fn canonical_percent_invalid() -> CanonPercent {
    CanonPercent { value: 101 }
}

#[prebindgen]
pub struct CanonLabel {
    pub value: String,
}
#[prebindgen]
pub fn canonical_label_new(value: String) -> Result<CanonLabel, String> {
    if value.is_empty() {
        Err("empty canonical label".into())
    } else {
        Ok(CanonLabel { value })
    }
}
#[prebindgen]
pub fn canonical_label_value(value: &CanonLabel) -> &str {
    &value.value
}
#[prebindgen]
pub fn canonical_label_echo(value: CanonLabel) -> CanonLabel {
    value
}
#[prebindgen]
pub struct CanonSettings {
    pub delay: CanonMillis,
    pub optional: Option<CanonMillis>,
    pub history: Vec<CanonMillis>,
    pub label: CanonLabel,
}
#[prebindgen]
pub fn canonical_settings(value: CanonSettings) -> CanonSettings {
    value
}
#[prebindgen]
pub fn canonical_settings_rows(value: Vec<CanonSettings>) -> Vec<CanonSettings> {
    value
}
#[prebindgen]
pub fn canonical_settings_callback(
    callback: impl Fn(CanonSettings) -> CanonSettings,
) -> CanonSettings {
    callback(CanonSettings {
        delay: CanonMillis { value: 3 },
        optional: None,
        history: vec![CanonMillis { value: 7 }],
        label: CanonLabel {
            value: "callback".into(),
        },
    })
}
#[prebindgen]
pub struct CanonView {
    pub delay: CanonMillis,
}
#[prebindgen]
pub fn canonical_view() -> &'static CanonView {
    static VALUE: CanonView = CanonView {
        delay: CanonMillis { value: 41 },
    };
    &VALUE
}

#[prebindgen]
pub struct CanonError {
    pub code: i32,
}
#[prebindgen]
pub fn canonical_error_code(error: &CanonError) -> i32 {
    error.code
}
#[prebindgen]
pub fn canonical_domain_error(value: CanonMillis) -> Result<CanonMillis, CanonError> {
    if value.value < 0 {
        Err(CanonError { code: 17 })
    } else {
        Ok(value)
    }
}

static LIVE: AtomicI64 = AtomicI64::new(0);
#[prebindgen]
pub struct CanonToken {
    pub value: i64,
}
impl Drop for CanonToken {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn canonical_token_new(value: i64) -> CanonToken {
    LIVE.fetch_add(1, Ordering::SeqCst);
    CanonToken { value }
}
#[prebindgen]
pub fn canonical_token_value(value: &CanonToken) -> i64 {
    value.value
}
#[prebindgen]
pub fn canonical_live() -> i64 {
    LIVE.load(Ordering::SeqCst)
}
#[prebindgen]
pub struct CanonOwned {
    pub token: CanonToken,
}
#[prebindgen]
pub fn canonical_owned_new(token: CanonToken) -> Result<CanonOwned, String> {
    if token.value < 0 {
        Err("negative canonical token".into())
    } else {
        Ok(CanonOwned { token })
    }
}
#[prebindgen]
pub fn canonical_owned_token(value: CanonOwned) -> CanonToken {
    value.token
}
#[prebindgen]
pub fn canonical_consume(
    first: CanonOwned,
    second: CanonOwned,
    third: CanonOwned,
) -> Result<i64, CanonError> {
    Ok(first.token.value + second.token.value + third.token.value)
}
#[prebindgen]
pub fn canonical_owned_callback(callback: impl Fn(CanonOwned) -> CanonOwned) -> CanonOwned {
    callback(CanonOwned {
        token: canonical_token_new(53),
    })
}

#[prebindgen]
pub fn canonical_millis_local(value: CanonMillis) -> CanonMillis {
    value
}

impl From<i32> for CanonMillis {
    fn from(value: i32) -> Self {
        Self {
            value: value as i64,
        }
    }
}
impl From<CanonMillis> for i32 {
    fn from(value: CanonMillis) -> Self {
        value.value as i32
    }
}
#[prebindgen]
pub fn canonical_millis_callback_local(
    callback: impl Fn(CanonMillis) -> CanonMillis,
) -> CanonMillis {
    callback(CanonMillis { value: 29 })
}

#[prebindgen]
pub fn canonical_view_echo(value: CanonView) -> CanonView {
    value
}

#[prebindgen]
pub fn canonical_millis_borrow(value: &CanonMillis) -> i64 {
    value.value
}
#[prebindgen]
pub fn canonical_from_borrowed_token(value: &CanonToken) -> CanonMillis {
    CanonMillis { value: value.value }
}
#[prebindgen]
pub fn canonical_millis_from_token(value: CanonMillis) -> CanonMillis {
    value
}

impl From<String> for CanonError {
    fn from(_: String) -> Self {
        Self { code: -1 }
    }
}
