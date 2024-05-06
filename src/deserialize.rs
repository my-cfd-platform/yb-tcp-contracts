use rust_fix::FixMessageReader;
use service_sdk::my_logger::LogEventCtx;

use crate::tcp_messages::YbMarketData;

pub fn deserialize_market_data(fix_message: &FixMessageReader<'_>) -> Result<YbMarketData, String> {
    // there shall be always no_md_entries in the message
    // skip message if it's not exist
    let no_md_entries = fix_message.get_value("268").unwrap();

    if no_md_entries == None {
        service_sdk::my_logger::LOGGER.write_error(
            String::from("FixMessageHandler"),
            format!("268 tag not found: {}", fix_message.to_string()),
            LogEventCtx::new(),
        );
        return Err("268 tag not found".to_string());
    }
    let no_md_entries = no_md_entries.unwrap().parse::<u32>().unwrap(); //.collect::<u32>().unwrap();

    // not sure why buy sometimes there are no prices available in the message,
    // so we skip the message
    if no_md_entries < 2 {
        service_sdk::my_logger::LOGGER.write_error(
            String::from("FixMessageHandler"),
            format!("Can not get md_entries: {}", fix_message.to_string()),
            LogEventCtx::new(),
        );
        return Err("md_entries less than 2".to_string());
    }
    let prices = fix_message
        .get_values("270")
        .unwrap()
        .iter()
        .map(|x| x.parse::<f64>().unwrap())
        .collect::<Vec<f64>>();

    // I think the clients have to know that we do like this,
    // this may be a regulatory issue for them if they not aware
    //let (bid, ask) = match prices[1] > prices[0] {
    //    true => (prices[0], prices[1]),
    //    false => (prices[1], prices[0]),
    //};

    let (bid, ask) = (prices[0], prices[1]);

    let external_market = fix_message.get_value("55").unwrap().unwrap();
    let date_time = fix_message.get_value("52").unwrap().unwrap();

    let result = YbMarketData {
        instrument_id: external_market.to_string(),
        date: crate::date_utils::parse_fix_date(date_time),
        bid,
        ask,
    };

    Ok(result)
}
