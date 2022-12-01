use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

use random_string::generate;
use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::{market::*, subscribe_each_other};
struct FskMarket {
    goods: HashMap<GoodKind, GoodLabel>,
    //the key is the token given as ret value of a buy/sell lock fn
    buy_contracts_archive: ContractsArchive,
    sell_contracts_archive: ContractsArchive,
    subs: Vec<Weak<RefCell<dyn Notifiable>>>,
    time: u64,
}

struct ContractsArchive {
    contracts_by_token: HashMap<String, Rc<LockContract>>,
    expired_contracts: HashSet<String>,
    contracts_by_timestamp: VecDeque<Rc<LockContract>>,
}

impl ContractsArchive {
    fn new() -> ContractsArchive {
        ContractsArchive {
            contracts_by_token: HashMap::new(),
            expired_contracts: HashSet::new(),
            contracts_by_timestamp: VecDeque::new(),
        }
    }

    fn new_token(&self) -> String {
        let charset = "1234567890abcdefghijklmnopqrstuwxyz";
        loop {
            let res = generate(10, charset);
            if !self.contracts_by_token.contains_key(&res) {
                return res;
            }
        }
    }

    fn add_contract(&mut self, contract: &Rc<LockContract>) {
        //will always work since token is unique
        self.contracts_by_token
            .insert(contract.token.clone(), contract.clone());
        self.contracts_by_timestamp.push_back(contract.clone());
    }

    fn pop_expired(&mut self, timestamp: u64) -> Option<Rc<LockContract>> {
        if !self.contracts_by_timestamp.is_empty() {
            if self.contracts_by_timestamp.get(0).unwrap().expiry_time >= timestamp {
                //can't fail no need to check option
                let res = self.contracts_by_timestamp.pop_front().unwrap();
                self.contracts_by_token.remove(&res.token);
                self.expired_contracts.insert(res.token.clone());
                return Some(res);
            }
        }
        None
    }
}

#[derive(Debug)]
struct LockContract {
    token: String,
    good: Good,
    price: f32,
    expiry_time: u64,
}

impl Notifiable for FskMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        //self.subs.push(Rc::downgrade(&Rc::new(RefCell::new(*subscriber))))
        todo!()
    }

    fn on_event(&mut self, event: Event) {
        // here we apply logic of changing good quantities, as described in https://github.com/orgs/WG-AdvancedProgramming/discussions/38#discussioncomment-4167913
        //every event triggers a new tick
        self.time += 1;
        match event.kind {
            EventKind::LockedBuy => {}
            EventKind::Bought => {}
            EventKind::LockedSell => {}
            EventKind::Sold => {}
            EventKind::Wait => (),
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
        /* let mut good = self.goods.get_mut(&kind_to_buy);
        println!("{:?}", good);

        match &good {
            Some(a) if quantity_to_buy <= 0.0 => {
                return Err(LockBuyError::NonPositiveQuantityToBuy {
                    negative_quantity_to_buy: quantity_to_buy,
                })
            }
            Some(a) if bid <= 0.0 => {
                return Err(LockBuyError::NonPositiveBid { negative_bid: bid })
            }
            Some(a) if a.quantity < quantity_to_buy => {
                return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                    requested_good_kind: (kind_to_buy),
                    requested_good_quantity: (quantity_to_buy),
                    available_good_quantity: (a.quantity),
                })
            } //controllo se c'è abbastanza quantity
            Some(a) if bid / quantity_to_buy < a.exchange_rate_buy => {
                return Err(LockBuyError::BidTooLow {
                    requested_good_kind: (kind_to_buy),
                    requested_good_quantity: (quantity_to_buy),
                    low_bid: (bid),
                    lowest_acceptable_bid: (a.exchange_rate_buy),
                })
            }
            Some(_) => (),
            None => return Err(LockBuyError::MaxAllowedLocksReached), //in realtà il kind non è stato trovato ma non esiste quell'errore
        }
        //Everything is okay
        good.as_mut().unwrap().quantity -= quantity_to_buy;

        //register (via the market-local Good Metadata) the fact that quantity quantity_to_buy of good kind_to_buy is to be bought for price bid.

        println!("[{}]", token);

        self.locked_goods_to_buy.insert(
            token.to_string(),
            LockContract {
                token: token.to_string(),
                good: Good::new(kind_to_buy, good.as_mut().unwrap().quantity),
                price: bid,
                expiry_time: u64::MAX,
            },
        );
        return Ok(token.to_string()); */
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
        for sub in &mut self.subs {
            match sub.upgrade() {
                Option::None => { /* market was dropped */ }
                Option::Some(m) => (*m).borrow_mut().on_event(event.clone()),
            };
        }
    }
}

#[test]
fn test_add_subscriber() {
    //test goods
    let mut test_goods: HashMap<GoodKind, GoodLabel> = HashMap::new();

    test_goods.insert(
        GoodKind::EUR,
        GoodLabel {
            good_kind: GoodKind::EUR,
            quantity: 12501.5,
            exchange_rate_buy: 1.002,
            exchange_rate_sell: 0.982,
        },
    );

    test_goods.insert(
        GoodKind::USD,
        GoodLabel {
            good_kind: GoodKind::USD,
            quantity: 12.65,
            exchange_rate_buy: 1.32,
            exchange_rate_sell: 0.895,
        },
    );

    //dummy markets creation - wrong way to create markets!
    let mut test_market_1 = FskMarket {
        goods: test_goods,
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
    };

    let mut test_market_2 = FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
    };

    test_market_1.add_subscriber(Box::new(test_market_2));
    assert!(!test_market_1.subs.is_empty())
}

#[test]
fn test_on_event() {
    //dummy market creation - wrong way to create a market!
    let mut test_market_1 = FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
    };

    test_market_1.on_event(Event {
        kind: EventKind::Wait,
        good_kind: GoodKind::EUR,
        quantity: 0.,
        price: 0.,
    });

    assert_ne!(test_market_1.time, 0)
}

#[test]
fn test_notify() {
    //dummy market creation - wrong way to create a market!
    let mut test_market_1 = FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
    };

    let mut test_market_2 = FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
    };

    subscribe_each_other!();
}

fn main() {
    println!("Hello, world!");
}
