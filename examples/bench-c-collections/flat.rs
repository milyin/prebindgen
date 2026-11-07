use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();

pub struct ResourceImpl(u64);
#[prebindgen]
pub type Resource = ResourceImpl;
#[prebindgen]
pub fn resource_value(value: &Resource) -> u64 {
    value.0
}
#[prebindgen]
pub struct Record {
    pub first: Resource,
    pub tag: u64,
    pub second: Resource,
}
#[prebindgen]
pub fn record_into(value: Record) -> Record {
    value
}
#[prebindgen]
pub fn make_handles(n: usize) -> Vec<Resource> {
    std::hint::black_box((0..n).map(|i| ResourceImpl(i as u64)).collect())
}
#[prebindgen]
pub fn make_records(n: usize) -> Vec<Record> {
    std::hint::black_box(
        (0..n)
            .map(|i| Record {
                first: ResourceImpl(i as u64),
                tag: i as u64,
                second: ResourceImpl(i as u64 + 1),
            })
            .collect(),
    )
}
#[prebindgen]
pub fn make_nested(n: usize) -> Vec<Vec<Resource>> {
    std::hint::black_box(vec![vec![], make_handles(n), vec![]])
}
#[prebindgen]
pub fn make_many(n: usize) -> Vec<Vec<Resource>> {
    std::hint::black_box((0..n).map(|i| vec![ResourceImpl(i as u64)]).collect())
}
#[prebindgen]
pub fn make_numbers(n: usize) -> Vec<u64> {
    std::hint::black_box((0..n as u64).collect())
}
#[prebindgen]
pub fn control(n: usize) -> u64 {
    std::hint::black_box(n as u64)
}

#[prebindgen]
pub fn resource_new(value: u64) -> Resource {
    ResourceImpl(value)
}
#[prebindgen]
pub fn record_new(first: Resource, tag: u64, second: Resource) -> Record {
    Record { first, tag, second }
}
#[prebindgen]
pub fn take_handles(values: Vec<Resource>) -> u64 {
    values.iter().map(|v| v.0).sum()
}
#[prebindgen]
pub fn take_records(values: Vec<Record>) -> u64 {
    values.iter().map(|v| v.first.0 + v.tag + v.second.0).sum()
}
#[prebindgen]
pub struct WideInput {
    pub resource: Resource,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub e: u64,
    pub f: u64,
    pub g: u64,
}
#[prebindgen]
#[allow(clippy::too_many_arguments)]
pub fn wide_new(
    resource: Resource,
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
    g: u64,
) -> WideInput {
    WideInput {
        resource,
        a,
        b,
        c,
        d,
        e,
        f,
        g,
    }
}
#[prebindgen]
pub fn take_wide(values: Vec<WideInput>) -> u64 {
    values
        .iter()
        .map(|v| v.resource.0 + v.a + v.b + v.c + v.d + v.e + v.f + v.g)
        .sum()
}
#[prebindgen]
pub struct TextInput {
    pub resource: Resource,
    pub text: String,
}
#[prebindgen]
pub fn text_new(resource: Resource, text: String) -> TextInput {
    TextInput { resource, text }
}
#[prebindgen]
pub fn take_text(values: Vec<TextInput>) -> u64 {
    values
        .iter()
        .map(|v| v.resource.0 + v.text.len() as u64)
        .sum()
}
#[prebindgen]
pub struct BytesInput {
    pub resource: Resource,
    pub bytes: Vec<u8>,
}
#[prebindgen]
pub fn bytes_new(resource: Resource, bytes: Vec<u8>) -> BytesInput {
    BytesInput { resource, bytes }
}
fn bytes_score(bytes: &[u8]) -> u64 {
    bytes.len() as u64
        + u64::from(bytes.first().copied().unwrap_or(0))
        + u64::from(bytes.last().copied().unwrap_or(0))
}
#[prebindgen]
pub fn take_bytes(values: Vec<BytesInput>) -> u64 {
    values
        .iter()
        .map(|v| v.resource.0 + bytes_score(&v.bytes))
        .sum()
}
#[prebindgen]
pub struct BuffersInput {
    pub resource: Resource,
    pub text: String,
    pub bytes: Vec<u8>,
}
#[prebindgen]
pub fn buffers_new(resource: Resource, text: String, bytes: Vec<u8>) -> BuffersInput {
    BuffersInput {
        resource,
        text,
        bytes,
    }
}
#[prebindgen]
pub fn take_buffers(values: Vec<BuffersInput>) -> u64 {
    values
        .iter()
        .map(|v| v.resource.0 + v.text.len() as u64 + bytes_score(&v.bytes))
        .sum()
}
#[prebindgen]
pub enum InputTag {
    Low = -3,
    Normal = 0,
    High = 17,
}
fn tag_score(tag: &InputTag) -> u64 {
    match tag {
        InputTag::Low => 1,
        InputTag::Normal => 2,
        InputTag::High => 3,
    }
}
#[prebindgen]
pub struct TaggedInput {
    pub resource: Resource,
    pub tag: InputTag,
}
#[prebindgen]
pub fn tagged_new(resource: Resource, tag: InputTag) -> TaggedInput {
    TaggedInput { resource, tag }
}
#[prebindgen]
pub fn take_tagged(values: Vec<TaggedInput>) -> u64 {
    values
        .iter()
        .map(|v| v.resource.0 + tag_score(&v.tag))
        .sum()
}
#[prebindgen]
pub struct TaggedBuffersInput {
    pub resource: Resource,
    pub tag: InputTag,
    pub text: String,
    pub bytes: Vec<u8>,
}
#[prebindgen]
pub fn tagged_buffers_new(
    resource: Resource,
    tag: InputTag,
    text: String,
    bytes: Vec<u8>,
) -> TaggedBuffersInput {
    TaggedBuffersInput {
        resource,
        tag,
        text,
        bytes,
    }
}
#[prebindgen]
pub fn take_tagged_buffers(values: Vec<TaggedBuffersInput>) -> u64 {
    values
        .iter()
        .map(|v| v.resource.0 + tag_score(&v.tag) + v.text.len() as u64 + bytes_score(&v.bytes))
        .sum()
}
#[prebindgen]
pub fn take_numbers(values: Vec<u64>) -> u64 {
    values.iter().sum()
}

#[prebindgen]
pub fn fixed_borrow(first: &Resource, second: &mut Resource) -> u64 {
    std::hint::black_box(first.0 + second.0)
}
#[prebindgen]
pub fn fixed_move(first: Resource, second: Resource) -> u64 {
    std::hint::black_box(first.0 + second.0)
}
#[prebindgen]
pub fn callback_fixed(callback: impl Fn() -> Record) -> u64 {
    let v = callback();
    v.first.0 + v.tag + v.second.0
}
#[prebindgen]
pub fn callback_many(callback: impl Fn() -> Vec<Resource>) -> u64 {
    callback().iter().map(|v| v.0).sum()
}

// Native inline owners retain both payload allocations during C input conversion.
#[prebindgen]
#[derive(Default)]
pub struct NativePacket {
    pub bytes: Vec<u8>,
    pub text: String,
    pub id: u64,
}
#[prebindgen]
pub fn native_packet_new(id: u64) -> NativePacket {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(b"abc");
    let mut text = String::with_capacity(64);
    text.push_str("abc");
    NativePacket { bytes, text, id }
}
#[prebindgen]
pub fn take_native(values: Vec<NativePacket>) -> u64 {
    values
        .iter()
        .map(|v| v.id + v.bytes.len() as u64 + v.text.len() as u64)
        .sum()
}
#[prebindgen]
pub struct NativeMixed {
    pub first: Resource,
    pub packet: NativePacket,
    pub last: Resource,
}
#[prebindgen]
pub fn native_mixed_new(first: Resource, packet: NativePacket, last: Resource) -> NativeMixed {
    NativeMixed {
        first,
        packet,
        last,
    }
}
#[prebindgen]
pub fn take_native_mixed(values: Vec<NativeMixed>) -> u64 {
    values
        .iter()
        .map(|v| {
            v.first.0
                + v.packet.id
                + v.packet.bytes.len() as u64
                + v.packet.text.len() as u64
                + v.last.0
        })
        .sum()
}

#[prebindgen]
pub fn callback_native(callback: impl Fn() -> Vec<NativePacket>) -> u64 {
    take_native(callback())
}
#[prebindgen]
pub fn callback_native_mixed(callback: impl Fn() -> Vec<NativeMixed>) -> u64 {
    take_native_mixed(callback())
}

// Borrowed source handles allow stable input fixtures across measured calls.
#[prebindgen]
pub fn history_read(value: &Resource) -> u64 {
    std::hint::black_box(value.0)
}
#[prebindgen]
pub fn history_write(value: &mut Resource) -> u64 {
    std::hint::black_box(value.0)
}
#[prebindgen]
pub fn history_flat(values: Vec<u64>) -> u64 {
    std::hint::black_box(values).iter().sum()
}
#[prebindgen]
pub fn history_nested(values: Vec<Vec<u64>>) -> u64 {
    std::hint::black_box(values).iter().flatten().sum()
}
#[prebindgen]
pub fn history_shared(values: Vec<Vec<u64>>, token: &mut Resource) -> u64 {
    std::hint::black_box(values).iter().flatten().sum::<u64>() + token.0
}
#[prebindgen]
pub fn history_callback(callback: impl Fn() -> Vec<Vec<Resource>>) -> u64 {
    callback().iter().flatten().map(|v| v.0).sum()
}

#[prebindgen]
pub fn history_readonly(values: Vec<Vec<u64>>) -> u64 {
    history_nested(values)
}
