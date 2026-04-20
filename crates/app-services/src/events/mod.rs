use serde::Serialize;

pub mod ask;
pub mod loop_events;
pub mod plan;

pub trait Event: Send + 'static {
    const NAME: &'static str;
    type Payload: Serialize + Send + 'static;

    fn payload(&self) -> Self::Payload;
}
