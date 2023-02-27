use assembly::Assembler;
use criterion::{criterion_group, criterion_main, Criterion};
use miden::{ProofOptions};
use std::time::Duration;
use stdlib::StdLibrary;
use vm_core::{Program, ProgramInputs};
use log::{debug, error, info, logger, LevelFilter};
use math::FieldElement;
use processor::math::Felt;


pub fn get_example(n: usize) -> Example<MemAdviceProvider> {
    // generate the program and expected results
    let program = generate_fibonacci_program(n);
    let expected_result = vec![compute_fibonacci(n).as_int()];
    println!(
        "Generated a program to compute {}-th Fibonacci term; expected result: {}",
        n, expected_result[0]
    );

    Example {
        program,
        stack_inputs: StackInputs::try_from_values([0, 1]).unwrap(),
        advice_provider: MemAdviceProvider::empty(),
        expected_result,
        num_outputs: 1,
    }
}

/// Generates a program to compute the `n`-th term of Fibonacci sequence
fn generate_fibonacci_program(n: usize) -> Program {
    // the program is a simple repetition of 4 stack operations:
    // the first operation moves the 2nd stack item to the top,
    // the second operation duplicates the top 2 stack items,
    // the third operation removes the top item from the stack
    // the last operation pops top 2 stack items, adds them, and pushes
    // the result back onto the stack
    let program = format!(
        "begin
            repeat.{}
                swap dup.1 add
            end
        end",
        n - 1
    );

    Assembler::default().compile(&program).unwrap()
}

/// Computes the `n`-th term of Fibonacci sequence
fn compute_fibonacci(n: usize) -> Felt {
    let mut t0 = Felt::ZERO;
    let mut t1 = Felt::ONE;

    for _ in 0..n {
        t1 = t0 + t1;
        core::mem::swap(&mut t0, &mut t1);
    }
    t0
}


fn program_prover(c: &mut Criterion) {
    // env_logger::Builder::new()
    //         .format(|buf, record| writeln!(buf, "{}", record.args()))
    //         .filter_level(log::LevelFilter::Debug)
    //         .init();

    let mut group = c.benchmark_group("program_prover");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    group.bench_function("program_prover", |bench| {
        bench.iter(|| {
            let program = format!(
                "begin
                    repeat.{}
                        swap dup.1 add
                    end
                end",
                1 << 16
            );

            let program = Assembler::
                .with_module_provider(StdLibrary::default())
                .compile(&program)
                .unwrap();

            let inputs = ProgramInputs::from_stack_inputs(&[0, 1]).unwrap();

            let (mut outputs, proof) = miden::prove(&program, &inputs, &ProofOptions::default()).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, program_prover);
criterion_main!(benches);
