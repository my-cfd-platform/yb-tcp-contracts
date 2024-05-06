use my_tcp_sockets::TcpSerializerState;

use crate::{tcp_messages::FixMessage, YbFixSettings};

pub struct YbTcpSate {
    pub settings: YbFixSettings,
}

impl YbTcpSate {
    pub fn new(settings: YbFixSettings) -> Self {
        Self { settings }
    }

    pub fn get_settings(&self) -> &YbFixSettings {
        &self.settings
    }
}

impl TcpSerializerState<FixMessage> for YbTcpSate {
    fn is_tcp_contract_related_to_metadata(&self, _: &FixMessage) -> bool {
        false
    }
    fn apply_tcp_contract(&mut self, _: &FixMessage) {}
}
