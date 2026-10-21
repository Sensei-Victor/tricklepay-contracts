//! Events emitted by the stream contract.
//!
//! These types form part of the contract's public interface. Indexers and
//! off-chain consumers depend on the topic layout and payload fields to filter
//! streams by participant and to reconstruct stream state without follow-up
//! contract calls. The shapes of these events are therefore effectively an API
//! contract: any change to a topic, a field name, or a field type is a
//! breaking change for downstream consumers and should be treated with the
//! same care as a change to a contract entry point.

use soroban_sdk::{contractevent, Address, Env};

use crate::types::Stream;

/// Emitted when a new stream is opened. Indexers can filter on the `sender`
/// and `recipient` topics to find streams for either party. The schedule fields
/// are included so an indexer can record the full stream without a follow-up
/// `get_stream` call.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Created {
    #[topic]
    pub sender: Address,
    #[topic]
    pub recipient: Address,
    pub id: u64,
    pub token: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub cliff_time: u64,
}

/// Emitted when a recipient withdraws vested tokens.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Withdrawn {
    #[topic]
    pub recipient: Address,
    pub id: u64,
    pub amount: i128,
}

/// Emitted when a sender cancels a stream. Carries both sides of the split so
/// the recipient's accrued amount and the sender's refund are visible without
/// a follow-up query.
///
/// # Decision: recipient as a topic
///
/// The `Cancelled` event indexes only `sender` as a topic. `Created` indexes
/// both `sender` and `recipient`, and `Withdrawn` indexes `recipient`, but
/// `Cancelled` does not carry `recipient` as a second topic.
///
/// **Indexer impact:** An indexer that wants to find every cancellation
/// affecting a specific recipient must either:
/// 1. Fetch all `Cancelled` events and filter by the `recipient_amount` field
///    (which is not indexed), requiring a full event scan, or
/// 2. Subscribe to `Created` events for that recipient, record the stream ids,
///    then query each one with `get_stream` to check the `cancelled` flag, or
/// 3. Subscribe to `Cancelled` events, fetch the corresponding stream record
///    with `get_stream(id)`, and check if `stream.recipient` matches the
///    target address.
///
/// Adding `#[topic] recipient: Address` would eliminate all three workarounds:
/// an indexer could filter `Cancelled` by recipient the same way it filters
/// `Created` and `Withdrawn`.
///
/// **Trade-off:** Adding a topic is an event schema change. Every off-chain
/// consumer that parses `Cancelled` events — indexers, explorers, client
/// libraries, monitoring tools — would need to update their event parsers to
/// handle the new topic layout. Existing code that expects `Cancelled` to have
/// one topic (`sender`) would break when it encounters a two-topic event from
/// a redeployed contract. The convenience of recipient-filtered cancellation
/// queries must be weighed against the coordination cost of getting every
/// downstream consumer to update simultaneously.
///
/// **Who would break:** Any service that decodes `Cancelled` events using a
/// fixed schema (e.g., a Horizon indexer, the tricklepay-backend event
/// processor, a block explorer, or a client library that maps event topics to
/// struct fields). These services would see parsing errors or misaligned fields
/// when the topic count changes. If the contract were redeployed with the new
/// schema while an indexer still expected the old one, every `Cancelled` event
/// would fail to parse until the indexer updated. This is not a silent
/// degradation — it is a hard break that surfaces immediately.
///
/// **Workaround cost:** Without the topic, an indexer can still track
/// cancellations for a recipient by listening to `Created` events for that
/// address (which are already indexed by recipient) and periodically polling
/// `get_stream(id)` or listening to all `Cancelled` events and doing a
/// post-filter lookup. This is less efficient than a direct topic filter, but
/// it works with the existing schema and does not require redeployment or
/// coordinated updates.
///
/// **Decision:** The recipient is **not added as a topic** to `Cancelled`.
/// The convenience of recipient-filtered cancellation queries does not justify
/// the coordination cost and breaking change for all downstream consumers. If
/// this decision is revisited in a future contract version, the change should
/// be accompanied by:
/// - A changelog entry marking the event schema as breaking.
/// - Updated event documentation in `events.rs` and the README showing the new
///   two-topic layout.
/// - Notification to all known indexers (Horizon, tricklepay-backend, any
///   third-party explorers) with a schema migration guide.
/// - A version bump or redeployment announcement so clients can distinguish
///   old-schema and new-schema contract instances.
///
/// See the event schema documentation in the README's "Event schemas for
/// indexers" section for the full compatibility note on event changes.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cancelled {
    #[topic]
    pub sender: Address,
    pub id: u64,
    pub recipient_amount: i128,
    pub sender_refund: i128,
}

/// Publish [`Created`] for a newly stored stream `id`.
///
/// Every field is read from the stored `stream`, which at creation holds
/// exactly the arguments the stream was opened with.
pub(crate) fn publish_created(env: &Env, id: u64, stream: &Stream) {
    Created {
        sender: stream.sender.clone(),
        recipient: stream.recipient.clone(),
        id,
        token: stream.token.clone(),
        total_amount: stream.total_amount,
        start_time: stream.start_time,
        end_time: stream.end_time,
        cliff_time: stream.cliff_time,
    }
    .publish(env);
}

/// Publish [`Withdrawn`] after tokens have been transferred to the recipient.
pub(crate) fn publish_withdrawn(env: &Env, recipient: &Address, id: u64, amount: i128) {
    Withdrawn {
        recipient: recipient.clone(),
        id,
        amount,
    }
    .publish(env);
}
