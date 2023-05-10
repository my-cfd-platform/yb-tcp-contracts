use my_tcp_sockets::tcp_connection::TcpContract;
use rust_fix::FixMessageBuilder;
#[derive(Debug, Clone)]
pub struct FixLogonCredentials {
    pub password: String,
    pub sender: String,
    pub target: String,
}

impl TcpContract for FixMessage {
    fn is_pong(&self) -> bool {
        match self{
            FixMessage::Income(src) => src.is_pong(),
            FixMessage::Outcome(_) => false,
        }
    }
}

pub enum FixIncomeMessage {
    Logon(FixMessageBuilder),
    Reject(FixMessageBuilder),
    Logout(FixMessageBuilder),
    MarketData(FixMessageBuilder),
    MarketDataReject(FixMessageBuilder),
    ExecutionReport(FixMessageBuilder),
    Others(FixMessageBuilder),
    Pong,
    Ping,
}

impl FixIncomeMessage {
    pub fn to_string(&self) -> String{
        match self{
            FixIncomeMessage::Logon(src) => src.to_string(),
            FixIncomeMessage::Reject(src) => src.to_string(),
            FixIncomeMessage::Logout(src) => src.to_string(),
            FixIncomeMessage::MarketData(src) => src.to_string(),
            FixIncomeMessage::MarketDataReject(src) => src.to_string(),
            FixIncomeMessage::ExecutionReport(src) => src.to_string(),
            FixIncomeMessage::Others(src) => src.to_string(),
            FixIncomeMessage::Pong => "Pong".to_string(),
            FixIncomeMessage::Ping => "Ping".to_string(),
        }
    }
}

impl TcpContract for FixIncomeMessage {
    fn is_pong(&self) -> bool {
        match self {
            FixIncomeMessage::Pong => true,
            _ => false,
        }
    }
}

pub enum FixOutcomeMessage {
    InstrumentsSubscribe(String),
    Logon,
    Ping,
    PlaceOrder(PlaceOrderYbTcpContract),
}

pub enum FixMessage {
    Income(FixIncomeMessage),
    Outcome(FixOutcomeMessage),
}

impl FixMessage {
    pub fn to_string(&self) -> String {
        match self{
            FixMessage::Income(src) => src.to_string(),
            FixMessage::Outcome(_) => todo!(),
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
pub enum OrderType {
    Market,
    Limit,
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
                self.get_value_string("39").unwrap().as_str(),
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
