use crate::error::RuntimeError;
use std::time::Duration;

use constant::{MAX_DELAY_SEC, MAX_FUNC_SIZE, MAX_PARAMS_SIZE};

mod constant {
    pub const MAX_DELAY_SEC: u64 = 60 * 60 * 24 * 365;
    pub const MAX_PARAMS_SIZE: usize = 1024 * 1024;
    pub const MAX_FUNC_SIZE: usize = 1024;
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn timer_set_delay(
        delay: i32,
        func_ptr: *const u8,
        func_len: i32,
        params_ptr: *const u8,
        params_len: i32,
    ) -> i32;

    fn now_timestamp() -> i32;
}

pub fn now() -> i32 {
    unsafe { now_timestamp() }
}

pub fn _set_timer(ts: Duration, func: &[u8], params: &[u8]) -> Result<(), RuntimeError> {
    if params.len() > MAX_PARAMS_SIZE {
        return Err(RuntimeError::TimerError(
            "params size exceeds maximum allowed size".to_string(),
        ));
    }
    if ts.as_secs() > MAX_DELAY_SEC {
        return Err(RuntimeError::TimerError(
            "delay exceeds maximum allowed size".to_string(),
        ));
    }
    if func.len() > MAX_FUNC_SIZE {
        return Err(RuntimeError::TimerError(
            "func size exceeds maximum allowed size".to_string(),
        ));
    }
    let status = unsafe {
        timer_set_delay(
            ts.as_secs() as i32,
            func.as_ptr(),
            func.len() as i32,
            params.as_ptr(),
            params.len() as i32,
        )
    };
    if status != 0 {
        Err(RuntimeError::TimerError("timer queue is full".to_string()))
    } else {
        Ok(())
    }
}

#[macro_export]
macro_rules! set_timer {
    ($duration:expr, $func_call:ident, $($param:expr,)*) => {{
        let __duration: std::time::Duration = $duration;
        let __func_name_bytes = stringify!($func_call);
        let __func_bytes = __func_name_bytes.as_bytes();
        let __param = ($($param,)*);
        let __params_bytes = ::vrs_core_sdk::codec::Encode::encode(&__param);
        ::vrs_core_sdk::timer::_set_timer(__duration, __func_bytes, __params_bytes.as_slice())
    }};
    ($duration:expr, $func_call:ident, $($param:expr),*) => {{
        let __duration: std::time::Duration = $duration;
        let __func_name_bytes = stringify!($func_call);
        let __func_bytes = __func_name_bytes.as_bytes();
        let __param = ($($param,)*);
        let __params_bytes = ::vrs_core_sdk::codec::Encode::encode(&__param);
        ::vrs_core_sdk::timer::_set_timer(__duration, __func_bytes, __params_bytes.as_slice())
    }};
    ($duration:expr, $func_call:ident) => {{
        let __duration: std::time::Duration = $duration;
        let __func_name_bytes = stringify!($func_call);
        let __func_bytes = __func_name_bytes.as_bytes();
        let __params_bytes = ::vrs_core_sdk::codec::Encode::encode(&());
        ::vrs_core_sdk::timer::_set_timer(__duration, __func_bytes, __params_bytes.as_slice())
    }};
}
