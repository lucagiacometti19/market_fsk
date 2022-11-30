use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_error::GoodKindError;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::*;
struct FskMarket {
    goods: HashMap<GoodKind, GoodLabel>,
    //the key is the token given as ret value of a buy/sell lock fn
    locked_goods_to_sell: HashMap<String, GoodLabel>,
    locked_goods_to_buy: HashMap<String, GoodLabel>,
    subs: Vec<Box<dyn Notifiable>>,
}

impl Notifiable for FskMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.subs.push(subscriber)
    }

    fn on_event(&mut self, event: Event) {
        // here we apply logic of changing good quantities, as described in
        // https://github.com/orgs/WG-AdvancedProgramming/discussions/38#discussioncomment-4167913
        match event.kind {
            EventKind::LockedBuy => {}
            EventKind::Bought => {}
            EventKind::LockedSell => {}
            EventKind::Sold => {}
            EventKind::Wait => {}
        }
    }
}

impl Market for FskMarket {
    fn new_random() -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn get_name(&self) -> &'static str {
        "FSK"
    }

    ///What is this fn needed for? What should it return?
    fn get_budget(&self) -> f32 {
        //Budget as sum of every good value in euros
        /* let mut res = 0.;
        for (_, good_label) in &self.goods {
            res += good_label.quantity as f32 * good_label.good_kind.get_default_exchange_rate();
        }
        res */

        //Budget as quantity of euros - as in specs
        let mut res = 0.;
        for (_, good_label) in &self.goods {
            if GoodKind::EUR == good_label.good_kind {
                res += good_label.quantity
            }
        }
        res
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        let mut good_quantity = 0.;

        //the quantity is not positive
        if quantity < 0. {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }
        //the quantity the trader is asking to buy is higher than the quantity the market owns
        if let Some(good) = self.goods.get(&kind) {
            good_quantity = good.quantity;
            if good.quantity > quantity {
                todo!("add price calculation and return value");
            }
        }
        //either goodkind was not found in self.goods or its quantity was not enough
        Err(MarketGetterError::InsufficientGoodQuantityAvailable {
            requested_good_kind: kind,
            requested_good_quantity: quantity,
            available_good_quantity: good_quantity,
        })
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        //the quantity is not positive
        if quantity < 0. {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        todo!("Price calculation and return value")
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        let mut res = Vec::new();
        for (_, good_label) in &self.goods {
            res.push(good_label.clone());
        }
        res
    }

    fn lock_buy(
        &mut self,
        kind_to_buy: GoodKind,
        quantity_to_buy: f32,
        bid: f32,
        trader_name: String,
    ) -> Result<String, LockBuyError> {
        todo!()
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        todo!()
    }

    fn lock_sell(
        &mut self,
        kind_to_sell: GoodKind,
        quantity_to_sell: f32,
        offer: f32,
        trader_name: String,
    ) -> Result<String, LockSellError> {
        todo!()
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        todo!()
    }
}

impl FskMarket {
    fn notify(&mut self, event: Event) {
        for subscriber in &mut self.subs {
            subscriber.as_mut().on_event(event.clone());
        }
    }
}

fn main() {
    println!("Hello, world!");
}
