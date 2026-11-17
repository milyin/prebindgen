// Selected declarations from eclipse-zenoh/zenoh-flat at
// 99a7e7084e00824f85c94cbabfa9573108d53dea (stable + unstable enabled).
// Attributes, documentation and function bodies are omitted. This fixture is
// parsed by syn, not compiled as a replacement for the Zenoh source crate.

pub type Duration = std::time::Duration;
pub type Sample = zenoh::sample::Sample;
pub type Reply = zenoh::query::Reply;
pub type ReplyError = zenoh::query::ReplyError;
pub type TimestampStack = zenoh::timestamp_stack::TimestampStack;

pub enum RecoveryMode {
    PeriodicQueries(Duration),
    Heartbeat,
}

pub struct RecoveryConfig {
    pub mode: Option<RecoveryMode>,
    pub retention_period: Option<Duration>,
}

pub struct Timestamp {
    pub ntp64: u64,
    pub id: Vec<u8>,
}

pub enum InterceptionPoint {
    Send,
    Route,
    Receive,
}

pub enum InstrumentationTimestamp {
    Uhlc(Timestamp),
    Custom(Vec<u8>),
}

pub struct TimestampStackRecord {
    pub point: InterceptionPoint,
    pub timestamp: InstrumentationTimestamp,
}

pub const ZENOH_ID_MAX_SIZE: usize = 16;

pub struct ZenohId {
    pub bytes: [u8; ZENOH_ID_MAX_SIZE],
}

pub struct EntityGlobalId {
    pub zid: ZenohId,
    pub eid: u32,
}

pub enum ReplyResult {
    Sample(Sample),
    Error(ReplyError),
}

pub struct ReplyStruct {
    pub result: ReplyResult,
    pub replier_id: Option<EntityGlobalId>,
}
