#[allow(unused_braces)]
/// Implements arithmetic traits (eg: `Add`, `Sub`, `Mul`) for the given type
/// using the impl in `$impl`.
///
/// Used primarily for implementing these traits for `FieldVar`s and
/// `GroupVar`s.
#[macro_export]
macro_rules! impl_ops {
    (
        $type: ty,
        $native: ty,
        $trait: ident,
        $fn: ident,
        $assign_trait: ident,
        $assign_fn: ident,
        $impl: expr,
        $constant_impl: expr,
        $($args:tt)*
    ) => {
        impl_bounded_ops!($type, $native, $trait, $fn, $assign_trait, $assign_fn, $impl, $constant_impl, ($($args)+), );
    };
}

/// Implements arithmetic traits (eg: `Add`, `Sub`, `Mul`) for the given type
/// using the impl in `$impl`.
///
/// Used primarily for implementing these traits for `FieldVar`s and
/// `GroupVar`s.
///
/// When compared to `impl_ops`, this macro allows specifying additional trait
/// bounds.
#[macro_export]
macro_rules! impl_bounded_ops {
    (
        $type: ty,
        $native: ty,
        $trait: ident,
        $fn: ident,
        $assign_trait: ident,
        $assign_fn: ident,
        $impl: expr,
        $constant_impl: expr,
        ($($params:tt)+),
        $($bounds:tt)*
    ) => {
        impl<'a, $($params)+> core::ops::$trait<&'a $type> for &'a $type
        where
            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces, clippy::redundant_closure_call)]
            fn $fn(self, other: &'a $type) -> Self::Output {
                ($impl)(self, other)
            }
        }

        impl<'a, $($params)+> core::ops::$trait<$type> for &'a $type
        where
            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: $type) -> Self::Output {
                core::ops::$trait::$fn(self, &other)
            }
        }

        impl<'a, $($params)+> core::ops::$trait<&'a $type> for $type
        where
            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: &'a $type) -> Self::Output {
                core::ops::$trait::$fn(&self, other)
            }
        }

        impl<$($params)+> core::ops::$trait<$type> for $type
        where

            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: $type) -> Self::Output {
                core::ops::$trait::$fn(&self, &other)
            }
        }

        impl<$($params)+> core::ops::$assign_trait<$type> for $type
        where

            $($bounds)*
        {
            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $assign_fn(&mut self, other: $type) {
                let result = core::ops::$trait::$fn(&*self, &other);
                *self = result
            }
        }

        impl<'a, $($params)+> core::ops::$assign_trait<&'a $type> for $type
        where

            $($bounds)*
        {
            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $assign_fn(&mut self, other: &'a $type) {
                let result = core::ops::$trait::$fn(&*self, other);
                *self = result
            }
        }

        impl<'a, $($params)+> core::ops::$trait<$native> for &'a $type
        where

            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces, clippy::redundant_closure_call)]
            fn $fn(self, other: $native) -> Self::Output {
                ($constant_impl)(self, other)
            }
        }

        impl<$($params)+> core::ops::$trait<$native> for $type
        where

            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: $native) -> Self::Output {
                core::ops::$trait::$fn(&self, other)
            }
        }

        impl<$($params)+> core::ops::$assign_trait<$native> for $type
        where

            $($bounds)*
        {

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $assign_fn(&mut self, other: $native) {
                let result = core::ops::$trait::$fn(&*self, other);
                *self = result
            }
        }
    }
}

/// Implements arithmetic traits (eg: `Add`, `Sub`, `Mul`) for the given type
/// using the impl in `$impl`.
///
/// Used primarily for implementing these traits for `FieldVar`s and
/// `GroupVar`s.
///
/// When compared to `impl_ops`, this macro allows specifying additional trait
/// bounds.
#[macro_export]
macro_rules! impl_bounded_ops_diff {
    (
        $type: ty,
        $native: ty,
        $other_type: ty,
        $other_native: ty,
        $trait: ident,
        $fn: ident,
        $assign_trait: ident,
        $assign_fn: ident,
        $impl: expr,
        $constant_impl: expr,
        ($($params:tt)+),
        $($bounds:tt)*
    ) => {
        impl<'a, $($params)+> core::ops::$trait<&'a $other_type> for &'a $type
        where
            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces, clippy::redundant_closure_call)]
            fn $fn(self, other: &'a $other_type) -> Self::Output {
                ($impl)(self, other)
            }
        }

        impl<'a, $($params)+> core::ops::$trait<$other_type> for &'a $type
        where
            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: $other_type) -> Self::Output {
                core::ops::$trait::$fn(self, &other)
            }
        }

        impl<'a, $($params)+> core::ops::$trait<&'a $other_type> for $type
        where
            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: &'a $other_type) -> Self::Output {
                core::ops::$trait::$fn(&self, other)
            }
        }

        impl<$($params)+> core::ops::$trait<$other_type> for $type
        where

            $($bounds)*
        {
            type Output = $type;

            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $fn(self, other: $other_type) -> Self::Output {
                core::ops::$trait::$fn(&self, &other)
            }
        }

        impl<$($params)+> core::ops::$assign_trait<$other_type> for $type
        where

            $($bounds)*
        {
            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $assign_fn(&mut self, other: $other_type) {
                let result = core::ops::$trait::$fn(&*self, &other);
                *self = result
            }
        }

        impl<'a, $($params)+> core::ops::$assign_trait<&'a $other_type> for $type
        where

            $($bounds)*
        {
            #[tracing::instrument(target = "gr1cs", skip(self))]
            #[allow(unused_braces)]
            fn $assign_fn(&mut self, other: &'a $other_type) {
                let result = core::ops::$trait::$fn(&*self, other);
                *self = result
            }
        }
    }
}
