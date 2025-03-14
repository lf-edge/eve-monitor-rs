use std::borrow::Borrow;

pub fn compute_lcs<'a, T, U>(good: &'a [U], bad: &'a [U]) -> Vec<&'a U>
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
            lcs.push(&good[i - 1]);
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

pub fn collect_diff<'a, T, U>(
    old_good: &'a [U],
    new: &'a [U],
    lcs: &[&'a U],
) -> (Vec<&'a U>, Vec<&'a U>)
where
    T: PartialEq + ?Sized,
    U: Borrow<T>,
    &'a U: Borrow<T>,
{
    // Find deletions (elements in old_good not present in LCS)
    let deletions: Vec<_> = old_good
        .iter()
        .filter(|e| {
            !lcs.iter().any(|l| {
                // Compare borrowed T values, not U or references
                let l_borrowed: &T = (*l).borrow(); // Dereference `l` first
                let e_borrowed: &T = e.borrow();
                l_borrowed == e_borrowed
            })
        })
        .collect();

    // Find insertions (elements in new not present in LCS)
    let insertions: Vec<_> = new
        .iter()
        .filter(|e| {
            !lcs.iter().any(|l| {
                let l_borrowed: &T = (*l).borrow();
                let e_borrowed: &T = e.borrow();
                l_borrowed == e_borrowed
            })
        })
        .collect();

    (deletions, insertions)
}
