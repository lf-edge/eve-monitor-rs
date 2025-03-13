pub fn compute_lcs<'a, T>(good: &[&'a T], bad: &[&'a T]) -> Vec<&'a T>
where
    T: PartialEq + ?Sized,
{
    let good_len = good.len();
    let bad_len = bad.len();

    // Initialize DP table with dimensions (good_len + 1) x (bad_len + 1)
    let mut dp = vec![vec![0; bad_len + 1]; good_len + 1];

    // Fill the DP table
    for i in 1..=good_len {
        for j in 1..=bad_len {
            if good[i - 1] == bad[j - 1] {
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
        if good[i - 1] == bad[j - 1] {
            // Include the matching event in the LCS
            lcs.push(good[i - 1]);
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

pub fn collect_diff<'a, T>(
    old_good: &[&'a T],
    new: &[&'a T],
    lcs: &[&'a T],
) -> (Vec<&'a T>, Vec<&'a T>)
where
    T: PartialEq + ?Sized,
{
    // Find deleted (events in `old` but not in LCS)
    let deletions: Vec<_> = old_good
        .iter()
        .filter(|e| !lcs.contains(e))
        .copied()
        .collect();

    // Find added (events in `new` but not in LCS)
    let insertions: Vec<_> = new.iter().filter(|e| !lcs.contains(e)).copied().collect();

    (deletions, insertions)
}
