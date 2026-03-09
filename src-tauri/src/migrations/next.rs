use serde_json::Value;

type PendingStep = fn(Value) -> Result<Value, String>;

const PENDING_STEPS: &[PendingStep] = &[
    // Add pending migration steps here, in order.
];

pub fn migrate(mut data: Value) -> Result<Value, String> {
    for step in PENDING_STEPS {
        data = step(data)?;
    }

    Ok(data)
}
