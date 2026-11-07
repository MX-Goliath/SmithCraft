//! Independent text-format implementation. No original application code or assets.
use crate::{project::*, rf::*};
mod original;
pub use original::{circle_text, original_xml, point_text};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Noise {
    pub hz: f64,
    pub fmin_db: f64,
    pub opt: C,
    pub rn: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Network {
    pub name: String,
    pub z0: f64,
    pub ports: usize,
    pub points: Vec<SPoint>,
    pub noise: Vec<Noise>,
    pub source: String,
}
fn nums(line: &str) -> Result<Vec<f64>, String> {
    line.replace(',', " ")
        .split_whitespace()
        .map(|s| {
            s.replace(['d', 'D'], "E")
                .parse::<f64>()
                .map_err(|_| format!("Invalid number: {s}"))
        })
        .collect()
}
fn pair(a: f64, b: f64, fmt: &str) -> C {
    match fmt {
        "RI" => c(a, b),
        "DB" | "DBANGLE" => C::from_polar(10f64.powf(a / 20.), b.to_radians()),
        _ => C::from_polar(a, b.to_radians()),
    }
}
pub fn read_network(path: &Path) -> Result<Network, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&bytes);
    let mut n = if text.to_uppercase().contains("CITIFILE") {
        parse_citi(&text)
    } else if text.to_uppercase().contains("EZNEC") || text.contains("Freq MHz") {
        parse_eznec(&text)
    } else {
        parse_touchstone(
            &text,
            path.extension()
                .and_then(|x| x.to_str())
                .unwrap_or("s2p")
                .eq_ignore_ascii_case("s1p"),
        )
    }?;
    n.name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into();
    Ok(n)
}
pub fn parse_touchstone(text: &str, one_port: bool) -> Result<Network, String> {
    let text = text
        .trim_start_matches('\u{feff}')
        .split('\u{001a}')
        .next()
        .unwrap_or("");
    let mut n = Network {
        name: "Touchstone".into(),
        z0: 50.,
        ports: if one_port { 1 } else { 2 },
        points: vec![],
        noise: vec![],
        source: text.into(),
    };
    let mut unit = 1e9;
    let mut fmt = "MA".to_string();
    let mut parameter = "S".to_string();
    let mut pending = Vec::new();
    let mut noise = false;
    let mut network_data = true;
    let mut matrix_12_21 = false;
    let mut normalized_parameters = true;
    for (li, raw) in text.lines().enumerate() {
        let line = raw.split('!').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let upper = line.to_uppercase();
        if let Some(option_line) = upper.strip_prefix('#') {
            let a: Vec<_> = option_line.split_whitespace().collect();
            for (i, t) in a.iter().enumerate() {
                match *t {
                    "HZ" => unit = 1.,
                    "KHZ" => unit = 1e3,
                    "MHZ" => unit = 1e6,
                    "GHZ" => unit = 1e9,
                    "S" | "Z" | "Y" | "H" | "A" | "G" => parameter = t.to_string(),
                    "MA" | "DB" | "RI" => fmt = t.to_string(),
                    "R" => {
                        n.z0 = a
                            .get(i + 1)
                            .ok_or("Missing reference resistance")?
                            .parse()
                            .map_err(|_| "Invalid reference resistance")?;
                    }
                    _ => {}
                }
            }
            continue;
        }
        if upper.starts_with('[') {
            if upper.starts_with("[VERSION]") {
                normalized_parameters = false;
            }
            if upper.starts_with("[NUMBER OF PORTS]") {
                n.ports = upper
                    .split(']')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .parse()
                    .map_err(|_| "Invalid port count")?;
                if n.ports > 2 || n.ports == 0 {
                    return Err("Only one- and two-port networks are supported".into());
                }
            }
            if upper.starts_with("[TWO-PORT DATA ORDER]") {
                matrix_12_21 = upper.contains("12_21");
            }
            if upper.starts_with("[NOISE DATA]") {
                noise = true;
            }
            if upper.starts_with("[END]") {
                break;
            }
            if upper.starts_with("[REFERENCE]") {
                let values = nums(upper.split(']').nth(1).unwrap_or(""))?;
                if let Some(v) = values.first() {
                    n.z0 = *v;
                }
                if values.iter().any(|v| *v != n.z0) {
                    return Err("Unequal port reference impedances are not supported".into());
                }
            }
            if upper.starts_with("[MATRIX FORMAT]") && !upper.contains("FULL") {
                return Err("Only full Touchstone matrices are supported".into());
            }
            network_data = !upper.starts_with("[NUMBER OF FREQUENCIES]");
            continue;
        }
        if !network_data {
            continue;
        }
        let values = nums(line).map_err(|e| format!("Line {}: {e}", li + 1))?;
        if pending.is_empty() && n.ports == 2 && values.len() == 5 && !n.points.is_empty() {
            noise = true;
        }
        pending.extend(values);
        let count = if noise { 5 } else { 1 + 2 * n.ports * n.ports };
        while pending.len() >= count {
            let row: Vec<_> = pending.drain(..count).collect();
            let hz = row[0] * unit;
            if noise {
                n.noise.push(Noise {
                    hz,
                    fmin_db: row[1],
                    opt: C::from_polar(row[2], row[3].to_radians()),
                    rn: row[4],
                });
            } else {
                let mut s = [c(0., 0.); 4];
                for j in 0..n.ports * n.ports {
                    s[j] = pair(row[1 + 2 * j], row[2 + 2 * j], &fmt);
                }
                if matrix_12_21 {
                    s.swap(1, 2);
                }
                if parameter != "S" {
                    if n.ports == 1 {
                        s[0] = match parameter.as_str() {
                            "Z" => {
                                gamma(s[0] * if normalized_parameters { n.z0 } else { 1. }, n.z0)
                            }
                            "Y" => gamma(
                                inv(s[0] / if normalized_parameters { n.z0 } else { 1. }),
                                n.z0,
                            ),
                            _ => {
                                return Err(
                                    "Only S, Z and Y parameters are supported for one-port files"
                                        .into(),
                                );
                            }
                        };
                    } else {
                        s = parameters_to_s(s, &parameter, n.z0, normalized_parameters)?;
                    }
                }
                n.points.push(SPoint { hz, s });
            }
        }
    }
    if !pending.is_empty() {
        return Err("Incomplete Touchstone record".into());
    }
    validate_network(n)
}
fn validate_network(n: Network) -> Result<Network, String> {
    if n.points.is_empty() {
        return Err("No network records found".into());
    }
    if n.z0 <= 0.
        || !n.z0.is_finite()
        || n.points.iter().any(|p| {
            !p.hz.is_finite()
                || p.hz <= 0.
                || p.s.iter().any(|x| !x.re.is_finite() || !x.im.is_finite())
        })
    {
        return Err("Non-finite data or nonpositive frequency/reference resistance".into());
    }
    if n.points.windows(2).any(|p| p[1].hz <= p[0].hz) {
        return Err("Network frequencies must be strictly increasing".into());
    }
    Ok(n)
}
fn parameters_to_s(v: [C; 4], kind: &str, z0: f64, normalized: bool) -> Result<[C; 4], String> {
    let [a, b, d, e] = v;
    let det = a * e - b * d;
    let scale = if normalized { z0 } else { 1. };
    if kind == "A" {
        let bseries = d * scale;
        let cshunt = b / scale;
        let den = a + bseries / z0 + cshunt * z0 + e;
        if den.norm() < 1e-25 {
            return Err("Singular ABCD matrix".into());
        }
        return Ok([
            (a + bseries / z0 - cshunt * z0 - e) / den,
            2. / den,
            2. * det / den,
            (-a + bseries / z0 - cshunt * z0 + e) / den,
        ]);
    }
    if kind == "H" {
        let h11 = a * scale / z0;
        let h22 = e / scale * z0;
        let den = (h11 + 1.) * (h22 + 1.) - d * b;
        if den.norm() < 1e-25 {
            return Err("Singular H matrix".into());
        }
        return Ok([
            ((h11 - 1.) * (h22 + 1.) - d * b) / den,
            -2. * b / den,
            2. * d / den,
            ((h11 + 1.) * (1. - h22) + d * b) / den,
        ]);
    }
    let z = match kind {
        "Z" => v.map(|x| x * scale),
        "Y" => [e / det, -b / det, -d / det, a / det].map(|x| x * scale),
        "H" => [a - b * d / e, -b / e, d / e, inv(e)].map(|x| x * scale),
        "A" => [a / b, inv(b), det / b, e / b].map(|x| x * scale),
        _ => return Err(format!("Unsupported input parameter type {kind}")),
    };
    let [a, b, d, e] = z;
    let den = (a + z0) * (e + z0) - b * d;
    if den.norm() < 1e-25 {
        return Err("Singular parameter matrix".into());
    }
    Ok([
        ((a - z0) * (e + z0) - b * d) / den,
        2. * z0 * b / den,
        2. * z0 * d / den,
        ((e - z0) * (a + z0) - b * d) / den,
    ])
}
pub fn parse_eznec(text: &str) -> Result<Network, String> {
    let mut n = Network {
        name: "EZNEC".into(),
        z0: 50.,
        ports: 1,
        points: vec![],
        noise: vec![],
        source: text.into(),
    };
    for line in text.lines() {
        if let Ok(v) = nums(line)
            && v.len() >= 4
            && v[1] == 1.
        {
            n.points.push(SPoint {
                hz: v[0] * 1e6,
                s: [gamma(c(v[2], v[3]), 50.), c(0., 0.), c(0., 0.), c(0., 0.)],
            });
        }
    }
    validate_network(n)
}
pub fn parse_citi(text: &str) -> Result<Network, String> {
    let mut n = Network {
        name: "CITI".into(),
        z0: 50.,
        ports: 2,
        points: vec![],
        noise: vec![],
        source: text.into(),
    };
    let lines: Vec<_> = text.lines().map(str::trim).collect();
    let mut frequency = Vec::new();
    let mut defs: Vec<(String, String)> = vec![];
    let mut arrays: Vec<Vec<Vec<f64>>> = vec![];
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].to_uppercase();
        if line.starts_with("DATA ") {
            let tokens: Vec<_> = line.split_whitespace().collect();
            if tokens.len() >= 3 {
                defs.push((tokens[1].into(), tokens[2].into()));
            }
        }
        if line == "VAR_LIST_BEGIN" {
            i += 1;
            while i < lines.len() && lines[i].to_uppercase() != "VAR_LIST_END" {
                frequency.extend(nums(lines[i])?);
                i += 1;
            }
        }
        if line.starts_with("SEG ") {
            let v = nums(&lines[i][4..])?;
            if v.len() != 3 || v[2] < 2. {
                return Err("Invalid CITI segment".into());
            }
            for j in 0..v[2] as usize {
                frequency.push(v[0] + (v[1] - v[0]) * j as f64 / (v[2] - 1.));
            }
        }
        if line == "BEGIN" {
            let mut arr = Vec::new();
            i += 1;
            while i < lines.len() && lines[i].to_uppercase() != "END" {
                if !lines[i].is_empty() {
                    arr.push(nums(lines[i])?);
                }
                i += 1;
            }
            arrays.push(arr);
        }
        i += 1;
    }
    let find = |name: &str| defs.iter().position(|(key, _)| key == name);
    if let Some(idx) = find("PORTZ[1]")
        && let Some(v) = arrays
            .get(idx)
            .and_then(|a| a.first())
            .and_then(|a| a.first())
    {
        n.z0 = *v;
    }
    for (i, hz) in frequency.iter().enumerate() {
        let mut s = [c(0., 0.); 4];
        for (j, name) in ["S[1,1]", "S[2,1]", "S[1,2]", "S[2,2]"].iter().enumerate() {
            if let Some(idx) = find(name) {
                let v = arrays
                    .get(idx)
                    .and_then(|a| a.get(i))
                    .ok_or("CITI array length does not match frequency list")?;
                if v.len() < 2 {
                    return Err("CITI S parameter needs a numeric pair".into());
                }
                s[j] = pair(v[0], v[1], &defs[idx].1);
            }
        }
        n.points.push(SPoint { hz: *hz, s });
        if let (Some(fi), Some(oi), Some(ri)) = (find("NFMIN"), find("SOPT"), find("RN"))
            && let (Some(f), Some(o), Some(r)) = (
                arrays.get(fi).and_then(|a| a.get(i)),
                arrays.get(oi).and_then(|a| a.get(i)),
                arrays.get(ri).and_then(|a| a.get(i)),
            )
            && o.len() >= 2
            && !f.is_empty()
            && !r.is_empty()
        {
            n.noise.push(Noise {
                hz: *hz,
                fmin_db: f[0],
                opt: pair(o[0], o[1], &defs[oi].1),
                rn: r[0] / n.z0,
            });
        }
    }
    if let Some(idx) = find("PORTZ[1]")
        && let Some(v) = arrays
            .get(idx)
            .and_then(|a| a.first())
            .and_then(|a| a.first())
    {
        n.z0 = *v;
    }
    if find("S[1,1]").is_none() {
        return Err("CITI has no S[1,1] array".into());
    }
    if find("S[2,2]").is_none() {
        n.ports = 1;
    }
    validate_network(n)
}
pub fn export_parameters(n: &Network, kind: &str, normalized: bool) -> Result<String, String> {
    if n.ports == 1 && !matches!(kind, "S" | "Z" | "Y") {
        return Err("H and ABCD parameters require a two-port network".into());
    }
    let mut out = format!(
        "! SmithCraft {kind} parameters; {}\n# HZ {kind} RI R {}\n",
        if normalized {
            "normalized"
        } else {
            "physical SI units"
        },
        n.z0
    );
    if kind != "S" && !normalized {
        out = format!(
            "! SmithCraft physical SI units\n[Version] 2.0\n# HZ {kind} RI R {}\n[Number of Ports] {}\n[Number of Frequencies] {}\n[Network Data]\n",
            n.z0,
            n.ports,
            n.points.len()
        );
    }
    for p in &n.points {
        let v = if n.ports == 1 {
            let z = impedance(p.s[0], n.z0);
            let value = match kind {
                "Z" => z / if normalized { n.z0 } else { 1. },
                "Y" => inv(z) * if normalized { n.z0 } else { 1. },
                _ => p.s[0],
            };
            [value, c(0., 0.), c(0., 0.), c(0., 0.)]
        } else {
            p.to_parameters(kind, n.z0, normalized)?
        };
        out.push_str(&format!("{:.12e}", p.hz));
        for x in v.iter().take(if n.ports == 1 { 1 } else { 4 }) {
            out.push_str(&format!(" {:.12e} {:.12e}", x.re, x.im));
        }
        out.push('\n');
    }
    if kind == "S" && n.ports == 2 && !n.noise.is_empty() {
        out.push_str("! Noise: Hz, Fmin dB, GammaOpt magnitude, angle degrees, Rn/Z0\n");
        for noise in &n.noise {
            out.push_str(&format!(
                "{:.12e} {:.12e} {:.12e} {:.12e} {:.12e}\n",
                noise.hz,
                noise.fmin_db,
                noise.opt.norm(),
                noise.opt.arg().to_degrees(),
                noise.rn
            ));
        }
    }
    if kind != "S" && !normalized {
        out.push_str("[End]\n");
    }
    Ok(out)
}
pub fn point_csv(p: &Project, delimiter: char) -> String {
    let mut out = format!(
        "Type{delimiter}Point{delimiter}R[ohm]{delimiter}X[ohm]{delimiter}Q{delimiter}Frequency[Hz]{delimiter}ReturnLoss[dB]{delimiter}VSWR\n"
    );
    let mut add = |kind: &str, name: &str, z: C, hz: f64| {
        out.push_str(&format!("{kind}{delimiter}{name}{delimiter}{:.9}{delimiter}{:.9}{delimiter}{:.9}{delimiter}{hz:.9}{delimiter}{:.9}{delimiter}{:.9}\n",z.re,z.im,quality(z),-db(gamma(z,p.z0).norm()),vswr(gamma(z,p.z0))));
    };
    for d in &p.points {
        add("DP", &d.name, d.z, d.hz);
    }
    if let Some(a) = p.active() {
        for (i, z) in p.stages().iter().enumerate().skip(1) {
            add("TP", &i.to_string(), *z, a.hz);
        }
    }
    for (i, (f, z)) in p.swept().iter().enumerate() {
        add("SP", &(i + 1).to_string(), *z, *f);
    }
    out
}
pub fn circle_csv(p: &Project) -> String {
    circle_delimited(p, ',')
}
pub fn circle_delimited(p: &Project, delimiter: char) -> String {
    let d = delimiter;
    let mut out = [
        "Label", "CenterRe", "CenterIm", "Radius", "Q", "Visible", "Details",
    ]
    .join(&d.to_string());
    out.push('\n');
    for x in &p.circles {
        out.push_str(&format!(
            "\"{}\"{d}{}{d}{}{d}{}{d}{}{d}{}{d}\"{}\"\n",
            x.label.replace('"', "\"\""),
            x.center.re,
            x.center.im,
            x.radius,
            x.q.map(|x| x.to_string()).unwrap_or_default(),
            x.visible,
            x.details.replace('"', "\"\"")
        ));
    }
    out
}
pub fn netlist(p: &Project) -> String {
    let mut out="! SmithCraft matching network (load to generator)\nDIM FREQ=1e6 RES=1 IND=1e-9 CAP=1e-12 LNG=1e-3\nCKT\n".to_string();
    let mut node = 1;
    for e in p.elements.iter().filter(|e| e.enabled) {
        let next = if e.kind.shunt() { 0 } else { node + 1 };
        let v = e.value;
        let line = match e.kind {
            Kind::SeriesR | Kind::ShuntR => format!("RES {node} {next} R={v}"),
            Kind::SeriesL | Kind::ShuntL => format!("IND {node} {next} L={v}"),
            Kind::SeriesC | Kind::ShuntC => format!("CAP {node} {next} C={v}"),
            Kind::Transformer => format!("TRF {node} {next} 0 0 N={v} R1=0"),
            Kind::SeriesParallelRlc => {
                format!("PRLC {node} {next} R={v} L={} C={}", e.l_nh, e.c_pf)
            }
            Kind::ShuntSeriesRlc => format!("SRLC {node} 0 R={v} L={} C={}", e.l_nh, e.c_pf),
            _ => format!(
                "TLINP {node} {} 0 Z={} L={} K={} A={} F=0",
                if e.kind == Kind::OpenStub { 99 } else { next },
                e.z0,
                e.physical_m() * e.er.sqrt() * 1000.,
                e.er,
                e.loss
            ),
        };
        out.push_str(&line);
        out.push('\n');
        if next != 0 {
            node = next;
        }
    }
    out.push_str(&format!(
        "DEF2P 1 {node} A\nFREQ\nFIXED {}\n",
        p.active().map_or(p.default_hz, |a| a.hz) / 1e6
    ));
    out
}
pub fn save_project(path: &Path, p: &Project) -> Result<(), String> {
    p.validate()?;
    let json = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    let bytes = if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("xmlsc"))
    {
        original_xml(p)?
    } else {
        json
    };
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}
pub fn load_project(path: &Path) -> Result<Project, String> {
    load_project_with_reference(path, 50.)
}
/// Original Smith XML uses the application's global reference impedance.
pub fn load_project_with_reference(path: &Path, z0: f64) -> Result<Project, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let text = text.trim_start_matches('\u{feff}');
    let p: Project = if text.trim_start().starts_with('<') {
        let doc = roxmltree::Document::parse(text).map_err(|e| e.to_string())?;
        if doc.root_element().has_tag_name("CSmithChartDesign") {
            let mut p = parse_original_project_with_reference(&doc, z0)?;
            p.title = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into();
            p
        } else {
            let node = doc
                .descendants()
                .find(|n| n.has_tag_name("project"))
                .ok_or("Unknown project XML schema")?;
            let json: String = node.children().filter_map(|n| n.text()).collect();
            serde_json::from_str(&json).map_err(|e| e.to_string())?
        }
    } else {
        serde_json::from_str(text).map_err(|e| e.to_string())?
    };
    p.validate()?;
    Ok(p)
}

fn xml_value(node: roxmltree::Node<'_, '_>, name: &str, default: f64) -> f64 {
    node.children()
        .find(|n| n.tag_name().name().eq_ignore_ascii_case(name))
        .and_then(|n| n.text())
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
fn xml_flag(node: roxmltree::Node<'_, '_>, name: &str, default: bool) -> bool {
    node.children()
        .find(|n| n.tag_name().name().eq_ignore_ascii_case(name))
        .and_then(|n| n.text())
        .map_or(default, |s| s == "true" || s == "1")
}
fn xml_s(node: roxmltree::Node<'_, '_>) -> [C; 4] {
    ["S11", "S21", "S12", "S22"].map(|name| {
        c(
            xml_value(node, &format!("{name}re"), 0.),
            xml_value(node, &format!("{name}im"), 0.),
        )
    })
}
pub fn parse_original_project(doc: &roxmltree::Document<'_>) -> Result<Project, String> {
    parse_original_project_with_reference(doc, 50.)
}
fn parse_original_project_with_reference(
    doc: &roxmltree::Document<'_>,
    z0: f64,
) -> Result<Project, String> {
    let root = doc.root_element();
    let mut p = Project {
        z0,
        ..Project::default()
    };
    for node in root.children().filter(|n| n.has_tag_name("CDatapoint")) {
        if xml_value(node, "IsRealDP", 1.) != 1. {
            continue;
        }
        let hz = xml_value(node, "Frequency", p.default_hz);
        let z = c(xml_value(node, "R", 0.), xml_value(node, "X", 0.));
        if xml_value(node, "StartDatapoint", 0.) == 1. {
            p.start = p.points.len();
        }
        p.points.push(Datapoint {
            name: format!("DP{}", p.points.len() + 1),
            z,
            hz,
        });
    }
    if let Some(d) = p.active() {
        p.default_hz = d.hz;
    }
    for node in root.children().filter(|n| n.has_tag_name("CComponent")) {
        let typ = xml_value(node, "Typ", -1.) as i32;
        let value = xml_value(node, "Value", 0.);
        let known = match typ {
            2 => Some(Kind::SeriesC),
            3 => Some(Kind::ShuntC),
            4 => Some(Kind::SeriesL),
            5 => Some(Kind::ShuntL),
            6 => Some(Kind::SeriesR),
            7 => Some(Kind::ShuntR),
            12 => Some(Kind::SeriesParallelRlc),
            13 => Some(Kind::ShuntSeriesRlc),
            20 => Some(Kind::Transformer),
            8 => Some(Kind::Line),
            9 => Some(Kind::OpenStub),
            30 => Some(Kind::ShortStub),
            _ => None,
        };
        let make = |kind| {
            let mut e = Element::new(kind, p.default_hz);
            e.value = match kind {
                Kind::SeriesC | Kind::ShuntC => value * 1e12,
                Kind::SeriesL | Kind::ShuntL => value * 1e9,
                _ => value,
            };
            e.z0 = xml_value(node, "LineImp", 50.).max(1e-6);
            e.er = xml_value(node, "Er", 1.).max(1e-6);
            e.loss = xml_value(node, "Alpha", 0.);
            e.l_nh = xml_value(node, "LValueRLC", 1e-8).max(1e-24) * 1e9;
            e.c_pf = xml_value(node, "CValueRLC", 1e-11).max(1e-24) * 1e12;
            if matches!(kind, Kind::SeriesParallelRlc | Kind::ShuntSeriesRlc) {
                e.value = xml_value(node, "RValueRLC", 50.);
            }
            e
        };
        let e = if let Some(k) = known {
            make(k)
        } else {
            let zin = c(xml_value(node, "Rbegin", 0.), xml_value(node, "Xbegin", 0.));
            let zout = c(xml_value(node, "Rend", 0.), xml_value(node, "Xend", 0.));
            let candidate = Kind::ALL
                .iter()
                .map(|k| {
                    let e = make(*k);
                    let distance = (e.apply(zin, p.default_hz) - zout).norm();
                    (e, distance)
                })
                .filter(|(_, d)| d.is_finite())
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match candidate {
                Some((e, d)) if d < 1e-3 * (1. + zout.norm()) => e,
                _ => return Err(format!("Unsupported original component type {typ}")),
            }
        };
        let mut e = e;
        // Lossy-line examples store a rounded wavelength value but a more precise
        // endpoint. Recover the actual electrical length and attenuation from Γ.
        if e.kind == Kind::Line && e.loss > 0. {
            let zin = c(xml_value(node, "Rbegin", 0.), xml_value(node, "Xbegin", 0.));
            let zout = c(xml_value(node, "Rend", 0.), xml_value(node, "Xend", 0.));
            let ratio = gamma(zout, e.z0) / gamma(zin, e.z0);
            if ratio.norm().is_finite() && ratio.norm() > 0. && ratio.norm() < 1. {
                let base = (-ratio.arg() / (2. * std::f64::consts::TAU)).rem_euclid(0.5);
                e.value = base + ((e.value - base) / 0.5).round() * 0.5;
                e.loss = -ratio.norm().ln() * 8.685889638 / (2. * e.physical_m());
            }
        }
        p.elements.push(e);
    }
    for node in root.children() {
        let tag = node.tag_name().name();
        let mut q = None;
        let result = match tag {
            "QList" => {
                let value = node
                    .text()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .map_err(|_| "Invalid Q contour")?;
                q = Some(value);
                Some((c(0., 0.), 0., format!("Q = {value}")))
            }
            "VSWRList" => {
                let value = node
                    .text()
                    .unwrap_or("1")
                    .parse::<f64>()
                    .map_err(|_| "Invalid VSWR contour")?;
                Some((
                    c(0., 0.),
                    (value - 1.) / (value + 1.),
                    format!("VSWR = {value}"),
                ))
            }
            "CStabilityCircles" => {
                let sp = SPoint {
                    hz: xml_value(node, "F", 1.),
                    s: xml_s(node),
                };
                let source = xml_value(node, "Typ", 1.) == 1.;
                sp.stability_circle(source).map(|(center, r)| {
                    (
                        center,
                        r,
                        format!("Stability {}", if source { "input" } else { "output" }),
                    )
                })
            }
            "CGainCirclesList" => {
                let sp = SPoint {
                    hz: xml_value(node, "F", 1.),
                    s: xml_s(node),
                };
                let source = xml_value(node, "Typ", 1.) == 1.;
                let gain = xml_value(node, "P", 0.);
                sp.gain_circle(source, gain, xml_value(node, "GainType", 1.) == 2.)
                    .map(|(center, r)| (center, r, format!("Gain {gain:.3} dB")))
            }
            "CNoiseList" => {
                let nf = xml_value(node, "NF", 0.);
                let fmin = xml_value(node, "NFmin", 0.);
                let opt = c(
                    xml_value(node, "RNFminRE", 0.),
                    xml_value(node, "RNFminIM", 0.),
                );
                let rn = xml_value(node, "Rnoise", 1.)
                    / if xml_value(node, "Normalized", 1.) == 1. {
                        1.
                    } else {
                        p.z0
                    };
                noise_circle(nf, fmin, opt, rn)
                    .map(|(center, r)| (center, r, format!("Noise NF={nf:.3} dB")))
            }
            _ => None,
        };
        if let Some((center, radius, label)) = result {
            p.circles.push(Circle {
                definition: match tag {
                    "CStabilityCircles" => Some(CircleDefinition::Stability {
                        hz: xml_value(node, "F", p.default_hz),
                        s: xml_s(node),
                        source: xml_value(node, "Typ", 1.) == 1.,
                    }),
                    "CGainCirclesList" => Some(CircleDefinition::Gain {
                        hz: xml_value(node, "F", p.default_hz),
                        s: xml_s(node),
                        source: xml_value(node, "Typ", 1.) == 1.,
                        unilateral: xml_value(node, "GainType", 1.) == 2.,
                        gain_db: xml_value(node, "P", 0.),
                    }),
                    "CNoiseList" => Some(CircleDefinition::Noise {
                        hz: xml_value(node, "F", p.default_hz),
                        nf_db: xml_value(node, "NF", 0.),
                        minimum_db: xml_value(node, "NFmin", 0.),
                        optimum: c(
                            xml_value(node, "RNFminRE", 0.),
                            xml_value(node, "RNFminIM", 0.),
                        ),
                        rn: xml_value(node, "Rnoise", 1.)
                            / if xml_value(node, "Normalized", 1.) == 1. {
                                1.
                            } else {
                                p.z0
                            },
                    }),
                    _ => None,
                },
                details: node
                    .children()
                    .filter(|n| n.is_element())
                    .map(|n| format!("{}={}", n.tag_name().name(), n.text().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join("; "),
                center,
                radius,
                label,
                q,
                visible: xml_flag(node, "Visible", true),
                highlighted: xml_flag(node, "Highlight", false),
            });
        }
    }
    if let Some(node) = root
        .children()
        .find(|n| n.has_tag_name("CSweepDatapointParameters"))
    {
        let start = xml_value(node, "StartFrequency", p.sweep.start);
        let stop = xml_value(node, "StopFrequency", p.sweep.stop);
        let step = xml_value(node, "StepFrequency", 1e7);
        p.sweep = Sweep {
            start,
            stop,
            step_hz: Some(step),
            count: (((stop - start) / step).round() as usize + 1).clamp(2, 10001),
            datapoints: xml_value(node, "Sweeptype", 2.) == 1.,
            visible: true,
            ..Sweep::default()
        };
    }
    p.validate()?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversions_round_trip() {
        let original = parse_touchstone(
            "# GHz S RI R 75\n1 0.3 0.1 2 -0.5 0.03 0.01 0.2 -0.1",
            false,
        )
        .unwrap();
        for kind in ["S", "Z", "Y", "H", "A"] {
            for normalized in [false, true] {
                let text = export_parameters(&original, kind, normalized).unwrap();
                let n = parse_touchstone(&text, false).unwrap();
                for (i, v) in original.points[0].s.iter().enumerate() {
                    assert!(
                        (*v - n.points[0].s[i]).norm() < 1e-9,
                        "{kind} {normalized}: {text}"
                    );
                }
            }
        }
        let one = parse_touchstone("# GHz S RI R 50\n1 0.3 0.2", true).unwrap();
        for kind in ["Z", "Y"] {
            for normalized in [false, true] {
                let text = export_parameters(&one, kind, normalized).unwrap();
                let back = parse_touchstone(&text, true).unwrap();
                assert!((back.points[0].s[0] - one.points[0].s[0]).norm() < 1e-10);
            }
        }
        let thru = parse_touchstone("# GHz S RI R 50\n1 0 0 1 0 1 0 0 0", false).unwrap();
        for kind in ["H", "A"] {
            let text = export_parameters(&thru, kind, false).unwrap();
            let back = parse_touchstone(&text, false).unwrap();
            assert!((back.points[0].s[1] - c(1., 0.)).norm() < 1e-10);
        }
    }
    #[test]
    fn xml_cdata_roundtrip() {
        let mut p = Project::demo();
        p.title = "String ]]> inside XML".into();
        let path =
            std::env::temp_dir().join(format!("smithcraft-cdata-{}.xmlsc", std::process::id()));
        // Legacy envelopes remain readable after switching .xmlsc output to Smith.
        let json = serde_json::to_string(&p).unwrap();
        std::fs::write(
            &path,
            format!(
                "<SmithCraft><project><![CDATA[{}]]></project></SmithCraft>",
                json.replace("]]>", "]]]]><![CDATA[>")
            ),
        )
        .unwrap();
        assert_eq!(load_project(&path).unwrap(), p);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn touchstone_order_noise() {
        let n=parse_touchstone("# MHz S RI R 75\n100 0.1 0 2 0 0.02 0 0.3 0\n200 0.2 0 3 0 0.03 0 0.4 0\n100 1.2 0.3 45 0.1",false).unwrap();
        assert_eq!(n.points[0].s[1], c(2., 0.));
        assert_eq!(n.points[0].s[2], c(0.02, 0.));
        assert_eq!(n.points[0].hz, 1e8);
        assert_eq!(n.noise.len(), 1);
        assert_eq!(n.z0, 75.);
        let exported = export_parameters(&n, "S", false).unwrap();
        let reread = parse_touchstone(&exported, false).unwrap();
        assert_eq!(reread.noise.len(), 1);
        assert!((reread.noise[0].opt - n.noise[0].opt).norm() < 1e-10);
        assert_eq!(reread.noise[0].rn, n.noise[0].rn);
    }
    #[test]
    fn citi_order() {
        let n=parse_citi("CITIFILE A.01.00\nVAR FREQ MAG 1\nDATA S[1,1] RI\nDATA S[1,2] RI\nDATA S[2,1] RI\nDATA S[2,2] RI\nVAR_LIST_BEGIN\n1e9\nVAR_LIST_END\nBEGIN\n0.1,0\nEND\nBEGIN\n0.02,0\nEND\nBEGIN\n2,0\nEND\nBEGIN\n0.3,0\nEND").unwrap();
        assert_eq!(n.points[0].s[1], c(2., 0.));
        assert_eq!(n.points[0].s[2], c(0.02, 0.));
    }
    #[test]
    fn eznec() {
        let n = parse_eznec("\"EZNEC\"\n3.5,1,50,0,1,1").unwrap();
        assert_eq!(n.points[0].s[0], c(0., 0.));
    }
    #[test]
    fn malformed_rejected() {
        assert!(parse_touchstone("# MHz S RI R 50\n1 0.1", false).is_err());
        assert!(parse_touchstone("# MHz S RI R 50\n1 NaN 0", true).is_err());
    }
    #[test]
    fn project_roundtrip() {
        let p = Project::demo();
        let path =
            std::env::temp_dir().join(format!("smithcraft-test-{}.smithcraft", std::process::id()));
        save_project(&path, &p).unwrap();
        assert_eq!(load_project(&path).unwrap(), p);
        std::fs::remove_file(path).unwrap();
    }
}
