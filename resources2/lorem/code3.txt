neuro_setup(() => {
    neuro_source('in');
    let canvas = createCanvas(800, 800);
});

function virtual(color, t) {
    const [t0, t1, t2, t3] = [t * 0.01, t * 0.04, t * 0.1, t * 0.5];

    const [hue, sat, val] = hsv(...color);
    strokeCap(PROJECT);

    const outline = w => (noFill(), stroke(...color), strokeWeight(w));
    const full = () => (noStroke(), fill(...color));

    const circ = n => range(0, 2*PI, 2*PI / n);
    const poly = (n, r) => circ(n).amap(t => rectangular(r, t));
    const polyshift = (n, r, shift = PI) => circ(n).amap(t => rectangular(r, t + shift));
    const polyf = (n, r, f) => circ(n).amap(t => rectangular(r, f(t)));

    // Outer Circles
    outline(2);
    arc(0, 0, 630, 630, t0, 1 + t0);
    arc(0, 0, 630, 630, 1.5 + t0, 2 + t0);
    arc(0, 0, 630, 630, 2.5 + t0, 3.8 + t0);
    arc(0, 0, 630, 630, 4.3 + t0, 4.6 + t0);
    arc(0, 0, 630, 630, 5.1 + t0, 5.8 + t0);
    circle(0, 0, 580);
    circle(0, 0, 460);

    // Radial lines
    const rad0 = poly(70, 240);
    const rad1 = poly(70, 280);
    const ts = range(0, 1, 1 / 70);

    outline(2);
    const radial = ninterp([0, 0], [0, 0.1], [val, 0.25], [0, 0.5], [0, 0.6], [val, 0.75], [0, 1]);
    zip(rad0, rad1, ts).each(([[x0, y0], [x1, y1], t]) => {
        stroke(...rgb(hue, sat, radial(t + t0)));
        line(x0, y0, x1, y1);
    });

    // Triangle Lines
    outline(4);
    polyf(3, 230, t => t - t1 + PI).each(([x, y]) =>
        line(x, y, 0, 0));

    // Triangle
    outline(2);
    fill(0);
    const tri = polyf(3, 300, t => t - t1);
    triangle(...tri.flat());

    outline(0.5);
    triangle(...polyf(3, 150, t => t - t1 + PI).flat());

    outline(2);
    fill(0);

    // Triangle Points
    tri.each(([x, y], i) => {
        circle(x, y, 100);
    });

    outline(2);
    fill(...color);

    {
        // Triangle Pattern 0
        const [x, y] = tri[0];
        const ps = [
            [x + 25, y + 8],
            [x + 5, y - 5],
            [x - 14, y + 20],
            [x - 20, y - 15]
        ];

        ps.map(p => circle(...p, 8));
        line(...ps[0], ...ps[1]);
        line(...ps[1], ...ps[2]);
        line(...ps[1], ...ps[3]);
    }

    {
        // Triangle Pattern 1
        const [x, y] = tri[1];
        const ps = [
            [x + 5, y - 10],
            [x + 5, y + 20],
            [x - 11, y + 20],
            [x - 11, y - 20],
        ];

        ps.map(p => circle(...p, 8));
        line(...ps[0], ...ps[1]);
        line(...ps[1], ...ps[2]);
        line(...ps[1], ...ps[3]);
    }

    {
        // Triangle Pattern 2
        const [x, y] = tri[2];
        const ps = [
            [x + 25, y],
            [x, y],
            [x, y + 20],
            [x - 20, y - 15]
        ];

        ps.map(p => circle(...p, 8));
        line(...ps[0], ...ps[1]);
        line(...ps[1], ...ps[2]);
        line(...ps[1], ...ps[3]);
    }

    fill(0);

    // Triangle Line Circles
    polyf(3, 230, t => t - t1 + PI).each(([x, y]) =>
        circle(x, y, 20));

    // Center Circles
    outline(2);
    circ(12).zip_next().each(([a0, a1], i) => even(i) && arc(0, 0, 220, 220, a0 + t1, a1 + t1));
    circ(12).zip_next().each(([a0, a1], i) => odd(i) && arc(0, 0, 210, 210, a0 + t1, a1 + t1));
    zip(poly(50, 85), poly(50, 70)).each(([[x0, y0], [x1, y1]]) => {
        line(x0, y0, x1, y1);
    });

    outline(12);
    circ(16).zip_next().each(([a0, a1], i) => even(i) && arc(0, 0, 100, 100, a0 + t1, a1 + t1));

    // Floaty circle
    outline(1);
    const [fx, fy] = [200 * cos(t3 + 0.2), 200 * sin(t3)];
    fill(...color);
    circle(fx, fy, 40);
    fill(0);
    circle(fx, fy, 30);
    fill(...color);
    circle(fx, fy, 18);
}

neuro_draw(() => {
    const t = neuro_get('t', 0);
    background(0);

    translate(400, 400);
    virtual([0, 192, 255], t);

    neuro_set('t', t + amp.getLevel());
});
