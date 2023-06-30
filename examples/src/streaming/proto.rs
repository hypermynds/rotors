use rotors::rotors;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EchoResponse {
    pub message: String,
}

rotors! {
    package grpc.examples.echo;

    service Echo {
       rpc UnaryEcho(super::EchoRequest) returns (super::EchoResponse);
       rpc ServerStreamingEcho(super::EchoRequest) returns (stream super::EchoResponse);
       rpc ClientStreamingEcho(stream super::EchoRequest) returns (super::EchoResponse);
       rpc BidirectionalStreamingEcho(stream super::EchoRequest) returns (stream super::EchoResponse);
     }
}
