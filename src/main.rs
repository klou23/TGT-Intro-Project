mod order_book {
    use std::error::Error;
    use std::str::FromStr;
    use reqwest::blocking::Client;
    use tungstenite::{connect, Message};
    use serde_json::{json, from_slice, Value};

    /// Attempts to get a public token from KuCoin.
    /// Returns the token as a string if successful
    pub fn get_token() -> Result<String, Box<dyn Error>> {
        let client = Client::new();
        let response = client
            .post("https://api.kucoin.com/api/v1/bullet-public")
            .send()?
            .json::<serde_json::Value>()?;

        let token = response["data"]["token"]
            .as_str()
            .expect("Failed to get token");
        Ok(String::from(token))
    }

    /// Attempts to connect to the KuCoin WebSocket server and subscribe to ETH-USDT futures data
    /// Prints the order book after every market data update
    pub fn connect_data(token: String) -> Result<(), Box<dyn Error>> {
        let url = format!("wss://ws-api-spot.kucoin.com/?token={}", token);
        let (mut socket, _) = connect(url)?;

        socket.send(Message::Text(json!({
            "id": 1545910660740u64,
            "type": "subscribe",
            "topic": "/contractMarket/level2Depth5:ETHUSDTM",
            "response": true
        }).to_string()))?;

        loop {
            let msg = socket.read()?;
            if let Message::Text(text) = msg {
                let msg: Value = from_slice(text.as_ref()).expect("Can't parse to JSON");
                if msg["type"] == "message" {
                    print_order_book(msg);
                }
            }
        }
    }

    /// Parses a message from the KuCoin websocket and prints the bids and asks in columnar format
    fn print_order_book(msg: Value) -> Result <(), Box<dyn Error>> {
        println!("\nType\tPrice\t\tSize");
        for ask in msg["data"]["asks"].as_array().expect("Parsing Error").iter().rev() {
            let price = f64::from_str(ask[0].as_str().unwrap()).unwrap();
            let size = ask[1].as_i64().unwrap();
            println!("A\t{:.2}\t\t{}", price, size);
        }
        for bid in msg["data"]["bids"].as_array().expect("Parsing Error") {
            let price = f64::from_str(bid[0].as_str().unwrap()).unwrap();
            let size = bid[1].as_i64().unwrap();
            println!("B\t{:.2}\t\t{}", price, size);
        }
        Ok(())
    }

}

fn main() {
    let token = order_book::get_token()
        .expect("Failed to get token");
    order_book::connect_data(token).expect("TODO: panic message");
}
