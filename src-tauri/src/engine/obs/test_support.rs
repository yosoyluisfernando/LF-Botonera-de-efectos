use super::transport::Session;
use crate::model::obs::ObsConfig;
use serde_json::{json, Value};
use std::{net::TcpListener, thread::JoinHandle};
use tungstenite::Message;

type Step = (&'static str, Value, Result<Value, u64>);
pub(super) fn mock(steps: Vec<Step>) -> (Session, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        socket
            .send(Message::Text(
                json!({"op":0,"d":{"rpcVersion":1}}).to_string().into(),
            ))
            .unwrap();
        socket.read().unwrap();
        socket
            .send(Message::Text(
                json!({"op":2,"d":{"negotiatedRpcVersion":1}})
                    .to_string()
                    .into(),
            ))
            .unwrap();
        for (kind, data, reply) in steps {
            let message = socket.read().unwrap();
            let request: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
            assert_eq!(request["d"]["requestType"], kind);
            assert_eq!(request["d"]["requestData"], data);
            let (result, code, response) = match reply {
                Ok(v) => (true, 100, v),
                Err(code) => (false, code, Value::Null),
            };
            socket.send(Message::Text(json!({"op":7,"d":{
                "requestId":request["d"]["requestId"],"requestStatus":{"result":result,"code":code},
                "responseData":response
            }}).to_string().into())).unwrap();
        }
    });
    let session = Session::connect(
        &ObsConfig {
            port,
            ..Default::default()
        },
        "",
    )
    .unwrap();
    (session, server)
}
