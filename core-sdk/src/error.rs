use codec::{Decode, Encode};

#[derive(Debug, Clone, Decode, Encode)]
pub enum RuntimeError {
    #[codec(index = 0)]
    DecodeReturnValueError,
    #[codec(index = 1)]
    ReadOnly,
    #[codec(index = 2)]
    MemoryAccessOutOfBounds,
    #[codec(index = 3)]
    KvStorageError(String),
    #[codec(index = 4)]
    HttpError(String),
    #[codec(index = 5)]
    TimerError(String),
    #[codec(index = 6)]
    TssError(String),
}

impl core::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::DecodeReturnValueError => write!(f, "Decode return value error"),
            RuntimeError::ReadOnly => write!(f, "Write is not allowed in Read-only mode"),
            RuntimeError::MemoryAccessOutOfBounds => write!(f, "Memory access out of bounds"),
            RuntimeError::KvStorageError(e) => write!(f, "Kv storage error: {}", e),
            RuntimeError::HttpError(e) => write!(f, "Http error: {}", e),
            RuntimeError::TimerError(e) => write!(f, "Timer error: {}", e),
            RuntimeError::TssError(e) => write!(f, "Tss error: {}", e),
        }
    }
}
