//! HOMT Category-Theory Functors: D (Decode), F (Interpret), O (Evolve).
//!
//! HOMT = O∘F∘D — every data path through BahyWay must compose these three.
//!
//! Invariant: no functor stores state.  All state lives in the Orbit (Journal).

/// D functor — decode raw bytes from storage into a structured form.
///
/// Implementors: `Kaki::from_bytes`, AAOL parser, KISHIB expander.
pub trait Decode: Sized {
    /// The output type produced by decoding `self`.
    type Output;

    /// Pure decode — no I/O, no allocation beyond `Self::Output`.
    fn decode(&self) -> Self::Output;
}

/// F functor — interpret decoded data into a projected score or fuzzy value.
///
/// `D` is the decoded form (output of `Decode`).  Implementors: StoryEngine
/// projection, VGCA-Δ quality operator, HPS scorer.
pub trait Interpret<D> {
    /// The projection produced — typically a `ProjectedState` or score vector.
    type Projection;

    /// Derive a projection from a decoded particle.  Read-only.
    fn interpret(&self, decoded: &D) -> Self::Projection;
}

/// O functor (Orbit Evolution) — fold an event into the current state.
///
/// This is the CQRS "left fold" that drives the StoryEngine.
/// Each call produces a *new* value; `Self` is never mutated.
pub trait Evolve: Sized {
    /// The event type that can advance this state.
    type Event;

    /// Apply `event` to `self` and return the successor state.
    /// Deterministic — same (state, event) always → same result.
    fn evolve(self, event: &Self::Event) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Minimal smoke test: verify the functor traits compose correctly ──

    #[derive(Clone, Copy)]
    struct RawScore(u8);

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct NormScore(f32);

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Label(&'static str);

    // D: raw u8 → normalised f32
    impl Decode for RawScore {
        type Output = NormScore;
        fn decode(&self) -> NormScore {
            NormScore(self.0 as f32 / 255.0)
        }
    }

    struct Classifier;

    // F: normalised score → human label
    impl Interpret<NormScore> for Classifier {
        type Projection = Label;
        fn interpret(&self, d: &NormScore) -> Label {
            if d.0 >= 0.6 {
                Label("GOLDEN")
            } else if d.0 >= 0.2 {
                Label("FUZZY")
            } else {
                Label("DEAD")
            }
        }
    }

    // O: accumulate scores (trivial fold for test)
    impl Evolve for NormScore {
        type Event = f32;
        fn evolve(self, delta: &f32) -> Self {
            NormScore((self.0 + delta).clamp(0.0, 1.0))
        }
    }

    #[test]
    fn d_functor_normalises() {
        let raw = RawScore(255);
        assert_eq!(raw.decode(), NormScore(1.0));
        let raw = RawScore(0);
        assert_eq!(raw.decode(), NormScore(0.0));
    }

    #[test]
    fn f_functor_classifies() {
        let c = Classifier;
        assert_eq!(c.interpret(&NormScore(0.9)), Label("GOLDEN"));
        assert_eq!(c.interpret(&NormScore(0.4)), Label("FUZZY"));
        assert_eq!(c.interpret(&NormScore(0.1)), Label("DEAD"));
    }

    #[test]
    fn o_functor_folds_deterministically() {
        let s = NormScore(0.5);
        let s2 = s.evolve(&0.2);
        let s3 = s2.evolve(&(-0.1));
        assert!((s3.0 - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn homt_composition_d_then_f() {
        let c = Classifier;
        // HOMT = F(D(raw)) — the full composition
        let raw = RawScore(200);
        let decoded = raw.decode();
        let label = c.interpret(&decoded);
        assert_eq!(label, Label("GOLDEN")); // 200/255 ≈ 0.78 → GOLDEN
    }
}
