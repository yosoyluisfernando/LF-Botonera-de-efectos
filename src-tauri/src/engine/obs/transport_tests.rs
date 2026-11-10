use super::*;
use std::net::TcpListener;
#[test]
fn authentication_matches_independent_sha256_vector() {
    assert_eq!(
        authentication(
            "supersecretpassword",
            "lM1GncleQOaCu9lT1yeUZhFYnqhsLLP1G5lAGo3ixaI=",
            "+IxH4CnCiqpX1rM9scsNynZzbOe4KhDeYcTNS3PDaeY="
        ),
        "1Ct943GAT+6YQUUX47Ia/ncufilbe6+oD6lY+5kaCu4="
    );
}
#[test]
fn handshake_and_request_correlate_past_events() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        socket
            .send(Message::Text(
                json!({"op":0,"d":{"rpcVersion":1}}).to_string().into(),
            ))
            .unwrap();
        let identify: Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(identify["op"], 1);
        socket
            .send(Message::Text(
                json!({"op":2,"d":{"negotiatedRpcVersion":1}})
                    .to_string()
                    .into(),
            ))
            .unwrap();
        let request: Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        socket
            .send(Message::Text(
                json!({"op":5,"d":{"eventType":"RecordStateChanged"}})
                    .to_string()
                    .into(),
            ))
            .unwrap();
        socket.send(Message::Text(json!({"op":7,"d":{"requestId":request["d"]["requestId"],"requestStatus":{"result":true},"responseData":{"outputActive":true}}}).to_string().into())).unwrap();
    });
    let cfg = ObsConfig {
        port,
        ..Default::default()
    };
    let mut session = Session::connect(&cfg, "").unwrap();
    assert_eq!(
        session.request("GetRecordStatus", json!({})).unwrap()["outputActive"],
        true
    );
    server.join().unwrap();
}
