use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

use assassin::quote::Quote;
use assassin::traits::*;

extern crate chrono;
use self::chrono::prelude::*;

extern crate greenback;
use greenback::Greenback as Money;

pub struct DiscountOptionData {
    enumerator: BufReader<File>,
    line: String,
}

impl DiscountOptionData {
    pub fn new(filename: &'static str) -> DiscountOptionData {
        let enumerator = BufReader::new(File::open(filename).unwrap());

        // eat the header row
        // enumerator.next();

        DiscountOptionData {
            enumerator: enumerator,
            line: String::with_capacity(128),
        }
    }
}

//    0         1            2      3        4       5         6        7         8        9
// Symbol ExpirationDate AskPrice AskSize BidPrice BidSize LastPrice PutCall StrikePrice Volume
// AAPL   2013-01-04     10.55            10.35            10.55     call    540         14292
// Symbol,ExpirationDate,AskPrice,AskSize,BidPrice,BidSize,LastPrice,PutCall,StrikePrice,Volume,
// AAPL,2013-01-04,10.55,,10.35,,10.55,call,540,14292,0.295,0.7809,2.4778,11.9371,,8666,549.03,

//       10           11     12    13       14     15             16         17
// ImpliedVolatility Delta  Gamma  Vega,    Rho OpenInterest UnderlyingPrice DataDate
// 0.295             0.7809 2.4778 11.9371      8666         549.03          2013-01-02
// ImpliedVolatility,Delta,Gamma,Vega,Rho,OpenInterest,UnderlyingPrice,DataDate
// 2013-01-02

impl DataFeed for DiscountOptionData {
    fn next_quote(&mut self) -> Option<Quote> {
        self.line.clear();

        let res = self.enumerator.read_line(&mut self.line);
        if !res.is_ok() {
            return None;
        }

        let len = self.line.len();

        if len == 0 {
            return None;
        } else {
            self.line.truncate(len - 1);
        }

        // ----- split CSV and parse fields -------------------------------

        let v: Vec<&str> = self.line.split(',').collect();
        assert_eq!(v.len(), 18);

        let symbol = v[0].parse().unwrap();

        let expiration_date = {
            let date_parts: Vec<u32> = v[1].split('-').map(|p| p.parse().unwrap()).collect();
            Utc.ymd(date_parts[0] as i32, date_parts[1], date_parts[2])
                .and_hms(0, 0, 0)
        };

        let ask: f32 = v[2].parse().unwrap();
        let bid: f32 = v[4].parse().unwrap();

        let last_price: f32 = v[6].parse().unwrap();
        let call: String = v[7].parse().unwrap();

        if call != "call" && call != "put" {
            panic!("expected 'call' or 'put', got {}", call);
        }

        let strike_price: f32 = v[8].parse().unwrap();
        let volume: i32 = v[9].parse().unwrap();
        let implied_volatility: f32 = v[10].parse().unwrap();
        let delta: f32 = v[11].parse().unwrap();
        let gamma: f32 = v[12].parse().unwrap();
        let vega: f32 = v[13].parse().unwrap();
        let open_interest: i32 = v[15].parse().unwrap();
        let underlying_price: f32 = v[16].parse().unwrap();

        let date = {
            let date_parts: Vec<u32> = v[17].split('-').map(|p| p.parse().unwrap()).collect();
            Utc.ymd(date_parts[0] as i32, date_parts[1], date_parts[2])
                .and_hms(0, 0, 0)
        };

        let t = Quote::new(
            symbol,
            expiration_date,
            Money::from_float(ask),
            Money::from_float(bid),
            Money::from_float(last_price),
            call == "call",
            Money::from_float(strike_price),
            volume,
            implied_volatility,
            delta,
            gamma,
            vega,
            open_interest,
            Money::from_float(underlying_price),
            date,
        );

        Some(t)
    }
}
