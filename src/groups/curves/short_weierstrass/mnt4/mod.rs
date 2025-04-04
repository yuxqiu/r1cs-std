use ark_ec::mnt4::{
    g2::{AteAdditionCoefficients, AteDoubleCoefficients},
    G1Prepared, G2Prepared, MNT4Config,
};
use ark_ff::Field;
use ark_relations::gr1cs::{Namespace, SynthesisError};
use ark_std::vec::Vec;

use crate::{
    fields::{fp::FpVar, fp2::Fp2Var},
    groups::curves::short_weierstrass::ProjectiveVar,
    pairing::mnt4::PairingVar,
    prelude::*,
};
use core::borrow::Borrow;
use educe::Educe;

/// Represents a projective point in G1.
pub type G1Var<P> = ProjectiveVar<<P as MNT4Config>::G1Config, FpVar<<P as MNT4Config>::Fp>>;

/// Represents a projective point in G2.
pub type G2Var<P> = ProjectiveVar<<P as MNT4Config>::G2Config, Fp2G<P>>;

/// Represents the cached precomputation that can be performed on a G1 element
/// which enables speeding up pairing computation.
#[derive(Educe)]
#[educe(Clone, Debug)]
pub struct G1PreparedVar<P: MNT4Config> {
    #[doc(hidden)]
    pub x: FpVar<P::Fp>,
    #[doc(hidden)]
    pub y: FpVar<P::Fp>,
    #[doc(hidden)]
    pub x_twist: Fp2Var<P::Fp2Config>,
    #[doc(hidden)]
    pub y_twist: Fp2Var<P::Fp2Config>,
}

impl<P: MNT4Config> AllocVar<G1Prepared<P>, P::Fp> for G1PreparedVar<P> {
    #[tracing::instrument(target = "gr1cs", skip(cs, f))]
    fn new_variable<T: Borrow<G1Prepared<P>>>(
        cs: impl Into<Namespace<P::Fp>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let g1_prep = f().map(|b| *b.borrow());

        let x = FpVar::new_variable(ark_relations::ns!(cs, "x"), || g1_prep.map(|g| g.x), mode)?;
        let y = FpVar::new_variable(ark_relations::ns!(cs, "y"), || g1_prep.map(|g| g.y), mode)?;
        let x_twist = Fp2Var::new_variable(
            ark_relations::ns!(cs, "x_twist"),
            || g1_prep.map(|g| g.x_twist),
            mode,
        )?;
        let y_twist = Fp2Var::new_variable(
            ark_relations::ns!(cs, "y_twist"),
            || g1_prep.map(|g| g.y_twist),
            mode,
        )?;
        Ok(Self {
            x,
            y,
            x_twist,
            y_twist,
        })
    }
}

impl<P: MNT4Config> G1PreparedVar<P> {
    /// Returns the value assigned to `self` in the underlying constraint
    /// system.
    pub fn value(&self) -> Result<G1Prepared<P>, SynthesisError> {
        let (x, y, x_twist, y_twist) = (
            self.x.value()?,
            self.y.value()?,
            self.x_twist.value()?,
            self.y_twist.value()?,
        );
        Ok(G1Prepared {
            x,
            y,
            x_twist,
            y_twist,
        })
    }

    /// Constructs `Self` from a `G1Var`.
    #[tracing::instrument(target = "gr1cs")]
    pub fn from_group_var(q: &G1Var<P>) -> Result<Self, SynthesisError> {
        let q = q.to_affine()?;
        let x_twist = Fp2Var::new(&q.x * P::TWIST.c0, &q.x * P::TWIST.c1);
        let y_twist = Fp2Var::new(&q.y * P::TWIST.c0, &q.y * P::TWIST.c1);
        Ok(G1PreparedVar {
            x: q.x,
            y: q.y,
            x_twist,
            y_twist,
        })
    }
}

impl<P: MNT4Config> ToBytesGadget<P::Fp> for G1PreparedVar<P> {
    #[inline]
    #[tracing::instrument(target = "gr1cs")]
    fn to_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut x = self.x.to_bytes_le()?;
        let mut y = self.y.to_bytes_le()?;
        let mut x_twist = self.x_twist.to_bytes_le()?;
        let mut y_twist = self.y_twist.to_bytes_le()?;

        x.append(&mut y);
        x.append(&mut x_twist);
        x.append(&mut y_twist);
        Ok(x)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn to_non_unique_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut x = self.x.to_non_unique_bytes_le()?;
        let mut y = self.y.to_non_unique_bytes_le()?;
        let mut x_twist = self.x_twist.to_non_unique_bytes_le()?;
        let mut y_twist = self.y_twist.to_non_unique_bytes_le()?;

        x.append(&mut y);
        x.append(&mut x_twist);
        x.append(&mut y_twist);
        Ok(x)
    }
}

type Fp2G<P> = Fp2Var<<P as MNT4Config>::Fp2Config>;

/// Represents the cached precomputation that can be performed on a G2 element
/// which enables speeding up pairing computation.
#[derive(Educe)]
#[educe(Clone, Debug)]
pub struct G2PreparedVar<P: MNT4Config> {
    #[doc(hidden)]
    pub x: Fp2Var<P::Fp2Config>,
    #[doc(hidden)]
    pub y: Fp2Var<P::Fp2Config>,
    #[doc(hidden)]
    pub x_over_twist: Fp2Var<P::Fp2Config>,
    #[doc(hidden)]
    pub y_over_twist: Fp2Var<P::Fp2Config>,
    #[doc(hidden)]
    pub double_coefficients: Vec<AteDoubleCoefficientsVar<P>>,
    #[doc(hidden)]
    pub addition_coefficients: Vec<AteAdditionCoefficientsVar<P>>,
}

impl<P: MNT4Config> AllocVar<G2Prepared<P>, P::Fp> for G2PreparedVar<P> {
    #[tracing::instrument(target = "gr1cs", skip(cs, f))]
    fn new_variable<T: Borrow<G2Prepared<P>>>(
        cs: impl Into<Namespace<P::Fp>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let g2_prep = f().map(|b| b.borrow().clone());
        let g2 = g2_prep.as_ref().map_err(|e| *e);

        let x = Fp2Var::new_variable(ark_relations::ns!(cs, "x"), || g2.map(|g| g.x), mode)?;
        let y = Fp2Var::new_variable(ark_relations::ns!(cs, "y"), || g2.map(|g| g.y), mode)?;
        let x_over_twist = Fp2Var::new_variable(
            ark_relations::ns!(cs, "x_over_twist"),
            || g2.map(|g| g.x_over_twist),
            mode,
        )?;
        let y_over_twist = Fp2Var::new_variable(
            ark_relations::ns!(cs, "y_over_twist"),
            || g2.map(|g| g.y_over_twist),
            mode,
        )?;
        let double_coefficients = Vec::new_variable(
            ark_relations::ns!(cs, "double coeffs"),
            || g2.map(|g| g.double_coefficients.clone()),
            mode,
        )?;
        let addition_coefficients = Vec::new_variable(
            ark_relations::ns!(cs, "add coeffs"),
            || g2.map(|g| g.addition_coefficients.clone()),
            mode,
        )?;
        Ok(Self {
            x,
            y,
            x_over_twist,
            y_over_twist,
            double_coefficients,
            addition_coefficients,
        })
    }
}

impl<P: MNT4Config> ToBytesGadget<P::Fp> for G2PreparedVar<P> {
    #[inline]
    #[tracing::instrument(target = "gr1cs")]
    fn to_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut x = self.x.to_bytes_le()?;
        let mut y = self.y.to_bytes_le()?;
        let mut x_over_twist = self.x_over_twist.to_bytes_le()?;
        let mut y_over_twist = self.y_over_twist.to_bytes_le()?;

        x.append(&mut y);
        x.append(&mut x_over_twist);
        x.append(&mut y_over_twist);

        for coeff in &self.double_coefficients {
            x.extend_from_slice(&coeff.to_bytes_le()?);
        }
        for coeff in &self.addition_coefficients {
            x.extend_from_slice(&coeff.to_bytes_le()?);
        }
        Ok(x)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn to_non_unique_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut x = self.x.to_non_unique_bytes_le()?;
        let mut y = self.y.to_non_unique_bytes_le()?;
        let mut x_over_twist = self.x_over_twist.to_non_unique_bytes_le()?;
        let mut y_over_twist = self.y_over_twist.to_non_unique_bytes_le()?;

        x.append(&mut y);
        x.append(&mut x_over_twist);
        x.append(&mut y_over_twist);

        for coeff in &self.double_coefficients {
            x.extend_from_slice(&coeff.to_non_unique_bytes_le()?);
        }
        for coeff in &self.addition_coefficients {
            x.extend_from_slice(&coeff.to_non_unique_bytes_le()?);
        }
        Ok(x)
    }
}

impl<P: MNT4Config> G2PreparedVar<P> {
    /// Returns the value assigned to `self` in the underlying constraint
    /// system.
    pub fn value(&self) -> Result<G2Prepared<P>, SynthesisError> {
        let x = self.x.value()?;
        let y = self.y.value()?;
        let x_over_twist = self.x_over_twist.value()?;
        let y_over_twist = self.y_over_twist.value()?;
        let double_coefficients = self
            .double_coefficients
            .iter()
            .map(|coeff| coeff.value())
            .collect::<Result<Vec<AteDoubleCoefficients<P>>, _>>()?;
        let addition_coefficients = self
            .addition_coefficients
            .iter()
            .map(|coeff| coeff.value())
            .collect::<Result<Vec<AteAdditionCoefficients<P>>, _>>()?;
        Ok(G2Prepared {
            x,
            y,
            x_over_twist,
            y_over_twist,
            double_coefficients,
            addition_coefficients,
        })
    }

    /// Constructs `Self` from a `G2Var`.
    #[tracing::instrument(target = "gr1cs")]
    pub fn from_group_var(q: &G2Var<P>) -> Result<Self, SynthesisError> {
        let twist_inv = P::TWIST.inverse().unwrap();
        let q = q.to_affine()?;

        let mut g2p = G2PreparedVar {
            x: q.x.clone(),
            y: q.y.clone(),
            x_over_twist: &q.x * twist_inv,
            y_over_twist: &q.y * twist_inv,
            double_coefficients: vec![],
            addition_coefficients: vec![],
        };

        let mut r = G2ProjectiveExtendedVar {
            x: q.x.clone(),
            y: q.y.clone(),
            z: Fp2G::<P>::one(),
            t: Fp2G::<P>::one(),
        };

        for bit in P::ATE_LOOP_COUNT.iter().skip(1) {
            let (r2, coeff) = PairingVar::<P>::doubling_step_for_flipped_miller_loop(&r)?;
            g2p.double_coefficients.push(coeff);
            r = r2;

            let add_coeff;
            let r_temp;
            match bit {
                1 => {
                    (r_temp, add_coeff) =
                        PairingVar::<P>::mixed_addition_step_for_flipped_miller_loop(
                            &q.x, &q.y, &r,
                        )?;
                },
                -1 => {
                    (r_temp, add_coeff) =
                        PairingVar::<P>::mixed_addition_step_for_flipped_miller_loop(
                            &q.x,
                            &q.y.negate()?,
                            &r,
                        )?;
                },
                _ => continue,
            }
            g2p.addition_coefficients.push(add_coeff);
            r = r_temp;
        }

        if P::ATE_IS_LOOP_COUNT_NEG {
            let rz_inv = r.z.inverse()?;
            let rz2_inv = rz_inv.square()?;
            let rz3_inv = &rz_inv * &rz2_inv;

            let minus_r_affine_x = &r.x * &rz2_inv;
            let minus_r_affine_y = r.y.negate()? * &rz3_inv;

            let add_result = PairingVar::<P>::mixed_addition_step_for_flipped_miller_loop(
                &minus_r_affine_x,
                &minus_r_affine_y,
                &r,
            )?;
            g2p.addition_coefficients.push(add_result.1);
        }

        Ok(g2p)
    }
}

#[doc(hidden)]
#[derive(Educe)]
#[educe(Clone, Debug)]
pub struct AteDoubleCoefficientsVar<P: MNT4Config> {
    pub c_h: Fp2Var<P::Fp2Config>,
    pub c_4c: Fp2Var<P::Fp2Config>,
    pub c_j: Fp2Var<P::Fp2Config>,
    pub c_l: Fp2Var<P::Fp2Config>,
}

impl<P: MNT4Config> AllocVar<AteDoubleCoefficients<P>, P::Fp> for AteDoubleCoefficientsVar<P> {
    #[tracing::instrument(target = "gr1cs", skip(cs, f))]
    fn new_variable<T: Borrow<AteDoubleCoefficients<P>>>(
        cs: impl Into<Namespace<P::Fp>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let c_prep = f().map(|c| c.borrow().clone());
        let c = c_prep.as_ref().map_err(|e| *e);

        let c_h = Fp2Var::new_variable(ark_relations::ns!(cs, "c_h"), || c.map(|c| c.c_h), mode)?;
        let c_4c =
            Fp2Var::new_variable(ark_relations::ns!(cs, "c_4c"), || c.map(|c| c.c_4c), mode)?;
        let c_j = Fp2Var::new_variable(ark_relations::ns!(cs, "c_j"), || c.map(|c| c.c_j), mode)?;
        let c_l = Fp2Var::new_variable(ark_relations::ns!(cs, "c_l"), || c.map(|c| c.c_l), mode)?;
        Ok(Self {
            c_h,
            c_4c,
            c_j,
            c_l,
        })
    }
}

impl<P: MNT4Config> ToBytesGadget<P::Fp> for AteDoubleCoefficientsVar<P> {
    #[inline]
    #[tracing::instrument(target = "gr1cs")]
    fn to_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut c_h = self.c_h.to_bytes_le()?;
        let mut c_4c = self.c_4c.to_bytes_le()?;
        let mut c_j = self.c_j.to_bytes_le()?;
        let mut c_l = self.c_l.to_bytes_le()?;

        c_h.append(&mut c_4c);
        c_h.append(&mut c_j);
        c_h.append(&mut c_l);
        Ok(c_h)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn to_non_unique_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut c_h = self.c_h.to_non_unique_bytes_le()?;
        let mut c_4c = self.c_4c.to_non_unique_bytes_le()?;
        let mut c_j = self.c_j.to_non_unique_bytes_le()?;
        let mut c_l = self.c_l.to_non_unique_bytes_le()?;

        c_h.append(&mut c_4c);
        c_h.append(&mut c_j);
        c_h.append(&mut c_l);
        Ok(c_h)
    }
}

impl<P: MNT4Config> AteDoubleCoefficientsVar<P> {
    /// Returns the value assigned to `self` in the underlying constraint
    /// system.
    pub fn value(&self) -> Result<AteDoubleCoefficients<P>, SynthesisError> {
        let (c_h, c_4c, c_j, c_l) = (
            self.c_l.value()?,
            self.c_4c.value()?,
            self.c_j.value()?,
            self.c_l.value()?,
        );
        Ok(AteDoubleCoefficients {
            c_h,
            c_4c,
            c_j,
            c_l,
        })
    }
}

#[doc(hidden)]
#[derive(Educe)]
#[educe(Clone, Debug)]
pub struct AteAdditionCoefficientsVar<P: MNT4Config> {
    pub c_l1: Fp2Var<P::Fp2Config>,
    pub c_rz: Fp2Var<P::Fp2Config>,
}

impl<P: MNT4Config> AllocVar<AteAdditionCoefficients<P>, P::Fp> for AteAdditionCoefficientsVar<P> {
    #[tracing::instrument(target = "gr1cs", skip(cs, f))]
    fn new_variable<T: Borrow<AteAdditionCoefficients<P>>>(
        cs: impl Into<Namespace<P::Fp>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();

        let c_prep = f().map(|c| c.borrow().clone());
        let c = c_prep.as_ref().map_err(|e| *e);

        let c_l1 =
            Fp2Var::new_variable(ark_relations::ns!(cs, "c_l1"), || c.map(|c| c.c_l1), mode)?;
        let c_rz =
            Fp2Var::new_variable(ark_relations::ns!(cs, "c_rz"), || c.map(|c| c.c_rz), mode)?;
        Ok(Self { c_l1, c_rz })
    }
}

impl<P: MNT4Config> ToBytesGadget<P::Fp> for AteAdditionCoefficientsVar<P> {
    #[inline]
    #[tracing::instrument(target = "gr1cs")]
    fn to_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut c_l1 = self.c_l1.to_bytes_le()?;
        let mut c_rz = self.c_rz.to_bytes_le()?;

        c_l1.append(&mut c_rz);
        Ok(c_l1)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn to_non_unique_bytes_le(&self) -> Result<Vec<UInt8<P::Fp>>, SynthesisError> {
        let mut c_l1 = self.c_l1.to_non_unique_bytes_le()?;
        let mut c_rz = self.c_rz.to_non_unique_bytes_le()?;

        c_l1.append(&mut c_rz);
        Ok(c_l1)
    }
}

impl<P: MNT4Config> AteAdditionCoefficientsVar<P> {
    /// Returns the value assigned to `self` in the underlying constraint
    /// system.
    pub fn value(&self) -> Result<AteAdditionCoefficients<P>, SynthesisError> {
        let (c_l1, c_rz) = (self.c_l1.value()?, self.c_rz.value()?);
        Ok(AteAdditionCoefficients { c_l1, c_rz })
    }
}

#[doc(hidden)]
pub struct G2ProjectiveExtendedVar<P: MNT4Config> {
    pub x: Fp2Var<P::Fp2Config>,
    pub y: Fp2Var<P::Fp2Config>,
    pub z: Fp2Var<P::Fp2Config>,
    pub t: Fp2Var<P::Fp2Config>,
}
