/// Counter but only for even numbers
use expose_macro::expose;
use wasmlanche_sdk::program::{Program, Value};
use wasmlanche_sdk::store::ProgramContext;
use wasmlanche_sdk::types::Address;

#[expose]
fn init_program() -> i64 {
    let even_program = Program::new();
    even_program.publish().unwrap().into()
}

#[expose]
fn set(ctx: ProgramContext, counter_ctx: ProgramContext) {
    ctx.store_value("counter", &Value::ProgramObject(counter_ctx))
        .expect("Failed to store token contract address");
}

/// Calls the counter program to increment by twice the amount.
#[expose]
fn inc(ctx: ProgramContext, whose: Address, amt: i64) {
    let call_ctx = match ctx.get_value("counter") {
        Ok(value) => ProgramContext::from(value),
        Err(_) => {
            // Can return error here, up to smart contract designer. Skipping for now.
            return;
        }
    };
    let _ = ctx.invoke_program_method(
        &call_ctx,
        "inc",
        &[Value::from(whose), Value::IntObject(amt * 2)],
    );
}

/// Returns the value of whose's counter from the counter program.
#[expose]
fn value(ctx: ProgramContext, whose: Address) -> i64 {
    let call_ctx = match ctx.get_value("counter") {
        Ok(value) => ProgramContext::from(value),
        Err(_) => {
            // Can return error here, up to smart contract designer. Skipping for now.
            return 0;
        }
    };

    let result = ctx.invoke_program_method(&call_ctx, "value", &[Value::from(whose)]);
    i64::from(result)
}
