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
        //market_test::test_sell_err_expired_token::<MarketType>();
        //market_test::test_sell_err_insufficient_good_quantity::<MarketType>();
        //market_test::test_sell_err_unrecognized_token::<MarketType>();
        //market_test::test_sell_err_wrong_good_kind::<MarketType>();
        //market_test::test_sell_success::<MarketType>();
        ////test_price_change
        //market_test::price_changes_waiting::<MarketType>();
        market_test::test_price_change_after_buy::<MarketType>();
        //market_test::test_price_change_after_sell::<MarketType>();
    }
}
