use serde::Serialize;

#[derive(Serialize)]
pub struct SocketIOEvent<T: Serialize> {
    pub name: String,
    pub data: T,
    pub ts: u64,
}

