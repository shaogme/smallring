#![cfg(feature = "loom")]

use loom::sync::Arc;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::thread;
use smallring::atomic::AtomicRingBuf;

#[test]
fn test_atomic_queue_mpsc_loom() {
    loom::model(|| {
        // MPSC test: 2 Producers, 1 Consumer
        // Capacity 4
        let buf = Arc::new(AtomicRingBuf::<AtomicUsize, 4, false>::new(4));
        let buf1 = buf.clone();
        let buf2 = buf.clone();

        let t1 = thread::spawn(move || {
            buf1.push(1, Ordering::Relaxed).unwrap();
            buf1.push(2, Ordering::Relaxed).unwrap();
        });

        let t2 = thread::spawn(move || {
            buf2.push(3, Ordering::Relaxed).unwrap();
            buf2.push(4, Ordering::Relaxed).unwrap();
        });

        // Consumer
        let t3 = thread::spawn(move || {
            let mut received = 0;
            let mut sum = 0;
            // Expect 4 values
            while received < 4 {
                if let Some(val) = buf.pop(Ordering::Relaxed) {
                    received += 1;
                    sum += val;
                } else {
                    thread::yield_now();
                }
            }
            // 1+2+3+4 = 10
            assert_eq!(sum, 10);
        });

        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
    });
}

#[test]
fn test_atomic_queue_overwrite_loom() {
    loom::model(|| {
        // Overwrite mode, capacity 1
        let buf = Arc::new(AtomicRingBuf::<AtomicUsize, 4, true>::new(1));

        let producer = buf.clone();
        let overwrote_10 = Arc::new(AtomicUsize::new(0));
        let overwrote_10_clone = overwrote_10.clone();
        let producer_done = Arc::new(AtomicUsize::new(0));
        let producer_done_clone = producer_done.clone();

        let t1 = thread::spawn(move || {
            producer.push(10, Ordering::Relaxed);
            let res = producer.push(20, Ordering::Relaxed);
            if res == Some(10) {
                overwrote_10_clone.store(1, Ordering::Relaxed);
            }
            producer_done_clone.store(1, Ordering::Release);
        });

        let consumer = buf.clone();
        let popped_10 = Arc::new(AtomicUsize::new(0));
        let popped_20 = Arc::new(AtomicUsize::new(0));
        let popped_10_clone = popped_10.clone();
        let popped_20_clone = popped_20.clone();

        let t2 = thread::spawn(move || {
            loop {
                if let Some(val) = consumer.pop(Ordering::Relaxed) {
                    if val == 10 {
                        popped_10_clone.store(1, Ordering::Relaxed);
                    } else if val == 20 {
                        popped_20_clone.store(1, Ordering::Relaxed);
                    }
                } else if producer_done.load(Ordering::Acquire) == 1 {
                    // Drain any remaining elements
                    while let Some(val) = consumer.pop(Ordering::Relaxed) {
                        if val == 10 {
                            popped_10_clone.store(1, Ordering::Relaxed);
                        } else if val == 20 {
                            popped_20_clone.store(1, Ordering::Relaxed);
                        }
                    }
                    break;
                } else {
                    thread::yield_now();
                }
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let overwrote = overwrote_10.load(Ordering::Relaxed) == 1;
        let popped10 = popped_10.load(Ordering::Relaxed) == 1;
        let popped20 = popped_20.load(Ordering::Relaxed) == 1;

        // Verify consistency:
        // 1. If 10 was overwritten, then 10 must not have been popped, and 20 must have been popped.
        // 2. If 10 was not overwritten, then both 10 and 20 must have been popped.
        if overwrote {
            assert!(!popped10, "10 was overwritten but also popped!");
            assert!(popped20, "10 was overwritten but 20 was not popped!");
        } else {
            assert!(popped10, "10 was not overwritten but not popped!");
            assert!(popped20, "10 was not overwritten but 20 was not popped!");
        }
    });
}

#[test]
fn test_atomic_queue_mpmc_loom() {
    loom::model(|| {
        // MPMC test: 2 Producers, 2 Consumers
        // Capacity 2, non-overwrite mode to keep Loom state space bounded
        let buf = Arc::new(AtomicRingBuf::<AtomicUsize, 4, false>::new(2));

        let buf_p1 = buf.clone();
        let t1 = thread::spawn(move || {
            let _ = buf_p1.push(1, Ordering::Relaxed);
        });

        let buf_p2 = buf.clone();
        let t2 = thread::spawn(move || {
            let _ = buf_p2.push(2, Ordering::Relaxed);
        });

        let popped_sum = Arc::new(AtomicUsize::new(0));
        let popped_count = Arc::new(AtomicUsize::new(0));

        let buf_c1 = buf.clone();
        let sum_c1 = popped_sum.clone();
        let count_c1 = popped_count.clone();
        let t3 = thread::spawn(move || {
            if let Some(val) = buf_c1.pop(Ordering::Relaxed) {
                sum_c1.fetch_add(val, Ordering::Relaxed);
                count_c1.fetch_add(1, Ordering::Relaxed);
            }
        });

        let buf_c2 = buf.clone();
        let sum_c2 = popped_sum.clone();
        let count_c2 = popped_count.clone();
        let t4 = thread::spawn(move || {
            if let Some(val) = buf_c2.pop(Ordering::Relaxed) {
                sum_c2.fetch_add(val, Ordering::Relaxed);
                count_c2.fetch_add(1, Ordering::Relaxed);
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        t4.join().unwrap();

        // Collect all successfully popped elements and any remaining elements to verify sum consistency
        let mut final_sum = popped_sum.load(Ordering::Relaxed);
        let mut final_count = popped_count.load(Ordering::Relaxed);

        while let Some(val) = buf.pop(Ordering::Relaxed) {
            final_sum += val;
            final_count += 1;
        }

        assert!(final_count <= 2);
        if final_count == 2 {
            assert_eq!(final_sum, 3); // 1 + 2 = 3
        } else if final_count == 1 {
            assert!(final_sum == 1 || final_sum == 2);
        } else {
            assert_eq!(final_sum, 0);
        }
    });
}
