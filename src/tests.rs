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
        //test_sell_success::<MarketType>();
        //test_sell_err_unrecognized_token::<MarketType>();
        //test_sell_err_expired_token::<MarketType>();
        //test_sell_err_wrong_good_kind::<MarketType>();
        //test_sell_err_insufficient_good_quantity::<MarketType>();
    }
}
