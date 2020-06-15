//!
//! The `std::convert::from_bits_signed` function.
//!

use num_bigint::BigInt;

use ff::PrimeField;
use franklin_crypto::bellman::ConstraintSystem;
use franklin_crypto::circuit::expression::Expression;
use franklin_crypto::circuit::num::AllocatedNum;

use zinc_bytecode::IntegerType;

use crate::core::execution_state::evaluation_stack::EvaluationStack;
use crate::error::MalformedBytecode;
use crate::error::RuntimeError;
use crate::gadgets;
use crate::gadgets::scalar::Scalar;
use crate::instructions::call_std::INativeCallable;
use crate::IEngine;

pub struct SignedFromBits {
    bitlength: usize,
}

impl SignedFromBits {
    pub fn new(inputs_count: usize) -> Self {
        Self {
            bitlength: inputs_count,
        }
    }
}

impl<E: IEngine> INativeCallable<E> for SignedFromBits {
    fn call<CS: ConstraintSystem<E>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result<(), RuntimeError> {
        if self.bitlength >= E::Fr::CAPACITY as usize {
            return Err(MalformedBytecode::InvalidArguments(format!(
                "signed_from_bits: integer type with length {} is not supported",
                self.bitlength
            ))
            .into());
        }

        let mut bits = Vec::with_capacity(self.bitlength);
        for i in 0..self.bitlength {
            let bit = stack.pop()?.try_into_value()?;
            let boolean = bit.to_boolean(cs.namespace(|| format!("to_boolean {}", i)))?;
            bits.push(boolean);
        }

        let sign_bit = bits[self.bitlength - 1].clone();
        bits.push(sign_bit.not());

        let num =
            AllocatedNum::pack_bits_to_element(cs.namespace(|| "pack_bits_to_element"), &bits)?;

        let num_expr = Expression::from(&num);
        let base_value = BigInt::from(1) << self.bitlength;
        let base_expr = Expression::<E>::constant::<CS>(
            gadgets::scalar::fr_bigint::bigint_to_fr::<E>(&base_value).expect("length is too big"),
        );

        let num = (num_expr - base_expr).into_number(cs.namespace(|| "result"))?;

        let int_type = IntegerType {
            is_signed: true,
            bitlength: self.bitlength,
        };

        let scalar =
            Scalar::new_unchecked_variable(num.get_value(), num.get_variable(), int_type.into());

        stack.push(scalar.into())?;

        Ok(())
    }
}
