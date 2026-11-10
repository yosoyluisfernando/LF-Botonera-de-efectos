//! HTTP acotado: una petición por conexión, sin chunking, proxy ni acceso al disco.
use std::{
    io::{Read, Write},
    net::SocketAddr,
};
pub struct Request {
    pub method: String,
    pub path: String,
    pub authorization: String,
    pub body: Vec<u8>,
}
pub fn read(input: &mut impl Read, address: SocketAddr) -> Result<Request, ()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    let mut head = Vec::new();
    let mut byte = [0];
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() >= 8192
            || std::time::Instant::now() >= deadline
            || input.read(&mut byte).map_err(|_| ())? != 1
        {
            return Err(());
        }
        head.push(byte[0]);
    }
    let mut headers = [httparse::EMPTY_HEADER; 48];
    let mut parsed = httparse::Request::new(&mut headers);
    if !parsed.parse(&head).map_err(|_| ())?.is_complete() || parsed.version != Some(1) {
        return Err(());
    }
    let method = parsed.method.ok_or(())?.to_string();
    let path = parsed.path.ok_or(())?.to_string();
    let mut authorization = None;
    let mut length = None;
    let mut host = None;
    let mut origin = None;
    for header in parsed.headers {
        let value = std::str::from_utf8(header.value).map_err(|_| ())?.trim();
        match header.name.to_ascii_lowercase().as_str() {
            "content-length" if length.is_none() && value.bytes().all(|b| b.is_ascii_digit()) => {
                length = Some(value.parse::<usize>().map_err(|_| ())?);
            }
            "authorization" if authorization.is_none() => authorization = Some(value.to_string()),
            "host" if host.is_none() => host = Some(value),
            "origin" if origin.is_none() => origin = Some(value),
            "content-length" | "authorization" | "host" | "origin" | "transfer-encoding" => {
                return Err(())
            }
            _ => (),
        }
    }
    let host = host.ok_or(())?;
    if host.parse::<SocketAddr>().ok() != Some(address)
        || origin.is_some_and(|value| value != format!("http://{address}"))
    {
        return Err(());
    }
    let length = length.unwrap_or(0);
    if length > 4096 {
        return Err(());
    }
    let mut body = vec![0; length];
    for item in &mut body {
        if std::time::Instant::now() >= deadline {
            return Err(());
        }
        input
            .read_exact(std::slice::from_mut(item))
            .map_err(|_| ())?;
    }
    Ok(Request {
        method,
        path,
        authorization: authorization.unwrap_or_default(),
        body,
    })
}
pub fn reply(out: &mut impl Write, code: u16, mime: &str, body: &[u8]) {
    let reason = match code {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Bad Request",
    };
    let _ = write!(out, "HTTP/1.1 {code} {reason}\r\nContent-Type: {mime}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'\r\n\r\n", body.len());
    let _ = out.write_all(body);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_browser_post() {
        let raw = b"POST /api HTTP/1.1\r\nHost: 192.168.1.3:17865\r\nContent-Length: 2\r\nAuthorization: Bearer abc\r\n\r\n{}";
        let request = read(&mut raw.as_slice(), "192.168.1.3:17865".parse().unwrap()).unwrap();
        assert_eq!(request.body, b"{}");
        assert_eq!(request.authorization, "Bearer abc");
    }
    #[test]
    fn rejects_smuggling_and_unbounded_input() {
        for headers in [
            "Content-Length: 5000",
            "Content-Length: 0\r\nContent-Length: 1",
            "Transfer-Encoding: chunked",
            "Host: evil.example",
            "Origin: http://192.168.1.2:17865",
            "Origin: null",
        ] {
            let raw = format!("POST /api HTTP/1.1\r\nHost: 127.0.0.1:17865\r\n{headers}\r\n\r\n");
            assert!(read(&mut raw.as_bytes(), "127.0.0.1:17865".parse().unwrap()).is_err());
        }
        assert!(read(
            &mut vec![b'x'; 9000].as_slice(),
            "127.0.0.1:17865".parse().unwrap()
        )
        .is_err());
    }
}
