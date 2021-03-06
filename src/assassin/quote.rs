use std::rc::Rc;

extern crate chrono;
use self::chrono::prelude::*;

extern crate greenback;
use greenback::Greenback as Money;

#[derive(Clone)]
pub struct Quote {
    symbol: Rc<str>,
    expiration_date: DateTime<Utc>,
    ask: Money,
    bid: Money,
    last_price: Money,
    call: bool,
    strike_price: Money,
    volume: i32,
    implied_volatility: f32,
    delta: f32,
    gamma: f32,
    vega: f32,
    open_interest: i32,
    underlying_price: Money,
    date: DateTime<Utc>,
    name: Rc<str>,
    // TODO: depth, etc. if available
}

impl Quote {
    pub fn new(
        symbol: String,
        expiration_date: DateTime<Utc>,
        ask: Money,
        bid: Money,
        last_price: Money,
        call: bool,
        strike_price: Money,
        volume: i32,
        implied_volatility: f32,
        delta: f32,
        gamma: f32,
        vega: f32,
        open_interest: i32,
        underlying_price: Money,
        date: DateTime<Utc>,
    ) -> Quote {
        let name = format!(
            "{symbol}{year}{month}{day}{t}{price:>0width$}0",
            symbol = symbol,
            year = expiration_date.year(),
            month = expiration_date.month(),
            day = expiration_date.day(),
            t = if call { "C" } else { "P" },
            // this used to be multiplied by 100 but raw_value() is the same thing
            price = strike_price.raw_value(),
            width = 7,
        );

        let symbol_ref: &str = &symbol;
        let name_ref: &str = &name;

        Quote {
            symbol: Rc::from(symbol_ref),
            expiration_date: expiration_date,
            ask: ask,
            bid: bid,
            last_price: last_price,
            call: call,
            strike_price: strike_price,
            volume: volume,
            implied_volatility: implied_volatility,
            delta: delta,
            gamma: gamma,
            vega: vega,
            open_interest: open_interest,
            underlying_price: underlying_price,
            date: date,
            name: Rc::from(name_ref),
        }
    }

    pub fn is_call(&self) -> bool {
        self.call
    }

    #[allow(dead_code)]
    pub fn is_put(&self) -> bool {
        !self.is_call()
    }

    pub fn midpoint_price(&self) -> Money {
        (self.ask + self.bid) / 2
    }

    pub fn strike_price(&self) -> Money {
        self.strike_price
    }

    pub fn symbol(&self) -> Rc<str> {
        Rc::clone(&self.symbol)
    }

    pub fn bid(&self) -> Money {
        self.bid
    }

    pub fn ask(&self) -> Money {
        self.ask
    }

    pub fn expiration_date(&self) -> DateTime<Utc> {
        self.expiration_date
    }

    pub fn days_to_expiration(&self, current_date: DateTime<Utc>) -> i32 {
        self.expiration_date.num_days_from_ce() - current_date.num_days_from_ce()
    }

    pub fn underlying_price(&self) -> Money {
        self.underlying_price
    }

    // See: https://en.wikipedia.org/wiki/Option_naming_convention#Proposed_revision
    // e.g., CSCO171117C00019000
    pub fn name(&self) -> Rc<str> {
        Rc::clone(&self.name)
    }

    #[allow(dead_code)]
    pub fn days_until_expiration(&self) -> i32 {
        self.expiration_date.num_days_from_ce() - self.date.num_days_from_ce()
    }

    #[allow(dead_code)]
    pub fn intrinsic_value(&self) -> Money {
        if self.call {
            if self.underlying_price > self.strike_price {
                self.underlying_price - self.strike_price
            } else {
                Money::zero()
            }
        } else {
            if self.underlying_price < self.strike_price {
                self.strike_price - self.underlying_price
            } else {
                Money::zero()
            }
        }
    }

    #[allow(dead_code)]
    pub fn extrinsic_value(&self) -> Money {
        self.midpoint_price() - self.intrinsic_value()
    }

    #[allow(dead_code)]
    pub fn value_ratio(&self) -> f32 {
        // TODO: if i_value is 0, this is division by 0 and becomes infinity.
        //       see if we should return an Option<Money> in light of that...

        let extrinsic = self.extrinsic_value().raw_value() as f32;
        let intrinsic = self.intrinsic_value().raw_value() as f32;

        (extrinsic / intrinsic) / 100.0
    }

    #[allow(dead_code)]
    pub fn print_deets(&self) {
        debug!("=======================");
        debug!("name: {}", self.name());
        debug!("spread: {}", self.ask - self.bid);
        debug!("intrinsic: {}", self.intrinsic_value());
        debug!("extrinsic: {}", self.extrinsic_value());
        debug!("value ratio: {:.2}%", self.value_ratio());
        debug!("last price: {}", self.last_price);
        debug!("underlying price: {}", self.underlying_price);
        debug!("date: {} expiration: {}", self.date, self.expiration_date);
        debug!("days left: {}", self.days_until_expiration());
    }

    pub fn date(&self) -> DateTime<Utc> {
        self.date
    }
}
