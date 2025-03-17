use std::borrow::Borrow;

use log::trace;

#[derive(Debug)]
pub enum DiffOp {
    Del(usize),
    Add(usize),
    Mod(usize, usize),
    Unchanged(usize),
}

pub fn compute_lcs<'a, T, U>(old: &'a [U], new: &'a [U]) -> Vec<(usize, usize)>
where
    T: PartialEq + ?Sized,
    U: Borrow<T> + PartialEq,
{
    let old_len = old.len();
    let new_len = new.len();

    // Initialize DP table with dimensions (good_len + 1) x (bad_len + 1)
    let mut dp = vec![vec![0; new_len + 1]; old_len + 1];

    // Fill the DP table
    for i in 1..=old_len {
        for j in 1..=new_len {
            if old[i - 1].borrow() == new[j - 1].borrow() {
                // Events match: extend the LCS
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                // Take the maximum of the left or top cell
                dp[i][j] = std::cmp::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    // Backtrack to reconstruct the LCS
    let mut i = old_len;
    let mut j = new_len;
    let mut lcs = Vec::new();

    while i > 0 && j > 0 {
        if old[i - 1].borrow() == new[j - 1].borrow() {
            // Include the matching event in the LCS
            lcs.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            // Move up (prioritize the good log)
            i -= 1;
        } else {
            // Move left (prioritize the bad log)
            j -= 1;
        }
    }

    // Reverse to restore original order
    lcs.reverse();
    lcs
}

pub fn compute_lcs_orig<'a, T, U>(good: &'a [U], bad: &'a [U]) -> Vec<usize>
where
    T: PartialEq + ?Sized,
    U: Borrow<T> + PartialEq,
{
    let good_len = good.len();
    let bad_len = bad.len();

    // Initialize DP table with dimensions (good_len + 1) x (bad_len + 1)
    let mut dp = vec![vec![0; bad_len + 1]; good_len + 1];

    // Fill the DP table
    for i in 1..=good_len {
        for j in 1..=bad_len {
            if good[i - 1].borrow() == bad[j - 1].borrow() {
                // Events match: extend the LCS
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                // Take the maximum of the left or top cell
                dp[i][j] = std::cmp::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    // Backtrack to reconstruct the LCS
    let mut i = good_len;
    let mut j = bad_len;
    let mut lcs = Vec::new();

    while i > 0 && j > 0 {
        if good[i - 1].borrow() == bad[j - 1].borrow() {
            // Include the matching event in the LCS
            lcs.push(i - 1);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            // Move up (prioritize the good log)
            i -= 1;
        } else {
            // Move left (prioritize the bad log)
            j -= 1;
        }
    }

    // Reverse to restore original order
    lcs.reverse();
    lcs
}

// pub fn collect_diff<'a, T, U>(
//     old_good: &'a [U],
//     new: &'a [U],
//     lcs: &[usize],
// ) -> (Vec<&'a U>, Vec<&'a U>)
// where
//     T: PartialEq + ?Sized,
//     U: Borrow<T>,
//     &'a U: Borrow<T>,
// {
//     // Find deletions (elements in old_good not present in LCS)
//     let deletions: Vec<_> = old_good
//         .iter()
//         .filter(|e| {
//             !lcs.iter().any(|l| {
//                 // Compare borrowed T values, not U or references
//                 let l_borrowed: &T = (*l).borrow(); // Dereference `l` first
//                 let e_borrowed: &T = e.borrow();
//                 l_borrowed == e_borrowed
//             })
//         })
//         .collect();

//     // Find insertions (elements in new not present in LCS)
//     let insertions: Vec<_> = new
//         .iter()
//         .filter(|e| {
//             !lcs.iter().any(|l| {
//                 let l_borrowed: &T = (*l).borrow();
//                 let e_borrowed: &T = e.borrow();
//                 l_borrowed == e_borrowed
//             })
//         })
//         .collect();

//     (deletions, insertions)
// }

pub fn collect_diff<'a, U>(
    old_good: &'a [U],
    new: &'a [U],
    lcs: &[(usize, usize)],
) -> (Vec<usize>, Vec<usize>) {
    // Find deletions (elements in old_good not present in LCS)
    let deletions: Vec<_> = old_good
        .iter()
        .enumerate()
        .filter(|(i, _)| !lcs.iter().any(|(j, _)| i == j))
        .map(|(i, _)| i)
        .collect();

    // Find insertions (elements in new not present in LCS)
    let insertions: Vec<_> = new
        .iter()
        .enumerate()
        .filter(|(i, _)| !lcs.iter().any(|(_, j)| i == j))
        .map(|(i, _)| i)
        .collect();

    (deletions, insertions)
}

pub fn produce_diff_ops(
    lcs: &[(usize, usize)],
    add: &[usize],
    del: &[usize],
    mods: &[(usize, usize)],
) -> (Vec<DiffOp>, Vec<DiffOp>) {
    let mut old_diff = Vec::new();
    let mut new_diff = Vec::new();

    let mut mods_old = mods
        .iter()
        .map(|&(old, new)| (old, new))
        .collect::<Vec<_>>();
    mods_old.sort_by_key(|e| e.0);

    let mut mods_new = mods
        .iter()
        .map(|&(old, new)| (new, old))
        .collect::<Vec<_>>();
    mods_new.sort_by_key(|e| e.0);

    // Merge del, mods (old indices), and lcs (old indices) for old array
    let (mut d, mut m, mut l) = (0, 0, 0);
    while d < del.len() || m < mods.len() || l < lcs.len() {
        let current_d = if d < del.len() { del[d] } else { usize::MAX };
        let current_m = if m < mods.len() {
            mods_old[m].0
        } else {
            usize::MAX
        };
        let current_l = if l < lcs.len() { lcs[l].0 } else { usize::MAX };

        let min_val = current_d.min(current_m).min(current_l);

        trace!("MIN: {}", min_val);

        if min_val.saturating_sub(old_diff.len()) > 0 {
            // fill the gap with events from original array
            let start = old_diff.len();
            for i in start..min_val {
                trace!("GAP: {}", i);
                old_diff.push(DiffOp::Unchanged(i));
            }
        }

        if min_val == current_d {
            old_diff.push(DiffOp::Del(current_d));
            d += 1;
        } else if min_val == current_m {
            let (old, new) = mods_old[m];
            old_diff.push(DiffOp::Mod(old, new));
            m += 1;
        } else {
            let (old, _new) = lcs[l];
            old_diff.push(DiffOp::Unchanged(old));
            l += 1;
        }
    }

    // Merge add, mods (new indices), and lcs (new indices) for new array
    let (mut a, mut m, mut l) = (0, 0, 0);
    while a < add.len() || m < mods.len() || l < lcs.len() {
        let current_a = if a < add.len() { add[a] } else { usize::MAX };
        let current_m = if m < mods.len() {
            mods_new[m].0
        } else {
            usize::MAX
        };
        let current_l = if l < lcs.len() { lcs[l].1 } else { usize::MAX };

        let min_val = current_a.min(current_m).min(current_l);

        trace!("MIN: {}", min_val);

        if min_val.saturating_sub(new_diff.len()) > 0 {
            // fill the gap with events from original array
            let start = new_diff.len();

            for i in start..min_val {
                trace!("GAP: {}", i);
                new_diff.push(DiffOp::Unchanged(i));
            }
        }

        // print all current and min values
        trace!(
            "a: {}, m: {}, l: {}: MIN: {}",
            current_a,
            current_m,
            current_l,
            min_val
        );

        if min_val == current_a {
            new_diff.push(DiffOp::Add(current_a));
            a += 1;
        } else if min_val == current_m {
            let (new, old) = mods_new[m];
            new_diff.push(DiffOp::Mod(old, new));
            m += 1;
        } else {
            let (_old, new) = lcs[l];
            new_diff.push(DiffOp::Unchanged(new));

            l += 1;
        }
    }

    (old_diff, new_diff)
}
