use core::ffi::*;
use core::mem;
use core::primitive::u64;
const TA_FLAGS: u32 = 0u32;
const TA_DATA_SIZE: u32 = 1048576u32;
const TA_STACK_SIZE: u32 = 2048u32;
const TA_VERSION: &[u8] = b"0.1.0\0";
const TA_DESCRIPTION: &[u8] = b"test_before_2024\0";
#[no_mangle]
pub static mut trace_level: c_int = 4i32;
#[no_mangle]
pub static trace_ext_prefix: &[u8] = b"TA\0";
#[no_mangle]
pub unsafe extern "C" fn tahead_get_trace_level() -> c_int {
    unsafe {
        return trace_level;
    }
}
const EXT_PROP_VALUE_1: &[u8] = b"test before 2024\0";
const EXT_PROP_VALUE_2: u32 = 16u32;
static FLAG_BOOL: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_SINGLE_INSTANCE) != 0;
static FLAG_MULTI: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_MULTI_SESSION) != 0;
static FLAG_INSTANCE: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_INSTANCE_KEEP_ALIVE)
    != 0;
#[no_mangle]
pub static ta_num_props: usize = 9usize;
#[no_mangle]
pub static ta_props: [optee_utee_sys::user_ta_property; 9usize] = [
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_SINGLE_INSTANCE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &FLAG_BOOL as *const bool as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_MULTI_SESSION,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &FLAG_MULTI as *const bool as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_KEEP_ALIVE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &FLAG_INSTANCE as *const bool as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_DATA_SIZE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_DATA_SIZE as *const u32 as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_STACK_SIZE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_STACK_SIZE as *const u32 as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_VERSION,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: TA_VERSION as *const [u8] as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_DESCRIPTION,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: TA_DESCRIPTION as *const [u8] as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: b"gp.ta.description\0".as_ptr(),
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: EXT_PROP_VALUE_1 as *const [u8] as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: b"gp.ta.version\0".as_ptr(),
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &EXT_PROP_VALUE_2 as *const u32 as *mut _,
    },
];
#[no_mangle]
#[link_section = ".ta_head"]
pub static ta_head: optee_utee_sys::ta_head = optee_utee_sys::ta_head {
    uuid: optee_utee_sys::TEE_UUID {
        timeLow: 642817260u32,
        timeMid: 18987u16,
        timeHiAndVersion: 18741u16,
        clockSeqAndNode: [135u8, 171u8, 118u8, 45u8, 137u8, 251u8, 240u8, 176u8],
    },
    stack_size: 4096u32,
    flags: TA_FLAGS,
    depr_entry: u64::MAX,
};
#[no_mangle]
#[link_section = ".bss"]
pub static ta_heap: [u8; TA_DATA_SIZE as usize] = [0; TA_DATA_SIZE as usize];
#[no_mangle]
pub static ta_heap_size: usize = mem::size_of::<u8>() * TA_DATA_SIZE as usize;
