# Contract model

`escrow-core` is intentionally narrow. It is a custody state machine, not a
business workflow engine.

## State transitions

| Current state | Operation | Required authority | Result |
| --- | --- | --- | --- |
| absent | `deposit` | payer | `Pending` and token transfer into contract |
| `Pending` | `release` | configured verifier | token transfer to payee, `Released` |
| `Pending` | `refund` after deadline | payer | token transfer to payer, `Refunded` |
| terminal | any settlement operation | any | rejected |

The base contract has no fee leg. A fee-bearing product should compose an
explicit fee policy and test whether fees are refundable, rather than hiding
that policy inside a generic primitive.

## Extension rule

An extension must preserve the conservation rule:

```text
contract-held token amount = escrowed principal not yet paid out
```

If an extension uses an AMM, fee, insurance pool, or multiple tranches, it
must expose the corresponding accounting fields and test partial failure and
replay behavior.

