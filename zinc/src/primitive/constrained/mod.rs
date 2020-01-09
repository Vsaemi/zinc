use std::fmt::{Debug, Display, Error, Formatter};
use std::marker::PhantomData;

use bellman::pairing::Engine;
use bellman::{ConstraintSystem, Variable};
use ff::{Field, PrimeField};
use franklin_crypto::bellman::{Namespace, SynthesisError};
use franklin_crypto::circuit::num::AllocatedNum;
use num_bigint::{BigInt, ToBigInt};

use crate::primitive::{utils, Primitive, PrimitiveOperations};
use crate::vm::RuntimeError;

#[derive(Debug, Clone)]
struct DataType {
    signed: bool,
    length: usize,
}

/// ConstrainedElement is an implementation of Element
/// that for every operation on elements generates corresponding R1CS constraints.
#[derive(Debug, Clone)]
pub struct FrPrimitive<E: Engine> {
    value: Option<E::Fr>,
    variable: Variable,
    data_type: Option<DataType>,
}

impl<E: Engine> FrPrimitive<E> {
    fn new(value: Option<E::Fr>, variable: Variable) -> Self {
        Self {
            value,
            variable,
            data_type: None
        }
    }

    fn as_allocated_num<CS: ConstraintSystem<E>>(&self, mut cs: CS) -> Result<AllocatedNum<E>, RuntimeError> {
        let num = AllocatedNum::alloc(
            cs.namespace(|| "allucated num"),
            || self.value.ok_or(SynthesisError::AssignmentMissing))
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "allocated num",
            |lc| lc + self.variable,
            |lc| lc + CS::one(),
            |lc| lc + num.get_variable());

        Ok(num)
    }
}

impl<E: Engine> Display for FrPrimitive<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.value {
            Some(value) => {
                let bigint = utils::fr_to_bigint::<E>(value);
                Display::fmt(&bigint, f)
            }
            None => Display::fmt("none", f),
        }
    }
}

impl<E: Engine> ToBigInt for FrPrimitive<E> {
    fn to_bigint(&self) -> Option<BigInt> {
        self.value
            .map(|fr| -> BigInt { utils::fr_to_bigint::<E>(&fr) })
    }
}

impl<EN: Debug + Engine> Primitive for FrPrimitive<EN> {}

pub struct ConstrainingFrOperations<E, CS>
where
    E: Engine,
    CS: ConstraintSystem<E>,
{
    cs: CS,
    counter: usize,
    pd: PhantomData<E>,
}

impl<E, CS> ConstrainingFrOperations<E, CS>
where
    E: Engine + Debug,
    CS: ConstraintSystem<E>,
{
    pub fn new(cs: CS) -> Self {
        Self {
            cs,
            counter: 0,
            pd: PhantomData,
        }
    }

    fn cs_namespace(&mut self) -> Namespace<E, CS::Root> {
        let s = format!("{}", self.counter);
        self.counter += 1;
        self.cs.namespace(|| s)
    }

    fn zero(&mut self) -> Result<FrPrimitive<E>, RuntimeError> {
        let value = E::Fr::zero();
        let mut cs = self.cs_namespace();
        let variable = cs
            .alloc(|| "zero_var", || Ok(value))
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "zero constraint",
            |lc| lc + variable,
            |lc| lc + CS::one(),
            |lc| lc,
        );

        Ok(FrPrimitive::new(Some(value), variable))
    }

    fn one() -> FrPrimitive<E> {
        FrPrimitive::new(Some(E::Fr::one()), CS::one())
    }

    #[allow(dead_code)]
    pub fn constraint_system(&mut self) -> &mut CS {
        &mut self.cs
    }

    fn abs(&mut self, value: FrPrimitive<E>) -> Result<FrPrimitive<E>, RuntimeError> {
        let zero = self.zero()?;
        let neg = PrimitiveOperations::neg(self, value.clone())?;
        let lt0 = PrimitiveOperations::lt(self, value.clone(), zero)?;
        self.conditional_select(lt0, neg, value)
    }

    fn bits(
        &mut self,
        index: FrPrimitive<E>,
        array_length: usize,
    ) -> Result<Vec<FrPrimitive<E>>, RuntimeError> {
        let length = self.constant_bigint(&array_length.into())?;
        let index_lt_length = self.lt(index.clone(), length)?;
        self.assert(index_lt_length)?;


        let mut cs = self.cs_namespace();
        let num = index.as_allocated_num(cs.namespace(|| "into_allocated_num"))?;
        let bit_length = utils::tree_height(array_length);
        let bits = num.into_bits_le_fixed(
            cs.namespace(|| "bits_le_fixed"), bit_length)
            .map_err(RuntimeError::SynthesisError)?;

        let bits = bits
            .into_iter()
            .map(|bit| FrPrimitive::new(
                bit.get_value_field::<E>(),
                bit.get_variable().expect("bit value expected").get_variable()
            ))
            .collect();

        Ok(bits)
    }

    fn recursive_select(
        &mut self,
        array: &[FrPrimitive<E>],
        index_bits: &[FrPrimitive<E>],
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        if array.len() == 1 {
            return Ok(array[0].clone());
        }

        let bit = index_bits.first().expect("recursion error");

        let mut new_array = Vec::new();
        for i in 0..(array.len() / 2) {
            let p = self.conditional_select(
                bit.clone(),
                array[i * 2 + 1].clone(),
                array[i * 2].clone(),
            )?;
            new_array.push(p);
        }

        if array.len() % 2 == 1 {
            new_array.push(array.last().unwrap().clone());
        }

        self.recursive_select(new_array.as_slice(), &index_bits[1..])
    }
}

impl<E, CS> PrimitiveOperations<FrPrimitive<E>> for ConstrainingFrOperations<E, CS>
where
    E: Debug + Engine,
    CS: ConstraintSystem<E>,
{
    fn variable_none(&mut self) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let variable = cs
            .alloc(
                || "variable value",
                || Err(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        Ok(FrPrimitive::new(None, variable))
    }

    fn variable_bigint(&mut self, value: &BigInt) -> Result<FrPrimitive<E>, RuntimeError> {
        let value = utils::bigint_to_fr::<E>(value)
            .ok_or_else(|| RuntimeError::InternalError("bigint_to_fr".into()))?;

        let mut cs = self.cs_namespace();

        let variable = cs
            .alloc(|| "variable value", || Ok(value))
            .map_err(RuntimeError::SynthesisError)?;

        Ok(FrPrimitive::new(Some(value), variable))
    }

    fn constant_bigint(&mut self, value: &BigInt) -> Result<FrPrimitive<E>, RuntimeError> {
        let value = utils::bigint_to_fr::<E>(value)
            .ok_or_else(|| RuntimeError::InternalError("bigint_to_fr".into()))?;

        let mut cs = self.cs_namespace();

        let variable = cs
            .alloc(|| "constant value", || Ok(value))
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "constant equation",
            |lc| lc + CS::one(),
            |lc| lc + (value, CS::one()),
            |lc| lc + variable,
        );

        Ok(FrPrimitive::new(Some(value), variable))
    }

    fn output(&mut self, element: FrPrimitive<E>) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let variable = cs
            .alloc_input(
                || "output value",
                || element.value.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "enforce output equality",
            |lc| lc + variable,
            |lc| lc + CS::one(),
            |lc| lc + element.variable,
        );

        Ok(FrPrimitive::new(element.value, variable))
    }

    fn type_check(&mut self, value: &FrPrimitive<E>) -> Result<(), RuntimeError> {
        let data_type = match value.data_type {
            Some(ref data_type) => data_type,
            None => return Ok(())
        };

        let adjusted_value = if data_type.signed {
            let offset_value = 1 << (data_type.length - 1);
            let offset = self.constant_bigint(&offset_value.into())?;
            self.add(value.clone(), offset)?
        } else {
            value.clone()
        };

        let mut cs = self.cs_namespace();
        let num = adjusted_value.as_allocated_num(cs.namespace(|| "as_allocated_num"))?;
        let _bits = num
            .into_bits_le_fixed(cs.namespace(|| "into_bits_le_fixed"), data_type.length)
            .map_err(RuntimeError::SynthesisError)?;

        Ok(())
    }

    fn add(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let sum = match (left.value, right.value) {
            (Some(l), Some(r)) => {
                let mut sum = l;
                sum.add_assign(&r);
                Some(sum)
            }
            _ => None,
        };

        let mut cs = self.cs_namespace();

        let sum_var = cs
            .alloc(
                || "sum variable",
                || sum.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "sum constraint",
            |lc| lc + left.variable + right.variable,
            |lc| lc + CS::one(),
            |lc| lc + sum_var,
        );

        Ok(FrPrimitive::new(sum, sum_var))
    }

    fn sub(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let diff = match (left.value, right.value) {
            (Some(l), Some(r)) => {
                let mut diff = l;
                diff.sub_assign(&r);
                Some(diff)
            }
            _ => None,
        };

        let mut cs = self.cs_namespace();

        let sum_var = cs
            .alloc(
                || "diff variable",
                || diff.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "diff constraint",
            |lc| lc + left.variable - right.variable,
            |lc| lc + CS::one(),
            |lc| lc + sum_var,
        );

        Ok(FrPrimitive::new(diff, sum_var))
    }

    fn mul(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let prod = match (left.value, right.value) {
            (Some(l), Some(r)) => {
                let mut prod = l;
                prod.mul_assign(&r);
                Some(prod)
            }
            _ => None,
        };

        let mut cs = self.cs_namespace();

        let sum_var = cs
            .alloc(
                || "prod variable",
                || prod.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "prod constraint",
            |lc| lc + left.variable,
            |lc| lc + right.variable,
            |lc| lc + sum_var,
        );

        Ok(FrPrimitive::new(prod, sum_var))
    }

    fn div_rem(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<(FrPrimitive<E>, FrPrimitive<E>), RuntimeError> {
        let nominator = left;
        let denominator = right;

        let mut quotient_value: Option<E::Fr> = None;
        let mut remainder_value: Option<E::Fr> = None;

        if let (Some(nom), Some(denom)) = (nominator.value, denominator.value) {
            let nom_bi = utils::fr_to_bigint::<E>(&nom);
            let denom_bi = utils::fr_to_bigint::<E>(&denom);

            let (q, r) = utils::euclidean_div_rem(&nom_bi, &denom_bi);

            quotient_value = utils::bigint_to_fr::<E>(&q);
            remainder_value = utils::bigint_to_fr::<E>(&r);
        }

        let (quotient, remainder) = {
            let mut cs = self.cs_namespace();

            let qutioent_var = cs
                .alloc(
                    || "qutioent",
                    || quotient_value.ok_or(SynthesisError::AssignmentMissing),
                )
                .map_err(RuntimeError::SynthesisError)?;

            let remainder_var = cs
                .alloc(
                    || "remainder",
                    || remainder_value.ok_or(SynthesisError::AssignmentMissing),
                )
                .map_err(RuntimeError::SynthesisError)?;

            cs.enforce(
                || "equality",
                |lc| lc + qutioent_var,
                |lc| lc + denominator.variable,
                |lc| lc + nominator.variable - remainder_var,
            );

            let quotient = FrPrimitive::new(quotient_value, qutioent_var);
            let remainder = FrPrimitive::new(remainder_value, remainder_var);

            (quotient, remainder)
        };

        let abs_denominator = self.abs(denominator)?;
        let lt = self.lt(remainder.clone(), abs_denominator)?;
        let zero = self.zero()?;
        let ge = self.ge(remainder.clone(), zero)?;
        let mut cs = self.cs_namespace();
        cs.enforce(
            || "0 <= rem < |denominator|",
            |lc| lc + CS::one() - lt.variable,
            |lc| lc + CS::one() - ge.variable,
            |lc| lc,
        );

        Ok((quotient, remainder))
    }

    fn neg(&mut self, element: FrPrimitive<E>) -> Result<FrPrimitive<E>, RuntimeError> {
        let neg_value = match element.value {
            Some(value) => {
                let mut neg = E::Fr::zero();
                neg.sub_assign(&value);
                Some(neg)
            }
            _ => None,
        };

        let mut cs = self.cs_namespace();

        let neg_variable = cs
            .alloc(
                || "neg variable",
                || neg_value.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "neg constraint",
            |lc| lc + element.variable,
            |lc| lc + CS::one(),
            |lc| lc - neg_variable,
        );

        Ok(FrPrimitive::new(neg_value, neg_variable))
    }

    fn not(&mut self, element: FrPrimitive<E>) -> Result<FrPrimitive<E>, RuntimeError> {
        let one = Self::one();
        self.sub(one, element)
    }

    fn and(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let value = match (left.value, right.value) {
            (Some(a), Some(b)) => {
                let mut conj = a;
                conj.mul_assign(&b);
                Some(conj)
            }
            _ => None,
        };

        let variable = cs
            .alloc(|| "and", || value.ok_or(SynthesisError::AssignmentMissing))
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "equality",
            |lc| lc + left.variable,
            |lc| lc + right.variable,
            |lc| lc + variable,
        );

        Ok(FrPrimitive::new(value, variable))
    }

    fn or(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let value = match (left.value, right.value) {
            (Some(a), Some(b)) => {
                if a.is_zero() && b.is_zero() {
                    Some(E::Fr::zero())
                } else {
                    Some(E::Fr::one())
                }
            }
            _ => None,
        };

        let variable = cs
            .alloc(|| "or", || value.ok_or(SynthesisError::AssignmentMissing))
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "equality",
            |lc| lc + CS::one() - left.variable,
            |lc| lc + CS::one() - right.variable,
            |lc| lc + CS::one() - variable,
        );

        Ok(FrPrimitive::new(value, variable))
    }

    fn xor(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let value = match (left.value, right.value) {
            (Some(a), Some(b)) => {
                if a.is_zero() == b.is_zero() {
                    Some(E::Fr::zero())
                } else {
                    Some(E::Fr::one())
                }
            }
            _ => None,
        };

        let variable = cs
            .alloc(
                || "conjunction",
                || value.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        // (a + a) * (b) = (a + b - c)
        cs.enforce(
            || "equality",
            |lc| lc + left.variable + left.variable,
            |lc| lc + right.variable,
            |lc| lc + left.variable + right.variable - variable,
        );

        Ok(FrPrimitive::new(value, variable))
    }

    fn lt(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let one = Self::one();
        let right_minus_one = self.sub(right, one)?;
        self.le(left, right_minus_one)
    }

    fn le(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let diff = self.sub(right, left)?;

        let mut cs = self.cs_namespace();

        let diff_num = AllocatedNum::alloc(cs.namespace(|| "diff_num variable"), || {
            diff.value.ok_or(SynthesisError::AssignmentMissing)
        })
        .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "allocated_num equality",
            |lc| lc + diff.variable,
            |lc| lc + CS::one(),
            |lc| lc + diff_num.get_variable(),
        );

        let bits = diff_num
            .into_bits_le(cs.namespace(|| "diff_num bits"))
            .map_err(RuntimeError::SynthesisError)?;

        let diff_num_repacked = AllocatedNum::pack_bits_to_element(
            cs.namespace(|| "diff_num_repacked"),
            &bits[0..(E::Fr::CAPACITY as usize - 1)],
        )
        .map_err(RuntimeError::SynthesisError)?;

        let lt = AllocatedNum::equals(cs.namespace(|| "equals"), &diff_num, &diff_num_repacked)
            .map_err(RuntimeError::SynthesisError)?;

        Ok(FrPrimitive::new(lt.get_value_field::<E>(), lt.get_variable()))
    }

    fn eq(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let l_num = AllocatedNum::alloc(cs.namespace(|| "l_num"), || {
            left.value.ok_or(SynthesisError::AssignmentMissing)
        })
        .map_err(RuntimeError::SynthesisError)?;

        let r_num = AllocatedNum::alloc(cs.namespace(|| "r_num"), || {
            right.value.ok_or(SynthesisError::AssignmentMissing)
        })
        .map_err(RuntimeError::SynthesisError)?;

        let eq = AllocatedNum::equals(cs, &l_num, &r_num).map_err(RuntimeError::SynthesisError)?;

        Ok(FrPrimitive::new(eq.get_value_field::<E>(), eq.get_variable()))
    }

    fn ne(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let eq = self.eq(left, right)?;
        self.not(eq)
    }

    fn ge(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let not_ge = self.lt(left, right)?;
        self.not(not_ge)
    }

    fn gt(
        &mut self,
        left: FrPrimitive<E>,
        right: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let not_gt = self.le(left, right)?;
        self.not(not_gt)
    }

    fn conditional_select(
        &mut self,
        condition: FrPrimitive<E>,
        if_true: FrPrimitive<E>,
        if_false: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let mut cs = self.cs_namespace();

        let value = match condition.value {
            Some(value) => {
                if !value.is_zero() {
                    if_true.value
                } else {
                    if_false.value
                }
            }
            None => None,
        };

        let variable = cs
            .alloc(
                || "variable",
                || value.ok_or(SynthesisError::AssignmentMissing),
            )
            .map_err(RuntimeError::SynthesisError)?;

        // Selected, Right, Left, Condition
        // s = r + c * (l - r)
        // (l - r) * (c) = (s - r)
        cs.enforce(
            || "constraint",
            |lc| lc + if_true.variable - if_false.variable,
            |lc| lc + condition.variable,
            |lc| lc + variable - if_false.variable,
        );

        Ok(FrPrimitive::new(value, variable))
    }

    fn assert(&mut self, element: FrPrimitive<E>) -> Result<(), RuntimeError> {
        let inverse_value = element.value
            .map(|fr| {
                fr
                    .inverse()
                    .unwrap_or_else(E::Fr::zero)
            });

        let mut cs = self.cs_namespace();
        let inverse_variable = cs
            .alloc(|| "inverse", || inverse_value.ok_or(SynthesisError::AssignmentMissing))
            .map_err(RuntimeError::SynthesisError)?;

        cs.enforce(
            || "assertion",
            |lc| lc + element.variable,
            |lc| lc + inverse_variable,
            |lc| lc + CS::one(),
        );

        Ok(())
    }

    fn array_get(
        &mut self,
        array: &[FrPrimitive<E>],
        index: FrPrimitive<E>,
    ) -> Result<FrPrimitive<E>, RuntimeError> {
        let bits = self.bits(index, array.len())?;

        self.recursive_select(array, bits.as_slice())
    }

    fn array_set(
        &mut self,
        array: &[FrPrimitive<E>],
        index: FrPrimitive<E>,
        value: FrPrimitive<E>,
    ) -> Result<Vec<FrPrimitive<E>>, RuntimeError> {
        let mut new_array = Vec::new();

        for (i, p) in array.iter().enumerate() {
            let curr_index = self.constant_bigint(&i.into())?;

            let cond = self.eq(curr_index, index.clone())?;
            let value = self.conditional_select(cond, value.clone(), p.clone())?;
            new_array.push(value);
        }

        Ok(new_array)
    }
}
