use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();
#[derive(Clone)]
pub struct Key(String);
#[prebindgen]
pub type KeyExpr = Key;
#[prebindgen]
#[derive(Clone)]
pub struct Payload {
    pub key: KeyExpr,
    pub data: Vec<u8>,
}
#[prebindgen]
pub fn keyexpr_from_string(s: String) -> KeyExpr {
    Key(s)
}
#[prebindgen]
pub fn keyexpr_to_string(k: &KeyExpr) -> String {
    k.0.clone()
}
#[prebindgen]
pub fn keyexpr_text(k: KeyExpr) -> String {
    k.0
}
#[prebindgen]
pub fn bytes_from_slice(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}
#[prebindgen]
pub fn bytes_read(data: Vec<u8>) -> Vec<u8> {
    data
}
#[prebindgen]
pub fn payload_from_string(k: KeyExpr, d: String) -> Payload {
    Payload {
        key: k,
        data: d.into_bytes(),
    }
}
#[prebindgen]
pub fn payload_from_slice(k: KeyExpr, d: &[u8]) -> Payload {
    Payload {
        key: k,
        data: d.to_vec(),
    }
}
#[prebindgen]
pub fn payload_text(p: Payload) -> String {
    format!("{}:{}", p.key.0, String::from_utf8_lossy(&p.data))
}
#[prebindgen]
pub fn foo(k: KeyExpr, mut x: Payload, y: Payload) -> Payload {
    assert_ne!(k.0, "panic", "requested source panic");
    x.key.0 = format!("{}:{}:{}", k.0, x.key.0, y.key.0);
    x.data.extend(y.data);
    x
}
#[prebindgen]
pub fn noop() {}

#[prebindgen]
pub fn echo_key(k: KeyExpr) -> KeyExpr {
    k
}
#[prebindgen]
pub fn echo_key_raw(k: KeyExpr) -> KeyExpr {
    k
}
#[prebindgen]
pub fn keyexpr_pair(k: KeyExpr) -> KeyExpr {
    k
}
#[prebindgen]
pub fn noop_extra() {}
