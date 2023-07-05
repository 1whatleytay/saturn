use std::panic;
use std::panic::UnwindSafe;
use serde::Serialize;

#[derive(Serialize)]
pub enum TestResult {
    Unset,
    Failed,
    Passed
}

#[derive(Serialize)]
pub struct TestItem {
    name: String,
    result: TestResult
}

mod tests {
    use std::time::Duration;
    use titan::unit::device::{RegisterName, UnitDevice};
    use titan::unit::device::RegisterName::V0;
    use titan::unit::device::StopCondition::Label;

    fn check_saved_registers<F: FnMut()>(device: &UnitDevice, mut f: F) {
        let initial = device.registers();

        f();

        let finished = device.registers();

        assert_eq!(initial.get(RegisterName::S0), finished.get(RegisterName::S0));
        assert_eq!(initial.get(RegisterName::S1), finished.get(RegisterName::S1));
        assert_eq!(initial.get(RegisterName::S2), finished.get(RegisterName::S2));
        assert_eq!(initial.get(RegisterName::S3), finished.get(RegisterName::S3));
        assert_eq!(initial.get(RegisterName::S4), finished.get(RegisterName::S4));
        assert_eq!(initial.get(RegisterName::S5), finished.get(RegisterName::S5));
        assert_eq!(initial.get(RegisterName::S6), finished.get(RegisterName::S6));
        assert_eq!(initial.get(RegisterName::S7), finished.get(RegisterName::S7));
    }

    pub fn zero_fib(path: &str) {
        let device = UnitDevice::make(path.into()).unwrap();

        device.call("fib", [0], Some(Duration::from_secs(2))).unwrap();

        assert_eq!(device.get(V0), 0)
    }

    pub fn one_fib(path: &str) {
        let device = UnitDevice::make(path.into()).unwrap();

        device.call("fib", [1], Some(Duration::from_secs(2))).unwrap();

        assert_eq!(device.get(V0), 1)
    }

    pub fn four_fib(path: &str) {
        let device = UnitDevice::make(path.into()).unwrap();

        device.execute_until([Label("fib".into())]).unwrap();

        device.call("fib", [4], Some(Duration::from_secs(2))).unwrap();

        assert_eq!(device.get(V0), 3)
    }

    pub fn ten_fib(path: &str) {
        let device = UnitDevice::make(path.into()).unwrap();

        device.call("fib", [10], Some(Duration::from_secs(2))).unwrap();

        assert_eq!(device.get(V0), 55)
    }

    pub fn preserves_saved(path: &str) {
        let device = UnitDevice::make(path.into()).unwrap();

        check_saved_registers(&device, || {
            device.call("fib", [4], Some(Duration::from_secs(2))).unwrap();
        })
    }
}

#[tauri::command]
pub fn all_tests() -> Vec<TestItem> {
    vec![
        // TestItem { name: "0th Fib Number".to_string(), result: TestResult::Unset },
        // TestItem { name: "1th Fib Number".to_string(), result: TestResult::Unset },
        // TestItem { name: "4th Fib Number".to_string(), result: TestResult::Unset },
        // TestItem { name: "10th Fib Number".to_string(), result: TestResult::Unset },
        // TestItem { name: "Preserves Saved Registers".to_string(), result: TestResult::Unset },
    ]
}

fn run_test<F: FnOnce () + UnwindSafe>(f: F) -> TestResult {
    match panic::catch_unwind(f) {
        Ok(_) => TestResult::Passed,
        Err(_) => TestResult::Failed
    }
}

#[tauri::command]
pub fn run_tests(path: &str) -> Vec<TestItem> {
    vec![
        // TestItem { name: "0th Fib Number".to_string(), result: run_test(|| tests::zero_fib(path)) },
        // TestItem { name: "1th Fib Number".to_string(), result: run_test(|| tests::one_fib(path)) },
        // TestItem { name: "4th Fib Number".to_string(), result: run_test(|| tests::four_fib(path)) },
        // TestItem { name: "10th Fib Number".to_string(), result: run_test(|| tests::ten_fib(path)) },
        // TestItem { name: "Preserves Saved Registers".to_string(), result: run_test(|| tests::preserves_saved(path)) },
    ]
}
