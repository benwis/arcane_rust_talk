use std::fmt::Display;

pub trait IntoLabel<Marker> {
    fn into_label(self) -> String;
}

pub fn label<M>(x: impl IntoLabel<M>) -> String {
    x.into_label()
}

// Uninhabited: these exist only at the type level.
pub enum ViaDisplay {}
pub enum ViaFn {}
pub enum ViaIter {}
pub enum ViaOption {}

impl<T: Display> IntoLabel<ViaDisplay> for T {
    fn into_label(self) -> String {
        self.to_string()
    }
}

impl<F: FnOnce() -> String> IntoLabel<ViaFn> for F {
    fn into_label(self) -> String {
        self()
    }
}

impl<I> IntoLabel<ViaIter> for I
where
    I: Iterator,
    I::Item: Display,
{
    fn into_label(self) -> String {
        self.map(|item| item.to_string()).collect::<Vec<_>>().join(", ")
    }
}

// The inner marker `M` has to show up in the trait's parameters, or the impl
// would have a type parameter that nothing constrains (E0207).
impl<T, M> IntoLabel<(ViaOption, M)> for Option<T>
where
    T: IntoLabel<M>,
{
    fn into_label(self) -> String {
        match self {
            Some(inner) => inner.into_label(),
            None => String::from("none"),
        }
    }
}
