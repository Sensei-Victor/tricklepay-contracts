#![cfg(test)]

use proptest::prelude::*;

use super::helpers::StreamTest;

/// Operations that can be applied to a stream.
#[derive(Debug, Clone)]
enum StreamOp {
    Withdraw(u64),
    WithdrawAmount(u64, i128),
    Cancel(u64),
    AdvanceTime(u64),
}

/// Generate a stream operation over valid stream ids.
fn stream_op_strategy(num_streams: u64) -> impl Strategy<Value = StreamOp> {
    prop_oneof![
        (0..num_streams).prop_map(StreamOp::Withdraw),
        (0..num_streams, 1i128..=1000).prop_map(|(id, amt)| StreamOp::WithdrawAmount(id, amt)),
        (0..num_streams).prop_map(StreamOp::Cancel),
        (1u64..=500).prop_map(StreamOp::AdvanceTime),
    ]
}

/// The invariant: the contract's token balance equals the sum of unpaid escrow
/// across all streams. Unpaid escrow = total_amount - withdrawn.
fn assert_escrow_balance_invariant(t: &StreamTest, stream_ids: &[u64]) {
    let mut expected_balance: i128 = 0;

    for &id in stream_ids {
        let stream = t
            .contract
            .try_get_stream(&id)
            .expect("stream lookup should succeed")
            .expect("stream should exist");
        let unpaid = stream.total_amount - stream.withdrawn;
        expected_balance += unpaid;
    }

    let actual_balance = t.token.balance(&t.contract.address);
    assert_eq!(
        actual_balance, expected_balance,
        "escrow balance invariant violated: contract holds {} but unpaid escrow is {}",
        actual_balance, expected_balance
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Apply a random sequence of operations to multiple streams and verify
    /// the contract's token balance always equals the sum of unpaid escrow
    /// after every step.
    #[test]
    fn random_operations_preserve_escrow_balance(
        ops in prop::collection::vec(stream_op_strategy(3), 10..50)
    ) {
        let t = StreamTest::setup(100_000);
        t.set_time(100);

        // Create 3 streams with different amounts and schedules.
        let id0 = t.contract.create_stream(
            &t.sender,
            &t.recipient,
            &t.token_address,
            &10_000,
            &100,
            &10_100,
            &100,
        );
        let id1 = t.contract.create_stream(
            &t.sender,
            &t.recipient,
            &t.token_address,
            &20_000,
            &100,
            &20_100,
            &100,
        );
        let id2 = t.contract.create_stream(
            &t.sender,
            &t.recipient,
            &t.token_address,
            &15_000,
            &100,
            &15_100,
            &100,
        );

        let stream_ids = [id0, id1, id2];

        // Initial invariant check.
        assert_escrow_balance_invariant(&t, &stream_ids);

        // Apply operations and check invariant after each.
        for op in ops {
            match op {
                StreamOp::Withdraw(id) => {
                    let _ = t.contract.try_withdraw(&id);
                }
                StreamOp::WithdrawAmount(id, amount) => {
                    let _ = t.contract.try_withdraw_amount(&id, &amount);
                }
                StreamOp::Cancel(id) => {
                    let _ = t.contract.try_cancel(&id);
                }
                StreamOp::AdvanceTime(delta) => {
                    let current = t.env.ledger().timestamp();
                    t.set_time(current.saturating_add(delta));
                }
            }

            // Check the invariant holds after every operation.
            assert_escrow_balance_invariant(&t, &stream_ids);
        }
    }
}
