use chrono::{prelude, Datelike, Timelike, Utc};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fs::File;
use std::rc::{Rc, Weak};

use random_string::generate;
use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::event::wrapper::NotifiableMarketWrapper;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{self, GoodKind};
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::{market::*, subscribe_each_other, wait_one_day};

const LOCK_INITIAL_TTL: u64 = 9;

struct FskMarket {
    goods: HashMap<GoodKind, GoodLabel>,
    //the key is the token given as ret value of a buy/sell lock fn
    buy_contracts_archive: ContractsArchive,
    sell_contracts_archive: ContractsArchive,
    log_output: Option<File>,
    subs: Vec<Box<dyn Notifiable>>,
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

    fn consume_contract(&mut self, token: &String) -> Option<Rc<LockContract>> {
        self.contracts_by_token.remove(token)
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
        self.subs.push(subscriber);
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

        //restore locked default currency for expired sell
        while let Some(expired_contract) = self.sell_contracts_archive.pop_expired(self.time) {
            self.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap().quantity += expired_contract.price;
        }

        //restore locked good for expired buyout
        while let Some(expired_contract) = self.buy_contracts_archive.pop_expired(self.time) {
            self.goods
                .get_mut(&expired_contract.good.get_kind())
                .unwrap()
                .quantity += expired_contract.good.get_qty();
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
        self.goods.get(&DEFAULT_GOOD_KIND).unwrap().quantity
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
        if quantity <= 0. {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let maximum_price = quantity / self.goods.get(&kind).unwrap().exchange_rate_sell;
        //how much money the market pay (at max) for the good

        let available_default_good = self.get_budget();

        if available_default_good > maximum_price {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind,
                requested_good_quantity: quantity, //TODO: check if this is the right variable to return
                available_good_quantity: maximum_price,
            });
        }

        Ok(maximum_price)
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
        let get_buy_price_result = self.get_buy_price(kind_to_buy, quantity_to_buy);
        let mut good = self.goods.get_mut(&kind_to_buy);
        match get_buy_price_result {
            Ok(min_bid) => match &good {
                Some(_) if bid < 0. => {
                    return Err(LockBuyError::NonPositiveBid { negative_bid: bid })
                }

                Some(good_label) if bid < min_bid => {
                    return Err(LockBuyError::BidTooLow {
                        requested_good_kind: (kind_to_buy),
                        requested_good_quantity: (quantity_to_buy),
                        low_bid: (bid),
                        lowest_acceptable_bid: (good_label.exchange_rate_buy),
                    })
                }
                _ => (),
            },
            Err(err) => match err {
                MarketGetterError::NonPositiveQuantityAsked => {
                    return Err(LockBuyError::NonPositiveQuantityToBuy {
                        negative_quantity_to_buy: quantity_to_buy,
                    })
                }
                MarketGetterError::InsufficientGoodQuantityAvailable {
                    requested_good_kind,
                    requested_good_quantity,
                    available_good_quantity,
                } => {
                    return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                        requested_good_kind,
                        requested_good_quantity,
                        available_good_quantity,
                    })
                }
            },
        }
        //Everything is okay
        good.as_mut().unwrap().quantity -= quantity_to_buy;

        let token = self.buy_contracts_archive.new_token();

        self.buy_contracts_archive
            .add_contract(&Rc::new(LockContract {
                token: token.to_string(),
                good: Good::new(kind_to_buy, good.as_mut().unwrap().quantity),
                price: bid,
                expiry_time: self.time + LOCK_INITIAL_TTL,
            }));

        //register (via the market-local Good Metadata) the fact that quantity quantity_to_buy of good kind_to_buy is to be bought for price bid.
        self.notify(Event {
            kind: EventKind::LockedBuy,
            good_kind: kind_to_buy,
            quantity: quantity_to_buy,
            price: bid,
        });
        return Ok(token.to_string());
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
        //1
        if quantity_to_sell <= 0. {
            self.write_log_lock_sell_error(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::NonPositiveQuantityToSell {
                negative_quantity_to_sell: quantity_to_sell,
            });
        }

        //2
        if offer < 0. {
            self.write_log_lock_sell_error(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::NonPositiveOffer {
                negative_offer: offer,
            });
        }

        //5
        if self.get_budget() < offer {
            self.write_log_lock_sell_error(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: offer,
                available_good_quantity: self.get_budget(),
            });
        }

        //6
        let highest_acceptable_offer = self
            .get_sell_price(kind_to_sell, quantity_to_sell)
            .unwrap_or(0.);
        if highest_acceptable_offer < offer {
            self.write_log_lock_sell_error(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::OfferTooHigh {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                high_offer: offer,
                highest_acceptable_offer,
            });
        }

        //we chose to decrease the budget when goods are locked, to avoid having to keep track of locked default good. In case the lock expires, default currency will be put back in goods.
        self.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap().quantity -= offer;

        let token = self.sell_contracts_archive.new_token();

        self.sell_contracts_archive
            .add_contract(&Rc::new(LockContract {
                token: token.clone(),
                good: Good::new(kind_to_sell, quantity_to_sell),
                price: offer,
                expiry_time: self.time + LOCK_INITIAL_TTL,
            }));

        self._write_log_sell_ok(trader_name, kind_to_sell, quantity_to_sell, offer, &token);

        self.notify(Event {
            kind: EventKind::LockedSell,
            good_kind: kind_to_sell,
            quantity: quantity_to_sell,
            price: offer,
        });

        Ok(token)
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        let op_contract = self.sell_contracts_archive.contracts_by_token.get(&token);

        //1
        if op_contract.is_none() {
            self.write_log_sell_error(&token);
            
            if self
                .sell_contracts_archive
                .expired_contracts
                .contains(&token)
            {
                return Err(SellError::ExpiredToken {
                    expired_token: token,
                });
            }

            return Err(SellError::UnrecognizedToken {
                unrecognized_token: token,
            });
        }

        let contract = op_contract.unwrap();

        //2
        if contract.expiry_time <= self.time {
            self.write_log_sell_error(&token);
            return Err(SellError::ExpiredToken {
                expired_token: token,
            });
        }

        //3
        if contract.good.get_kind() != good.get_kind() {
            self.write_log_sell_error(&token);
            return Err(SellError::WrongGoodKind {
                wrong_good_kind: good.get_kind(),
                pre_agreed_kind: contract.good.get_kind(),
            });
        }

        //4
        if good.get_qty() < contract.good.get_qty() {
            self.write_log_sell_error(&token);

            return Err(SellError::InsufficientGoodQuantity {
                contained_quantity: good.get_qty(),
                pre_agreed_quantity: contract.good.get_qty(),
            });
        }

        //everything checks out, the sell can proceed

        //this is the default currency that is going to be returned to the seller (the trader)
        let good_to_return = Good::new(DEFAULT_GOOD_KIND, contract.price); //don't need to decrease owned good, already did that in lock_sell(...)

        self.goods.get_mut(&good.get_kind()).unwrap().quantity +=
            good.split(contract.good.get_qty()).unwrap().get_qty();
        //unwrapping should be safe as errors error conditions were alread checked in gate 4

        self.write_log_entry(format!("SELL-TOKEN:{}-OK", token));

        self.sell_contracts_archive.consume_contract(&token);

        self.notify(Event {
            kind: EventKind::Sold,
            good_kind: good.get_kind(),
            quantity: good.get_qty(),
            price: good_to_return.get_qty(),
        });

        Ok(good_to_return)
    }
}

impl FskMarket {
    fn notify(&mut self, event: Event) {
        for sub in &mut self.subs {
            sub.on_event(event.clone());
        }
    }

    fn initialize_log_file(&mut self) {
        let log_file_name = format!("log_{}.txt", self.get_name());
        self.log_output = File::create(log_file_name).ok();

        assert!(self.log_output.is_some());
    }

    fn write_log_entry(&self, entry: String) {
        println!(
            "{}|{}|{}\n",
            self.get_name(),
            Utc::now().format("%y:%m:%d:%H:%M:%S:%4f"),
            entry
        );
        //YY:MM:DD:HH:MM:SEC:MSES
    }

    fn _write_log_buy_ok(
        &self,
        trader_name: String,
        kind_to_buy: GoodKind,
        quantity_to_buy: f32,
        bid: f32,
        token: &String,
    ) {
        self.write_log_entry(format!(
            "LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{:+e}-BID:{:+e}-TOKEN:{}",
            trader_name, kind_to_buy, quantity_to_buy, bid, token
        ));
    }

    fn _write_log_buy_error(
        &self,
        trader_name: String,
        kind_to_buy: GoodKind,
        quantity_to_buy: f32,
        bid: f32,
    ) {
        self.write_log_entry(format!(
            "LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{:+e}-BID:{:+e}-ERROR",
            trader_name, kind_to_buy, quantity_to_buy, bid
        ));
    }

    fn _write_log_sell_ok(
        &self,
        trader_name: String,
        kind_to_sell: GoodKind,
        quantity_to_sell: f32,
        offer: f32,
        token: &String,
    ) {
        self.write_log_entry(format!(
            "LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{:+e}-OFFER:{:+e}-TOKEN:{}",
            trader_name, kind_to_sell, quantity_to_sell, offer, token
        ));
    }

    fn write_log_lock_sell_error(
        &self,
        trader_name: String,
        kind_to_sell: GoodKind,
        quantity_to_sell: f32,
        offer: f32,
    ) {
        self.write_log_entry(format!(
            "LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{:+e}-OFFER:{:+e}-ERROR",
            trader_name, kind_to_sell, quantity_to_sell, offer
        ));
    }

    fn write_log_sell_error(&self, token: &String) {
        self.write_log_entry(format!("SELL-TOKEN:{}-ERROR", token));
    }

    fn write_log_buy_error(&self, token: &String) {
        self.write_log_entry(format!("BUY-TOKEN:{}-ERROR", token));
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
        log_output: None,
    };

    let mut test_market_2 = FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
        log_output: None,
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
        log_output: None,
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
    let mut test_market_1 = Rc::new(RefCell::new(FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
        log_output: None,
    }));

    let mut test_market_2 = Rc::new(RefCell::new(FskMarket {
        goods: HashMap::new(),
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
        log_output: None,
    }));

    /* subscribe_each_other!(test_market_1.clone(), test_market_2.clone());
    assert_eq!(test_market_1.subs[0], 1); */
}

#[test]
fn test_sell() {
    let mut test_goods: HashMap<GoodKind, GoodLabel> = HashMap::new();

    test_goods.insert(
        GoodKind::EUR,
        GoodLabel {
            good_kind: GoodKind::EUR,
            quantity: 100000.,
            exchange_rate_buy: 1.,
            exchange_rate_sell: 1.,
        },
    );

    test_goods.insert(
        GoodKind::USD,
        GoodLabel {
            good_kind: GoodKind::USD,
            quantity: 1100.,
            exchange_rate_buy: 1.32,
            exchange_rate_sell: 0.895,
        },
    );

    //dummy markets creation - wrong way to create markets!
    let mut test_market_1 = Rc::new(RefCell::new(FskMarket {
        goods: test_goods,
        subs: Vec::new(),
        time: 0,
        buy_contracts_archive: ContractsArchive::new(),
        sell_contracts_archive: ContractsArchive::new(),
        log_output: None,
    }));

    //lock_sell & sell err: UnrecognizedToken
    let offer = test_market_1
        .borrow_mut()
        .get_sell_price(GoodKind::USD, 1000.)
        .unwrap();
    let result_token =
        test_market_1
            .borrow_mut()
            .lock_sell(GoodKind::USD, 1000., offer, "Sergio".to_string());
    if let Ok(_) = result_token {
        let result_sell = test_market_1
            .borrow_mut()
            .sell("token".to_string(), &mut Good::new(GoodKind::YEN, 1000.));
        if let Err(sell_error) = result_sell {
            assert_eq!(
                sell_error,
                SellError::UnrecognizedToken {
                    unrecognized_token: "token".to_string()
                }
            );
            assert_eq!(
                test_market_1
                    .borrow_mut()
                    .goods
                    .get(&GoodKind::USD)
                    .unwrap()
                    .quantity,
                1100.
            );
            assert_eq!(
                test_market_1
                    .borrow_mut()
                    .goods
                    .get(&DEFAULT_GOOD_KIND)
                    .unwrap()
                    .quantity,
                100000. - offer
            );
        }
    }

    //lock_sell & sell err: ExpiredToken
    let offer = test_market_1
        .borrow_mut()
        .get_sell_price(GoodKind::USD, 1000.)
        .unwrap();
    let result_token =
        test_market_1
            .borrow_mut()
            .lock_sell(GoodKind::USD, 1000., offer, "Sergio".to_string());
    if let Ok(token) = result_token {
        for _ in 0..10 {
            wait_one_day!(test_market_1);
        }
        let result_sell = test_market_1
            .borrow_mut()
            .sell(token.clone(), &mut Good::new(GoodKind::USD, 1000.));
        if let Err(sell_error) = result_sell {
            assert_eq!(
                sell_error,
                SellError::ExpiredToken {
                    expired_token: token.clone()
                }
            );
            assert_eq!(
                test_market_1
                    .borrow_mut()
                    .goods
                    .get(&GoodKind::USD)
                    .unwrap()
                    .quantity,
                1100.
            );
            assert_eq!(
                test_market_1
                    .borrow_mut()
                    .goods
                    .get(&DEFAULT_GOOD_KIND)
                    .unwrap()
                    .quantity,
                100000. - offer
            );
        }
    }

    //sell no err
    let offer = test_market_1
        .borrow_mut()
        .get_sell_price(GoodKind::USD, 1000.)
        .unwrap();
    let result_token =
        test_market_1
            .borrow_mut()
            .lock_sell(GoodKind::USD, 1000., offer, "Sergio".to_string());
    if let Ok(token) = result_token {
        wait_one_day!(test_market_1);
        let result_sell = test_market_1
            .borrow_mut()
            .sell(token, &mut Good::new(GoodKind::USD, 1000.));
        if let Ok(returned_good) = result_sell {
            assert!(returned_good.get_qty() >= 0.);
            assert!(
                test_market_1
                    .borrow_mut()
                    .goods
                    .get(&DEFAULT_GOOD_KIND)
                    .unwrap()
                    .quantity
                    <= 100000.
            );
            assert!(
                test_market_1
                    .borrow_mut()
                    .goods
                    .get(&GoodKind::USD)
                    .unwrap()
                    .quantity
                    >= 1100.
            );
        }
    }
}

fn main() {
    println!("Hello, world!");
}
