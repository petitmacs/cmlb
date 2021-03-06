use std::io::prelude::*;
use std::net::TcpStream;
use std::rc::Rc;
use std::time::Duration;

use crate::io::read_line;
use crate::server::config::relay_connection_info;
use crate::server::http_request::{http_request_info, read_http_request};
use crate::server::http_response::{http_response_first_line, http_response_header, http_response_info, read_header};

pub struct Upstream {
    relay: Rc<relay_connection_info>,
    request: Rc<http_request_info>,
    pub stream: TcpStream,
}

impl Upstream {
    pub fn new(relay: Rc<relay_connection_info>, request: Rc<http_request_info>) -> Option<Upstream> {
        let result: std::io::Result<TcpStream> = relay.connect_relay();
        if result.is_err() {
            return None;
        }
        let stream = result.unwrap();
        let upstream = Upstream {
            relay,
            request,
            stream,
        };
        return Some(upstream);
    }

    pub fn send_first_line(&self) {
        let mut stream = &self.stream;
        stream.write(self.request.http_first_line.method.as_bytes()).unwrap();
        stream.write(b" ").unwrap();
        stream.write(self.request.http_first_line.uri.as_bytes()).unwrap();
        stream.write(b" ").unwrap();
        stream.write(self.request.http_first_line.protool_version.as_bytes()).unwrap();
        stream.write(b"\r\n").unwrap();

        log::trace!("{}", self.request.http_first_line.method);
        log::trace!("{}", self.request.http_first_line.uri);
        log::trace!("{}", self.request.http_first_line.protool_version);
    }

    pub fn send_headers(&self) {
        let mut stream = &self.stream;
        let a = self.request.clone();
        let request = &a;

        //Host
        if self.relay.host.is_empty() == false {
            stream.write(b"Host: ").unwrap();
            //stream.write(self.relay.host.as_bytes());
            stream.write(request.http_request_header.host.as_bytes()).unwrap();
            stream.write(b"\r\n").unwrap();
            log::debug!("host:{}",request.http_request_header.host);
            log::trace!("end send host.")
        }
        //Connection
        if request.http_request_header.keep_alive {}
        if request.http_request_header.content_length > 0 {
            stream.write(b"Content-Length: ").unwrap();
            stream.write(request.http_request_header.content_length.to_string().as_bytes()).unwrap();
            stream.write(b"\r\n").unwrap();
        }
        //ヘッダー
        let headers = &a.http_request_header.headers;
        for header in headers {
            let name = &header.name;
            let value = &header.value;
            stream.write(name.as_bytes()).unwrap();
            stream.write(b": ").unwrap();
            stream.write(value.as_bytes()).unwrap();
            stream.write(b"\r\n").unwrap();
        }
        stream.write(b"\r\n").unwrap();
        log::trace!("end send header.")
    }

    pub fn send_body(&self, reader: &mut Read) {
        let mut unsend_data_length = self.request.http_request_header.content_length;
        let mut buf = [0; 4096];
        while unsend_data_length > 0 {
            let size = reader.read(&mut buf).unwrap();
            let d = size.to_string();
            let data_length: i64 = d.parse().unwrap();
            self.send(&buf[0..size]);
            log::trace!("request {} bytes",d);
            unsend_data_length = unsend_data_length - data_length;
        }
    }


    pub fn send(&self, buf: &[u8]) {
        let mut stream = &self.stream;
        stream.write(buf).unwrap();
    }
    pub fn flush(&self) {
        let mut stream = &self.stream;
        stream.flush().unwrap();
    }


    pub fn read_http_response_info(&mut self) -> std::io::Result<http_response_info> {
        let mut read = &self.stream;
        return crate::server::http_response::read_http_response_info(&mut read);
    }
}