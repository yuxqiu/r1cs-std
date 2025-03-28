use ark_relations::gr1cs::SynthesisError;

use super::PairingVar as PG;

use crate::{
    fields::{fp::FpVar, fp2::Fp2Var, fp4::Fp4Var, FieldVar},
    groups::mnt4::{
        AteAdditionCoefficientsVar, AteDoubleCoefficientsVar, G1PreparedVar, G1Var, G2PreparedVar,
        G2ProjectiveExtendedVar, G2Var,
    },
};
use ark_ec::mnt4::{MNT4Config, MNT4};

use core::marker::PhantomData;

/// Specifies the constraints for computing a pairing in a MNT4 bilinear group.
pub struct PairingVar<P: MNT4Config>(PhantomData<P>);

type Fp2G<P> = Fp2Var<<P as MNT4Config>::Fp2Config>;
type Fp4G<P> = Fp4Var<<P as MNT4Config>::Fp4Config>;
/// A variable corresponding to `ark_ec::mnt4::GT`.
pub type GTVar<P> = Fp4G<P>;

impl<P: MNT4Config> PairingVar<P> {
    #[tracing::instrument(target = "gr1cs", skip(r))]
    pub(crate) fn doubling_step_for_flipped_miller_loop(
        r: &G2ProjectiveExtendedVar<P>,
    ) -> Result<(G2ProjectiveExtendedVar<P>, AteDoubleCoefficientsVar<P>), SynthesisError> {
        let a = r.t.square()?;
        let b = r.x.square()?;
        let c = r.y.square()?;
        let d = c.square()?;
        let e = (&r.x + &c).square()? - &b - &d;
        let f = (b.double()? + &b) + &a * P::TWIST_COEFF_A;
        let g = f.square()?;

        let d_eight = d.double()?.double()?.double()?;

        let e2 = e.double()?;
        let x = &g - &e2.double()?;

        let y = &f * (&e2 - &x) - &d_eight;
        let z = (&r.y + &r.z).square()? - &c - &r.z.square()?;
        let t = z.square()?;

        let r2 = G2ProjectiveExtendedVar { x, y, z, t };
        let c_h = (&r2.z + &r.t).square()? - &r2.t - &a;
        let c_4c = c.double()?.double()?;
        let c_j = (&f + &r.t).square()? - &g - &a;
        let c_l = (&f + &r.x).square()? - &g - &b;
        let coeff = AteDoubleCoefficientsVar {
            c_h,
            c_4c,
            c_j,
            c_l,
        };

        Ok((r2, coeff))
    }

    #[tracing::instrument(target = "gr1cs", skip(r))]
    pub(crate) fn mixed_addition_step_for_flipped_miller_loop(
        x: &Fp2G<P>,
        y: &Fp2G<P>,
        r: &G2ProjectiveExtendedVar<P>,
    ) -> Result<(G2ProjectiveExtendedVar<P>, AteAdditionCoefficientsVar<P>), SynthesisError> {
        let a = y.square()?;
        let b = &r.t * x;
        let d = ((&r.z + y).square()? - &a - &r.t) * &r.t;
        let h = &b - &r.x;
        let i = h.square()?;
        let e = i.double()?.double()?;
        let j = &h * &e;
        let v = &r.x * &e;
        let ry2 = r.y.double()?;
        let l1 = &d - &ry2;

        let x = l1.square()? - &j - &v.double()?;
        let y = &l1 * &(&v - &x) - j * &ry2;
        let z = (&r.z + &h).square()? - &r.t - &i;
        let t = z.square()?;

        let r2 = G2ProjectiveExtendedVar {
            x,
            y,
            z: z.clone(),
            t,
        };
        let coeff = AteAdditionCoefficientsVar { c_l1: l1, c_rz: z };

        Ok((r2, coeff))
    }

    #[tracing::instrument(target = "gr1cs", skip(p, q))]
    pub(crate) fn ate_miller_loop(
        p: &G1PreparedVar<P>,
        q: &G2PreparedVar<P>,
    ) -> Result<Fp4G<P>, SynthesisError> {
        let l1_coeff = Fp2G::<P>::new(p.x.clone(), FpVar::<P::Fp>::zero()) - &q.x_over_twist;

        let mut f = Fp4G::<P>::one();

        let mut add_idx: usize = 0;

        // code below gets executed for all bits (EXCEPT the MSB itself) of
        // mnt6_param_p (skipping leading zeros) in MSB to LSB order
        let y_over_twist_neg = &q.y_over_twist.negate()?;
        for (dbl_idx, bit) in P::ATE_LOOP_COUNT.iter().skip(1).enumerate() {
            let dc = &q.double_coefficients[dbl_idx];

            let g_rr_at_p = Fp4G::<P>::new(
                &dc.c_l - &dc.c_4c - &dc.c_j * &p.x_twist,
                &dc.c_h * &p.y_twist,
            );

            f = f.square()? * &g_rr_at_p;

            let g_rq_at_p;
            // Compute l_{R,Q}(P) if bit == 1, and l_{R,-Q}(P) if bit == -1
            if *bit == 1 {
                let ac = &q.addition_coefficients[add_idx];
                add_idx += 1;

                g_rq_at_p = Fp4G::<P>::new(
                    &ac.c_rz * &p.y_twist,
                    (&q.y_over_twist * &ac.c_rz + &l1_coeff * &ac.c_l1).negate()?,
                );
            } else if *bit == -1 {
                let ac = &q.addition_coefficients[add_idx];
                add_idx += 1;

                g_rq_at_p = Fp4G::<P>::new(
                    &ac.c_rz * &p.y_twist,
                    (y_over_twist_neg * &ac.c_rz + &l1_coeff * &ac.c_l1).negate()?,
                );
            } else {
                continue;
            }

            f *= &g_rq_at_p;
        }

        if P::ATE_IS_LOOP_COUNT_NEG {
            let ac = &q.addition_coefficients[add_idx];

            let g_rnegr_at_p = Fp4G::<P>::new(
                &ac.c_rz * &p.y_twist,
                (&q.y_over_twist * &ac.c_rz + &l1_coeff * &ac.c_l1).negate()?,
            );
            f = (&f * &g_rnegr_at_p).inverse()?;
        }

        Ok(f)
    }

    #[tracing::instrument(target = "gr1cs", skip(value))]
    pub(crate) fn final_exponentiation(value: &Fp4G<P>) -> Result<GTVar<P>, SynthesisError> {
        let value_inv = value.inverse()?;
        let value_to_first_chunk = Self::final_exponentiation_first_chunk(value, &value_inv)?;
        let value_inv_to_first_chunk = Self::final_exponentiation_first_chunk(&value_inv, value)?;
        Self::final_exponentiation_last_chunk(&value_to_first_chunk, &value_inv_to_first_chunk)
    }

    #[tracing::instrument(target = "gr1cs", skip(elt, elt_inv))]
    fn final_exponentiation_first_chunk(
        elt: &Fp4G<P>,
        elt_inv: &Fp4G<P>,
    ) -> Result<Fp4G<P>, SynthesisError> {
        // (q^2-1)

        // elt_q2 = elt^(q^2)
        let elt_q2 = elt.unitary_inverse()?;
        // elt_q2_over_elt = elt^(q^2-1)
        Ok(elt_q2 * elt_inv)
    }

    #[tracing::instrument(target = "gr1cs", skip(elt, elt_inv))]
    fn final_exponentiation_last_chunk(
        elt: &Fp4G<P>,
        elt_inv: &Fp4G<P>,
    ) -> Result<Fp4G<P>, SynthesisError> {
        let elt_clone = elt.clone();
        let elt_inv_clone = elt_inv.clone();

        let mut elt_q = elt.clone();
        elt_q.frobenius_map_in_place(1)?;

        let w1_part = elt_q.cyclotomic_exp(&P::FINAL_EXPONENT_LAST_CHUNK_1)?;
        let w0_part = if P::FINAL_EXPONENT_LAST_CHUNK_W0_IS_NEG {
            elt_inv_clone.cyclotomic_exp(&P::FINAL_EXPONENT_LAST_CHUNK_ABS_OF_W0)?
        } else {
            elt_clone.cyclotomic_exp(&P::FINAL_EXPONENT_LAST_CHUNK_ABS_OF_W0)?
        };

        Ok(w1_part * &w0_part)
    }
}

impl<P: MNT4Config> PG<MNT4<P>> for PairingVar<P> {
    type G1Var = G1Var<P>;
    type G2Var = G2Var<P>;
    type G1PreparedVar = G1PreparedVar<P>;
    type G2PreparedVar = G2PreparedVar<P>;
    type GTVar = GTVar<P>;

    #[tracing::instrument(target = "gr1cs")]
    fn miller_loop(
        ps: &[Self::G1PreparedVar],
        qs: &[Self::G2PreparedVar],
    ) -> Result<Self::GTVar, SynthesisError> {
        let mut result = Fp4G::<P>::one();
        for (p, q) in ps.iter().zip(qs) {
            result *= Self::ate_miller_loop(p, q)?;
        }

        Ok(result)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn final_exponentiation(r: &Self::GTVar) -> Result<Self::GTVar, SynthesisError> {
        Self::final_exponentiation(r)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn prepare_g1(p: &Self::G1Var) -> Result<Self::G1PreparedVar, SynthesisError> {
        Self::G1PreparedVar::from_group_var(p)
    }

    #[tracing::instrument(target = "gr1cs")]
    fn prepare_g2(q: &Self::G2Var) -> Result<Self::G2PreparedVar, SynthesisError> {
        Self::G2PreparedVar::from_group_var(q)
    }
}
