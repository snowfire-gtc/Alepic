pub struct FeeCalculator;

impl FeeCalculator {
    /// Initial Sale: Treasury 95%, Referrer 5%
    pub fn calculate_initial_sale(price: u64, referrer: bool) -> (u64, u64, u64) {
        let treasury = (price as f64 * 0.95) as u64;
        let referrer_fee = if referrer { (price as f64 * 0.05) as u64 } else { 0 };
        let seller = 0; // При первоначальной покупке продавца нет
        (treasury, referrer_fee, seller)
    }

    /// Secondary Sale: Seller 95%, Treasury 4%, Referrer 1%
    pub fn calculate_secondary_sale(price: u64, referrer: bool) -> (u64, u64, u64) {
        let seller = (price as f64 * 0.95) as u64;
        let treasury = (price as f64 * 0.04) as u64;
        let referrer_fee = if referrer { (price as f64 * 0.01) as u64 } else { 0 };
        (seller, treasury, referrer_fee)
    }
}