//! A two-minute argument for Effect: why its error, dependency, and concurrency
//! model is worth the learning curve. Narration is local (Pocket TTS); every beat
//! is a title and a short body line that settles in when its sentence starts.
//!
//! Run with `pocket-tts serve` up, then render `target/effect-ts.json`.

use std::{fs, path::PathBuf};

use anyhow::Result;
use psychopomp::{
    author::{PlanBuilder, seconds},
    face::Face,
    stage::{StageActor, StageElement},
    tone::Tone,
};
use psychopomp_media::{Media, Voice};

const GAP: u64 = 700_000_000;

struct Beat {
    id: &'static str,
    title: &'static str,
    body: &'static str,
    line: &'static str,
}

const BEATS: &[Beat] = &[
    Beat {
        id: "intro",
        title: "Why Effect?",
        body: "Your program is a description of work, not a pile of side effects.",
        line: "Effect is a TypeScript library that treats a program as a description of work, \
               not a pile of side effects. Here is why that is worth a learning curve.",
    },
    Beat {
        id: "hidden",
        title: "Plain TypeScript hides three things",
        body: "Thrown errors. Imported dependencies. Hand-rolled retries.",
        line: "Plain TypeScript hides three things. Errors are thrown, so the type system never \
               sees them. Dependencies arrive through imports, so a signature never mentions them. \
               And retries, timeouts, and cancellation get reinvented in every project.",
    },
    Beat {
        id: "throws",
        title: "A function that throws tells you nothing",
        body: "The caller must know which errors exist. The compiler cannot check they were handled.",
        line: "A function that throws tells you nothing. The caller has to know which errors can \
               appear, and the compiler cannot check that anyone handled them.",
    },
    Beat {
        id: "typed",
        title: "Errors become part of the signature",
        body: "Success and failure travel together. An unhandled failure is a compile error.",
        line: "Effect makes errors part of the return type. Success and failure sit in the same \
               signature, so an unhandled failure becomes a compile error rather than a \
               production incident.",
    },
    Beat {
        id: "layers",
        title: "Dependencies become a value",
        body: "A Layer declares what a program needs. You supply it at the edge.",
        line: "Dependencies become a value. A layer describes what a program needs, and you supply \
               it at the edge, which makes tests cheap and mocks honest.",
    },
    Beat {
        id: "builtin",
        title: "Concurrency comes with it",
        body: "Retries, timeouts, cancellation, and parallel work compose as values.",
        line: "Concurrency, retries, timeouts, and cancellation are built in, and they compose, \
               because they are values too, not control flow written by hand.",
    },
    Beat {
        id: "suite",
        title: "One model for the whole toolkit",
        body: "Schema for untrusted input. Stream for events. Schedule for policy.",
        line: "Effect ships a schema library for parsing untrusted data, a stream library, and a \
               scheduler. All of it shares the same error and dependency model.",
    },
    Beat {
        id: "cost",
        title: "The cost is real",
        body: "It is a large abstraction, and the error messages take practice.",
        line: "The cost is real. Effect is a large abstraction and the error messages take \
               practice. Start with one boundary, a single endpoint or one background job, and \
               expand from there.",
    },
    Beat {
        id: "when",
        title: "Use it where correctness pays",
        body: "Not in a prototype. Not in a script that runs once.",
        line: "Use it where correctness pays for itself. Not in a prototype, and not in a script \
               that runs once.",
    },
    Beat {
        id: "close",
        title: "Start at one boundary",
        body: "Measure the pain it removes. Let the rest follow.",
        line: "That is the case for Effect. Start at one boundary, measure the pain it removes, \
               and let the rest follow.",
    },
];

/// Ids are `&str` in the plan; a scene binary runs once, so leaking is fine.
fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let media = Media::open(&root)?;
    let voice = Voice::pocket("alba");

    let mut lines = Vec::new();
    for beat in BEATS {
        lines.push(media.say(beat.id, &voice, beat.line)?);
    }
    media.finish()?;
    println!("{}", psychopomp_media::show(&root)?);

    let mut at = seconds(0.4);
    let mut starts = Vec::new();
    for line in &lines {
        starts.push(at);
        at += line.duration() + GAP;
    }
    let duration = at;

    let mut scene = PlanBuilder::new("effect-ts", duration);
    let mut elements = Vec::new();
    for beat in BEATS {
        elements.push(
            StageElement::label(
                leak(format!("{}-title", beat.id)),
                [960.0, 380.0, 40.0],
                68.0,
                &[(beat.title, Tone::Plain)],
            )
            .face(Face::SansBold),
        );
        elements.push(
            StageElement::label(
                leak(format!("{}-body", beat.id)),
                [960.0, 560.0, 40.0],
                36.0,
                &[(beat.body, Tone::Muted)],
            )
            .face(Face::Light),
        );
    }
    elements.push(
        StageElement::label(
            "mark",
            [960.0, 900.0, 20.0],
            26.0,
            &[("Effect · TypeScript", Tone::Muted)],
        )
        .face(Face::SansBold),
    );

    let mut stage = StageActor::declare(&mut scene, "stage", &psychopomp::stage::StagePlan {
        post: Default::default(),
        elements,
    })?;

    // Labels carry an `opacity` channel. Start every beat invisible, fade it in
    // when its sentence starts, and fade it out as the next one begins.
    for beat in BEATS {
        stage.channel(&mut scene, leak(format!("{}-title.opacity", beat.id)), 0.0);
        stage.channel(&mut scene, leak(format!("{}-body.opacity", beat.id)), 0.0);
    }
    stage.channel(&mut scene, "mark.opacity", 0.0);
    stage.glide(&mut scene, "mark.opacity", 0, 1.0, 0.6);

    for (index, (beat, (line, start))) in BEATS.iter().zip(lines.iter().zip(&starts)).enumerate() {
        line.place(&mut scene, *start);
        stage.glide(&mut scene, leak(format!("{}-title.opacity", beat.id)), *start, 1.0, 0.45);
        stage.glide(&mut scene, leak(format!("{}-body.opacity", beat.id)), *start, 1.0, 0.45);
        if let Some(next) = starts.get(index + 1) {
            let out = next.saturating_sub(seconds(0.35));
            stage.glide(&mut scene, leak(format!("{}-title.opacity", beat.id)), out, 0.0, 0.3);
            stage.glide(&mut scene, leak(format!("{}-body.opacity", beat.id)), out, 0.0, 0.3);
        }
    }

    fs::create_dir_all("target")?;
    fs::write(
        "target/effect-ts.json",
        serde_json::to_string_pretty(&scene.finish()?)?,
    )?;
    println!("duration {:.1} s", duration as f64 / 1e9);
    Ok(())
}
