//! Shared ladder drain for QEMU and physical UART streams. The stream owns time and I/O.
use crate::ladder::{self, Config};
use crate::verdict::args;
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};
pub trait Stream {
    fn now(&mut self) -> Result<f64>;
    fn read_serial(&mut self, timeout: f64) -> Result<String>;
}
fn finite(n: f64, name: &str) -> Result<f64> {
    if n.is_finite() {
        Ok(n)
    } else {
        Err(Error::new(format!("ladder {name} must be finite")))
    }
}
fn now(stream: &mut dyn Stream) -> Result<f64> {
    finite(stream.now()?, "clock")
}
fn outstanding(config: &Config, text: &str, staged: &[String]) -> Vec<String> {
    let mut out = ladder::optional_in_flight(config, text);
    for guest in staged {
        if !text.contains(&format!("{guest}.exe exit = ")) && !out.contains(guest) {
            out.push(guest.clone())
        }
    }
    out
}
/// Returns newly consumed text only. `seen` remains the caller's owned earlier serial.
pub fn watch(
    stream: &mut dyn Stream,
    config: &Config,
    seconds: f64,
    seen: &str,
    staged: &[String],
    program_done_expected: bool,
    floor: f64,
    optional_extra: f64,
    winetest_extra: f64,
) -> Result<String> {
    finite(seconds, "seconds")?;
    finite(floor, "drain floor")?;
    finite(optional_extra, "optional extension")?;
    finite(winetest_extra, "winetest extension")?;
    if floor < 0. || optional_extra < 0. || winetest_extra < 0. {
        return Err("ladder floor and extensions must not be negative".into());
    }
    let need: Vec<_> = ladder::markers(config)
        .into_iter()
        .filter(|marker| !seen.contains(marker))
        .collect();
    if floor == 0.
        && ((program_done_expected && seen.contains(ladder::PROGRAM_DONE))
            || (!program_done_expected
                && need.is_empty()
                && outstanding(config, seen, staged).is_empty()
                && ladder::winetests_lost(seen)?.is_empty()
                && ladder::sweeps_unfinished(seen).is_empty()))
    {
        return Ok(String::new());
    }
    let start = now(stream)?;
    let mut deadline = finite(start + seconds.max(floor), "deadline")?;
    let mut extended = BTreeSet::new();
    let mut out = String::new();
    while now(stream)? < deadline {
        out.push_str(&stream.read_serial(0.5)?);
        let text = format!("{seen}{out}");
        let pending = outstanding(config, &text, staged);
        let running = ladder::winetests_lost(&text)?;
        let sweeping = ladder::sweeps_unfinished(&text);
        let finished = if program_done_expected {
            text.contains(ladder::PROGRAM_DONE)
        } else {
            need.iter().all(|marker| out.contains(marker))
                && pending.is_empty()
                && running.is_empty()
                && sweeping.is_empty()
        };
        if finished && now(stream)? - start >= floor {
            break;
        }
        for suite in sweeping {
            if extended.insert(format!("{suite}:--list")) {
                deadline = finite(deadline + winetest_extra, "deadline")?
            }
        }
        for guest in pending {
            if extended.insert(guest) {
                deadline = finite(deadline + optional_extra, "deadline")?
            }
        }
        for label in running {
            if extended.insert(label) {
                deadline = finite(deadline + winetest_extra, "deadline")?
            }
        }
    }
    Ok(out)
}
struct CallbackStream<'a> {
    host: &'a mut dyn Host,
}
impl Stream for CallbackStream<'_> {
    fn now(&mut self) -> Result<f64> {
        self.host
            .call("ladder_clock", json!({}))?
            .as_f64()
            .ok_or_else(|| Error::new("ladder clock callback did not return seconds"))
    }
    fn read_serial(&mut self, timeout: f64) -> Result<String> {
        let value = self
            .host
            .call("ladder_read_serial", json!({"timeout":timeout}))?;
        if value.is_null() {
            return Ok(String::new());
        }
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::new("ladder serial callback did not return text"))
    }
}
pub fn dispatch(_root: &Path, request: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = args(request);
    if request["op"].as_str() != Some("ladder_watch") {
        return Err("unknown live ladder operation".into());
    }
    let config: Config = a
        .get("config")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let staged: Vec<String> = a
        .get("optional_staged")
        .filter(|v| !v.is_null())
        .cloned()
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let floor = ladder::drain_floor(a["drain_floor_raw"].as_str())?;
    let mut stream = CallbackStream { host };
    Ok(json!(watch(
        &mut stream,
        &config,
        a["seconds"].as_f64().unwrap_or(ladder::TIMEOUT_SECONDS),
        a["seen"].as_str().unwrap_or(""),
        &staged,
        a["program_done_expected"].as_bool().unwrap_or(false),
        floor,
        a["optional_extra_seconds"]
            .as_f64()
            .unwrap_or(ladder::OPTIONAL_EXTRA_SECONDS),
        a["winetest_extra_seconds"]
            .as_f64()
            .unwrap_or(ladder::WINETEST_EXTRA_SECONDS)
    )?))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    struct Script {
        time: f64,
        step: f64,
        chunks: VecDeque<String>,
        reads: usize,
    }
    impl Stream for Script {
        fn now(&mut self) -> Result<f64> {
            self.time += self.step;
            Ok(self.time)
        }
        fn read_serial(&mut self, _timeout: f64) -> Result<String> {
            self.reads += 1;
            Ok(self.chunks.pop_front().unwrap_or_default())
        }
    }
    fn all(c: &Config) -> String {
        c.ladder
            .iter()
            .map(|(g, _)| format!("[WINLOGON] {g}.exe exit = 0x600d\n"))
            .collect()
    }
    fn script(chunks: Vec<String>) -> Script {
        Script {
            time: 0.,
            step: 0.01,
            chunks: chunks.into(),
            reads: 0,
        }
    }
    #[test]
    fn earlier_verdicts_and_staged_unlaunched_guests_are_honored() {
        let c = Config::default();
        let mut s = script(vec![]);
        assert_eq!(
            watch(&mut s, &c, 5., &all(&c), &[], false, 0., 1., 1.).unwrap(),
            ""
        );
        assert_eq!(s.reads, 0);
        let mut s = script(vec![
            all(&c),
            "[WINLOGON] lua32.exe (x) created\n".into(),
            "[WINLOGON] lua32.exe exit = 0x600d\n".into(),
        ]);
        let out = watch(&mut s, &c, 5., "", &["lua32".into()], false, 0., 1., 1.).unwrap();
        assert_eq!(s.reads, 3);
        assert!(out.contains("lua32.exe exit = "));
    }
    #[test]
    fn extension_is_additive_once_per_guest_subtest_and_enumeration() {
        let c = Config::default();
        let text=format!("{}[WINLOGON] lua32.exe (x)\n[WINLOGON] sqlite32.exe (x)\n[WT] user32:--list: started (x)\n[WT] user32:first: started (x)\n[WT] user32:second: started (x)\n",all(&c));
        let mut s = script(vec![text]);
        watch(&mut s, &c, 0.05, "", &[], false, 0., 0.25, 0.2).unwrap();
        assert!(s.time >= 1.15, "0.05 + 2*0.25 + 3*0.2; observed {}", s.time);
        assert!(s.time < 1.25, "a label must not earn a second extension");
    }
    #[test]
    fn floor_holds_an_already_complete_drain_open() {
        let c = Config::default();
        let mut s = script(vec![]);
        watch(&mut s, &c, 0.01, &all(&c), &[], false, 0.4, 1., 1.).unwrap();
        assert!(s.time >= 0.4);
        assert!(s.reads > 0);
    }
    #[test]
    fn program_done_is_authoritative_and_new_text_is_kept() {
        let c = Config::default();
        let mut s = script(vec![
            "partial\n".into(),
            format!("{}\n", ladder::PROGRAM_DONE),
        ]);
        let out = watch(&mut s, &c, 5., "old\n", &[], true, 0., 1., 1.).unwrap();
        assert_eq!(s.reads, 2);
        assert!(out.starts_with("partial\n"));
        assert!(!out.contains("old"));
        assert!(out.ends_with(&format!("{}\n", ladder::PROGRAM_DONE)));
    }
}
