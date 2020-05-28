use nannou::draw::Draw;
use nannou::prelude::*;
use nannou::text::Font;

fn circle_pt(angle: f32) -> Point2 {
    pt2(angle.cos(), angle.sin())
}

fn poly_pts(n: i32, angle: f32) -> impl Iterator<Item = Point2> {
    (0..n).map(move |i| circle_pt(i as f32 / n as f32 * TAU + angle))
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
        .radius(r * 0.83)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0);
    draw.ellipse()
        .radius(r * 0.85)
        .no_fill()
        .stroke(WHITE)
        .stroke_weight(1.0);

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
        .points(tri1.iter().map(|p| *p));

    // Triangle point to center lines
    tri0.iter().chain(tri1.iter()).for_each(|p| {
        draw.line().color(WHITE).start(*p).end(pt2(0.0, 0.0));
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
