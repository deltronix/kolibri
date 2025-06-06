use core::convert::Infallible;
use core::ops::Sub;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

pub struct WidgetFramebuf<'a, C: PixelColor> {
    buf: &'a mut [C],
    size: Size,
    position: Point,
    len: usize,
}

impl<'a, C: PixelColor> WidgetFramebuf<'a, C> {
    pub fn new(buf: &'a mut [C], size: Size, position: Point) -> Self {
        let len = size.width as usize * size.height as usize;
        assert!(len <= buf.len(), "buf too small for framebuffer");
        Self {
            buf,
            size,
            position,
            len,
        }
    }

    pub fn try_new(buf: &'a mut [C], size: Size, position: Point) -> Option<Self> {
        let len = size.width as usize * size.height as usize;
        if len <= buf.len() {
            Some(Self {
                buf,
                size,
                position,
                len,
            })
        } else {
            None
        }
    }

    pub fn get_pos(&self) -> Point {
        self.position
    }

    pub fn get_size(&self) -> Size {
        self.size
    }
}

impl<C: PixelColor> Dimensions for WidgetFramebuf<'_, C> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.position, self.size)
    }
}

impl<C: PixelColor> DrawTarget for WidgetFramebuf<'_, C> {
    type Color = C;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let pt = pixel.0.sub(self.position);
            let pos = pt.y * self.size.width as i32 + pt.x;
            if pos < 0 || pos >= self.len as i32
            /* check for trunc maybe? */
            {
                // !! Make sure that len is correct in new() !!
                // skip pixels outside of the framebuffer
                continue;
            }
            self.buf[pos as usize] = pixel.1;
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());
        if drawable_area.is_zero_sized() {
            return Ok(());
        }

        let top_skip = drawable_area.top_left.y - area.top_left.y;
        let left_skip = drawable_area.top_left.x - area.top_left.x;
        let right_skip = area.size.width as i32 - (left_skip + drawable_area.size.width as i32);
        // let bottom_skip = area.size.height as i32 - (top_skip + drawable_area.size.height as i32);

        let mut color_iter = colors.into_iter();

        // skip all un-drawable rows
        for _ in 0..top_skip {
            for _ in 0..area.size.width as usize {
                color_iter.next();
            }
        }
        for y in drawable_area.top_left.y as usize
            ..drawable_area.top_left.y as usize + drawable_area.size.height as usize
        {
            for _ in 0..left_skip {
                color_iter.next();
                // skip all left
            }
            for x in drawable_area.top_left.x as usize
                ..drawable_area.top_left.x as usize + drawable_area.size.width as usize
            {
                let pos = (y as i32 - self.position.y) as usize * self.size.width as usize
                    + (x as i32 - self.position.x) as usize;
                match color_iter.next() {
                    Some(color) => self.buf[pos] = color,
                    None => return Ok(()),
                }
            }
            for _ in 0..right_skip {
                color_iter.next();
                // skip all right
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        // Clamp area to drawable part of the display target
        let drawable_area = area.intersection(&self.bounding_box());

        // Draw the rectangle
        for y in drawable_area.top_left.y as usize
            ..drawable_area.top_left.y as usize + drawable_area.size.height as usize
        {
            for x in drawable_area.top_left.x as usize
                ..drawable_area.top_left.x as usize + drawable_area.size.width as usize
            {
                let pos = (y as i32 - self.position.y) as usize * self.size.width as usize
                    + (x as i32 - self.position.x) as usize;
                self.buf[pos] = color;
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.buf[0..(self.size.width * self.size.height) as usize].fill(color);
        Ok(())
    }
}

impl<C: PixelColor> Drawable for WidgetFramebuf<'_, C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.fill_contiguous(
            &Rectangle::new(self.position, self.size),
            self.buf.iter().cloned(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::pixelcolor::*;
    use embedded_graphics::primitives::*;
    use embedded_graphics::primitives::{Circle, PrimitiveStyle, StyledDrawable};

    #[test]
    fn test_basic_fbuf() {
        const SIZE: usize = 8;

        let mut data = [BinaryColor::Off; SIZE * SIZE];
        let mut fbuf = WidgetFramebuf::new(
            &mut data,
            Size::new(SIZE as u32, SIZE as u32),
            Point::new(0, 0),
        );

        let color = BinaryColor::On;

        let circ = Circle::new(Point::zero(), 8);

        let mut expected = MockDisplay::new();
        expected.set_allow_overdraw(true);
        // clear area
        Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(&mut expected)
            .unwrap();

        circ.draw_styled(&PrimitiveStyle::with_fill(color), &mut expected)
            .unwrap();

        let mut actual = MockDisplay::new();
        circ.draw_styled(&PrimitiveStyle::with_fill(color), &mut fbuf)
            .unwrap();
        actual
            .fill_contiguous(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                data,
            )
            .unwrap();

        actual.assert_eq(&expected)
    }

    #[test]
    #[should_panic(expected = "buf too small for framebuffer")]
    fn crash_at_new_with_too_large_size() {
        let mut data = [BinaryColor::Off; 9]; // enugh for 3*3

        // crashes
        let mut _fbuf = WidgetFramebuf::new(&mut data, Size::new(3, 5), Point::new(0, 0));
    }

    #[test]
    fn test_widget_framebuf_new() {
        let mut buf = [Rgb888::BLACK; 9];
        let framebuf = WidgetFramebuf::new(&mut buf, Size::new(3, 3), Point::new(0, 0));

        assert_eq!(framebuf.size.width, 3);
        assert_eq!(framebuf.size.height, 3);
        assert_eq!(framebuf.len, 9);
    }

    #[test]
    fn test_widget_framebuf_try_new() {
        let mut buf = [Rgb888::BLACK; 9];
        let framebuf = WidgetFramebuf::try_new(&mut buf, Size::new(3, 3), Point::zero());

        assert!(framebuf.is_some());
        let framebuf = framebuf.unwrap();
        assert_eq!(framebuf.size.width, 3);
        assert_eq!(framebuf.size.height, 3);
        assert_eq!(framebuf.len, 9);
    }

    #[test]
    fn test_widget_framebuf_try_new_fail() {
        let mut buf = [Rgb888::BLACK; 8];
        let framebuf = WidgetFramebuf::try_new(&mut buf, Size::new(3, 3), Point::new(0, 0));

        assert!(framebuf.is_none());
    }

    #[test]
    fn test_widget_framebuf_draw_line() {
        let mut buf = [Rgb888::BLACK; 9];
        let mut framebuf = WidgetFramebuf::new(&mut buf, Size::new(3, 3), Point::new(0, 0));

        let line = Line::new(Point::new(0, 0), Point::new(2, 2));
        let styled_line = line.into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1));

        styled_line.draw(&mut framebuf).unwrap();

        assert_eq!(buf[0], Rgb888::RED);
        assert_eq!(buf[4], Rgb888::RED);
        assert_eq!(buf[8], Rgb888::RED);
    }

    #[test]
    fn test_fill_contiguous() {
        const SIZE: usize = 8;
        let mut data = [BinaryColor::Off; SIZE * SIZE];
        let mut fbuf = WidgetFramebuf::new(
            &mut data,
            Size::new(SIZE as u32, SIZE as u32),
            Point::new(0, 0),
        );

        let colors = [
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
        ];
        let area = Rectangle::new(Point::new(2, 2), Size::new(4, 4));

        fbuf.fill_contiguous(&area, colors.iter().cloned()).unwrap();

        let mut expected = MockDisplay::new();
        expected.set_allow_overdraw(true);
        expected
            .fill_solid(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                BinaryColor::Off,
            )
            .unwrap();
        expected
            .fill_solid(
                &Rectangle::new(Point::new(2, 2), Size::new(4, 1)),
                BinaryColor::On,
            )
            .unwrap();
        expected
            .fill_solid(
                &Rectangle::new(Point::new(2, 3), Size::new(4, 1)),
                BinaryColor::Off,
            )
            .unwrap();
        expected
            .fill_solid(
                &Rectangle::new(Point::new(2, 4), Size::new(4, 1)),
                BinaryColor::On,
            )
            .unwrap();
        expected
            .fill_solid(
                &Rectangle::new(Point::new(2, 5), Size::new(4, 1)),
                BinaryColor::Off,
            )
            .unwrap();

        let mut actual = MockDisplay::new();
        actual
            .fill_contiguous(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                data,
            )
            .unwrap();

        actual.assert_eq(&expected);
    }

    #[test]
    fn test_fill_contiguous_clipped() {
        let mut data = [BinaryColor::Off; 9];
        let mut fbuf = WidgetFramebuf::new(&mut data, Size::new(3, 3), Point::new(0, 0));

        let colors = [
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
        ];
        let area = Rectangle::new(Point::new(-1, -1), Size::new(5, 5));

        fbuf.fill_contiguous(&area, colors.iter().cloned()).unwrap();

        let expected = [
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::Off,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_fill_contiguous_clipped_2() {
        let mut data = [BinaryColor::Off; 12];
        let mut fbuf = WidgetFramebuf::new(&mut data, Size::new(4, 3), Point::new(2, 1));

        let colors = [
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::On,
        ];
        let area = Rectangle::new(Point::new(1, 0), Size::new(6, 3));

        fbuf.fill_contiguous(&area, colors.iter().cloned()).unwrap();

        let expected = [
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_fill_contiguous_short_iterator() {
        let mut data = [BinaryColor::Off; 9];
        let mut fbuf = WidgetFramebuf::new(&mut data, Size::new(3, 3), Point::new(0, 0));

        let colors = [BinaryColor::On; 5];
        let area = Rectangle::new(Point::new(0, 0), Size::new(3, 3));

        fbuf.fill_contiguous(&area, colors.iter().cloned()).unwrap();

        let expected = [
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_fill_solid() {
        const SIZE: usize = 8;
        let mut data = [BinaryColor::Off; SIZE * SIZE];
        let mut fbuf = WidgetFramebuf::new(
            &mut data,
            Size::new(SIZE as u32, SIZE as u32),
            Point::new(0, 0),
        );

        let area = Rectangle::new(Point::new(1, 1), Size::new(6, 6));
        fbuf.fill_solid(&area, BinaryColor::On).unwrap();

        let mut expected = MockDisplay::new();
        expected.set_allow_overdraw(true);
        expected
            .fill_solid(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                BinaryColor::Off,
            )
            .unwrap();
        expected.fill_solid(&area, BinaryColor::On).unwrap();

        let mut actual = MockDisplay::new();
        actual
            .fill_contiguous(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                data,
            )
            .unwrap();

        actual.assert_eq(&expected);
    }

    #[test]
    fn test_fill_solid_clipped() {
        let mut data = [BinaryColor::Off; 9];
        let mut fbuf = WidgetFramebuf::new(&mut data, Size::new(3, 3), Point::new(0, 0));

        let area = Rectangle::new(Point::new(-1, -1), Size::new(5, 5));

        fbuf.fill_solid(&area, BinaryColor::On).unwrap();

        let expected = [
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::On,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_fill_solid_clipped_2() {
        let mut data = [BinaryColor::Off; 12];
        let mut fbuf = WidgetFramebuf::new(&mut data, Size::new(4, 3), Point::new(2, 1));

        let area = Rectangle::new(Point::new(1, 0), Size::new(3, 3));

        fbuf.fill_solid(&area, BinaryColor::On).unwrap();

        let expected = [
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::On,
            BinaryColor::On,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
            BinaryColor::Off,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_clear() {
        const SIZE: usize = 8;
        let mut data = [BinaryColor::On; SIZE * SIZE];
        let mut fbuf = WidgetFramebuf::new(
            &mut data,
            Size::new(SIZE as u32, SIZE as u32),
            Point::new(0, 0),
        );

        fbuf.clear(BinaryColor::Off).unwrap();

        let mut expected = MockDisplay::new();
        expected
            .fill_solid(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                BinaryColor::Off,
            )
            .unwrap();

        let mut actual = MockDisplay::new();
        actual
            .fill_contiguous(
                &Rectangle::new(Point::zero(), Size::new(SIZE as u32, SIZE as u32)),
                data,
            )
            .unwrap();

        actual.assert_eq(&expected);
    }
}
