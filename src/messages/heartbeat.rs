// src/messages/heartbeat.rs
/// 心跳消息 (MsgType=3)，没有消息体
#[derive(Debug, Clone, Copy)]
pub struct Heartbeat;

// 无需解析函数，仅用作类型标识
