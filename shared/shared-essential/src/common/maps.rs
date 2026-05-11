//! map_n implementations
#[allow(unused_macros)]
macro_rules! map_n {
    ($fn:expr, $head:expr, $($tail:expr),+) => {
        if $head.is_some() && $($tail.is_some())&&* {
            Some($fn($head.unwrap(), $($tail.unwrap()),+))
        } else {
            None
        }
    };
}

pub fn map_2<A, B, C>(a: Option<A>, b: Option<B>, f: fn(A, B) -> C) -> Option<C> {
    match (a, b) {
        (Some(a), Some(b)) => Some(f(a, b)),
        _ => None,
    }
}

pub fn map_3<A, B, C, D>(
    a: Option<A>,
    b: Option<B>,
    c: Option<C>,
    f: fn(A, B, C) -> D,
) -> Option<D> {
    match (a, b, c) {
        (Some(a), Some(b), Some(c)) => Some(f(a, b, c)),
        _ => None,
    }
}
