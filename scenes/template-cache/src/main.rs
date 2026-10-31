//! Template: a system-design worked example, built on the research rules.
//!
//! One claim per beat, a diagram that builds in place, packets that carry the
//! request, subgoal labels instead of sentences, one prediction pause, and an
//! unfinished step at the end.

use std::{fs, path::PathBuf};

use anyhow::Result;
use psychopomp::{
    author::{PlanBuilder, seconds},
    caption::{CaptionActor, CaptionAlign, CaptionPlan, CaptionSpanPlan},
    stage::{StageActor, StageElement},
    tone::Tone,
};
use psychopomp_media::{Media, Voice};

const HIT: &str = "A read goes to the cache first. On a hit, the cache answers and the database \
                   is never touched.";
const MISS: &str = "On a miss, the cache asks the database, and the answer travels back through \
                    the cache before the client sees it.";
const FILL: &str = "The fill step is yours. Write the value back, then decide the expiry, because \
                    that decision sets how stale a second reader can be.";

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let media = Media::open(&root)?;
    let voice = Voice::pocket("alba");
    let hit = media.say("hit", &voice, HIT)?;
    let miss = media.say("miss", &voice, MISS)?;
    let fill = media.say("fill", &voice, FILL)?;
    media.finish()?;
    println!("{}", psychopomp_media::show(&root)?);

    let start = seconds(0.5);
    let at_miss = start + hit.duration() + seconds(0.6);
    let at_pause = at_miss + miss.duration() + seconds(0.6);
    let pause = seconds(4.0);
    let at_fill = at_pause + pause + seconds(0.4);
    let duration = at_fill + fill.duration() + seconds(1.2);

    let mut scene = PlanBuilder::new("cache-read", duration);
    hit.place(&mut scene, start);
    miss.place(&mut scene, at_miss);
    fill.place(&mut scene, at_fill);

    let elements = vec![
        StageElement::card("client", [230.0, 640.0, 0.0], [300.0, 110.0], "Client"),
        StageElement::card("cache", [960.0, 640.0, 0.0], [300.0, 110.0], "Cache"),
        StageElement::card("db", [1690.0, 640.0, 0.0], [300.0, 110.0], "Database"),
        StageElement::beam("ask", "client", "cache"),
        StageElement::beam("answer", "cache", "client"),
        StageElement::beam("lookup", "cache", "db"),
        StageElement::beam("fill", "db", "cache"),
        StageElement::packet("p-ask", "ask").labeled("read"),
        StageElement::packet("p-answer", "answer").labeled("value"),
        StageElement::packet("p-lookup", "lookup").labeled("miss"),
        StageElement::packet("p-fill", "fill").labeled("fill"),
        // Subgoal labels: five words or fewer, one per step.
        StageElement::label("l-1", [230.0, 520.0, 10.0], 30.0, &[("1  resolve key", Tone::Muted)]),
        StageElement::label("l-2", [960.0, 520.0, 10.0], 30.0, &[("2  hit", Tone::Success)]),
        StageElement::label("l-3", [1690.0, 520.0, 10.0], 30.0, &[("3  miss", Tone::Warning)]),
        StageElement::label("l-4", [960.0, 800.0, 10.0], 30.0, &[("4  fill + expiry", Tone::Accent)]),
    ];
    let mut stage = StageActor::declare(&mut scene, "stage", &psychopomp::stage::StagePlan {
        post: Default::default(),
        elements,
    })?;

    for id in ["client", "cache", "db", "ask", "answer", "lookup", "fill", "l-1"] {
        stage.channel(&mut scene, &format!("{id}.opacity"), 0.0);
        stage.glide(&mut scene, &format!("{id}.opacity"), start, 1.0, 0.5);
    }
    for id in ["l-2", "l-3", "l-4", "p-ask", "p-answer", "p-lookup", "p-fill"] {
        stage.channel(&mut scene, &format!("{id}.opacity"), 0.0);
    }

    // Beat one: the hit path. Ask, answer, and one label per subgoal.
    let ask = start + seconds(1.6);
    stage.send(&mut scene, "p-ask", ask, 0.8);
    let answered = stage.send(&mut scene, "p-answer", ask + seconds(1.0), 0.8);
    stage.land(&mut scene, "cache", answered);
    stage.glide(&mut scene, "l-2.opacity", answered, 1.0, 0.3);

    // Beat two: the miss path, same diagram, one more edge used.
    let lookup = at_miss + seconds(1.8);
    stage.glide(&mut scene, "p-lookup.opacity", lookup - seconds(0.2), 1.0, 0.2);
    stage.send(&mut scene, "p-lookup", lookup, 0.9);
    stage.glide(&mut scene, "l-3.opacity", lookup + seconds(0.6), 1.0, 0.3);

    // Prediction pause: the frame holds. The viewer answers before the fill beat.
    let question = CaptionPlan::line(
        [960.0, 980.0],
        34.0,
        vec![CaptionSpanPlan::new("What does a second reader see?", Tone::Plain)],
    )
    .aligned(CaptionAlign::Center);
    if let Ok(mut caption) = CaptionActor::declare(&mut scene, "ask-second", &question) {
        caption.show(&mut scene, at_pause);
        caption.hide(&mut scene, at_fill - seconds(0.3));
    }

    // Beat three: the fill, and the unfinished decision.
    stage.glide(&mut scene, "p-fill.opacity", at_fill - seconds(0.2), 1.0, 0.2);
    let filled = stage.send(&mut scene, "p-fill", at_fill + seconds(1.4), 0.9);
    stage.land(&mut scene, "cache", filled);
    stage.glide(&mut scene, "l-4.opacity", filled, 1.0, 0.35);
    stage.jolt(&mut scene, filled, [0.0, -1.0], 0.4);

    fs::create_dir_all("target")?;
    fs::write(
        "target/cache-read.json",
        serde_json::to_string_pretty(&scene.finish()?)?,
    )?;
    println!("duration {:.1} s", duration as f64 / 1e9);
    Ok(())
}
