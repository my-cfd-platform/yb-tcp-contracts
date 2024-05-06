use my_tcp_sockets::TcpContract;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_fix::{FixMessageReader, FixSerializeError};

pub enum FixMessage {
    Logon,
    Reject,
    Logout,
    InstrumentSubscribe(String),
    MarketData(YbMarketData),
    MarketDataReject(String),
    ExecutionReport(ExecutionReportModel),
    PlaceOrder(PlaceOrderYbTcpContract),
    Others(String),
    Pong,
    Ping,
}

impl FixMessage {
    pub fn from_slice(src: &[u8]) -> Self {
        let fix_message_reader = FixMessageReader::from_bytes(src);

        if std::env::var("DEBUG_FIX").is_ok() {
            println!("In  Fix Message: {:?}", fix_message_reader.to_string());
        }

        match fix_message_reader.get_message_type().unwrap() {
            "A" => Self::Logon,
            "W" => {
                let model = crate::deserialize::deserialize_market_data(&fix_message_reader);
                match model {
                    Ok(model) => Self::MarketData(model),
                    Err(err) => Self::Others(format!(
                        "Error reading fix message: {}, Err: {}",
                        fix_message_reader.to_string(),
                        err
                    )),
                }
            }

            "Y" => Self::MarketDataReject(fix_message_reader.to_string()),
            "3" => Self::Reject,
            "5" => Self::Logout,
            "8" => Self::ExecutionReport(ExecutionReportModel::new(&fix_message_reader).unwrap()),
            _ => Self::Others(fix_message_reader.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Logon => "Logon".to_string(),
            Self::Reject => "Reject".to_string(),
            Self::Logout => "Logout".to_string(),
            Self::InstrumentSubscribe(src) => format!("InstrumentSubscribe: {}", src),
            Self::MarketData(model) => format!("MarketData: {:?}", model),
            Self::MarketDataReject(src) => format!("MarketDataReject: {}", src),
            Self::ExecutionReport(model) => format!("ExecutionReport: {:?}", model),
            Self::PlaceOrder(contract) => format!("PlaceOrder: {:?}", contract),
            Self::Others(src) => format!("Others: {}", src),
            Self::Pong => "Pong".to_string(),
            Self::Ping => "Ping".to_string(),
        }
    }
}

impl TcpContract for FixMessage {
    fn is_pong(&self) -> bool {
        match self {
            Self::Pong => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlaceOrderYbTcpContractSide {
    Buy = 1,
    Sell = 2,
}

impl PlaceOrderYbTcpContractSide {
    pub fn from_str(src: &str) -> Self {
        match src {
            "1" => Self::Buy,
            "2" => Self::Sell,
            _ => panic!("Unknown side"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct PlaceOrderYbTcpContract {
    pub id: String,
    pub symbol: String,
    pub side: PlaceOrderYbTcpContractSide,
    pub qty: f64,
}

#[derive(Debug, Clone)]
pub enum ExecutionReportModelStatus {
    PendingNew,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

impl ExecutionReportModelStatus {
    pub fn from_str(src: &str) -> Self {
        match src {
            "A" => Self::PendingNew,
            "1" => Self::PartiallyFilled,
            "2" => Self::Filled,
            "4" => Self::Canceled,
            "8" => Self::Rejected,
            _ => panic!("Unknown status"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionReportModelExecutionType {
    PendingNew,
    Canceled,
    Rejected,
    Trade,
}

impl ExecutionReportModelExecutionType {
    pub fn from_str(src: &str) -> Self {
        match src {
            "A" => Self::PendingNew,
            "F" => Self::Trade,
            "4" => Self::Canceled,
            "8" => Self::Rejected,
            _ => panic!("Unknown execution type"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

impl OrderType {
    pub fn from_str(src: &str) -> Self {
        match src {
            "1" => Self::Market,
            "2" => Self::Limit,
            _ => panic!("Unknown order type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionReportModel {
    pub internal_order_id: String,
    pub external_order_id: String,
    pub execute_id: String,
    pub ord_status: ExecutionReportModelStatus,
    pub execution_type: ExecutionReportModelExecutionType,
    pub symbol: String,
    pub side: PlaceOrderYbTcpContractSide,
    pub qty: f64,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub time_in_force: Option<String>,
    pub last_price: Option<f64>,
    pub avg_price: f64,
    pub trade_date: Option<String>,
    pub reject_reason: Option<String>,
    pub reject_text: Option<String>,
}

impl ExecutionReportModel {
    pub fn new(src: &FixMessageReader) -> Result<Self, FixSerializeError> {
        let price = match src.get_value("44")? {
            Some(src) => Some(src.parse::<f64>().unwrap()),
            None => None,
        };

        let last_price = match src.get_value("31")? {
            Some(src) => Some(src.parse::<f64>().unwrap()),
            None => None,
        };

        let trade_date = match src.get_value("75")? {
            Some(src) => Some(src.to_string()),
            None => None,
        };

        let reject_reason = match src.get_value("103")? {
            Some(src) => Some(src.to_string()),
            None => None,
        };

        let reject_text = match src.get_value("58")? {
            Some(src) => Some(src.to_string()),
            None => None,
        };

        let result = ExecutionReportModel {
            internal_order_id: src.get_value("11")?.unwrap().to_string(),
            external_order_id: src.get_value("37")?.unwrap().to_string(),
            execute_id: src.get_value("17")?.unwrap().to_string(),
            ord_status: ExecutionReportModelStatus::from_str(src.get_value("39")?.unwrap()),
            execution_type: ExecutionReportModelExecutionType::from_str(
                src.get_value("150")?.unwrap(),
            ),
            symbol: src.get_value("55")?.unwrap().to_string(),
            side: PlaceOrderYbTcpContractSide::from_str(src.get_value("54")?.unwrap()),
            qty: src.get_value("38")?.unwrap().parse().unwrap(),
            order_type: OrderType::from_str(src.get_value("40")?.unwrap()),
            price,
            time_in_force: src.get_value("59")?.map(|src| src.to_string()),
            last_price,
            avg_price: src.get_value("6")?.unwrap().parse().unwrap(),
            trade_date,
            reject_reason,
            reject_text,
        };

        Ok(result)
    }
}

/*
impl Into<ExecutionReportModel> for FixMessageBuilder {
    fn into(self) -> ExecutionReportModel {
        let price = match self.get_value_string("44") {
            Some(src) => Some(src.parse::<f64>().unwrap()),
            None => None,
        };

        let last_price = match self.get_value_string("31") {
            Some(src) => Some(src.parse::<f64>().unwrap()),
            None => None,
        };

        let trade_date = match self.get_value_string("75") {
            Some(src) => Some(src),
            None => None,
        };

        let reject_reason = match self.get_value_string("103") {
            Some(src) => Some(src),
            None => None,
        };

        let reject_text = match self.get_value_string("58") {
            Some(src) => Some(src),
            None => None,
        };

        ExecutionReportModel {
            internal_order_id: self.get_value_string("11").unwrap(),
            external_order_id: self.get_value_string("37").unwrap(),
            execute_id: self.get_value_string("17").unwrap(),
            ord_status: ExecutionReportModelStatus::from_str(
                self.get_value_string("39").unwrap().as_str(),
            ),
            execution_type: ExecutionReportModelExecutionType::from_str(
                self.get_value_string("150").unwrap().as_str(),
            ),
            symbol: self.get_value_string("55").unwrap(),
            side: PlaceOrderYbTcpContractSide::from_str(
                self.get_value_string("54").unwrap().as_str(),
            ),
            qty: self.get_value_string("38").unwrap().parse().unwrap(),
            order_type: OrderType::from_str(self.get_value_string("40").unwrap().as_str()),
            price,
            time_in_force: self.get_value_string("59"),
            last_price,
            avg_price: self.get_value_string("6").unwrap().parse().unwrap(),
            trade_date,
            reject_reason,
            reject_text,
        }
    }
}
 */
#[derive(Debug)]
pub struct YbMarketData {
    pub instrument_id: String,
    pub date: DateTimeAsMicroseconds,
    pub bid: f64,
    pub ask: f64,
}
