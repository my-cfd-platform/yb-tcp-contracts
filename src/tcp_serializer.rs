use std::sync::atomic::AtomicU64;

use my_tcp_sockets::{
    socket_reader::{ReadBuffer, ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer, TcpWriteBuffer,
};
use rust_fix::FixMessageItem;

use crate::{tcp_messages::*, YbTcpSate, FIX_DELIMITER_AS_ARR};

pub struct YourBourseFixTcpSerializer {
    message_counter: AtomicU64,
    buffer: ReadBuffer,
}

impl YourBourseFixTcpSerializer {
    pub fn new() -> Self {
        Self {
            message_counter: AtomicU64::new(1),
            buffer: ReadBuffer::new(2048 * 24),
        }
    }

    fn get_next_message_id(&self) -> u64 {
        self.message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    async fn receive_fix_payload(
        &mut self,
        socket_reader: &mut impl SocketReader,
    ) -> Result<Vec<u8>, ReadingTcpContractFail> {
        let mut result = vec![];
        loop {
            let chunk = socket_reader
                .read_until_end_marker(&mut self.buffer, &FIX_DELIMITER_AS_ARR.as_slice())
                .await;
            match chunk {
                Ok(res) => {
                    result.extend_from_slice(res);
                    let item = FixMessageItem::from_slice(res);
                    if item.key == "10".to_string() {
                        break;
                    }
                }
                Err(err) => {
                    println!("Err: {:?}", err);
                    break;
                }
            };
        }

        if result.len() == 0 {
            return Err(ReadingTcpContractFail::ErrorReadingSize);
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl TcpSocketSerializer<FixMessage, YbTcpSate> for YourBourseFixTcpSerializer {
    fn serialize(&self, out: &mut impl TcpWriteBuffer, contract: &FixMessage, state: &YbTcpSate) {
        let fix_message_writer = match contract {
            FixMessage::InstrumentSubscribe(instrument) => crate::serialize::instrument_subscribe(
                &state.settings,
                instrument,
                self.get_next_message_id(),
            ),
            FixMessage::Logon => {
                crate::serialize::logon(&state.settings, self.get_next_message_id())
            }
            FixMessage::Ping => crate::serialize::ping(&state.settings, self.get_next_message_id()),
            FixMessage::PlaceOrder(contract) => crate::serialize::place_order_contract(
                &state.settings,
                contract,
                self.get_next_message_id(),
            ),
            _ => {
                panic!(
                    "Contract {} can not be used as outgoing fix message",
                    contract.to_string()
                );
            }
        };

        out.write_slice(fix_message_writer.compile_message().as_slice());
    }

    fn get_ping(&self) -> FixMessage {
        return FixMessage::Ping;
    }

    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
        _state: &YbTcpSate,
    ) -> Result<FixMessage, ReadingTcpContractFail> {
        let fix_payload = self.receive_fix_payload(socket_reader).await?;

        let fix_message = FixMessage::from_slice(fix_payload.as_slice());

        return Ok(fix_message);

        /*
        match FixMessageBuilder::from_bytes(&result, false) {
            Ok(fix) => {
                let message_type = fix.get_message_type_as_string();

                let payload = match message_type.as_str() {
                    "A" => FixIncomeMessage::Logon(fix),
                    "W" => FixIncomeMessage::MarketData(fix),
                    "Y" => FixIncomeMessage::MarketDataReject(fix),
                    "3" => FixIncomeMessage::Reject(fix),
                    "5" => FixIncomeMessage::Logout(fix),
                    "8" => FixIncomeMessage::ExecutionReport(fix),
                    _ => FixIncomeMessage::Others(fix),
                };
                return Ok(FixMessage::Income(payload));
            }
            Err(err) => {
                panic!("Fix serialization error: {:?}", err)
            }
        };
         */
    }
}
