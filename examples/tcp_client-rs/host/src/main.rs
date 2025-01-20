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

use std::net::{SocketAddr, ToSocketAddrs};

use optee_teec::{Context, Operation, ParamType, Session, Uuid};
use optee_teec::{ParamNone, ParamTmpRef, ParamValue};
use proto::{Command, IpVersion, UUID};

fn tcp_client(
    session: &mut Session,
    address: &str,
    port: u16,
    ip_version: IpVersion,
    host_name: &str,
) -> optee_teec::Result<()> {
    let http_data = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", host_name);
    let mut operation = Operation::new(
        0,
        ParamTmpRef::new_input(address.as_bytes()),
        ParamValue::new(port as u32, ip_version as u32, ParamType::ValueInput),
        ParamTmpRef::new_input(http_data.as_bytes()),
        ParamNone,
    );
    session.invoke_command(Command::Start as u32, &mut operation)?;
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID).unwrap();
    let mut session = ctx.open_session(uuid)?;

    // test ipv4
    const IPV4_HOST: &str = "teaclave.apache.org";
    tcp_client(&mut session, IPV4_HOST, 80, IpVersion::V4, IPV4_HOST)?;
    // test ipv6
    const IPV6_HOST: &str = "ipv6.google.com";
    let addr = resolve_ipv6_addr(format!("{}:80", IPV6_HOST).as_str())?;
    tcp_client(
        &mut session,
        addr.ip().to_string().as_str(),
        addr.port(),
        IpVersion::V6,
        IPV6_HOST,
    )?;

    println!("Success");
    Ok(())
}

// making sure we get an ipv6 address
fn resolve_ipv6_addr(domain: &str) -> optee_teec::Result<SocketAddr> {
    for addr in domain.to_socket_addrs().unwrap() {
        if addr.is_ipv6() {
            return Ok(addr.clone());
        }
    }
    Err(optee_teec::ErrorKind::BadState.into())
}
