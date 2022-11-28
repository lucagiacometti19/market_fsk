use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_error::GoodKindError;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::market::*;
use unitn_market_2022::market::good_label::GoodLabel;
struct FskMarket {
    goods: Vec<GoodLabel>,
    //the key is the token given as ret value of a buy/sell lock fn
    locked_goods_to_sell: HashMap<String, GoodLabel>,
    locked_goods_to_buy: HashMap<String, GoodLabel>,
}

impl Notifiable for FskMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        todo!()
    }

    fn on_event(&mut self, event: Event) {
        todo!()
    }
}

impl Market for FskMarket {
    fn new_random() -> Rc<RefCell<dyn Market>> where Self: Sized {
        todo!()
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> where Self: Sized {
        todo!()
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>> where Self: Sized {
        todo!()
    }

    fn get_name(&self) -> &'static str {
        "FSK"
    }

    fn get_budget(&self) -> f32 {
        let mut res = 0.;
        for elem in &self.goods {
            res += elem.quantity as f32 * elem.good_kind.get_default_exchange_rate();
        }
        res
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity < 0. {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }
        Ok(0.)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        todo!()
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        self.goods.clone()
    }

    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        todo!()
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        todo!()
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        todo!()
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
