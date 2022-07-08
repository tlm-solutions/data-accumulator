use super::DataPipelineReceiver;
use std::env;

use telegrams::{R09GrpcTelegram, ReceivesTelegramsClient};

pub struct ProcessorGrpc {
    grpc_hosts: Vec<String>,
    receiver_grpc: DataPipelineReceiver,
}

impl ProcessorGrpc {
    pub fn new(receiver_grpc: DataPipelineReceiver) -> ProcessorGrpc {
        let mut grpc_hosts = Vec::new();

        for (k, v) in env::vars() {
            if k.starts_with("GRPC_HOST_") {
                grpc_hosts.push(v);
            }
        }

        ProcessorGrpc {
            grpc_hosts: grpc_hosts,
            receiver_grpc: receiver_grpc,
        }
    }

    pub async fn process_grpc(&mut self) {
        loop {
            let (telegram, meta) = self.receiver_grpc.recv().unwrap();
            println!(
                "[ProcessorGrpc] post: queue size: {}",
                self.receiver_grpc.try_iter().count()
            );

            //TODO: optimize
            for grpc_host in self.grpc_hosts.clone().into_iter() {
                match ReceivesTelegramsClient::connect(grpc_host).await {
                    Ok(mut client) => {
                        let request = tonic::Request::new(R09GrpcTelegram::from(
                            telegram.clone(),
                            meta.clone(),
                        ));
                        match client.receive_r09(request).await {
                            Err(e) => {
                                println!("[ProcessorGrpc] Error while sending: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        //println!("[ProcessorGrpc] Cannot connect to GRPC Host: {}", &grpc_host);
                        //stdout().flush();
                    }
                };
            }
        }
    }
}
