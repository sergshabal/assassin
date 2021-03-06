use std::time::Instant;

use assassin::broker::Broker;
use assassin::traits::*;

extern crate greenback;
use greenback::Greenback as Money;
use greenback::util::add_commas;

pub struct Simulation {
    model: Box<Model>,
    broker: Box<Broker>,

    // TODO: add settings variables (slippage, spread multipliers, etc.)
    // TODO: add target stats that the model must hit (sharpe, DD, etc.)
    start_time: Instant,
    starting_balance: Money,
}

impl Simulation {
    pub fn new(model: Box<Model>, broker: Box<Broker>) -> Simulation {
        let starting_balance = broker.account_balance();

        Simulation {
            model: model,
            broker: broker,
            start_time: Instant::now(),
            starting_balance: starting_balance,
        }
    }

    pub fn run(&mut self) {
        self.model.before_simulation(&mut *self.broker);

        while self.broker.process_simulation_data() {
            let orders = self.model.run_logic(&self.broker);

            for o in orders {
                self.broker.process_order(o);
            }
        }

        self.model.after_simulation(&mut *self.broker);
    }

    pub fn print_stats(&self) {
        info!("");
        info!("===============================================================");
        info!("");

        let balance = self.broker.account_balance();

        info!("===== POSITIONS =====");
        info!("");

        let mut running_total = self.starting_balance;

        let positions = self.broker.positions();

        for pos in &positions {
            info!("----- {} -----", pos.name());

            for o in pos.orders() {
                running_total = running_total + o.canonical_cost_basis();

                // BUY 10 contracts @ $15
                info!(
                    "  {} {} {} contracts @ {}",
                    o.buy_or_sell_string(),
                    o.quantity(),
                    o.option_name(),
                    o.fill_price(),
                );
            }
            info!("");

            info!("Commission paid: {}", pos.commission_paid());
            info!("Position value: {}", pos.realized_profit());
            info!("Running total: {}", running_total);
            info!("");
        }

        let balance_change = balance - self.starting_balance;

        info!("===== RESULTS =====");
        info!("");
        info!("Starting balance: {}", self.starting_balance);
        info!("Ending balance: {}", balance);
        info!("Change: {}", balance_change);

        let capital_growth = ((balance.raw_value() as f32
            / self.starting_balance.raw_value() as f32) * 100.0)
            - 100.0;

        let total_commish: Money = positions.iter().map(|p| p.commission_paid()).sum();

        let commish_percent_of_profit = if balance_change > Money::zero() {
            (total_commish.raw_value() as f32 / balance_change.raw_value() as f32) * 100.0
        } else {
            0.0
        };

        let order_counts: Vec<i32> = positions.iter().map(|p| p.order_count()).collect();
        let total_order_count: i32 = order_counts.iter().sum();
        let broker_closed_order_count: i32 = positions
            .iter()
            .map(|p| p.broker_closed_order_count())
            .sum();

        let broker_closed_percent =
            (broker_closed_order_count as f32 / total_order_count as f32) * 100.0;

        let average_commission = {
            if total_order_count > 0 {
                total_commish / total_order_count
            } else {
                Money::zero()
            }
        };

        info!("Capital growth: {:.2}%", capital_growth);
        info!("Total orders: {}", total_order_count);
        info!(
            "Orders closed by broker: {} ({:.2}%)",
            broker_closed_order_count,
            broker_closed_percent,
        );
        info!(
            "Commission paid: {} ({:.2}% of profit)",
            total_commish,
            commish_percent_of_profit,
        );
        info!("Average commission per order: {}", average_commission);
        info!(
            "Highest realized account balance: {}",
            self.broker.highest_realized_account_balance()
        );
        info!(
            "Lowest realized account balance: {}",
            self.broker.lowest_realized_account_balance()
        );
        info!(
            "Highest unrealized account balance: {}",
            self.broker.highest_unrealized_account_balance()
        );
        info!(
            "Lowest unrealized account balance: {}",
            self.broker.lowest_unrealized_account_balance()
        );
        info!("");

        let quotes_per_sec = self.broker.quotes_processed() as f32 / self.total_run_time();

        info!(
            "Ran simulation ({} ticks) in {:.2} seconds ({}/sec)",
            add_commas(self.broker.quotes_processed()),
            self.total_run_time(),
            add_commas(quotes_per_sec as i64),
        );
        info!("");
    }

    pub fn total_run_time(&self) -> f32 {
        let seconds = self.start_time.elapsed().as_secs() as f32;
        let nanoseconds = self.start_time.elapsed().subsec_nanos() as f32 * 1e-9;

        seconds + nanoseconds
    }
}
