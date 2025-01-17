// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![no_std]
#![no_main]

mod http;
extern crate alloc;

use core::str::FromStr;

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Command;

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destory");
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let mut param0 = unsafe { params.0.as_memref()? }; // for url
    let mut param1 = unsafe { params.1.as_memref()? }; // for headers
    let mut param2 = unsafe { params.2.as_memref()? }; // for request body
    let mut param3 = unsafe { params.3.as_memref()? }; // for output

    let url = core::str::from_utf8(param0.buffer()).map_err(|err| {
        optee_utee::trace_println!("invalid str at param0: {:?}", err);
        ErrorKind::BadParameters
    })?;
    let http_url = http_io::url::HttpUrl::from_str(url).map_err(|err| {
        optee_utee::trace_println!("invalid url at param0: {:?}", err);
        ErrorKind::BadParameters
    })?;
    let headers: http::Headers = serde_json::from_slice(param1.buffer()).map_err(|err| {
        optee_utee::trace_println!("invalid headers at param1: {:?}", err);
        ErrorKind::BadParameters
    })?;

    let result = match Command::from(cmd_id) {
        Command::GET => http::get(http_url, headers)?,
        Command::DELETE => http::delete(http_url, headers)?,
        Command::POST => http::post(http_url, headers, param2.buffer())?,
        Command::PUT => http::put(http_url, headers, param2.buffer())?,
        Command::Unknown => return Err(ErrorKind::BadParameters.into()),
    };

    let buffer = param3.buffer();
    assert!(buffer.len() >= result.len());
    buffer[0..result.len()].copy_from_slice(&result);
    param3.set_updated_size(result.len());

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));

/// Workaround for those rustc bugs:
/// * https://github.com/rust-lang/rust/issues/47493
/// * https://github.com/rust-lang/rust/issues/56152
///
/// It shouldn't even be possible to reach this function, thanks to panic=abort,
/// but libcore is compiled with unwinding enabled and that ends up making
/// unreachable references to this.
#[cfg(not(target_os = "optee"))]
#[no_mangle]
extern "C" fn _Unwind_Resume() -> ! {
    unreachable!("Unwinding not supported");
}

#[cfg(not(target_os = "optee"))]
#[no_mangle]
extern "C" fn rust_eh_personality() -> ! {
    unreachable!("Unwinding not supported");
}
