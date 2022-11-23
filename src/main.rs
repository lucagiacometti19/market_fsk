use calloop::{
    timer::{TimeoutAction, Timer},
    EventLoop,
};
use std::collections::HashMap;
use std::time::Duration;

use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::*;
use unitn_market_2022::market::*;
struct FskMarket {
    goods: HashMap<good_kind::GoodKind, good::Good>,
    locked_goods_to_sell: HashMap<String, good::Good>,
    budget: f32,
}
impl MarketTrait for FskMarket {
    fn get_market_name(&self) -> String {
        "FSK".to_string()
    }

    fn get_budget(&self) -> f32 {
        self.budget
    }

    fn get_goods(&self) -> std::collections::HashMap<good_kind::GoodKind, &good::Good> {
        let mut result: HashMap<GoodKind, &good::Good> = HashMap::new();

        for (k, v) in &self.goods {
            result.insert(k.clone(), v);
        }

        result
    }

    fn lock_trader_buy_from_market(
        &mut self,
        g: good_kind::GoodKind,
        p: f32,
        q: f32,
        d: String,
    ) -> Result<String, MarketError> {
        

        let mut event_loop = EventLoop::try_new().expect("Failed to initialize the event loop!");

        let timer = Timer::from_duration(Duration::from_secs(5));

        event_loop
            .handle()
            .insert_source(timer, |deadline, _: &mut (), _shared_data| {
                println!("Event fired for: {:?}", deadline);
                TimeoutAction::Drop
            })
            .expect("Failed to insert event source!");

        event_loop
            .dispatch(None, &mut ())
            .expect("Error during event loop!");
    }

    fn trader_buy_from_market(
        &mut self,
        token: String,
        cash: &mut good::Good,
    ) -> Result<good::Good, MarketError> {
        todo!()
    }

    fn lock_trader_sell_to_market(
        &mut self,
        g: good_kind::GoodKind,
        qty: f32,
        price: f32,
        d: String,
    ) -> Result<String, MarketError> {
        todo!()
    }

    fn trader_sell_to_market(
        &mut self,
        token: String,
        good: &mut good::Good,
    ) -> Result<good::Good, MarketError> {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
