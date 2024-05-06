use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_fix::FixMessageWriter;

use crate::{tcp_messages::PlaceOrderYbTcpContract, YbFixSettings, OUR_FIX_VERSION};

pub fn logon(settings: &YbFixSettings, count: u64) -> FixMessageWriter {
    let now = DateTimeAsMicroseconds::now();
    let date_string = crate::date_utils::to_fix_date_string(now);

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "A");
    fill_from_settings(&mut fix_builder, settings, count, date_string.as_str());
    fix_builder.with_value("108", "30");
    fix_builder.with_value("141", "Y");
    fix_builder.with_value("554", &settings.password);
    fix_builder.with_value("98", "0");

    println!("Logon message: {}", fix_builder.to_string());
    return fix_builder;
}

pub fn ping(settings: &YbFixSettings, count: u64) -> FixMessageWriter {
    let now = DateTimeAsMicroseconds::now();
    let date_string = crate::date_utils::to_fix_date_string(now);

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "0");

    fill_from_settings(&mut fix_builder, settings, count, date_string.as_str());

    return fix_builder;
}

pub fn instrument_subscribe(
    settings: &YbFixSettings,
    instrument: &str,
    count: u64,
) -> FixMessageWriter {
    let now = DateTimeAsMicroseconds::now();
    let date_string = crate::date_utils::to_fix_date_string(now);

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "V");
    let uuid = now.unix_microseconds;

    /*
       fix_builder.with_value("49", &settings.sender_company_id);
       fix_builder.with_value("52", date_string.as_str());
       fix_builder.with_value("56", &settings.target_company_id);
       fix_builder.with_value("34", count.to_string().as_str());
    */
    fill_from_settings(&mut fix_builder, settings, count, date_string.as_str());

    //MDReqID - can be just a symbol name
    fix_builder.with_value("262", &uuid.to_string());
    //SubscriptionRequestType 1 = Snapshot + Updates
    fix_builder.with_value("263", "1");
    //Market Depth 1 = Top of Book
    fix_builder.with_value("264", "1");
    //MDUpdateType
    fix_builder.with_value("265", "0");
    //NoMDEntryTypes
    fix_builder.with_value("267", "2");
    //Bid
    fix_builder.with_value("269", "0");
    //Ask
    fix_builder.with_value("269", "1");
    //NoRelatedSym
    fix_builder.with_value("146", "1");
    //Symbol
    fix_builder.with_value("55", instrument);

    return fix_builder;
}

pub fn place_order_contract(
    settings: &YbFixSettings,
    contract: &PlaceOrderYbTcpContract,
    count: u64,
) -> FixMessageWriter {
    let now = DateTimeAsMicroseconds::now();
    let date_string = crate::date_utils::to_fix_date_string(now);

    let mut fix_builder = FixMessageWriter::new(OUR_FIX_VERSION, "D");

    fill_from_settings(&mut fix_builder, settings, count, date_string.as_str());
    //ClOrdID
    fix_builder.with_value("11", &contract.id);
    //HandIlnst
    fix_builder.with_value("21", "1");
    //Symbol
    fix_builder.with_value("55", &contract.symbol);
    //Side
    fix_builder.with_value("54", &(contract.side.clone() as i32).to_string());
    //OrderQty
    fix_builder.with_value("38", contract.qty.to_string().as_str());
    //OrdType - market
    fix_builder.with_value("40", "1");
    //TimeInForce - IOC
    fix_builder.with_value("59", "3");
    //TransactTime
    fix_builder.with_value("60", &date_string);

    return fix_builder;
}

fn fill_from_settings(
    fix_builder: &mut FixMessageWriter,
    settings: &YbFixSettings,
    count: u64,
    date_string: &str,
) {
    fix_builder.with_value("49", &settings.sender_company_id);
    fix_builder.with_value("52", date_string);
    fix_builder.with_value("56", &settings.target_company_id);
    fix_builder.with_value("34", count.to_string().as_str());
}
