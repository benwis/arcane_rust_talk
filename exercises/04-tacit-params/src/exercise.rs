// ---------------------------------------------------------------------------
// Step 0: see the problem. Uncomment this block and run `cargo build`.
// ---------------------------------------------------------------------------
//
// pub trait IntoLabelNaive {
//     fn into_label(self) -> String;
// }
//
// impl<T: std::fmt::Display> IntoLabelNaive for T {
//     fn into_label(self) -> String {
//         self.to_string()
//     }
// }
//
// impl<F: FnOnce() -> String> IntoLabelNaive for F {
//     fn into_label(self) -> String {
//         self()
//     }
// }
//
// error[E0119]: conflicting implementations of trait `IntoLabelNaive`

// ---------------------------------------------------------------------------
// Step 1: add a marker parameter.
//
// The trait and `label` are given. Write:
//
//   * four uninhabited marker types: `pub enum ViaDisplay {}`, `ViaFn`,
//     `ViaIter`, `ViaOption` (the tests turbofish them, so use these names)
//   * `impl<T: Display> IntoLabel<ViaDisplay> for T`
//   * `impl<F: FnOnce() -> String> IntoLabel<ViaFn> for F`
//   * `impl<I> IntoLabel<ViaIter> for I where I: Iterator, I::Item: Display`
//
// Since each impl names a different `Marker`, they're impls of *different
// traits* (`IntoLabel<ViaDisplay>` vs `IntoLabel<ViaFn>`) as far as coherence
// is concerned, so they can't overlap.
//
// Step 2: `Option<T>` where `T` is itself labelable.
//
// `Option<T>` should work for *any* `T: IntoLabel<M>`, so the `Option` impl
// has to carry the inner marker along too or `M` is unconstrained (E0207).
// Hint: markers can be tuples: `IntoLabel<(ViaOption, M)>`.
// ---------------------------------------------------------------------------

pub trait IntoLabel<Marker> {
    fn into_label(self) -> String;
}

/// Callers just write `label(x)`. `M` is inferred from whichever impl fits.
pub fn label<M>(x: impl IntoLabel<M>) -> String {
    x.into_label()
}
