//! Gate composition. The transport owns external task handles; native code owns all
//! dependencies, retries, verdict aggregation and permission to publish a stamp.
use crate::{callback::Host, Error, Result};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v[k].as_str().unwrap_or(d)
}
pub fn arches(raw: &str) -> Result<Vec<String>> {
    let mut out = vec![];
    for a in raw.split(',').map(str::trim).filter(|a| !a.is_empty()) {
        let a = match a {
            "amd64-uefi" | "amd64-bios" => "amd64",
            "i386-uefi" => "i386",
            a => a,
        };
        vibeos_build::arch(a)?;
        if !out.iter().any(|s| s == a) {
            out.push(a.to_string())
        }
    }
    if out.is_empty() {
        out = vibeos_build::ARCHES.iter().map(|s| s.to_string()).collect()
    }
    Ok(out)
}
pub const RETRYABLE: &[&str] = &[
    "KNOWN_FLAKE",
    "FLAKY",
    "LAUNCH_FAIL",
    "NET_FLAKE",
    "HOST_STARVED",
    "CONFORMANCE_RED",
];
pub fn retryable(cls: &str, summary: &Value) -> bool {
    RETRYABLE.contains(&cls)
        || (cls == "HANG_TIMEOUT"
            && summary["hang_kind"] == "slow"
            && summary["wedge_confirmed"] != true)
}
#[derive(Clone)]
struct Task {
    kind: String,
    lane: String,
    attempt: usize,
}
struct Gate<'a> {
    host: &'a mut dyn Host,
    args: Value,
    root: String,
    pending: BTreeMap<u64, Task>,
    class: String,
    failed: bool,
    stages: Vec<Value>,
    results: BTreeMap<String, Value>,
    built: BTreeSet<String>,
    ladder: BTreeMap<String, Value>,
    lint: BTreeMap<String, bool>,
    cohab: BTreeMap<String, bool>,
    tooling: bool,
    difftest: bool,
    do_lint: bool,
    do_build: bool,
    net: bool,
    conf: bool,
    stor: bool,
    diff_started: bool,
    dependencies: BTreeSet<String>,
    ready: BTreeSet<String>,
}
impl Gate<'_> {
    fn stage(
        &mut self,
        name: &str,
        status: &str,
        detail: String,
        code: i64,
        seconds: f64,
        salient: Vec<String>,
    ) -> Result<()> {
        let st = json!({"name":name,"status":status,"detail":detail,"exit":code,"seconds":seconds,"salient":salient});
        if status == "fail" {
            self.failed = true;
        }
        self.host.call("stage", st.clone())?;
        self.stages.push(st);
        Ok(())
    }
    fn classification(&mut self, c: &str) -> Result<()> {
        if self.class.is_empty() && !c.is_empty() {
            self.class = c.to_owned();
            self.host.call("set_run", json!({"classification":c}))?;
        }
        Ok(())
    }
    fn submit(&mut self, kind: &str, lane: &str, attempt: usize, extra: Value) -> Result<()> {
        let folder = if attempt > 1 {
            format!("smoke-{}-r{}", lane.replace(':', "-"), attempt - 1)
        } else {
            match kind {
                "lane" => {
                    if lane.starts_with("conf:") || lane.starts_with("stor:") {
                        format!("{}-a1", lane.replace(':', "-"))
                    } else {
                        format!("smoke-{lane}-a1")
                    }
                }
                "lint" => format!("lint-{lane}"),
                "cohab" => format!("cohab-{lane}"),
                other => other.into(),
            }
        };
        let extra = if matches!(kind, "lane" | "retry")
            && lane != "net"
            && !lane.starts_with("conf:")
            && !lane.starts_with("stor:")
        {
            let mut flags = vec![json!("--no-net"), json!("--smp"), json!("2")];
            flags.extend(extra.as_array().cloned().unwrap_or_default());
            json!(flags)
        } else {
            extra
        };
        let id=self.host.call("gate_task_submit",json!({"kind":kind,"lane":lane,"root":format!("{}/{folder}",self.root),"args":self.args,"extra":extra}))?.as_u64().ok_or("invalid gate task token")?;
        self.pending.insert(
            id,
            Task {
                kind: kind.into(),
                lane: lane.into(),
                attempt,
            },
        );
        Ok(())
    }
    fn launch(&mut self, arch: &str) -> Result<()> {
        self.ready.insert(arch.into());
        if self.do_lint && !self.diff_started && self.dependencies.is_subset(&self.ready) {
            self.submit("difftest", "", 1, Value::Null)?;
            self.diff_started = true;
        }
        let (extra, staged, why) = if let Some(v) = self.ladder.get(arch) {
            (v.clone(), None, String::new())
        } else {
            let d = self.host.call("gate_ladder", json!({"lane":arch}))?;
            let v = d[0].clone();
            let staged = d[1].as_bool().unwrap_or(false);
            let why = d[2].as_str().unwrap_or("").into();
            self.ladder.insert(arch.into(), v.clone());
            (v, Some(staged), why)
        };
        if let Some(staged) = staged {
            self.stage(
                &format!("ladder:{arch}"),
                if staged { "ok" } else { "skip" },
                why,
                0,
                0.,
                vec![],
            )?;
        }
        self.submit("lane", arch, 1, extra)?;
        if arch == "amd64" {
            if self.net {
                self.submit("lane", "net", 1, json!([]))?;
            }
            self.submit("lane", "amd64-bios", 1, json!([]))?;
        }
        if self.conf {
            self.submit("lane", &format!("conf:{arch}"), 1, json!([]))?;
        }
        if self.stor {
            self.submit("lane", &format!("stor:{arch}"), 1, json!([]))?;
        }
        if self.do_lint {
            self.submit("lint", arch, 1, Value::Null)?;
            if self.do_build {
                self.submit("cohab", arch, 1, Value::Null)?;
            }
        }
        Ok(())
    }
    fn collect_check(&mut self, t: &Task, r: &Value) -> Result<()> {
        let ok = r["ok"] == true;
        let label = &t.kind;
        let cls = s(
            r,
            "classification",
            match label.as_str() {
                "lint" => "LINT_FAIL",
                "cohab" => "COHAB_FAIL",
                "tooling" => "TOOLING_FAIL",
                _ => "DIFFTEST_FAIL",
            },
        );
        let name = if matches!(label.as_str(), "lint" | "cohab") {
            format!("{label}:{}", t.lane)
        } else {
            label.clone()
        };
        let detail = if ok {
            match label.as_str() {
                "lint" => "check --strict clean",
                "cohab" => "flipped modules fully covered",
                "tooling" => "check-tooling clean",
                _ => "difftest clean",
            }
            .into()
        } else {
            format!("{label} failed (exit {})", r["exit"])
        };
        self.stage(
            &name,
            if ok { "ok" } else { "fail" },
            detail,
            r["exit"].as_i64().unwrap_or(0),
            r["seconds"].as_f64().unwrap_or(0.),
            if ok {
                vec![]
            } else {
                vec![s(r, "salient", &format!("{label} failed")).into()]
            },
        )?;
        match label.as_str() {
            "lint" => {
                self.lint.insert(t.lane.clone(), ok);
            }
            "cohab" => {
                self.cohab.insert(t.lane.clone(), ok);
            }
            "tooling" => self.tooling = ok,
            "difftest" => self.difftest = ok,
            _ => {}
        }
        if !ok {
            if cls == "SCANNER_ERROR" {
                self.class = cls.into();
                self.host.call("set_run", json!({"classification":cls}))?;
            } else {
                self.classification(if cls.is_empty() {
                    match label.as_str() {
                        "lint" => "LINT_FAIL",
                        "cohab" => "COHAB_FAIL",
                        "tooling" => "TOOLING_FAIL",
                        _ => "DIFFTEST_FAIL",
                    }
                } else {
                    cls
                })?;
            }
        }
        Ok(())
    }
    fn build_stage(&mut self, arch: &str, r: &Value, detail: &str, seconds: f64) -> Result<()> {
        let ok = r["ok"] == true;
        let cls = s(r, "classification", "");
        self.stage(
            &format!("build:{arch}"),
            if ok { "ok" } else { "fail" },
            if ok {
                detail.into()
            } else {
                format!("{detail} ({cls})")
            },
            r["exit"].as_i64().unwrap_or(0),
            seconds,
            if ok {
                vec![]
            } else {
                vec![format!(
                    "vibeos-cli build --arch {arch} exited {}",
                    r["exit"]
                )]
            },
        )?;
        if ok {
            self.built.insert(arch.into());
        } else {
            self.classification(cls)?;
        }
        Ok(())
    }
    fn poll(&mut self) -> Result<Vec<(Task, Value)>> {
        let ids = self.pending.keys().copied().collect::<Vec<_>>();
        let done = self.host.call("gate_task_poll", json!({"tokens":ids}))?;
        let mut out = vec![];
        for d in done.as_array().ok_or("invalid completed gate tasks")? {
            let id = d["token"].as_u64().ok_or("invalid completed task id")?;
            let task = self
                .pending
                .remove(&id)
                .ok_or("unknown completed gate task")?;
            if let Some(error) = d.get("error") {
                return Err(Error::new(format!(
                    "gate task {}:{} failed: {error}",
                    task.kind, task.lane
                )));
            }
            out.push((task, d["result"].clone()))
        }
        if out.is_empty() {
            return Err("gate task poll returned no completion".into());
        }
        Ok(out)
    }
}
pub fn run(_repo: &Path, a: &Value, host: &mut dyn Host) -> Result<Value> {
    let args = a["args"].clone();
    let requested = arches(s(&args, "arches", ""))?;
    let do_build = args["no_build"] != true;
    let do_lint = args["no_lint"] != true;
    let retries = args["retries"].as_i64().unwrap_or(2).max(0) as usize;
    let mut g = Gate {
        host,
        args: args.clone(),
        root: s(a, "run_dir", "").into(),
        pending: BTreeMap::new(),
        class: String::new(),
        failed: false,
        stages: vec![],
        results: BTreeMap::new(),
        built: BTreeSet::new(),
        ladder: BTreeMap::new(),
        lint: BTreeMap::new(),
        cohab: BTreeMap::new(),
        tooling: true,
        difftest: true,
        do_build,
        do_lint,
        net: args["no_net"] != true,
        conf: args["no_conformance"] != true,
        stor: args["no_stortest"] != true,
        diff_started: false,
        dependencies: requested
            .iter()
            .filter(|s| matches!(s.as_str(), "amd64" | "i386"))
            .cloned()
            .collect(),
        ready: BTreeSet::new(),
    };
    if (args["artifact_store_verify"] == true && args["artifact_store"].is_null())
        || (!args["artifact_store"].is_null() && !do_build)
    {
        g.classification("ARTIFACT_STORE_FAIL")?;
        g.stage(
            "artifact-store",
            "fail",
            "store verification requires --artifact-store; store gating requires a build".into(),
            0,
            0.,
            vec![],
        )?;
        return g.host.call("finalize", json!({}));
    }
    if g.host.call("gate_preflight", json!({"arches":requested}))? != true {
        return g.host.call("finalize", json!({}));
    }
    let gated = g.host.call("gate_tree_state", json!({}))?;
    if g.conf {
        let provision = g.host.call("gate_kmtest", json!({"arches":requested}))?;
        if provision[0] != true {
            g.classification("CONFORMANCE_UNAVAILABLE")?;
            g.stage(
                "conformance",
                "fail",
                "requested ratchet could not be provisioned".into(),
                1,
                0.,
                vec![provision[1]
                    .as_str()
                    .unwrap_or("unknown provisioning failure")
                    .to_owned()],
            )?;
            return g.host.call("finalize", json!({}));
        }
    }
    let mut order = requested.clone();
    order.sort_by_key(|s| s != "amd64");
    let tooling_waits =
        do_lint && do_build && g.host.call("gate_tooling_waits", json!({"arches":order}))? == true;
    if do_lint {
        g.submit("lint", "source-early", 1, Value::Null)?;
        if !tooling_waits {
            g.submit("tooling", "", 1, Value::Null)?;
        }
    }
    let defer =
        do_build && requested.iter().any(|s| s == "amd64") && requested.iter().any(|s| s == "i386");
    let mut amd = None;
    let mut i386 = None;
    let mut restage_submitted = false;
    let mut builds_left = if do_build { order.len() } else { 0 };
    let mut deferred_started = false;
    if do_build {
        for arch in &order {
            g.submit(
                "build",
                arch,
                1,
                json!({"defer_wow64":defer&&arch=="amd64"}),
            )?;
        }
    } else {
        for arch in &order {
            g.built.insert(arch.clone());
            g.launch(arch)?;
        }
    }
    loop {
        if builds_left == 0 && !deferred_started {
            deferred_started = true;
            if do_lint {
                g.submit("lint", "source-deferred", 1, Value::Null)?;
                if tooling_waits {
                    g.submit("tooling", "", 1, Value::Null)?;
                }
            }
        }
        if g.pending.is_empty() {
            break;
        }
        for (t, r) in g.poll()? {
            match t.kind.as_str() {
                "build" => {
                    builds_left -= 1;
                    if defer && t.lane == "i386" {
                        i386 = Some(r.clone());
                    }
                    if r["ok"] != true {
                        g.build_stage(
                            &t.lane,
                            &r,
                            "build failed",
                            r["seconds"].as_f64().unwrap_or(0.),
                        )?;
                    } else if defer && t.lane == "amd64" {
                        amd = Some(r.clone());
                    } else {
                        g.build_stage(&t.lane, &r, "built", r["seconds"].as_f64().unwrap_or(0.))?;
                        g.launch(&t.lane)?;
                    }
                    if !restage_submitted {
                        if let (Some(amd), Some(i386)) = (&amd, &i386) {
                            restage_submitted = true;
                            if i386["ok"] == true {
                                builds_left += 1;
                                g.submit("restage", "amd64", 1, json!({"restage":true}))?;
                            } else {
                                g.build_stage(
                                    "amd64",
                                    amd,
                                    "built (WoW64 NOT staged: i386 failed)",
                                    amd["seconds"].as_f64().unwrap_or(0.),
                                )?;
                                g.launch("amd64")?;
                            }
                        }
                    }
                }
                "restage" => {
                    builds_left -= 1;
                    let seconds = r["seconds"].as_f64().unwrap_or(0.)
                        + amd
                            .as_ref()
                            .and_then(|r| r["seconds"].as_f64())
                            .unwrap_or(0.);
                    g.build_stage(
                        "amd64",
                        &r,
                        if r["ok"] == true {
                            "built + WoW64 restaged"
                        } else {
                            "WoW64 restage failed"
                        },
                        seconds,
                    )?;
                    if r["ok"] == true {
                        g.launch("amd64")?;
                    }
                }
                "lane" => {
                    let mut r = r;
                    r["arch"] = json!(t.lane);
                    r["attempts"] = json!(t.attempt);
                    g.results.insert(t.lane, r);
                }
                _ => g.collect_check(&t, &r)?,
            }
        }
    }
    // Retrying lanes remain parallel; each lane has only one attempt alive at a time.
    let lanes = g.results.keys().cloned().collect::<Vec<_>>();
    for lane in &lanes {
        let r = &g.results[lane];
        if retries > 0 && r["ok"] != true && r["retryable"] == true {
            let extra = g.ladder.get(lane).cloned().unwrap_or(json!([]));
            g.submit("retry", lane, 2, extra)?;
        }
    }
    while !g.pending.is_empty() {
        for (t, mut r) in g.poll()? {
            let previous = g
                .results
                .get_mut(&t.lane)
                .ok_or("retry without first attempt")?;
            previous["attempts"] = json!(t.attempt);
            if r["ok"] == true {
                r["attempts"] = json!(t.attempt);
                r["arch"] = json!(t.lane);
                r["recovered"] = json!(true);
                r["first_classification"] = previous["classification"].clone();
                g.results.insert(t.lane.clone(), r);
            } else if r["retryable"] != true {
                r["attempts"] = json!(t.attempt);
                r["arch"] = json!(t.lane);
                r["first_classification"] = previous["classification"].clone();
                g.results.insert(t.lane.clone(), r);
            } else if t.attempt <= retries {
                let extra = g.ladder.get(&t.lane).cloned().unwrap_or(json!([]));
                g.submit("retry", &t.lane, t.attempt + 1, extra)?;
            }
        }
    }
    let mut presentation = requested.clone();
    if requested.iter().any(|a| a == "amd64") {
        if g.net {
            presentation.push("net".into())
        }
        presentation.push("amd64-bios".into())
    }
    if g.conf {
        presentation.extend(requested.iter().map(|a| format!("conf:{a}")))
    }
    if g.stor {
        presentation.extend(requested.iter().map(|a| format!("stor:{a}")))
    }
    for lane in &presentation {
        let Some(r) = g.results.get(lane).cloned() else {
            continue;
        };
        let ok = r["ok"] == true;
        let mut bits = vec![if ok {
            s(&r, "verdict", "OK")
        } else {
            s(&r, "classification", "FAIL")
        }
        .to_owned()];
        let att = r["attempts"].as_u64().unwrap_or(1);
        if att > 1 {
            bits.push(format!("{att} attempts"))
        }
        if r["recovered"] == true {
            bits.push("recovered-after-flake".into())
        }
        if let Some(rate) = r["flake_rate"].as_str().filter(|s| !s.is_empty()) {
            bits.push(format!("flake {rate}"))
        }
        let mut salient = vec![];
        if !ok && !s(&r, "salient", "").is_empty() {
            salient.push(s(&r, "salient", "").into())
        }
        if ok && r["recovered"] == true {
            salient.push(format!(
                "first pass was {}; passed on retry",
                s(&r, "first_classification", "a flake")
            ))
        }
        let name = if let Some(a) = lane.strip_prefix("conf:") {
            format!("conformance:{a}")
        } else if let Some(a) = lane.strip_prefix("stor:") {
            format!("stortest:{a}")
        } else {
            format!("smoke:{lane}")
        };
        g.stage(
            &name,
            if ok { "ok" } else { "fail" },
            bits.join(", "),
            r["exit"].as_i64().unwrap_or(0),
            r["seconds"].as_f64().unwrap_or(0.),
            salient,
        )?;
        if !ok {
            g.classification(s(&r, "classification", "FAIL"))?;
        }
    }
    let mut needed = requested.clone();
    if requested.iter().any(|s| s == "amd64") {
        if g.net {
            needed.push("net".into())
        }
        needed.push("amd64-bios".into())
    }
    for a in &requested {
        if g.conf {
            needed.push(format!("conf:{a}"))
        }
        if g.stor {
            needed.push(format!("stor:{a}"))
        }
    }
    let all_ok = !g.failed
        && !g.results.is_empty()
        && needed.len() == g.results.len()
        && needed
            .iter()
            .all(|a| g.results.get(a).is_some_and(|r| r["ok"] == true))
        && (!do_lint
            || ["source-early", "source-deferred"]
                .into_iter()
                .chain(requested.iter().map(String::as_str))
                .all(|a| g.lint.get(a) == Some(&true)))
        && (!do_lint || !do_build || requested.iter().all(|a| g.cohab.get(a) == Some(&true)))
        && g.tooling
        && g.difftest;
    if all_ok && do_lint {
        g.host
            .call("gate_stamp", json!({"arches":requested,"gated":gated}))?;
    }
    if g.built.is_empty() && do_build {
        g.classification("BUILD_FAIL")?;
    }
    let mut matrix = serde_json::Map::new();
    for (a, r) in &g.results {
        matrix.insert(a.clone(),json!({"verdict":r["verdict"],"classification":r["classification"],"attempts":r.get("attempts").cloned().unwrap_or(json!(1)),"recovered":r["recovered"]==true,"hang_kind":r["hang_kind"],"wedge_confirmed":r["wedge_confirmed"]==true,"census_stuck":r.get("census_stuck").cloned().unwrap_or(json!(0)),"run_dir":r["run_dir"]}));
    }
    let built = requested
        .iter()
        .filter(|a| g.built.contains(*a))
        .collect::<Vec<_>>();
    g.host.call(
        "finalize",
        json!({"extra":{"arches":built,"matrix":matrix}}),
    )
}
pub fn dispatch(repo: &Path, r: &Value, host: &mut dyn Host) -> Result<Value> {
    let a = &r["kwargs"];
    match r["op"].as_str().unwrap_or("") {
        "gate_run" => run(repo, a, host),
        "gate_retry_policy" => Ok(json!(RETRYABLE)),
        "gate_arches" => Ok(json!(arches(s(a, "raw", ""))?)),
        "gate_retryable" => Ok(json!(retryable(s(a, "classification", ""), &a["summary"]))),
        op => Err(Error::new(format!("unknown gate operation {op}"))),
    }
}
