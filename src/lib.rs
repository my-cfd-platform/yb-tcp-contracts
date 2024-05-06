pub mod deserialize;
mod tcp_messages;
pub use tcp_messages::*;
pub mod tcp_serializer;
mod tcp_state;
use my_tcp_sockets::tcp_connection::TcpSocketConnection;
use tcp_serializer::YourBourseFixTcpSerializer;
pub use tcp_state::*;

pub mod serialize;
pub type YbTcpSocketConnection =
    TcpSocketConnection<tcp_messages::FixMessage, YourBourseFixTcpSerializer, YbTcpSate>;

pub const OUR_FIX_VERSION: &'static str = "FIX.4.4";
pub const FIX_DELIMITER: u8 = 0x1;
pub mod date_utils;

const FIX_DELIMITER_AS_ARR: [u8; 1] = [FIX_DELIMITER];

pub struct YbFixSettings {
    pub url: String,
    pub password: String,
    pub sender_company_id: String,
    pub target_company_id: String,
}
