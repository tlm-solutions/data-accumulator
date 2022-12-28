use crate::filter::Filter;
use crate::DataPipelineReceiverR09;

use log::info;
use std::env;

use tlms::telegrams::r09::{R09GrpcTelegram, ReceivesTelegramsClient};

use log::warn;

pub struct ProcessorGrpc {
    grpc_hosts: Vec<String>,
    receiver_grpc: DataPipelineReceiverR09,
    filter: Filter,
}

impl ProcessorGrpc {
    pub fn new(receiver_grpc: DataPipelineReceiverR09) -> ProcessorGrpc {
        let mut grpc_hosts = Vec::new();

        for (k, v) in env::vars() {
            if k.starts_with("GRPC_HOST_") {
                grpc_hosts.push(v);
            }
        }

        ProcessorGrpc {
            grpc_hosts,
            receiver_grpc,
            filter: Filter::new(),
        }
    }

    pub async fn process_grpc(&mut self) {
        loop {
            let (telegram, meta) = self.receiver_grpc.recv().unwrap();
            let contained = self.filter.deduplicate(&telegram);
            info!(
                "[ProcessorGrpc] post: queue size: {}",
                self.receiver_grpc.try_iter().count()
            );

            // is filtered out because was already transmitted shortly before
            if contained.await {
                continue;
            }

            //TODO: optimize
            for grpc_host in self.grpc_hosts.clone().into_iter() {
                let grpc_host_copy = grpc_host.clone();
                match ReceivesTelegramsClient::connect(grpc_host).await {
                    Ok(mut client) => {
                        let request = tonic::Request::new(R09GrpcTelegram::from(
                            telegram.clone(),
                            meta.clone(),
                        ));
                        match client.receive_r09(request).await {
                            Err(e) => {
                                warn!("[ProcessorGrpc] Error while sending: {:?}", e);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        warn!(
                            "[ProcessorGrpc] Cannot connect to GRPC Host: {} with error {:?}",
                            grpc_host_copy, &e
                        );
                    }
                };
            }
        }
    }
}
