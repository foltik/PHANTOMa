use nannou::draw::Draw;
use nannou::geom::Range;
use nannou::prelude::*;
use nannou::text::Font;

fn circle_pt(angle: f32) -> Point2 {
    pt2(angle.cos(), angle.sin())
}

fn poly_pts(n: i32, angle: f32) -> impl Iterator<Item = Point2> {
    (0..n).map(move |i| circle_pt(i as f32 / n as f32 * TAU + angle))
}

fn rand(seed: f32) -> f32 {
    let p = pt2(seed + 10.0, seed + 3.0);
    let dt = p.perp_dot(pt2(12.9898, 78.233));
    let sn = dt % 3.14;
    2.0 * (sn.sin() * 43758.5453).fract() - 1.0
}

pub fn rnames(draw: &Draw, font: Font, f: f32, t: f32) {
    let res = pt2(1920.0, 1080.0) / 2.0;

    let n = 4.0;
    for (j, txt) in ["THOMAS LEGACY", "FOLTIK"].iter().enumerate() {
        for i in 0..n as u32 {
            let (i, j) = (i as f32, j as f32);
            let pos = pt2(
                rand((i + 5.0 + j * 10.0) * (t * Range::new(5.0, 12.0).lerp(f)).floor()),
                rand((i + 5.0 + j * 100.0) * (t * Range::new(5.0, 12.0).lerp(f)).floor() + 1.0),
            ) * res
                * 0.95;
            draw.text(txt)
                .font(font.clone())
                .xy(pos)
                .width(1000.0)
                .font_size(48)
                .color(Rgba::new(
                    1.0,
                    1.0,
                    1.0,
                    Range::new(0.02 * f, 0.0).lerp(i / n),
                ));
        }
    }
}

pub fn fnames(draw: &Draw, font: Font, f: f32, t: f32) {
    let res = pt2(1920.0, 1080.0) / 2.0;

    let n = 4.0;
    for (j, txt) in ["THOMAS LEGACY", "FOLTIK"].iter().enumerate() {
        for i in 0..n as u32 {
            for k in 0..3 {
                let (i, j, k) = (i as f32, j as f32, k as f32);

                let tt = t * Range::new(0.2, 0.8).lerp(f);
                let ti = tt.floor() + k;
                let tfr = tt.fract() - rand((i + 5.0 + j * 20.0) * (ti + 5.0));

                let pos = pt2(tfr * 2.0, rand((i + 5.0 + j * 20.0) * (ti + 10.0))) * res - (res * 0.5);
                draw.text(txt)
                    .font(font.clone())
                    .xy(pos)
                    .width(1000.0)
                    .font_size(48)
                    .color(Rgba::new(
                        1.0,
                        1.0,
                        1.0,
                        Range::new(0.02 * f, 0.0).lerp(i / n),
                    ));
                }
        }
    }
}

pub fn orbit(draw: &Draw, p: &Point2, r: f32, t: f32, alpha: f32) {
    let t = t / 500.0;
    let draw = draw.translate(pt3(p.x, p.y, 0.0)).rotate(t);

    let c = Rgba::new(1.0, 1.0, 1.0, alpha);

    draw.ellipse()
        .radius(r)
        .no_fill()
        .stroke(c)
        .stroke_weight(1.0);

    let p = pt2(t.cos(), t.sin()) * r;

    draw.ellipse().radius(15.0).color(c).xy(p);

    draw.ellipse()
        .radius(18.0)
        .no_fill()
        .stroke(c)
        .stroke_weight(1.0)
        .xy(p);

    let draw = draw.translate(pt3(p.x, p.y, 0.0)).rotate(t * 15.0);

    draw.ellipse()
        .radius(60.0)
        .no_fill()
        .stroke(c)
        .stroke_weight(1.0);

    draw.ellipse().radius(5.0).color(c).x(60.0);
}

//#[rustfmt_skip]
pub fn demon1(draw: &Draw, font: Font, p: &Point2, r: f32, t: f32) {
    let draw = draw.translate(pt3(p.x, p.y, 0.0)).rotate(t / 500.0);

    let tri0: Vec<_> = poly_pts(3, TAU / 4.0).map(|p| p * r).collect();
    let tri1: Vec<_> = poly_pts(3, -TAU / 4.0).map(|p| p * r).collect();

    // Outer ring
    draw.ellipse()
        .radius(r)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Second layer double ring
    draw.ellipse()
        .radius(r * 0.86)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(3.0);

    // Outer triangles
    draw.polygon()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(tri0.iter().map(|p| *p));
    draw.polygon()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(tri1.iter().map(|p| *p * 1.2));

    // Triangle point to center lines
    tri0.iter().for_each(|p| {
        draw.line().color(WHITE).start(*p).end(pt2(0.0, 0.0));
    });
    tri1.iter().for_each(|p| {
        draw.line().color(WHITE).start(*p * 1.2).end(pt2(0.0, 0.0));
    });

    // Triangle point rune circles
    tri0.iter()
        .zip(/*"AFH".chars()*/ lib::chars(5.123))
        .for_each(|(p, ch)| {
            draw.ellipse()
                .xy(*p)
                .radius(r * 0.133)
                .color(BLACK)
                .stroke(WHITE)
                .stroke_weight(1.0);
            draw.text(ch)
                .font(font.clone())
                .font_size((r * 0.12).round() as u32)
                .xy(*p)
                .color(WHITE);
        });

    // Inner circle ring
    draw.ellipse()
        .radius(r * 0.5)
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Inner runes
    poly_pts(26, t / 500.0)
        .map(|p| p * r * 0.465)
        .zip(lib::chars(1.0))
        .enumerate()
        .for_each(|(i, (p, ch))| {
            let angle = i as f32 / 30.0 * -TAU;
            draw.text(ch)
                .font(font.clone())
                .font_size((r * 0.04).round() as u32)
                .xy(p)
                .rotate(angle)
                .color(WHITE);
        });

    // Innner circle fill
    draw.ellipse()
        .radius(r * 0.433)
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0);

    // Inner tri
    draw.polygon()
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0)
        .points(tri0.iter().map(|p| *p * 0.5));

    tri0.iter().for_each(|p| {
        // Outer fat circles
        draw.ellipse()
            .xy(*p * 0.7)
            .radius(r * 0.033)
            .color(BLACK)
            .stroke(WHITE)
            .stroke_weight(2.0);

        // Inner fat circle lines
        draw.line().start(*p / 2.0).end(pt2(0.0, 0.0)).color(WHITE);

        // Inner fat circles
        draw.ellipse()
            .xy(*p * 0.3)
            .radius(r * 0.033)
            .color(BLACK)
            .stroke(WHITE)
            .stroke_weight(4.0);
    });

    // Center outer, smaller, mid, point
    draw.ellipse()
        .color(BLACK)
        .stroke(WHITE)
        .stroke_weight(1.0)
        .radius(r * 0.2);
    draw.ellipse()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .radius(r * 0.1733);
    draw.ellipse()
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0)
        .radius(r * 0.1);
    draw.ellipse().color(WHITE).radius(r * 0.033);
}
