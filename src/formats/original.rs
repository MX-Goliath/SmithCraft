//! Smith V4.1 XML interchange. This schema contains design data; display
//! preferences and the reference impedance are global settings in Smith.
use super::*;
use std::fmt::Write;
pub fn point_text(p: &Project, delimiter: char) -> String {
    let mut rows = vec![vec![
        "Start DP".into(),
        "Point".into(),
        "R[Ohm]".into(),
        "X[Ohm]".into(),
        "Q".into(),
        "Frequency[Hz]".into(),
    ]];
    let mut add = |start: bool, index: usize, z: C, f: f64| {
        rows.push(vec![
            start.to_string(),
            index.to_string(),
            format!("{:.3}", z.re),
            format!("{:.3}", z.im),
            format!("{:.3}", quality(z)),
            format!("{f:.3}"),
        ])
    };
    for (i, d) in p.points.iter().enumerate() {
        add(i == p.start, i + 1, d.z, d.hz);
    }
    if let Some(d) = p.active() {
        for (i, z) in p.stages().into_iter().enumerate().skip(1) {
            add(false, p.points.len() + i, z, d.hz);
        }
    }
    for (i, (f, z)) in p.swept().into_iter().enumerate() {
        add(false, i + 1, z, f);
    }
    delimited(rows, delimiter)
}
fn delimited(rows: Vec<Vec<String>>, delimiter: char) -> String {
    rows.into_iter()
        .map(|row| {
            if delimiter == ' ' {
                row.into_iter()
                    .map(|v| format!("{v:<18}"))
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            } else {
                row.join(&delimiter.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n")
        + "\r\n"
}
pub fn circle_text(p: &Project, delimiter: char) -> String {
    let mut indices: Vec<_> = p
        .circles
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.family(), 'G' | 'S' | 'N'))
        .map(|(i, _)| i)
        .collect();
    indices.sort_by_key(|i| match p.circles[*i].family() {
        'G' => 0,
        'S' => 1,
        _ => 2,
    });
    let mut rows = vec![];
    let f = |hz: f64| {
        if hz >= 1e9 {
            format!("{:.1}GHz", hz / 1e9)
        } else if hz >= 1e6 {
            format!("{:.1}MHz", hz / 1e6)
        } else {
            format!("{hz:.1}Hz")
        }
    };
    for i in indices {
        let circle = &p.circles[i];
        let mut row = vec![p.circle_name(i)];
        match &circle.definition {
            Some(CircleDefinition::Gain {
                hz,
                s,
                source,
                unilateral,
                gain_db,
            }) => {
                row.extend([
                    format!(
                        "{} plane {} gain circle",
                        if *source { "input" } else { "output" },
                        if *unilateral { "unilateral" } else { "const." }
                    ),
                    format!("{gain_db:.2}"),
                    format!("{:.2}", SPoint { hz: *hz, s: *s }.stability().gain_db),
                ]);
                for j in [0, 2, 1, 3] {
                    row.push(format!("{:.2}", s[j].norm()));
                    row.push(format!("{:.2}", s[j].arg().to_degrees()));
                }
                row.push(f(*hz));
            }
            Some(CircleDefinition::Stability { hz, s, source }) => {
                let stable = if *source {
                    s[3].norm() < 1.
                } else {
                    s[0].norm() < 1.
                };
                row.extend([
                    format!(
                        "{} plane stability circle",
                        if *source { "Input" } else { "Output" }
                    ),
                    format!(
                        "stable {}",
                        if stable == (circle.center.norm() < circle.radius) {
                            "inside"
                        } else {
                            "outside"
                        }
                    ),
                    format!("{:.2}", SPoint { hz: *hz, s: *s }.stability().k),
                ]);
                for j in [0, 2, 1, 3] {
                    row.push(format!("{:.2}", s[j].norm()));
                    row.push(format!("{:.2}", s[j].arg().to_degrees()));
                }
                row.push(f(*hz));
            }
            Some(CircleDefinition::Noise {
                hz,
                nf_db,
                minimum_db,
                optimum,
                rn,
            }) => row.extend([
                format!("{nf_db:.2}"),
                format!("{minimum_db:.2}"),
                format!("{:.2}", optimum.norm()),
                format!("{:.2}", optimum.arg().to_degrees()),
                format!("{rn:.2}"),
                f(*hz),
            ]),
            None => row.extend([circle.label.clone(), circle.details.clone()]),
        }
        rows.push(row);
    }
    delimited(rows, delimiter)
}

fn value(out: &mut String, name: &str, value: impl std::fmt::Display) {
    let _ = write!(out, "<{name}>{value}</{name}>");
}
fn datapoint(out: &mut String, tag: &str, index: usize, real: bool, start: bool, z: C, hz: f64) {
    let _ = write!(out, "<{tag}>");
    for (name, v) in [
        ("StartDatapoint", usize::from(start)),
        ("IsRealDP", usize::from(real)),
        ("IndexOfDP", index),
    ] {
        value(out, name, v);
    }
    for (name, v) in [
        ("Frequency", hz),
        ("R", z.re),
        ("X", z.im),
        ("U", 0.),
        ("V", 0.),
    ] {
        value(out, name, v);
    }
    let _ = write!(out, "</{tag}>");
}
fn parameters(out: &mut String, s: [C; 4], stability: bool) {
    for (name, z) in [("S11", s[0]), ("S12", s[2]), ("S21", s[1]), ("S22", s[3])] {
        let (re, im) = if !stability {
            ("re", "im")
        } else if name == "S11" {
            ("Re", "Im")
        } else {
            ("RE", "IM")
        };
        value(out, &format!("{name}{re}"), z.re);
        value(out, &format!("{name}{im}"), z.im);
    }
}
fn flags(out: &mut String, circle: &Circle) {
    value(out, "Visible", circle.visible);
    value(out, "Highlight", circle.highlighted);
}
pub fn original_xml(p: &Project) -> Result<String, String> {
    p.validate()?;
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><CSmithChartDesign xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">",
    );
    for (i, d) in p.points.iter().enumerate() {
        datapoint(&mut out, "CDatapoint", i, true, i == p.start, d.z, d.hz);
    }
    if let Some(start) = p.active() {
        let mut z = start.z;
        let components: Vec<_> = p.elements.iter().filter(|e| e.enabled).collect();
        for (i, e) in components.iter().enumerate() {
            z = e.apply(z, start.hz);
            datapoint(
                &mut out,
                "CDatapoint",
                p.points.len() + i,
                false,
                false,
                z,
                start.hz,
            );
        }
        z = start.z;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for (i, e) in components.iter().enumerate() {
            out.push_str("<CComponent>");
            let id = stamp + i as u128;
            value(
                &mut out,
                "GUID",
                format!(
                    "{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
                    (id >> 96) as u32,
                    (id >> 80) as u16,
                    (id >> 64) & 0xfff,
                    (id >> 48) & 0xfff,
                    id & 0xffffffffffff
                ),
            );
            let typ = match e.kind {
                Kind::SeriesC => 2,
                Kind::ShuntC => 3,
                Kind::SeriesL => 4,
                Kind::ShuntL => 5,
                Kind::SeriesR => 6,
                Kind::ShuntR => 7,
                Kind::Line => 8,
                Kind::OpenStub => 9,
                Kind::ShortStub => 30,
                Kind::Transformer => 20,
                Kind::SeriesParallelRlc => 12,
                Kind::ShuntSeriesRlc => 13,
            };
            value(&mut out, "Typ", typ);
            let line = matches!(e.kind, Kind::Line | Kind::OpenStub | Kind::ShortStub);
            // Smith stores wavelength and attenuation at the design frequency.
            let ratio = start.hz / e.reference_hz;
            let v = match e.kind {
                Kind::SeriesC | Kind::ShuntC => e.value * 1e-12,
                Kind::SeriesL | Kind::ShuntL => e.value * 1e-9,
                _ if line => e.value * ratio,
                _ => e.value,
            };
            value(&mut out, "Value", v);
            let end = e.apply(z, start.hz);
            let rlc = matches!(e.kind, Kind::SeriesParallelRlc | Kind::ShuntSeriesRlc);
            for (name, v) in [
                ("Xend", end.im),
                ("Rend", end.re),
                ("LineImp", if line { e.z0 } else { 0. }),
                ("Xbegin", z.im),
                ("Rbegin", z.re),
                ("Er", if line { e.er } else { 0. }),
                ("Alpha", if line { e.loss * ratio.sqrt() } else { 0. }),
                ("Lambda", if line { e.value * ratio } else { 0. }),
                ("RValueRLC", if rlc { e.value } else { 0. }),
                ("LValueRLC", if rlc { e.l_nh * 1e-9 } else { 0. }),
                ("CValueRLC", if rlc { e.c_pf * 1e-12 } else { 0. }),
            ] {
                value(&mut out, name, v);
            }
            out.push_str("</CComponent>");
            z = end;
        }
    }
    for circle in &p.circles {
        if let Some(q) = circle.q {
            value(&mut out, "QList", q);
            continue;
        }
        if circle.label.starts_with("VSWR") {
            value(
                &mut out,
                "VSWRList",
                (1. + circle.radius) / (1. - circle.radius),
            );
            continue;
        }
        match circle.definition.as_ref().ok_or_else(||format!("Source parameters are missing for '{}'. Save as .smithcraft to retain this circle.",circle.label))? {
            CircleDefinition::Stability{hz,s,source}=>{
                out.push_str("<CStabilityCircles>");flags(&mut out,circle);value(&mut out,"F",hz);value(&mut out,"Typ",if *source{1}else{2});let st=SPoint{hz:*hz,s:*s}.stability();value(&mut out,"K",st.k);let stable_zero=if *source{s[3].norm()<1.}else{s[0].norm()<1.};value(&mut out,"Stable",if stable_zero==(circle.center.norm()<circle.radius){1}else{2});parameters(&mut out,*s,true);out.push_str("</CStabilityCircles>");
            }
            CircleDefinition::Gain{hz,s,source,unilateral,gain_db}=>{
                out.push_str("<CGainCirclesList>");value(&mut out,"Typ",if *source{1}else{2});value(&mut out,"ManualCyrcle",0);value(&mut out,"GainType",if *unilateral{2}else{1});flags(&mut out,circle);value(&mut out,"Marked",false);parameters(&mut out,*s,false);let st=SPoint{hz:*hz,s:*s}.stability();value(&mut out,"K",st.k);value(&mut out,"P",gain_db);value(&mut out,"Gmax",st.gain_db);value(&mut out,"F",hz);out.push_str("</CGainCirclesList>");
            }
            CircleDefinition::Noise{hz,nf_db,minimum_db,optimum,rn}=>{
                out.push_str("<CNoiseList>");value(&mut out,"ManualCyrcle",0);value(&mut out,"FreqUnit",6);flags(&mut out,circle);value(&mut out,"Normalized",1);for(name,v)in[("F",*hz),("NF",*nf_db),("Rnoise",*rn),("RNFminIM",optimum.im),("RNFminRE",optimum.re),("NFmin",*minimum_db)]{value(&mut out,name,v);}out.push_str("</CNoiseList>");
            }
        }
    }
    if p.sweep.visible {
        for (i, (hz, z)) in p.swept().into_iter().enumerate() {
            datapoint(&mut out, "CSweepDatapoint", i, false, false, z, hz);
        }
        out.push_str("<CSweepDatapointParameters>");
        value(
            &mut out,
            "Sweeptype",
            if p.sweep.datapoints { 1 } else { 2 },
        );
        value(&mut out, "StartFrequency", p.sweep.start);
        value(&mut out, "StopFrequency", p.sweep.stop);
        value(
            &mut out,
            "StepFrequency",
            p.sweep
                .step_hz
                .unwrap_or((p.sweep.stop - p.sweep.start) / (p.sweep.count - 1) as f64),
        );
        out.push_str("</CSweepDatapointParameters>");
    }
    out.push_str("</CSmithChartDesign>");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interchange_line_at_different_reference_frequency() {
        for kind in [Kind::Line, Kind::OpenStub, Kind::ShortStub] {
            let mut p = Project::default();
            p.points.push(Datapoint {
                name: "DP1".into(),
                z: c(32., 17.),
                hz: 8e8,
            });
            let mut e = Element::new(kind, 5e8);
            e.value = 0.17;
            e.loss = if kind == Kind::Line { 0.7 } else { 0. };
            e.er = 2.3;
            p.elements.push(e);
            let xml = original_xml(&p).unwrap();
            let doc = roxmltree::Document::parse(&xml).unwrap();
            let loaded = parse_original_project(&doc).unwrap();
            for f in [4e8, 8e8, 1.2e9] {
                assert!(
                    (loaded.elements[0].apply(c(32., 17.), f)
                        - p.elements[0].apply(c(32., 17.), f))
                    .norm()
                        < 1e-8,
                    "{kind:?} at {f}"
                );
            }
        }
    }
    #[test]
    fn interchange_all_component_types() {
        for kind in Kind::ALL {
            let mut p = Project::default();
            p.points.push(Datapoint {
                name: "DP1".into(),
                z: c(32., 17.),
                hz: 5e8,
            });
            p.elements.push(Element::new(kind, 5e8));
            let xml = original_xml(&p).unwrap();
            let doc = roxmltree::Document::parse(&xml).unwrap();
            let loaded = parse_original_project(&doc).unwrap();
            assert_eq!(loaded.elements[0].kind, kind);
            assert!(
                (loaded.stages()[1] - p.stages()[1]).norm() < 1e-8,
                "{kind:?}"
            );
        }
    }
    #[test]
    fn interchange_reference_circles() {
        let path = Path::new(".reference/examples/Example13-input.xmlsc");
        if !path.exists() {
            return;
        }
        let p = load_project(path).unwrap();
        let xml = original_xml(&p).unwrap();
        let doc = roxmltree::Document::parse(&xml).unwrap();
        // The .NET reader uses different, case-sensitive field names in each
        // circle record. A case-insensitive round trip alone misses this.
        for (tag, names) in [
            ("CStabilityCircles", ["S11Re", "S11Im", "S12RE", "S12IM"]),
            ("CGainCirclesList", ["S11re", "S11im", "S12re", "S12im"]),
        ] {
            let node = doc.descendants().find(|n| n.has_tag_name(tag)).unwrap();
            for name in names {
                assert!(
                    node.children().any(|n| n.has_tag_name(name)),
                    "{tag}/{name}"
                );
            }
        }
        let loaded = parse_original_project(&doc).unwrap();
        assert_eq!(p.circles.len(), loaded.circles.len());
        for (a, b) in p.circles.iter().zip(&loaded.circles) {
            assert!((a.center - b.center).norm() < 1e-10);
            assert!((a.radius - b.radius).abs() < 1e-10);
            assert_eq!(a.definition, b.definition);
        }
    }
}
