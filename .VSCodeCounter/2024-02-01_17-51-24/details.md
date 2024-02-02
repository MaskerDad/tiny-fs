# Details

Date : 2024-02-01 17:51:24

Directory c:\\Users\\26896\\Desktop\\repo_sync\\new_repo\\tiny-fs

Total : 30 files,  638 codes, 73 comments, 149 blanks, all 860 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [README.md](/README.md) | Markdown | 199 | 0 | 84 | 283 |
| [tfs-test/Cargo.lock](/tfs-test/Cargo.lock) | TOML | 4 | 2 | 2 | 8 |
| [tfs-test/Cargo.toml](/tfs-test/Cargo.toml) | TOML | 5 | 1 | 3 | 9 |
| [tfs-test/src/main.rs](/tfs-test/src/main.rs) | Rust | 3 | 0 | 1 | 4 |
| [tfs-test/target/.rustc_info.json](/tfs-test/target/.rustc_info.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs-test/target/debug/.fingerprint/tfs-test-0e0152f3fab2dd26/bin-tfs-test.json](/tfs-test/target/debug/.fingerprint/tfs-test-0e0152f3fab2dd26/bin-tfs-test.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs-test/target/debug/.fingerprint/tfs-test-10e77788d8eec420/bin-tfs-test.json](/tfs-test/target/debug/.fingerprint/tfs-test-10e77788d8eec420/bin-tfs-test.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs-test/target/debug/.fingerprint/tfs-test-20a308751dfaea9e/test-bin-tfs-test.json](/tfs-test/target/debug/.fingerprint/tfs-test-20a308751dfaea9e/test-bin-tfs-test.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs-test/target/debug/.fingerprint/tfs-test-a45a30887355dfbb/test-bin-tfs-test.json](/tfs-test/target/debug/.fingerprint/tfs-test-a45a30887355dfbb/test-bin-tfs-test.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/Cargo.lock](/tfs/Cargo.lock) | TOML | 26 | 2 | 5 | 33 |
| [tfs/Cargo.toml](/tfs/Cargo.toml) | TOML | 10 | 1 | 3 | 14 |
| [tfs/src/bitmap.rs](/tfs/src/bitmap.rs) | Rust | 62 | 12 | 6 | 80 |
| [tfs/src/block_cache.rs](/tfs/src/block_cache.rs) | Rust | 121 | 10 | 19 | 150 |
| [tfs/src/block_dev.rs](/tfs/src/block_dev.rs) | Rust | 5 | 3 | 1 | 9 |
| [tfs/src/disk_manager.rs](/tfs/src/disk_manager.rs) | Rust | 0 | 0 | 1 | 1 |
| [tfs/src/layout.rs](/tfs/src/layout.rs) | Rust | 174 | 40 | 19 | 233 |
| [tfs/src/lib.rs](/tfs/src/lib.rs) | Rust | 12 | 2 | 4 | 18 |
| [tfs/src/vfs.rs](/tfs/src/vfs.rs) | Rust | 0 | 0 | 1 | 1 |
| [tfs/target/.rustc_info.json](/tfs/target/.rustc_info.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/lazy_static-ce2560039bfb2f0c/lib-lazy_static.json](/tfs/target/debug/.fingerprint/lazy_static-ce2560039bfb2f0c/lib-lazy_static.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/spin-b3d97f5589c830c3/lib-spin.json](/tfs/target/debug/.fingerprint/spin-b3d97f5589c830c3/lib-spin.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/spin-ee2759a277fd8c13/lib-spin.json](/tfs/target/debug/.fingerprint/spin-ee2759a277fd8c13/lib-spin.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-1619fcdba348644e/test-lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-1619fcdba348644e/test-lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-28b5044f1e64bfd9/test-lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-28b5044f1e64bfd9/test-lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-3d070fa476a70916/lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-3d070fa476a70916/lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-3e8499823353e8e8/test-lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-3e8499823353e8e8/test-lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-51b84f7101ae008f/test-lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-51b84f7101ae008f/test-lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-65a8ba3a22a92b83/lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-65a8ba3a22a92b83/lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-6f5cb38abaa79ba0/lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-6f5cb38abaa79ba0/lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |
| [tfs/target/debug/.fingerprint/tfs-dbf744d25132fb55/lib-tfs.json](/tfs/target/debug/.fingerprint/tfs-dbf744d25132fb55/lib-tfs.json) | JSON | 1 | 0 | 0 | 1 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)