//! Utilities for signature of *systems* in ECS.

use std::any::TypeId;

use crate::Component;

/// Signature of the *system* in ECS.
///
/// Describes which types are handled by the system.
///
pub trait Signature {
    /// Array of [TypeId]s which represents set of types in this signature.
    fn type_ids() -> Box<[TypeId]>;
}

// Generate implementations of Signature for empty tuple (unit type)
// and for tuples up to 12 elements.

impl Signature for () {
    fn type_ids() -> Box<[TypeId]> {
        Box::from([])
    }
}

impl<A> Signature for (A,)
where
    A: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([TypeId::of::<A>()])
    }
}

impl<A, B> Signature for (A, B)
where
    A: Component,
    B: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([TypeId::of::<A>(), TypeId::of::<B>()])
    }
}

impl<A, B, C> Signature for (A, B, C)
where
    A: Component,
    B: Component,
    C: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()])
    }
}

impl<A, B, C, D> Signature for (A, B, C, D)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
        ])
    }
}

impl<A, B, C, D, E> Signature for (A, B, C, D, E)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
        ])
    }
}

impl<A, B, C, D, E, F> Signature for (A, B, C, D, E, F)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
        ])
    }
}

impl<A, B, C, D, E, F, G> Signature for (A, B, C, D, E, F, G)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
    G: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
            TypeId::of::<G>(),
        ])
    }
}

impl<A, B, C, D, E, F, G, H> Signature for (A, B, C, D, E, F, G, H)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
    G: Component,
    H: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
            TypeId::of::<G>(),
            TypeId::of::<H>(),
        ])
    }
}

impl<A, B, C, D, E, F, G, H, I> Signature for (A, B, C, D, E, F, G, H, I)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
    G: Component,
    H: Component,
    I: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
            TypeId::of::<G>(),
            TypeId::of::<H>(),
            TypeId::of::<I>(),
        ])
    }
}

impl<A, B, C, D, E, F, G, H, I, J> Signature for (A, B, C, D, E, F, G, H, I, J)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
    G: Component,
    H: Component,
    I: Component,
    J: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
            TypeId::of::<G>(),
            TypeId::of::<H>(),
            TypeId::of::<I>(),
            TypeId::of::<J>(),
        ])
    }
}

impl<A, B, C, D, E, F, G, H, I, J, K> Signature for (A, B, C, D, E, F, G, H, I, J, K)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
    G: Component,
    H: Component,
    I: Component,
    J: Component,
    K: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
            TypeId::of::<G>(),
            TypeId::of::<H>(),
            TypeId::of::<I>(),
            TypeId::of::<J>(),
            TypeId::of::<K>(),
        ])
    }
}

impl<A, B, C, D, E, F, G, H, I, J, K, L> Signature for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    A: Component,
    B: Component,
    C: Component,
    D: Component,
    E: Component,
    F: Component,
    G: Component,
    H: Component,
    I: Component,
    J: Component,
    K: Component,
    L: Component,
{
    fn type_ids() -> Box<[TypeId]> {
        Box::from([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
            TypeId::of::<E>(),
            TypeId::of::<F>(),
            TypeId::of::<G>(),
            TypeId::of::<H>(),
            TypeId::of::<I>(),
            TypeId::of::<J>(),
            TypeId::of::<K>(),
            TypeId::of::<L>(),
        ])
    }
}
