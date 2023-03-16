use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    halo2curves::bn256::Fr,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};

#[derive(Clone)]
struct FibConfig<F> {
    q_enable: Selector,
    acc: Column<Advice>,
    _marker: PhantomData<F>,
}

impl<F: Field> FibConfig<F> {
    fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let acc = meta.advice_column();
        let q_enable = meta.selector();

        meta.create_gate("fib", |meta| {
            let q_enable = meta.query_selector(q_enable);
            let acc_prepre = meta.query_advice(acc, Rotation(-2));
            let acc_pre = meta.query_advice(acc, Rotation::prev());
            let acc_cur = meta.query_advice(acc, Rotation::cur());
            [(
                "acc_cur === acc_pre + acc_prepre",
                q_enable * (acc_cur - acc_pre - acc_prepre),
            )]
        });

        Self {
            q_enable,
            acc,
            _marker: PhantomData,
        }
    }
    fn assign(&self, layouter: &mut impl Layouter<F>, rounds: usize) -> Result<(), Error> {
        layouter.assign_region(
            || "fib",
            |mut region| {
                let mut value_prepre = F::one();
                region.assign_advice(|| "0 row", self.acc, 0, || Value::known(value_prepre))?;
                let mut value_pre = F::one();
                region.assign_advice(|| "1 row", self.acc, 1, || Value::known(value_pre))?;

                for round in 0..rounds {
                    let offset = round + 2;
                    let value = value_pre + value_prepre;
                    region.assign_advice(|| "1 row", self.acc, offset, || Value::known(value))?;
                    self.q_enable.enable(&mut region, offset)?;
                    value_prepre = value_pre;
                    value_pre = value;
                }

                Ok(())
            },
        )
    }
}

#[derive(Default)]
struct FibCircuit<F: Field> {
    _marker: PhantomData<F>,
}

impl<F: Field> Circuit<F> for FibCircuit<F> {
    type Config = FibConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let config = FibConfig::configure(meta);
        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.assign(&mut layouter, 5)
    }
}

fn main() {
    println!("running circuit");
    let circuit = FibCircuit::default();
    let k = 4;
    let prover = MockProver::<Fr>::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied_par();
}
