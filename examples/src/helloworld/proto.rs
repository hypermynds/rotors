use rotors::rotors;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HelloReply {
    pub message: String,
}

rotors! {
    package helloworld;

    service Greeter {
        rpc SayHello (super::HelloRequest) returns (super::HelloReply);
    }
}
