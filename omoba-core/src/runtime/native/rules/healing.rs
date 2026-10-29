//! 治療倍率（原作 `vscripts/events.lua:279-380` HealFilter）。
//!
//! - 增療（倍率 > 1，例如光忠、燈籠、黑甲、自療時的長船）相乘。
//! - 減療（倍率 < 1，例如 decrease_HR、薩摩、C07T、毒、B33T）多個來源**只取最強**（最小倍率），
//!   不疊加。
//! - 原作 `events.lua:340-341` 的 bug：沒有 `modifier_decrease_HR` 時把 decrease_health 重設為 1，
//!   會蓋掉先算的 B33T。這是腳本順序造成的副作用，違反「取最強」的設計意圖；B33 不在遊戲裡，
//!   這裡照設計意圖（v1 §11 第 3 點的修正候選，目前沒有受影響的內容）。
//!
//! 倍率是 Q10 定點（1024 = 1）。同一個 `__aggregation_family` 先在族內取離 1 最遠者
//! （`UnitStats::received_healing`），再交給 [`combine`]。

/// 依呼叫端給的固定順序合成倍率：減療只留最強的那一個（同強度取順序最前者），其餘依序相乘。
/// 只有一個減療時，運算順序與逐一相乘完全相同。溢位回傳 None。
pub fn combine(values: &[i64]) -> Option<i64> {
    let strongest_cut = values.iter().enumerate().filter(|(_, raw)| **raw < 1024)
        .min_by_key(|(index, raw)| (**raw, *index)).map(|(index, _)| index);
    let mut factor = 1024i64;
    for (index, raw) in values.iter().enumerate() {
        if *raw < 1024 && Some(index) != strongest_cut { continue; }
        factor = i64::try_from(i128::from(factor) * i128::from(*raw) / 1024).ok()?;
    }
    Some(factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuts_take_the_strongest_and_boosts_multiply() {
        assert_eq!(combine(&[]), Some(1024));
        assert_eq!(combine(&[512, 819]), Some(512), "0.5 and 0.8 cuts: only 0.5");
        assert_eq!(combine(&[819, 512]), Some(512));
        assert_eq!(combine(&[1280, 1331]), Some(1280 * 1331 / 1024), "1.25 × 1.3");
        assert_eq!(combine(&[1280, 512, 819]), Some(1280 * 512 / 1024));
        assert_eq!(combine(&[0, 512]), Some(0), "a full cut wins");
    }

    #[test]
    fn a_single_cut_keeps_the_ordered_product() {
        let ordered = |values: &[i64]| values.iter().fold(1024i64, |f, v| f * v / 1024);
        for values in [[1331, 700, 1229], [700, 1331, 1229], [1331, 1229, 700]] {
            assert_eq!(combine(&values), Some(ordered(&values)));
        }
    }
}
