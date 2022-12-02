#[cfg(test)]
mod test {

    use crate::wait_one_day;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    //import here the market_test module and the Market trait
    use unitn_market_2022::market::market_test;
    use unitn_market_2022::market::*;
    //import here your implementation of the market
    use super::super::FskMarket;
    //make an alias to your market
    type MarketType = FskMarket;
    //test every aspect of your market using the generic function
    #[test]
    fn tests() {
        market_test::test_name::<MarketType>();
        market_test::test_new_random::<MarketType>();
        market_test::test_get_buy_price_insufficient_qty_error::<MarketType>();
        market_test::test_get_buy_price_non_positive_error::<MarketType>();
        market_test::test_get_buy_price_success::<MarketType>();
        //market_test::test_get_sell_price_insufficient_qty_error::<MarketType>();
        market_test::test_get_sell_price_non_positive_error::<MarketType>();
        market_test::test_get_sell_price_success::<MarketType>();
        //@todo!(market deadlock prevention)
        //market_test::test_deadlock_prevention::<MarketType>();
        test_sell_success::<MarketType>();
        test_sell_err_unrecognized_token::<MarketType>();
        test_sell_err_expired_token::<MarketType>();
        test_sell_err_wrong_good_kind::<MarketType>();
        test_sell_err_insufficient_good_quantity::<MarketType>();
    }

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is a **positive or zero** quantity of `DEFAULT_GOOD_KIND`.
    ///2. the quantity of the *default good* in the market **does not** increase.
    ///4. the quantity of the *traded good* in the market **does not** decrease.
    ///
    ///Since the market is initialized with `new_random`, which is most
    ///likely non-deterministic, this test may fail unexpectedly, but should
    ///catch bugs in the long run.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_success<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let market = T::new_random();

        //save starting quantities
        let mut default_good_kind_starting_quantity = 0.;
        let mut traded_good_kind_starting_quantity = 0.;
        for good_label in &market.borrow().get_goods() {
            match good_label.good_kind {
                DEFAULT_GOOD_KIND => default_good_kind_starting_quantity = good_label.quantity,
                GoodKind::USD => traded_good_kind_starting_quantity = good_label.quantity,
                _ => (),
            }
        }

        //call to unwrap since we assume get_sell_price doesn't throws error
        let offer = market
            .borrow_mut()
            .get_sell_price(GoodKind::USD, traded_good_kind_starting_quantity / 2.)
            .unwrap();
        let result_token = market.borrow_mut().lock_sell(
            GoodKind::USD,
            traded_good_kind_starting_quantity / 2.,
            offer,
            "Sergio".to_string(),
        );
        if let Ok(token) = result_token {
            //lock_sell doesn't throw errors
            let result_sell = market.borrow_mut().sell(
                token,
                &mut Good::new(GoodKind::USD, traded_good_kind_starting_quantity / 2.),
            );
            if let Ok(returned_good) = result_sell {
                //check returned good quantity >= .0
                assert!(returned_good.get_qty() >= 0.);
                //check returned good is the DEFAULT_GOOD_KIND
                assert_eq!(returned_good.get_kind(), DEFAULT_GOOD_KIND);
                //get market goods and check quantities
                let market_goods = market.borrow_mut().get_goods();
                let mut default_good_kind_quantity = 0.;
                let mut traded_good_quantity = 0.;
                for good_label in &market_goods {
                    match good_label.good_kind {
                        DEFAULT_GOOD_KIND => default_good_kind_quantity = good_label.quantity,
                        GoodKind::USD => traded_good_quantity = good_label.quantity,
                        _ => (),
                    }
                }
                //check default good is less or equal then before
                assert!(default_good_kind_quantity <= default_good_kind_starting_quantity);
                //check traded good is greater or equal then before
                assert!(traded_good_quantity >= traded_good_kind_starting_quantity);
            } else if let Err(sell_error) = result_sell {
                //sell threw some kind of errors
                assert_eq!(0, 1, "sell returned an error: {:?}", sell_error);
            }
        } else if let Err(lock_sell_error) = result_token {
            //lock_sell threw some kind of errors
            assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
        }
    }

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::UnrecognizedToken`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Since the market is initialized with `new_random`, which is most
    ///likely non-deterministic, this test may fail unexpectedly, but should
    ///catch bugs in the long run.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_unrecognized_token<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let market = T::new_random();

        //save starting quantities
        let mut default_good_kind_starting_quantity = 0.;
        let mut traded_good_kind_starting_quantity = 0.;
        for good_label in &market.borrow().get_goods() {
            match good_label.good_kind {
                DEFAULT_GOOD_KIND => default_good_kind_starting_quantity = good_label.quantity,
                GoodKind::USD => traded_good_kind_starting_quantity = good_label.quantity,
                _ => (),
            }
        }

        //call to unwrap since we assume get_sell_price doesn't throws error
        let offer = market
            .borrow_mut()
            .get_sell_price(GoodKind::USD, traded_good_kind_starting_quantity / 2.)
            .unwrap();
        let result_token = market.borrow_mut().lock_sell(
            GoodKind::USD,
            traded_good_kind_starting_quantity / 2.,
            offer,
            "Sergio".to_string(),
        );
        if let Ok(_) = result_token {
            //lock_sell doesn't throw errors
            //call sell with a wrong token
            let result_sell = market.borrow_mut().sell(
                "token".to_string(),
                &mut Good::new(GoodKind::USD, traded_good_kind_starting_quantity / 2.),
            );
            if let Err(sell_error) = result_sell {
                //check that the returned value is an error of type SellError::UnrecognizedToken
                assert_eq!(
                    sell_error,
                    SellError::UnrecognizedToken {
                        unrecognized_token: "token".to_string()
                    }
                );
                //get market goods and check quantities
                let market_goods = market.borrow_mut().get_goods();
                let mut default_good_kind_quantity = 0.;
                let mut traded_good_quantity = 0.;
                for good_label in &market_goods {
                    match good_label.good_kind {
                        DEFAULT_GOOD_KIND => default_good_kind_quantity = good_label.quantity,
                        GoodKind::USD => traded_good_quantity = good_label.quantity,
                        _ => (),
                    }
                }
                //check that the default good quantity didn't change
                assert_eq!(
                    default_good_kind_starting_quantity,
                    default_good_kind_quantity
                );
                //check that the traded good quantity didn't change
                assert_eq!(traded_good_kind_starting_quantity, traded_good_quantity);
            } else if let Ok(returned_good) = result_sell {
                //sell dind't return an error
                assert_eq!(
                    0, 1,
                    "sell returned a good even if it was supposed to fail.\nGood returned: {:?}",
                    returned_good
                );
            }
        } else if let Err(lock_sell_error) = result_token {
            //lock_sell threw some kind of errors
            assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
        }
    }

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::ExpiredToken`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Since the market is initialized with `new_random`, which is most
    ///likely non-deterministic, this test may fail unexpectedly, but should
    ///catch bugs in the long run.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_expired_token<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let market = T::new_random();

        //save starting quantities
        let mut default_good_kind_starting_quantity = 0.;
        let mut traded_good_kind_starting_quantity = 0.;
        for good_label in &market.borrow().get_goods() {
            match good_label.good_kind {
                DEFAULT_GOOD_KIND => default_good_kind_starting_quantity = good_label.quantity,
                GoodKind::USD => traded_good_kind_starting_quantity = good_label.quantity,
                _ => (),
            }
        }

        //call to unwrap since we assume get_sell_price doesn't throws error
        let offer = market
            .borrow_mut()
            .get_sell_price(GoodKind::USD, traded_good_kind_starting_quantity / 2.)
            .unwrap();
        let result_token = market.borrow_mut().lock_sell(
            GoodKind::USD,
            traded_good_kind_starting_quantity / 2.,
            offer,
            "Sergio".to_string(),
        );
        if let Ok(token) = result_token {
            //lock_sell doesn't throw errors
            //wait 15 days since that's the max TTL of a lock
            for _ in 0..15 {
                wait_one_day!(market);
            }
            //call to sell with an expired token
            let result_sell = market.borrow_mut().sell(
                token.clone(),
                &mut Good::new(GoodKind::USD, traded_good_kind_starting_quantity / 2.),
            );
            if let Err(sell_error) = result_sell {
                //check that the returned value is an error of type SellError::ExpiredToken
                assert_eq!(
                    sell_error,
                    SellError::ExpiredToken {
                        expired_token: token.clone()
                    }
                );
                //get market goods and check quantities
                let market_goods = market.borrow_mut().get_goods();
                let mut default_good_kind_quantity = 0.;
                let mut traded_good_quantity = 0.;
                for good_label in &market_goods {
                    match good_label.good_kind {
                        DEFAULT_GOOD_KIND => default_good_kind_quantity = good_label.quantity,
                        GoodKind::USD => traded_good_quantity = good_label.quantity,
                        _ => (),
                    }
                }
                //check that the default good quantity didn't change
                assert_eq!(
                    default_good_kind_starting_quantity,
                    default_good_kind_quantity
                );
                //check that the traded good quantity didn't change
                assert_eq!(traded_good_kind_starting_quantity, traded_good_quantity);
            } else if let Ok(returned_good) = result_sell {
                //sell dind't return an error
                assert_eq!(
                    0, 1,
                    "sell returned a good even if it was supposed to fail.\nGood returned: {:?}",
                    returned_good
                );
            }
        } else if let Err(lock_sell_error) = result_token {
            //lock_sell threw some kind of errors
            assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
        }
    }

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::WrongGoodKind`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Since the market is initialized with `new_random`, which is most
    ///likely non-deterministic, this test may fail unexpectedly, but should
    ///catch bugs in the long run.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_wrong_good_kind<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let market = T::new_random();

        //save starting quantities
        let mut default_good_kind_starting_quantity = 0.;
        let mut traded_good_kind_starting_quantity = 0.;
        for good_label in &market.borrow().get_goods() {
            match good_label.good_kind {
                DEFAULT_GOOD_KIND => default_good_kind_starting_quantity = good_label.quantity,
                GoodKind::USD => traded_good_kind_starting_quantity = good_label.quantity,
                _ => (),
            }
        }

        //call to unwrap since we assume get_sell_price doesn't throws error
        let offer = market
            .borrow_mut()
            .get_sell_price(GoodKind::USD, traded_good_kind_starting_quantity / 2.)
            .unwrap();
        let result_token = market.borrow_mut().lock_sell(
            GoodKind::USD,
            traded_good_kind_starting_quantity / 2.,
            offer,
            "Sergio".to_string(),
        );
        if let Ok(token) = result_token {
            //lock_sell doesn't throw errors
            let result_sell = market.borrow_mut().sell(
                token.clone(),
                &mut Good::new(GoodKind::USD, traded_good_kind_starting_quantity / 2.),
            );
            if let Err(sell_error) = result_sell {
                //check that the returned value is an error of type SellError::WrongGoodKind
                assert_eq!(
                    sell_error,
                    SellError::WrongGoodKind {
                        wrong_good_kind: GoodKind::YEN,
                        pre_agreed_kind: GoodKind::USD
                    }
                );
                //get market goods and check quantities
                let market_goods = market.borrow_mut().get_goods();
                let mut default_good_kind_quantity = 0.;
                let mut traded_good_quantity = 0.;
                for good_label in &market_goods {
                    match good_label.good_kind {
                        DEFAULT_GOOD_KIND => default_good_kind_quantity = good_label.quantity,
                        GoodKind::USD => traded_good_quantity = good_label.quantity,
                        _ => (),
                    }
                }
                //check that the default good quantity didn't change
                assert_eq!(
                    default_good_kind_starting_quantity,
                    default_good_kind_quantity
                );
                //check that the traded good quantity didn't change
                assert_eq!(traded_good_kind_starting_quantity, traded_good_quantity);
            } else if let Ok(returned_good) = result_sell {
                //sell dind't return an error
                assert_eq!(
                    0, 1,
                    "sell returned a good even if it was supposed to fail.\nGood returned: {:?}",
                    returned_good
                );
            }
        } else if let Err(lock_sell_error) = result_token {
            //lock_sell threw some kind of errors
            assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
        }
    }

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::InsufficientGoodQuantity`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Since the market is initialized with `new_random`, which is most
    ///likely non-deterministic, this test may fail unexpectedly, but should
    ///catch bugs in the long run.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_insufficient_good_quantity<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let market = T::new_random();

        //save starting quantities
        let mut default_good_kind_starting_quantity = 0.;
        let mut traded_good_kind_starting_quantity = 0.;
        for good_label in &market.borrow().get_goods() {
            match good_label.good_kind {
                DEFAULT_GOOD_KIND => default_good_kind_starting_quantity = good_label.quantity,
                GoodKind::USD => traded_good_kind_starting_quantity = good_label.quantity,
                _ => (),
            }
        }

        //call to unwrap since we assume get_sell_price doesn't throws error
        let offer = market
            .borrow_mut()
            .get_sell_price(GoodKind::USD, traded_good_kind_starting_quantity / 2.)
            .unwrap();
        let result_token = market.borrow_mut().lock_sell(
            GoodKind::USD,
            traded_good_kind_starting_quantity / 2.,
            offer,
            "Sergio".to_string(),
        );
        if let Ok(token) = result_token {
            //lock_sell doesn't throw errors
            //call to sell with less USD than the pre-agreed quantity in the contract
            let result_sell = market.borrow_mut().sell(
                token.clone(),
                &mut Good::new(GoodKind::USD, traded_good_kind_starting_quantity / 3.),
            );
            if let Err(sell_error) = result_sell {
                //check that the returned value is an error of type SellError::InsufficientGoodQuantity
                assert_eq!(
                    sell_error,
                    SellError::InsufficientGoodQuantity {
                        contained_quantity: traded_good_kind_starting_quantity / 3.,
                        pre_agreed_quantity: traded_good_kind_starting_quantity / 2.
                    }
                );
                //get market goods and check quantities
                let market_goods = market.borrow_mut().get_goods();
                let mut default_good_kind_quantity = 0.;
                let mut traded_good_quantity = 0.;
                for good_label in &market_goods {
                    match good_label.good_kind {
                        DEFAULT_GOOD_KIND => default_good_kind_quantity = good_label.quantity,
                        GoodKind::USD => traded_good_quantity = good_label.quantity,
                        _ => (),
                    }
                }
                //check that the default good quantity didn't change
                assert_eq!(
                    default_good_kind_starting_quantity,
                    default_good_kind_quantity
                );
                //check that the traded good quantity didn't change
                assert_eq!(traded_good_kind_starting_quantity, traded_good_quantity);
            } else if let Ok(returned_good) = result_sell {
                //sell dind't return an error
                assert_eq!(
                    0, 1,
                    "sell returned a good even if it was supposed to fail.\nGood returned: {:?}",
                    returned_good
                );
            }
        } else if let Err(lock_sell_error) = result_token {
            //lock_sell threw some kind of errors
            assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
        }
    }
}
