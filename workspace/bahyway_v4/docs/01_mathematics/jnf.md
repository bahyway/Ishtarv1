# Jordan Normal Form in EnkiDB

> **DubSar Help** | `Math > JNF` | Mathematics

## Purpose

Jordan Normal Form (JNF) is the algebraic structure underlying every Tribe in
EnkiDB. A Tribe is a Jordan Block; its particles are the basis vectors; the
Orbit is the nilpotent chain connecting historical states.

## Structure

```
J_λ = [ λ  1  0  0 ]
      [ 0  λ  1  0 ]
      [ 0  0  λ  1 ]
      [ 0  0  0  λ ]
```

- λ = particle eigenvalue (HPS-derived).
- Off-diagonal 1s = the Orbit chain connecting successive states.
- Nilpotency: N^k = 0 for k ≥ orbit length — old events vanish naturally.

## Scaling Property

Because the full EnkiDB matrix is block-diagonal (one Jordan Block per Tribe),
adding a new Tribe adds a new block with zero interference to existing blocks.
This is the mathematical basis for infinite horizontal scaling.

## See Also

- `01_mathematics/enlil_algebra.md`
- `05_storage/enkidb.md`
