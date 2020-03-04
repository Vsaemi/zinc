use ff::PrimeField;
use bellman::ConstraintSystem;
use franklin_crypto::circuit::baby_eddsa::EddsaSignature;
use franklin_crypto::circuit::ecc::EdwardsPoint;
use franklin_crypto::jubjub::{FixedGenerators, JubjubParams};

use crate::{Engine, MalformedBytecode, Result};
use crate::core::EvaluationStack;
use crate::gadgets::Scalar;
use crate::gadgets::stdlib::NativeFunction;

pub struct VerifySchnorrSignature {
    msg_len: usize
}

impl VerifySchnorrSignature {
    pub fn new(args_count: usize) -> Result<Self> {
        if args_count < 6 {
            return Err(MalformedBytecode::InvalidArguments("schnorr::verify needs at least 6 arguments".into()).into());
        }

        Ok(Self { msg_len: args_count - 5 })
    }
}

impl<E: Engine> NativeFunction<E> for VerifySchnorrSignature {
    fn execute<CS>(&self, mut cs: CS, stack: &mut EvaluationStack<E>) -> Result
        where
            CS: ConstraintSystem<E>,
    {
        if self.msg_len > E::Fs::CAPACITY as usize {
            return Err(MalformedBytecode::InvalidArguments(
                format!("maximum message length for schnorr signature is {}", E::Fs::CAPACITY)
            ).into())
        }

        let mut message = Vec::new();
        for _ in 0..self.msg_len {
            let bit = stack.pop()?.value()?;
            message.push(bit);
        }
        // message.reverse();

        let pk_y = stack
            .pop()?
            .value()?
            .to_expression::<CS>()
            .into_number(cs.namespace(|| "to_number pk_y"))?;
        let pk_x = stack
            .pop()?
            .value()?
            .to_expression::<CS>()
            .into_number(cs.namespace(|| "to_number pk_x"))?;
        let s = stack
            .pop()?
            .value()?
            .to_expression::<CS>()
            .into_number(cs.namespace(|| "to_number s"))?;
        let r_y = stack
            .pop()?
            .value()?
            .to_expression::<CS>()
            .into_number(cs.namespace(|| "to_number r_y"))?;
        let r_x = stack
            .pop()?
            .value()?
            .to_expression::<CS>()
            .into_number(cs.namespace(|| "to_number r_x"))?;

        let r = EdwardsPoint::interpret(cs.namespace(|| "r"), &r_x, &r_y, E::jubjub_params())?;
        let pk = EdwardsPoint::interpret(cs.namespace(|| "pk"), &pk_x, &pk_y, E::jubjub_params())?;

        let signature = EddsaSignature { r, s, pk };

        let is_valid = verify_signature(
            cs.namespace(|| "verify_signature"),
            &message,
            &signature,
            E::jubjub_params(),
        )?;

        stack.push(is_valid.into())
    }
}

pub fn verify_signature<E, CS>(
    mut cs: CS,
    message: &[Scalar<E>],
    signature: &EddsaSignature<E>,
    params: &E::Params,
) -> Result<Scalar<E>>
    where
        E: Engine,
        CS: ConstraintSystem<E>,
{
    let message_bits = message
        .iter()
        .enumerate()
        .map(|(i, bit)| {
            bit.to_boolean(cs.namespace(|| format!("message bit {}", i)))
        })
        .collect::<Result<Vec<_>>>()?;

    let public_generator = params
        .generator(FixedGenerators::SpendingKeyGenerator)
        .clone();

    let generator = EdwardsPoint::witness(
        cs.namespace(|| "allocate public generator"),
        Some(public_generator),
        params,
    )?;

    let is_verified = signature.is_verified_raw_message_signature(
        cs.namespace(|| "is_verified_signature"),
        params,
        &message_bits,
        generator,
        E::Fr::CAPACITY as usize / 8
    )?;

    Scalar::from_boolean(cs.namespace(|| "from_boolean"), is_verified)
}

#[cfg(test)]
mod tests {
    use ff::Field;
    use ff::{PrimeField, PrimeFieldRepr};
    use franklin_crypto::circuit::test::TestConstraintSystem;
    use pairing::bn256::{Bn256, Fr};

    use zinc_bytecode::scalar::ScalarType;

    use super::*;
    use franklin_crypto::{eddsa, jubjub};
    use rand::Rng;
    use franklin_crypto::alt_babyjubjub::AltJubjubBn256;
    use franklin_crypto::jubjub::JubjubEngine;

    #[test]
    fn test_verify() -> Result {
        let params = AltJubjubBn256::new();
        let p_g = jubjub::FixedGenerators::SpendingKeyGenerator;
        let message = b"abc";

        let message_bits = message
            .iter()
            .map(|byte| {
                let mut bits = Vec::new();

                for i in 0..8 {
                    bits.push(byte & (1 << i) != 0);
                }

                bits
            })
            .flatten()
            .map(|b| Scalar::new_constant_bool(b))
            .collect::<Vec<_>>();

        let mut rng = rand::thread_rng();
        let key = eddsa::PrivateKey::<Bn256>(rng.gen());
        let pub_key = eddsa::PublicKey::from_private(&key, p_g, &params);
        let seed = eddsa::Seed::random_seed(&mut rng, message);

        let signature = key.sign_raw_message(message, &seed, p_g, &params, <Bn256 as JubjubEngine>::Fs::CAPACITY as usize / 8);

        let mut stack = EvaluationStack::<Bn256>::new();

        let mut sigs_bytes = [0u8; 32];
        signature.s.into_repr().write_le(& mut sigs_bytes[..]).expect("get LE bytes of signature S");
        let mut sigs_repr = <Fr as PrimeField>::Repr::from(0);
        sigs_repr.read_le(&sigs_bytes[..]).expect("interpret S as field element representation");
        let sigs_converted = Fr::from_repr(sigs_repr).unwrap();

        let (r_x, r_y) = signature.r.into_xy();
        let s = sigs_converted;
        let (pk_x, pk_y) = pub_key.0.into_xy();

        stack.push(Scalar::new_constant_fr(r_x, ScalarType::Field).into())?;
        stack.push(Scalar::new_constant_fr(r_y, ScalarType::Field).into())?;
        stack.push(Scalar::new_constant_fr(s, ScalarType::Field).into())?;
        stack.push(Scalar::new_constant_fr(pk_x, ScalarType::Field).into())?;
        stack.push(Scalar::new_constant_fr(pk_y, ScalarType::Field).into())?;
        for bit in message_bits.into_iter().rev() {
            stack.push(bit.into())?;
        }

        let mut cs = TestConstraintSystem::new();
        VerifySchnorrSignature::new(5 + 8 * message.len())
            .unwrap()
            .execute(cs.namespace(|| "signature check"), &mut stack)?;

        let is_valid = stack.pop()?.value()?;

        assert_eq!(is_valid.get_value(), Some(Fr::one()), "success");
        assert!(cs.is_satisfied(), "unsatisfied");
        assert_eq!(cs.which_is_unsatisfied(), None, "unconstrained");

        Ok(())
    }
}
