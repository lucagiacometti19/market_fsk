#[cfg(test)]
mod test {
    use unitn_market_2022::{
        good::{good::Good, good_kind::GoodKind},
        market::{market_test, Market, SellError}, wait_one_day,
    };

    //import here the market_test module and the Market trait
    //import here your implementation of the market
    use super::super::FskMarket;
    //make an alias to your market 37 TEST
    type MarketType = FskMarket;
    //test every aspect of your market using the generic function
    #[test]
    fn tests() {
        /* let m = MarketType::new_with_quantities(10000., 10000., 10000., 10000.);
        let bid = m.borrow().get_buy_price(GoodKind::USD, 10.).unwrap();
        let token = m
            .borrow_mut()
            .lock_buy(GoodKind::USD, 10., bid, "Sergio".to_string())
            .unwrap();
        println!("token: {}", token);
        let mut cash = Good::new(GoodKind::EUR, bid);
        let purchase = m.borrow_mut().buy(token, &mut cash).unwrap();
        println!(
            "Comprato {} USD con {} EUR e resto {}",
            purchase.get_qty(),
            bid,
            cash
        );

        let offer = m.borrow().get_sell_price(GoodKind::USD, 10.).unwrap();
        let token = m
            .borrow_mut()
            .lock_sell(GoodKind::USD, 10., offer, "Sergio".to_string())
            .unwrap();
        println!("token: {}", token);
        let mut good_to_sell = Good::new(GoodKind::USD, 10.);
        let gain = m.borrow_mut().sell(token, &mut good_to_sell).unwrap();
        println!(
            "Venduto {} USD per {} EUR e mi è rimasto {} USD",
            10., gain, good_to_sell
        );
        drop(m.borrow_mut()); */
        /* let m = FskMarket::new_file("snapshots/market_FSK_snapshot.json");
        drop(m.borrow_mut()); */
        //market_test::test_get_name::<MarketType>();
        ////test new_random
        //market_test::test_new_random::<MarketType>();
        //market_test::new_random_should_not_exceeed_starting_capital::<MarketType>();
        ////test new with quantities
        //market_test::should_initialize_with_right_quantity::<MarketType>();
        ////test get_buy_price
        //market_test::test_get_buy_price_insufficient_qty_error::<MarketType>();
        //market_test::test_get_buy_price_non_positive_error::<MarketType>();
        //market_test::test_get_buy_price_success::<MarketType>();
        ////test get_sell_price
        //market_test::test_get_sell_price_non_positive_error::<MarketType>();
        //market_test::test_get_sell_price_success::<MarketType>();
        ////test deadlock prevention
        //market_test::test_deadlock_prevention::<MarketType>();
        ////test sell
        //market_test::test_sell_expired_token::<MarketType>();
        //market_test::test_sell_insufficient_good_quantity::<MarketType>();
        //market_test::test_sell_unrecognized_token::<MarketType>();
        //market_test::test_sell_wrong_good_kind::<MarketType>();
        /* market_test:: */test_sell_success::<MarketType>();
        ////test buy
        //market_test::test_buy_good_kind_not_default::<MarketType>();
        //market_test::test_buy_insufficient_good_quantity::<MarketType>();
        //market_test::test_buy_unrecognized_token::<MarketType>();
        //market_test::test_buy_success::<MarketType>();
        ////test price_change
        //market_test::price_changes_waiting::<MarketType>();
        //market_test::test_price_change_after_buy::<MarketType>();
        //market_test::test_price_change_after_sell::<MarketType>();
        ////test get budget
        //market_test::test_get_budget::<MarketType>();
        ////test get buy price
        //market_test::test_get_buy_price_insufficient_qty_error::<MarketType>();
        //market_test::test_get_buy_price_non_positive_error::<MarketType>();
        //market_test::test_get_buy_price_success::<MarketType>();
        ////test get goods
        ////market_test::test_get_goods::<MarketType>(); echange rate of euro is not always 1!
        ////test lock buy
        //market_test::test_lock_buy_bid_too_low::<MarketType>();
        //market_test::test_lock_buy_insufficient_good_quantity_available::<MarketType>();
        //market_test::test_lock_buy_non_positive_bid::<MarketType>();
        //market_test::test_lock_buy_non_positive_quantity_to_buy::<MarketType>();
        ////test lock sell
        //market_test::test_lock_sell_insufficientDefaultGoodQuantityAvailable::<MarketType>();
        //market_test::test_lock_sell_nonPositiveOffer::<MarketType>();
        //market_test::test_lock_sell_offerTooHigh::<MarketType>();
        //market_test::test_working_function_lock_sell_token::<MarketType>();
    }

    fn test_sell_success<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let kinds = vec![GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        let market = T::new_with_quantities(10000., 10000., 10000., 10000.);
        for kind in kinds {
            let offer_res = market.borrow_mut().get_sell_price(kind, 1000.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind, 1000., offer, "Sergio".to_string());
                if let Ok(token) = result_token {
                    //lock_sell didn't throw error
                    let result_sell = market.borrow_mut().sell(token, &mut Good::new(kind, 1000.));
                    if let Ok(returned_good) = result_sell {
                        //check returned good quantity >= .0
                        assert!(returned_good.get_qty() >= 0.);
                        //check returned good is the DEFAULT_GOOD_KIND
                        assert_eq!(returned_good.get_kind(), DEFAULT_GOOD_KIND);
                    } else if let Err(sell_error) = result_sell {
                        //sell threw some kind of error
                        assert_eq!(0, 1, "sell returned an error: {:?}", sell_error);
                    }
                } else if let Err(lock_sell_error) = result_token {
                    //lock_sell threw some kind of error
                    assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
                }
            } else if let Err(get_sell_price_err) = offer_res {
                //get_sell_price threw some kind of error
                assert_eq!(
                    0, 1,
                    "get_sell_price returned an error: {:?}",
                    get_sell_price_err
                );
            }
        }
    }

    fn test_sell_expired_token<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let kinds = vec![GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        let market = T::new_with_quantities(1000., 1000., 1000., 1000.);
        for kind in kinds {

            let offer_res = market.borrow_mut().get_sell_price(kind, 100.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind, 100., offer, "Sergio".to_string());
                if let Ok(token) = result_token {
                    //lock_sell didn't throw error
                    //wait 15 days since that's the max TTL of a lock
                    for _ in 0..15 {
                        wait_one_day!(market);
                    }
                    //call to sell with an expired token
                    let result_sell = market
                        .borrow_mut()
                        .sell(token.clone(), &mut Good::new(kind, 100.));
                    if let Err(sell_error) = result_sell {
                        //check that the returned value is an error of type SellError::ExpiredToken
                        assert_eq!(
                            sell_error,
                            SellError::ExpiredToken {
                                expired_token: token.clone()
                            }
                        );
                    } else if let Ok(returned_good) = result_sell {
                        //sell dind't return an error
                        assert_eq!(
                        0, 1,
                        "sell returned a good even if it was supposed to fail.\nGood returned: {:?}",
                        returned_good
                    );
                    }
                } else if let Err(lock_sell_error) = result_token {
                    //lock_sell threw some kind of error
                    assert_eq!(0, 1, "lock_sell returned an error: {:?}", lock_sell_error);
                }
            } else if let Err(get_sell_price_err) = offer_res {
                //get_sell_price threw some kind of error
                assert_eq!(
                    0, 1,
                    "get_sell_price returned an error: {:?}",
                    get_sell_price_err
                );
            }
        }
    }
}
