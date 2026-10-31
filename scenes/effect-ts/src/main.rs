//! A short argument for Effect, shown rather than read.
//!
//! Two films from one narration set, concatenated at render time:
//!   A  a function morphs from a `Promise` that throws into an `Effect` whose
//!      error is in the signature (a stepped diff, not a caption)
//!   B  dependencies as cards feeding one program, three requests leaving at once
//!
//! Run with `pocket-tts serve` up, then render both plans.

use std::{fs, path::PathBuf};

use anyhow::Result;
use psychopomp::{
    author::{PlanBuilder, seconds},
    caption::{CaptionActor, CaptionAlign, CaptionPlan, CaptionSpanPlan},
    editor::diff::{Diff, add, keep, remove},
    tone::Tone,
    stage::{StageActor, StageElement},
};
use psychopomp_media::{Media, Voice};

const THROWS: &str = "A function that throws tells you nothing. The caller has to know which \
                      errors can appear, and the compiler cannot check that anyone handled them.";
const TYPED: &str = "Effect makes errors part of the return type. Success and failure sit in the \
                     same signature, so an unhandled failure becomes a compile error rather than a \
                     production incident.";
const LAYERS: &str = "Dependencies become a value. A layer declares what a program needs, and you \
                      supply it at the edge, which makes tests cheap and mocks honest.";

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let media = Media::open(&root)?;
    let voice = Voice::pocket("alba");
    let throws = media.say("throws", &voice, THROWS)?;
    let typed = media.say("typed", &voice, TYPED)?;
    let layers = media.say("layers", &voice, LAYERS)?;
    media.finish()?;
    println!("{}", psychopomp_media::show(&root)?);

    code_film(&throws, &typed)?;
    diagram_film(&layers)?;
    Ok(())
}

/// Beat one and two: the same function, changed in front of you.
fn code_film(throws: &psychopomp_media::Audio, typed: &psychopomp_media::Audio) -> Result<()> {
    let start = seconds(0.4);
    let second = start + throws.duration() + seconds(0.5);
    let duration = second + typed.duration() + seconds(0.6);

    let mut scene = PlanBuilder::new("effect-ts-code", duration);
    let throws_spoken = throws.place(&mut scene, start);
    let typed_spoken = typed.place(&mut scene, second);

    let diff = Diff {
        file_name: "user.ts",
        lines: vec![
            remove(2, "async function loadUser(id: string): Promise<User> {"),
            add(2, "function loadUser(id: string): Effect<User, HttpError> {"),
            keep("  const res = await fetch(`/users/${id}`)"),
            keep("  if (!res.ok)"),
            add(1, "    throw new Error(`HTTP ${res.status}`)"),
            keep("  return (await res.json()) as User"),
            keep("}"),
        ],
    };
    // Step one lands on the word that names the problem; step two on the fix.
    let step_times = [throws_spoken.at("throws"), typed_spoken.at("signature")];
    diff.declare(&mut scene, &step_times, seconds(0.9), true)?;

    subtitle(&mut scene, "sub-throws", THROWS, start, throws.duration());
    subtitle(
        &mut scene,
        "sub-typed",
        TYPED,
        second,
        typed.duration(),
    );

    fs::create_dir_all("target")?;
    write_plan(scene, "target/effect-ts-code.json")?;
    println!("code film {:.1}s", duration as f64 / 1e9);
    Ok(())
}

/// Beat three: three dependencies, one program, three requests leaving together.
fn diagram_film(layers: &psychopomp_media::Audio) -> Result<()> {
    let start = seconds(0.4);
    let duration = start + layers.duration() + seconds(1.0);

    let mut scene = PlanBuilder::new("effect-ts-diagram", duration);
    layers.place(&mut scene, start);
    subtitle(&mut scene, "sub-layers", LAYERS, start, layers.duration());

    let elements = vec![
        StageElement::card("db", [420.0, 790.0, 0.0], [300.0, 110.0], "Database"),
        StageElement::card("logger", [960.0, 900.0, 0.0], [300.0, 110.0], "Logger"),
        StageElement::card("clock", [1500.0, 790.0, 0.0], [300.0, 110.0], "Clock"),
        StageElement::orb("program", [960.0, 360.0, 0.0], 150.0),
        StageElement::beam("to-db", "program", "db"),
        StageElement::beam("to-logger", "program", "logger"),
        StageElement::beam("to-clock", "program", "clock"),
        StageElement::packet("ask-db", "to-db").labeled("query"),
        StageElement::packet("ask-logger", "to-logger").labeled("log"),
        StageElement::packet("ask-clock", "to-clock").labeled("now"),
    ];
    let mut stage = StageActor::declare(&mut scene, "stage", &psychopomp::stage::StagePlan {
        post: Default::default(),
        elements,
    })?;

    for id in [
        "db", "logger", "clock", "program", "to-db", "to-logger", "to-clock", "ask-db",
        "ask-logger", "ask-clock",
    ] {
        stage.channel(&mut scene, &format!("{id}.opacity"), 0.0);
        stage.glide(&mut scene, &format!("{id}.opacity"), start, 1.0, 0.5);
    }

    let wired = start + seconds(1.2);
    stage.connect(&mut scene, "to-db", wired, 0.5);
    stage.connect(&mut scene, "to-logger", wired + seconds(0.15), 0.5);
    stage.connect(&mut scene, "to-clock", wired + seconds(0.3), 0.5);
    // Three requests leave together: that is the point of the beat.
    let fired = wired + seconds(1.1);
    let a = stage.send(&mut scene, "ask-db", fired, 0.7);
    let b = stage.send(&mut scene, "ask-logger", fired, 0.7);
    let c = stage.send(&mut scene, "ask-clock", fired, 0.7);
    let landed = a.max(b).max(c);
    stage.land(&mut scene, "program", landed);
    stage.jolt(&mut scene, landed, [0.0, -1.0], 0.5);

    write_plan(scene, "target/effect-ts-diagram.json")?;
    println!("diagram film {:.1}s", duration as f64 / 1e9);
    Ok(())
}

/// A subtitle in the lower third, wrapped to fit the frame, shown for its line.
fn subtitle(scene: &mut PlanBuilder, id: &str, text: &'static str, at: u64, for_nanos: u64) {
    let mut plan = CaptionPlan::line(
        [960.0, 980.0],
        28.0,
        vec![CaptionSpanPlan::new("", Tone::Muted)],
    )
    .aligned(CaptionAlign::Center);
    plan.lines = wrap(text, 62)
        .into_iter()
        .map(|line| vec![CaptionSpanPlan::new(line, Tone::Muted)])
        .collect();
    match CaptionActor::declare(scene, id, &plan) {
        Ok(mut caption) => {
            caption.show(scene, at);
            caption.hide(scene, at + for_nanos);
        }
        Err(error) => eprintln!("caption {id} rejected: {error}"),
    }
}

/// Break `text` into lines of at most `width` characters on word boundaries.
fn wrap(text: &'static str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn write_plan(scene: PlanBuilder, path: &str) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(&scene.finish()?)?)?;
    Ok(())
}
