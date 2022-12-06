use chrono::Local;
use rand::Rng;
use serde::{Deserialize, Serialize};
mod tests;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{create_dir_all, File, OpenOptions};
use std::rc::Rc;

use std::io::{Read, Write};
use std::path::Path;

use random_string::generate;
use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::{
    DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE,
    DEFAULT_GOOD_KIND, STARTING_CAPITAL,
};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::*;

const LOCK_INITIAL_TTL: u64 = 9;
//higher -> greedier
const MARKET_GREEDINESS: f32 = 1.01;

const BLACK_FRIDAY_DISCOUNT: f32 = 0.20; //discount the goods of 20%

const EXCHANGE_RATE_CHANGE_RATE_OVER_TIME: f32 = 0.999;

#[derive(Serialize, Deserialize)]
struct MarketSnapshot {
    goods: HashMap<GoodKind, GoodLabel>,
    time: u64,
}

#[derive(Debug)]
struct LockContract {
    token: String,
    good: Good,
    price: f32,
    expiry_time: u64,
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

    /// This function returns an expired contract each time it's called.
    /// That contract will be removed from the struct.
    ///
    /// It is the caller responsibility to restore resources contained in the returned contract.
    ///
    /// After all expired contracts have been popped, None is returned.
    fn pop_expired(&mut self, timestamp: u64) -> Option<Rc<LockContract>> {
        //While there are still contracts...
        while let Some(contract_ref) = self.contracts_by_timestamp.front() {
            let contract = contract_ref.clone();
            //...and the first contract has expired...
            if contract.expiry_time <= timestamp {
                //...remove it from the contracts vector, as we don't need it anymore.
                self.contracts_by_timestamp.pop_front();
                //If the contract is still in the hashmap, it means that it has never been claimed, as buy and sell methods only remove claimed contracts from the hashmap.
                if self.contracts_by_token.remove(&contract.token).is_some() {
                    //If the contract has expired without being claimed, put it in the expired contracts set and return it.
                    self.expired_contracts.insert(contract.token.clone());
                    return Some(contract);
                }
                //If the contract is not in the hashmap, it means that it had been claimed. Let the 'while' cycle check the next contract in the vector.
            } else {
                break;
            }
        }
        //If we reached this statement, it means that all expired contracts have been cleared.
        None
    }
}

struct FskMarket {
    goods: HashMap<GoodKind, GoodLabel>,
    buy_contracts_archive: ContractsArchive,
    sell_contracts_archive: ContractsArchive,
    subs: Vec<Box<dyn Notifiable>>,
    log_output: RefCell<File>,
    time: u64,
}

impl FskMarket {
    fn notify(&mut self, event: Event) {
        self.on_event(event.clone());
        for sub in &mut self.subs {
            sub.on_event(event.clone());
        }
    }

    fn restore_all_lock_contracts(&mut self) -> HashMap<GoodKind, f32> {
        //remove buy contracts
        let mut old_quantities: HashMap<GoodKind, f32> = HashMap::new();
        for (_, buy_contract) in &self.buy_contracts_archive.contracts_by_token {
            let gk = buy_contract.good.get_kind();
            //save old quantity for return value
            let old_qty = self.goods.get(&gk).unwrap().quantity;
            old_quantities.insert(gk, old_qty);
            //these unwraps shouldn't fail!
            let new_gl = GoodLabel {
                good_kind: gk,
                quantity: self.goods.get(&gk).unwrap().quantity + buy_contract.good.get_qty(),
                exchange_rate_buy: self.goods.get(&gk).unwrap().exchange_rate_buy,
                exchange_rate_sell: self.goods.get(&gk).unwrap().exchange_rate_sell,
            };
            //add the lock quantity back to the market
            self.goods.insert(buy_contract.good.get_kind(), new_gl);
        }
        //remove sell contracts
        for (_, sell_contract) in &self.sell_contracts_archive.contracts_by_token {
            let gk = sell_contract.good.get_kind();
            //these unwraps shouldn't fail!
            let new_gl = GoodLabel {
                good_kind: gk,
                quantity: self.goods.get(&gk).unwrap().quantity + sell_contract.good.get_qty(),
                exchange_rate_buy: self.goods.get(&gk).unwrap().exchange_rate_buy,
                exchange_rate_sell: self.goods.get(&gk).unwrap().exchange_rate_sell,
            };
            //add the lock quantity back to the market
            self.goods.insert(sell_contract.good.get_kind(), new_gl);
        }
        old_quantities
    }

    fn update_prices(&mut self, old_quantities: HashMap<GoodKind, f32>) {
        for (gk, qty) in &old_quantities {
            if *gk != GoodKind::EUR {
                //calculate new exchange_rate_buy given the quantity bought
                let new_exchange_rate_buy = FskMarket::get_new_exchange_rate_buy(
                    self.goods.get(gk).unwrap().exchange_rate_buy,
                    self.goods.get(gk).unwrap().quantity,
                    self.goods.get(gk).unwrap().quantity - qty,
                );
                self.goods.get_mut(gk).unwrap().exchange_rate_buy = new_exchange_rate_buy;
                //calculate new exchange_rate_sell given the new exchange_rate_buy
                let new_exchange_rate_sell =
                    FskMarket::get_new_exchange_rate_sell(new_exchange_rate_buy);
                self.goods.get_mut(gk).unwrap().exchange_rate_sell = new_exchange_rate_sell;
            }
        }
    }

    fn update_price(&mut self, gk: &GoodKind, qty: f32) {
        //calculate new exchange_rate_buy given the quantity bought
        let new_exchange_rate_buy = FskMarket::get_new_exchange_rate_buy(
            self.goods.get(gk).unwrap().exchange_rate_buy,
            self.goods.get(gk).unwrap().quantity + qty,
            qty,
        );
        self.goods.get_mut(gk).unwrap().exchange_rate_buy = new_exchange_rate_buy;
        //calculate new exchange_rate_sell given the new exchange_rate_buy
        let new_exchange_rate_sell = FskMarket::get_new_exchange_rate_sell(new_exchange_rate_buy);
        self.goods.get_mut(gk).unwrap().exchange_rate_sell = new_exchange_rate_sell;
    }

    fn take_snapshot(&self, mut filename: String) {
        if filename.is_empty() {
            filename = format!("snapshots/market_FSK_snapshot_{}.json", self.time)
        }
        //copy market values to save to market snapshot
        let snapshot = MarketSnapshot {
            goods: self.goods.clone(),
            time: self.time,
        };
        let json_parser_result = serde_json::to_string(&snapshot);
        if let Ok(snapshot_json) = json_parser_result {
            if let Err(err) = create_dir_all("snapshots") {
                println!(
                    "Couldn't create snapshot directory, check error below:\n{:?}",
                    err
                );
            } else {
                //directory created succesfully
                let file_res = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(filename);
                if let Ok(mut file) = file_res {
                    if let Err(err) = file.write(snapshot_json.as_bytes()) {
                        println!(
                            "Couldn't write in snapshot file, check error below:\n{:?}",
                            err
                        );
                    }
                } else if let Err(err) = file_res {
                    println!(
                        "Couldn't create snapshot file, check error below:\n{:?}",
                        err
                    );
                }
            }
        } else if let Err(err) = json_parser_result {
            println!("Serde couldn't parse the market snapshot:\n{:?}", err);
        }
    }

    fn initialize_log_file(market_name: String) -> RefCell<File> {
        let log_file_name = format!("log_{}.txt", market_name);
        RefCell::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_name)
                .unwrap(),
        )
    }

    fn write_log_entry(&self, entry: String) {
        if self
            .log_output
            .borrow_mut()
            .write(
                format!(
                    "{}|{}|{}\n",
                    self.get_name(),
                    Local::now().format("%y:%m:%d:%H:%M:%S:%3f"),
                    entry
                )
                .as_bytes(),
            )
            .is_err()
        {
            println!("{}: Couldn't write to log file", self.get_name())
        }
        //YY:MM:DD:HH:MM:SEC:MSES
    }

    fn write_log_market_init(&self) {
        self.write_log_entry(format!("\nMARKET_INITIALIZATION\nEUR: {:+e}\nUSD: {:+e}\nYEN: {:+e}\nYUAN: {:+e}\nEND_MARKET_INITIALIZATION",
            self.goods.get(&GoodKind::EUR).unwrap().quantity,
            self.goods.get(&GoodKind::USD).unwrap().quantity,
            self.goods.get(&GoodKind::YEN).unwrap().quantity,
            self.goods.get(&GoodKind::YUAN).unwrap().quantity
        ));
    }

    fn write_log_buy_ok(
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

    fn write_log_lock_buy_error(
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

    fn write_log_sell_ok(
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

    fn get_new_exchange_rate_sell(exchange_rate_buy: f32) -> f32 {
        exchange_rate_buy / MARKET_GREEDINESS
    }

    fn get_new_exchange_rate_buy(
        current_exchange_rate_buy: f32,
        current_quantity: f32,
        quantity_to_buy: f32,
    ) -> f32 {
        current_exchange_rate_buy * current_quantity / (current_quantity - quantity_to_buy)
    }
}

impl Drop for FskMarket {
    fn drop(&mut self) {
        //removes all the locks and restore all good quantities before snapshot
        let old_quantities = self.restore_all_lock_contracts();
        //update prices accordingly
        self.update_prices(old_quantities);
        //take snapshot
        self.take_snapshot("snapshots/market_FSK_snapshot_at_drop.json".to_string())
    }
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
            EventKind::Wait => {
                //decrease exchange rate over time
                for (good_kind, good_label) in &mut self.goods {
                    match *good_kind {
                        DEFAULT_GOOD_KIND => {}
                        _ => {
                            good_label.exchange_rate_buy *= EXCHANGE_RATE_CHANGE_RATE_OVER_TIME;
                            good_label.exchange_rate_sell =
                                FskMarket::get_new_exchange_rate_sell(good_label.exchange_rate_buy)
                        }
                    }
                }
            }
        }

        //restore locked default currency for expired sell
        while let Some(expired_contract) = self.sell_contracts_archive.pop_expired(self.time) {
            /* contract_expired_type = 1;
            expired_gk = DEFAULT_GOOD_KIND; */
            self.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap().quantity += expired_contract.price;
        }

        //restore locked good for expired buyout
        while let Some(expired_contract) = self.buy_contracts_archive.pop_expired(self.time) {
            /* contract_expired_type = 2;
            expired_gk = expired_contract.good.get_kind(); */
            self.goods
                .get_mut(&expired_contract.good.get_kind())
                .unwrap()
                .quantity += expired_contract.good.get_qty();
        }

        //black_friday_handling
        //black friday begins
        if self.time % 7 == 4 {
            for (good_kind, good_label) in &mut self.goods {
                match *good_kind {
                    DEFAULT_GOOD_KIND => {}
                    _ => {
                        good_label.exchange_rate_buy *= 1. - BLACK_FRIDAY_DISCOUNT;
                        good_label.exchange_rate_sell =
                            FskMarket::get_new_exchange_rate_sell(good_label.exchange_rate_buy)
                    }
                }
            }
        }
        //black friday ends
        if self.time % 7 == 5 {
            for (good_kind, good_label) in &mut self.goods {
                match *good_kind {
                    DEFAULT_GOOD_KIND => {}
                    _ => {
                        good_label.exchange_rate_buy /= 1. - BLACK_FRIDAY_DISCOUNT;
                        good_label.exchange_rate_sell =
                            FskMarket::get_new_exchange_rate_sell(good_label.exchange_rate_buy)
                    }
                }
            }
        }

        //take snapshot and save to file for visualizer
        self.take_snapshot(String::new());
    }
}

impl Market for FskMarket {
    fn new_random() -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        let mut rng = rand::thread_rng();
        //-0.1 to make up for float rounding errors
        let mut remainder = STARTING_CAPITAL - 0.1;

        let mut temp = rng.gen_range(0..remainder as i32);
        let yen_qty = temp as f32 * DEFAULT_EUR_YEN_EXCHANGE_RATE;
        remainder -= temp as f32;

        temp = rng.gen_range(0..remainder as i32);
        let usd_qty = temp as f32 * DEFAULT_EUR_USD_EXCHANGE_RATE;
        remainder -= temp as f32;

        temp = rng.gen_range(0..remainder as i32);
        let yuan_qty = temp as f32 * DEFAULT_EUR_YUAN_EXCHANGE_RATE;
        remainder -= temp as f32;

        let eur_qty = remainder;
        FskMarket::new_with_quantities(eur_qty, yen_qty, usd_qty, yuan_qty)
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        let mut goods_result = HashMap::new();
        goods_result.insert(
            GoodKind::EUR,
            GoodLabel {
                good_kind: GoodKind::EUR,
                quantity: eur,
                exchange_rate_buy: 1.,
                exchange_rate_sell: 1. / MARKET_GREEDINESS,
            },
        );
        goods_result.insert(
            GoodKind::YEN,
            GoodLabel {
                good_kind: GoodKind::YEN,
                quantity: yen,
                exchange_rate_buy: 1. / DEFAULT_EUR_YEN_EXCHANGE_RATE,
                exchange_rate_sell: 1. / (DEFAULT_EUR_YEN_EXCHANGE_RATE * MARKET_GREEDINESS),
            },
        );
        goods_result.insert(
            GoodKind::USD,
            GoodLabel {
                good_kind: GoodKind::USD,
                quantity: usd,
                exchange_rate_buy: 1. / DEFAULT_EUR_USD_EXCHANGE_RATE,
                exchange_rate_sell: 1. / (DEFAULT_EUR_USD_EXCHANGE_RATE * MARKET_GREEDINESS),
            },
        );
        goods_result.insert(
            GoodKind::YUAN,
            GoodLabel {
                good_kind: GoodKind::YUAN,
                quantity: yuan,
                exchange_rate_buy: 1. / DEFAULT_EUR_YUAN_EXCHANGE_RATE,
                exchange_rate_sell: 1. / (DEFAULT_EUR_YUAN_EXCHANGE_RATE * MARKET_GREEDINESS),
            },
        );

        let new_market = Rc::new(RefCell::new(FskMarket {
            goods: goods_result,
            buy_contracts_archive: ContractsArchive::new(),
            sell_contracts_archive: ContractsArchive::new(),
            subs: vec![],
            log_output: FskMarket::initialize_log_file("FSK".to_string()),
            time: 0,
        }));
        //log market init
        new_market.borrow().write_log_market_init();
        //take the first snapshot
        new_market.borrow().take_snapshot(String::new());
        //return it
        new_market
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        let file_pat = path;
        let exists = Path::new(file_pat).exists();
        if !exists {
            return FskMarket::new_random();
        }

        let file_open_res = OpenOptions::new().read(true).open(path);
        //check if file open threw error
        if let Ok(mut file) = file_open_res {
            let mut market_json = String::new();
            //if not try to read its content
            if let Ok(_) = file.read_to_string(&mut market_json) {
                //if read is succesfull try to parse the content into a MarketSnapshot
                let market_parse_res: Result<MarketSnapshot, _> =
                    serde_json::from_str(&market_json[..]);
                if let Ok(market) = market_parse_res {
                    //if parse is succesfull create a new FskMarket
                    let new_market = Rc::new(RefCell::new(FskMarket {
                        goods: market.goods,
                        buy_contracts_archive: ContractsArchive::new(),
                        sell_contracts_archive: ContractsArchive::new(),
                        subs: vec![],
                        log_output: FskMarket::initialize_log_file("FSK".to_string()),
                        time: market.time,
                    }));
                    //log market init
                    new_market.borrow().write_log_market_init();
                    //take the first snapshot
                    new_market.borrow().take_snapshot(String::new());
                } else if let Err(err) = market_parse_res {
                    println!(
                        "Couldn't parse the market snapshot, check error below:\n{:?}",
                        err
                    );
                }
            }
        } else if let Err(err) = file_open_res {
            println!("Couldn't open snapshot file, check error below:\n{:?}", err);
        }
        //in case of error just return a random market
        return FskMarket::new_random();
    }

    fn get_name(&self) -> &'static str {
        "FSK"
    }

    fn get_budget(&self) -> f32 {
        self.goods.get(&DEFAULT_GOOD_KIND).unwrap().quantity
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        let mut good_quantity = 0.;

        //the quantity is not positive
        if quantity < 0. {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        //the quantity the trader is asking to buy is lower than the quantity the market owns
        if let Some(good) = self.goods.get(&kind) {
            good_quantity = good.quantity;
            if good.quantity > quantity {
                //the market has enough quantity
                return Ok(FskMarket::get_new_exchange_rate_buy(
                    good.exchange_rate_buy,
                    good_quantity,
                    quantity,
                ) * quantity);
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

        let maximum_price = quantity * self.goods.get(&kind).unwrap().exchange_rate_sell;
        //how much money the market pay (at max) for the good

        let available_default_good = self.get_budget();

        Ok(maximum_price.min(available_default_good))
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        let mut res = Vec::new();
        for (gk, good_label) in &self.goods {
            let mut new_good_label = good_label.clone();
            if *gk != DEFAULT_GOOD_KIND {
                //price impact calculation
                new_good_label.exchange_rate_buy = FskMarket::get_new_exchange_rate_buy(
                    good_label.exchange_rate_buy,
                    good_label.quantity,
                    1.,
                );
            }
            res.push(new_good_label);
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
        //1
        if quantity_to_buy < 0. {
            self.write_log_lock_buy_error(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::NonPositiveQuantityToBuy {
                negative_quantity_to_buy: quantity_to_buy,
            });
        }

        //2
        if bid < 0. {
            self.write_log_lock_buy_error(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::NonPositiveBid { negative_bid: bid });
        }
        //get immutable reference so there are no borrow errors
        let good = self.goods.get(&kind_to_buy).unwrap(); //assume that goods always contains every goodkind

        //5
        if good.quantity < quantity_to_buy {
            self.write_log_lock_buy_error(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                available_good_quantity: good.quantity,
            });
        }

        //unwrap won't panic
        let get_buy_price_result = self
            .get_buy_price(kind_to_buy.clone(), quantity_to_buy)
            .unwrap();

        //6
        if bid < get_buy_price_result {
            self.write_log_lock_buy_error(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::BidTooLow {
                requested_good_kind: kind_to_buy.clone(),
                requested_good_quantity: quantity_to_buy,
                low_bid: bid,
                lowest_acceptable_bid: get_buy_price_result,
            });
        }

        //this time we need the mutable reference
        let mut good = self.goods.get_mut(&kind_to_buy).unwrap(); //assume that goods always contains every goodkind
        good.quantity -= quantity_to_buy;

        //create the token
        let token = self.buy_contracts_archive.new_token();

        //register (via the market-local Good Metadata) the fact that quantity quantity_to_buy of good kind_to_buy is to be bought for price bid.
        self.buy_contracts_archive
            .add_contract(&Rc::new(LockContract {
                token: token.to_string(),
                good: Good::new(kind_to_buy.clone(), quantity_to_buy),
                price: bid,
                expiry_time: self.time + LOCK_INITIAL_TTL,
            }));
        //log
        self.write_log_buy_ok(trader_name, kind_to_buy, quantity_to_buy, bid, &token);

        //notify all the markets of the lock buy
        self.notify(Event {
            kind: EventKind::LockedBuy,
            good_kind: kind_to_buy.clone(),
            quantity: quantity_to_buy,
            price: bid,
        });

        return Ok(token.to_string());
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        //check if the token is valid or expired or unrecognized
        let op_contract = self.buy_contracts_archive.contracts_by_token.get(&token);

        //1
        if op_contract.is_none() {
            self.write_log_buy_error(&token);
            if self
                .buy_contracts_archive
                .expired_contracts
                .contains(&token)
            {
                return Err(BuyError::ExpiredToken {
                    expired_token: token,
                });
            }

            return Err(BuyError::UnrecognizedToken {
                unrecognized_token: token,
            });
        }

        let contract = op_contract.unwrap();
        let contract_price = contract.price;

        //2
        if contract.expiry_time <= self.time {
            self.write_log_buy_error(&token);
            return Err(BuyError::ExpiredToken {
                expired_token: token,
            });
        }

        //3
        if cash.get_kind() != DEFAULT_GOOD_KIND {
            self.write_log_buy_error(&token);
            return Err(BuyError::GoodKindNotDefault {
                non_default_good_kind: cash.get_kind(),
            });
        }

        //4
        if cash.get_qty() < contract_price {
            self.write_log_buy_error(&token);
            return Err(BuyError::InsufficientGoodQuantity {
                contained_quantity: cash.get_qty(),
                pre_agreed_quantity: contract_price,
            });
        }

        //everything checks out, the buy can proceed
        //removing the pre-agreed quantity from cash
        cash.split(contract_price);

        //put the pre-agreed quantity in the market
        self.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap().quantity += contract_price;

        let good_to_return = Good::new(contract.good.get_kind(), contract.good.get_qty());

        //update the price of all de goods according to the rules in the Market prices fluctuation section
        //new exchange rates of the traded good
        let gk = &contract.good.get_kind();
        self.update_price(gk, contract.good.get_qty());

        //log
        self.write_log_entry(format!("BUY-TOKEN:{}-OK", token));

        //remove the corresponding contract
        self.buy_contracts_archive.consume_contract(&token);

        //notify all the markets of the transaction
        self.notify(Event {
            kind: (EventKind::Bought),
            good_kind: good_to_return.get_kind(),
            quantity: good_to_return.get_qty(),
            price: contract_price,
        });

        Ok(good_to_return)
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
                offered_good_quantity: quantity_to_sell,
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

        //create token
        let token = self.sell_contracts_archive.new_token();

        //add the contract
        self.sell_contracts_archive
            .add_contract(&Rc::new(LockContract {
                token: token.clone(),
                good: Good::new(kind_to_sell, quantity_to_sell),
                price: offer,
                expiry_time: self.time + LOCK_INITIAL_TTL,
            }));

        //log
        self.write_log_sell_ok(trader_name, kind_to_sell, quantity_to_sell, offer, &token);

        //notify all the markets of the lock sell
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
        let good_to_return = Good::new(DEFAULT_GOOD_KIND, contract.price); //don't need to decrease owned good, already did that in lock_sell

        //add the good the trader gave us to the supply of the corresponding good in the market
        //assume goods contains every goodkind: unwrap is safe
        self.goods.get_mut(&good.get_kind()).unwrap().quantity += contract.good.get_qty();
        //split the good the trader gave to the market - in case he sent more than previously decided by the lock
        good.split(contract.good.get_qty());

        //update the price of all de goods according to the rules in the Market prices fluctuation section
        //new exchange rates of the traded good
        let gk = &contract.good.get_kind();
        if *gk != GoodKind::EUR {
            self.update_price(gk, -contract.good.get_qty());
        }

        //log
        self.write_log_entry(format!("SELL-TOKEN:{}-OK", token));

        //remove the corresponding contract
        self.sell_contracts_archive.consume_contract(&token);

        //notify all the markets of the transaction
        self.notify(Event {
            kind: EventKind::Sold,
            good_kind: good.get_kind(),
            quantity: good.get_qty(),
            price: good_to_return.get_qty(),
        });

        Ok(good_to_return)
    }
}

fn main() {
    FskMarket::new_file("data.txt");
}
