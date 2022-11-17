//! Kleisli Iterator Composition
//!
//! This library exposes methods of composing iterators, such that we can
//! compile branching and sequencing operations easily, as well as
//! *fixed point* operations, which are need for path queries.
//!
//! The Kleisli composiiton takes a generator of an iterator:
//! ```rust
//! let f: Fn(A) -> Iterator<Item=B>
//! let g: Fn(B) -> Iterator<Item=C>
//! let f_g = kleislicompose(f,g)
//! let x : Vec<C> = f_g.collect()
//! ```
//!
//! The iterators should also be clonable, which is important for
//! producing back tracking combinators.
//!
use std::iter::IntoIterator;

pub struct KleisliCompose<
    A: Copy,
    U: IntoIterator,
    S: IntoIterator,
    F: FnMut(A) -> U,
    G: FnMut(U::Item) -> S,
> {
    _a: std::marker::PhantomData<A>,
    f: F,
    g: G,
}

impl<A: Copy, U: IntoIterator, S: IntoIterator, F: FnMut(A) -> U, G: FnMut(U::Item) -> S>
    KleisliCompose<A, U, S, F, G>
{
    pub fn new(f: F, g: G) -> KleisliCompose<A, U, S, F, G> {
        KleisliCompose {
            _a: Default::default(),
            f,
            g,
        }
    }
}

// Composition of Kleisli arrows (>=>)x
pub fn kleisli_compose<A, U, S, F, G>(f: F, g: G) -> KleisliCompose<A, U, S, F, G>
where
    A: Copy,
    U: IntoIterator,
    S: IntoIterator,
    F: FnMut(A) -> U,
    G: FnMut(U::Item) -> S,
{
    KleisliCompose::new(f, g)
}

pub struct ApplyKleisliCompose<
    A: Copy,
    U: IntoIterator,
    S: IntoIterator,
    F: FnMut(A) -> U,
    G: FnMut(U::Item) -> S,
> {
    a: A,
    k: KleisliCompose<A, U, S, F, G>,
}

impl<A: Copy, U: IntoIterator, S: IntoIterator, F: FnMut(A) -> U, G: FnMut(U::Item) -> S>
    ApplyKleisliCompose<A, U, S, F, G>
{
    pub fn new(a: A, kc: KleisliCompose<A, U, S, F, G>) -> Self {
        ApplyKleisliCompose { a, k: kc }
    }
}

impl<A: Copy, U: IntoIterator, S: IntoIterator, F: FnMut(A) -> U, G: FnMut(U::Item) -> S> Iterator
    for ApplyKleisliCompose<A, U, S, F, G>
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        (self.k.f)(self.a)
            .into_iter()
            .flat_map(|x| (self.k.g)(x).into_iter())
            .next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_it() {
        let db = vec![(1, 2), (2, 3), (3, 4), (1, 5), (5, 7)];
        let iter = db.into_iter();
        let iter2 = iter.clone();
        let iter3 = iter.clone();
        let res: Vec<_> = iter
            .flat_map(|i| {
                ApplyKleisliCompose::new(
                    i,
                    kleisli_compose(
                        move |t: (usize, usize)| {
                            iter3.clone().map(|x| x.1).filter(move |x| *x == t.0)
                        },
                        move |s| iter2.clone().map(|x| x.1).filter(move |x| *x == s),
                    ),
                )
            })
            .collect();
        eprintln!("{res:?}");
        panic!()
    }
}
