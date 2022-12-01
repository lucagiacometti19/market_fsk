use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use random_string::generate;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_error::{GoodKindError, GoodSplitError};
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::event::event::Event;
use unitn_market_2022::market::*;
use unitn_market_2022::market::good_label::GoodLabel;

#[derive(Debug)]
struct LockContract{
    token: String,
    good: Good,
    price: f32,
    expiry_time: u64
}

struct FskMarket {
    goods: HashMap<GoodKind, GoodLabel>,
    //the key is the token given as ret value of a buy/sell lock fn
    locked_goods_to_sell: HashMap<String, LockContract>,
    locked_goods_to_buy: HashMap<String, LockContract>,
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
         self.goods.values().cloned().collect::<Vec<GoodLabel>>()
    }

    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        /*
        kind_to_buy : type of good the trader want to buy
        quantity_to_buy : the quantity of good the trader wants to buy
        bid : the total amount of default currency the trader will give in exchange for the total amount of goods the trader will receive
        trader_name : the name of the trader
 
        Failure Cases
 
        The Lock procedure can fail if one of the following conditions is met.
 
        The specified good is already locked and the market allows just 1 lock per good. //frega un cazzo
        The maximum number of different goods that can be locked at the same time has been reached. //da capire
        The market doesn't have enough quantity available (i.e. not locked) of the requested Good.
        The bid is below the minimum acceptable by the market.
 
 
        */
 
        
 
        //let bene  = binding.iter_mut().find(|x| x.good_kind == kind_to_buy);
        let mut good = self.goods.get_mut(&kind_to_buy);
        println!("{:?}", good);
 
        match &good{
            Some(a) if quantity_to_buy <= 0.0 => return Err(LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: quantity_to_buy }),
            Some(a) if bid <= 0.0 => return Err(LockBuyError::NonPositiveBid { negative_bid: bid }),
            Some(a) if a.quantity < quantity_to_buy => return  Err(LockBuyError::InsufficientGoodQuantityAvailable { requested_good_kind: (kind_to_buy), requested_good_quantity: (quantity_to_buy), available_good_quantity: (a.quantity) }), //controllo se c'è abbastanza quantity 
            Some(a) if bid/quantity_to_buy < a.exchange_rate_buy => return Err(LockBuyError::BidTooLow { requested_good_kind: (kind_to_buy), requested_good_quantity: (quantity_to_buy), low_bid: (bid), lowest_acceptable_bid: (a.exchange_rate_buy) }),
            Some(_)=>(),
            None => return Err(LockBuyError::MaxAllowedLocksReached), //in realtà il kind non è stato trovato ma non esiste quell'errore
        }
        //Everything is okay
        good.as_mut().unwrap().quantity -= quantity_to_buy;
 

        //register (via the market-local Good Metadata) the fact that quantity quantity_to_buy of good kind_to_buy is to be bought for price bid.
        let charset = "1234567890abcdefghijklmnopqrstuwxyz";
        let token = generate(5, charset);
        println!("[{}]", token);
        
        self.locked_goods_to_buy.insert(token.to_string(), LockContract { token: token.to_string(), good: Good::new(kind_to_buy, good.as_mut().unwrap().quantity), price: bid, expiry_time: u64::MAX});
        
        println!("printo i locked_goods_to_buy {:?}", self.locked_goods_to_buy);
        //notify lock  
 
        //update price //COME?
        //println!("{:?}", good); 
        return Ok(token.to_string());
        
 
    }
 
    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
 
        //Check if it is the Default good kind
        if let Some(kind_of_good) = Some(cash.get_kind()){
            if kind_of_good != GoodKind::EUR{
                return Err(BuyError::GoodKindNotDefault { non_default_good_kind: kind_of_good });
            }
        }
 
        //Check if the token exists
        if let Some(good_lock) = self.locked_goods_to_buy.get_mut(&token){
            //Check if the token is expired (TODO!!!!!!!)
            let result = cash.split(good_lock.good.get_qty());
            /*match result {
                Some(Err(GoodSplitError::NonPositiveSplitQuantity)),
                Some(Err(GoodSplitError::NotEnoughQuantityToSplit))
                Some //tutto ok
            }*/
        }else{
            return Err(BuyError::UnrecognizedToken { unrecognized_token: (token) });
        }
 
        /*
        if by_positive_quantity <= 0. {
            return Err(GoodSplitError::NonPositiveSplitQuantity);
        }

        // a Good cannot be split by a quantity that exceeds its own
        if self.quantity - by_positive_quantity < 0. {
            return Err(GoodSplitError::NotEnoughQuantityToSplit);
        }
        */
        //notify all the markets of the transaction
        //update the price of all de goods according to the rules in the Market prices fluctuation section
        //return the pre-agreed quantity of the pre-agreed good kind
 
 
        
 
        
        return Err(BuyError::InsufficientGoodQuantity { contained_quantity: (12.0), pre_agreed_quantity: (10.0) })
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        /*let token = generate(5, "1234567890abcdefghijklmnopqrstuvwxyz");

        //1
        if quantity_to_sell < 0. {
            return Err(LockSellError::NonPositiveQuantityToSell { negative_quantity_to_sell: quantity_to_sell });
        }
        
        //2
        if offer < 0. {
            return Err(LockSellError::NonPositiveOffer { negative_offer: offer });
        }

        //5
        if self.get_budget() < offer{
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable { offered_good_kind: kind_to_sell, offered_good_quantity: offer, available_good_quantity: self.get_budget() })
        }

        //6
        let highest_acceptable_offer = self.goods.get(&kind_to_sell).unwrap().exchange_rate_sell * quantity_to_sell; //using unwrap because good_kinds are predetermined and goods map must contain the according GoodLabel
        if highest_acceptable_offer < offer {
            return Err(LockSellError::OfferTooHigh { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, high_offer: offer, highest_acceptable_offer});
        }

        self.locked_goods_to_sell.insert(token, LockContract{token, good: Good{kind: kind_to_sell, quantity: quantity_to_sell}, price: offer, expiry_time: 0 /* TODO: change time */});
        Ok(token)*/

        todo!()
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        /*let op_contract = self.locked_goods_to_sell.get(&token);

        //1
        if op_contract.is_none(){
            return Err(SellError::UnrecognizedToken { unrecognized_token: token });
        }

        let contract = op_contract.unwrap();

        //2
        if contract.expiry_time < 0 /* TODO: get market time */{
            return Err(SellError::ExpiredToken { expired_token: token });
        }

        //3
        if contract.good.get_kind() != good.get_kind(){
            return Err(SellError::WrongGoodKind { wrong_good_kind: good.get_kind(), pre_agreed_kind: contract.good.get_kind() });
        }

        //4
        if good.get_qty() < contract.good.get_qty() {
            return Err(SellError::InsufficientGoodQuantity { contained_quantity: good.get_qty(), pre_agreed_quantity: contract.good.get_qty() })
        }

        //everything checks out, the sell can proceed

        self.goods.get_mut(&good.get_kind()).unwrap().quantity += good.get_qty();

        Ok(Good{kind: DEFAULT_GOOD_KIND, quantity: contract.price})
    }*/

    todo!()
}
}

fn main() {
    let mut goods = HashMap::new();
    goods.insert(GoodKind::USD, GoodLabel{ good_kind: GoodKind::USD, quantity: 50.0, exchange_rate_buy: 3.0, exchange_rate_sell: 3.0 });
    goods.insert(GoodKind::EUR, GoodLabel{ good_kind: GoodKind::EUR, quantity: 50.0, exchange_rate_buy: 3.0, exchange_rate_sell: 3.0 });
    goods.insert(GoodKind::YEN, GoodLabel{ good_kind: GoodKind::YEN, quantity: 50.0, exchange_rate_buy: 3.0, exchange_rate_sell: 3.0 });
    
    let mut market = FskMarket{ goods: goods, locked_goods_to_sell: HashMap::new(), locked_goods_to_buy: HashMap::new() };

    market.borrow_mut().lock_buy(GoodKind::EUR, 10.0, 150.0, "Trader1".to_string());
}
