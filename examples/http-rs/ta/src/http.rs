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

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use http_io::client::HttpRequestBuilder;
use http_io::error::Error;
use http_io::protocol::HttpMethod;
use http_io::url::HttpUrl;
use optee_utee::net;

struct TcpStream(net::TcpStream);
pub type Headers = hashbrown::HashMap<String, String>;

impl TcpStream {
    fn new(url: &HttpUrl) -> http_io::error::Result<Self> {
        let setup = net::Setup::new_v4(url.host(), url.port()).map_err(|err| Error::UrlError(err.to_string()))?;
        match net::TcpStream::open(setup) {
            Ok(mut v) => {
                v.set_recv_timeout_in_milli(1000);
                v.set_send_timeout_in_milli(1000);
                Ok(Self(v))
            },
            Err(err) => Err(Error::Other(err.to_string()))
        }
    }
}

impl http_io::io::Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> http_io::io::Result<usize> {
        Ok(self.0.send(buf).map_err(|err| Error::Other(err.to_string()))?)
    }
    fn flush(&mut self) -> http_io::io::Result<()> {
        Ok(())
    }
}

impl http_io::io::Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> http_io::io::Result<usize> {
        Ok(self.0.recv(buf).map_err(|err| Error::Other(err.to_string()))?)
    }
}

fn perform_request(
    url: HttpUrl,
    method: HttpMethod,
    headers: Headers,
    body: Option<&[u8]>,
) -> http_io::error::Result<Vec<u8>> {
    use http_io::io::{Write, Read};

    let stream = TcpStream::new(&url)?;
    let mut req = HttpRequestBuilder::new(url, method)?;
    for (key, value) in headers.iter() {
        req = req.add_header(key, value)
    }

    let mut body_writer = req.send(stream)?;

    if body.is_some() {
        body_writer.write_all(body.unwrap())?;
    }

    let mut response = body_writer.finish()?;
    let mut result = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let size = response.body.read(&mut buffer)?;
        result.extend_from_slice(&buffer[0..size]);
        if size < buffer.len() {
            break;
        }
    }
    Ok(result)
}

pub fn get(url: HttpUrl, headers: Headers) -> optee_utee::Result<Vec<u8>> {
    Ok(perform_request(url, HttpMethod::Get, headers, Option::None).map_err(|err| {
        optee_utee::trace_println!("failed to get due to {:?}", err);
        optee_utee::ErrorKind::Generic
    })?)
}

pub fn post(url: HttpUrl, headers: Headers, body: &[u8]) -> optee_utee::Result<Vec<u8>> {
    Ok(perform_request(url, HttpMethod::Post, headers, Option::Some(body)).map_err(|err| {
        optee_utee::trace_println!("failed to post due to {:?}", err);
        optee_utee::ErrorKind::Generic
    })?)
}

pub fn put(url: HttpUrl, headers: Headers, body: &[u8]) -> optee_utee::Result<Vec<u8>> {
    Ok(perform_request(url, HttpMethod::Put, headers, Option::Some(body)).map_err(|err| {
        optee_utee::trace_println!("failed to put due to {:?}", err);
        optee_utee::ErrorKind::Generic
    })?)
}

pub fn delete(url: HttpUrl, headers: Headers) -> optee_utee::Result<Vec<u8>> {
    Ok(perform_request(url, HttpMethod::Delete, headers, Option::None).map_err(|err| {
        optee_utee::trace_println!("failed to delete due to {:?}", err);
        optee_utee::ErrorKind::Generic
    })?)
}
