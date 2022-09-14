use devotee::visual::canvas::Canvas;
use devotee::visual::prelude::*;
use devotee::visual::sprite::Sprite;

#[test]
fn canvas_lines_are_sane() {
    let mut canvas = Canvas::with_resolution(false, 8, 8);
    canvas.line((0, 0), (7, 7), paint(true));
    for i in 0..=7 {
        assert_eq!(canvas.pixel((i, i)).cloned(), Some(true));
    }

    let mut canvas = Canvas::with_resolution(false, 8, 8);
    canvas.line((7, 7), (0, 0), paint(true));
    for i in 0..=7 {
        assert_eq!(canvas.pixel((i, i)).cloned(), Some(true));
    }

    let mut canvas = Canvas::with_resolution(false, 8, 8);
    canvas.line((0, 0), (0, 7), paint(true));
    for i in 0..=7 {
        assert_eq!(canvas.pixel((0, i)).cloned(), Some(true));
    }

    let mut canvas = Canvas::with_resolution(false, 8, 8);
    canvas.line((0, 7), (0, 0), paint(true));
    for i in 0..=7 {
        assert_eq!(canvas.pixel((0, i)).cloned(), Some(true));
    }
}

#[test]
fn sprite_lines_are_sane() {
    let mut sprite = Sprite::<_, 8, 8>::with_color(false);
    sprite.line((0, 0), (7, 7), paint(true));
    for i in 0..=7 {
        assert_eq!(sprite.pixel((i, i)).cloned(), Some(true));
    }

    let mut sprite = Sprite::<_, 8, 8>::with_color(false);
    sprite.line((7, 7), (0, 0), paint(true));
    for i in 0..=7 {
        assert_eq!(sprite.pixel((i, i)).cloned(), Some(true));
    }

    let mut sprite = Sprite::<_, 8, 8>::with_color(false);
    sprite.line((0, 0), (0, 7), paint(true));
    for i in 0..=7 {
        assert_eq!(sprite.pixel((0, i)).cloned(), Some(true));
    }

    let mut sprite = Sprite::<_, 8, 8>::with_color(false);
    sprite.line((0, 7), (0, 0), paint(true));
    for i in 0..=7 {
        assert_eq!(sprite.pixel((0, i)).cloned(), Some(true));
    }
}

#[test]
fn canvas_rectangles_are_sane() {
    let mut canvas = Canvas::with_resolution(false, 16, 8);
    canvas.filled_rect((1, 1), (15, 7), paint(true));
    for x in 0..16 {
        for y in 0..8 {
            let expected = x == 0 || x == 15 || y == 0 || y == 7;
            assert_eq!(canvas.pixel((x, y)).cloned(), Some(!expected),);
        }
    }

    let mut canvas = Canvas::with_resolution(false, 16, 8);
    canvas.rect((1, 1), (15, 7), paint(true));
    for x in 0..16 {
        for y in 0..8 {
            let top = y == 1 && (1..15).contains(&x);
            let bottom = y == 6 && (1..15).contains(&x);
            let left = x == 1 && (1..7).contains(&y);
            let right = x == 14 && (1..7).contains(&y);
            let expected = top || bottom || left || right;
            assert_eq!(canvas.pixel((x, y)).cloned(), Some(expected),);
        }
    }
}

#[test]
fn sprite_rectangles_are_sane() {
    let mut sprite = Sprite::<_, 16, 8>::with_color(false);
    sprite.filled_rect((1, 1), (15, 7), paint(true));
    for x in 0..16 {
        for y in 0..8 {
            let expected = x == 0 || x == 15 || y == 0 || y == 7;
            assert_eq!(sprite.pixel((x, y)).cloned(), Some(!expected),);
        }
    }

    let mut sprite = Sprite::<_, 16, 8>::with_color(false);
    sprite.rect((1, 1), (15, 7), paint(true));
    for x in 0..16 {
        for y in 0..8 {
            let top = y == 1 && (1..15).contains(&x);
            let bottom = y == 6 && (1..15).contains(&x);
            let left = x == 1 && (1..7).contains(&y);
            let right = x == 14 && (1..7).contains(&y);
            let expected = top || bottom || left || right;
            assert_eq!(sprite.pixel((x, y)).cloned(), Some(expected),);
        }
    }
}
