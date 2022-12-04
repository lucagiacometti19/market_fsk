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
        //market_test::test_name::<MarketType>();
        ////test new_random
        //market_test::test_new_random::<MarketType>();
        //market_test::new_random_should_not_exceeed_starting_capital::<MarketType>();
        ////test get_buy_price
        //market_test::test_get_buy_price_insufficient_qty_error::<MarketType>();
        //market_test::test_get_buy_price_non_positive_error::<MarketType>();
        //market_test::test_get_buy_price_success::<MarketType>();
        ////test get_sell_price
        //market_test::test_get_sell_price_non_positive_error::<MarketType>();
        //market_test::test_get_sell_price_success::<MarketType>();
        //market_test::test_deadlock_prevention::<MarketType>();
        ////sell tests
        test_sell_err_expired_token::<MarketType>();
        test_sell_err_insufficient_good_quantity::<MarketType>();
        test_sell_err_unrecognized_token::<MarketType>();
        test_sell_err_wrong_good_kind::<MarketType>();
        test_sell_success::<MarketType>();
        ////test_price_change
        //market_test::price_changes_waiting::<MarketType>();
        //market_test::test_price_change_after_buy::<MarketType>();
        //market_test::test_price_change_after_sell::<MarketType>();
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
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_success<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            let market = T::new_with_quantities(100., 100., 100., 100.);

            //call to unwrap since we assume get_sell_price doesn't throws error
            let offer_res = market.borrow_mut().get_sell_price(kind, 10.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind, 10., offer, "Sergio".to_string());
                if let Ok(token) = result_token {
                    //lock_sell doesn't throw errors
                    let result_sell = market.borrow_mut().sell(token, &mut Good::new(kind, 10.));
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
                                DEFAULT_GOOD_KIND => {
                                    default_good_kind_quantity = good_label.quantity
                                }
                                current_kind if current_kind == kind => {
                                    traded_good_quantity = good_label.quantity
                                }
                                _ => (),
                            }
                        }
                        //check default good is less or equal then before
                        assert!(default_good_kind_quantity <= 100.);
                        //check traded good is greater or equal then before
                        assert!(traded_good_quantity >= 100.);
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

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::UnrecognizedToken`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_unrecognized_token<T: Market>() {

        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            let market = T::new_with_quantities(100., 100., 100., 100.);

            //call to unwrap since we assume get_sell_price doesn't throws error
            let offer_res = market.borrow_mut().get_sell_price(kind, 10.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind, 10., offer, "Sergio".to_string());
                if let Ok(_) = result_token {
                    //lock_sell doesn't throw errors
                    //call sell with a wrong token
                    let result_sell = market
                        .borrow_mut()
                        .sell("token".to_string(), &mut Good::new(kind, 10.));
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
                        let mut traded_good_quantity = 0.;
                        for good_label in &market_goods {
                            match good_label.good_kind {
                                curr_kind if curr_kind == kind => {
                                    traded_good_quantity = good_label.quantity
                                }
                                _ => (),
                            }
                        }
                        //assumed the lock_sell actually locks the correct quantity of default good
                        //check that the traded good quantity didn't change
                        assert_eq!(100., traded_good_quantity);
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

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::ExpiredToken`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_expired_token<T: Market>() {
        use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            let market = T::new_with_quantities(100., 100., 100., 100.);

            //call to unwrap since we assume get_sell_price doesn't throws error
            let offer_res = market.borrow_mut().get_sell_price(kind, 10.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind, 10., offer, "Sergio".to_string());
                if let Ok(token) = result_token {
                    //lock_sell doesn't throw errors
                    //wait 15 days since that's the max TTL of a lock
                    for _ in 0..15 {
                        wait_one_day!(market);
                    }
                    //call to sell with an expired token
                    let result_sell = market
                        .borrow_mut()
                        .sell(token.clone(), &mut Good::new(kind, 10.));
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
                                DEFAULT_GOOD_KIND => {
                                    default_good_kind_quantity = good_label.quantity
                                }
                                curr_kind if curr_kind == kind => {
                                    traded_good_quantity = good_label.quantity
                                }
                                _ => (),
                            }
                        }
                        //expired lock means that default good quantity should be the same as the beginning
                        assert_eq!(100., default_good_kind_quantity);
                        //check that the traded good quantity didn't change
                        assert_eq!(100., traded_good_quantity);
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

    ///The `sell` function needs to be called after `lock_sell`, which creates
    ///an immutable contract ready to be fulfilled. This test assumes
    ///that `lock_sell` and `get_sell_price` work correctly, otherwise fails.
    ///
    ///This test checks that:
    ///1. the returned value is an error of type `SellError::WrongGoodKind`
    ///2. the quantity of the *default good* in the market **does not** change.
    ///4. the quantity of the *traded good* in the market **does not** change.
    ///
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_wrong_good_kind<T: Market>() {
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        let mut index = 1;
        for kind in &kinds {
            let market = T::new_with_quantities(100., 100., 100., 100.);
            if index == 2 {
                index = 0;
            }
            let wrong_good_kind = kinds.get(index).unwrap();
            //call to unwrap since we assume get_sell_price doesn't throws error
            let offer_res = market.borrow_mut().get_sell_price(kind.clone(), 10.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind.clone(), 10., offer, "Sergio".to_string());
                if let Ok(token) = result_token {
                    //lock_sell doesn't throw errors
                    let result_sell = market
                        .borrow_mut()
                        .sell(token.clone(), &mut Good::new(wrong_good_kind.clone(), 10.));
                    if let Err(sell_error) = result_sell {
                        //check that the returned value is an error of type SellError::WrongGoodKind
                        assert_eq!(
                            sell_error,
                            SellError::WrongGoodKind {
                                wrong_good_kind: wrong_good_kind.clone(),
                                pre_agreed_kind: kind.clone()
                            }
                        );
                        //get market goods and check quantities
                        let market_goods = market.borrow_mut().get_goods();
                        let mut traded_good_quantity = 0.;
                        for good_label in &market_goods {
                            match good_label.good_kind {
                                curr_kind if curr_kind == kind.clone() => {
                                    traded_good_quantity = good_label.quantity
                                }
                                _ => (),
                            }
                        }
                        //assumed the lock_sell actually locks the correct quantity of default good
                        //check that the traded good quantity didn't change
                        assert_eq!(100., traded_good_quantity);
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
            index += 1;
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
    ///Reference to the [specs](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/8e8c44803ff4e379ec7b730d5a458b1e77788ddb/market-protocol-specifications.md#the-sell-function).
    ///
    /// FSK team
    pub fn test_sell_err_insufficient_good_quantity<T: Market>() {
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            let market = T::new_with_quantities(100., 100., 100., 100.);

            //call to unwrap since we assume get_sell_price doesn't throws error
            let offer_res = market.borrow_mut().get_sell_price(kind, 10.);
            if let Ok(offer) = offer_res {
                let result_token =
                    market
                        .borrow_mut()
                        .lock_sell(kind, 10., offer, "Sergio".to_string());
                if let Ok(token) = result_token {
                    //lock_sell doesn't throw errors
                    //call to sell with less USD than the pre-agreed quantity in the contract
                    let result_sell = market
                        .borrow_mut()
                        .sell(token.clone(), &mut Good::new(kind, 9.99));
                    if let Err(sell_error) = result_sell {
                        //check that the returned value is an error of type SellError::InsufficientGoodQuantity
                        assert_eq!(
                            sell_error,
                            SellError::InsufficientGoodQuantity {
                                contained_quantity: 9.99,
                                pre_agreed_quantity: 10.
                            }
                        );
                        //get market goods and check quantities
                        let market_goods = market.borrow_mut().get_goods();
                        let mut traded_good_quantity = 0.;
                        for good_label in &market_goods {
                            match good_label.good_kind {
                                curr_kind if curr_kind == kind => traded_good_quantity = good_label.quantity,
                                _ => (),
                            }
                        }
                        //assumed the lock_sell actually locks the correct quantity of default good
                        //check that the traded good quantity didn't change
                        assert_eq!(100., traded_good_quantity);
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
