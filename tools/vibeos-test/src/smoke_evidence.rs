//! Prefer the VM's saved deadline witness to a later live sample.
use crate::callback::Host;
use serde_json::{json, Value};
pub fn cpu_state_at_deadline(host: &mut dyn Host, id: u64) -> Option<String> {
    let nonempty = |v: Value| v.as_str().filter(|s| !s.is_empty()).map(str::to_owned);
    if let Some(saved) = host
        .call("vm_get_hang_cpu_state", json!({"id":id}))
        .ok()
        .and_then(nonempty)
    {
        return Some(saved);
    }
    host.call("vm_capture_cpu_state", json!({"id":id}))
        .ok()
        .and_then(nonempty)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    struct Capture {
        stored: &'static str,
        live: &'static str,
        calls: Vec<String>,
    }
    impl Host for Capture {
        fn call(&mut self, op: &str, _a: Value) -> Result<Value> {
            self.calls.push(op.into());
            Ok(json!(match op {
                "vm_get_hang_cpu_state" => self.stored,
                "vm_capture_cpu_state" => self.live,
                _ => return Err("unexpected evidence callback".into()),
            }))
        }
    }
    #[test]
    fn saved_deadline_state_wins_without_resampling() {
        let mut s = Capture {
            stored: "CPU#0\nPC=ffff800004001234\n",
            live: "CPU#0\nPC=ffff8000040567ac\n",
            calls: vec![],
        };
        assert!(cpu_state_at_deadline(&mut s, 1)
            .unwrap()
            .contains("ffff800004001234"));
        assert_eq!(s.calls, vec!["vm_get_hang_cpu_state"]);
    }
    #[test]
    fn empty_saved_state_falls_back_to_a_live_capture() {
        let mut s = Capture {
            stored: "",
            live: "CPU#0\nPC=ffff8000040567ac\n",
            calls: vec![],
        };
        assert!(cpu_state_at_deadline(&mut s, 1)
            .unwrap()
            .contains("ffff8000040567ac"));
        assert_eq!(
            s.calls,
            vec!["vm_get_hang_cpu_state", "vm_capture_cpu_state"]
        );
    }
}
