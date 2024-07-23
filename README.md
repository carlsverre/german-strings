An (extremely unsafe) experiment in implementing "German strings" in Rust. Please don't use this code in anything important.

To learn more about German strings:
- https://cedardb.com/blog/german_strings/
- https://db.in.tum.de/~freitag/papers/p29-neumann-cidr20.pdf

This implementation tries to present the beginnings of a rust ergonomic string api while optimizing the interior layout per the paper.

Note that this library does not implement storage classes (yet?). To do so will require a bit more black magic to steal some bits off the long heap pointer. The [ointers] library is a good reference for doing this.

[ointers]: https://docs.rs/ointers/latest/ointers/

## Benchmarks?

Included are some very very very simple Criterion benchmarks that attempt to show when this pattern is more optimal than the built in str types. Specifically, this pattern is very well suited to prefix filtering workloads as we are able to use the interned prefix to reject needles without looking at the heap.

Generally, if the datasets larger than cache and your search term is highly selective on the first 4 bytes this technique should be quite a bit faster than the stdlib.

Raw benchmark output on my machine:
```
     Running benches/scan_prefix.rs (target/release/deps/scan_prefix-7c1f89db07611068)
scan_prefix_long/rust_string
                        time:   [40.725 ms 40.879 ms 41.046 ms]
                        change: [-0.2346% +0.3164% +0.8369%] (p = 0.27 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
scan_prefix_long/german_string
                        time:   [24.356 ms 24.447 ms 24.545 ms]
                        change: [-1.1887% -0.2928% +0.4911%] (p = 0.51 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

scan_prefix_short/rust_string
                        time:   [36.850 ms 37.088 ms 37.418 ms]
                        change: [+1.4856% +2.2537% +3.1663%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
scan_prefix_short/german_string
                        time:   [26.231 ms 26.351 ms 26.497 ms]
                        change: [-0.6634% -0.0980% +0.5519%] (p = 0.76 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
```

You can run the benchmark with `cargo bench`