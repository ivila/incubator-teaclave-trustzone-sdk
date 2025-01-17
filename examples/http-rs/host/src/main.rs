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

use optee_teec::{Context, Operation, ParamTmpRef, Session, Uuid};
use proto::{Command, UUID};
use rand::{thread_rng, Rng};

type Headers = std::collections::HashMap<String, String>;

fn main() -> optee_teec::Result<()> {
    let server = httpmock::MockServer::start();
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID).unwrap();
    let mut session = ctx.open_session(uuid)?;
    test_get(&mut session, &server)?;
    test_post(&mut session, &server)?;
    test_put(&mut session, &server)?;
    test_delete(&mut session, &server)?;
    Ok(())
}

fn random_string(length: usize) -> String {
    use rand::{Rng, thread_rng};
    use rand::distributions::Alphanumeric;

    thread_rng().sample_iter(Alphanumeric).take(length).map(char::from).collect()
}

fn call(
    sess: &mut Session,
    cmd: Command,
    url: &str,
    headers: &Headers,
    body: &[u8],
) -> optee_teec::Result<Vec<u8>> {
    let header_data = serde_json::to_vec(&headers).unwrap();
    let mut output = vec![0_u8; 1024 * 1024];
    let size = {
        let mut operation = Operation::new(
            0,
            ParamTmpRef::new_input(url.as_bytes()),
            ParamTmpRef::new_input(&header_data),
            ParamTmpRef::new_input(body),
            ParamTmpRef::new_output(output.as_mut_slice()),
        );
        sess.invoke_command(cmd as u32, &mut operation)?;
        operation.parameters().3.updated_size()
    };
    output.resize(size, 0);
    Ok(output)
}

fn generate_random_headers() -> Headers {
    let count = thread_rng().gen::<usize>() % 9 + 1;
    let mut headers = Headers::new();
    for _ in 0..count {
        let key = random_string(10);
        let value = random_string(20);
        headers.insert(key, value);
    }
    headers
}

fn test(sess: &mut Session, server: &httpmock::MockServer, cmd: Command, method: httpmock::Method, body: &[u8]) -> optee_teec::Result<()> {
    let path = format!("/{}", random_string(20));
    let exp_result = random_string(30);
    let headers = generate_random_headers();
    let url = format!("http://{}:{}{}", server.host(), server.port(), &path);

    println!("perform request from TEE");
    println!("request method: {}", method);
    println!("request body size: {}", body.len());
    println!("request headers:");
    for (key, value) in headers.iter() {
        println!("Header: {}, Value: {}", key, value);
    }
    println!("exp result: {}", exp_result);

    let mock = server.mock(|mut when, then| {
        when = when.path(path).method(method);
        for (key, value) in headers.iter() {
            when = when.header(key, value);
        };
        then.status(200).body(exp_result.clone());
    });

    let result = call(sess, cmd, &url, &headers, &body)?;
    let s = core::str::from_utf8(&result).unwrap();
    mock.assert_hits(1);
    assert_eq!(s, exp_result);
    println!("pass\n");
    Ok(())
}

fn test_get(sess: &mut Session, server: &httpmock::MockServer) -> optee_teec::Result<()> {
    let body = [];
    test(sess, server, Command::GET, httpmock::Method::GET, &body)
}

fn test_post(sess: &mut Session, server: &httpmock::MockServer) -> optee_teec::Result<()> {
    let body = random_string(100);
    test(sess, server, Command::POST, httpmock::Method::POST, body.as_bytes())
}

fn test_put(sess: &mut Session, server: &httpmock::MockServer) -> optee_teec::Result<()> {
    let body = random_string(100);
    test(sess, server, Command::PUT, httpmock::Method::PUT, body.as_bytes())
}

fn test_delete(sess: &mut Session, server: &httpmock::MockServer) -> optee_teec::Result<()> {
    let body = [];
    test(sess, server, Command::DELETE, httpmock::Method::DELETE, &body)
}
