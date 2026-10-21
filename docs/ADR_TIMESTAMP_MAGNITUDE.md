# ADR: Do Not Bound Timestamp Magnitude

**Status:** Accepted

## Context

`create_stream` accepts `start_time`, `end_time`, and `cliff_time` as Unix
timestamps in seconds. A client that passes a JavaScript millisecond timestamp
such as `Date.now()` can create a stream that remains pending much longer than
intended. The sender can cancel a pending stream before its end time to recover
the unvested funds, but must notice and submit that transaction.

## Options considered

1. **Reject timestamps beyond a maximum distance from the current ledger time.**
   This would catch common millisecond values, but requires choosing and
   maintaining a time horizon. Any fixed horizon could reject a legitimate
   long-dated schedule, and the horizon would not reliably identify the units
   a caller intended.
2. **Keep accepting timestamps that satisfy the existing schedule checks
   (chosen).** No timestamp-magnitude limit is applied; callers are responsible
   for passing Unix seconds.

## Decision

Keep the current behavior and do not add a timestamp-magnitude bound or error
code. The contract has no principled way to distinguish an accidental
millisecond value from an intentionally distant Unix-second schedule. A
threshold would turn an integration mistake into a new ABI-level rejection
rule, while still being unable to detect every unit error. The existing
timestamp documentation warns clients to use seconds, and a sender can cancel
a mistaken pending stream before its end time.

## Consequences

- Clients must validate timestamp units before calling `create_stream`.
- A millisecond timestamp can still make a stream appear pending for an
  unexpectedly long time; the contract does not convert units or cap the
  timestamp magnitude.
- No contract behavior or ABI error codes change.
