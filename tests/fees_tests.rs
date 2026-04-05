// SPDX-License-Identifier: GPL-3.0
// Copyright (C) 2026 Sergey Antonov
//
// This file is part of Alepic (Alephium Collaborative Canvas).
//
// Alepic is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Alepic is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Comprehensive test suite for Fee System
//! 
//! Tests cover:
//! - Initial sale fee calculations
//! - Secondary sale fee calculations
//! - Referrer fee handling
//! - Edge cases

#[cfg(test)]
mod tests {
    use crate::blockchain::fees::FeeCalculator;

    // ==================== Initial Sale Tests ====================

    #[test]
    fn test_initial_sale_with_referrer() {
        let price = 1000u64;
        let (treasury, referrer_fee, seller) = FeeCalculator::calculate_initial_sale(price, true);
        
        // Treasury: 95%
        assert_eq!(treasury, 950);
        // Referrer: 5%
        assert_eq!(referrer_fee, 50);
        // Seller: 0 (no seller in initial sale)
        assert_eq!(seller, 0);
        
        // Total should equal price
        assert_eq!(treasury + referrer_fee + seller, price);
    }

    #[test]
    fn test_initial_sale_without_referrer() {
        let price = 1000u64;
        let (treasury, referrer_fee, seller) = FeeCalculator::calculate_initial_sale(price, false);
        
        // Treasury: 95%
        assert_eq!(treasury, 950);
        // Referrer: 0% (no referrer)
        assert_eq!(referrer_fee, 0);
        // Seller: 0
        assert_eq!(seller, 0);
        
        // Note: 5% is not distributed when no referrer
        assert_eq!(treasury + referrer_fee + seller, 950);
    }

    #[test]
    fn test_initial_sale_rounding() {
        // Test rounding behavior with non-divisible amounts
        let price = 100u64;
        let (treasury, referrer_fee, _seller) = FeeCalculator::calculate_initial_sale(price, true);
        
        // 95% of 100 = 95
        assert_eq!(treasury, 95);
        // 5% of 100 = 5
        assert_eq!(referrer_fee, 5);
    }

    #[test]
    fn test_initial_sale_small_amounts() {
        let price = 1u64;
        let (treasury, referrer_fee, seller) = FeeCalculator::calculate_initial_sale(price, true);
        
        // 95% of 1 = 0 (integer division)
        assert_eq!(treasury, 0);
        // 5% of 1 = 0
        assert_eq!(referrer_fee, 0);
        assert_eq!(seller, 0);
    }

    #[test]
    fn test_initial_sale_large_amounts() {
        let price = 1_000_000u64;
        let (treasury, referrer_fee, seller) = FeeCalculator::calculate_initial_sale(price, true);
        
        assert_eq!(treasury, 950_000);
        assert_eq!(referrer_fee, 50_000);
        assert_eq!(seller, 0);
        assert_eq!(treasury + referrer_fee + seller, price);
    }

    #[test]
    fn test_initial_sale_zero_price() {
        let (treasury, referrer_fee, seller) = FeeCalculator::calculate_initial_sale(0, true);
        
        assert_eq!(treasury, 0);
        assert_eq!(referrer_fee, 0);
        assert_eq!(seller, 0);
    }

    // ==================== Secondary Sale Tests ====================

    #[test]
    fn test_secondary_sale_with_referrer() {
        let price = 1000u64;
        let (seller, treasury, referrer_fee) = FeeCalculator::calculate_secondary_sale(price, true);
        
        // Seller: 95%
        assert_eq!(seller, 950);
        // Treasury: 4%
        assert_eq!(treasury, 40);
        // Referrer: 1%
        assert_eq!(referrer_fee, 10);
        
        // Total should equal price
        assert_eq!(seller + treasury + referrer_fee, price);
    }

    #[test]
    fn test_secondary_sale_without_referrer() {
        let price = 1000u64;
        let (seller, treasury, referrer_fee) = FeeCalculator::calculate_secondary_sale(price, false);
        
        // Seller: 95%
        assert_eq!(seller, 950);
        // Treasury: 4%
        assert_eq!(treasury, 40);
        // Referrer: 0% (no referrer)
        assert_eq!(referrer_fee, 0);
        
        // Note: 1% is not distributed when no referrer
        assert_eq!(seller + treasury + referrer_fee, 990);
    }

    #[test]
    fn test_secondary_sale_rounding() {
        let price = 100u64;
        let (seller, treasury, referrer_fee) = FeeCalculator::calculate_secondary_sale(price, true);
        
        // 95% of 100 = 95
        assert_eq!(seller, 95);
        // 4% of 100 = 4
        assert_eq!(treasury, 4);
        // 1% of 100 = 1
        assert_eq!(referrer_fee, 1);
        assert_eq!(seller + treasury + referrer_fee, price);
    }

    #[test]
    fn test_secondary_sale_small_amounts() {
        let price = 1u64;
        let (seller, treasury, referrer_fee) = FeeCalculator::calculate_secondary_sale(price, true);
        
        // All values round to 0 with integer division
        assert_eq!(seller, 0);
        assert_eq!(treasury, 0);
        assert_eq!(referrer_fee, 0);
    }

    #[test]
    fn test_secondary_sale_large_amounts() {
        let price = 1_000_000u64;
        let (seller, treasury, referrer_fee) = FeeCalculator::calculate_secondary_sale(price, true);
        
        assert_eq!(seller, 950_000);
        assert_eq!(treasury, 40_000);
        assert_eq!(referrer_fee, 10_000);
        assert_eq!(seller + treasury + referrer_fee, price);
    }

    #[test]
    fn test_secondary_sale_zero_price() {
        let (seller, treasury, referrer_fee) = FeeCalculator::calculate_secondary_sale(0, true);
        
        assert_eq!(seller, 0);
        assert_eq!(treasury, 0);
        assert_eq!(referrer_fee, 0);
    }

    // ==================== Comparison Tests ====================

    #[test]
    fn test_initial_vs_secondary_treasury_share() {
        let price = 1000u64;
        
        let (initial_treasury, _, _) = FeeCalculator::calculate_initial_sale(price, true);
        let (_, secondary_treasury, _) = FeeCalculator::calculate_secondary_sale(price, true);
        
        // Initial sale gives more to treasury (95% vs 4%)
        assert!(initial_treasury > secondary_treasury);
        assert_eq!(initial_treasury, 950);
        assert_eq!(secondary_treasury, 40);
    }

    #[test]
    fn test_initial_vs_secondary_seller_share() {
        let price = 1000u64;
        
        let (_, _, initial_seller) = FeeCalculator::calculate_initial_sale(price, true);
        let (secondary_seller, _, _) = FeeCalculator::calculate_secondary_sale(price, true);
        
        // Initial sale: no seller
        assert_eq!(initial_seller, 0);
        // Secondary sale: seller gets 95%
        assert_eq!(secondary_seller, 950);
    }

    #[test]
    fn test_referrer_fee_comparison() {
        let price = 1000u64;
        
        let (_, initial_referrer, _) = FeeCalculator::calculate_initial_sale(price, true);
        let (_, _, secondary_referrer) = FeeCalculator::calculate_secondary_sale(price, true);
        
        // Initial sale referrer gets 5%, secondary gets 1%
        assert_eq!(initial_referrer, 50);
        assert_eq!(secondary_referrer, 10);
        assert!(initial_referrer > secondary_referrer);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_fee_calculation_boundary_values() {
        // Test various boundary values
        let test_prices = [10u64, 20, 25, 50, 100, 200, 250, 500];
        
        for &price in &test_prices {
            let (init_t, init_r, init_s) = FeeCalculator::calculate_initial_sale(price, true);
            let (sec_s, sec_t, sec_r) = FeeCalculator::calculate_secondary_sale(price, true);
            
            // Initial sale totals
            let init_total = init_t + init_r + init_s;
            assert!(init_total <= price, "Initial sale total {} exceeds price {}", init_total, price);
            
            // Secondary sale totals
            let sec_total = sec_s + sec_t + sec_r;
            assert!(sec_total <= price, "Secondary sale total {} exceeds price {}", sec_total, price);
        }
    }

    #[test]
    fn test_fee_percentages_accuracy() {
        // Use a price divisible by all percentages
        let price = 10_000u64;
        
        // Initial sale
        let (treasury, referrer, seller) = FeeCalculator::calculate_initial_sale(price, true);
        assert_eq!(treasury, 9500);  // Exactly 95%
        assert_eq!(referrer, 500);   // Exactly 5%
        assert_eq!(seller, 0);
        
        // Secondary sale
        let (seller, treasury, referrer) = FeeCalculator::calculate_secondary_sale(price, true);
        assert_eq!(seller, 9500);    // Exactly 95%
        assert_eq!(treasury, 400);   // Exactly 4%
        assert_eq!(referrer, 100);   // Exactly 1%
    }

    #[test]
    fn test_consistent_behavior_multiple_calls() {
        let price = 500u64;
        
        // Multiple calls should return same results
        let result1 = FeeCalculator::calculate_initial_sale(price, true);
        let result2 = FeeCalculator::calculate_initial_sale(price, true);
        let result3 = FeeCalculator::calculate_initial_sale(price, true);
        
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }
}
