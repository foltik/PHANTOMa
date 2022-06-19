const bins = 64;
neuro_config({
    fft_smooth: 0.8,
    fft_bins: bins
});

neuro_setup(() => {
    neuro_source('in');
    let canvas = createCanvas(720, 720, WEBGL);
});

neuro_init(() => {
    const bpm = 250;
    onset = new OnsetDetect(40, 120, 0.015, bpm);
    decay = new Decay(1, 1.05, 1 / (bpm / 60), 50);

    neuro_set_all({onset, decay});
});

function crystal(h, r, prop) {
    const [p_top, p_bot] = [[0, -h/2, 0], [0, h/2, 0]];

    const hex = range(0, 2*PI, 2*PI / 6).amap(t => rectangular(r, t));

    hex.each(([x, z]) =>
        line(...p_top, x, -h * prop / 2, z));

    hex.zip_next().each(([[x0, z0], [x1, z1]]) =>
        line(x0, -h * prop / 2, z0, x1, -h * prop / 2, z1));

    hex.each(([x, z]) =>
        line(...p_bot, x, h * prop / 2, z));

    hex.zip_next().each(([[x0, z0], [x1, z1]]) =>
        line(x0, h * prop / 2, z0, x1, h * prop / 2, z1));

    hex.each(([x, z]) =>
        line(x, h * prop / 2, z, x, -h * prop / 2, z));
}

neuro_draw(() => {
    const [onset, decay] = neuro_get_all('onset', 'decay');
    let t = neuro_get('t', 0);

    //onset.detect() && decay.set(1.05);
    t += amp.getLevel() * decay.get() * 2;

    background(0);

    fill(0, 0, 0, 0);
    stroke(255, 255, 255, 255);
    strokeWeight(1);
    rotateZ(0.5);
    rotateY(t * 0.05);

    stroke(0, 195, 255);
    strokeWeight(3);
    crystal(600, 150 * decay.get(), lerp(0.5, 0.4)(mirror(2)(t * 0.03)));


    rotateX(degrees(90));

    stroke(255);
    strokeWeight(3);
    circle(0, 0, 500);

    stroke(0, 195, 255);
    strokeWeight(25);
    point(250 * cos(t), 250 * sin(t));

    rotateX(degrees(-180.001));

    stroke(255);
    strokeWeight(3);
    circle(0, 0, 500);

    stroke(0, 195, 255);
    strokeWeight(25);
    point(250 * cos(t * 1.1), 250 * sin(t * 1.1));

    neuro_set('t', t);
});
