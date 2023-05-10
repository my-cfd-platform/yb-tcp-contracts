use std::sync::atomic::AtomicU64;

use chrono::Utc;
use my_tcp_sockets::{
    socket_reader::{ReadBuffer, ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};
use rust_fix::{FixMessageBuilder, FIX_DELIMETR, FIX_EQUALS};

use crate::{
    FixIncomeMessage, FixLogonCredentials, FixMessage, FixOutcomeMessage, PlaceOrderYbTcpContract,
};
const FIX_DELIMETR_AS_ARR: [u8; 1] = [FIX_DELIMETR];
pub struct YourBourseFixTcpSerializer {
    message_counter: AtomicU64,
    auth_credentials: FixLogonCredentials,
    buffer: ReadBuffer,
}

impl YourBourseFixTcpSerializer {
    pub fn new(auth_credentials: FixLogonCredentials) -> Self {
        Self {
            message_counter: AtomicU64::new(1),
            auth_credentials: auth_credentials.to_owned(),
            buffer: ReadBuffer::new(2048 * 24),
        }
    }

    pub fn serialize_logon(
        &self,
        password: &str,
        sender_comp_id: &str,
        target_comp_id: &str,
    ) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();
        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "A");
        fix_builder.with_value(49, sender_comp_id);
        fix_builder.with_value(56, target_comp_id);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(34, count.to_string().as_str());
        fix_builder.with_value(108, "30");
        fix_builder.with_value(141, "Y");
        fix_builder.with_value(554, password);
        fix_builder.with_value(98, "0");

        return fix_builder;
    }

    pub fn serialize_place_order_contract(
        &self,
        contract: &PlaceOrderYbTcpContract,
    ) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();
        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "D");

        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        fix_builder.with_value(49, &self.auth_credentials.sender);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(56, &self.auth_credentials.target);
        fix_builder.with_value(34, count.to_string().as_str());
        //ClOrdID
        fix_builder.with_value(11, &contract.id);
        //HandIlnst
        fix_builder.with_value(21, "1");
        //Symbol
        fix_builder.with_value(55, &contract.symbol);
        //Side
        fix_builder.with_value(54, &(contract.side.clone() as i32).to_string());
        //OrderQty
        fix_builder.with_value(38, contract.qty.to_string().as_str());
        //OrdType - market
        fix_builder.with_value(40, "1");
        //TimeInForce - IOC
        fix_builder.with_value(59, "3");
        //TransactTime
        fix_builder.with_value(60, &date_string);

        return fix_builder;
    }

    pub fn serialize_instrument_subscribe(
        &self,
        instrument: &String,
        sender_comp_id: &str,
        target_comp_id: &str,
    ) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();
        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "V");
        let uuid = chrono::Utc::now().timestamp_nanos().to_string();

        fix_builder.with_value(49, sender_comp_id);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(56, target_comp_id);
        fix_builder.with_value(34, count.to_string().as_str());
        //MDReqID - can be just a symbol name
        fix_builder.with_value(262, &uuid.to_string());
        //SubscriptionRequestType 1 = Snapshot + Updates
        fix_builder.with_value(263, "1");
        //Market Depth 1 = Top of Book
        fix_builder.with_value(264, "1");
        //MDUpdateType
        fix_builder.with_value(265, "0");
        //NoMDEntryTypes
        fix_builder.with_value(267, "2");
        //Bid
        fix_builder.with_value(269, "0");
        //Ask
        fix_builder.with_value(269, "1");
        //NoRelatedSym
        fix_builder.with_value(146, "1");
        //Symbol
        fix_builder.with_value(55, instrument);

        return fix_builder;
    }

    pub fn serialize_ping(&self, sender_comp_id: &str, target_comp_id: &str) -> FixMessageBuilder {
        let date = Utc::now();
        let date_string = date.format("%Y%m%d-%H:%M:%S.%3f").to_string();

        let count = self
            .message_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut fix_builder = FixMessageBuilder::new("FIX.4.4", "0");
        fix_builder.with_value(49, sender_comp_id);
        fix_builder.with_value(52, date_string.as_str());
        fix_builder.with_value(56, target_comp_id);
        fix_builder.with_value(34, count.to_string().as_str());

        return fix_builder;
    }
}

#[async_trait::async_trait]
impl TcpSocketSerializer<FixMessage> for YourBourseFixTcpSerializer {
    const PING_PACKET_IS_SINGLETONE: bool = false;

    fn serialize(&self, contract: FixMessage) -> Vec<u8> {
        let FixLogonCredentials {
            password,
            sender,
            target,
        } = &self.auth_credentials;

        let fix_message = match contract {
            FixMessage::Income(_) => panic!("cant serialize outcome, only for income"),
            FixMessage::Outcome(message) => match message {
                crate::FixOutcomeMessage::InstrumentsSubscribe(instrument) => {
                    self.serialize_instrument_subscribe(&instrument, &sender, &target)
                }
                crate::FixOutcomeMessage::Logon => {
                    self.serialize_logon(&password, &sender, &target)
                }
                crate::FixOutcomeMessage::Ping => todo!(),
                crate::FixOutcomeMessage::PlaceOrder(contract) => {
                    self.serialize_place_order_contract(&contract)
                }
            },
        };

        fix_message.as_bytes()
    }

    fn serialize_ref(&self, contract: &FixMessage) -> Vec<u8> {
        let FixLogonCredentials {
            password,
            sender,
            target,
        } = &self.auth_credentials;

        let fix_message = match contract {
            FixMessage::Income(_) => panic!("cant serialize outcome, only for income"),
            FixMessage::Outcome(message) => match message {
                crate::FixOutcomeMessage::InstrumentsSubscribe(instrument) => {
                    self.serialize_instrument_subscribe(&instrument, &sender, &target)
                }
                crate::FixOutcomeMessage::Logon => {
                    self.serialize_logon(&password, &sender, &target)
                }
                crate::FixOutcomeMessage::Ping => todo!(),
                crate::FixOutcomeMessage::PlaceOrder(contract) => {
                    self.serialize_place_order_contract(contract)
                }
            },
        };

        fix_message.as_bytes()
    }

    fn get_ping(&self) -> FixMessage {
        return FixMessage::Outcome(FixOutcomeMessage::Ping);
    }

    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<FixMessage, ReadingTcpContractFail> {
        let mut result = vec![];
        loop {
            let chunk = socket_reader
                .read_until_end_marker(&mut self.buffer, &FIX_DELIMETR_AS_ARR.as_slice())
                .await;
            match chunk {
                Ok(res) => {
                    let equals_index = res.iter().position(|x| x == &FIX_EQUALS);
                    //sometimes panics here
                    if equals_index == None {
                        panic!("Not found equals sign during deserialization fix chunk")
                    }

                    let equals_index = equals_index.unwrap();
                    let key = String::from_utf8(res[0..equals_index].to_vec()).unwrap();
                    result.extend_from_slice(res);
                    if key == "10".to_string() {
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
    }

    fn apply_packet(&mut self, _: &FixMessage) -> bool{
        true
    }
}
